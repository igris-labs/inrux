// Copyright 2018 the Xilem Authors and the Druid Authors
// SPDX-License-Identifier: Apache-2.0
// Adapted from Masonry 0.4.0; native editing retained with horizontal viewport support.

use masonry::widgets::{InsertNewline, TextAction};
mod events;
mod rendering;
use std::any::TypeId;

use masonry::accesskit::{Node, NodeId, Role};
use masonry::parley::PlainEditor;
use masonry::parley::editor::{Generation, SplitString};
use masonry::vello::Scene;
use masonry::vello::kurbo::{Affine, Point, Rect, Size};
use masonry::vello::peniko::Fill;

use masonry::core::keyboard::{Key, KeyState, NamedKey};
use masonry::core::{
    AccessCtx, AccessEvent, BoxConstraints, BrushIndex, ChildrenIds, CursorIcon, EventCtx, Ime,
    LayoutCtx, PaintCtx, PointerButton, PointerButtonEvent, PointerEvent, PointerUpdate,
    PropertiesMut, PropertiesRef, QueryCtx, RegisterCtx, StyleProperty, TextEvent, Update,
    UpdateCtx, Widget, WidgetId, WidgetMut,
};
use masonry::properties::{
    CaretColor, ContentColor, DisabledContentColor, SelectionColor, UnfocusedSelectionColor,
};
use masonry::theme::default_text_styles;
use masonry::util::debug_panic;
use masonry::{accesskit, theme};

/// `TextArea` implements the core of interactive text.
///
/// It is used to implement [`TextInput`](super::TextInput) and [`Prose`](super::Prose).
/// It is rare that you will use a raw `TextArea` as a widget in your app; most users
/// should prefer one of those wrappers.
///
/// This ensures that the editable and read-only text have the same text selection and
/// copy/paste behaviour.
///
/// The `USER_EDITABLE` const generic parameter determines whether the text area's contents can be
/// edited by the user of the app.
/// This is true for `TextInput` and false for `Prose`.
///
/// This widget emits [`TextAction`] only when `USER_EDITABLE` is true.
///
/// The exact semantics of how much horizontal space this widget takes up has not been determined.
/// In particular, this has consequences when the text alignment is set.
pub struct TextArea<const USER_EDITABLE: bool> {
    /// The underlying `PlainEditor`, which provides a high-level interface for us to dispatch into.
    editor: PlainEditor<BrushIndex>,
    fonts: masonry::parley::FontContext,
    viewport: inrux_adapter::editing::TextViewport,
    vertical: inrux_adapter::editing::TextViewport,
    /// The generation of `editor` which we have rendered.
    ///
    /// Mirrors the pinned native editor generation used for editing admission.
    rendered_generation: Generation,

    /// Whether to wrap words in this area.
    ///
    /// Note that if clipping is desired, that should be added by the parent widget.
    /// Can be set using [`set_word_wrap`](Self::set_word_wrap).
    word_wrap: bool,
    /// The amount of horizontal space available when [layout](Widget::layout) was
    /// last performed.
    ///
    /// If word wrapping is enabled, we use this for line breaking.
    /// We store this to avoid redoing work in layout and to set the
    /// width when `word_wrap` is re-enabled.
    last_available_width: Option<f32>,

    /// What key combination should trigger a newline insertion.
    /// If this is set to `InsertNewline::OnEnter` then `Enter` will insert a newline and _not_ trigger a [`TextAction::Entered`] event.
    insert_newline: InsertNewline,

    /// Whether to show the cursor, used for the blink animation.
    anim_cursor_visible: bool,

    /// Previous interval (ms), used for the cursor's blink animation.
    anim_prev_interval: u64,

    /// Time elapsed (ms) to calculate the timeout of the cursor's blink animation.
    anim_elapsed: u64,
}

