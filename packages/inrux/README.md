# inrux

The public application API for **Inrux — Igris Native Rust UI eXperience**. Import this crate for component contracts, state/computed values, models/events, styles, scopes, native mounting, and background tasks. Internal packages are re-exported so applications normally need one runtime dependency.

This is an unpublished, experimental API. The SFC compiler is a separate build dependency. See the [workspace guide](../../README.md) and [supported features](../../docs/STATUS.md).

## Start an application

From the framework checkout:

```sh
cargo run -p inrux-cli -- new ../hello-inrux --framework-path . --backend egui
cd ../hello-inrux
cargo run
```

The starter creates main.rs, App.iru, components, a composable, global CSS, and a build script. Ordinary Rust modules and local Cargo libraries remain usable from the component's `<rust>` block.

```xml
<view>
    <Column gap={16.0}>
        <Text text={format!("Count: {}", state.count)} />
        <Button text="Increment" @click={increment} />
    </Column>
</view>
<rust>
use inrux::prelude::*;
#[derive(Clone)]
pub struct State { count: u32 }
/// Initialize state once for this mount.
pub fn setup() -> State { State { count: 0 } }
/// Apply a normal function on the application coordinator.
fn increment(state: &mut State, _: &Context<State>) { state.count += 1; }
</rust>
```

main.rs includes the generated bootstrap and calls `inrux::mount(title, app::setup(), Scope::default())`. For global CSS, use `mount_with_styles`; the [playground entry point](../../playground/egui/src/main.rs) is a complete example.

## Select an engine

The facade defaults to no engine. Enable exactly one feature (`egui`, experimental `masonry` or experimental `gpui`); the scaffold writes that choice into Cargo.toml. Changing the feature and rebuilding switches compatible applications without changing `.iru` imports. Unsupported Masonry/GPUI controls and styles fail visibly; full backend parity is still pending. GPUI setup requirements and the optional macOS runtime-shader fallback are in [its README](../inrux-gpui/README.md). Refresh an existing lockfile with `cargo update` if GPUI's exact transitive versions conflict after an engine switch.

`mount` and `mount_with_styles` select the sole enabled engine. `mount_with(adapter, WindowOptions, state, scope, styles)` supports an explicit adapter at startup, including comparison binaries intentionally linking multiple engines. Configuration and native failures return the engine-neutral `AdapterError`. egui-specific `render`/`render_ref` remain in `inrux-egui`, outside the facade. The devtools feature activates the inspector only when egui is already selected.

See [adapter contracts](../inrux-adapter/README.md), [Masonry coverage](../inrux-masonry/README.md) and [alpha status](../../docs/STATUS.md).

## Core APIs

- `Component`, `Events`, `Emit<T>`: typed composition and queued handlers.
- `Signal<T>`, `Computed<T>`: automatically tracked state and cached pure derived values.
- `Context::spawn_task`: bounded background work with typed result delivery.
- `Scope`: typed provide/inject; `Stylesheet`: runtime global theme replacement.
- `Node`, `Style`, `Color`, `Size`, `Align`, `Justify`: Rust descriptions and styling.
- `ComponentHost`, `Instances`, `InstanceHandle`: retained composition and explicit ownership.

State updates and view construction run on the coordinator. Native input, layout, and painting run on the main thread. Blocking work belongs in a background task; an action enum is optional.

## Composition and tooling

The working prototype uses `<view>` for UI markup and `<Children />` to render default child content. Lowercase child outlets, named/scoped content and animation refs are not implemented; see [alpha status](../../docs/STATUS.md). `<template>` and `<Slot />` remain compatible aliases. Required `Props` fields and emitted payloads are checked by rustc. See the [authoring contract](../../docs/AUTHORING.md) for models, keyed loops, dynamic components, and providers.

Enable `devtools` through the facade to use the native F12 inspector:

```sh
cargo run -p inrux-playground-egui --features inrux/devtools
cargo test -p inrux --locked
```

Node refs for component/child animation, named content, and scoped render functions are specified in [alpha status](../../docs/STATUS.md). Those future APIs are not currently exported.

## Architecture direction

The [alpha status](../../docs/STATUS.md) keeps styling, components and behavior in Inrux. Each adapter will prefer conforming native services and provide suitable fallbacks for gaps. The adapter now exposes operational layout/text contracts and service readiness through the re-exported services module. All bindings currently select Taffy layout and shared text; alternative native layout implementations and other independent service families remain planned. [Diagrams](../../docs/ARCHITECTURE.md) show the target, and [implementation status](../../docs/STATUS.md) records what exists.

## Content components

`Node::card`, `Node::badge` and `Node::separator` are available through this facade, with corresponding built-in `.iru` tags. They compose existing primitives, so applications use the same API with every current engine. See [examples and limits](../../docs/CONTENT-COMPONENTS.md).

## Clipping and scrolling

[ScrollArea and overflow usage](../../docs/SCROLLING.md) covers the shared component, CSS/Rust overflow, UI-owned offsets, typed observations and revision-bearing commands. Native bindings supply rectangular paint/input clipping and native focus/pointer operations; shared policy supplies scroll bounds, nested routing and scrollbar geometry. The Scrolling playground view exercises these paths. Virtualization and complete accessible scroll actions remain separate work.

Input/TextInput are strictly single-line; TextArea provides multiline editing, wrapping and vertical caret scrolling on all three engines. See [text editing](../../docs/TEXT-EDITING.md) for `.iru`, Rust, model and styling usage.

## Switch, ranges and control styling

Switch, determinate Progress and bounded Meter share portable descriptors and work with all three engines. Inherited `accent-color` and independent `row-gap`/`column-gap` compile into typed styles. See [APIs, styling, validation and accessibility limits](../../docs/BOOLEAN-AND-RANGE-COMPONENTS.md) and [CSS support](../../docs/CSS-ENGINE.md). Native bindings remain private; only the selected engine is required.

## Headless styling ownership

Components own behavior, keyboard/pointer interaction and accessibility, with minimal overridable defaults. Application themes and visual recipes belong to the application; the showcase design stays in the playground. See the [accepted contract](../../docs/HEADLESS-COMPONENTS.md) for current native-part customization gaps and the [CSS guide](../../docs/CSS-ENGINE.md) for implemented styling. Unsupported CSS features remain explicit errors.

## Headless style foundation

[Public parts and owner-state styles](../../docs/STYLING-PARTS.md) now provide overrideable boolean metrics/paint, labels, range fills and editor placeholder ink across all three engines. CSS and Rust share the same resolver and physical margin cascade. [Literal color values](../../docs/STYLE-COLORS.md) include alpha/OKLCH through the pinned color parser, with native alpha preserved. The guide records the unpublished Color tuple-pattern migration and remaining CSS/part limits. Application recipes remain in playground styles.
