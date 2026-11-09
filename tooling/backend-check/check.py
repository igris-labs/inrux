#!/usr/bin/env python3
"""Resolve standalone consumers to detect accidental backend feature activation."""
import json
from pathlib import Path
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[2]
CASES = {
    "markdown-headless": (["markdown"], {"inrux-markdown", "pulldown-cmark"}, {"inrux-text", "parley", "swash", "taffy", "inrux-egui", "eframe", "egui", "masonry", "masonry_winit", "gpui"}),
    "headless": ([], set(), {"inrux-text", "parley", "swash", "inrux-markdown", "pulldown-cmark", "taffy", "inrux-egui", "eframe", "egui", "masonry", "masonry_winit", "inrux-devtools", "gpui"}),
    "egui": (["egui"], {"inrux-text", "inrux-egui", "eframe", "egui"}, {"masonry", "masonry_winit", "gpui"}),
    "masonry": (["masonry"], {"inrux-text", "masonry", "masonry_winit"}, {"inrux-egui", "eframe", "egui", "inrux-devtools", "gpui"}),
    "gpui": (["gpui"], {"inrux-text", "gpui", "inrux-gpui"}, {"inrux-egui", "eframe", "egui", "masonry", "masonry_winit", "inrux-devtools"}),
    "gpui-devtools": (["gpui", "devtools"], {"gpui"}, {"inrux-egui", "eframe", "egui", "masonry", "masonry_winit", "inrux-devtools"}),
    "masonry-devtools": (["masonry", "devtools"], {"masonry"}, {"inrux-egui", "eframe", "egui", "inrux-devtools", "gpui"}),
}


def main():
    """Use separate roots: Cargo workspace feature unification would hide leakage."""
    with tempfile.TemporaryDirectory(prefix="inrux-backends-") as directory:
        root = Path(directory)
        (root / "src").mkdir()
        (root / "src/main.rs").write_text("fn main() {}\n")
        for label, (features, required, forbidden) in CASES.items():
            (root / "Cargo.toml").write_text(
                '[package]\nname = "backend-isolation"\nversion = "0.0.0"\nedition = "2024"\n'
                '[dependencies]\ninrux = { path = ' + json.dumps(str(ROOT / "packages/inrux"))
                + ', default-features = false, features = ' + json.dumps(features) + ' }\n'
            )
            # GPUI pins several transitive versions; refresh this temporary consumer's lock
            # when switching features, as an existing application may need to do.
            if (root / "Cargo.lock").exists():
                update = subprocess.run(["cargo", "update", "--offline", "--manifest-path", str(root / "Cargo.toml")], capture_output=True, text=True)
                if update.returncode:
                    raise SystemExit(f"{label}: lockfile refresh failed\n{update.stderr}")
            result = subprocess.run(
                ["cargo", "tree", "--offline", "--manifest-path", str(root / "Cargo.toml"),
                 "--edges", "normal,build", "--prefix", "none", "--format", "{p}"],
                check=False, capture_output=True, text=True,
            )
            if result.returncode:
                raise SystemExit(f"{label}: dependency resolution failed\n{result.stderr}")
            packages = {line.split()[0] for line in result.stdout.splitlines() if line.strip()}
            assert required <= packages, (label, "missing", required - packages)
            assert not forbidden & packages, (label, "leaked", forbidden & packages)
            if "gpui" in features:
                feature_result = subprocess.run(
                    ["cargo", "tree", "--offline", "--manifest-path", str(root / "Cargo.toml"),
                     "--edges", "features", "--invert", "gpui", "--prefix", "none"],
                    capture_output=True, text=True, check=True,
                )
                active = set(feature_result.stdout.splitlines())
                if sys.platform == "darwin":
                    assert 'gpui feature "font-kit"' in active, "GPUI needs real macOS text, not its no-op text system"
                for unused in ["test-support", "inspector", "screen-capture", "leak-detection"]:
                    assert f'gpui feature "{unused}"' not in active, (label, "unexpected production feature", unused)
            print(f"{label}: dependency isolation passed")


if __name__ == "__main__":
    main()
