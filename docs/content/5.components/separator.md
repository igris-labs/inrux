---
title: "Separator"
description: "Draw a decorative divider."
navigation:
  icon: "tabler:separator"
---

Available with **egui, Masonry and GPUI** through the same public component API. Platform input and accessibility coverage still have [alpha limits](/getting-started/status).

## Usage

Replace `src/App.iru` in a [scaffolded project](/getting-started/first-app) with this complete file, then run `cargo run`. Keep the scaffold's `main.rs` and `build.rs`; this file includes its state, handlers and scoped styles.

```xml [src/App.iru]
<view>
    <Column class="example">
        <Checkbox label="Use a vertical divider" model:checked={vertical} />
        <Flex direction={direction} gap={16.0} class="sample">
            <Text text="Before" width={Size::Content} />
            <Separator vertical={vertical} width={divider_width} height={divider_height} />
            <Text text="After" width={Size::Content} />
        </Flex>
    </Column>
</view>
<rust>
use inrux::prelude::*;

let vertical: Signal<bool> = Signal::new(false);
let direction_vertical = vertical.clone();
let direction: Computed<FlexDirection> = Computed::new(move || {
    if direction_vertical.get() {
        FlexDirection::Row
    } else {
        FlexDirection::Column
    }
});
let divider_width_vertical = vertical.clone();
let divider_width: Computed<Size> = Computed::new(move || {
    if divider_width_vertical.get() {
        Size::Px(1.0)
    } else {
        Size::Fill
    }
});
let divider_height_vertical = vertical.clone();
let divider_height: Computed<Size> = Computed::new(move || {
    if divider_height_vertical.get() {
        Size::Px(48.0)
    } else {
        Size::Px(1.0)
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

| Prop | Type | Default | Description |
| --- | --- | --- | --- |
| `vertical` | `bool` | `false` | Choose a vertical divider instead of a horizontal divider. |

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

## Behavior

Separator is decorative, childless and not focusable. Use surrounding layout constraints to provide the required cross-axis length. It does not create a content heading or landmark.

## Styling and limits

Change line color and dimensions through the documented root styling API. Decorative separation should not be the only explanation of a content relationship.

Read the [detailed reference](/components/content-reference) for validation and native integration details. The [playgrounds](/getting-started/playgrounds) include a working Separator specimen with its surrounding state and reset controls.
