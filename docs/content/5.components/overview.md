---
title: "Component catalogue"
description: "Choose a portable control or layout primitive and check its current API."
navigation:
  icon: "tabler:components"
---

Components provide behavior and minimal overridable defaults. You supply the application design with CSS-like or Rust styles. Native bindings implement primitives; your application does not import engine widget types.

## Portable catalogue

| Component | Use it to | Engines |
| --- | --- | --- |
| [Button](/components/button) | Trigger an action from a labeled control. | All three |
| [Input](/components/input) | Edit a controlled single-line string. | All three |
| [TextArea](/components/text-area) | Edit multiline text with wrapping and caret scrolling. | All three |
| [Checkbox](/components/checkbox) | Control a labeled boolean choice. | All three |
| [Switch](/components/switch) | Turn a labeled setting on or off. | All three |
| [Progress](/components/progress) | Show a finite amount of completed work. | All three |
| [Meter](/components/meter) | Show a finite measurement within explicit bounds. | All three |
| [Text](/components/text) | Display text with shared shaping and glyph pixels. | All three |
| [Badge](/components/badge) | Display a short status label. | All three |
| [Card](/components/card) | Group related child content in a surface. | All three |
| [Separator](/components/separator) | Draw a decorative divider. | All three |
| [Box](/components/box) | Create a physical vertical container. | All three |
| [Container](/components/container) | Constrain and center a reading or application column. | All three |
| [Row](/components/row) | Arrange child nodes horizontally. | All three |
| [Column](/components/column) | Arrange child nodes vertically. | All three |
| [Center](/components/center) | Center a group within available free space. | All three |
| [Spacer](/components/spacer) | Consume free main-axis space in a Row or Column. | All three |
| [Flex](/components/flex) | Choose a flexible layout direction and wrapping policy. | All three |
| [Grid](/components/grid) | Arrange direct children in equal-width columns. | All three |
| [AspectRatio](/components/aspect-ratio) | Size a container around a positive width-to-height ratio. | All three |
| [ScrollArea](/components/scroll-area) | Clip and scroll bounded child content. | All three |

`TextInput` is an authored alias for Input. All three playgrounds show these 21 component pages with working controls and shared fonts. Each component Usage section contains a complete `src/App.iru` you can paste into the scaffold, including state, handlers and scoped styles. Follow the engine/feature notes for egui-only controls and Markdown.

## Additional components

- [Select](/components/select), [Dialog](/components/dialog) and [Heartbeat](/components/heartbeat) currently require egui.
- [Markdown](/components/markdown) is an optional renderer using shared components; enable the facade's `markdown` feature.
- Custom components use explicit imports, typed Props and Emit values. Read [the component contract](/authoring/component-contract).

Portable availability does not certify full screen-reader, IME or keyboard parity on every OS. Read [alpha status](/getting-started/status), and test your application's target platform.

## Source and appearance

Framework component constructors live in `packages/inrux-components/src/components/`. Application examples and the documentation-style theme live in `playground/<engine>/src/`. Use the exact [CSS subset](/styling/css-reference) and [public parts](/styling/parts-and-states) before assuming arbitrary native internals can be styled.

Each component page groups its contract under **API Reference**, with Props (type, default and description), common styling/identity props, and Emits (event, payload and description). A component with no emitted events says so explicitly.
