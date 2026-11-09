//! Multiline keyboard commands use the same native glyph geometry as mouse and IME selection.
use super::*;
impl TextInput {
    /// Resolve visual line edges while preserving single-line shortcut behavior.
    pub(super) fn line_edge(&self, end: bool) -> usize {
        if !self.multiline {
            return if end { self.content.len() } else { 0 };
        }
        if let Some(layout) = &self.last_layout {
            let position = layout.position(self.cursor_offset());
            return layout.index(point(if end { Pixels::MAX } else { px(0.0) }, position.y));
        }
        let cursor = self.cursor_offset();
        if end {
            cursor
                + self.content[cursor..]
                    .find('\n')
                    .unwrap_or(self.content.len() - cursor)
        } else {
            self.content[..cursor].rfind('\n').map_or(0, |i| i + 1)
        }
    }
    pub(super) fn document_start(
        &mut self,
        _: &DocumentStart,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.move_to(0, cx);
    }
    pub(super) fn document_end(&mut self, _: &DocumentEnd, _: &mut Window, cx: &mut Context<Self>) {
        self.move_to(self.content.len(), cx);
    }
    pub(super) fn select_document_start(
        &mut self,
        _: &SelectDocumentStart,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.select_to(0, cx);
    }
    pub(super) fn select_document_end(
        &mut self,
        _: &SelectDocumentEnd,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.select_to(self.content.len(), cx);
    }

    pub(super) fn newline(&mut self, _: &Newline, window: &mut Window, cx: &mut Context<Self>) {
        if self.multiline {
            self.replace_text_in_range(None, "\n", window, cx);
        }
    }
    fn vertical_move(&mut self, direction: f32, selecting: bool, cx: &mut Context<Self>) {
        if !self.multiline {
            return;
        }
        let Some(layout) = self.last_layout.as_ref() else {
            return;
        };
        let position = layout.position(self.cursor_offset());
        let index = layout.index(position + point(px(0.0), layout.line_height * direction));
        if selecting {
            self.select_to(index, cx);
        } else {
            self.move_to(index, cx);
        }
    }
    pub(super) fn up(&mut self, _: &Up, _: &mut Window, cx: &mut Context<Self>) {
        self.vertical_move(-1.0, false, cx);
    }
    pub(super) fn down(&mut self, _: &Down, _: &mut Window, cx: &mut Context<Self>) {
        self.vertical_move(1.0, false, cx);
    }
    pub(super) fn select_up(&mut self, _: &SelectUp, _: &mut Window, cx: &mut Context<Self>) {
        self.vertical_move(-1.0, true, cx);
    }
    pub(super) fn select_down(&mut self, _: &SelectDown, _: &mut Window, cx: &mut Context<Self>) {
        self.vertical_move(1.0, true, cx);
    }
}
