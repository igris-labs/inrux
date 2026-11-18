---
title: "Progress"
description: "Show a finite amount of completed work."
navigation:
  icon: "tabler:progress"
---

Available with **egui, Masonry and GPUI** through the same public component API. Platform input and accessibility coverage still have [alpha limits](/getting-started/status).

## Usage

Replace `src/App.iru` in a [scaffolded project](/getting-started/first-app) with this complete file, then run `cargo run`. Keep the scaffold's `main.rs` and `build.rs`; this file includes its state, handlers and scoped styles.

```xml [src/App.iru]
<view>
    <Column class="example">
        <Progress label="Download progress" value={completed} max={100.0} class="range" />
        <Text text={format!("Completed: {}%", completed.get())} />
        <Row class="actions">
            <Button text="Advance" @click={advance} disabled={finished} class="action" />
            <Button text="Reset" @click={reset} />
        </Row>
    </Column>
</view>
<rust>
use inrux::prelude::*;

let completed: Signal<f64> = Signal::new(25.0);
let source = completed.clone();
let finished: Computed<bool> = Computed::new(move || source.get() >= 100.0);

/// Keep progress within its declared finite bounds.
fn advance(state: &mut State, _: &Context<State>) {
    state
        .completed
        .update(|value| *value = (*value + 10.0).min(100.0));
}
/// Start a new progress cycle.
fn reset(state: &mut State, _: &Context<State>) {
    state.completed.set(0.0);
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
| `label` | `String-like expression` | Required | Nonempty description of the task. |
| `value` | `f64` | Required | Finite completion value, normalized to the declared range. |
| `max` | `f64` | Required | Finite positive upper bound; lower bound is zero. |

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

Progress is a readout, not an interactive range input. Values must be finite and within the valid range. An invalid descriptor fails validation before native rendering. Indeterminate progress is not part of this API.

## Styling and limits

The root forms the track; `::part(indicator)` controls fill paint/radius. Use inherited accent-color for a shared control accent. Custom visuals do not establish complete accessible announcement support.

Read the [detailed reference](/components/range-reference) for validation and native integration details. The [playgrounds](/getting-started/playgrounds) include a working Progress specimen with its surrounding state and reset controls.
