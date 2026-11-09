//! Shared text presentation without interactive or live-status semantics.
use crate::Node;
use inrux_style::{Color, Style};

impl Node {
    /// Display a compact label; parent flex alignment still controls stretching.
    pub fn badge(text: String) -> Self {
        let mut node = Self::text(text);
        node.defaults = Style::new()
            .padding(4.0)
            .font_size(12.0)
            .rounded(4.0)
            .background(Color(228, 235, 231))
            .color(Color(32, 40, 37));
        node.component_type("Badge")
    }
}
