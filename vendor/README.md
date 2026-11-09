# Third-party compatibility patches

These are copies of the locked crates.io releases, selected by the root Cargo.toml `[patch.crates-io]` table. They are external dependencies, excluded from the framework workspace. Framework code and engine selection are unchanged. Do not edit the global Cargo registry cache or suppress future-incompatibility reporting.

## block 0.1.6

Upstream: https://github.com/SSheldon/rust-block (Steven Sheldon; MIT, as declared in the published manifest). The published archive contains no standalone license file; its original source, README and attribution metadata are retained.

Patch: replace the uninhabited `enum Class {}` used by the `_NSConcreteStackBlock` foreign static with an inhabited opaque `#[repr(C)]` zero-sized struct. Only its address is used. The native block header still contains the same pointer-sized `isa`; block creation/copy/release logic is unchanged. Foreign declarations now spell the existing default C ABI explicitly (`extern "C"`), avoiding new `missing_abi` deprecation warnings exposed by a local path dependency. This fixes Rust's `uninhabited_static` diagnostic (rust-lang/rust#74840).

Manifest adjustment: remove the upstream dev-dependency's `path = "test_utils"`, since the published archive omits that sibling test crate. Its original registry version requirement is retained. Upstream tests that require that fixture are not part of the framework workspace checks; native GPUI/Masonry tests and a block-copy smoke check validate the integrated dependency.

## proc-macro-error2 2.0.1

Upstream: https://github.com/GnomedDev/proc-macro-error-2 (MIT OR Apache-2.0). Original license files are retained.

Patch: change `extern crate proc_macro` to `pub extern crate proc_macro`, matching the existing public re-export in `__export`. This fixes Rust's `pub_use_of_private_extern_crate` diagnostic (rust-lang/rust#127909). Macro behavior and dependencies are unchanged.

## Maintenance

As of November 7, 2026, crates.io still lists these exact versions as the latest releases. Remove each patch after a compatible fixed release is available, update only the affected lockfile entries, and rerun the native/workspace checks with `--future-incompat-report`. Existing Cargo reports are historical; use the new build output/report to evaluate the current dependency graph. A consuming application's Cargo root must apply these patches too until upstream publishes fixes; dependency-local patch tables do not propagate into another workspace.
