# Layout components

Box, Container, Center, Spacer, Flex, Grid and AspectRatio are available through `inrux`, in Rust and `.iru`. They compose the existing Column primitive and work through all three current adapters. Each is a real physical node, not a transparent `<view>` group. Inspectors currently display the lowered Column primitive.

## Defaults and content

| Component | Default behavior | Rust constructor |
| --- | --- | --- |
| Box | Vertical, full available width, zero gap/padding | `Node::box_container(children)` |
| Container | Box capped at 1200 logical pixels, horizontal automatic margins | `Node::container(children)` |
| Center | Box with centered main-axis and cross-axis alignment | `Node::center(children)` |
| Spacer | Childless zero-intrinsic-size Box with `flex-grow: 1` | `Node::spacer()` |

Box uses `box_container` in Rust to avoid confusion with `std::boxed::Box`. Existing Row and Column remain the horizontal and vertical layout primitives. Text, Button, Input and Checkbox compose within these containers using the existing event/model contracts.

Container centers within its parent; use an outer Box for the application surface and gutters. The mounted root establishes the layout origin and has no parent to distribute margins. Padding is included in the constrained width. There are no implicit gutters. Override `max-width` through CSS or `Style::max_width`; `margin-inline: 0` disables its automatic margins.

Center needs free space to center content. Set its height for vertical centering, and give a full-width child `width: auto` or a fixed width if it should center horizontally. Center does not change explicitly filled child dimensions. Multiple children form one centered vertical group; `gap` separates them.

Spacer consumes available parent main-axis space: width inside Row, height inside Column. A vertical Spacer needs a bounded parent height. Multiple spacers share free space according to their growth factors. They do not create extra space when no space remains, although parent gaps still apply. Spacer has no focus target, content, model or events.

## Single-file component

```xml
<view>
    <Box class="surface">
        <Container class="content">
            <Row gap={12.0}>
                <Text text="Workspace" />
                <Spacer />
                <Button text="Save" @click={save} />
            </Row>
            <Center height={Size::Px(100.0)}>
                <Text text="Centered content" />
            </Center>
        </Container>
    </Box>
</view>
<rust>
use inrux::prelude::*;
#[derive(Clone)]
pub struct State;
pub fn setup() -> State { State }
pub fn save(_state: &mut State, _context: &Context<State>) {
    println!("Save requested");
}
</rust>
<style scoped>
.surface { padding: 24px; background: #ffffff; }
.content { max-width: 800px; gap: 16px; }
</style>
```

No application import is required for these built-in tags. Common `class`, `style`, `key`, `gap`, `padding`, `width` and `height` attributes work as on Row/Column. Sizes in Rust expressions use `Size`; CSS uses pixels, percentages or `auto`. Unsupported props or children on Spacer produce source diagnostics.

## Rust composition and styling

```rust
use inrux::prelude::*;

fn panel() -> Node {
    Node::box_container(vec![
        Node::container(vec![
            Node::row(vec![
                Node::text("Workspace".into()),
                Node::spacer(),
                Node::button("Save".into(), None),
            ]).styled(Style::new().gap(12.0)),
        ]).styled(Style::new().max_width(Size::Px(800.0))),
    ]).styled(Style::new().padding(24.0))
}
```

CSS `flex-grow: 2` equals `Style::new().flex_grow(2.0)`. CSS `margin-inline: auto` equals `.margin_inline_auto(true)`; `margin-inline: 0` equals `.margin_inline_auto(false)`. Growth must be finite and nonnegative, including through Rust. The margin subset deliberately supports only auto/zero in the current horizontal writing mode; arbitrary margins and RTL/writing modes are not implemented. These are layout properties and cannot be set in paint-only interaction selectors.

Precedence is component defaults → global classes → scoped classes → matching interaction layers → explicit Rust/template styles. The new defaults layer means a global Container class can replace its width cap. Typography still inherits, children retain their keys/events/mount generations, and all validation runs through the common service boundary.

## Verification and limits

The three independent applications use identical [layout view source](../playground/egui/src/views/Layout.iru) to demonstrate all four components under egui, Masonry and GPUI, with one bundled font and application logic. The [playground catalogue](../playground/README.md) provides interactive specimens. Layout contracts cover narrow/wide and padded containers, both centering axes, filled children, multiple/horizontal/vertical spacers, exhausted space and CSS/Rust overrides. Compiler tests verify public tags and authored diagnostics; application tests preserve editing, events and background work.

