//! Portable viewport queries and revision-bearing UI commands.
use inrux_core::Emit;
/// Positive logical offset from the content's top-left corner.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ScrollOffset {
    /// Horizontal offset in logical pixels.
    pub x: f32,
    /// Vertical offset in logical pixels.
    pub y: f32,
}
/// A UI-owned viewport observation, not an application-controlled model.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ScrollMetrics {
    /// Current clamped content offset.
    pub offset: ScrollOffset,
    /// Viewport width in logical pixels.
    pub width: f32,
    /// Viewport height in logical pixels.
    pub height: f32,
    /// Content extent including overflow.
    pub content_width: f32,
    /// Content extent including overflow.
    pub content_height: f32,
}
/// An explicit scroll command; change revision to issue another command.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollRequest {
    /// Command identity; repeated revisions do not reset user scrolling.
    pub revision: u64,
    /// Requested logical position; bounds are clamped by the viewport.
    pub offset: ScrollOffset,
}
impl crate::Node {
    /// Observe changed UI scroll metrics through the normal bounded event channel.
    pub fn on_scroll(mut self, event: Emit<ScrollMetrics>) -> Self {
        self.on_scroll = Some(event);
        self
    }
    /// Issue a one-shot UI command without taking ownership of subsequent user scrolling.
    pub fn scroll_to(mut self, request: ScrollRequest) -> Self {
        self.scroll_request = Some(request);
        self
    }
}
