# inrux-adapter

Backend-neutral hosting contracts and shared native policy. This crate imports no egui, Masonry, GPUI, windowing or graphics types. Applications normally use its exports through `inrux`.

`Adapter` supplies primitive capabilities, service readiness/prerequisite checks and a native `run` entry point. `WindowOptions` and `AdapterError` keep startup/error types portable. Selection is static at startup; live windows are not migrated between engines.

## Implement a backend

Implement `Adapter` in a separate crate. In `run`, validate the window configuration, create the native event loop on the main thread, and start `Session::start(initial, scope, rules, wake)`. The thread-safe wake callback should schedule native work. Take the newest frame through `Session::take_frame`, validate capabilities before translation, and, for the current implementation, select LayoutService with a provider owning its native text context; current bindings select TaffyLayout with TextService. The target permits a conforming native layout provider instead; it does not require running both layout paths. Route native actions through the frame's typed event handles. See [egui](../inrux-egui/src/app.rs), [Masonry](../inrux-masonry/src/app.rs) and [GPUI](../inrux-gpui/src/app.rs) for concrete integrations.

## Module boundaries

- `session.rs`: shared coordinator, component host, style resolution, reactive subscription and coalescing publication.
- `editing.rs`: controlled input revision/acknowledgement policy.
- `services/layout.rs`, `services/text.rs`, `services/support.rs`: replaceable layout, fallible text and readiness contracts.
- `services/clipping.rs`, `services/scroll.rs`: rectangular bounds, UI-owned viewport/commands/queries, nested routing and scrollbar policy.
- `layout.rs`: geometry and style/identity validation; feature-gated compatibility wrapper.
- `layout/fallback.rs`, `layout_policy.rs`, `services/layout/taffy.rs`: optional Taffy provider/translation; omitted when taffy-layout is disabled.
- `surface.rs`: common background, radius and inside-border defaults.
- `capabilities.rs`, `window.rs`, `error.rs`: validation and public contracts.

Public component constructors remain solely in `inrux-components`. An engine supplies private native widget bindings rather than another public component library. Backend-specific APIs must stay outside core/compiler/components; shared semantics belong here or in the existing owning package.

## Verification and limits

```sh
cargo test -p inrux-adapter --locked
python3 tooling/backend-check/check.py
```

Tests cover coordinator/event flow, thread ownership, invalid keys/styles/windows, layout wrapping and input acknowledgements. Capability checks describe primitives and interaction styling; they do not establish complete visual/accessibility/IME parity. Native controls retain platform-specific editing behavior; shared shaping and glyph rasterization supply common text geometry and pixels. See [alpha status](../../docs/STATUS.md) for conformance and performance gates. Animation refs, platform commands, large-list virtualization and final backend selection remain future work.

## Application fonts

Register bundled TrueType/OpenType bytes with `WindowOptions.fonts.push(FontAsset::new("Noto Sans", bytes))`. Use the font's intrinsic family name in `Style::font_family("Noto Sans")` or CSS `font-family: "Noto Sans"`. The family inherits through the component tree and is used in measurement and paint. The current CSS subset accepts one named family, without a fallback list. See [shared playground](../../playground/README.md) for a complete licensed-font example.

## Accepted service architecture

[alpha status](../../docs/STATUS.md) refines the original adapter design. Inrux owns behavior, styling and service contracts; private adapters prefer native implementations when conformant and supply fallback providers where needed. Shared means shared semantics/interface, not a mandatory shared algorithm. A fallback used by only one adapter may stay private, and existing libraries should be reused when suitable.

Layout/text provider contracts are implemented, all native bindings use them, and Adapter exposes services()/require_services(). The selected layout provider remains Taffy on all three; alternative native layout implementations are not yet delivered. Future contracts must define ownership, errors, capabilities, threading and invalidation without exposing engine types. Retained resources, selective updates, caching and virtualization require their own measured/conformance gates. See the [target diagrams](../../docs/ARCHITECTURE.md).

The shared controls module now includes muted placeholder color alongside accent/border/line-height tokens. [Cross-engine conformance](../../docs/STATUS.md) records native disabled-input, focus, typography and lifetime coverage; these corrections do not introduce service-provider substitution.

