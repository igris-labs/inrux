# TextInput and TextArea

`Input` (also spelled `TextInput`) edits one line. `TextArea` edits multiple lines with native wrapping and vertical scrolling. Both work in egui, Masonry and GPUI and use the same controlled-model acknowledgements, styling, placeholder and disabled contract.

```xml
<TextInput model:value={state.name} placeholder="Workspace name" class="input" />
<TextArea model:value={state.notes} placeholder="Notes" height={Size::Px(140.0)} width={Size::Fill} class="notes" />
```

Use `value` and `@input` for an explicitly controlled string, or `model:value` with an optional `@input` handler. Native edits stay on the UI thread; application handlers run through the existing worker/event channel. External resets preserve the edit-revision protocol. `disabled` rejects text and releases focus.

Rust constructors are `Node::input` / `Node::text_input` and `Node::text_area`, with `Node::model_input` / `Node::model_text_area` for revision-bearing events. Each accepts value, placeholder and event. The public implementations are in `components/input/input.rs` and `components/text_area/text_area.rs`. Both lower to the same backend-independent editing descriptor with an explicit multiline mode; native integration stays private.

Input converts hard line breaks to spaces in supplied values, clipboard and IME text. CRLF becomes one space. TextArea normalizes hard line separators to LF and preserves paragraphs; Enter inserts a newline. Tab moves focus rather than inserting indentation. TextArea wraps to its content width and scrolls vertically to reveal the active caret. Native selections, clipboard and IME geometry use the same translation as glyph painting. There is no public textarea resize handle in this increment.

Default allocated heights are 40 logical pixels for Input and 120 for TextArea; default intrinsic width is 220. Set dimensions with ordinary classes/Rust styles. A stylesheet class that sets Input's height also sets TextArea's height if applied to it, so use a separate textarea class when a taller editor is desired. Overflow properties remain container-only; editor scrolling is internal.

The Scrolling view in each playground includes an identical multiline model demo, line/character counts and disabled-state toggle. Existing single-line examples remain in Controls and Scrolling.

Masonry uses its pinned private Parley editor integration for both modes. GPUI uses native shaped paragraphs for wrap, pointer hit tests and IME geometry. egui uses its native multiline editor and scroll area. Full cross-platform accessibility/IME certification, undo/redo parity and advanced editor features remain outside the current prototype's validated scope.
