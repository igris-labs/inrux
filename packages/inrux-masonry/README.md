# inrux-masonry

Experimental retained native host using published Masonry 0.4.0 and masonry_winit 0.4.0. Masonry owns the native widget tree, input/focus, accessibility tree and drawing. Inrux owns the component contract and application state; common layout and surface policy live in `inrux-adapter`.

## Run an application

From the checkout, generate the same starter used by the egui adapter:

```sh
cargo run -p inrux-cli -- new ../masonry-app --framework-path . --backend masonry
cd ../masonry-app
cargo run
```

The facade dependency enables only `masonry`, with defaults disabled. To switch a compatible application, change that feature to `egui` and rebuild; its `.iru` component imports, props, events and styles remain unchanged. Both engines may be linked deliberately for comparison using `mount_with`, but ordinary apps should enable only one. GPUI is also available as an experimental binding; see [its current coverage](../inrux-gpui/README.md).

## Current coverage

Column, Row, Text, Button, Input/TextArea, Checkbox, Switch, Progress, Meter and Empty are translated. Shared Taffy policy handles gap, padding, sizes, flex alignment and wrapping. Keys/generations preserve or remount native identities; controlled inputs reuse the adapter's acknowledgement policy. Application updates run on the shared coordinator, with event-driven wakeups onto the main thread.

Select, Dialog, Heartbeat and interaction-state style layers are unsupported and produce visible capability errors. Errors replace the native surface with a diagnostic; they are not silently substituted. Native control skins, detailed styling, text metrics and platform behavior do not yet have full egui parity. This adapter is not a production-interchangeability claim or a performance winner.

## Modules and verification

- `app.rs`: main-thread host, coordinator wakeups and action routing.
- `capabilities.rs`: supported feature gate.
- `widgets/element.rs`: private retained translation/reconciliation and native event mapping.
- `widgets/{text,button,checkbox}.rs` and `widgets/input/`: private native leaf bindings, not public Inrux components.
- `layout.rs`: translate common geometry to Masonry constraints/placement and measure shared text.
- `paint.rs`: draw the common surface policy with Vello.
- `theme.rs`: light native defaults, overridable by resolved application styles.

```sh
cargo test -p inrux-masonry --locked
cargo check -p inrux --no-default-features --features masonry --locked
python3 tooling/backend-check/check.py
```

Native Masonry harness tests exercise pointer activation, keyboard input, keyed focus preservation, generational remounts, layout and unsupported controls. A graphical session is needed for a real window. Startup/memory/frame benchmarks, heavy-tree virtualization, complete IME/screen-reader auditing and the remaining component/style surface are open gates; see [alpha status](../../docs/STATUS.md).

The host and target dependency graphs explicitly enable Naga 26 termcolor diagnostics to coexist with GPUI's Naga 25/codespan features. The small build.rs activates that host-side feature alignment; it generates no framework code. This workaround stays private to the native package.

## Application fonts

Register bundled TrueType/OpenType bytes with `WindowOptions.fonts.push(FontAsset::new("Noto Sans", bytes))`. Use the font's intrinsic family name in `Style::font_family("Noto Sans")` or CSS `font-family: "Noto Sans"`. The family inherits through the component tree and is used in measurement and paint. The current CSS subset accepts one named family, without a fallback list. See [shared playground](../../playground/README.md) for a complete licensed-font example.

## Native-service evolution

This binding currently uses common Taffy geometry with shared text measurement. The [alpha status](../../docs/STATUS.md) allows each service to delegate to this engine when it meets Inrux's contract, or use a suitable fallback when it does not. Layout/text provider interfaces are operational; this binding selects the optional Taffy fallback plus the shared `inrux-text` provider. Alternative native layout implementations, selective invalidation and complete incremental updates remain planned. Styling/component semantics stay in Inrux; native resources and execution stay private to the adapter. No native provider is assumed faster without comparable measurement.

## Consistency checks

Checkbox pointer admission now excludes secondary clicks while retaining primary/touch and keyboard behavior. Focused disabled-input/reset behavior and initial placeholder typography have native harness regressions. A first-mount mutation applies authored styles to TextInput's separate native placeholder Label before paint. The placeholder/caret use shared tokens. See [conformance scope](../../docs/STATUS.md); this is not complete platform/IME/accessibility certification.

`services.rs` reports Layout as a fallback and TextMeasurement as native, and leaves unfinished clip/scroll/focus-command/overlay contracts unavailable. The native layout path now calls the [common service interface](../../docs/ADAPTER-SERVICES.md); application components do not select providers. Only the required fallback feature is enabled by this binding.

