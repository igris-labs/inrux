//! Text viewport axis policy; shaping, editing and input coordinates remain native.
/// Retains manual panning until editing, caret navigation, focus or geometry requires reveal.
#[derive(Clone, Debug, Default)]
pub struct TextViewport {
    offset: f32,
    width: f32,
    extent: f32,
    caret: Option<f32>,
    focused: bool,
}
impl TextViewport {
    /// Clamp after resize/content changes and reveal a newly moved focused caret.
    pub fn update(&mut self, width: f32, extent: f32, caret: f32, focused: bool) {
        let width = width.max(0.0);
        let extent = extent.max(0.0);
        let reveal = focused
            && (!self.focused
                || self.caret != Some(caret)
                || self.width != width
                || self.extent != extent);
        self.width = width;
        self.extent = extent;
        self.caret = Some(caret);
        self.focused = focused;
        self.pan(0.0);
        if reveal {
            let right = (width - 2.0).max(0.0);
            if caret < self.offset {
                self.offset = caret.max(0.0);
            } else if caret > self.offset + right {
                self.offset = (caret - right).max(0.0);
            }
            self.pan(0.0);
        }
    }
    /// Reveal a complete caret rectangle along an axis, including upward navigation.
    pub fn update_range(&mut self, size: f32, extent: f32, start: f32, end: f32, focused: bool) {
        let reveal = focused
            && (!self.focused
                || self.caret != Some(start)
                || self.width != size
                || self.extent != extent);
        self.update(size, extent, start, focused);
        if reveal && end > self.offset + self.width {
            self.pan(end - self.offset - self.width);
        }
    }
    /// Reveal after an explicit navigation command, even when its caret index stays unchanged.
    pub fn request_reveal(&mut self) {
        self.caret = None;
    }
    /// Move without changing the caret; ordinary repaints preserve this position.
    pub fn pan(&mut self, delta: f32) -> bool {
        if !delta.is_finite() {
            return false;
        }
        let old = self.offset;
        self.offset = (self.offset + delta).clamp(0.0, (self.extent + 2.0 - self.width).max(0.0));
        old != self.offset
    }
    /// Available logical width of the clipped editing surface.
    pub fn width(&self) -> f32 {
        self.width
    }
    /// Logical text translation, also used for pointer and IME coordinates.
    pub fn offset(&self) -> f32 {
        self.offset
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    /// Manual pan survives repaint; navigation, shrinking content and resize clamp/reveal correctly.
    #[test]
    fn pan_reveal_and_resize_preserve_a_bounded_text_viewport() {
        let mut view = TextViewport::default();
        view.update(100.0, 500.0, 0.0, true);
        assert!(view.pan(80.0));
        view.update(100.0, 500.0, 0.0, true);
        assert_eq!(view.offset(), 80.0);
        view.request_reveal();
        view.update(100.0, 500.0, 0.0, true);
        assert_eq!(view.offset(), 0.0);
        view.update(100.0, 500.0, 480.0, true);
        assert_eq!(view.offset(), 382.0);
        view.update(200.0, 500.0, 480.0, true);
        assert_eq!(view.offset(), 302.0);
        view.update(200.0, 20.0, 20.0, true);
        assert_eq!(view.offset(), 0.0);
        assert!(!view.pan(f32::NAN));
    }
}
