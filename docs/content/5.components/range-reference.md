---
title: "Boolean and range reference"
description: "Use controlled switches and validated progress and measurement ranges."
navigation:
  icon: "tabler:adjustments-horizontal"
---

These shared components work with egui, Masonry and GPUI. Their constructors live in `inrux-components/src/components/<name>/<name>.rs`; private engine bindings handle native input, painting and available accessibility integration. Applications use the same facade API with every engine.

## Usage

Replace `src/App.iru` in a [scaffolded project](/getting-started/first-app) with this complete file, then run `cargo run`. Keep the scaffold's `main.rs` and `build.rs`; the example includes its state, handlers and scoped styles.

```xml [src/App.iru]
<view>
    <Column class="example">
        <Switch label="Automatic sync" model:checked={sync} />
        <Progress label="Download progress" value={completed} max={100.0} class="range" />
        <Meter label="Temperature" value={temperature} min={-10.0} max={40.0} class="range" />
        <Text text={format!("Sync: {} | Progress: {}% | Temperature: {} C", sync, completed, temperature)} />
        <Row class="actions">
            <Button text="Advance values" @click={advance} class="action" />
            <Button text="Reset" @click={reset} />
        </Row>
    </Column>
</view>
<rust>
use inrux::prelude::*;

let sync: bool = false;
let completed: f64 = 0.0;
let temperature: f64 = 20.0;

/// Keep values inside the finite ranges declared above.
fn advance(state: &mut State, _: &Context<State>) {
    state.completed = (state.completed + 10.0).min(100.0);
    state.temperature = (state.temperature + 2.0).min(40.0);
}
/// Restore the complete local state.
fn reset(state: &mut State, _: &Context<State>) {
    state.sync = false;
    state.completed = 0.0;
    state.temperature = 20.0;
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

| Component | Prop | Type | Default | Description |
| --- | --- | --- | --- | --- |
| Switch | `label` | `String-like expression` | Required | Visible label describing the setting. |
| Switch | `checked` | `bool` | Required without `model:checked` | Controlled on/off value. |
| Switch | `model:checked` | `retained field: bool or Signal<bool>` | Not set | Two-way checked binding; use instead of `checked`. |
| Switch | `disabled` | `bool` | `false` | Reject setting changes. |
| Progress | `label` | `String-like expression` | Required | Nonempty description of the task. |
| Progress | `value` | `f64` | Required | Finite completion value, normalized to the declared range. |
| Progress | `max` | `f64` | Required | Finite positive upper bound; lower bound is zero. |
| Meter | `label` | `String-like expression` | Required | Nonempty description of the measurement. |
| Meter | `value` | `f64` | Required | Finite measurement, normalized to the declared range. |
| Meter | `min` | `f64` | Required | Finite lower bound, strictly below `max`. |
| Meter | `max` | `f64` | Required | Finite upper bound, strictly above `min`. |

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

| Component | Event | Payload | Description |
| --- | --- | --- | --- |
| Switch | `@change` | `bool` | Requested on/off value. Required without a model; a model listener runs after assignment. |

Bind events with `@event={handler}`. Ordinary handlers receive `&mut State` and `&Context<State>`; controlled value handlers can also receive the listed payload. With `model:value` or `model:checked`, the setter runs before the optional model listener, which reads the updated state. Stateless children forward a typed `Emit<T>`.

Switch also accepts `checked={bool}` and `@change={handler}` instead of a model, like Checkbox. Models accept plain boolean fields or `Signal<bool>`. Pointer and Enter/Space activate enabled switches; disabled controls reject activation. State remains controlled by application updates. The label must be nonempty.

Rust constructors:

```rust
let switch = Node::switch("Automatic sync".into(), enabled, events.model(|state, value, _| state.sync = value));
let progress = Node::progress("Download progress".into(), completed, 100.0);
let meter = Node::meter("Temperature".into(), temperature, -10.0, 40.0);
```

Progress reports completion from zero to a positive maximum. Meter reports a measurement between explicit minimum and maximum. Values and bounds are `f64`; they must be finite, strictly ordered, and contain the value. An empty label, invalid range, or overflow in the interval produces an adapter validation error before native rendering. Values are not silently clamped. Range indicators are read-only and do not join keyboard focus traversal.

## Styling

```css
.controls {
    accent-color: #705bea;
    gap: 16px 12px; /* row, column */
}
Progress,
Meter {
    height: 8px;
    background: #e4ebe7;
    border-radius: 4px;
}
```

`accent-color` inherits through the component tree and colors Checkbox selection, Switch tracks and range fills. A common contrast heuristic chooses the indicator mark/thumb color. It is not a contrast certification; choose accessible theme palettes. Rust parity is `Style::new().accent_color(Color(112, 91, 234)).row_gap(16.0).column_gap(12.0)`.

Range defaults fill the available width at 8 logical pixels high with 4px rounding. Switch defaults to a 34×20 logical pixel track, overridable through public parts; labels inherit typography. Geometry remains shared across engines. CSS selector parsing and numeric range validation do not run on a frame animation timer; these controls do not create one.

## Current limits and validation

Progress is determinate only. Meter has no low/high/optimum bands or editable slider behavior. These are separate semantics, not styles inferred from component names. egui and Masonry publish Switch/Meter/Progress roles and numeric range data through AccessKit. GPUI retains the existing limitation in platform accessibility delivery; shared labels and descriptors do not imply an OS accessibility bridge.

The three catalogue apps include controlled Switch, bounded increase/decrease, reset and inherited violet-accent examples. Tests cover generated `.iru` type checking, native input and disabled behavior, native range semantics, range validation, shared geometry and CSS cascade/source order. No new startup, memory or frame-time performance claim is made by this increment.

[Public parts](/styling/parts-and-states) allow track/thumb/label/indicator styling without replacing native behavior. Owner :hover/:focus-visible/:disabled/:checked/:unchecked paint rules are available across all three engines. The playground's 42×24 track is application styling, not a new framework default.
