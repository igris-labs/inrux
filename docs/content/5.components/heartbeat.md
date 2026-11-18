---
title: "Heartbeat"
description: "Inspect an egui-only UI-thread animation diagnostic."
navigation:
  icon: "tabler:heartbeat"
---

::warning{title="egui only"}
Masonry and GPUI do not support this primitive. The playgrounds show a support notice; native capability validation reports unsupported usage.
::

## Usage

Replace `src/App.iru` in a [scaffolded project](/getting-started/first-app) with this complete file, then run `cargo run`. Keep the scaffold's `main.rs` and `build.rs`; this file includes its state, handlers and scoped styles. Use an **egui** project for this example.

```xml [src/App.iru]
<view>
    <Column class="example">
        <Text text="UI-thread animation diagnostic" />
        <Heartbeat />
        <Button text="Increment counter" @click={increment} class="action" />
        <Text text={format!("Count: {}", count)} />
    </Column>
</view>
<rust>
use inrux::prelude::*;

let count: u32 = 0;

/// Exercise normal events while the native diagnostic animates.
fn increment(state: &mut State, _: &Context<State>) {
    state.count += 1;
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

## Behavior and limits

Heartbeat is an animated diagnostic, not a general animation API or proof of frame-rate performance. It has no children or component-specific props. Use the Engine playground to compare UI activity while coordinator work runs.

Choose the egui facade feature to use it. Read the [backend support matrix](/getting-started/status) before changing engines. Common style support does not add missing overlay/animation services to another backend.
