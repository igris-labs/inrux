---
title: "Button"
description: "Trigger an action from a labeled control."
navigation:
  icon: "tabler:hand-click"
---

Available with **egui, Masonry and GPUI** through the same public component API. Platform input and accessibility coverage still have [alpha limits](/getting-started/status).

## Usage

Replace `src/App.iru` in a [scaffolded project](/getting-started/first-app) with this complete file, then run `cargo run`. Keep the scaffold's `main.rs` and `build.rs`; this file includes its state, handlers and scoped styles.

```xml [src/App.iru]
<view>
    <Column class="example">
        <Text text="Save a change" />
        <Row class="actions">
            <Button text="Save" @click={save} disabled={saved} class="action" />
            <Button text="Reset" @click={reset} />
        </Row>
        <Text text={save_status} />
    </Column>
</view>
<rust>
use inrux::prelude::*;

let saved: Signal<bool> = Signal::new(false);
let save_status_saved = saved.clone();
let save_status: Computed<&'static str> = Computed::new(move || {
    if save_status_saved.get() {
        "Saved in memory."
    } else {
        "Changes are pending."
    }
});

/// Record the action and disable repeated saves.
fn save(state: &mut State, _: &Context<State>) {
    state.saved.set(true);
}
/// Restore the initial example state.
fn reset(state: &mut State, _: &Context<State>) {
    state.saved.set(false);
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
</style>
```

## API Reference

### Props

| Prop | Type | Default | Description |
| --- | --- | --- | --- |
| `text` | `String-like expression` | `""` | Visible button label. |
| `disabled` | `bool` | `false` | Reject activation while disabled. |

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
| `@click` | `()` | Activation request from an enabled button; optional listener. |

Bind events with `@event={handler}`. Ordinary handlers receive `&mut State` and `&Context<State>`; controlled value handlers can also receive the listed payload. With `model:value` or `model:checked`, the setter runs before the optional model listener, which reads the updated state. Stateless children forward a typed `Emit<T>`.

## Behavior

Use a handler such as `fn save(state: &mut State, _: &Context<State>)`. Pointer activation and Enter/Space dispatch an event for an enabled button. Tab navigation uses native focus integration. A disabled button must not execute its action. Mouse selection does not require a keyboard focus outline.

## Styling and limits

Style the root with classes, ID selectors or explicit Style values. Use `:hover`, `:focus-visible` and `:disabled` for the supported paint properties. Keep a visible keyboard-focus indicator.

Read the [detailed reference](/authoring/props-events-models) for validation and native integration details. The [playgrounds](/getting-started/playgrounds) include a working Button specimen with its surrounding state and reset controls.
