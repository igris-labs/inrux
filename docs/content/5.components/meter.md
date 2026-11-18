---
title: "Meter"
description: "Show a finite measurement within explicit bounds."
navigation:
  icon: "tabler:gauge"
---

Available with **egui, Masonry and GPUI** through the same public component API. Platform input and accessibility coverage still have [alpha limits](/getting-started/status).

## Usage

Replace `src/App.iru` in a [scaffolded project](/getting-started/first-app) with this complete file, then run `cargo run`. Keep the scaffold's `main.rs` and `build.rs`; this file includes its state, handlers and scoped styles.

```xml [src/App.iru]
<view>
    <Column class="example">
        <Meter label="Temperature" value={temperature} min={-10.0} max={40.0} class="range" />
        <Text text={format!("Temperature: {} C", temperature)} />
        <Row class="actions">
            <Button text="Cool down" @click={cool} />
            <Button text="Warm up" @click={warm} class="action" />
        </Row>
    </Column>
</view>
<rust>
use inrux::prelude::*;

let temperature: f64 = 20.0;

/// Clamp changes to the meter's lower bound.
fn cool(state: &mut State, _: &Context<State>) {
    state.temperature = (state.temperature - 5.0).max(-10.0);
}
/// Clamp changes to the meter's upper bound.
fn warm(state: &mut State, _: &Context<State>) {
    state.temperature = (state.temperature + 5.0).min(40.0);
}
</rust>
<style scoped>
.example {
    padding: 24px;
    gap: 16px;
}
.actions {
    gap: 8px;
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
.range {
    height: 16px;
    accent-color: #b84200;
}
</style>
```

## API Reference

### Props

| Prop | Type | Default | Description |
| --- | --- | --- | --- |
| `label` | `String-like expression` | Required | Nonempty description of the measurement. |
| `value` | `f64` | Required | Finite measurement, normalized to the declared range. |
| `min` | `f64` | Required | Finite lower bound, strictly below `max`. |
| `max` | `f64` | Required | Finite upper bound, strictly above `min`. |

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

Use Meter for a measurement and Progress for completion. Value must lie between min and max. Invalid ranges return a configuration error. This is not a draggable slider.

## Styling and limits

Style the track at the root and the fill through `::part(indicator)`. Borders, size and accent follow the shared style contract.

Read the [detailed reference](/components/range-reference) for validation and native integration details. The [playgrounds](/getting-started/playgrounds) include a working Meter specimen with its surrounding state and reset controls.
