# Getting started

## Requirements

Use a current stable Rust toolchain with Cargo and a native desktop build environment. This alpha uses Rust edition 2024; an exact minimum supported Rust version has not been established. Python 3 runs the repository consistency checks. Node.js 22 and npm are needed only for the VS Code extension.

```sh
rustup update stable
rustup component add rustfmt clippy
```

macOS needs Xcode Command Line Tools (`xcode-select --install`). GPUI's normal shader build may also need full Xcode and its Metal compiler. If that compiler is unavailable, the playground provides the `runtime-shaders` feature described below.

On Ubuntu, the CI environment installs:

```sh
sudo apt-get update
sudo apt-get install -y libx11-dev libxi-dev libxcursor-dev libxrandr-dev libxinerama-dev libxkbcommon-dev libwayland-dev libgl1-mesa-dev libfontconfig1-dev libxkbcommon-x11-dev libxcb-xkb-dev libvulkan-dev libssl-dev pkg-config clang
```

Windows requires the MSVC Rust target and Visual Studio C++ build tools with a Windows SDK. GPU/window-system availability and upstream backend requirements can affect native builds. A CI matrix exists for Linux, macOS and Windows; that is not a claim that every desktop configuration is validated.

## Get the source

```sh
git clone https://github.com/igris-labs/inrux.git
cd inrux
```

Keep `Cargo.lock` and the checked-in `vendor/` patches. They are part of the reproducible checkout. There is no crates.io release yet.

## Run an example

From the repository root, run one application at a time:

```sh
cargo run -p inrux-playground-egui --locked
cargo run -p inrux-playground-masonry --locked
cargo run -p inrux-playground-gpui --locked
```

On macOS without the Metal compiler:

```sh
cargo run -p inrux-playground-gpui --features runtime-shaders --locked
```

The runtime shader option changes startup work; account for it in comparisons. The first build downloads and compiles native dependencies and can take substantial time. [Playground usage](PLAYGROUNDS.md) explains the controls and examples.

## Create your own app

From the framework root:

```sh
cargo run -p inrux-cli -- new ../hello-inrux --framework-path . --backend egui
cd ../hello-inrux
cargo run
```

The destination must not already exist. `--framework-path` points to your Inrux checkout. Choose `egui`, `masonry` or `gpui`; the generated manifest selects only that backend. For a generated GPUI app on macOS without the Metal compiler, enable the facade's `gpui-runtime-shaders` feature in its Cargo.toml.

The generated project contains:

```text
hello-inrux/
├── Cargo.toml
├── build.rs
└── src/
    ├── main.rs
    ├── App.iru
    ├── components/Counter.iru
    ├── composables/
    └── styles/global.css
```

`build.rs` compiles `App.iru` and its imported component graph. `main.rs` includes the generated bootstrap and mounts the root. Edit `App.iru` for layout and logic; import reusable `.iru` components explicitly from its `<rust>` block. Ordinary `.rs` helpers and Cargo dependencies remain available.

The scaffold currently parses its global stylesheet once at startup. The playground build scripts show how to precompile standalone CSS into Rust as well as compiling `.iru` style blocks. See [authoring](AUTHORING.md) and [styling](CSS-ENGINE.md).

## Development commands

You may install the local CLI once:

```sh
cargo install --path packages/inrux-cli --locked
```

Then, from an application directory, use `cargo inrux check`, `cargo inrux run` or `cargo inrux dev`. The development command watches sources and rebuilds/restarts the process; it does not preserve application state. Plain Cargo commands continue to work. See the [CLI README](../packages/inrux-cli/README.md).

For `.iru` formatting and Rust editing features, follow [VS Code setup](VSCODE.md). For reproducible bug reports, include `rustc --version`, your OS and selected backend.
