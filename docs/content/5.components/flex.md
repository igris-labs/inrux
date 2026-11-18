---
title: "Flex"
description: "Choose a flexible layout direction and wrapping policy."
navigation:
  icon: "tabler:layout-distribute-horizontal"
---

Available with **egui, Masonry and GPUI** through the same public component API. Platform input and accessibility coverage still have [alpha limits](/getting-started/status).

## Usage

Replace `src/App.iru` in a [scaffolded project](/getting-started/first-app) with this complete file, then run `cargo run`. Keep the scaffold's `main.rs` and `build.rs`; this file includes its state, handlers and scoped styles.

```xml [src/App.iru]
<view>
    <Column class="example">
        <Checkbox label="Allow wrapping" model:checked={wrap} />
        <Checkbox label="Use column direction" model:checked={vertical} />
        <Flex direction={direction} wrap={wrap} style={Style::new().max_width(Size::Px(360.0))} class="sample">
            <Button text="First" @click={increment} width={Size::Px(120.0)} />
            <Button text="Second" @click={increment} width={Size::Px(120.0)} />
            <Button text="Third" @click={increment} width={Size::Px(120.0)} />
        </Flex>
        <Text text={format!("Clicks: {}", count)} />
    </Column>
</view>
<rust>
use inrux::prelude::*;

let vertical: Signal<bool> = Signal::new(false);
let direction_vertical = vertical.clone();
let direction: Computed<FlexDirection> = Computed::new(move || {
    if direction_vertical.get() {
        FlexDirection::Column
    } else {
        FlexDirection::Row
    }
});

let wrap: bool = true;
let count: u32 = 0;

/// Keep the action independent of the current flex direction.
fn increment(state: &mut State, _: &Context<State>) {
    state.count += 1;
}
</rust>
<style scoped>
.example {
    padding: 24px;
    gap: 16px;
}
.sample {
    gap: 12px;
    padding: 16px;
    background: #f5f5f5;
}
</style>
```

## API Reference

### Props

| Prop | Type | Default | Description |
| --- | --- | --- | --- |
| `direction` | `FlexDirection` | `FlexDirection::Row` | Main-axis direction. |
| `wrap` | `bool` | `false` | Allow children to wrap. |

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

Flex defaults to horizontal flow without wrap. Changing direction updates geometry while retaining child identity and source order. Reverse axes and visual order APIs are not supplied.

## Styling and limits

CSS display/flex-direction/flex-wrap and the typed Rust representation follow the same layout policy. Check the exact subset before using browser flex shorthand.

Read the [detailed reference](/components/layout-reference#flex-grid-and-aspectratio) for validation and native integration details. The [playgrounds](/getting-started/playgrounds) include a working Flex specimen with its surrounding state and reset controls.
