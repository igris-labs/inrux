---
title: "Card"
description: "Group related child content in a surface."
navigation:
  icon: "tabler:id"
---

Available with **egui, Masonry and GPUI** through the same public component API. Platform input and accessibility coverage still have [alpha limits](/getting-started/status).

## Usage

Replace `src/App.iru` in a [scaffolded project](/getting-started/first-app) with this complete file, then run `cargo run`. Keep the scaffold's `main.rs` and `build.rs`; this file includes its state, handlers and scoped styles.

```xml [src/App.iru]
<view>
    <Column class="example">
        <Card class="sample">
            <Text text="Workspace activity" />
            <Text text={format!("Actions: {}", count)} />
            <Button text="Record an action" @click={increment} class="action" />
        </Card>
    </Column>
</view>
<rust>
use inrux::prelude::*;

let count: u32 = 0;

/// Demonstrate that controls inside the card retain their normal event wiring.
fn increment(state: &mut State, _: &Context<State>) {
    state.count += 1;
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
.sample {
    padding: 20px;
    gap: 12px;
    background: #f5f5f5;
    border-radius: 8px;
}
</style>
```

## API Reference

### Props

This component has no component-specific props. It accepts these common attributes:

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

Card composes ordinary child Nodes and supplies minimal presentation defaults. Child events, models and identities keep their usual ownership. A Card does not add named header/footer slots or an independent accessibility role.

## Styling and limits

Override background, border, radius, gap and padding with application classes. Avoid styling a whole card as a button unless you also supply correct interaction semantics.

Read the [detailed reference](/components/content-reference) for validation and native integration details. The [playgrounds](/getting-started/playgrounds) include a working Card specimen with its surrounding state and reset controls.
