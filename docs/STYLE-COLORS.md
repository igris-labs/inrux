# CSS color values

The generic style engine uses the pinned Linebender `color` 0.3.3 library already present in the workspace dependency graph. It parses literal CSS colors during stylesheet compilation/loading and converts them to compact straight-alpha sRGBA8. Native bindings receive bytes; rendering and theme toggles do not parse color strings in compiled playground styles.

```css
:root {
    --foreground: oklch(24.35% 0 0);
    --border: rgb(38 38 38 / 8%);
    --surface: #ffffff;
}
Card { background: var(--surface); border-color: var(--border); }
```

Implemented literal forms include CSS named colors and transparent, short/long RGB/RGBA hex, rgb/rgba, hsl/hsla, hwb, lab/lch, oklab/oklch and the absolute color() spaces supported by the pinned parser. Parsing consumes the entire value and rejects malformed/context-dependent values. Native paint is sRGB-clipped and quantized to eight-bit channels; this is not a promise of wide-gamut or browser gamut-mapping equivalence.

Alpha is preserved for surface/text/border/indicator/placeholder/scrollbar colors at the egui, Masonry and GPUI boundaries. Framed native controls own their background/border paint once, so a translucent fill is not accidentally composited by both wrapper and widget. Derived default button-hover paint preserves alpha. Compositing over actual underlying content remains the renderer's responsibility; the compiler never flattens against a guessed theme background.

Subtree opacity is a separate operation and is not implemented. Gradients, color-mix, relative colors, currentColor, light-dark, contextual/system colors and color-space-preserving interpolation remain unsupported. The contrast heuristic for basic marks remains a fallback, not alpha-aware contrast certification. Use deliberate accessible application palettes.

## Rust construction and migration

```rust
use inrux::{Color, Rgba, Style};
const INK: Color = Color(38, 38, 38); // Existing RGB calls remain valid.
let border = Color::rgba(38, 38, 38, 20);
let style = Style::new().background(Color::parse("oklch(60% 0.1 250 / 50%)")?);
let Rgba(red, green, blue, alpha) = border;
```

Color is now a type alias for the exported four-byte Rgba tuple type, plus a const RGB constructor function named Color. The original strict `Color::hex` still accepts only #rrggbb. `Color::parse` accepts the broader literal subset. Existing RGB constructor and channel-access calls remain valid, but tuple patterns such as `let Color(r,g,b)` must migrate to `let Rgba(r,g,b,a)`. This is an unpublished API representation change. Generated Rust uses RGB calls for opaque values and `Color::rgba` for alpha values, with no runtime parsing.

The IPEX application tokens and visual recipes remain in playground styles; framework components do not gain an IPEX palette dependency.
