---
title: "Select"
description: "Choose a controlled string value from options."
navigation:
  icon: "tabler:selector"
---

::warning{title="egui only"}
Masonry and GPUI do not support this primitive. The playgrounds show a support notice; native capability validation reports unsupported usage.
::

## Usage

Replace `src/App.iru` in a [scaffolded project](/getting-started/first-app) with this complete file, then run `cargo run`. Keep the scaffold's `main.rs` and `build.rs`; this file includes its state, handlers and scoped styles. Use an **egui** project for this example.

```xml [src/App.iru]
<view>
    <Column class="example">
        <Text text="Performance profile" />
        <Select model:value={profile} options={vec!["Eco".into(), "Balanced".into(), "Fast".into()]} />
        <Text text={format!("Selected: {}", profile)} />
        <Button text="Reset profile" @click={reset} class="action" />
    </Column>
</view>
<rust>
use inrux::prelude::*;

let profile: String = "Balanced".into();

/// Replace the controlled selection with one of the declared options.
fn reset(state: &mut State, _: &Context<State>) {
    state.profile = "Balanced".into();
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

| Prop | Type | Default | Description |
| --- | --- | --- | --- |
| `value` | `String-like expression` | Required without `model:value` | Controlled selected value. |
| `model:value` | `retained field: String or Signal<String>` | Not set | Two-way selected-value binding; use instead of `value`. |
| `options` | `Vec<String>` | Required | Owned list of selectable values. |
| `disabled` | `bool` | `false` | Prevent selection changes. |

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
| `@change` | `String` | Selected value. Required without a model; a model listener runs after assignment. |

Bind events with `@event={handler}`. Ordinary handlers receive `&mut State` and `&Context<State>`; controlled value handlers can also receive the listed payload. With `model:value` or `model:checked`, the setter runs before the optional model listener, which reads the updated state. Stateless children forward a typed `Emit<T>`.

## Behavior and limits

Use a String/Signal<String> model or `value` with `@change`; options is an owned Vec<String>. `disabled` prevents changing the value. Use the egui Engine playground for the current popup and keyboard behavior. Do not treat this prototype as a portable combobox.

Choose the egui facade feature to use it. Read the [backend support matrix](/getting-started/status) before changing engines. Common style support does not add missing overlay/animation services to another backend.
