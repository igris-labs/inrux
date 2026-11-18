---
title: "Switch"
description: "Turn a labeled setting on or off."
navigation:
  icon: "tabler:toggle-left"
---

Available with **egui, Masonry and GPUI** through the same public component API. Platform input and accessibility coverage still have [alpha limits](/getting-started/status).

## Usage

Replace `src/App.iru` in a [scaffolded project](/getting-started/first-app) with this complete file, then run `cargo run`. Keep the scaffold's `main.rs` and `build.rs`; this file includes its state, handlers and scoped styles.

```xml [src/App.iru]
<view>
    <Column class="example">
        <Switch label="Automatic sync" model:checked={enabled} disabled={locked} class="choice" />
        <Checkbox label="Disable the control above" model:checked={locked} />
        <Text text={setting_label} />
        <Button text="Reset" @click={reset} class="action" />
    </Column>
</view>
<rust>
use inrux::prelude::*;

let enabled: Signal<bool> = Signal::new(false);
let setting_label_enabled = enabled.clone();
let setting_label: Computed<&'static str> = Computed::new(move || {
    if setting_label_enabled.get() {
        "Setting: on"
    } else {
        "Setting: off"
    }
});

let locked: bool = false;

/// Restore the value and enabled state together.
fn reset(state: &mut State, _: &Context<State>) {
    state.enabled.set(false);
    state.locked = false;
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
.choice {
    accent-color: #b84200;
}
.choice::part(label) {
    font-size: 16px;
}
</style>
```

## API Reference

### Props

| Prop | Type | Default | Description |
| --- | --- | --- | --- |
| `label` | `String-like expression` | Required | Visible label describing the setting. |
| `checked` | `bool` | Required without `model:checked` | Controlled on/off value. |
| `model:checked` | `retained field: bool or Signal<bool>` | Not set | Two-way checked binding; use instead of `checked`. |
| `disabled` | `bool` | `false` | Reject setting changes. |

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
| `@change` | `bool` | Requested on/off value. Required without a model; a model listener runs after assignment. |

Bind events with `@event={handler}`. Ordinary handlers receive `&mut State` and `&Context<State>`; controlled value handlers can also receive the listed payload. With `model:value` or `model:checked`, the setter runs before the optional model listener, which reads the updated state. Stateless children forward a typed `Emit<T>`.

## Behavior

Switch uses the same controlled boolean contract as Checkbox. Pointer or Enter/Space changes an enabled switch. Keep the source of truth in the parent/component State.

## Styling and limits

The public parts are `track`, `thumb` and `label`. Static dimensions can change track/thumb size; conditional rules are paint-only. The thumb must fit within the track.

Read the [detailed reference](/components/range-reference) for validation and native integration details. The [playgrounds](/getting-started/playgrounds) include a working Switch specimen with its surrounding state and reset controls.
