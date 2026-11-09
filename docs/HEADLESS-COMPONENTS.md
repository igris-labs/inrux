# Headless component and styling ownership

Components own behavior; applications own appearance. This guide defines the current ownership boundary and customization gaps.

Inrux is a headless-first component framework. Components supply state and events, binding contracts, keyboard/pointer behavior, focus and accessibility semantics. Users supply the visual design. Minimal usable defaults are allowed, and user styles must be able to override their visual properties without replacing component behavior.

## Ownership

| Location | Owns |
| --- | --- |
| `packages/inrux-components` | Component semantics, controlled props/models/events and composition; minimal overridable defaults |
| `packages/inrux-style` and compiler | Generic CSS/Rust style values, selectors, cascade, scope, validation and compilation |
| Adapter and native bindings | Generic layout/text/paint/input/accessibility translation, fallback metrics and low-level native services; no application design system |
| `playground/<engine>/src/styles` | The showcase's IPEX-inspired tokens, light/dark palette, density, typography, visual variants, surfaces and decorative states |
| Playground components/layouts/views | Example application composition and recipes; shared portable source across engines |
| Consumer application | Its own global/scoped CSS, Rust styles, component recipes and fonts |

“Styling belongs to the playground” means application/theme styling. It does not remove generic style-engine code, native rendering support, minimal defaults, or style test fixtures from framework packages. IPEX-specific names, palettes, easing recipes, button variants and documentation layouts must not become mandatory framework design choices. No styled component package is added by this decision.

## Required customization contract

Every component needs stable public styling parts for its exposed visual structure and semantic states. Typical parts include a root, label/description, Switch track/thumb, Checkbox control/indicator, Input editor/placeholder, range track/indicator/value and overlay/content. The initial exported API is CSS `::part(name)` and Rust `styled_part(Part, Style)`; [supported targets/properties](STYLING-PARTS.md) are explicit. Further parts remain future work.

Part overrides must cover relevant geometry, spacing, typography, colors, borders, focus decoration and motion as those CSS features ship. Native widget handles/types remain private. Visual size must remain separate from minimum interaction target size. A styled thumb or label must not steal activation, break hit testing, duplicate accessible roles or discard keyboard focus.

Public state must be typed and meaningful: checked/unchecked, selected, expanded, invalid, disabled, orientation, loading or range state where the component actually implements that behavior. Style selectors observe state; they do not implement validation, focus trapping, dismissal or action admission. Disabled components remain correctly exposed to assistive technology.

Minimal defaults sit below application global/scoped styles and explicit styles. Theme replacement and part/state styling must preserve controlled values, composition, native identity, focus and caret behavior. A user must not need to fork an adapter to change a component's appearance.

## Current implementation and gaps

Application `.css` files and theme sheets currently live in the playground; framework packages contain no application CSS files. Existing component defaults use the defaults layer and yield to matching global/scoped rules and explicit styles for supported properties.

The prototype is not yet fully headless at the native-part level. The shared adapter still has fixed fallback indicator metrics and palettes; the initial public parts now override indicator metrics, track/thumb/mark/label paint and placeholder ink, while other internal details still need customization APIs. Card/Badge retain basic presentation defaults. Authored owner/part paint-state styles now work in all three engines. An overridden root style does not prove that a user can restyle every internal part.

These are explicit implementation gaps, not permission to move showcase styling into native bindings. Broader part coverage, richer metrics and further CSS capabilities remain foundation work. The [status page](STATUS.md) records remaining gaps. Reference application design choices stay in the playground.
