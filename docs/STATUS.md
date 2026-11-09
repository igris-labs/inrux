# Early alpha status

This source release is experimental. Public API, `.iru` grammar and internal architecture can change. It is intended for hands-on feedback and contributors, not production adoption guarantees.

## Current coverage

| Area | Available now | Limits |
| --- | --- | --- |
| Components | 21 portable catalogue components across egui, Masonry and GPUI | Select, Dialog and Heartbeat are egui-only |
| Authoring | `<view>`, `<rust>`, global/scoped styles, explicit imports, typed props/models/emits, default `<Children />`, block conditionals and keyed loops | Attribute-first control flow, transparent nested view groups, named/scoped/lazy content and animation refs are not implemented |
| State | Signals, cached computed values, provide/inject, retained component ownership and background tasks | No dedicated router/global-store package |
| Styling | Compiled/indexed subset, classes/IDs, compound selectors, tokens, viewport conditions, selected states and parts, Rust styles | No browser-complete CSS; see the exact [subset](CSS-ENGINE.md) |
| Layout/text | Shared Taffy layout; Parley/Swash shaping, geometry and glyph pixels | Full incremental layout/native tree patching and alternate native layout providers remain open |
| Native integration | Three selectable engines, single-line Input, multiline TextArea, scrolling and focus handling | Full keyboard, accessibility, clipboard, IME and per-platform conformance still need testing |
| Tooling | Scaffolding, rebuild/restart development command, language server and linked VS Code extension | No state-preserving reload; Rust/template diagnostic parity and editor refactoring are incomplete |
| Distribution | GitHub source checkout, locked dependencies and local patches | Not published to crates.io or the VS Code Marketplace |

`<template>` and `<Slot />` are accepted compatibility aliases for the current view/default-child forms. Future syntax is not a stable compatibility promise. Use the checked-in [authoring guide](AUTHORING.md) and working playground sources.

## Platform and performance limits

Local native development has focused on macOS. The checked-in CI matrix targets Linux, macOS and Windows, but configuration alone does not establish successful builds or UX parity on every platform. Inspect the actual [Actions results](https://github.com/igris-labs/inrux/actions) once available.

There is no verified claim of instant startup, zero translation cost, optimal idle/heavy memory or superior performance to another framework. Shared layout/text, native upload costs and runtime shader compilation must be included in measurements. The style benchmark is synthetic and does not measure end-to-end application performance.

Large-list virtualization, portable overlays, advanced text/grid/paint/motion styles, animation refs and complete accessibility remain significant gaps. Headless-first describes appearance ownership; it does not certify every interaction or accessibility requirement yet.

## Feedback that helps

Report engine, OS/display scale, `rustc --version`, reproduction steps, expected/actual behavior and a minimal source example. For rendering differences, attach screenshots and name the fonts. For performance, include build profile, warm/cold conditions, workload and measurements. Use [GitHub issues](https://github.com/igris-labs/inrux/issues).
