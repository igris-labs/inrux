# Compiled styling

Inrux owns selectors, tokens, cascade and responsive semantics. egui, Masonry and GPUI receive typed styles and geometry through the same adapter boundary. No native type enters the stylesheet IR.

## Authoring

```xml
<view>
    <Button id="save" class="action primary" text="Save" />
</view>
<rust>
use inrux::prelude::*;
pub struct Props {}
</rust>
<style scoped>
:root { --space: 12px; --ink: #262626; }
Button.action { padding: var(--space); color: var(--ink); }
#save.primary { border-radius: 6px; }
@media (max-width: 900px) {
    .action { padding: 8px; }
}
</style>
```

An ID is style metadata, independent of retention keys and refs. IDs are not currently checked for document-wide uniqueness. Selectors match the framework's case-sensitive component type: `Button`, `Card`, `Center`, etc.; they do not inspect native widget classes. `Input` and the `TextInput` template alias can be selected by their authored names; Rust `Node::input` reports `Input`. Custom component tags preserve their returned root's logical type. `Children` and dynamic component outlets preserve the returned node type too.

## Compile/load path

`.iru` style blocks compile into Rust `Rule` values. Standalone files can use `inrux_compiler::compile_stylesheet(css)` in `build.rs`, write its expression to OUT_DIR, and load it with `include!`. The three playground build scripts provide complete examples, including Cargo rebuild tracking and separate light/dark themes. No CSS parsing or token substitution takes place during their startup, theme toggle or rendering. Runtime `parse_css` remains available for user-loaded themes; parse once on load, then construct a `CompiledStyles` index.

Each immutable index anchors a rule by ID, first class, component type or universal bucket. Matching gathers candidates, checks the remaining compound conditions and viewport predicate, then orders by specificity and source order. It does not scan every unrelated class rule. The matched-index buffer still allocates for nonempty candidate sets; universal/type-heavy sheets and many rules on a common class can reduce the advantage.

Generated local sheets use OnceLock and Arc. Sessions retain the global index and read reactive themes through an Arc instead of cloning every rule each render. A theme change rebuilds the combined global index only when necessary; if there are no static globals it reuses the theme index directly. `Node::class(names, rules)` remains a convenience compatibility API which indexes its passed local rules; hot Rust construction paths should reuse `Arc<CompiledStyles>` with `.classes(names).with_styles(sheet.clone())`.

## Cascade and responsiveness

Precedence is component defaults → global sheet → component-local scoped sheets → explicit Rust/template style. Within a sheet, ID count wins, then class/interaction count, then type count, then later source order. Class attribute order does not affect precedence. Interaction layers replay in cascade order, so a low-specificity hover rule cannot overwrite a stronger base selector merely because it is conditional. Interaction rules remain paint-only and capability-checked: authored root and public-part interaction paint styles work in egui, Masonry and GPUI. Default native hover/focus affordances are separate. Conditional color affects the matched node; propagation of interaction-dependent inherited text colors to descendants remains future work. Scrollbar palette changes happen during publication and are rejected in state selectors.

Text color, font size, font family, white-space policy, accent-color and scrollbar palette inherit. Other declarations do not. `:style` remains an explicit final override. Scoped rules stay on their authored nodes and exposed child roots; they are not a global descendant selector mechanism.

All hosts report logical viewport dimensions through `ViewportState`, supplied in Scope. Unchanged reports do not invalidate. Actual changes notify the coordinator; it rebuilds the description and evaluates compiled media predicates before publication. This preserves retained component state. It is **not** per-property graph invalidation or a style/layout cache; the current native hosts still rebuild shared Taffy layout as previously documented.

## Supported syntax and boundaries

