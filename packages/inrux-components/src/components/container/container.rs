//! Container component construction.
use crate::Node;
use inrux_style::{Size, Style};

impl Node {
    /// Center full-width content horizontally, capped at 1200 logical pixels.
    pub fn container(children: Vec<Self>) -> Self {
        let mut node = Self::box_container(children);
        node.defaults = Style::new()
            .max_width(Size::Px(1200.0))
            .margin_inline_auto(true);
        node.component_type("Container")
    }
}
