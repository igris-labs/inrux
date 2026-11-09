# inrux-components

Owned UI descriptions and component composition for Inrux. This package contains no native window or renderer code; applications normally use its exports through `inrux`.

## Build descriptions in Rust

```rust
use inrux_components::Node;
use inrux_style::Style;

/// Return an owned description that can be passed to the native renderer.
fn panel() -> Node {
    Node::column(vec![
        Node::text("Workspace".into()),
        Node::button("Save".into(), None),
    ]).styled(Style::new().padding(24.0).gap(12.0))
}
```

Constructors cover Card, Badge, Separator, Flex, Grid, AspectRatio, Box (`box_container` in Rust), Container, Center, Spacer, Column, Row, Text, Button, Input/TextInput, TextArea, Checkbox, Select, Dialog, Heartbeat and Empty descriptions. Supply Emit<T> callbacks for interaction; generated Events handlers enqueue application functions. Native painting and interaction live in the selected private binding: `inrux-egui` (egui), `inrux-masonry` or `inrux-gpui`. Constructor availability does not imply every engine supports that primitive; adapters diagnose unsupported controls. All three support the shared settings scene, while Select/Dialog/Heartbeat remain egui-only.

Each constructor lives in `src/components/<name>/<name>.rs`. Shared Node, event/model, host, binding, and stylesheet behavior lives in separate modules. [Button](src/components/button/button.rs), [Input](src/components/input/input.rs), and [Dialog](src/components/dialog/dialog.rs) show the convention.

## Reusable SFC content

```xml
<view>
    <Column class="panel"><Children /></Column>
</view>
<rust>
use inrux::prelude::*;
/// The parent supplies the default content as an owned description.
pub struct Props { pub children: Node }
</rust>
<style scoped>
.panel { padding: 20px; gap: 12px; }
</style>
```

The caller imports Panel explicitly and writes `<Panel><Text text="Hello" /></Panel>`. `<Children />` renders the supplied content. `<Slot />` remains an alias. Default content is currently eager; lazy content, named fields/outlets and scoped render functions remain [alpha status](../../docs/STATUS.md).

## Retained state and bindings

A Component supplies `view` and `update`; the compiler can generate the update method for ordinary function handlers. Stateful SFCs declare cloneable Props and State with a `props` field. `ComponentHost::render` retains local state across prop changes and keyed reordering, and disposes mounts omitted from the resulting tree. Native mount creates the host automatically; manual/headless hosts call it explicitly.

`Events::listener` infers typed payloads or allows a handler to ignore them. Custom `model:value` supplies `value` and `on_update_value`. `@update:value` augments that setter atomically. Native Input models also carry editing acknowledgements; plain custom Emit<String> transforms do not yet provide the full revision protocol.

NodeRef-based animation targeting is planned. Refs will address the rendered component root, a keyed child, or a set of descendants without exposing mutable component state or native widgets.

## Check the package

```sh
cargo test -p inrux-components --locked
cargo doc -p inrux-components --no-deps
```

See the [authoring contract](../../docs/AUTHORING.md) and [retained child example](../../playground/egui/src/components/RetainedCounter.iru) for complete usage.

## Layout components

Box is a physical vertical container. Container caps width at 1200 logical pixels and centers within its parent; Center aligns children on both axes; Spacer shares available main-axis space. They compose Column without native-engine dependencies. Defaults yield to global/scoped CSS and explicit styles. See [usage, defaults and sizing limits](../../docs/LAYOUT-COMPONENTS.md), with Rust and `.iru` examples. Each implementation follows the existing per-component directory convention.

Flex, Grid and AspectRatio extend the physical layout group with a changeable axis, bounded uniform grid columns and ratio-based container sizing. They follow the same per-component module convention and shared defaults/style precedence. See [props, CSS/Rust examples and limits](../../docs/LAYOUT-COMPONENTS.md#flex-grid-and-aspectratio). Grid/ratio geometry runs through the selected layout service; it is not an engine-specific component implementation.

## Shared content components

Card, Badge and decorative Separator compose the existing Column/Text primitives once for all engines. Card groups ordered child content; Badge presents plain text; Separator supplies horizontal or explicitly sized vertical geometry. They add no native widget types, implicit model/events or accessibility roles. Classes and explicit styles override defaults. See [usage, defaults, sizing and semantic limits](../../docs/CONTENT-COMPONENTS.md). Each follows the per-component module convention.

## Clipping and scrolling

[ScrollArea and overflow usage](../../docs/SCROLLING.md) covers the shared component, CSS/Rust overflow, UI-owned offsets, typed observations and revision-bearing commands. Native bindings supply rectangular paint/input clipping and native focus/pointer operations; shared policy supplies scroll bounds, nested routing and scrollbar geometry. The Scrolling playground view exercises these paths. Virtualization and complete accessible scroll actions remain separate work.

Input/TextInput are strictly single-line; TextArea provides multiline editing, wrapping and vertical caret scrolling on all three engines. See [text editing](../../docs/TEXT-EDITING.md) for `.iru`, Rust, model and styling usage.

## Indexed styles and runtime themes

Nodes retain a static logical component type, authored classes and an optional style ID separately from keys/refs. Sessions resolve immutable `CompiledStyles` on the coordinator using specificity/source order and viewport predicates. Reuse `.classes(names).with_styles(arc_sheet)` in hot Rust paths; `.class(names, rules)` remains the one-off compatibility helper. `Stylesheet::from_compiled` and `replace_compiled` share indexes across theme toggles. [CSS engine](../../docs/CSS-ENGINE.md) documents precedence, inherited properties and limitations.

`Node::with_defaults(style)` lets reusable presentation libraries supply defaults below global/scoped/explicit styles. The optional [Markdown package](../inrux-markdown/README.md) uses it for portable document blocks. Native component defaults remain engine-independent; plain surfaces are square and Button keeps an explicit 6px default radius.

## Switch, ranges and control styling

Switch, determinate Progress and bounded Meter share portable descriptors and work with all three engines. Inherited `accent-color` and independent `row-gap`/`column-gap` compile into typed styles. See [APIs, styling, validation and accessibility limits](../../docs/BOOLEAN-AND-RANGE-COMPONENTS.md) and [CSS support](../../docs/CSS-ENGINE.md). Native bindings remain private; only the selected engine is required.

## Headless styling ownership

Components own behavior, keyboard/pointer interaction and accessibility, with minimal overridable defaults. Application themes and visual recipes belong to the application; the showcase design stays in the playground. See the [accepted contract](../../docs/HEADLESS-COMPONENTS.md) for current native-part customization gaps and the [CSS guide](../../docs/CSS-ENGINE.md) for implemented styling. Unsupported CSS features remain explicit errors.

## Headless style foundation

[Public parts and owner-state styles](../../docs/STYLING-PARTS.md) now provide overrideable boolean metrics/paint, labels, range fills and editor placeholder ink across all three engines. CSS and Rust share the same resolver and physical margin cascade. [Literal color values](../../docs/STYLE-COLORS.md) include alpha/OKLCH through the pinned color parser, with native alpha preserved. The guide records the unpublished Color tuple-pattern migration and remaining CSS/part limits. Application recipes remain in playground styles.
