//! Column component construction.
use crate::{Node, NodeKind};

impl Node {
    /// Arrange children vertically.
    pub fn column(children: Vec<Self>) -> Self {
        Self::new(NodeKind::Column, children)
    }
}
