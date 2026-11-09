//! Common surface defaults, before translating colors and strokes to a graphics engine.
use inrux_style::{Color, Style};
/// Engine-independent background and inside-border paint policy.
pub struct Surface {
    /// Uniform corner radius in logical pixels.
    pub radius: f32,
    /// Optional background fill.
    pub background: Option<Color>,
    /// Optional inside stroke: thickness and color.
    pub border: Option<(f32, Color)>,
}
impl Surface {
    /// Resolve defaults once for both immediate and retained native surfaces.
    pub fn resolve(style: &Style) -> Self {
        Self {
            radius: style.rounded.unwrap_or(0.0),
            background: style.background,
            border: (style.border_color.is_some() || style.border_width.is_some()).then(|| {
                (
                    style.border_width.unwrap_or(1.0),
                    style
                        .border_color
                        .or(style.color)
                        .unwrap_or(Color(27, 30, 33)),
                )
            }),
        }
    }
}
