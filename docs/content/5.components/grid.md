---
title: "Grid"
description: "Arrange direct children in equal-width columns."
navigation:
  icon: "tabler:layout-grid"
---

Available with **egui, Masonry and GPUI** through the same public component API. Platform input and accessibility coverage still have [alpha limits](/getting-started/status).

## Usage

Replace `src/App.iru` in a [scaffolded project](/getting-started/first-app) with this complete file, then run `cargo run`. Keep the scaffold's `main.rs` and `build.rs`; this file includes its state, handlers and scoped styles.

```xml [src/App.iru]
<view>
    <Column class="example">
        <Button text="Change column count" @click={change_columns} class="action" />
        <Text text={format!("Columns: {}", columns)} />
        <Grid columns={columns} gap={12.0}>
            <Text text="One" class="cell" />
            <Text text="Two" class="cell" />
            <Text text="Three" class="cell" />
            <Text text="Four" class="cell" />
            <Text text="Five" class="cell" />
            <Text text="Six" class="cell" />
        </Grid>
    </Column>
</view>
<rust>
use inrux::prelude::*;

let columns: u16 = 2;

/// Stay within Grid's supported positive column-count range.
fn change_columns(state: &mut State, _: &Context<State>) {
    state.columns = if state.columns == 3 {
        1
    } else {
        state.columns + 1
    };
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
.cell {
    padding: 16px;
    background: #f5f5f5;
    border-radius: 6px;
}
</style>
```

## API Reference

### Props

| Prop | Type | Default | Description |
| --- | --- | --- | --- |
| `columns` | `u16` | `2` | Equal-width column count; valid range is 1 through 256. |

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

Grid currently supports equal-width columns with automatic rows. It does not implement browser-complete tracks, spans or placement. If/For wrappers are physical groups, so they can occupy one direct cell rather than distributing their descendants into cells.

## Styling and limits

Use supported repeat(N, 1fr) CSS or the columns prop. For direct repeated cells, construct a Rust Node::grid from a collection. Do not assume CSS subgrid or arbitrary minmax support.

Read the [detailed reference](/components/layout-reference#flex-grid-and-aspectratio) for validation and native integration details. The [playgrounds](/getting-started/playgrounds) include a working Grid specimen with its surrounding state and reset controls.
