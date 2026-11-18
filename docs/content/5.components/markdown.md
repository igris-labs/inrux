---
title: "Markdown"
description: "Render a subset of CommonMark through existing native components."
navigation:
  icon: "tabler:markdown"
---

Enable the facade's optional `markdown` feature. This native renderer is separate from Nuxt Content, which renders the documentation website.

## Usage

Replace `src/App.iru` in a [scaffolded project](/getting-started/first-app) with this complete file, then run `cargo run`. Keep the scaffold's `main.rs` and `build.rs`; this file includes its state, handlers and scoped styles. Add `markdown` to the existing `inrux` dependency features in Cargo.toml, keeping your selected engine feature. For egui, that list becomes `features = ["egui", "markdown"]`.

```xml [src/App.iru]
<view>
    <Column class="example">
        <Button text="Update document" @click={update_document} class="action" />
        <Markdown document={document.clone()} class="document" />
    </Column>
</view>
<rust>
use inrux::markdown::Markdown;
use inrux::prelude::*;

let document: Markdown = document(0);
let revision: u32 = 0;

/// Parse once for each explicit document update, not on every view evaluation.
fn document(revision: u32) -> Markdown {
    Markdown::parse(&format!(
        "## Workspace\n\nRevision: {revision}\n\n- Edit the model\n- Save your work\n"
    ))
}
/// Retain the parsed document in component state.
fn update_document(state: &mut State, _: &Context<State>) {
    state.revision += 1;
    state.document = document(state.revision);
}
</rust>
<style scoped>
.example {
    padding: 24px;
    gap: 16px;
}
.action {
    background: #b84200;
    color: white;
    padding: 12px;
    border-radius: 6px;
}
.action:hover {
    background: #913400;
}
.action:focus-visible {
    border-width: 2px;
    border-color: #262626;
}
.document {
    gap: 12px;
}
</style>
```

## API Reference

### Props

| Prop | Type | Default | Description |
| --- | --- | --- | --- |
| `document` | `inrux::markdown::Markdown` | Required | Parsed document retained by the owner; enable the `markdown` feature. |

### Common props

| Prop | Type | Default | Description |
| --- | --- | --- | --- |
| `id` | `String-like expression` | Not set | Style-selector identity; independent of retained mount identity. |
| `class` | `String-like expression` | Not set | Whitespace-separated stylesheet classes. |
| `style` | `Style` | Not set | Explicit Rust style overrides for this node. |
| `key` | `String-like expression` | Not set | Retained sibling identity; preserve it while reordering children. |
| `width` | `Size` | Component style | Explicit width constraint; otherwise use the resolved style. |
| `height` | `Size` | Component style | Explicit height constraint; otherwise use the resolved style. |
| `padding` | `f32` | Component style | Uniform padding in logical pixels; otherwise use the resolved style. |
| `gap` | `f32` | Component style | Child spacing in logical pixels; otherwise use the resolved style. |

“Component style” means the attribute is omitted and the normal style cascade supplies the value. Application styles can override framework defaults.

### Emits

| Event | Payload | Description |
| --- | --- | --- |
| — | — | No component events are emitted. |

Parse during setup or an explicit document update and retain the result in State. The view clones the parsed document without reparsing it.

## Supported blocks

Headings, paragraphs, nested ordered/unordered lists, blockquotes, thematic rules, code blocks and line breaks render through shared Nodes. Long code lines scroll horizontally. Register Noto Sans Mono or another code font and style `.md-code-text` to use it.

Inline emphasis/code flatten to readable text. Links display their labels and images display alt text; they do not navigate or load images. Raw HTML appears as literal text. Tables, footnotes, rich inline font runs, syntax highlighting and heading-specific accessibility roles are not part of this native presentation subset.

Use application classes such as `md-heading`, `md-paragraph`, `md-code` and `md-code-text` for supported styling. See the [native Markdown package](https://github.com/igris-labs/inrux/blob/main/packages/inrux-markdown/README.md) for complete usage and nesting limits.
