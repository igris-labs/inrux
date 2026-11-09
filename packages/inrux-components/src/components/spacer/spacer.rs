//! Spacer component construction.
use crate::Node;
use inrux_style::{Size, Style};

impl Node {
    /// Consume remaining Row/Column space without intrinsic content or a focus target.
    pub fn spacer() -> Self {
        let mut node = Self::box_container(vec![]);
        node.defaults = Style::new()
            .width(Size::Px(0.0))
            .height(Size::Px(0.0))
            .flex_grow(1.0);
        node.component_type("Spacer")
    }
}
