# inrux-egui

Optional eframe/egui native host for Inrux. Enable the facade's `egui` feature; applications normally import only `inrux`. The host owns native windowing, input and painting; shared text services supply measurement and glyph pixels. Shared component ownership, reactivity, layout policy and input acknowledgements live outside the engine.

## Run an application

```sh
cargo run --release -p inrux-playground-egui --locked
cargo run -p inrux-cli -- new ../egui-app --framework-path . --backend egui
```

The scaffold enables only egui. Its main.rs mounts the generated App.iru through `inrux::mount_with_styles`. The playground explicitly selects `inrux::backends::Egui` through `mount_with`, so workspace checks may deliberately enable all three engines. Ordinary applications should enable one. `WindowOptions` configures title, initial size and minimum size at startup.

## Existing egui host integration

Depend directly on matching versions of inrux-egui and eframe for engine-specific embedding. These functions are not re-exported from the engine-neutral facade:

```rust
use inrux_components::Node;
/// Paint a description inside an existing native or headless egui frame.
fn show(ui: &mut eframe::egui::Ui) -> Result<Option<String>, inrux_adapter::AdapterError> {
    inrux_egui::render(Node::text("Hello from Rust".into()), ui)
}
```

`render` resolves inherited text styling. `render_ref` consumes an already resolved description. Errors use the common AdapterError; the optional string reports interaction failure. Native-specific legacy mount functions remain available here, returning eframe::Error; normal apps use the facade's common mount contract.

## Modules and threading

- `app.rs`: native window, shared Session integration, repaint and optional inspector.
- `render.rs`: frame orchestration, shared text measurement and common layout invocation.
- `paint.rs`: geometry traversal and graphics translation of shared surface defaults.
- `widgets/<name>.rs`: private native control bindings, not public Inrux components.
- `widgets/input.rs`: uses the shared editor acknowledgement implementation.
- `typography.rs`, `platform.rs`: common line-height text shaping and macOS sRGB window configuration.

The coordinator owns state updates, retained children, views and reactive tracking. A coalescing mailbox delivers descriptions; the main thread performs shared text/layout work and native image/surface painting. UI animation frames do not rerun views. Background work uses the core's bounded pool. Keyed controlled inputs preserve newer typing and focus through reorder; custom plain event inputs have a limited acknowledgement protocol.

## Verification and limits

```sh
cargo test -p inrux-egui --all-features --locked
cargo run --release -p inrux-playground-egui --features inrux/devtools --locked
python3 tooling/backend-check/check.py
```

The optional F12 inspector uses egui and does not activate this engine when another is selected. Native launches require a graphical session. Layout still runs per native frame; IME/accessibility audits and startup/memory/frame distributions remain unestablished. Animation refs/platform commands are future work. See [implementation status](../../docs/STATUS.md) and [alpha status](../../docs/STATUS.md).

## Application fonts

Register bundled TrueType/OpenType bytes with `WindowOptions.fonts.push(FontAsset::new("Noto Sans", bytes))`. Use the font's intrinsic family name in `Style::font_family("Noto Sans")` or CSS `font-family: "Noto Sans"`. The family inherits through the component tree and is used in measurement and paint. The current CSS subset accepts one named family, without a fallback list. See [shared playground](../../playground/README.md) for a complete licensed-font example.

## Native-service evolution

This binding currently uses common Taffy geometry with shared text measurement. The [alpha status](../../docs/STATUS.md) allows each service to delegate to this engine when it meets Inrux's contract, or use a suitable fallback when it does not. Layout/text provider interfaces are operational; this binding selects the optional Taffy fallback plus the shared `inrux-text` provider. Alternative native layout implementations, selective invalidation and complete incremental updates remain planned. Styling/component semantics stay in Inrux; native resources and execution stay private to the adapter. No native provider is assumed faster without comparable measurement.

## Package naming

