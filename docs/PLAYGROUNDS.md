# Playground applications

Run these commands from the repository root, one at a time:

| Engine | Command |
| --- | --- |
| egui | `cargo run -p inrux-playground-egui --locked` |
| Masonry | `cargo run -p inrux-playground-masonry --locked` |
| GPUI | `cargo run -p inrux-playground-gpui --locked` |
| GPUI, macOS runtime shaders | `cargo run -p inrux-playground-gpui --features runtime-shaders --locked` |

See [native prerequisites](GETTING-STARTED.md#requirements). Append `-- --compact` for a 900 × 820 logical-pixel window; the default is 1200 × 860. These are desktop examples; mobile navigation is not implemented.

## Explore

Each app has a sidebar, component documentation, interactive demos, `.iru` usage examples, light/dark switching and a page rail on wide windows. Navigation selects a component page; Reset example restores its local specimen state. Switching away disposes the view's owned events/tasks. Nothing persists to disk.

The 21 portable catalogue pages are Button, Input, TextArea, Checkbox, Switch, Progress, Meter, Text, Badge, Card, Separator, Box, Container, Row, Column, Center, Spacer, Flex, Grid, AspectRatio and ScrollArea. `TextInput` is also an authored alias for single-line Input.

The integration pages cover Layout, Controls, Composition, Background, More layout, Content, Engine and Scrolling: props/models/emits, computed values, keyed composition, typed injection, cancellable tasks, layout and nested scrolling. Select, Dialog and Heartbeat are currently egui-only; the other Engine pages show explicit support notices.

All three projects own matching portable sources, application CSS and licensed Noto Sans/Noto Sans Mono assets. Shared shaping/rasterization supplies consistent logical text and glyph pixels; native compositors and input integration can still differ. Do not assume whole-window pixel or accessibility parity.

## Source and checks

Start at `playground/<engine>/src/main.rs`, `App.iru`, `layouts/ShowcaseLayout.iru`, `catalogue/` and `views/`. `build.rs` compiles both light/dark CSS sheets into Rust. Theme replacement reuses immutable indexes. The application owns the documentation design; framework components provide behavior and overridable defaults.

```sh
python3 tooling/playground-check/check.py
python3 tooling/backend-check/check.py
cargo test -p inrux-playground-egui -p inrux-playground-masonry -p inrux-playground-gpui --all-features --locked
```

The scripts check source/font consistency and engine dependency isolation. They do not prove frame smoothness, platform input correctness or accessibility. Inspect each engine manually for those behaviors, and include the engine/OS when reporting a bug. See the individual [egui](../playground/egui/README.md), [Masonry](../playground/masonry/README.md) and [GPUI](../playground/gpui/README.md) READMEs.