// --- MARK: BUILDERS
impl TextArea<true> {
    /// Create a new `TextArea` which can be edited.
    ///
    /// Useful for creating a styled [`TextInput`](super::TextInput).
    // This is written out fully to appease rust-analyzer; StyleProperty is imported but not recognised.
    /// To change the font size, use `with_style`, setting [`StyleProperty::FontSize`](parley::StyleProperty::FontSize).
    pub fn new_editable(text: &str) -> Self {
        Self::new(text)
    }
}

impl TextArea<false> {}

impl<const EDITABLE: bool> TextArea<EDITABLE> {
    /// Create a new `TextArea` with the given text and default settings.
    ///
    // This is written out fully to appease rust-analyzer; StyleProperty is imported but not recognised.
    /// To change the font size, use `with_style`, setting [`StyleProperty::FontSize`](parley::StyleProperty::FontSize).
    pub fn new(text: &str) -> Self {
        let mut editor = PlainEditor::new(theme::TEXT_SIZE_NORMAL);
        editor.set_quantize(false);
        default_text_styles(editor.edit_styles());
        editor.set_text(text);
        Self {
            editor,
            fonts: inrux_text::font_context(),
            viewport: Default::default(),
            vertical: Default::default(),
            rendered_generation: Generation::default(),
            word_wrap: true,
            last_available_width: None,
            insert_newline: InsertNewline::default(),
            anim_cursor_visible: true,
            anim_prev_interval: 0,
            anim_elapsed: 0,
        }
    }

    /// Get the current text of this text area.
    ///
    /// To update the text of an active text area, use [`reset_text`](Self::reset_text).
    ///
    /// The return value is not just `&str` to handle IME preedits.
    pub fn text(&self) -> SplitString<'_> {
        self.editor.text()
    }

    /// Check if this text area holds nothing, including IME preedit content.
    pub fn is_empty(&self) -> bool {
        self.editor.raw_text().is_empty()
    }

    /// Set a style property for the new text area.
    ///
    /// Style properties set by this method include [text size](parley::StyleProperty::FontSize),
    /// [font family](parley::StyleProperty::FontStack), [font weight](parley::StyleProperty::FontWeight),
    /// and [variable font parameters](parley::StyleProperty::FontVariations).
    /// The styles inserted here apply to the entire text; we currently do not
    /// support inline rich text.
    ///
    /// Setting [`StyleProperty::Brush`](parley::StyleProperty::Brush) is not supported.
    /// Use [`ContentColor`] and [`DisabledContentColor`] properties instead.
    /// This is also not additive for [font stacks](parley::StyleProperty::FontStack), and
    /// instead overwrites any previous font stack.
    ///
    /// To set a style property on an active text area, use [`insert_style`](Self::insert_style).
    #[track_caller]
    pub fn with_style(mut self, property: impl Into<StyleProperty>) -> Self {
        self.insert_style_inner(property.into());
        self
    }

    /// Control [word wrapping](https://en.wikipedia.org/wiki/Line_wrap_and_word_wrap) for the new text area.
    ///
    /// When enabled, the text will be laid out to fit within the available width.
    /// If word wrapping is disabled, the text will likely flow past the available area.
    /// Note that parent widgets will often clip this, so the overflow will not be visible.
    ///
    /// Inrux Input uses unwrapped text; TextArea enables wrapping and newline insertion.
    ///
    /// To modify this on an active text area, use [`set_word_wrap`](Self::set_word_wrap).
    pub fn with_word_wrap(mut self, wrap_words: bool) -> Self {
        self.word_wrap = wrap_words;
        self.insert_newline = if wrap_words {
            InsertNewline::OnEnter
        } else {
            InsertNewline::Never
        };
        self
    }

    /// Shared logic between `with_style` and `insert_style`
    #[track_caller]
    fn insert_style_inner(&mut self, property: StyleProperty) -> Option<StyleProperty> {
        if let StyleProperty::Brush(idx @ BrushIndex(1..))
        | StyleProperty::UnderlineBrush(Some(idx @ BrushIndex(1..)))
        | StyleProperty::StrikethroughBrush(Some(idx @ BrushIndex(1..))) = &property
        {
            debug_panic!(
                "Can't set a non-zero brush index ({idx:?}) on a `TextArea`, as it only supports global styling.\n\
                To modify the active brush, use `set_brush` or `with_brush` instead"
            );
            None
        } else {
            self.editor.edit_styles().insert(property)
        }
    }
}