The egui binding uses the engine-specific package/directory name `inrux-egui` and Rust import path `inrux_egui`, consistent with `inrux-gpui` and `inrux-masonry`. Ordinary applications continue using the `inrux` facade, its `egui` feature and `inrux::backends::Egui`; engine-neutral hosting contracts stay in `inrux-adapter`.

## Consistency and input checks

Input disabling is enforced by the native UI, including focused-to-disabled transitions and external resets. Hover/focus skins stay within authored bounds. Bundled font registration derives leading from actual ascent/descent and the shared line height through private ttf-parser usage, and retains egui's missing-glyph fallback chain. Malformed assets return configuration errors before egui font loading. Tests reuse the shared playground's licensed font fixture in this unpublished workspace.

The [conformance matrix](../../docs/STATUS.md) distinguishes native observation from automated coverage and remaining IME/platform work. `typography.rs` owns font registration/metrics; no font or engine types enter the framework contract.

`services.rs` reports Layout as a fallback and TextMeasurement as native, and leaves unfinished clip/scroll/focus-command/overlay contracts unavailable. The native layout path now calls the [common service interface](../../docs/ADAPTER-SERVICES.md); application components do not select providers. Only the required fallback feature is enabled by this binding.

Unregistered named families now return an actionable WindowOptions.fonts diagnostic before native measurement/paint, including fixed-size nodes that do not request intrinsic measurement.

Default enabled Button hover derives from its resolved background using the shared Inrux palette; disabled buttons keep their base fill. Checkbox focus remains visible on the square, without outlining its full label row. See [playground guide](../../docs/PLAYGROUNDS.md).

## Clipping and scrolling

[ScrollArea and overflow usage](../../docs/SCROLLING.md) covers the shared component, CSS/Rust overflow, UI-owned offsets, typed observations and revision-bearing commands. Native bindings supply rectangular paint/input clipping and native focus/pointer operations; shared policy supplies scroll bounds, nested routing and scrollbar geometry. The Scrolling playground view exercises these paths. Virtualization and complete accessible scroll actions remain separate work.

Overflowing single-line Input supports retained horizontal gestures and caret reveal; vertical wheel input can reach its containing ScrollArea. See [behavior and native evidence](../../docs/SCROLLING.md#native-wheel-feel-and-overflowing-inputs).

Input/TextInput are strictly single-line; TextArea provides multiline editing, wrapping and vertical caret scrolling on all three engines. See [text editing](../../docs/TEXT-EDITING.md) for `.iru`, Rust, model and styling usage.

## Switch, ranges and control styling

Switch, determinate Progress and bounded Meter share portable descriptors and work with all three engines. Inherited `accent-color` and independent `row-gap`/`column-gap` compile into typed styles. See [APIs, styling, validation and accessibility limits](../../docs/BOOLEAN-AND-RANGE-COMPONENTS.md) and [CSS support](../../docs/CSS-ENGINE.md). Native bindings remain private; only the selected engine is required.

## Headless style foundation

[Public parts and owner-state styles](../../docs/STYLING-PARTS.md) now provide overrideable boolean metrics/paint, labels, range fills and editor placeholder ink across all three engines. CSS and Rust share the same resolver and physical margin cascade. [Literal color values](../../docs/STYLE-COLORS.md) include alpha/OKLCH through the pinned color parser, with native alpha preserved. The guide records the unpublished Color tuple-pattern migration and remaining CSS/part limits. Application recipes remain in playground styles.

## Shared typography

All application text now uses the [common text provider](../inrux-text/README.md), including control labels, placeholders and editors. Native input/focus/clipboard/IME integration remains in this binding. [Typography integration](../../docs/TEXT-RENDERING.md) explains fractional metrics, glyph uploads, cache budgets and validation limits. Register application fonts before mounting; retain intrinsic family names in styles.

For advanced embedding through `render` / `render_ref`, call `inrux_egui::configure_fonts(&context, &font_assets)` on the rendering thread first. It configures both the shared font registry and egui's editor metadata; setting raw egui `FontDefinitions` alone does not register assets with the common provider. Ordinary Inrux windows configure fonts automatically.
