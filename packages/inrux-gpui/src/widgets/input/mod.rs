//! Retained native text editing with shared controlled-model acknowledgements.
mod caret;
mod editing;
mod element;
mod layout;
mod multiline;
mod platform;
use super::Failure;
use element::TextElement;
use gpui::{actions, prelude::*, *};
use inrux_adapter::editing::Editor;
use inrux_components::{InputEvent, Node, NodeKind};
use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation;
actions!(
    inrux_text_input,
    [
        Backspace,
        Delete,
        Left,
        Right,
        SelectLeft,
        SelectRight,
        SelectAll,
        Home,
        End,
        SelectHome,
        SelectEnd,
        ShowCharacterPalette,
        Paste,
        Cut,
        Copy,
        DocumentStart,
        DocumentEnd,
        SelectDocumentStart,
        SelectDocumentEnd,
        Newline,
        Up,
        Down,
        SelectUp,
        SelectDown
    ]
);
/// Native editor state stays on the UI thread and is retained by logical key/generation.
pub(crate) struct TextInput {
    focus_handle: FocusHandle,
    content: SharedString,
    placeholder: SharedString,
    placeholder_ink: inrux_style::Color,
    selected_range: Range<usize>,
    selection_reversed: bool,
    marked_range: Option<Range<usize>>,
    viewport: inrux_adapter::editing::TextViewport,
    last_layout: Option<layout::TextLayout>,
    multiline: bool,
    vertical: inrux_adapter::editing::TextViewport,
    last_bounds: Option<Bounds<Pixels>>,
    is_selecting: bool,
    editor: Editor,
    event: InputEvent,
    style: inrux_style::Style,
    disabled: bool,
    failure: Failure,
    caret_visible: bool,
    caret_task: Option<Task<()>>,
}
impl TextInput {
    /// Construct a version-aware editor from an already validated Input description.
    pub(crate) fn new(node: &Node, failure: Failure, cx: &mut Context<Self>) -> Self {
        let (value, placeholder, event) = match &node.kind {
            NodeKind::Input {
                value,
                placeholder,
                event,
                ..
            } => (value.clone(), placeholder.clone(), event.clone()),
            _ => unreachable!("Input translation requires an Input node"),
        };
        Self {
            focus_handle: cx.focus_handle().tab_stop(true),
            selected_range: value.len()..value.len(),
            content: value.clone().into(),
            placeholder: placeholder.into(),
            placeholder_ink: inrux_adapter::control_parts::placeholder(node, false, false),
            selection_reversed: false,
            marked_range: None,
            viewport: Default::default(),
            vertical: Default::default(),
            multiline: matches!(
                node.kind,
                NodeKind::Input {
                    multiline: true,
                    ..
                }
            ),
            last_layout: None,
            last_bounds: None,
            is_selecting: false,
            editor: Editor::new(&value, event.applied()),
            event,
            style: node.style(),
            disabled: node.disabled,
            failure,
            caret_visible: true,
            caret_task: None,
        }
    }
    /// Apply acknowledgements without replacing pending typing, focus, selection or preedit text.
    pub(crate) fn reconcile(&mut self, node: &Node, cx: &mut Context<Self>) {
        let NodeKind::Input {
            value,
            placeholder,
            event,
            ..
        } = &node.kind
        else {
            return;
        };
        let old_content = self.content.clone();
        let placeholder_ink = inrux_adapter::control_parts::placeholder(node, false, false);
        let changed_props = self.placeholder_ink != placeholder_ink
            || self.placeholder.as_ref() != placeholder
            || self.style != node.style()
            || self.disabled != node.disabled;
        let previous = self.editor.value.clone();
        self.editor.reconcile(value, event.applied());
        if (node.disabled || self.marked_range.is_none() || self.editor.value != previous)
            && self.content.as_ref() != self.editor.value
        {
            self.content = self.editor.value.clone().into();
            let end = self.selected_range.end.min(self.content.len());
            let mut boundary = end;
            while !self.content.is_char_boundary(boundary) {
                boundary -= 1;
            }
            self.selected_range = boundary..boundary;
            self.marked_range = None;
        }
        self.placeholder = placeholder.clone().into();
        self.placeholder_ink = placeholder_ink;
        self.event = event.clone();
        self.style = node.style();
        self.disabled = node.disabled;
        self.multiline = matches!(
            node.kind,
            NodeKind::Input {
                multiline: true,
                ..
            }
        );
        if changed_props || self.content != old_content {
            self.restart_caret();
            cx.notify();
        }
    }
    /// Queue a committed edit and roll it back if the framework cannot accept the command.
    fn publish(&mut self, cx: &mut Context<Self>) {
        self.restart_caret();
        let previous = self.editor.value.clone();
        self.editor.value = self.content.to_string();
        let revision = self.editor.edited();
        if let Err(error) = self.event.emit(revision, self.editor.value.clone()) {
            self.editor.reject(previous);
            self.content = self.editor.value.clone().into();
            self.selected_range = self.content.len()..self.content.len();
            *self.failure.borrow_mut() =
                Some(inrux_adapter::AdapterError::Application(error.to_string()));
        }
        cx.notify();
    }
}
impl Render for TextInput {
    /// Native input services handle text; appearance comes from the resolved Inrux style.
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.disabled && self.focus_handle.is_focused(window) {
            window.blur();
        }
        self.sync_caret(window, cx);
        let mut view = div()
            .size_full()
            .flex()
            .when(!self.multiline, |view| view.items_center())
            .when(self.multiline, |view| {
                view.py(px(self.style.padding.unwrap_or(8.0)))
            })
            .px(px(self.style.padding.unwrap_or(12.0)))
            .overflow_hidden()
            .key_context("InruxInput")
            .cursor(CursorStyle::IBeam)
            .font_family(
                self.style
                    .font_family
                    .clone()
                    .unwrap_or_else(|| ".SystemUIFont".into()),
            )
            .text_size(px(self.style.font_size.unwrap_or(15.0)))
            .line_height(px(
                self.style.font_size.unwrap_or(15.0) * inrux_adapter::controls::LINE_HEIGHT
            ))
            .text_color(crate::widgets::surface::color(
                self.style.color.unwrap_or(inrux_style::Color(27, 30, 33)),
            ))
            .on_action(cx.listener(Self::backspace))
            .on_action(cx.listener(Self::delete))
            .on_action(cx.listener(Self::left))
            .on_action(cx.listener(Self::right))
            .on_action(cx.listener(Self::select_left))
            .on_action(cx.listener(Self::select_right))
            .on_action(cx.listener(Self::select_all))
            .on_action(cx.listener(Self::home))
            .on_action(cx.listener(Self::end))
            .on_action(cx.listener(Self::select_home))
            .on_action(cx.listener(Self::select_end))
            .on_action(cx.listener(Self::show_character_palette))
            .on_action(cx.listener(Self::paste))
            .on_action(cx.listener(Self::cut))
            .on_action(cx.listener(Self::copy))
            .on_action(cx.listener(Self::newline))
            .on_action(cx.listener(Self::document_start))
            .on_action(cx.listener(Self::document_end))
            .on_action(cx.listener(Self::select_document_start))
            .on_action(cx.listener(Self::select_document_end))
            .on_action(cx.listener(Self::up))
            .on_action(cx.listener(Self::down))
            .on_action(cx.listener(Self::select_up))
            .on_action(cx.listener(Self::select_down))
            .on_scroll_wheel(cx.listener(|input, event: &ScrollWheelEvent, window, cx| {
                let delta = event
                    .delta
                    .pixel_delta(px(inrux_adapter::services::scroll::LINE_STEP));
                let horizontal = if event.modifiers.shift && delta.x == px(0.0) {
                    delta.y
                } else {
                    delta.x
                };
                let vertical = if input.multiline && !event.modifiers.shift {
                    -f32::from(delta.y)
                } else {
                    0.0
                };
                if !input.disabled
                    && (input.viewport.pan(-f32::from(horizontal)) | input.vertical.pan(vertical))
                {
                    cx.stop_propagation();
                    window.refresh();
                }
            }))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_move(cx.listener(Self::on_mouse_move))
            .child(TextElement { input: cx.entity() });
        if !self.disabled {
            view = view.track_focus(&self.focus_handle);
        }
        view
    }
}
/// Register conventional shortcuts inside the input context, leaving Tab to the host.
pub(crate) fn bind_keys(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("enter", Newline, Some("InruxInput")),
        KeyBinding::new("up", Up, Some("InruxInput")),
        KeyBinding::new("down", Down, Some("InruxInput")),
        KeyBinding::new("shift-up", SelectUp, Some("InruxInput")),
        KeyBinding::new("shift-down", SelectDown, Some("InruxInput")),
        KeyBinding::new("backspace", Backspace, Some("InruxInput")),
        KeyBinding::new("delete", Delete, Some("InruxInput")),
        KeyBinding::new("left", Left, Some("InruxInput")),
        KeyBinding::new("right", Right, Some("InruxInput")),
        KeyBinding::new("shift-left", SelectLeft, Some("InruxInput")),
        KeyBinding::new("shift-right", SelectRight, Some("InruxInput")),
        KeyBinding::new("home", Home, Some("InruxInput")),
        KeyBinding::new("end", End, Some("InruxInput")),
        KeyBinding::new("shift-home", SelectHome, Some("InruxInput")),
        KeyBinding::new("shift-end", SelectEnd, Some("InruxInput")),
        KeyBinding::new(
            if cfg!(target_os = "macos") {
                "cmd-a"
            } else {
                "ctrl-a"
            },
            SelectAll,
            Some("InruxInput"),
        ),
        KeyBinding::new(
            if cfg!(target_os = "macos") {
                "cmd-v"
            } else {
                "ctrl-v"
            },
            Paste,
            Some("InruxInput"),
        ),
        KeyBinding::new(
            if cfg!(target_os = "macos") {
                "cmd-c"
            } else {
                "ctrl-c"
            },
            Copy,
            Some("InruxInput"),
        ),
        KeyBinding::new(
            if cfg!(target_os = "macos") {
                "cmd-x"
            } else {
                "ctrl-x"
            },
            Cut,
            Some("InruxInput"),
        ),
    ]);
    cx.bind_keys([
        KeyBinding::new("ctrl-home", DocumentStart, Some("InruxInput")),
        KeyBinding::new("ctrl-end", DocumentEnd, Some("InruxInput")),
        KeyBinding::new("ctrl-shift-home", SelectDocumentStart, Some("InruxInput")),
        KeyBinding::new("ctrl-shift-end", SelectDocumentEnd, Some("InruxInput")),
    ]);
    #[cfg(target_os = "macos")]
    cx.bind_keys([
        KeyBinding::new("cmd-up", DocumentStart, Some("InruxInput")),
        KeyBinding::new("cmd-down", DocumentEnd, Some("InruxInput")),
        KeyBinding::new("cmd-shift-up", SelectDocumentStart, Some("InruxInput")),
        KeyBinding::new("cmd-shift-down", SelectDocumentEnd, Some("InruxInput")),
        KeyBinding::new("cmd-left", Home, Some("InruxInput")),
        KeyBinding::new("cmd-right", End, Some("InruxInput")),
        KeyBinding::new("cmd-shift-left", SelectHome, Some("InruxInput")),
        KeyBinding::new("cmd-shift-right", SelectEnd, Some("InruxInput")),
        KeyBinding::new("ctrl-cmd-space", ShowCharacterPalette, Some("InruxInput")),
    ]);
}

#[cfg(test)]
mod tests;
