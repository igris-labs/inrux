//! flex component construction.
use crate::Node;
use inrux_style::{FlexDirection, Style};
impl Node {
    /// Flexible container with a horizontal main axis by default.
    pub fn flex(children: Vec<Self>) -> Self {
        let mut node = Self::box_container(children);
        node.defaults = Style::new().flex_direction(FlexDirection::Row);
        node.component_type("Flex")
    }
}