## Operational service API

See [adapter services and component prerequisites](../../docs/ADAPTER-SERVICES.md). LayoutService statically binds a LayoutProvider; providers own their text context when native services share a mutable window. TextService validates fallible measurements, and readiness checks keep unfinished component prerequisites explicit. Provider output validation checks finite bounds and child structure, not complete CSS/identity semantics by itself.

The optional `taffy-layout` feature supplies TaffyLayout and the compatible layout::compute helper. Direct adapter dependencies enable it by default; the headless facade disables defaults and each engine explicitly enables its selected fallback. `cargo test -p inrux-adapter --no-default-features --locked` verifies contracts without Taffy. `cargo run -p inrux-adapter --example layout_services --locked` runs the illustrative headless specimen.

The selected Taffy provider now executes Flex direction, bounded equal-column Grid and positive container aspect ratios. Request validation rejects invalid counts/ratios and unsupported leaf application before allocation. Empty ratio containers select the native container algorithm with a private zero-size layout-only child; this workaround is absent from the framework/native widget tree and published geometry. See [the sizing contract](../../docs/LAYOUT-COMPONENTS.md#flex-grid-and-aspectratio).

## Clipping and scrolling

[ScrollArea and overflow usage](../../docs/SCROLLING.md) covers the shared component, CSS/Rust overflow, UI-owned offsets, typed observations and revision-bearing commands. Native bindings supply rectangular paint/input clipping and native focus/pointer operations; shared policy supplies scroll bounds, nested routing and scrollbar geometry. The Scrolling playground view exercises these paths. Virtualization and complete accessible scroll actions remain separate work.

`editing/viewport.rs` supplies retained horizontal text offset/clamping and caret-reveal policy for private bindings that need it. Native shaping, editing, pointer and IME handling remain in those bindings.

## Logical viewport and compiled themes

Session owns a `ViewportState` and supplies it through Scope. Each native host reports actual logical size changes; identical reports do not notify. The coordinator tracks viewport/theme reads, reuses indexed global/local styles and publishes responsive descriptions while retaining component state. Native adapters consume resolved styles, including inherited scrollbar colors. This does not add an engine-specific stylesheet or a native layout cache. See [compiled styling](../../docs/CSS-ENGINE.md).

Overlay scrollbar activity/expiry policy is shared in `services/indicators.rs`; native bindings reveal on edge hover/drag and stop expiry work at rest. Auto indicators use a 700ms activity window; explicit overflow:scroll remains persistent. Indicators overlay content, so place a gutter/padding inside edge-to-edge navigation viewports. Shared layout also supports relative/absolute positioning and insets, and preserves preformatted glyph extents for horizontal overflow. Plain surfaces default to square corners; components supply explicit radii.

`FocusVisibility` is a small shared native paint policy, separate from logical focus and application state. Pointer activation hides non-editor keyboard outlines, keyboard interaction restores them, and control focus/IDs/event handling remain owned by the native host. egui/GPUI observe native input modalities; Masonry consumes native active/focus/key transitions. Editors retain their focus/caret feedback.

## Switch, ranges and control styling

Switch, determinate Progress and bounded Meter share portable descriptors and work with all three engines. Inherited `accent-color` and independent `row-gap`/`column-gap` compile into typed styles. See [APIs, styling, validation and accessibility limits](../../docs/BOOLEAN-AND-RANGE-COMPONENTS.md) and [CSS support](../../docs/CSS-ENGINE.md). Native bindings remain private; only the selected engine is required.

## Headless style foundation

[Public parts and owner-state styles](../../docs/STYLING-PARTS.md) now provide overrideable boolean metrics/paint, labels, range fills and editor placeholder ink across all three engines. CSS and Rust share the same resolver and physical margin cascade. [Literal color values](../../docs/STYLE-COLORS.md) include alpha/OKLCH through the pinned color parser, with native alpha preserved. The guide records the unpublished Color tuple-pattern migration and remaining CSS/part limits. Application recipes remain in playground styles.
