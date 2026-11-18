---
title: "TextArea"
description: "Edit multiline text with wrapping and caret scrolling."
navigation:
  icon: "tabler:align-left"
---

Available with **egui, Masonry and GPUI** through the same public component API. Platform input and accessibility coverage still have [alpha limits](/getting-started/status).

## Usage

Replace `src/App.iru` in a [scaffolded project](/getting-started/first-app) with this complete file, then run `cargo run`. Keep the scaffold's `main.rs` and `build.rs`; this file includes its state, handlers and scoped styles.

```xml [src/App.iru]
<view>
    <Column class="example">
        <Text text="Workspace notes" />
        <TextArea model:value={notes} placeholder="Write several lines" disabled={locked} class="editor" />
        <Checkbox label="Disable the editor" model:checked={locked} />
        <Text text={format!("Lines: {}", notes.lines().count())} />
        <Button text="Reset notes" @click={reset} class="action" />
    </Column>
</view>
<rust>
use inrux::prelude::*;

let notes: String = "First line\nSecond line".into();
let locked: bool = false;

/// Replace the controlled text, including a newline.
fn reset(state: &mut State, _: &Context<State>) {
    state.notes = "First line\nSecond line".into();
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
.editor {
    height: 160px;
    width: 400px;
}
</style>
```

## API Reference

### Props

| Prop | Type | Default | Description |
| --- | --- | --- | --- |
| `value` | `String-like expression` | Required without `model:value` | Controlled multiline value. |
| `model:value` | `retained field: String or Signal<String>` | Not set | Two-way value binding; use instead of `value`. |
| `placeholder` | `String-like expression` | `""` | Hint displayed when empty. |
| `disabled` | `bool` | `false` | Prevent editing. |

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
| `@input` | `String` | New multiline value. Required without a model; a model listener runs after assignment. |

Bind events with `@event={handler}`. Ordinary handlers receive `&mut State` and `&Context<State>`; controlled value handlers can also receive the listed payload. With `model:value` or `model:checked`, the setter runs before the optional model listener, which reads the updated state. Stateless children forward a typed `Emit<T>`.

## Behavior

Enter inserts a newline. TextArea normalizes hard line separators to LF, wraps at the available width and scrolls vertically to reveal the caret. Tab moves focus instead of inserting indentation. There is no public resize handle or complete code-editor feature set.

## Styling and limits

Use a separate textarea class when the single-line Input style has a short fixed height. Root dimensions and placeholder paint are overridable. Undo/redo, multilingual IME and full accessibility parity are not certified.

Read the [detailed reference](/components/text-editing-reference) for validation and native integration details. The [playgrounds](/getting-started/playgrounds) include a working TextArea specimen with its surrounding state and reset controls.
