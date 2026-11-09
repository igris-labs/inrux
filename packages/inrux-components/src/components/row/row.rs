//! Row component construction.
use crate::{Node, NodeKind};

impl Node {
    /// Arrange children horizontally.
    pub fn row(children: Vec<Self>) -> Self {
        Self::new(NodeKind::Row, children)
    }
}
