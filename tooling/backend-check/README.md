# Backend dependency checks

Run `python3 tooling/backend-check/check.py` from the repository after `cargo fetch --locked`.
Python 3 and Cargo are required. No third-party Python packages are used.

The check creates temporary standalone consumers and resolves their production/build dependencies offline. It verifies that a headless facade links no engine, egui selects no Masonry, Masonry selects no egui/GPUI, and GPUI selects no egui/Masonry. Optional devtools do not pull egui into a Masonry/GPUI consumer. When moving to another engine, the temporary consumer refreshes its lockfile offline to accommodate GPUI's exact transitive versions. Workspace-wide feature unification intentionally differs from a standalone application; these consumers test the latter. Dependency resolution is not a performance measurement or a native launch test.

GPUI cases also verify that development-only test support, the native inspector, screen capture and leak detection are absent from production features. On macOS, font-kit must be enabled: omitting it selects GPUI's no-op text implementation even though the font package can appear in the dependency graph.

The three standalone apps under playground/egui, playground/masonry and playground/gpui can be inspected directly with `cargo tree -p inrux-playground-egui --edges normal,build` (substitute masonry or gpui). Each owns its application source with facade defaults disabled. `python3 tooling/playground-check/check.py` verifies portable source/font consistency and individual engine isolation. Building the whole workspace intentionally unifies selected features.

The headless case also forbids Taffy: adapter defaults are disabled by the facade, and each current engine explicitly requests the optional taffy-layout fallback feature. Direct custom providers and their tests compile with that feature disabled. This checks dependency boundaries, not native layout substitution or performance.
