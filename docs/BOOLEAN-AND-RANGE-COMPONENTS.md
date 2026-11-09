# Switch, Progress and Meter

These shared components work with egui, Masonry and GPUI. Their constructors live in `inrux-components/src/components/<name>/<name>.rs`; private engine bindings handle native input, painting and available accessibility integration. Applications use the same facade API with every engine.

## Authoring

```xml
<view>
    <Column>
        <Switch label="Automatic sync" model:checked={state.sync} />
        <Progress label="Download progress" value={state.completed} max={100.0} />
        <Meter label="Temperature" value={state.temperature} min={-10.0} max={40.0} />
    </Column>
</view>
<rust>
use inrux::prelude::*;
#[derive(Clone, Default)]
pub struct State {
    pub sync: bool,
    pub completed: f64,
    pub temperature: f64,
}
</rust>
```

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
Progress, Meter {
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

[Public parts](STYLING-PARTS.md) allow track/thumb/label/indicator styling without replacing native behavior. Owner :hover/:focus-visible/:disabled/:checked/:unchecked paint rules are available across all three engines. The playground's 42×24 track is application styling, not a new framework default.
