# Current authoring contract

The supported logic block is `<rust>`. Code examples here describe implemented APIs; future syntax is explicitly labeled. Rust expressions use braces; strings use quotes. Closing SFC block tags must be on their own lines.

The compiler currently uses capitalized `<Children />` and block conditionals/loops. Attribute-first control flow, nested transparent view groups, named/scoped content and animation refs are not implemented; see [alpha status](STATUS.md).

## Reusable components

Import each custom tag explicitly in its parent's Rust block:

```rust
use crate::components::counter as Counter;
```

`components/Counter.iru` generates `crate::components::counter`. Nested folders generate nested Rust modules. Aliases preserve their exact Rust spelling; there is no auto-import registry. A custom `<Missing />` produces an original-file diagnostic and import suggestion. Ordinary Rust imports and local Cargo libraries use the same Rust block.

Stateless children declare `pub struct Props`. Stateful children declare cloneable Props and State, put their current props in `State.props`, and expose `pub fn setup(props: Props) -> State`. Their templates read local fields through `state` and props through `state.props`. Setup is called once per mount generation, not on props updates or animation frames. The native host retains state; headless/manual hosts use `ComponentHost::render`.

Normal prop fields map directly to the typed Props struct. Missing/unknown fields and Rust type mismatches are enforced by rustc. Those type errors currently identify generated Rust, while framework syntax/import errors identify `.iru` sources.

## Events and models

`@click={save}` on a native Button accepts an ordinary function:

```rust
fn save(state: &mut State, context: &Context<State>) { /* mutate state or schedule work */ }
```

A child `@confirmed={confirmed}` maps to `on_confirmed: Emit<T>`. Payload-bearing parent functions accept `(&mut State, T, &Context<State>)`. Handlers can use the two-argument signature above when they do not need the emitted payload. A stateless child forwards an owned Emit handle, for example `<Button @click={props.on_save.clone()} />`.

Native models:

```xml
<Input model:value={state.name} @input={mark_dirty} />
<Checkbox label="Notifications" model:checked={state.notifications} />
<Select model:value={state.appearance} options={vec!["Light".into(), "Dark".into()]} />
```

Model targets are direct `state.field` bindings. Input/Select use String or Signal<String>; Checkbox uses bool or Signal<bool>. The setter executes before an optional model handler in the same coordinator command. Without a model, provide a controlled value and typed @input/@change handler.

A custom `<NameInput model:value={state.name} />` supplies `value` and `on_update_value`. Its Props declare those types explicitly. Add `@update:value={mark_dirty}` to run a parent listener after assignment within the same coordinator command. This works with a payload-consuming or payload-ignoring ordinary function, and avoids publishing an intermediate “new value, still saved” state. Additional emitted payload events remain separate typed props. Specifying the same prop/model or model update event twice is a framework error.

An advanced component may retain Action/update methods instead of ordinary functions. Use the Events helpers directly for more specialized Rust wiring. Avoid blocking code inside Emit callbacks; event factories enqueue work and return overload errors.

## Conditionals and keyed loops

Attributes are the preferred authoring direction; that shorthand is not implemented yet. The working block forms below remain supported alternatives. Nested transparent `<view>` groups and attribute/content lowering remain unavailable. Current block lowering creates Column wrappers; transparent grouping is still future work.

```xml
<Column>
    <If condition={state.busy}><Text text="Working" /></If>
    <ElseIf condition={state.failed}><Text text="Failed" /></ElseIf>
    <Else><Text text="Ready" /></Else>
    <For each={&state.rows} item="row" key={row.id}>
        <Editor title={row.title.clone()} />
    </For>
</Column>
```

ElseIf/Else must immediately follow an If/ElseIf sibling, and Else must be last. For uses Rust IntoIterator semantics; `item` names its Rust binding. Keys are mandatory on loops and must be unique among sibling iterations. Reordering preserves local state, controlled text and native focus/caret identity. Unmount/remount uses a fresh generation; old events and task completions cannot mutate the new mount.

Custom component `key={...}` changes its mount identity. Sibling duplicate keys are rejected before painting. Unkeyed ordinary nodes use structural positions, so dynamic lists should use explicit keys.

## Dynamic components

Dynamic selection uses a typed Rust expression returning Node. Candidates must be imported, and their Props remain individually checked:

```xml
<Component :is={if state.use_editor {
    Editor::view(Editor::Props { title: state.title.clone() }, events, scope)
} else {
    Preview::view(Preview::Props { title: state.title.clone() }, events, scope)
}} />
```

This prototype uses typed factories instead of string component-name lookup. The dynamic call receives its own hierarchical scope. Unselected stateful candidates are unmounted; reselecting them starts a fresh generation. Imported candidates are included in the build graph even if only used in this runtime expression. See the selector-driven dynamic preview in App.iru.

## Child content and scoped providers

A default slot is an owned Node prop:

```xml
<Panel><Text text="Child content" /></Panel>
```

Panel declares `pub children: Node` in Props and renders `<Children />`. Stateful panels read State.props.children. Slots are composed eagerly in this prototype; unused composed mounts are swept from the rendered tree afterward. Lazy content factories, named content and scoped render functions are required next composition work and are not implemented. Named and scoped content have no stable public syntax yet.

Providers are ordinary typed Rust values:

```xml
<Provide value={Theme { accent: Color(29, 112, 98) }}>
    <Summary name={state.name.clone()} saved={state.saved} @reset={reset} />
</Provide>
```

Descendants call `scope.inject::<Theme>()`, returning Option<Arc<Theme>>. The nearest provider wins. Plain provider values are immutable. Provide Signal<T> or another owned reactive service when descendants should track changes. Application services/global styles can be provided by main.rs through Scope before mount.

