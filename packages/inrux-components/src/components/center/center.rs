//! Center component construction.
use crate::Node;
use inrux_style::{Align, Justify, Style};

impl Node {
    /// Center children on both axes; provide height for vertical free space.
    pub fn center(children: Vec<Self>) -> Self {
        let mut node = Self::box_container(children);
        node.defaults = Style::new()
            .align_items(Align::Center)
            .justify_content(Justify::Center);
        node.component_type("Center")
    }
}
