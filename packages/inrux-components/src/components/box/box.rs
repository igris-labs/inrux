//! Box component construction.
use crate::Node;

impl Node {
    /// Generic physical container with vertical children and full available width.
    pub fn box_container(children: Vec<Self>) -> Self {
        Self::column(children).component_type("Box")
    }
}
