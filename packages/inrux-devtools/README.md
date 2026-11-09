# inrux-devtools

An optional in-process inspector for Inrux's owned render tree. It shows node kinds, keys, classes, disabled state, resolved styles and the number of descriptions. It does not start a browser or remote server.

## Use through the native host

From the workspace root:

```sh
cargo run --release -p inrux-playground-egui --features inrux/devtools --locked
```

Press F12 to toggle the inspector; some keyboards require Fn+F12. The application and its state remain owned by the normal coordinator. The inspector reads descriptions and does not evaluate user component views or invoke their handlers.

## Embed the inspector

An existing egui host can use these functions with the current Node and Ui:

```rust
let descriptions = inrux_devtools::node_count(&node);
ui.label(format!("{descriptions} descriptions"));
inrux_devtools::inspect(&node, ui);
```

This is an integration excerpt: node and ui come from the host. Use matching workspace versions of inrux-components and eframe. Counts include excluded/modal descriptions, so they are not a count of visible native widgets or mounted application components.

## Scope and development

The package's `tree.rs` owns traversal and display. Native feature gating and F12 handling live in inrux-egui; normal applications need no direct devtools dependency.

```sh
cargo test -p inrux-devtools --locked
cargo check -p inrux-playground-egui --features inrux/devtools --locked
```

A profiler, application-state debugger, event recorder, animation timeline and ref inspection are future capabilities. The [alpha status](../../docs/STATUS.md) requires the inspector to expose target identity and liveness without retaining unmounted widgets.

The current inspector UI is egui-specific and is not a cross-engine profiler. Its optional dependency does not select egui for Masonry/GPUI applications. Under the [alpha status](../../docs/STATUS.md), compiler/inspection metadata remains engine-independent and presentation stays at the edge. Do not route build/editor tooling through the rendering adapter.
