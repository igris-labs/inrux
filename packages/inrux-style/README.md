# inrux-style

One style representation shared by CSS compilation, Rust builders, component descriptions, and all three native bindings. The package has no renderer dependency. Most applications import Style and related types from `inrux`.

## CSS and Rust produce the same values

```rust
use inrux_style::{Color, Size, Style, parse_css};

/// Validate a stylesheet and compare its declarations with the equivalent builder.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rules = parse_css(".panel { padding: 24px; gap: 12px; background: #ffffff; width: 100%; }")?;
    let style = Style::new().padding(24.0).gap(12.0)
        .background(Color(255, 255, 255)).width(Size::Percent(100.0));
    assert_eq!(rules[0].style, style);
    Ok(())
}
```

Use Node::styled(style) for explicit Rust overrides, or scoped/global classes in `.iru` files. Style::overlay replaces only properties supplied by the new layer.

## Supported subset

| Area | Values |
| --- | --- |
| Spacing | gap/row-gap/column-gap, uniform padding, physical margin shorthand/longhands and margin-inline: auto or 0 |
| Dimensions | width/height and min/max constraints; px, percent and intrinsic sizing |
| Flex layout | flex-direction, align-items, justify-content, flex-wrap and unitless nonnegative flex-grow |
| Container layout | display: flex or grid; grid-template-columns: repeat(N, 1fr), N in 1..=256; positive numeric or fractional aspect-ratio |
| Paint | background, text color, painted border color/width and radius |
| Typography | font-size and one named font-family |
| Selectors | type, ID, class, universal, compound selectors and lists; optional hover/focus-visible/disabled/checked/unchecked and typed ::part targets |
| Tokens | immutable top-level :root constants and var() fallbacks, compiled before typed parsing |
| Responsive | nested min/max width/height @media conjunctions in logical px |
| Scrollbars | inherited scrollbar-color: thumb track, two literal CSS colors |
| Placement | position: relative/absolute; top/right/bottom/left px, percent or auto |
| Preformatted text | inherited white-space: normal/pre; unwrapped glyph extents feed horizontal overflow |

Literal CSS colors compile to sRGBA8, including alpha and OKLCH. CSS lengths must be finite; physical margins may be negative while other lengths are nonnegative; lengths use px or unitless zero. Unsupported declarations/selectors return StyleError. State selectors accept paint properties only, keeping hover/focus changes from silently altering layout.

Component defaults apply before global and then scoped sheets; specificity/source order resolves matches within a sheet, including interaction layers. Explicit layout/Rust styles win last. Text color, font size, font family and scrollbar colors inherit. A runtime Stylesheet is supplied by inrux-components and tracked by the native coordinator.

## Organization and checks

`types.rs` owns shared values, `style.rs` builders/merging, `values.rs` value parsing, `css.rs` compilation, `selector.rs` compound selectors, `media.rs` predicates, `syntax.rs` structural scanning, `tokens.rs` constant expansion, `compiled.rs` indexing, and `error.rs` diagnostics.

```sh
cargo test -p inrux-style --locked
cargo doc -p inrux-style --no-deps
```

Transitions, transform/opacity animation properties, DOM-inherited custom properties, container queries and richer grid tracks/placement are not implemented. Their animation/ref dependency is recorded in [alpha status](../../docs/STATUS.md); a handle alone will not imply renderer support for every CSS property.

## Planned CSS growth

The [alpha status](../../docs/STATUS.md) records class/ID selectors, specificity and cascade migration, public component parts, box/flex/grid/text/paint properties, tokens, media/container conditions and motion. See [the implemented CSS engine](../../docs/CSS-ENGINE.md) for the current support matrix, build-time standalone compilation, index/cascade APIs, measurements and remaining targets. IDs are distinct from classes, component refs and list keys.

## Application fonts

Register bundled TrueType/OpenType bytes with `WindowOptions.fonts.push(FontAsset::new("Noto Sans", bytes))`. Use the font's intrinsic family name in `Style::font_family("Noto Sans")` or CSS `font-family: "Noto Sans"`. The family inherits through the component tree and is used in measurement and paint. The current CSS subset accepts one named family, without a fallback list. See [shared playground](../../playground/README.md) for a complete licensed-font example.

Style authoring and observable semantics remain in Inrux regardless of engine. The [alpha status](../../docs/STATUS.md) permits private native execution where conformant, with fallbacks for gaps. It does not transfer public styling ownership to an engine or imply that every planned CSS property is implemented.

Display and FlexDirection are exported through the facade/prelude; grid column counts and ratios are validated before the provider allocates geometry. Ratio sizing and Grid display currently require physical containers. See [the layout contract](../../docs/LAYOUT-COMPONENTS.md#flex-grid-and-aspectratio).

## Clipping and scrolling

[ScrollArea and overflow usage](../../docs/SCROLLING.md) covers the shared component, CSS/Rust overflow, UI-owned offsets, typed observations and revision-bearing commands. Native bindings supply rectangular paint/input clipping and native focus/pointer operations; shared policy supplies scroll bounds, nested routing and scrollbar geometry. The Scrolling playground view exercises these paths. Virtualization and complete accessible scroll actions remain separate work.

## Switch, ranges and control styling

Switch, determinate Progress and bounded Meter share portable descriptors and work with all three engines. Inherited `accent-color` and independent `row-gap`/`column-gap` compile into typed styles. See [APIs, styling, validation and accessibility limits](../../docs/BOOLEAN-AND-RANGE-COMPONENTS.md) and [CSS support](../../docs/CSS-ENGINE.md). Native bindings remain private; only the selected engine is required.

## Headless styling ownership

Components own behavior, keyboard/pointer interaction and accessibility, with minimal overridable defaults. Application themes and visual recipes belong to the application; the showcase design stays in the playground. See the [accepted contract](../../docs/HEADLESS-COMPONENTS.md) for current native-part customization gaps and the [CSS guide](../../docs/CSS-ENGINE.md) for implemented styling. Unsupported CSS features remain explicit errors.

## Headless style foundation

[Public parts and owner-state styles](../../docs/STYLING-PARTS.md) now provide overrideable boolean metrics/paint, labels, range fills and editor placeholder ink across all three engines. CSS and Rust share the same resolver and physical margin cascade. [Literal color values](../../docs/STYLE-COLORS.md) include alpha/OKLCH through the pinned color parser, with native alpha preserved. The guide records the unpublished Color tuple-pattern migration and remaining CSS/part limits. Application recipes remain in playground styles.
