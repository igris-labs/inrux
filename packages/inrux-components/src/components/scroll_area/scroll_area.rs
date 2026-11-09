//! Shared viewport composition; engine bindings implement the overflow contract.
use crate::Node;
use inrux_style::{Overflow, Size, Style};
impl Node {
    /// Full-width vertical scrolling with a 240px default viewport and ordered retained children.
    pub fn scroll_area(children: Vec<Self>) -> Self {
        let mut node = Self::box_container(children);
        node.defaults = Style::new()
            .height(Size::Px(240.0))
            .overflow_x(Overflow::Hidden)
            .overflow_y(Overflow::Auto);
        node.component_type("ScrollArea")
    }
}
