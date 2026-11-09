# My Inrux application

A native Rust application using `.iru` components. `src/main.rs` loads global styles and mounts `src/App.iru`. `build.rs` compiles reachable components; ordinary Rust code lives in `src/composables/` and can be imported from component `<rust>` blocks.

```sh
cargo run
cargo check
```

This unpublished starter depends on your local Inrux checkout. Its Cargo.toml selects one optional engine. Select `egui`, `masonry` or `gpui` and rebuild to switch this compatible starter; enable exactly one for automatic mounting. Use `cargo update` if an existing lockfile conflicts with GPUI's pinned dependency versions. Masonry and GPUI are experimental and support fewer controls/styles. Read the checkout's packages/inrux-masonry/README.md or packages/inrux-gpui/README.md before expanding that application.

GPUI on macOS normally requires Xcode's Metal compiler component. If that component is missing, `cargo run --features inrux/gpui-runtime-shaders` is the explicit development fallback; shader compilation then happens during startup.

`src/components/Counter.iru` demonstrates typed reusable props/events and scoped CSS. Two independently initialized counters share an ordinary Rust composable. Global CSS loads explicitly in main.rs. Blocking work should use Context::spawn_task; state updates and views run on the coordinator while native UI work stays on the main thread.
