# Headless component styling

Public parts expose appearance while native bindings keep input, focus, controlled models and accessibility behavior. The same part data and geometry policy are used by egui, Masonry and GPUI. Application recipes belong to the playground or consumer application, not framework packages.

## CSS and Rust

```css
Switch.choice::part(track) {
    width: 42px;
    height: 24px;
    border-radius: 12px;
    background: #dedede;
    border-width: 1px;
    border-color: rgb(0 0 0 / 15%);
}
Switch.choice::part(thumb) {
    width: 18px;
    height: 18px;
    border-radius: 9px;
    background: white;
}
Switch.choice::part(track):checked { background: oklch(55% 0.15 257); }
Switch.choice::part(track):hover { border-color: black; }
Switch.choice::part(track):focus-visible { border-color: black; border-width: 2px; }
Switch.choice::part(track):disabled { background: #aaa; }
Switch.choice::part(label) { font-size: 14px; color: #262626; }
Input::part(placeholder) { color: rgb(38 38 38 / 55%); }
```

Attach the class to an ordinary controlled Switch. No new component handler or backend import is required. In Rust:

```rust
use inrux::prelude::*;
let control = Node::switch("Automatic sync".into(), enabled, event)
    .classes("choice")
    .styled_part(Part::Track, Style::new().width(Size::Px(42.0)).height(Size::Px(24.0)))
    .styled_part(Part::Thumb, Style::new().width(Size::Px(18.0)).height(Size::Px(18.0)));
```

`styled_part` is an explicit override above matching global/scoped part rules, independent of `styled`, which targets the root. Parts match the owner's type/ID/classes and use existing indexing, specificity, source order and media predicates. Root and part declarations have separate cascades: a track background never becomes a full-row background. Part declarations do not implicitly inherit other part declarations. Labels inherit the owner's resolved typography/ink before their own overrides.

## Implemented targets

| Component | Part | Supported properties |
| --- | --- | --- |
| Checkbox | control | width, height, background/color of the surface via background, border-color, border-width, border-radius |
| Checkbox | indicator | color or background supplies checkmark ink |
| Switch | track | width, height, background, border-color, border-width, border-radius |
| Switch | thumb | width, height, background, border-radius |
| Checkbox/Switch | label | color, font-size, one registered font-family |
| Input/TextInput/TextArea | placeholder | color |
| Progress/Meter | indicator | background or color supplies fill ink, border-radius |

The range root is its track: use ordinary root styles for background, height, width and radius. Label geometry and boolean-control sizes participate in shared intrinsic measurement and native painting. Track/control and thumb dimensions require finite positive logical px; the thumb must fit inside the track. Border/radius/font metrics must be finite and nonnegative. Unknown part names fail parsing; incompatible part targets/properties fail adapter validation before native translation. Placeholder metrics, thumb borders, arbitrary child parts and public native-widget handles are not supported.

## Owner states

One trailing condition is supported: `:hover`, `:focus-visible`, `:disabled`, `:checked` or `:unchecked`. Checked/unchecked apply to boolean controls. These conditions refer to the owning component, including when used after `::part(...)`; they are not independent pointer/focus regions on the thumb or label. Compound conditions, selected/invalid/expanded state selectors and arbitrary data/ARIA selectors remain unsupported.

Conditional rules can change only color, background, border color/width and radius. Layout/font changes are rejected in conditional rules, including rules built directly in Rust. Static part metrics can still be changed by publishing a new style from application state. Root authored paint states now work in all three engines, using native hover and focus events. Keyboard-visible focus is separate from pointer focus. Authored root/control focus borders replace the fallback focus outline; default feedback remains available when no border was authored. Editors retain native caret/focus behavior.

No animation loop is added. GPUI retains hover state only for owners whose authored paint depends on it, and sweeps disposed IDs. Masonry updates existing native properties; egui resolves paint during its normal frame. Backend paint snapshots exclude descendant subtrees and compiled sheet ownership. General style/layout invalidation is still coarse; this does not claim a complete incremental style cache. Dynamic inherited state colors do not propagate through arbitrary descendant trees; style a public label part or the actual child when needed.

## Physical margins

CSS supports `margin` with one to four logical px values and `margin-top/right/bottom/left`, including negative values. Rust has `.margin(value)` and physical side setters. `margin-inline:auto`/`0` remains available; a later inline declaration resets both horizontal sides, a later longhand overrides its side, and a later shorthand resets all sides. Source order and specificity apply normally. Generated Rust preserves that order.

Margins are shared flex/grid layout inputs; there is no browser block-margin collapse. Percent/rem/em margins, `margin: 4px auto` and general logical-direction margins remain unsupported. Uniform padding remains the current padding subset. All engines consume the same computed child geometry.

See [color values and migration](STYLE-COLORS.md), [current CSS support](CSS-ENGINE.md), and the [IPEX requirements audit](STATUS.md).
