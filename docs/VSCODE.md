# VS Code setup

The extension supports `.iru` highlighting, formatting, component Ctrl/Cmd-click and Rust editing through rust-analyzer. It is installed from the checkout; there is no marketplace release yet.

## Install a linked extension

Install VS Code and Node.js 22. Make the `code` command available in your shell. From the Inrux root:

```sh
code --install-extension rust-lang.rust-analyzer
rustup component add rust-src rustfmt
npm ci --prefix tooling/inrux-vscode
npm run install:local --prefix tooling/inrux-vscode
```

The installer builds the Rust language server, compiles the client and links:

```text
~/.vscode/extensions/inrux-local.inrux-vscode
  -> <checkout>/tooling/inrux-vscode
```

It refuses to replace an unrelated directory/link. Windows uses a directory junction. Set `INRUX_VSCODE_EXTENSIONS_DIR` before running the installer for a different extensions directory/profile. Keep the source checkout in place.

Run **Developer: Reload Window** once. You do not need to reinstall after every source edit. Recompile the changed client/server and reload the editor; for active development use:

```sh
npm run dev --prefix tooling/inrux-vscode
```

That command watches TypeScript and Rust tooling sources. It must remain running to watch changes. Reload VS Code after JavaScript/server/grammar/manifest changes.

## Use it

Open a trusted Inrux Cargo workspace. Build the application once so Cargo-aware Rust features have a generated module tree and source map:

```sh
cargo check -p inrux-playground-egui --locked
```

Open `App.iru`, verify language mode **Inrux**, then:

- Format Document formats view, Rust and CSS together. You can enable editor format-on-save.
- Lowercase tags and PascalCase components have separate theme-aware token categories.
- `.iru` files use the Inrux brand mark in light/dark variants, unless your file-icon theme overrides language icons.
- Type a tag/attribute or press Ctrl+Space for component, prop, event and model suggestions. Imported components expose public Rust `Props` types and doc comments, including unsaved changes. Markup completion/navigation do not require a generated build graph.
- Ctrl/Cmd-click or F12 on imported component tags opens the corresponding `.iru` source.
- `<rust>` uses Rust highlighting and source-mapped completion, hover, definitions and signature help through the installed rust-analyzer extension.
- Compiler syntax diagnostics and available mapped Rust diagnostics appear in Problems.

The default formatter is `inrux-local.inrux-vscode`. Set `inrux.server.path` only if automatic language-server discovery fails. The **Inrux** output channel reports missing build/source-map information.

Rebuild after changing component imports or generated props/view structure. Unsaved Rust bodies are projected in memory; the extension does not rewrite your source or build outputs. Unreachable `.iru` files still have markup navigation/formatting, but need a built component graph for Cargo-aware Rust projection.

## Limits and troubleshooting

Cargo remains the authoritative Rust type check. Full template-expression diagnostic mapping, rename/refactoring and every rust-analyzer feature are not implemented. Colors depend on your theme. If the extension fails to activate, check workspace trust, the linked checkout path and the language-server executable; then rebuild and reload.

For implementation details, checks and custom host-test configuration, see the [extension README](../tooling/inrux-vscode/README.md).
