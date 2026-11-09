# inrux-compiler

Build-time `.iru` compilation, explicit component import validation, reachable graph discovery, Rust generation, and original-source syntax diagnostics. Applications use this as a build dependency and `inrux` at runtime.

## Cargo integration

```rust
// build.rs
/// Compile the application root and its explicitly imported component graph.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    inrux_compiler::build_app("src/App.iru", std::env::var("OUT_DIR")?)?;
    Ok(())
}
```

Include `concat!(env!("OUT_DIR"), "/inrux_app.rs")` once at the main.rs crate root. The generated modules expose app and imported components; main.rs mounts app::setup(). `inrux_global_styles()` gathers global rules from reachable sources.

For `src/components/Counter.iru`, write `use crate::components::counter as Counter;` inside the parent's Rust block. Rust modules declared by main.rs and external Cargo libraries retain ordinary imports. Imported runtime-selected candidates are included; unimported bodies are not compiled. Source directories are Cargo rebuild inputs so additions/deletions trigger discovery.

## Source compilation

```rust
use inrux_compiler::{SourceFile, compile_source};

/// Preserve source identity while compiling a stateless component.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = "<view>\n<Text text=\"Hello\" />\n</view>\n<rust>\npub struct Props;\n</rust>\n";
    let generated = compile_source(SourceFile::new("src/Hello.iru", source))?;
    assert!(generated.contains("pub fn view"));
    Ok(())
}
```

The supported blocks are `<view>`, `<rust>`, `<style>` and `<style scoped>`. `<template>` is a compatible view alias. `<Children />` renders default content; `<Slot />` remains an alias. Both spellings share validation, code generation, source coordinates and formatting. Declaring both view block spellings in one file is a duplicate-block error.

`compile` handles in-memory input; `compile_file` compiles one file; `build_app` produces a reachable bootstrap. `format_source` delegates the Rust body to rustfmt and preserves view/style bytes, comments and block spellings.

## Editor metadata

`editor_document` returns original block/import/binding ranges and public Props fields with Rust types, doc comments and identifier spans. `builtin_attributes` returns the same built-in tag/attribute names used by validation; `schema.rs` owns that shared vocabulary. The language server exposes this metadata to VS Code without a separate Rust parser in TypeScript. Syntactically incomplete Rust temporarily has no custom import/Props metadata; built-in completion remains available when SFC blocks can be identified.

## Diagnostics and modules

`source.rs` indexes UTF-8 ranges; `sfc.rs` recognizes lexical block boundaries; `view.rs` parses markup; `imports.rs` resolves explicit aliases; `controls.rs` lowers bindings; `codegen.rs` lowers composition; `style_codegen.rs` translates styles; `build.rs` owns Cargo integration.

CompileError exposes a code, phase, original range, path, position, help and related locations. The public Phase::Template variant name is retained for compatibility and also covers canonical view syntax. CSS errors identify their block. Rust type errors still point into generated Rust; source mapping remains a release milestone.

## Verify

```sh
cargo test -p inrux-compiler --locked
cargo check -p inrux-playground-egui --locked
```

The playground includes real rustc pass/fail fixtures. Compiler-only parsing does not establish Rust type correctness. Ref attributes, named content and scoped render functions remain [alpha status](../../docs/STATUS.md), and are not silently accepted as implemented features.

## Built-in layout components

Box, Container, Center and Spacer lower to engine-independent Node constructors without application imports. Spacer rejects nested children. Common keys/classes/styles and explicit layout attributes remain available; compiled CSS preserves validated growth and automatic margins. See [the public component guide](../../docs/LAYOUT-COMPONENTS.md).

Flex, Grid and AspectRatio now have built-in lowering too. Flex accepts typed direction/wrap; Grid accepts a u16 columns prop; AspectRatio accepts a positive f32 ratio. CSS display/direction/column repeats/ratio lower into the same Style. Existing If/For wrappers remain physical groups, including inside grids; direct-cell repeated data can use Rust-built grid descriptions. See [usage and current grammar](../../docs/LAYOUT-COMPONENTS.md#flex-grid-and-aspectratio).

## Built-in content components

Card accepts ordered children; Badge accepts text; Separator accepts an optional Rust bool vertical prop. All lower to shared Node constructors and accept existing classes/keys/styles/dimensions. Badge/Separator reject children, and unsupported props/events are diagnosed against the authored tag. Prop types are checked by rustc. See [the content guide](../../docs/CONTENT-COMPONENTS.md).

## Editor source maps

`build_app` emits ordinary generated component `.rs` modules under `inrux_modules/`, referenced by the bootstrap with normal Rust module paths. It preserves authored Rust-body bytes and writes `inrux_app.map.json` next to `inrux_app.rs`. Version 1 records reachable original files/text and original/generated UTF-8 Rust ranges. These are build artifacts, not application runtime dependencies or embedded component strings. The editor uses them to place unsaved Rust in the actual Cargo module context and map results back to `.iru`. Rebuild after graph/view changes. `editor_document` supplies lexical block ranges and explicit import aliases even when Rust is incomplete; it reuses the same SFC parser as compilation. See [VS Code setup and limits](../../tooling/inrux-vscode/README.md).

## Clipping and scrolling

[ScrollArea and overflow usage](../../docs/SCROLLING.md) covers the shared component, CSS/Rust overflow, UI-owned offsets, typed observations and revision-bearing commands. Native bindings supply rectangular paint/input clipping and native focus/pointer operations; shared policy supplies scroll bounds, nested routing and scrollbar geometry. The Scrolling playground view exercises these paths. Virtualization and complete accessible scroll actions remain separate work.

Input/TextInput are strictly single-line; TextArea provides multiline editing, wrapping and vertical caret scrolling on all three engines. See [text editing](../../docs/TEXT-EDITING.md) for `.iru`, Rust, model and styling usage.

## Standalone compiled CSS

`compile_stylesheet(css)` returns a Rust expression constructing typed rules, including expanded immutable tokens, compiled selectors and media predicates. Write it to OUT_DIR in build.rs, track the input with Cargo rerun-if-changed, and `include!` it. The playground build scripts demonstrate complete light/dark compilation; [CSS engine](../../docs/CSS-ENGINE.md) documents scope, specificity, diagnostics and performance boundaries.

## Switch, ranges and control styling

Switch, determinate Progress and bounded Meter share portable descriptors and work with all three engines. Inherited `accent-color` and independent `row-gap`/`column-gap` compile into typed styles. See [APIs, styling, validation and accessibility limits](../../docs/BOOLEAN-AND-RANGE-COMPONENTS.md) and [CSS support](../../docs/CSS-ENGINE.md). Native bindings remain private; only the selected engine is required.

## Headless style foundation

[Public parts and owner-state styles](../../docs/STYLING-PARTS.md) now provide overrideable boolean metrics/paint, labels, range fills and editor placeholder ink across all three engines. CSS and Rust share the same resolver and physical margin cascade. [Literal color values](../../docs/STYLE-COLORS.md) include alpha/OKLCH through the pinned color parser, with native alpha preserved. The guide records the unpublished Color tuple-pattern migration and remaining CSS/part limits. Application recipes remain in playground styles.
