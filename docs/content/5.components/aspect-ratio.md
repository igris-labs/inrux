---
title: "AspectRatio"
description: "Size a container around a positive width-to-height ratio."
navigation:
  icon: "tabler:aspect-ratio"
---

Available with **egui, Masonry and GPUI** through the same public component API. Platform input and accessibility coverage still have [alpha limits](/getting-started/status).

## Usage

Replace `src/App.iru` in a [scaffolded project](/getting-started/first-app) with this complete file, then run `cargo run`. Keep the scaffold's `main.rs` and `build.rs`; this file includes its state, handlers and scoped styles.

```xml [src/App.iru]
<view>
    <Column class="example">
        <Checkbox label="Use a widescreen ratio" model:checked={widescreen} />
        <AspectRatio ratio={ratio} width={Size::Px(320.0)} class="sample">
            <Center height={Size::Fill}>
                <Text text={preview_label} width={Size::Content} />
            </Center>
        </AspectRatio>
    </Column>
</view>
<rust>
use inrux::prelude::*;

let widescreen: Signal<bool> = Signal::new(true);
let ratio_widescreen = widescreen.clone();
let ratio: Computed<f32> = Computed::new(move || {
    if ratio_widescreen.get() {
        16.0 / 9.0
    } else {
        1.0
    }
});
let preview_label_widescreen = widescreen.clone();
let preview_label: Computed<&'static str> = Computed::new(move || {
    if preview_label_widescreen.get() {
        "16:9 preview"
    } else {
        "Square preview"
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

| Prop | Type | Default | Description |
| --- | --- | --- | --- |
| `ratio` | `f32` | `1.0` | Finite positive width-to-height ratio; set a width or parent constraint. |

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

AspectRatio participates in container layout. Set a useful width or parent constraint so the ratio can determine the other dimension. Validation rejects invalid ratios before allocation/native translation.

## Styling and limits

CSS aspect-ratio and Rust Style::aspect_ratio use the same physical layout policy. Child constraints still matter; ratio does not imply arbitrary cropping behavior.

Read the [detailed reference](/components/layout-reference#flex-grid-and-aspectratio) for validation and native integration details. The [playgrounds](/getting-started/playgrounds) include a working AspectRatio specimen with its surrounding state and reset controls.
