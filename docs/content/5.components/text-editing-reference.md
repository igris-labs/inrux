---
title: "Text editing reference"
description: "Choose single-line Input or multiline TextArea and bind controlled text."
navigation:
  icon: "tabler:text-wrap"
---

`Input` (also spelled `TextInput`) edits one line. `TextArea` edits multiple lines with native wrapping and vertical scrolling. Both work in egui, Masonry and GPUI and use the same controlled-model acknowledgements, styling, placeholder and disabled contract.

## Usage

Replace `src/App.iru` in a [scaffolded project](/getting-started/first-app) with this complete file, then run `cargo run`. Keep the scaffold's `main.rs` and `build.rs`; the example includes its state, handlers and scoped styles.

```xml [src/App.iru]
<view>
    <Column class="example">
        <Text text="Workspace name" />
        <TextInput model:value={name} placeholder="Name" disabled={locked} class="input" />
        <Text text="Notes" />
        <TextArea model:value={notes} placeholder="Notes" disabled={locked} class="notes" />
        <Checkbox label="Disable both editors" model:checked={locked} />
        <Text text={format!("Name: {} | Note lines: {}", name, notes.lines().count())} />
        <Button text="Reset editors" @click={reset} class="action" />
    </Column>
</view>
<rust>
use inrux::prelude::*;

let name: String = "Inrux Workspace".into();
let notes: String = "First line\nSecond line".into();
let locked: bool = false;

/// External value replacement goes through the normal controlled-model path.
fn reset(state: &mut State, _: &Context<State>) {
    state.name = "Inrux Workspace".into();
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
.input {
    width: 320px;
}
.notes {
    width: 400px;
    height: 160px;
}
</style>
```

## API Reference

### Props

| Component | Prop | Type | Default | Description |
| --- | --- | --- | --- | --- |
| Input / TextInput | `value` | `String-like expression` | Required without `model:value` | Controlled single-line value. |
| Input / TextInput | `model:value` | `retained field: String or Signal<String>` | Not set | Two-way value binding; use instead of `value`. |
| Input / TextInput | `placeholder` | `String-like expression` | `""` | Hint displayed when empty. |
| Input / TextInput | `disabled` | `bool` | `false` | Prevent editing. |
| TextArea | `value` | `String-like expression` | Required without `model:value` | Controlled multiline value. |
| TextArea | `model:value` | `retained field: String or Signal<String>` | Not set | Two-way value binding; use instead of `value`. |
| TextArea | `placeholder` | `String-like expression` | `""` | Hint displayed when empty. |
| TextArea | `disabled` | `bool` | `false` | Prevent editing. |

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
| Input / TextInput | `@input` | `String` | New single-line value. Required without a model; a model listener runs after assignment. |
| TextArea | `@input` | `String` | New multiline value. Required without a model; a model listener runs after assignment. |

Bind events with `@event={handler}`. Ordinary handlers receive `&mut State` and `&Context<State>`; controlled value handlers can also receive the listed payload. With `model:value` or `model:checked`, the setter runs before the optional model listener, which reads the updated state. Stateless children forward a typed `Emit<T>`.

Use `value` and `@input` for an explicitly controlled string, or `model:value` with an optional `@input` handler. Native edits stay on the UI thread; application handlers run through the existing worker/event channel. External resets preserve the edit-revision protocol. `disabled` rejects text and releases focus.

Rust constructors are `Node::input` / `Node::text_input` and `Node::text_area`, with `Node::model_input` / `Node::model_text_area` for revision-bearing events. Each accepts value, placeholder and event. The public implementations are in `components/input/input.rs` and `components/text_area/text_area.rs`. Both lower to the same backend-independent editing descriptor with an explicit multiline mode; native integration stays private.

Input converts hard line breaks to spaces in supplied values, clipboard and IME text. CRLF becomes one space. TextArea normalizes hard line separators to LF and preserves paragraphs; Enter inserts a newline. Tab moves focus rather than inserting indentation. TextArea wraps to its content width and scrolls vertically to reveal the active caret. Native selections, clipboard and IME geometry use the same translation as glyph painting. There is no public textarea resize handle in this increment.

Default allocated heights are 40 logical pixels for Input and 120 for TextArea; default intrinsic width is 220. Set dimensions with ordinary classes/Rust styles. A stylesheet class that sets Input's height also sets TextArea's height if applied to it, so use a separate textarea class when a taller editor is desired. Overflow properties remain container-only; editor scrolling is internal.

The Scrolling view in each playground includes an identical multiline model demo, line/character counts and disabled-state toggle. Existing single-line examples remain in Controls and Scrolling.

Masonry and GPUI retain native editing and IME connections while using the common shaping, selection geometry and glyph provider. egui uses its native editor metadata with the same shared text pixels. Full cross-platform accessibility/IME certification, undo/redo parity and advanced editor features remain outside the current prototype's validated scope.
