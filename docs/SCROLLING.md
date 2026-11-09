# Clipping and ScrollArea

ScrollArea is one shared composition of Column, implemented in `inrux-components/src/components/scroll_area/scroll_area.rs`. The three bindings execute the same viewport policy using native paint clips, hit testing, focus and pointer capture. No engine-specific public component or engine branch is needed in application code.

```xml
<ScrollArea height={Size::Px(240.0)} class="messages">
    <Input key="draft" model:value={state.draft} />
    <For each={state.messages.iter()} item="message" key={message.id}>
        <Text :text={message.text.clone()} />
    </For>
</ScrollArea>
```

The default is full width, 240 logical pixels high, vertical `auto` overflow and horizontal `hidden` overflow. Dimensions, gap, padding, classes and explicit Rust styles override defaults normally. Physical Box/Column/Row containers can also use overflow styles; overflow is rejected on native control leaves. Bound the viewport's height/width explicitly when it must be smaller than its content.

```css
.messages { padding: 12px; gap: 8px; overflow-y: auto; }
.canvas { overflow: auto; }
.crop { overflow: hidden; }
```

Equivalent Rust: `Style::new().overflow(Overflow::Auto)`, or separate `overflow_x` and `overflow_y` setters. Supported values are `visible`, `hidden`, `auto`, `scroll`; two-value shorthand and `clip` are rejected. A visible axis paired with a non-visible axis computes to auto. These geometry properties are rejected in interaction-state selectors.

Layout providers return `LayoutBox::content_width/content_height` separately from viewport size. The selected Taffy provider preserves direct children's natural main-axis sizes inside scrollable containers; children are not squeezed to make overflowing content fit. Overflow hidden clips without exposing user/programmatic scrolling on that axis. Clipping is rectangular, including pointer exclusion outside ancestor clip intersections; a surface's border radius does not round its clip.

## Commands and observations

```xml
<ScrollArea :scroll_to={state.request} @scroll={observe}>
    <!-- Ordinary controls and keyed children -->
</ScrollArea>
```

Import `ScrollRequest`, `ScrollOffset` and `ScrollMetrics` from `inrux` or its prelude. A command is `ScrollRequest { revision: 1, offset: ScrollOffset { x: 0.0, y: 600.0 } }`. Increment its revision for each new command. The adapter applies a revision once and clamps the position; repeated application snapshots do not undo subsequent wheel movement. Non-finite coordinates are rejected before native allocation. To reach the end without knowing its length, use a finite large destination such as `f32::MAX`.

A typed function handler receives `(state: &mut State, metrics: ScrollMetrics, ctx: &Context<State>)`. Metrics contain offset, viewport width/height and content width/height. The observer runs through the normal bounded application event channel; native wheel handling remains on the UI thread. Notifications occur only when metrics change. Do not create a new command for every observation, which would turn an observation into a feedback loop. Stateful components using an Action enum must use a function-handler component for typed scroll observations in this increment. Rust-built nodes use `.on_scroll(Emit<ScrollMetrics>)` and `.scroll_to(ScrollRequest)`.

The complete modular example is [Scrolling.iru](../playground/egui/src/views/Scrolling.iru), with handlers in [scrolling.rs](../playground/egui/src/features/scrolling.rs); identical portable sources live in the Masonry and GPUI projects.

## Input and lifetime

- Wheel input is routed to the deepest viewport that can move; an entirely blocked event bubbles to an ancestor. Partially consumed diagonal deltas are not split. Native pixel gestures use logical pixels; line/key steps are 40 logical pixels. Shift maps vertical-only wheel input to horizontal; native already-horizontal Shift gestures retain their axis, and vertical wheel input also operates a horizontal-only pane.
- A focused viewport supports arrows, PageUp/PageDown, Home/End. Native editable children retain their own editing keys. Focus transitions reveal offscreen controls; manually scrolling while an editor retains focus does not repeatedly snap back.
- Overlay scrollbars share track/thumb, bounds and drag math. They reserve no layout gutter. Native pointer capture continues thumb dragging beyond its track.
- Stable keyed/generation-bearing nodes retain UI-owned offsets; omission or a new mount resets them. Resizing/content changes clamp offsets without replacing matching child controls.

