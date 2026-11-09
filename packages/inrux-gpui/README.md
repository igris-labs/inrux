# inrux-gpui

Experimental GPUI 0.2.2 native infrastructure behind `inrux-adapter`. This is a private engine binding, not another public component library. Inrux owns application state, component contracts, styling, model/event policy and geometry. GPUI supplies the native window, element processing, text shaping, focus/input and GPU painting.

## Run the same starter

From the checkout:

```sh
cargo run -p inrux-cli -- new ../gpui-app --framework-path . --backend gpui
cd ../gpui-app
cargo run
```

The generated facade dependency enables only `gpui`. Enable exactly one engine (`egui`, `masonry` or `gpui`) for automatic mounting. To switch a compatible app, change that feature, run `cargo update` if the existing lockfile conflicts with GPUI's exact transitive version requirements, then rebuild. No `.iru` changes are needed for this starter. GPUI is pinned to the published 0.2.2 API; this is not a claim about newer unreleased versions.

macOS builds require Xcode and its Metal compiler component for the normal precompiled-shader path. If `xcrun metal` reports a missing toolchain, the explicit development fallback is:

```sh
cargo run --features inrux/gpui-runtime-shaders
```

That feature compiles shaders at application startup and must be reported separately in startup benchmarks. It stays off by default. The binding still enables macOS `font-kit` for GPUI native integration; application glyphs now come from the shared provider, and font assets are registered only with that provider. Linux enables X11/Wayland support; see the workspace CI system dependencies. Windows uses the native manifest; release shader compilation also requires the Windows SDK shader compiler. Only macOS has been checked locally.

## Current surface

Column, Row, Text, Button, Checkbox, Switch, Progress, Meter, Input/TextArea and Empty are implemented. GPUI elements consume fixed geometry from the common Taffy policy, with shared text measurement. A bounded notification channel wakes the main thread when the shared Session publishes a frame; animation frames never rerun application views. Pointer and Enter/Space activation dispatch through framework events. Tab navigation and keyed/generational focus use private native handles.

Inrux CSS/Rust styles remain authoritative. Backgrounds/radii translate to native paint, borders reserve no extra layout space, and text measurement/painting share a shaping path. Shaping and glyph coverage now use the common `inrux-text` provider; final compositor output remains platform-dependent. The checkbox uses a shared vector shape and ordinary text remains nonselectable; native accessibility roles, selection and complete cross-engine visual/interaction conformance remain open.

Select, Dialog and Heartbeat remain unsupported and produce capability diagnostics. Input uses shared shaping and geometry with GPUI native input services, Unicode grapheme editing, clipboard, selection, horizontal clipping, versioned model acknowledgements and local IME preedit. Native harness tests cover committed Unicode text, external resets and UTF-16 preedit selection; comprehensive platform/IME validation remains open. Virtualization, overlays, animation refs, native accessibility auditing and heavy-data performance remain separate gates. The three independent playgrounds expose the same portable catalogue and report engine-specific controls explicitly.

## Modules and checks

- `app.rs`: main-thread host, shared Session and bounded asynchronous wakeups.
- `capabilities.rs`: validation before translation.
- `layout.rs`: common layout with shared text measurement.
- `widgets/element.rs`: private description-to-element translation and retained focus.
- `widgets/text.rs`, `surface.rs`: shared text glyph uploads and native surface painting.
- `widgets/events.rs`, `identity.rs`: current event targets and private key/generation IDs.

```sh
cargo test -p inrux-gpui --features runtime-shaders --locked
cargo check -p inrux --no-default-features --features gpui-runtime-shaders --locked
python3 tooling/backend-check/check.py
```

The optional runtime-shader flag is useful on this checkout's macOS host; omit it when testing precompiled shaders with the required compiler installed. GPUI's test-support feature is a development dependency, not a production default. Native harness tests cover pointer/keyboard activation, controlled checkbox updates/disabled state, retained keyed focus and remount disposal; pure tests cover disabled events, identities and unsupported selection controls. No startup/memory ranking or production parity claim is made. See [alpha status](../../docs/STATUS.md) and [validation](../../docs/STATUS.md).

