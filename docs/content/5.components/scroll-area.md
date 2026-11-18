---
title: "ScrollArea"
description: "Clip and scroll bounded child content."
navigation:
  icon: "tabler:arrows-vertical"
---

Available with **egui, Masonry and GPUI** through the same public component API. Platform input and accessibility coverage still have [alpha limits](/getting-started/status).

## Usage

Replace `src/App.iru` in a [scaffolded project](/getting-started/first-app) with this complete file, then run `cargo run`. Keep the scaffold's `main.rs` and `build.rs`; this file includes its state, handlers and scoped styles.

```xml [src/App.iru]
<view>
    <Column class="example">
        <Row class="actions">
            <Button text="Back to top" @click={top} class="action" />
            <Button text="Jump down" @click={bottom} />
        </Row>
        <Text text={format!("Vertical offset: {:.0}", offset)} />
        <ScrollArea height={Size::Px(240.0)} scroll_to={request} @scroll={observe_scroll} class="sample">
            <Input model:value={draft} placeholder="A draft stays editable while scrolling" />
            <For each={&messages} item="message" key={message.id}>
                <Text text={message.text.clone()} class="message" />
            </For>
        </ScrollArea>
    </Column>
</view>
<rust>
use inrux::prelude::*;

/// Stable keys identify the repeated rows.
#[derive(Clone)]
pub struct Message {
    id: u32,
    text: String,
}

let draft: String = String::new();
let messages: Vec<Message> = (1..=30)
    .map(|id| Message {
        id,
        text: format!("Message {id}"),
    })
    .collect();
let request: ScrollRequest = ScrollRequest {
    revision: 0,
    offset: ScrollOffset::default(),
};
let offset: f32 = 0.0;

/// A new revision issues a command without controlling later wheel movement.
fn top(state: &mut State, _: &Context<State>) {
    state.request.revision += 1;
    state.request.offset = ScrollOffset::default();
}
/// The viewport clamps an oversized requested offset to its content bounds.
fn bottom(state: &mut State, _: &Context<State>) {
    state.request.revision += 1;
    state.request.offset = ScrollOffset {
        x: 0.0,
        y: 10_000.0,
    };
}
/// Observe scrolling through the typed coordinator event path.
fn observe_scroll(state: &mut State, metrics: ScrollMetrics, _: &Context<State>) {
    state.offset = metrics.offset.y;
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
.sample {
    padding: 12px;
    gap: 8px;
    background: #f5f5f5;
}
.message {
    padding: 8px;
}
</style>
```

## API Reference

### Props

| Prop | Type | Default | Description |
| --- | --- | --- | --- |
| `scroll_to` | `ScrollRequest` | Not set | Programmatic offset request; increment its revision for each new command. |

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
| `@scroll` | `ScrollMetrics` | Viewport size, content size and current offset; optional typed listener. |

Bind events with `@event={handler}`. Ordinary handlers receive `&mut State` and `&Context<State>`; controlled value handlers can also receive the listed payload. With `model:value` or `model:checked`, the setter runs before the optional model listener, which reads the updated state. Stateless children forward a typed `Emit<T>`.

## Behavior

ScrollArea defaults to full width, 240 logical pixels high, vertical auto and horizontal hidden. UI-owned offsets preserve native wheel responsiveness and system-adjusted direction. Nested routing, bounds and scrollbar geometry share policy across engines.

## Styling and limits

Auto indicators hide after idle and reveal on edge interaction. overflow:scroll keeps them visible. Application padding belongs inside the viewport when the track should stay at the outer boundary. Input/TextArea manage their own editing viewports.

Read the [detailed reference](/components/scrolling-reference) for validation and native integration details. The [playgrounds](/getting-started/playgrounds) include a working ScrollArea specimen with its surrounding state and reset controls.
