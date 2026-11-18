---
title: "Box"
description: "Create a physical vertical container."
navigation:
  icon: "tabler:box"
---

Available with **egui, Masonry and GPUI** through the same public component API. Platform input and accessibility coverage still have [alpha limits](/getting-started/status).

## Usage

Replace `src/App.iru` in a [scaffolded project](/getting-started/first-app) with this complete file, then run `cargo run`. Keep the scaffold's `main.rs` and `build.rs`; this file includes its state, handlers and scoped styles.

```xml [src/App.iru]
<view>
    <Column class="example">
        <Checkbox label="Use a larger gap" model:checked={spacious} />
        <Box gap={spacing} class="sample">
            <Text text="First child" />
            <Text text="Second child" />
        </Box>
    </Column>
</view>
<rust>
use inrux::prelude::*;

let spacious: Signal<bool> = Signal::new(false);
let spacing_spacious = spacious.clone();
let spacing: Computed<f32> =
    Computed::new(move || if spacing_spacious.get() { 24.0 } else { 8.0 });
</rust>
<style scoped>
.example {
    padding: 24px;
    gap: 16px;
}
.sample {
    padding: 16px;
    background: #f5f5f5;
    border-radius: 8px;
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

Box is a physical layout node, not a transparent fragment or a browser div. It defaults to vertical composition with full available width and no implicit gap/padding. Child identities and events remain ordinary Inrux nodes.

## Styling and limits

Use dimensions, constraints, padding, gap and overflow from the supported CSS/Rust contract. A wrapper can affect layout, including Grid cell placement.

Read the [detailed reference](/components/layout-reference) for validation and native integration details. The [playgrounds](/getting-started/playgrounds) include a working Box specimen with its surrounding state and reset controls.
