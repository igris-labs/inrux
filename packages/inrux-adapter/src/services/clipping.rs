//! Logical rectangular clipping; native bindings enforce the same bounds for paint and hit testing.
/// A finite logical rectangle; negative positions are valid, sizes are nonnegative.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ClipRect {
    /// Left edge.
    pub x: f32,
    /// Top edge.
    pub y: f32,
    /// Horizontal extent.
    pub width: f32,
    /// Vertical extent.
    pub height: f32,
}
impl ClipRect {
    /// Intersect ancestor/child viewports without producing negative extents.
    pub fn intersect(self, other: Self) -> Self {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        Self {
            x,
            y,
            width: ((self.x + self.width).min(other.x + other.width) - x).max(0.0),
            height: ((self.y + self.height).min(other.y + other.height) - y).max(0.0),
        }
    }
    /// Whether a logical pointer is inside the half-open rectangle.
    pub fn contains(self, x: f32, y: f32) -> bool {
        x >= self.x && y >= self.y && x < self.x + self.width && y < self.y + self.height
    }
}
