---
title: "Badge"
description: "Display a short status label."
navigation:
  icon: "tabler:badge"
---

Available with **egui, Masonry and GPUI** through the same public component API. Platform input and accessibility coverage still have [alpha limits](/getting-started/status).

## Usage

Replace `src/App.iru` in a [scaffolded project](/getting-started/first-app) with this complete file, then run `cargo run`. Keep the scaffold's `main.rs` and `build.rs`; this file includes its state, handlers and scoped styles.

```xml [src/App.iru]
<view>
    <Column class="example">
        <Badge text={status_label} class="status" />
        <Switch label="Activate workspace" model:checked={active} />
    </Column>
</view>
<rust>
use inrux::prelude::*;

let active: Signal<bool> = Signal::new(true);
let status_label_active = active.clone();
let status_label: Computed<&'static str> = Computed::new(move || {
    if status_label_active.get() {
        "Active"
    } else {
        "Paused"
    }
});
</rust>
<style scoped>
.example {
    padding: 24px;
    gap: 16px;
}
.status {
    color: #913400;
    background: #fff0e8;
    padding: 8px;
    border-radius: 12px;
}
</style>
```

## API Reference

### Props

| Prop | Type | Default | Description |
| --- | --- | --- | --- |
| `text` | `String-like expression` | `""` | Badge label. |

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

Badge is a shared presentation wrapper with basic overridable defaults. It is not an interactive action or a separate semantic accessibility role. It does not accept nested children.

## Styling and limits

Apply root color, background, padding and radius styles. Do not rely on color alone to explain state; include a meaningful label.

Read the [detailed reference](/components/content-reference) for validation and native integration details. The [playgrounds](/getting-started/playgrounds) include a working Badge specimen with its surrounding state and reset controls.
