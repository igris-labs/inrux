# inrux-language-server

A Rust language service for `.iru` components. It reuses the compiler's original-source syntax/import diagnostics and delegates Rust-body formatting to rustfmt. The transport is standard LSP over stdin/stdout through lsp-server.

## Build and connect

```sh
cargo build -p inrux-language-server --locked
# Optional installation on PATH:
cargo install --path packages/inrux-language-server --locked
```

The executable speaks LSP rather than a terminal REPL. Start it through the [VS Code client](../../tooling/inrux-vscode/README.md) or configure another LSP client for language inrux and extension .iru. With no installation, point the client at target/debug/inrux-language-server (or the corresponding .exe).

The VS Code setting is `inrux.server.path`. Configure an executable path, not a shell command containing extra arguments. Install rustfmt with the Rust toolchain for Format Document.

## Current capabilities

- Full document synchronization, including unsaved text.
- Diagnostics for view/block/Rust/style syntax and missing explicit component imports.
- Original byte spans converted into zero-based LSP UTF-16 ranges.
- Diagnostics cleared on correction and document close.
- Formatting of `<rust>` with comments preserved and other block bytes unchanged.
- Standard initialize/shutdown handling and method-not-found responses for unsupported requests.

Both `<view>`/`<Children />` and their `<template>`/`<Slot />` aliases use the same compiler. Formatting preserves the chosen spelling.

## Structure and validation

main.rs owns the protocol lifecycle. documents.rs owns text/version state and diagnostic coordinates. requests.rs owns supported request handling. No second parser or formatter is maintained in this project.

```sh
cargo test -p inrux-language-server --locked
cargo test -p inrux-compiler --locked
```

Tests include a real subprocess handshake, unsaved diagnostics, formatting and shutdown, plus Unicode coordinate cases. They do not constitute a VS Code extension-host test.

The VS Code client now adds Rust completion/hover/definition/signature forwarding and view/CSS formatting; the Rust server itself remains focused on compiler metadata, syntax diagnostics and rustfmt. Complete type-diagnostic parity, rename/refactors and compatible reload remain future work. Named content, render functions and NodeRef capabilities must reuse compiler metadata when delivered; their [alpha status](../../docs/STATUS.md) is not an advertised server feature today.

## VS Code authoring integration

`inrux/documentInfo` exposes compiler-owned UTF-16 block regions and explicit imports, including unsaved/incomplete Rust. An optional text snapshot makes editor formatting/metadata requests independent of delayed full-sync notifications. Standard LSP formatting remains Rust-only and preserves other bytes. The [VS Code client](../../tooling/inrux-vscode/README.md) composes view and built-in CSS formatting atomically and forwards Cargo-aware Rust requests to rust-analyzer using generated source maps. These are separate services, not an in-process Rust analyzer implementation.