// --- MARK: HELPERS
impl<const EDITABLE: bool> TextArea<EDITABLE> {
    /// Query the retained horizontal translation in native conformance tests.
    #[cfg(test)]
    pub(crate) fn horizontal_offset(&self) -> f32 {
        self.viewport.offset()
    }
    /// Single-line containers center their text; multiline containers retain top padding.
    pub(crate) fn multiline(&self) -> bool {
        self.word_wrap
    }
    /// Get the IME area from the editor, accounting for padding.
    ///
    /// This should only be called when the editor layout is available.
    pub(crate) fn ime_area(&self) -> Rect {
        debug_assert!(
            self.editor.try_layout().is_some(),
            "TextArea::ime_area should only be called when the editor layout is available"
        );
        let layout = self
            .editor
            .try_layout()
            .expect("IME geometry requires layout");
        let caret =
            inrux_text::caret_rect(layout, self.editor.raw_selection().focus().index(), 1.5);
        let rect = Rect::new(
            caret[0].into(),
            caret[1].into(),
            (caret[0] + caret[2]).into(),
            (caret[1] + caret[3]).into(),
        ) - masonry::kurbo::Vec2::new(
            self.viewport.offset().into(),
            self.vertical.offset().into(),
        );
        Rect::new(
            rect.x0.max(0.0),
            rect.y0,
            rect.x1.min(f64::from(self.viewport.width())).max(0.0),
            rect.y1,
        )
    }
}

// --- MARK: WIDGETMUT
impl<const EDITABLE: bool> TextArea<EDITABLE> {
    /// Set font styling for an active text area.
    ///
    /// Style properties set by this method include [text size](parley::StyleProperty::FontSize),
    /// [font family](parley::StyleProperty::FontStack), [font weight](parley::StyleProperty::FontWeight),
    /// and [variable font parameters](parley::StyleProperty::FontVariations).
    /// The styles inserted here apply to the entire text; we currently do not
    /// support inline rich text.
    ///
    /// Setting [`StyleProperty::Brush`](parley::StyleProperty::Brush) is not supported.
    /// Use [`ContentColor`] and [`DisabledContentColor`] properties instead.
    /// This is also not additive for [font stacks](parley::StyleProperty::FontStack), and
    /// instead overwrites any previous font stack.
    ///
    /// This is the runtime equivalent of [`with_style`](Self::with_style).
    #[track_caller]
    pub fn insert_style(
        this: &mut WidgetMut<'_, Self>,
        property: impl Into<StyleProperty>,
    ) -> Option<StyleProperty> {
        let old = this.widget.insert_style_inner(property.into());

        this.ctx.request_layout();
        old
    }

    /// Set the text displayed in this widget.
    ///
    /// This is likely to be disruptive if the user is focused on this widget,
    /// as it does not retain selections, and may cause undesirable interactions with IME.
    pub fn reset_text(this: &mut WidgetMut<'_, Self>, new_text: &str) {
        // If the IME is currently composing, we need to clear the compose first. This is quite
        // disruptive, but we've warned about that. The platform's state is not reset, and the
        // preedit will show up again when the platform updates it.
        if this.widget.editor.is_composing() {
            let (_, lctx) = this.ctx.text_contexts();
            let fctx = &mut this.widget.fonts;
            this.widget.editor.driver(fctx, lctx).clear_compose();
        }
        this.widget.editor.set_text(new_text);

        let (_, lctx) = this.ctx.text_contexts();
        let fctx = &mut this.widget.fonts;
        this.widget.editor.driver(fctx, lctx).move_to_text_end();

        this.ctx.request_layout();
    }

