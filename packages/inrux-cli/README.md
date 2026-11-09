# inrux-cli

The `cargo-inrux` executable provides local scaffolding, Cargo command forwarding, SFC validation and a development preview. Applications continue to work with plain cargo build/run/test. Compilation remains Cargo's responsibility and filesystem watching uses notify.

## Install or run locally

```sh
cargo install --path packages/inrux-cli --locked
cargo inrux new ../hello-inrux --framework-path . --backend egui
cd ../hello-inrux
cargo run
```

Run these installation commands from the framework root. Until publication, --framework-path is required and must point to this checkout. The generated app uses explicit local runtime/build dependencies. The destination directory must not already exist.

Without installing globally, use `cargo run -p inrux-cli -- <command>` from the workspace.

## Commands

| Command | Usage |
| --- | --- |
| new | `cargo inrux new <directory> --framework-path <checkout> --backend <engine>` |
| check | `cargo inrux check [cargo check arguments]` |
| build | `cargo inrux build [cargo build arguments]` |
| run | `cargo inrux run [cargo run arguments]` |
| dev | `cargo inrux dev [cargo build arguments] -- [application arguments]` |
| validate | `cargo inrux validate src/App.iru` |

Use --manifest-path for another project and -p/--bin to select one executable in a workspace. Cargo command failures retain their exit codes. validate checks framework and embedded Rust syntax; it does not replace Cargo's Rust type checking.

## Generated structure

The starter contains main.rs, App.iru using `<view>`, an imported Counter component, a normal Rust composable, global CSS and a small build.rs call. Default nested content uses `<Children />`; legacy source spellings remain accepted by the compiler.

## Development preview

The watcher observes local Cargo package sources, existing assets directories and Cargo configuration. Edits are coalesced, then Cargo rebuilds the application. A failed build preserves the last successful preview process. A successful build copies the executable, stops/reaps the old preview and launches the replacement. Copying allows the original Cargo artifact to be rebuilt on platforms that lock running executables. Ctrl-C cleans up the active preview.

Restarting loses application state. State-preserving reload and broader asset/publication checks remain [roadmap work](../../docs/STATUS.md). Keep custom assets under watched source/assets paths; this is not a general asset bundler.

## Organization and checks

`commands.rs` handles dispatch, `scaffold.rs` starter files, `dev.rs` notifications/debouncing, and `preview.rs` Cargo output/process ownership. Starter sources live in templates/.

```sh
cargo test -p inrux-cli --locked
cargo run -p inrux-cli -- validate playground/egui/src/App.iru
```

Refs and richer content syntax will be added to templates only when their public compiler/runtime support exists; see [alpha status](../../docs/STATUS.md).

Engine selection is required while comparative validation is incomplete. `egui` provides the full current prototype surface; `masonry` and `gpui` are experimental and support the generated starter with fewer controls/styles. Unknown choices fail before the destination is created. Change the selected facade Cargo feature to switch a compatible starter; use `cargo update` if a lockfile conflicts with GPUI's exact dependency versions, then rebuild. See [GPUI native setup and shader options](../inrux-gpui/README.md). See [the adapter contract](../inrux-adapter/README.md).
