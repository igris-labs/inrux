---
title: "Center"
description: "Center a group within available free space."
navigation:
  icon: "tabler:align-box-center-middle"
---

Available with **egui, Masonry and GPUI** through the same public component API. Platform input and accessibility coverage still have [alpha limits](/getting-started/status).

## Usage

Replace `src/App.iru` in a [scaffolded project](/getting-started/first-app) with this complete file, then run `cargo run`. Keep the scaffold's `main.rs` and `build.rs`; this file includes its state, handlers and scoped styles.

```xml [src/App.iru]
<view>
    <Column class="example">
        <Checkbox label="Use a taller preview" model:checked={tall} />
        <Center height={Size::Px(preview_height.get().unwrap_or(120.0))} class="sample">
            <Text text="Centered on both axes" width={Size::Content} />
        </Center>
    </Column>
</view>
<rust>
use inrux::prelude::*;

let tall: Signal<bool> = Signal::new(true);
let preview_height_tall = tall.clone();
let preview_height: Computed<f32> = Computed::new(move || {
    if preview_height_tall.get() {
        240.0
    } else {
        120.0
    }
});
</rust>
<style scoped>
.example {
    padding: 24px;
    gap: 16px;
}
.sample {
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

Center aligns the main and cross axes. Vertical centering needs free height. A child with explicitly filled width cannot move horizontally; use automatic or bounded width when that is your intent. Multiple children form one vertical group.

## Styling and limits

Set the containing dimensions and child widths deliberately. Center does not resize an explicitly filled child merely to create empty space.

Read the [detailed reference](/components/layout-reference) for validation and native integration details. The [playgrounds](/getting-started/playgrounds) include a working Center specimen with its surrounding state and reset controls.
