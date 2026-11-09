# Playground checks

Run `python3 tooling/playground-check/check.py` from the repository. It checks that the three standalone apps have identical portable source, tests, styles and licensed font bytes, plus distinct engine-only production dependency graphs. `src/engine.rs` and egui's additional native harness are deliberately local. No runtime shared application, generated source copy, symlink or dependency on another playground is required.

Edit matching portable examples in all three applications; review intentional engine differences in `engine.rs`. The script is a consistency check, not a source generator. It does not compile or replace the application and native tests.
