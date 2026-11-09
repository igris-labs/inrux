//! Shared cluster geometry for native mouse, keyboard, selection and IME integration.
use super::*;
#[derive(Clone)]
pub(super) struct TextLayout {
    pub shaped: std::sync::Arc<inrux_text::Shaped>,
    pub line_height: Pixels,
    pub ink: [u8; 4],
}
impl TextLayout {
    /// Position a UTF-8 caret using the same shape that is painted.
    pub fn position(&self, index: usize) -> Point<Pixels> {
        let caret = self.shaped.caret(index);
        point(px(caret[0]), px(caret[1]))
    }
    /// Hit-test the shared visual clusters to a UTF-8 boundary.
    pub fn index(&self, position: Point<Pixels>) -> usize {
        self.shaped.hit(position.x.into(), position.y.into())
    }
    /// Natural logical content extent, independent of display scale.
    pub fn size(&self) -> Size<Pixels> {
        size(px(self.shaped.width()), px(self.shaped.height()))
    }
    /// Visual rectangles preserve bidirectional runs and wrapped selections.
    pub fn selection(&self, range: Range<usize>, _: Pixels) -> Vec<Bounds<Pixels>> {
        self.shaped
            .selection(range)
            .into_iter()
            .map(|r| Bounds::new(point(px(r[0]), px(r[1])), size(px(r[2]), px(r[3]))))
            .collect()
    }
}