    /// Set the selection to the given byte range.
    ///
    /// No-op if either index is not a char boundary.
    #[cfg(test)]
    pub fn select_byte_range(this: &mut WidgetMut<'_, Self>, start: usize, end: usize) {
        let (_, lctx) = this.ctx.text_contexts();
        let fctx = &mut this.widget.fonts;
        this.widget
            .editor
            .driver(fctx, lctx)
            .select_byte_range(start, end);
        this.ctx.request_layout();
        this.ctx.request_render();
    }
}

impl<const EDITABLE: bool> Widget for TextArea<EDITABLE> {
    type Action = TextAction;

    fn on_anim_frame(
        &mut self,
        ctx: &mut UpdateCtx<'_>,
        props: &mut PropertiesMut<'_>,
        interval: u64,
    ) {
        self.handle_on_anim_frame(ctx, props, interval)
    }

    fn on_pointer_event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        props: &mut PropertiesMut<'_>,
        event: &PointerEvent,
    ) {
        self.handle_on_pointer_event(ctx, props, event)
    }

    fn on_text_event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        props: &mut PropertiesMut<'_>,
        event: &TextEvent,
    ) {
        self.handle_on_text_event(ctx, props, event)
    }

    fn accepts_focus(&self) -> bool {
        EDITABLE
    }

    fn accepts_text_input(&self) -> bool {
        EDITABLE
    }

    fn on_access_event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        props: &mut PropertiesMut<'_>,
        event: &AccessEvent,
    ) {
        self.handle_on_access_event(ctx, props, event)
    }

    fn register_children(&mut self, _ctx: &mut RegisterCtx<'_>) {}

    fn property_changed(&mut self, ctx: &mut UpdateCtx<'_>, property_type: TypeId) {
        CaretColor::prop_changed(ctx, property_type);
        ContentColor::prop_changed(ctx, property_type);
        DisabledContentColor::prop_changed(ctx, property_type);
        SelectionColor::prop_changed(ctx, property_type);
        UnfocusedSelectionColor::prop_changed(ctx, property_type);
    }

    fn update(&mut self, ctx: &mut UpdateCtx<'_>, _props: &mut PropertiesMut<'_>, event: &Update) {
        match event {
            Update::FocusChanged(_) => {
                ctx.request_layout();
                ctx.request_render();
            }
            Update::DisabledChanged(_) => {
                // We might need to use the disabled brush, and stop displaying the selection.
                ctx.request_layout();
                ctx.request_render();
            }
            _ => {}
        }
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx<'_>,
        props: &mut PropertiesMut<'_>,
        bc: &BoxConstraints,
    ) -> Size {
        self.handle_layout(ctx, props, bc)
    }

    fn paint(&mut self, ctx: &mut PaintCtx<'_>, props: &PropertiesRef<'_>, scene: &mut Scene) {
        self.handle_paint(ctx, props, scene)
    }

    fn get_cursor(&self, _ctx: &QueryCtx<'_>, _pos: Point) -> CursorIcon {
        CursorIcon::Text
    }

    fn accessibility_role(&self) -> Role {
        if EDITABLE {
            match self.insert_newline {
                InsertNewline::OnShiftEnter | InsertNewline::OnEnter => Role::MultilineTextInput,
                _ => Role::TextInput,
            }
        } else {
            Role::Document
        }
    }

    fn accessibility(
        &mut self,
        ctx: &mut AccessCtx<'_>,
        props: &PropertiesRef<'_>,
        node: &mut Node,
    ) {
        self.handle_accessibility(ctx, props, node)
    }

    fn children_ids(&self) -> ChildrenIds {
        ChildrenIds::new()
    }

    fn get_debug_text(&self) -> Option<String> {
        Some(self.editor.text().chars().take(100).collect())
    }
}
