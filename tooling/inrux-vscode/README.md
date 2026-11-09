# Inrux for VS Code

`.iru` single-file component editing: syntax highlighting, full-document formatting, component/prop IntelliSense, component Ctrl/Cmd-click, compiler diagnostics and source-mapped Rust language features. Runtime/framework code remains Rust; this package is the editor integration.

## Install from this checkout

From the framework root:

```sh
code --install-extension rust-lang.rust-analyzer
rustup component add rust-src rustfmt
npm ci --prefix tooling/inrux-vscode
npm run install:local --prefix tooling/inrux-vscode
```

The installer builds `inrux-language-server`, compiles the TypeScript client and creates:

```text
~/.vscode/extensions/inrux-local.inrux-vscode
  -> <checkout>/tooling/inrux-vscode
```

It refuses to overwrite an unrelated directory or link. Set `INRUX_VSCODE_EXTENSIONS_DIR` for a different editor/profile extensions directory. On Windows it uses a directory junction. Keep the checkout in place while the extension is linked. The local installer does not publish or download Inrux from a marketplace.

Run **Developer: Reload Window** once after linking. Subsequent changes need no reinstall. During development:

```sh
npm run dev --prefix tooling/inrux-vscode
```

This watches/compiles TypeScript and rebuilds the Rust compiler/language-server when their source changes. Reload the VS Code window to load changed JavaScript/server code. Grammar/manifest changes need a reload too. The symlink is persistent; the watcher runs only while that command is running. `npm run compile` is available for a one-off client build.

## Usage

Open `App.iru` with language mode **Inrux** in a trusted Cargo workspace. The extension supplies the language-specific default formatter, so a global Prettier default can stay unchanged. Use Format Document or the editor's format-on-save setting.

| Feature | Behavior |
| --- | --- |
| Highlighting | Lowercase tags use tag/keyword scopes; PascalCase components use class/type scopes, with theme-aware semantic tokens. Attributes, strings, triggers, Rust expressions and embedded CSS have their own scopes. Exact colors follow the active theme. |
| Component IntelliSense | Built-in tags/accepted attributes come from compiler validation metadata. Explicitly imported `.iru` components expose public `Props` fields, Rust types/doc comments, `Emit` events and paired model bindings. Unsaved child edits are included; no prior Cargo build is required for markup completion/navigation. Use Ctrl+Space for manual suggestions. |
| File icons | `.iru` files use light/dark Inrux brand marks. VS Code may let the selected file-icon theme override language icons; see [language icon behavior](https://code.visualstudio.com/api/extension-guides/file-icon-theme). |
| Rust block | Uses the Rust grammar. Cargo-backed completion, hover, signature help and definitions forward to the installed rust-analyzer and map authored Rust ranges back to `.iru`. |
| Component navigation | Ctrl/Cmd-click or F12 on opening/closing component names follows explicit grouped/aliased Rust imports to the corresponding `.iru` file. Works on complete opening/closing tags while later markup is unfinished, and on imported alias declarations. No guessing imports from a global component registry. |
| Formatting | Reindents view tags while preserving attribute/expression/literal bytes; delegates Rust to rustfmt and CSS to VS Code's built-in CSS formatter. The result is one atomic edit; formatting again should produce no change. |
| Problems | The Rust language server supplies original-source framework/Rust syntax diagnostics. Available rust-analyzer diagnostics on projected Rust are mapped back to authored regions. |

`<view>`, `<rust>`, global/scoped `<style>` and current `<Children />` are recognized; legacy `<template>`/`<Slot />` spellings are preserved. Highlighting lowercase tags does not add unsupported syntax to the framework compiler.

## Cargo-aware Rust setup

Build/check the application once with the current compiler. `build_app` writes `inrux_app.rs`, ordinary generated component modules under `inrux_modules/`, and `inrux_app.map.json` in Cargo's output directory. For this checkout, for example:

```sh
cargo check -p inrux-playground-egui --locked
```

The extension uses Cargo metadata to find the target directory. The source map identifies original Rust bodies inside the **real generated module tree**, so `crate::` helpers and ordinary dependency types retain their Cargo context. Unsaved Rust is sent as an in-memory generated-file overlay; the editor does not overwrite `.iru`, `.rs` or build artifacts. Definitions into mapped Rust return to `.iru`; ordinary Rust definitions remain in their `.rs` files. The bridge handles UTF-8 build offsets and UTF-16 editor positions separately.

Rebuild when adding/removing/importing components or changing generated view/props structure. The currently built graph supplies generated declarations; unsaved Rust bodies update automatically. Unreachable `.iru` files have highlighting/formatting/component navigation, but need a built graph before Cargo-aware Rust projection works. The Inrux output channel explains missing build artifacts. Close a manually opened generated component `.rs` file before using its projected overlay.

This initial bridge exposes completion/hover/definition/signature support **inside `<rust>`**, not all rust-analyzer actions in every template expression. Rename/refactors, cross-language references, template expression type mapping, semantic Rust token forwarding and a full diagnostic release gate remain further work. Available rust-analyzer diagnostics depend on its configuration; original generated-file diagnostics can also appear in Problems. A successful Cargo check remains the authoritative type check. Missing imports are never silently synthesized outside mapped authored Rust.

## Configuration

| Setting | Meaning |
| --- | --- |
| `inrux.server.path` | Empty auto-detects the linked checkout's `target/debug/inrux-language-server`, then PATH. An explicit value overrides detection. |

Restart the extension host after changing the executable setting. Server stdout is reserved for LSP. The extension requires a trusted workspace before launching any configured server or Cargo integration. Rust support uses the installed rust-analyzer extension's running client; this integration is checked against the locally installed version, not a promise about every future extension API version.

## Modules and checks

`src/extension.ts` owns startup; `src/editor/` separates formatting, markup tokenization, completion context/items, shared component-path resolution, navigation, tag tokens, Rust projection and protocol types. The compiler owns built-in attribute and public Props metadata. `syntaxes/` owns TextMate highlighting. Source metadata and generation live in the compiler; protocol diagnostics/formatting live in the [Rust server](../../packages/inrux-language-server/README.md).

```sh
npm run check --prefix tooling/inrux-vscode
npm run test:unit --prefix tooling/inrux-vscode
npm test --prefix tooling/inrux-vscode
cargo test -p inrux-compiler -p inrux-language-server --locked
```

The host test launches an isolated VS Code profile and temporary Cargo fixture with paths containing spaces. It uses the local VS Code app and installed rust-analyzer, without changing normal user settings. Set `INRUX_VSCODE_EXECUTABLE` for a different VS Code binary. Unit grammar checks currently use the macOS VS Code installation's bundled Rust/CSS grammars. [VS Code's embedded-language guide](https://code.visualstudio.com/api/language-extensions/embedded-languages) explains grammar/service separation; [rust-analyzer configuration](https://rust-analyzer.github.io/book/configuration) describes its Cargo integration.
