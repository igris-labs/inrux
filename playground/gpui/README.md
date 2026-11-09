# Inrux gpui playground

A standalone Rust application with its own source, build script, views, reusable layouts, styles, tests and licensed Noto Sans font. It selects only the gpui adapter. No sibling playground dependency is required.

```sh
cargo run -p inrux-playground-gpui --features runtime-shaders --locked
cargo run -p inrux-playground-gpui --features runtime-shaders --locked -- --compact
cargo test -p inrux-playground-gpui --features runtime-shaders --locked
```

Run from the workspace root or this directory. The window opens at 1200 × 860 logical pixels; `--compact` uses the 900 × 820 minimum. GPUI's runtime-shaders option is a development fallback on macOS without the optional Metal compiler. No engine performance ranking is inferred.

## Source organization

- `src/main.rs`: font/global stylesheet registration and App.iru mounting.
- `src/App.iru`: application composition and view selection.
- `src/components/`: reusable Sidebar, DemoPanel, Counter, NameInput and RetainedCounter.
- `src/layouts/ShowcaseLayout.iru`: full-height dashboard shell using typed navigation props and Children.
- `src/views/`: keyed ComponentPreview plus Layout, More layout, Content, Controls, Composition, Background, Scrolling and Engine screens.
- `src/catalogue/`: component metadata, grouped sidebar navigation and modular live specimens.
- `src/features/`: per-view action handlers and small style helpers.
- `src/composables/`: reusable Rust signal/computed state.
- `src/styles/` and `src/assets/fonts/`: locally owned styling and licensed font.
- `src/engine.rs`: bootstrap choice and truthful native-only examples/support notices.
- `src/tests.rs` and `src/typecheck.rs`: compiled application behavior and real rustc type checks.

Switching views disposes their local state and cancels owned background work; returning mounts a fresh example. Views are screens in one native window, not a router or multi-window API. All examples save in memory only. Keyboard focus, disabled admission, native editing and property support remain subject to the framework's documented platform coverage.

See [the catalogue/checklist](../README.md) and [source consistency check](../../tooling/playground-check/README.md).

Select, Dialog and Heartbeat are not implemented by this adapter. The Engine view explicitly reports that limitation; it does not render simulated substitutes. Every currently portable component is exercised in the other screens.

The More layout screen demonstrates Flex direction with a controlled input, equal-column Grid and AspectRatio. Toggle direction after editing to confirm retained state; switch two/three columns and square/landscape ratios. This uses the same layout provider and public component contract on every engine.

The Content screen demonstrates the shared Card, Badge and decorative Separator implementations. Edit the workspace name, save/reset it, and compare CSS with Rust customizations. Its child controls keep the normal native editing and event paths. See [the component guide](../../docs/CONTENT-COMPONENTS.md).

## Clipping and scrolling

[ScrollArea and overflow usage](../../docs/SCROLLING.md) covers the shared component, CSS/Rust overflow, UI-owned offsets, typed observations and revision-bearing commands. Native bindings supply rectangular paint/input clipping and native focus/pointer operations; shared policy supplies scroll bounds, nested routing and scrollbar geometry. The Scrolling playground view exercises these paths. Virtualization and complete accessible scroll actions remain separate work.

The Scrolling view also demonstrates TextArea: multiline model updates, wrapping, vertical editing scroll and disabled state, alongside the existing single-line Input. See [text editing](../../docs/TEXT-EDITING.md).

The application opens on Button in the dashboard. Choose a component under Components for its live preview, `.iru` usage and local state, or scroll to Examples for the integration workflows. Sidebar and content scroll independently. [Dashboard source map and behavior](../README.md).

## Compiled documentation styling

The light/dark documentation shell follows the supplied IPEX layout. Both themes compile in build.rs; `src/theme.rs` swaps shared immutable indexes without resetting component state. `src/styles/global.css` contains semantic tokens, selectors and responsive rules; dark.css overrides the token constants before compilation. All three apps share Noto Sans, portable sources and themed scrollbar palettes. The right page rail appears at 1180 logical pixels. See [CSS engine](../../docs/CSS-ENGINE.md) for supported syntax and remaining limitations.

The documentation rail now stays stationary inside the main area, while scrolling indicators sit at the viewport edge and auto-hide at rest. Sidebar padding prevents indicator/button overlap; structural surfaces are square. Usage and appearance sections use optional Markdown, parsed once in setup, and bundled Noto Sans Mono for preformatted code. See [Markdown usage and limits](../../packages/inrux-markdown/README.md).

The catalogue now contains 21 shared components, including controlled Switch and live Progress/Meter examples. Range pages demonstrate bounded values, reset, disabled controls and inherited accent changes. See [range and switch usage](../../docs/BOOLEAN-AND-RANGE-COMPONENTS.md).

## Headless style foundation

[Public parts and owner-state styles](../../docs/STYLING-PARTS.md) now provide overrideable boolean metrics/paint, labels, range fills and editor placeholder ink across all three engines. CSS and Rust share the same resolver and physical margin cascade. [Literal color values](../../docs/STYLE-COLORS.md) include alpha/OKLCH through the pinned color parser, with native alpha preserved. The guide records the unpublished Color tuple-pattern migration and remaining CSS/part limits. Application recipes remain in playground styles.