## Reactivity, composables, and tasks

Composables are ordinary `.rs` modules. The use_ prefix is a convention. Each setup invocation should create independent owned state unless sharing is explicit. Signal::get tracks reads; set/update notify on changed values. Computed::new defines a lazy pure getter and get returns Result<T, ReactiveError>. Do not put I/O or mutations in computed getters. Concurrent evaluation returns Busy; recursive reads report Cycle.

Ordinary fields mutated by handlers also trigger publication. Use signals for reusable shared/computed dependencies. Arbitrary local Rust variables do not become reactive by syntax alone. Repeated native animation frames do not reconstruct component views.

```rust
context.spawn_task(
    move |cancellation| service_request(owned_input, cancellation),
    |state, result, _context| { /* apply the typed result on the coordinator */ },
)?;
```

Work must return Result<T,E> and own its inputs. Errors are TaskError::Failed(E) or Panicked. Handle TaskQueueError on admission. Child jobs inherit mount ownership; TaskHandle::cancel requests cooperative cancellation and discards late results. Cancellation does not forcibly terminate arbitrary Rust work. Superseding requests is explicit: cancel the previous handle or keep a request ID and check it when applying results.

## Styles and tooling

See the [compiled CSS guide](CSS-ENGINE.md) for implemented selectors, cascade, immutable tokens, public parts and responsive conditions. Unsupported syntax is diagnosed.

Load global CSS from main.rs before mount. One global `<style>` and one `<style scoped>` may coexist in a component. Source order within each sheet determines matching rules; scoped rules apply at authoring nodes and component-root style fallthrough, while globals match retained class names across the tree. Explicit Rust Style overrides win. See the CSS guide for the supported property matrix.

For runtime themes, create Stylesheet::new(rules), provide a clone through Scope, then call replace(new_rules) from an ordinary handler/service. The native host tracks its reads and republishes the tree. Typed theme tokens can be ordinary provided Rust structs containing signals.

The [VS Code client](../tooling/inrux-vscode/README.md) supports distinct lowercase/PascalCase tag colors, original-source framework diagnostics, imported component navigation, and atomic view/Rust/CSS formatting. Rust completion, hover, definitions and signature help use source-mapped generated modules in Cargo context; build the application once first. Available analyzer diagnostics map authored Rust ranges; full template-expression/type-diagnostic parity, rename and refactors remain open.

## View naming and planned animation refs

The outer `<view>` is the complete UI block in a .iru file. Nested `<view>` is the agreed direction for transparent groups and slot declarations; it is not accepted by the current compiler. `<Children />` is the default content outlet inside it. Legacy `<template>` and `<Slot />` remain aliases; opening and closing block names must match, and both UI spellings cannot coexist in one file. The formatter preserves either spelling.

Generation-safe NodeRef handles, a `node_ref` binding, keyed child/children targeting and bounded UI-thread animation commands are planned in [alpha status](STATUS.md). They are not available runtime APIs. Rust borrowed references and existing component-state handles do not provide native widget access.

### Bundled fonts

Register an application font with `WindowOptions.fonts.push(FontAsset::new("Noto Sans", include_bytes!("../assets/NotoSans-Regular.ttf").as_slice()))`, then use `font-family: "Noto Sans"` in CSS or `Style::new().font_family("Noto Sans")`. The family inherits through reusable components and is applied to native measurement and painting. Use the font's intrinsic family name and include its redistribution license. This subset accepts one named family; fallback lists, font weights/styles and @font-face are not implemented. See [the shared application](../playground/README.md).

Application source uses Inrux behavior/style contracts regardless of adapter. Native delegation and fallback selection stay private; this does not add engine-dependent authoring syntax or implement the planned composition/ref features.

## Structural layout components

Built-in Box, Container, Center and Spacer require no Rust component imports. They lower to real Column layout nodes, preserving ordinary children, styles and keys. See [the layout guide](LAYOUT-COMPONENTS.md) for width caps, centering, bounded Spacer growth and CSS/Rust examples. Component defaults are below application style layers.

## Views and layouts

Place screen-level components in src/views and shared application shells in src/layouts. Layouts are ordinary components receiving typed `children: Node` and rendering `<Children />`. [ShowcaseLayout](../playground/egui/src/layouts/ShowcaseLayout.iru) is reused by the standalone app views; [DemoPanel](../playground/egui/src/components/DemoPanel.iru) provides a smaller shared content shell. No router is required. Current children are eager default content; named/lazy/scoped content remains planned. Native Input accepts `:disabled={state.locked}` alongside its controlled/model value.

## Scrollable content

Use [ScrollArea](SCROLLING.md) for bounded content, with CSS/Rust overflow styles, normal keyed child controls, typed `@scroll` observations and revision-bearing `scroll_to` requests. Wheel-driven offsets remain UI-owned.

## Compiled CSS and style identity

Use `id="save"` for #save selectors, `class="action primary"` for reusable styling, and `key` for retained identity. These are independent. Scoped `.iru` styles and standalone `compile_stylesheet` inputs support compound type/ID/classes, lists, immutable :root tokens/var() and min/max width/height media conjunctions. Explicit Rust styles remain last. See [CSS engine](CSS-ENGINE.md) for the complete current subset, per-sheet token scope, build.rs examples and interaction-state capability limits.

## Headless components

Users own visual styling; framework components supply semantics, behavior and keyboard/pointer interaction with minimal overridable defaults. Application classes and scoped/global CSS remain the styling entry points. Initial public part/state customization is available through CSS ::part and Rust styled_part; further parts remain pending; see [the accepted contract and current gaps](HEADLESS-COMPONENTS.md). IPEX's component recipes are reference application styles, not built-in framework variants.