`Service::Clipping` advertises native rectangular paint/input clipping, and `Service::Scrolling` advertises the shared Inrux fallback policy with queries/commands. Neither label claims complete accessibility semantics or a provider performance ranking. Masonry focus reveal uses shared framework geometry rather than its current native ancestor pan-request path.

Virtualization, scroll anchoring, smooth programmatic motion, overscroll effects, rounded clipping, reserved gutters, full accessible scrollbar actions, generation-safe refs/focus commands and cross-platform/IME conformance remain separate work. Long content is laid out and retained in full; this is not a virtualized List implementation.

## Native wheel feel and overflowing inputs

November 6 correction: egui consumes its native `smooth_scroll_delta`, which passes fine trackpad samples through and interpolates coarse wheel steps. Reading raw events bypassed that native behavior. Shift is normalized once by egui; the host uses the shared 40px line distance. Masonry/GPUI preserve their native precise pixel streams and platform momentum. No second synthetic inertia loop or always-running timer is added. Easing curves are not promised to be identical across engines.

Single-line Input keeps its allocated width and clips its content. Horizontal trackpad/wheel gestures (or Shift plus a vertical wheel) pan long text; typing and caret navigation reveal the active caret. Manual panning persists while the caret stays still. Vertical wheel input reaches an enclosing ScrollArea. Home/End, selection, model replacement and resize keep native editing services and bounded offsets.

Use the Long text button in each playground's Scrolling view to reproduce.

The private egui input delegates clipping/panning/caret reveal to a native horizontal ScrollArea and TextEdit. GPUI uses retained text translation with the shared `editing::TextViewport` helper. Masonry 0.4.0 has no public caret-scroll API; its private input binding adapts the pinned native TextArea/field source, preserving Parley editing/selection/clipboard/IME logic and adding viewport translation to paint, pointer, IME and semantic geometry. Its source attribution is in `packages/inrux-masonry/NOTICE`. Sync this private adaptation against upstream when upgrading Masonry; it is not a second public Inrux component.

## System direction and text areas

Natural/reversed direction is owned by the operating system. Winit/egui and GPUI macOS receive already-adjusted signed scroll deltas; Inrux converts content displacement to positive viewport offsets once and never applies a separate natural-scroll inversion. Pixel gestures preserve their native units (with DPI conversion where required); coarse line events retain the common line-step policy. Synthetic signed-event regressions cover both signs and already-normalized Shift gestures. These tests do not certify physical mouse/trackpad behavior after toggling the global OS setting.

[TextArea](TEXT-EDITING.md) provides wrapped multiline editing with internal vertical scrolling. Input/TextInput remain single-line even when supplied or pasted text contains line breaks.

## Overlay indicator visibility and content gutters

Auto-overflow indicators are hidden at rest, reveal during movement for 700 ms or on edge hover/drag, and keep their hit targets discoverable while hidden. `overflow: scroll` retains persistent indicators. Shared indicator activity/coalescing lives in adapter/services/indicators.rs. egui schedules an expiry repaint, GPUI coalesces one weak expiry task per viewport, and Masonry requests animation passes only during the finite activity window. There is no permanent indicator animation loop.

Indicators overlay the viewport; they do not change layout width. Reserve inner content padding when controls extend to its edge, as the documentation sidebar does, so an indicator never covers button hover/focus bounds. Keyboard navigation, nested routing and OS-adjusted natural-scroll direction are unchanged. `white-space: pre` preserves unwrapped glyph extents for horizontal code viewports; native editors continue to own their editing viewports.

The sidebar scrollport now spans the full pane width. Padding lives on its header/menu/footer content wrappers, so it protects button bounds without moving the overlay track inward. Keyboard-visible focus decoration is independent of logical focus; mouse selection does not leave a focus outline, while keyboard interaction restores it.
