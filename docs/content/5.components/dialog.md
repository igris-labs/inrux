---
title: "Dialog"
description: "Show an egui-only modal prototype."
navigation:
  icon: "tabler:message"
---

::warning{title="egui only"}
Masonry and GPUI do not support this primitive. The playgrounds show a support notice; native capability validation reports unsupported usage.
::

## Usage

Replace `src/App.iru` in a [scaffolded project](/getting-started/first-app) with this complete file, then run `cargo run`. Keep the scaffold's `main.rs` and `build.rs`; this file includes its state, handlers and scoped styles. Use an **egui** project for this example.

```xml [src/App.iru]
<view>
    <Column class="example">
        <Button text="Open confirmation" @click={open_dialog} class="action" />
        <Text text={message.clone()} />
        <Dialog open={open} @close={close_dialog}>
            <Column class="dialog-body">
                <Text text="Save this workspace?" />
                <Row class="actions">
                    <Button text="Confirm" @click={confirm} class="action" />
                    <Button text="Cancel" @click={close_dialog} />
                </Row>
            </Column>
        </Dialog>
    </Column>
</view>
<rust>
use inrux::prelude::*;

let open: bool = false;
let message: String = "Nothing saved yet.".into();

/// The component owns whether the dialog is open.
fn open_dialog(state: &mut State, _: &Context<State>) {
    state.open = true;
}
/// Apply a close request from the dialog or its Cancel button.
fn close_dialog(state: &mut State, _: &Context<State>) {
    state.open = false;
}
/// Update local feedback and dismiss after confirmation.
fn confirm(state: &mut State, _: &Context<State>) {
    state.message = "Workspace saved in memory.".into();
    state.open = false;
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
.dialog-body {
    padding: 24px;
    gap: 16px;
    width: 320px;
}
</style>
```

## API Reference

### Props

| Prop | Type | Default | Description |
| --- | --- | --- | --- |
| `open` | `bool` | Required | Owner-controlled visibility; handle `@close` to change it. |

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
| `@close` | `()` | Close request. Required; update the owner's open state in the handler. |

Bind events with `@event={handler}`. Ordinary handlers receive `&mut State` and `&Context<State>`; controlled value handlers can also receive the listed payload. With `model:value` or `model:checked`, the setter runs before the optional model listener, which reads the updated state. Stateless children forward a typed `Emit<T>`.

## Behavior and limits

The owner controls open state and handles the close request. Dialog accepts child content. Portable overlay placement, dismissal/focus ownership and a complete cross-platform modal accessibility contract remain unfinished; do not assume full native dialog parity.

Choose the egui facade feature to use it. Read the [backend support matrix](/getting-started/status) before changing engines. Common style support does not add missing overlay/animation services to another backend.
