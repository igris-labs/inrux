---
title: "Text"
description: "Display text with shared shaping and glyph pixels."
navigation:
  icon: "tabler:typography"
---

Available with **egui, Masonry and GPUI** through the same public component API. Platform input and accessibility coverage still have [alpha limits](/getting-started/status).

## Usage

Replace `src/App.iru` in a [scaffolded project](/getting-started/first-app) with this complete file, then run `cargo run`. Keep the scaffold's `main.rs` and `build.rs`; this file includes its state, handlers and scoped styles.

```xml [src/App.iru]
<view>
    <Column class="example">
        <Text text="Enter a name to update the greeting." />
        <Input model:value={name} placeholder="Your name" />
        <Text text={format!("Hello, {}!", name)} class="greeting" />
    </Column>
</view>
<rust>
use inrux::prelude::*;

let name: String = "Rust developer".into();
</rust>
<style scoped>
.example {
    padding: 24px;
    gap: 16px;
}
.greeting {
    font-size: 24px;
    color: #b84200;
}
</style>
```

## API Reference

### Props

| Prop | Type | Default | Description |
| --- | --- | --- | --- |
| `text` | `String-like expression` | `""` | Text to display. |

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

Text uses the common Parley/Swash provider. It inherits text color, size, named family and white-space policy. Normal text wraps to the available width; preformatted text preserves unwrapped geometry. A decorative Text node has no action handler.

## Styling and limits

Register fonts before mounting. Font weight, line-height and tracking are not browser-complete CSS features in the alpha. System fallback coverage and native compositor output still need platform checks.

Read the [detailed reference](/internals/text-rendering) for validation and native integration details. The [playgrounds](/getting-started/playgrounds) include a working Text specimen with its surrounding state and reset controls.
