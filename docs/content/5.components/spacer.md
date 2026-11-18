---
title: "Spacer"
description: "Consume free main-axis space in a Row or Column."
navigation:
  icon: "tabler:space"
---

Available with **egui, Masonry and GPUI** through the same public component API. Platform input and accessibility coverage still have [alpha limits](/getting-started/status).

## Usage

Replace `src/App.iru` in a [scaffolded project](/getting-started/first-app) with this complete file, then run `cargo run`. Keep the scaffold's `main.rs` and `build.rs`; this file includes its state, handlers and scoped styles.

```xml [src/App.iru]
<view>
    <Column class="example">
        <Checkbox label="Use a narrower row" model:checked={narrow} />
        <Row style={Style::new().max_width(Size::Px(content_width.get().unwrap_or(640.0)))} class="sample">
            <Text text="Left" width={Size::Content} />
            <Spacer />
            <Text text="Right" width={Size::Content} />
        </Row>
    </Column>
</view>
<rust>
use inrux::prelude::*;

let narrow: Signal<bool> = Signal::new(false);
let content_width_narrow = narrow.clone();
let content_width: Computed<f32> = Computed::new(move || {
    if content_width_narrow.get() {
        320.0
    } else {
        640.0
    }
});
</rust>
<style scoped>
.example {
    padding: 24px;
    gap: 16px;
}
.sample {
    padding: 16px;
    background: #f5f5f5;
}
</style>
```

## API Reference

### Props

This component has no component-specific props. It accepts these common attributes:

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

## Behavior

Spacer is childless, has zero intrinsic size and grows into available main-axis space. Several spacers share free space according to growth. A bounded Column can use it vertically. It cannot create extra space after the parent has exhausted its bounds.

## Styling and limits

Use its growth and size constraints rather than adding artificial fixed margins. Spacer has no model, event or focus target.

Read the [detailed reference](/components/layout-reference) for validation and native integration details. The [playgrounds](/getting-started/playgrounds) include a working Spacer specimen with its surrounding state and reset controls.
