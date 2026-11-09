//! Decorative line built from an existing physical container.
use crate::Node;
use inrux_style::{Color, Size, Style};

impl Node {
    /// Draw a decorative divider; vertical lines default to 24px height, not parent stretch.
    pub fn separator(vertical: bool) -> Self {
        let mut node = Self::box_container(vec![]);
        node.defaults = Style::new()
            .width(if vertical { Size::Px(1.0) } else { Size::Fill })
            .height(Size::Px(if vertical { 24.0 } else { 1.0 }))
            .background(Color(210, 221, 215));
        node.component_type("Separator")
    }
}