| Feature | Supported |
| --- | --- |
| Selectors | `*`, `Button`, `.class`, `#id`, compounds such as `Button#save.action.primary`, comma-separated selector lists |
| States | One trailing `:hover`, `:focus-visible`, `:disabled`, `:checked` or `:unchecked`; color/background/border/radius only; native support checked |
| Tokens | Top-level `:root { --name: value; }`, forward references, later overrides, nested `var(--name, fallback)`, cycle/missing-value diagnostics and bounded expansion |
| Media | `(min-width: Npx)`, `(max-width: Npx)`, min/max-height, `and`, nested intersections; inclusive bounds |
| Positioning | relative/absolute position, top/right/bottom/left px/percent/auto insets; fixed/sticky are not CSS values yet |
| Preformatted text | white-space: pre uses unwrapped shared shaping and preserves glyph overflow; normal uses common paragraph wrapping |
| Scrollbars | Auto indicators hide after 700 ms idle, reveal on edge hover/drag; overflow:scroll keeps them visible. Inherited `scrollbar-color: #thumb #track`; Rust `.scrollbar_colors(thumb, track)`; all three engines |
| Existing properties | Uniform padding; one/two-value gap, row-gap and column-gap; inherited accent-color; dimensions and min/max constraints; flex alignment/direction/wrap/growth; equal-column grid; aspect ratio; overflow; background/color/borders/radius; physical margin shorthand/longhands; named font family and font size |

Tokens are **immutable stylesheet constants**, not browser DOM-inherited custom properties. Each compiled sheet resolves its own constants; component scoped sheets do not read token definitions from a separate global sheet. Compose token definitions into that sheet before compiling, or use Rust theme values. Theme tokens may differ between complete generated light/dark sheets.

Combinators, attribute selectors, `:is`, `:not`, `:has`, arbitrary pseudo-elements (public ::part targets are supported), selector escapes, `!important`, media OR/not/screen/orientation, container queries, arbitrary custom properties, calc, CSS layers, transforms, transitions, animation, subtree opacity and browser-complete typography/grid remain unsupported. Unsupported syntax returns an error. The subset uses ASCII selector identifiers and literal CSS colors compiled to sRGBA8; see [color values](STYLE-COLORS.md) for supported forms and gamut limits. Non-editor focus-visible paint now distinguishes pointer activation from keyboard interaction; native editor caret/focus feedback remains visible. Broader accessibility/programmatic/platform modality behavior is not a browser-complete heuristic. Borders paint inside the box without reserving layout space. Plain surfaces have square corners; rounding is explicit in components/styles. The documentation rail is an absolute sibling outside the scroller, not CSS position:fixed. Native editors retain their own single/multiline behavior.

`Selector` also contains an optional typed `part`; struct literals must initialize it or use Default. `Rule` now contains `selector`, `media`, `state`, and `style`; code constructing the old `Rule { class, ... }` literal must migrate. `parse_css`, `.class(...)` and CSS/Rust declaration parity remain available. This is an unpublished 0.1 API change.

## Measurement

Run `cargo run -p inrux-style --release --example style_bench --locked` from the repository root. This synthetic benchmark compares indexed selector matching with exhaustive matching on identical typed rules. It does not measure startup, memory, native translation, layout, painting or frame latency. Publish your hardware, toolchain and workload with results.

## Control accents and independent gaps

`gap: 16px 12px` means row-gap 16px and column-gap 12px. A one-value gap sets both axes. Longhands override their own axis; a later shorthand resets both axes according to ordinary specificity and source order. Rust uses `.gap(value)`, `.row_gap(value)` and `.column_gap(value)` with the same overlay behavior. Gaps must be finite and nonnegative. There is no percentage-gap support.

`accent-color: #705bea` is inherited and available as `.accent_color(Color(112, 91, 234))`. Checkbox, Switch, Progress and Meter consume it through shared palette policy. It accepts the supported literal color syntax. Accent changes resolve at publication; accent and gap declarations are rejected in interaction selectors. See [control APIs and semantic limits](BOOLEAN-AND-RANGE-COMPONENTS.md).

## Headless styling foundation

[Public styling parts](STYLING-PARTS.md) expose boolean-control surfaces/labels, range fills and editor placeholder ink without native handles. CSS `Switch::part(track):checked` and Rust `styled_part(Part::Track, style)` share the cascade. Root and part paint states work in all three engines; geometry in conditions is rejected. [Literal colors](STYLE-COLORS.md) now include alpha and OKLCH, with native alpha preserved. Physical margins support one-to-four-value shorthand, signed px longhands and existing inline-auto with source-order parity. Uniform padding, richer text/grid/layout, local variables and further paint/motion remain separate gaps.