Default enabled Button hover derives from its resolved background using the shared Inrux palette; disabled buttons keep their base fill. Checkbox focus remains visible on the square, without outlining its full label row. See [playground guide](../../docs/PLAYGROUNDS.md).

The shared Card, Badge and decorative Separator compositions use the existing Column/Text bindings. Text labels now explicitly use native word wrapping, matching shared measured geometry; a native regression checks shaped lines at narrow/wide widths including Badge padding. This adds no component-specific native widget. See [content usage](../../docs/CONTENT-COMPONENTS.md).

## Clipping and scrolling

[ScrollArea and overflow usage](../../docs/SCROLLING.md) covers the shared component, CSS/Rust overflow, UI-owned offsets, typed observations and revision-bearing commands. Native bindings supply rectangular paint/input clipping and native focus/pointer operations; shared policy supplies scroll bounds, nested routing and scrollbar geometry. The Scrolling playground view exercises these paths. Virtualization and complete accessible scroll actions remain separate work.

## Single-line text viewport

`widgets/input/mod.rs` reconciles the controlled model. `field.rs` keeps the native field/placeholder property wrapper; `area/mod.rs`, `area/events.rs` and `area/rendering.rs` adapt Masonry 0.4.0's Apache-2.0 TextArea around Parley. That version exposes no caret-scroll API, so the private adaptation adds horizontal panning/caret reveal while retaining native editing, selection, clipboard, IME and accessibility handling. Shared offset/reveal math lives in `inrux-adapter::editing::TextViewport`. See NOTICE for attribution. Sync the adapted native code when upgrading Masonry, and rerun native editing/IME checks; full OS IME conformance is not certified here.

Input/TextInput are strictly single-line; TextArea provides multiline editing, wrapping and vertical caret scrolling on all three engines. See [text editing](../../docs/TEXT-EDITING.md) for `.iru`, Rust, model and styling usage.

## Switch, ranges and control styling

Switch, determinate Progress and bounded Meter share portable descriptors and work with all three engines. Inherited `accent-color` and independent `row-gap`/`column-gap` compile into typed styles. See [APIs, styling, validation and accessibility limits](../../docs/BOOLEAN-AND-RANGE-COMPONENTS.md) and [CSS support](../../docs/CSS-ENGINE.md). Native bindings remain private; only the selected engine is required.

## Headless style foundation

[Public parts and owner-state styles](../../docs/STYLING-PARTS.md) now provide overrideable boolean metrics/paint, labels, range fills and editor placeholder ink across all three engines. CSS and Rust share the same resolver and physical margin cascade. [Literal color values](../../docs/STYLE-COLORS.md) include alpha/OKLCH through the pinned color parser, with native alpha preserved. The guide records the unpublished Color tuple-pattern migration and remaining CSS/part limits. Application recipes remain in playground styles.

## Sidebar/navigation update cost

Retained text/font/property updates now skip unchanged values, so hover/focus paint no longer requests a full shared layout. Shared geometry is reused across native passes until a new framework publication or changed window constraints invalidates it. Text metrics reuse exact text/font-size/family/wrap-width requests across publications; only the current layout's request set survives, so the cache does not retain navigation history. Native label creation and shared measurement use the same default bundled Noto Sans family.

`cargo test -p inrux-masonry dashboard_navigation_timing --all-features --locked -- --ignored --nocapture` runs the opt-in native CPU specimen. It excludes the coordinator and GPU presentation and is not an FPS or universal click-to-frame benchmark. See [current limitations](../../docs/STATUS.md). Runtime font registration is not supported; adding it must invalidate native text/geometry caches.

## Scroll precision

Wheel and viewport keyboard movement use native composition translations, retaining fractional logical offsets without relaying out or remeasuring content. Display scaling is applied once at the physical-input boundary; the system-adjusted direction is preserved. Changed geometry, focus reveal, explicit scroll commands and scrollbar dragging still use layout. Overlay paint and changed-metrics notifications follow the same shared scroll state. See [alpha status](../../docs/STATUS.md).

## Shared typography

All application text now uses the [common text provider](../inrux-text/README.md), including control labels, placeholders and editors. Native input/focus/clipboard/IME integration remains in this binding. [Typography integration](../../docs/TEXT-RENDERING.md) explains fractional metrics, glyph uploads, cache budgets and validation limits. Register application fonts before mounting; retain intrinsic family names in styles.
