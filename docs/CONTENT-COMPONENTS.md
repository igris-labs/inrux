# Card, Badge and Separator

These components are implemented once in `inrux-components` and exported through `inrux::Node`. They compose existing Column/Text descriptions. egui, Masonry and GPUI use their existing primitive bindings; there are no engine-specific Card, Badge or Separator implementations.

| Component | Rust | `.iru` props/content | Defaults |
| --- | --- | --- | --- |
| Card | `Node::card(children)` | Ordered child content | Full-width vertical container, 16px padding, 12px gap, white background, 8px radius, 1px neutral painted border |
| Badge | `Node::badge(text)` | `text`, no children | Plain text, 12px font, 4px padding/radius, pale neutral background, dark text |
| Separator | `Node::separator(vertical)` | Optional `vertical: bool`, default false; no children | Horizontal: full width × 1px. Vertical: 1px × 24px. Neutral background |

All accept existing class/style/key and dimension attributes. Component defaults are below application-global CSS, scoped CSS and explicit Rust/markup styles, in that order. No new CSS properties or token system are introduced. Styles can remove the surface decoration or replace individual defaults; customization does not require native-engine imports.

## Compose content and normal controls

```xml
<Card>
    <Row>
        <Text text="Workspace settings" />
        <Spacer />
        <Badge :text={if state.saved { "Saved" } else { "Unsaved changes" }} />
    </Row>
    <Separator />
    <Input model:value={state.name} placeholder="Workspace name" />
    <Row gap={12.0}>
        <Button text="Save" @click={save} />
        <Separator vertical={true} height={Size::Px(40.0)} />
        <Text text="Stored locally" />
    </Row>
</Card>
```

This view fragment uses the existing state/function model: the owning component declares `name`, `saved` and `save`. Card adds no state management, I/O or implicit save behavior. The complete compiled example is [Content.iru](../playground/egui/src/views/Content.iru), with [ordinary Rust handlers](../playground/egui/src/features/content.rs). Open **Content** in any playground.

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

Card is a generic container, Badge is plain text, and Separator is decorative geometry. This increment adds no heading, landmark, status/live announcement, semantic separator or resizable splitter role. Badge is not a toggle/button. There is no implicit keyboard action on these components. Native accessibility and text selection retain their existing engine-specific coverage; portable semantic/accessibility services remain [open](ADAPTER-SERVICES.md).

Card accepts ordinary default children; it adds no named header/footer outlets. Named/scoped content and generation-safe animation refs remain in [alpha status](STATUS.md). Rectangular [clipping and ScrollArea](SCROLLING.md) are now available. Viewport media queries and immutable theme tokens are supported. Stack layering, shadows, container queries and inherited dynamic CSS variables remain separate work. The examples fit the current minimum desktop window without depending on those features.

## Verification

```sh
cargo test -p inrux-adapter --test content_components --locked
cargo test -p inrux-compiler --test content_components --locked
cargo test -p inrux-playground-egui -p inrux-playground-masonry -p inrux-playground-gpui --features inrux-playground-gpui/runtime-shaders --locked
python3 tooling/playground-check/check.py
```

Layout tests cover padding/gaps, separator axes/bounds, Badge metrics/stretch and style precedence. Compiled applications check editing, save/reset, stable input identity, event disposal, font/style parity and real rustc prop types. Native observations and their platform limits are recorded in [validation](STATUS.md).
