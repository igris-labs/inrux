#!/usr/bin/env python3
"""Compile component usage files in the real CLI starter and validate initial scenes."""
from pathlib import Path
import os
import re
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[2]
COMPONENTS = next((ROOT / "docs/content").glob("*.components"))
SCENE_TEST = r'''
#[cfg(test)]
mod documentation_example {
    /// Check the published description without starting a native event loop.
    #[test]
    fn initial_scene_obeys_the_selected_backend_contract() {
        use inrux::Adapter;
        let state = crate::app::setup();
        let worker = inrux::Worker::spawn(state.clone(), 64, || {}).unwrap();
        let events = inrux::Events::new(worker.sender());
        let mut tree = inrux::ComponentHost::default().render(
            &state, &events, &inrux::Scope::default(),
        );
        let sheet = inrux::CompiledStyles::new(crate::inrux_global_styles());
        tree.resolve_styles(&[&sheet], inrux::Viewport { width: 960.0, height: 720.0 });
        tree.inherit(&inrux::Style::new());
        inrux::backends::Egui.capabilities().validate(&tree).unwrap();
    }
}
'''


def examples():
    """Read canonical Markdown; examples have no generated source copy to maintain."""
    for path in sorted(COMPONENTS.glob("*.md")):
        if path.stem == "overview":
            continue
        match = re.search(
            r"## Usage\n[\s\S]*?```xml \[src/App\.iru\]\n([\s\S]*?)\n```",
            path.read_text(),
        )
        if not match:
            raise ValueError(f"{path}: missing complete src/App.iru usage example")
        source = match[1] + "\n"
        for block in ("view", "rust", "style scoped"):
            if f"<{block}>" not in source:
                raise ValueError(f"{path}: missing <{block}> block")
        yield path, source
    # Keep the advanced manual form as runnable as the default component examples.
    guide = next((ROOT / "docs/content").glob("*.authoring")) / "01.iru-format.md"
    source = re.search(r"```xml \[src/App\.iru\]\n([\s\S]*?<rust explicit>[\s\S]*?)\n```", guide.read_text().split("## Manual setup with rust explicit", 1)[1])
    if not source:
        raise ValueError("Missing complete explicit-setup example")
    yield guide, source[1] + "\n"


def with_computed_checks(source):
    """Verify cached values invalidate and cloned state shares their signal sources."""
    rust = re.search(r"<rust(?: explicit)?>([\s\S]*?)</rust>", source)[1]
    computed = re.findall(r"^let (\w+): Computed<.+> =", rust, re.MULTILINE)
    if not computed:
        return source
    signals = re.findall(r"^let (\w+): Signal<(bool|u32|f64)> =", rust, re.MULTILINE)
    if not signals:
        raise ValueError("Computed example needs reactive sources to exercise")
    statements = ["let state = setup();", "let shared = state.clone();"]
    for name in computed:
        statements.append(f"let {name}_before = state.{name}.get().unwrap();")
    for name, kind in signals:
        statements.append(f"let {name}_before = state.{name}.get();")
        if kind == "bool":
            change = f"!{name}_before"
        elif kind == "f64":
            change = "100.0"
        else:
            change = f"{name}_before + 1"
        statements.append(f"shared.{name}.set({change});")
    for name in computed:
        statements.append(f'assert_ne!(state.{name}.get().unwrap(), {name}_before, "{name} must invalidate");')
    for name, _ in signals:
        statements.append(f"shared.{name}.set({name}_before);")
    for name in computed:
        statements.append(f'assert_eq!(state.{name}.get().unwrap(), {name}_before, "{name} must restore");')
    test = "\n#[cfg(test)] mod documentation_example_computed { use super::*; #[test] fn invalidates_and_restores() {\n"
    test += "\n".join(statements) + "\n} }\n"
    return source.replace("</rust>", test + "</rust>", 1)


def run(command, environment):
    """Keep Cargo errors visible and fail before validating another example."""
    subprocess.run(command, cwd=ROOT, env=environment, check=True)


def main():
    """Use egui for the superset, including the explicitly egui-only examples."""
    samples = list(examples())
    if not samples:
        raise ValueError("No component usage examples were found")
    environment = os.environ.copy()
    environment.setdefault("CARGO_TARGET_DIR", str(ROOT / "target"))
    with tempfile.TemporaryDirectory(prefix="inrux documentation ") as directory:
        project = Path(directory) / "application"
        run([
            "cargo", "run", "-p", "inrux-cli", "--locked", "--", "new", str(project),
            "--framework-path", str(ROOT), "--backend", "egui",
        ], environment)
        manifest = project / "Cargo.toml"
        manifest.write_text(manifest.read_text().replace(
            'features = ["egui"]', 'features = ["egui", "markdown"]',
        ))
        # Reuse reviewed dependency versions while Cargo adds the starter package entry.
        shutil.copyfile(ROOT / "Cargo.lock", project / "Cargo.lock")
        entry = project / "src/main.rs"
        entry.write_text(entry.read_text() + SCENE_TEST)
        for path, source in samples:
            print(f"Checking {path.name}: Rust types, bindings, styles, initial scene and computed invalidation", flush=True)
            (project / "src/App.iru").write_text(with_computed_checks(source))
            run([
                "cargo", "test", "--manifest-path", str(manifest), "--quiet",
                "documentation_example",
            ], environment)
    print(f"All {len(samples)} component usage examples passed.")


if __name__ == "__main__":
    main()
