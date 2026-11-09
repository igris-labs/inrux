//! grid component construction.
use crate::Node;
use inrux_style::{Display, Style};
impl Node {
    /// Uniform row-major grid with two equal-width columns by default.
    pub fn grid(children: Vec<Self>) -> Self {
        let mut node = Self::box_container(children);
        node.defaults = Style::new().display(Display::Grid).grid_columns(2);
        node.component_type("Grid")
    }
}
