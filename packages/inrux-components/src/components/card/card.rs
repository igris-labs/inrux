//! Shared surface composition; native bindings see an ordinary container.
use crate::Node;
use inrux_style::{Color, Style};

impl Node {
    /// Group ordered content in a padded surface; child controls keep their own behavior.
    pub fn card(children: Vec<Self>) -> Self {
        let mut node = Self::box_container(children);
        node.defaults = Style::new()
            .padding(16.0)
            .gap(12.0)
            .background(Color(255, 255, 255))
            .rounded(8.0)
            .border_color(Color(210, 221, 215))
            .border_width(1.0);
        node.component_type("Card")
    }
}
