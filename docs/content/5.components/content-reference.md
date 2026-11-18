---
title: "Content reference"
description: "Compose cards, badges and separators with shared component descriptions."
navigation:
  icon: "tabler:file-description"
---

These components are implemented once in `inrux-components` and exported through `inrux::Node`. They compose existing Column/Text descriptions. egui, Masonry and GPUI use their existing primitive bindings; there are no engine-specific Card, Badge or Separator implementations.

| Component | Rust | `.iru` props/content | Defaults |
| --- | --- | --- | --- |
| Card | `Node::card(children)` | Ordered child content | Full-width vertical container, 16px padding, 12px gap, white background, 8px radius, 1px neutral painted border |
| Badge | `Node::badge(text)` | `text`, no children | Plain text, 12px font, 4px padding/radius, pale neutral background, dark text |
| Separator | `Node::separator(vertical)` | Optional `vertical: bool`, default false; no children | Horizontal: full width × 1px. Vertical: 1px × 24px. Neutral background |

All accept existing class/style/key and dimension attributes. Component defaults are below application-global CSS, scoped CSS and explicit Rust/markup styles, in that order. No new CSS properties or token system are introduced. Styles can remove the surface decoration or replace individual defaults; customization does not require native-engine imports.

## Usage

Replace `src/App.iru` in a [scaffolded project](/getting-started/first-app) with this complete file, then run `cargo run`. Keep the scaffold's `main.rs` and `build.rs`; the example includes its state, handlers and scoped styles.

```xml [src/App.iru]
<view>
    <Column class="example">
        <Badge text={status_label} class="status" />
        <Card class="sample">
            <Text text="Workspace activity" />
            <Separator />
            <Text text={format!("Actions: {}", count.get())} />
            <Button text="Record an action" @click={increment} class="action" />
        </Card>
    </Column>
</view>
<rust>
use inrux::prelude::*;

let count: Signal<u32> = Signal::new(0);
let status_label_count = count.clone();
let status_label: Computed<&'static str> = Computed::new(move || {
    if status_label_count.get() == 0 {
        "Ready"
    } else {
        "Updated"
    }
});

/// Child controls keep their normal typed event path inside a card.
fn increment(state: &mut State, _: &Context<State>) {
    state.count.update(|value| *value += 1);
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
.sample {
    padding: 20px;
    gap: 12px;
    background: #f5f5f5;
}
.status {
    padding: 8px;
    background: #fff0e8;
    color: #913400;
}
</style>
```

## API Reference

### Props

| Component | Prop | Type | Default | Description |
| --- | --- | --- | --- | --- |
| Badge | `text` | `String-like expression` | `""` | Badge label. |
| Separator | `vertical` | `bool` | `false` | Choose a vertical divider instead of a horizontal divider. |

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

| Component | Event | Payload | Description |
| --- | --- | --- | --- |
| All listed components | — | — | No component events are emitted. |

This view fragment uses the existing state/function model: the owning component declares `name`, `saved` and `save`. Card adds no state management, I/O or implicit save behavior. The complete compiled example is [Content.iru](https://github.com/igris-labs/inrux/blob/main/playground/egui/src/views/Content.iru), with [ordinary Rust handlers](https://github.com/igris-labs/inrux/blob/main/playground/egui/src/features/content.rs). Open **Content** in any playground.

Rust can build the same descriptions directly:

```rust
use inrux::{Color, Node, Size, Style};

fn workspace_summary() -> Node {
    Node::card(vec![
        Node::row(vec![Node::badge("Saved".into())]),
        Node::separator(false),
        Node::text("Workspace settings are stored locally.".into()),
    ]).styled(Style::new()
        .padding(12.0)
        .gap(8.0)
        .background(Color(237, 244, 240))
        .width(Size::Px(360.0)))
}
```

## Sizing and styling

Card is a physical node, not transparent grouping. Its border is paint only and does not reserve extra layout space. Child text inherits ordinary typography; Badge explicitly defaults its own size/color but inherits the registered font family. A styled Card neither disables children nor becomes focusable/clickable. Keys/generations, controlled models and disposal are the existing component contracts.

Badge follows Text's wrapping and flex rules. In a Row, or a Column with `align-items: start`, it normally sizes to text plus padding. A stretching Column can make it full width; `width: auto`/`Size::Content` do not cancel parent stretch. Supply a width to constrain long labels. There is no ellipsis, truncation or automatic font-weight support. Preserve meaningful text for status; color alone should not carry the state.

Separator's orientation chooses default dimensions, rather than a separate layout/paint algorithm. CSS/Rust can override both axes. Explicit vertical height is useful alongside taller controls; it does not automatically fill a parent's height. A horizontal Separator inside a Row requests the available width, so use it in a vertical flow or set an explicit width. It cannot take child content.

For example, these declarations resolve to the same style:

```css
.custom-card {
    padding: 12px;
    gap: 8px;
    background: #edf4f0;
    border-color: #a9bdb3;
    border-width: 1px;
    border-radius: 6px;
}
```

```rust
Style::new().padding(12.0).gap(8.0)
    .background(Color(237, 244, 240))
    .border_color(Color(169, 189, 179))
    .border_width(1.0).rounded(6.0)
```

## Semantics and current limits

Card is a generic container, Badge is plain text, and Separator is decorative geometry. This increment adds no heading, landmark, status/live announcement, semantic separator or resizable splitter role. Badge is not a toggle/button. There is no implicit keyboard action on these components. Native accessibility and text selection retain their existing engine-specific coverage; portable semantic/accessibility services remain [open](/internals/adapter-services).

Card accepts ordinary default children; it adds no named header/footer outlets. Named/scoped content and generation-safe animation refs remain in [alpha status](/getting-started/status). Rectangular [clipping and ScrollArea](/components/scrolling-reference) are now available. Viewport media queries and immutable theme tokens are supported. Stack layering, shadows, container queries and inherited dynamic CSS variables remain separate work. The examples fit the current minimum desktop window without depending on those features.

## Verification

```sh
cargo test -p inrux-adapter --test content_components --locked
cargo test -p inrux-compiler --test content_components --locked
cargo test -p inrux-playground-egui -p inrux-playground-masonry -p inrux-playground-gpui --features inrux-playground-gpui/runtime-shaders --locked
python3 tooling/playground-check/check.py
```

Layout tests cover padding/gaps, separator axes/bounds, Badge metrics/stretch and style precedence. Compiled applications check editing, save/reset, stable input identity, event disposal, font/style parity and real rustc prop types. Native observations and their platform limits are recorded in [validation](/getting-started/status).
