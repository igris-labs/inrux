# inrux-markdown

Optional, engine-independent Markdown presentation backed by [pulldown-cmark](https://docs.rs/pulldown-cmark/0.13.6/pulldown_cmark/). Documents render with existing Inrux Nodes; this package has no window, renderer, browser or layout-engine dependency.

Enable `inrux = { ..., features = ["markdown", "egui"] }` (or your selected engine). Ordinary applications without the `markdown` feature do not compile this package or its parser. A headless consumer can depend directly on this crate.

## Parse once, render shared components

```rust
use inrux::markdown::Markdown;

let document = Markdown::parse("## Configuration\n\n- Edit the model\n- Save the workspace\n");
let node = document.view();
```

Keep the document in component State or a cached load result. Cloning the document shares its owned block tree through Arc; `.view()` constructs ordinary component descriptions without reparsing source. In `.iru`:

```xml
<view>
    <Markdown :document={state.documentation.clone()} />
</view>
<rust>
use inrux::markdown::Markdown;
#[derive(Clone)]
pub struct State { pub documentation: Markdown }
/// Parse the static document once per component mount.
pub fn setup() -> State {
    State { documentation: Markdown::parse("# Workspace\n\nNative documentation.") }
}
</rust>
```

`parse.rs` owns CommonMark event translation; `view.rs` maps owned blocks into shared components; `lib.rs` owns the immutable document API. Headings, paragraphs, ordered/unordered nested lists, blockquotes, thematic rules, fenced/indented code and hard/soft line breaks are supported. Code preserves spaces/newlines and uses a horizontal ScrollArea for long lines.

This first presentation subset flattens inline emphasis/strong/code into readable text, renders link labels and image alt text without interactive navigation or image loading, and shows raw HTML literally. It does not provide HTML execution, syntax highlighting, rich inline font runs, tables, footnotes, interactive links, document-anchor lookup or heading-specific accessibility roles. Nesting beyond 128 presentation frames falls back to literal source text rather than recursing into an unbounded native tree.

## Styling and fonts

Classes are `markdown`, `md-paragraph`, `md-heading`, `md-h1` through `md-h6`, `md-list`, `md-list-item`, `md-quote`, `md-code`, and `md-code-text`. Defaults use `Node::with_defaults`, below application CSS and explicit overrides.

Register a code face through `WindowOptions.fonts`, then apply `font-family` to `.md-code-text`. The renderer imposes no dependency on an installed font. The playgrounds bundle [Noto Sans Mono](https://github.com/google/fonts/tree/main/ofl/notosansmono) with its OFL license, identically across all three applications. `white-space: pre` and the Rust `WhiteSpace::Pre` value use unwrapped native measurement and painting; paragraph text continues to use the existing wrapping path. No font or document is fetched at runtime.

```css
.md-code-text { font-family: "Noto Sans Mono"; font-size: 13px; }
.md-code { padding: 12px; }
.md-paragraph { color: #717171; }
```

Run `cargo test -p inrux-markdown --locked`. The workspace also checks headless Markdown dependency isolation, compiled `.iru` usage, shared geometry, and all three native playgrounds.
