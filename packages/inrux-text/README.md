# inrux-text

Engine-independent text shaping, cluster geometry and glyph rasterization for Inrux. Parley shapes text; Swash renders its glyphs into the same straight-alpha sRGBA tiles for egui, Masonry and GPUI. Each binding uploads those tiles through its native image API. This crate depends on no GUI engine.

Applications normally use `inrux::FontAsset` and CSS/Rust `font_family` / `font_size`. Window startup registers those assets before the first frame. A bundled OFL-licensed Noto Sans Regular face provides the same default across bindings; the font license is in `assets/OFL.txt`. Registered families use their intrinsic font-family names. Application design-system styling remains in playground.

## Low-level usage

```rust
let text = inrux_text::shape("Rust café", 15.0, None, 240.0);
let logical_size = (text.width(), text.height());
let caret = text.caret("Rust ".len());
let pixels = inrux_text::rasterize(&text, 2.0, [27, 30, 33, 255]);
assert!(!pixels.is_empty());
```

Widths, line breaks and caret/selection positions are logical, unquantized coordinates. Device scale enters only glyph rasterization and texture placement. Default line height comes from `inrux-style::DEFAULT_LINE_HEIGHT`. `rasterize_region` limits long editors to their visible text region; complete text is tiled rather than truncated at a GPU texture limit. Tile offsets retain bearings and glyph overhang. Empty/transparent glyph regions produce no upload. Fallback and color-glyph rendering use the same provider for all bindings; missing-font coverage still depends on the registered/system fonts on that machine.

## Modules and ownership

- `shape.rs`: immutable shaping results, exact request keys, UTF-8 cluster hit testing and logical metrics.
- `geometry.rs`: common line-box caret and selection bounds, including visual selection runs.
- `raster.rs`: Swash scaling, color/mask alpha, physical glyph placement, clipped/tiled rendering and raster cache.
- `lib.rs`: UI-thread font registry and startup registration. Native Parley editor bridges can clone this registry without copying font bytes.
- `tests.rs`: proportional/Unicode metrics, logical wrapping/selection, display scale, upload identity, long-input regions and exact tile-boundary coverage.

Shape history has a 512-entry ceiling and an 8 MiB estimated working-set budget. Cached RGBA tiles have a 16 MiB byte budget. Native upload bridges also evict at 16 MiB; engine-managed GPU resources and font/shaping-library caches are additional. Malformed font assets return a configuration error consistently across bindings. Registration invalidates shared caches; fonts must be registered **before mounting** retained native editors. There is no runtime font mutation API. Colors do not change logical geometry, but colored RGBA tile cache entries are distinct.

Run `cargo test -p inrux-text --locked`. The opt-in provider timing fixture is:

```sh
cargo test -p inrux-text --release shared_text_cache_timing --locked -- --ignored --nocapture
```

It measures provider work, excluding native upload and presentation. See [framework integration and limits](../../docs/TEXT-RENDERING.md). Shared glyph pixels do not guarantee identical compositor output on every platform, and native editor IME/accessibility integration still requires platform testing.
