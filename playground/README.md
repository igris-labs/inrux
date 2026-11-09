# Inrux documentation playgrounds

Three complete applications demonstrate the same portable Inrux component catalogue and working integration examples. Each owns its `src/`, font, build script and README. Select an engine with the project you run:

```sh
cargo run -p inrux-playground-egui --locked
cargo run -p inrux-playground-masonry --locked
cargo run -p inrux-playground-gpui --features runtime-shaders --locked
```

Use `-- --compact` for a 900 × 820 content window; the default is 1200 × 860. GPUI's runtime-shaders flag is for macOS hosts without the optional Metal compiler. The supported desktop minimum remains 900 × 820; width/height CSS media queries respond to logical viewport changes; mobile navigation remains future work.

The documentation layout follows the local IPEX design: persistent header/breadcrumb, independently scrollable grouped navigation, a centered reading column, demo and usage panels, and a wide-window page rail. Light/dark switching uses compiled semantic tokens and preserves specimen state. The page rail offers working top/end scroll commands. Under **Components**, choose Button, Input, TextArea, Checkbox, Switch, Progress, Meter, Text, Badge, Card, Separator, Box, Container, Row, Column, Center, Spacer, Flex, Grid, AspectRatio or ScrollArea. Each page has a working preview, a minimal `.iru` usage snippet, state readouts where relevant, and Reset example. Input also accepts the TextInput alias in authored code.

Scroll farther down the sidebar to **Examples** for Layout, Controls, Composition, Background, More layout, Content, Engine and Scrolling. These retain the earlier multi-component workflows: save/reset, props/events, computed values, keyed children, injection, cancellable worker tasks and nested scrolling. Views and individual keyed previews own local state; switching away disposes their events and work. Everything saves in memory only.

Select, Dialog and Heartbeat retain their existing coverage: working examples in egui's Engine page and explicit unsupported notices in Masonry/GPUI. The dashboard does not change that support matrix.

## Source map

- `src/App.iru`: typed local navigation and reachable view composition.
- `src/layouts/ShowcaseLayout.iru`: full-window shell with default Children content.
- `src/components/Sidebar.iru`: navigation pane and footer.
- `src/navigation.rs`: page selection, scroll commands and synchronized header labels.
- `src/theme.rs`, `src/styles/{global,dark}.css`, `build.rs`: build-time theme compilation and shared indexed stylesheet replacement.
- `src/catalogue/metadata.rs`: the component list, descriptions and usage snippets.
- `src/catalogue/sidebar.rs`: grouped native button navigation with typed events.
- `src/catalogue/{controls,content,layout}.rs`: focused Rust-built component specimens.
- `src/views/ComponentPreview.iru`: keyed, view-local preview state and Reset example.
- `src/views/`, `src/features/`, `src/composables/`: integration views and their ordinary Rust logic.
- `src/main.rs`, `src/engine.rs`: mount, shared font/style registration and engine selection.

The individual specimens deliberately demonstrate Rust construction alongside the `.iru` snippets. All use the same framework components; no engine types appear in catalogue or dashboard code. A reusable app-level Sidebar and existing Box/Row/ScrollArea/Children were enough; no duplicate public framework component was introduced.

## Checks

Run `python3 tooling/playground-check/check.py` from the repository root to verify source/font parity and engine isolation. `src/tests.rs` in each app exercises all 21 component destinations, typed models, resets, remount disposal and retained integration behavior. egui also has native pointer/keyboard tests that scroll the sidebar before selecting hidden options. [playground guide](../docs/PLAYGROUNDS.md) document compact/larger layouts; see [validation](../docs/STATUS.md) for the checks and remaining platform limits.

The playground owns its application styles. It has no dependency on a separate design reference repository.

## Documentation corrections

The ToC is a stationary ruled overlay inside the document area, outside its scrolling content; the main scrollbar reaches the window edge. Sidebar content has an inner indicator gutter. Auto indicators hide after 700 ms without movement and remain discoverable on edge hover/drag. Structural backgrounds are square; the OS window frame handles outer corners.

Usage and appearance content use the optional `inrux-markdown` renderer, with documents parsed once during specimen setup. `src/assets/fonts/NotoSansMono.ttf` and its license are identical across the apps and registered as Noto Sans Mono. Code is preformatted and long lines scroll horizontally. See [Markdown API and current limits](../packages/inrux-markdown/README.md).

Sidebar spacing belongs to header, menu content and footer wrappers; the scrollport spans the pane width and its track sits at the boundary. Pointer-selected buttons keep selection/focus without a keyboard outline. Keyboard interaction restores visible focus in all three native bindings.

## Headless styling ownership

Components own behavior, keyboard/pointer interaction and accessibility, with minimal overridable defaults. Application themes and visual recipes belong to the application; the showcase design stays in the playground. See the [accepted contract](../docs/HEADLESS-COMPONENTS.md) for current native-part customization gaps and the [CSS guide](../docs/CSS-ENGINE.md) for implemented styling. Unsupported CSS features remain explicit errors.

## Headless style foundation

[Public parts and owner-state styles](../docs/STYLING-PARTS.md) now provide overrideable boolean metrics/paint, labels, range fills and editor placeholder ink across all three engines. CSS and Rust share the same resolver and physical margin cascade. [Literal color values](../docs/STYLE-COLORS.md) include alpha/OKLCH through the pinned color parser, with native alpha preserved. The guide records the unpublished Color tuple-pattern migration and remaining CSS/part limits. Application recipes remain in playground styles.
