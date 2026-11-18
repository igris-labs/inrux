---
title: "Row"
description: "Arrange child nodes horizontally."
navigation:
  icon: "tabler:columns"
---

Available with **egui, Masonry and GPUI** through the same public component API. Platform input and accessibility coverage still have [alpha limits](/getting-started/status).

## Usage

Replace `src/App.iru` in a [scaffolded project](/getting-started/first-app) with this complete file, then run `cargo run`. Keep the scaffold's `main.rs` and `build.rs`; this file includes its state, handlers and scoped styles.

```xml [src/App.iru]
<view>
    <Column class="example">
        <Checkbox label="Swap the labels" model:checked={swap} />
        <Row gap={16.0} class="sample">
            <Text text={first_label} width={Size::Content} />
            <Spacer />
            <Text text={second_label} width={Size::Content} />
        </Row>
    </Column>
</view>
<rust>
use inrux::prelude::*;

let swap: Signal<bool> = Signal::new(false);
let first_label_swap = swap.clone();
let first_label: Computed<&'static str> = Computed::new(move || {
    if first_label_swap.get() {
        "Second"
    } else {
        "First"
    }
});
let second_label_swap = swap.clone();
let second_label: Computed<&'static str> = Computed::new(move || {
    if second_label_swap.get() {
        "First"
    } else {
        "Second"
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

Row follows the shared Taffy layout policy. A Spacer consumes free main-axis width. Children keep their source order, keys and native interaction contracts.

## Styling and limits

Use gap, sizing, alignment and optional Rust wrap styles. Horizontal layout alone does not guarantee that content fits a narrow window; test wrapping and min/max constraints.

Read the [detailed reference](/components/layout-reference) for validation and native integration details. The [playgrounds](/getting-started/playgrounds) include a working Row specimen with its surrounding state and reset controls.
