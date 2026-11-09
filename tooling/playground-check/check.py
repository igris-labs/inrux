#!/usr/bin/env python3
"""Check independent playground source consistency and production engine isolation."""
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[2]
ENGINES = ("egui", "masonry", "gpui")
LOCAL_FILES = {"engine.rs", "native_tests.rs"}


def sources(engine):
    """Engine selection/native harnesses differ; portable code and font bytes must agree."""
    root = ROOT / "playground" / engine / "src"
    result = {}
    for path in sorted(root.rglob("*")):
        if not path.is_file() or path.name == ".DS_Store":
            continue
        relative = path.relative_to(root).as_posix()
        if relative in LOCAL_FILES:
            continue
        content = path.read_bytes()
        if relative == "main.rs":
            content = content.replace(b"\n#[cfg(test)]\nmod native_tests;\n", b"")
        result[relative] = content
    return result


def main():
    """Copied source stays locally editable, with reviewable drift caught in CI."""
    directories = {path.name for path in (ROOT / "playground").iterdir() if path.is_dir()}
    assert directories == set(ENGINES), ("unexpected playground directories", directories)
    reference = sources("egui")
    for engine in ENGINES:
        actual = sources(engine)
        assert actual.keys() == reference.keys(), (engine, "source file set differs")
        differences = [path for path in reference if reference[path] != actual[path]]
        assert not differences, (engine, "portable source drift", differences)
        result = subprocess.run([
            "cargo", "tree", "--locked", "--offline", "-p", f"inrux-playground-{engine}",
            "--edges", "normal,build", "--prefix", "none", "--format", "{p}",
        ], cwd=ROOT, text=True, capture_output=True, check=True)
        packages = {line.split()[0] for line in result.stdout.splitlines() if line.strip()}
        assert f"inrux-{engine}" in packages, (engine, "selected adapter missing")
        for other in set(ENGINES) - {engine}:
            assert f"inrux-{other}" not in packages, (engine, "unselected adapter", other)
        assert "inrux-playground-shared" not in packages, "retired shared runtime dependency"
        print(f"{engine}: portable source/font consistency and engine isolation passed")


if __name__ == "__main__":
    main()