Upstream references: [GPUI 0.2.2 API](https://docs.rs/gpui/0.2.2/gpui/), [source and examples](https://docs.rs/crate/gpui/0.2.2/source/).

Disabling upstream defaults does not remove GPUI's unconditional dependencies, such as its asset support. This binding avoids unneeded optional services; complete transitive feature granularity is constrained by the upstream crate.

## Shared font and playground

`WindowOptions.fonts` registers application font bytes before mounting. Resolved `Style::font_family` / CSS `font-family` is used for measurement and paint. Run `cargo run -p inrux-playground-gpui --features runtime-shaders --locked` to compare the same settings application with egui and Masonry.

The native input binding in `widgets/input/{editing,platform,element}.rs` adapts GPUI 0.2.2's `examples/input.rs` under Apache-2.0; see [NOTICE](NOTICE).

## Native-service evolution

This binding currently uses common Taffy geometry with shared text measurement. The [alpha status](../../docs/STATUS.md) allows each service to delegate to this engine when it meets Inrux's contract, or use a suitable fallback when it does not. Layout/text provider interfaces are operational; this binding selects the optional Taffy fallback plus the shared `inrux-text` provider. Alternative native layout implementations, selective invalidation and complete incremental updates remain planned. Styling/component semantics stay in Inrux; native resources and execution stay private to the adapter. No native provider is assumed faster without comparable measurement.

## Input conformance increment

Input now paints the common authored-radius focus outline, releases focus when disabled, and uses a weak-entity UI-thread caret timer: 500 ms half cycles, solid after ten idle seconds, reset on editing and cancelled on blur/removal. Active selection paint is limited to a focused active window. macOS Cmd+Left/Right and Shift variants select/navigate the line; Ctrl+Cmd+Space opens the native character palette. Clipboard and Unicode/preedit/reset cases, disabled focus and keyed input identity have native harness tests.

`widgets/input/caret.rs` owns scheduling; `widgets/input/tests.rs` and `widgets/element/tests.rs` keep editing and retained-control test coverage modular. [The conformance matrix](../../docs/STATUS.md) records the limits; these tests do not establish all input methods or native accessibility support.

`services.rs` reports Layout as a fallback and TextMeasurement as native, and leaves unfinished clip/scroll/focus-command/overlay contracts unavailable. The native layout path now calls the [common service interface](../../docs/ADAPTER-SERVICES.md); application components do not select providers. Only the required fallback feature is enabled by this binding.

Default enabled Button hover derives from its resolved background using the shared Inrux palette; disabled buttons keep their base fill. Checkbox focus remains visible on the square, without outlining its full label row. See [playground guide](../../docs/PLAYGROUNDS.md).

## Clipping and scrolling

[ScrollArea and overflow usage](../../docs/SCROLLING.md) covers the shared component, CSS/Rust overflow, UI-owned offsets, typed observations and revision-bearing commands. Native bindings supply rectangular paint/input clipping and native focus/pointer operations; shared policy supplies scroll bounds, nested routing and scrollbar geometry. The Scrolling playground view exercises these paths. Virtualization and complete accessible scroll actions remain separate work.

Overflowing single-line Input supports retained horizontal gestures and caret reveal; vertical wheel input can reach its containing ScrollArea. See [behavior and native evidence](../../docs/SCROLLING.md#native-wheel-feel-and-overflowing-inputs).

Input/TextInput are strictly single-line; TextArea provides multiline editing, wrapping and vertical caret scrolling on all three engines. See [text editing](../../docs/TEXT-EDITING.md) for `.iru`, Rust, model and styling usage.

The repository Cargo root applies documented compatibility fixes for transitive `block` and `proc-macro-error2` releases. See [third-party patches](../../vendor/README.md), including the requirement to apply patches in the consuming Cargo root when using this unpublished crate from another workspace.

## Switch, ranges and control styling

Switch, determinate Progress and bounded Meter share portable descriptors and work with all three engines. Inherited `accent-color` and independent `row-gap`/`column-gap` compile into typed styles. See [APIs, styling, validation and accessibility limits](../../docs/BOOLEAN-AND-RANGE-COMPONENTS.md) and [CSS support](../../docs/CSS-ENGINE.md). Native bindings remain private; only the selected engine is required.

## Headless style foundation

[Public parts and owner-state styles](../../docs/STYLING-PARTS.md) now provide overrideable boolean metrics/paint, labels, range fills and editor placeholder ink across all three engines. CSS and Rust share the same resolver and physical margin cascade. [Literal color values](../../docs/STYLE-COLORS.md) include alpha/OKLCH through the pinned color parser, with native alpha preserved. The guide records the unpublished Color tuple-pattern migration and remaining CSS/part limits. Application recipes remain in playground styles.

## Shared typography

All application text now uses the [common text provider](../inrux-text/README.md), including control labels, placeholders and editors. Native input/focus/clipboard/IME integration remains in this binding. [Typography integration](../../docs/TEXT-RENDERING.md) explains fractional metrics, glyph uploads, cache budgets and validation limits. Register application fonts before mounting; retain intrinsic family names in styles.