These are structural components without new native resource lifecycles or independent accessibility roles. They expose no native widget refs; generation-safe focus/animation refs await their service boundary. Width constraints and existing wrapping are usable now; viewport media queries and ScrollArea are supported; container queries, Stack layering, richer Grid and portable overlays remain separate work. See [component prerequisites](ADAPTER-SERVICES.md#next-component-boundaries) and [alpha status](STATUS.md).

## Flex, Grid and AspectRatio

Flex, Grid and AspectRatio are three additional physical components. Like Box, they compose Column descriptors and keep native engine types out of authoring code.

| Component | Default | Props | Rust |
| --- | --- | --- | --- |
| Flex | Full-width horizontal flow, no wrap | `direction: FlexDirection`, `wrap: bool` | `Node::flex(children)` |
| Grid | Two equal-width columns; automatic rows | `columns: u16` in 1..=256 | `Node::grid(children)` |
| AspectRatio | Square, full width, automatic height | `ratio: f32`, finite and positive | `Node::aspect_ratio(children)` |

Common classes, keys, gap/padding, dimensions and constraints apply. Component defaults remain below application classes; explicit props and Rust styles override them. None of these components owns a model or native focus target. Changing direction or column count updates geometry while preserving child identity and source order. Native input editing and button events retain their own contracts. No reverse-axis or visual order API is supplied.

```xml
<Flex direction={FlexDirection::Column} wrap={true} gap={12.0}>
    <Text text="Flexible content" />
    <Button text="Save" @click={save} />
</Flex>
<Grid columns={3} gap={8.0}>
    <Text text="First cell" />
    <Text text="Second cell" />
    <Text text="Third cell" />
    <Text text="Next row" />
</Grid>
<AspectRatio ratio={16.0 / 9.0} width={Size::Px(320.0)}>
    <Center height={Size::Fill}><Text text="Preview area" /></Center>
</AspectRatio>
```

These are inner view snippets; import `inrux::prelude::*` in the enclosing Rust block for `Size` and `FlexDirection`, and define the Save handler. The component tags require no application component imports.

```rust
use inrux::prelude::*;

fn cells() -> Node {
    Node::grid((1..=6).map(|number| Node::text(number.to_string())).collect())
        .styled(Style::new().grid_columns(3).gap(8.0))
}

fn preview() -> Node {
    Node::aspect_ratio(vec![])
        .styled(Style::new().width(Size::Px(320.0)).aspect_ratio(16.0 / 9.0))
}
```

CSS and Rust use one representation:

| CSS | Rust |
| --- | --- |
| `display: flex` / `display: grid` | `.display(Display::Flex)` / `.display(Display::Grid)` |
| `flex-direction: row` / `column` | `.flex_direction(FlexDirection::Row)` / `Column` |
| `grid-template-columns: repeat(3, 1fr)` | `.grid_columns(3)` |
| `aspect-ratio: 16 / 9` | `.aspect_ratio(16.0 / 9.0)` |

`display` currently accepts flex/grid only. Grid columns accept only bounded integer repeats of 1fr. Tracks have zero minimum sizing under Inrux's current min-width policy, so intrinsic text cannot force unequal columns. Rows size to content and placement follows row-major source order. Unsupported track syntax is an error; weighted/fixed tracks, spans, named areas, dense placement, auto-fit/fill, subgrid and virtualization are not implemented. Too-wide content may overflow a cell; there is no implicit clipping or scrolling.

For repeated data, the Rust `Node::grid(...collect())` example supplies direct cells. The current compiler's `<For>` and `<If>` blocks create Column wrappers, so they contribute a grouped grid child rather than transparent cells. Transparent groups and attribute loops remain separate compiler work. Do not assume Vue's wrapper-free grouping already exists.

Ratios express **width divided by height**. One automatic dimension follows the preferred ratio. Explicit width and height take precedence; min/max constraints can constrain the result and may also limit the opposite dimension. Use `.width(Size::Content).height(Size::Px(100.0)).aspect_ratio(2.0)` for height-driven sizing. Children remain ordinary flow content: a ratio container does not load, fit or crop media, clip overflow, or guarantee that oversized child content fits. Grid display and ratio sizing are currently supported on physical containers, with invalid leaf application diagnosed before layout. Geometry properties are rejected in paint-only interaction selectors.

All three adapters still select the same Taffy provider. A private zero-size Taffy child is used only for empty ratio containers to avoid Taffy 0.9's leaf path overriding explicit/max heights; it adds no framework node, native widget, key, focus target or published child geometry. Custom providers must meet the same sizing/conformance contract. No native-layout substitution or performance improvement is inferred.

The [More layout view](../playground/egui/src/views/LayoutMore.iru), copied into each independent engine app, demonstrates direction changes with a controlled input, two/three columns, square/landscape ratio and normal button interaction. The [playground guide](../playground/README.md) documents running each app.
