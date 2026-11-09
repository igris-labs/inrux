//! aspect_ratio component construction.
use crate::Node;
use inrux_style::Style;
impl Node {
    /// Square container by default; an automatic dimension follows its preferred ratio.
    pub fn aspect_ratio(children: Vec<Self>) -> Self {
        let mut node = Self::box_container(children);
        node.defaults = Style::new().aspect_ratio(1.0);
        node.component_type("AspectRatio")
    }
}
