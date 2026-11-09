//! Empty component construction.
use crate::{Node, NodeKind};

impl Node {
    /// Create a node excluded from layout and painting.
    pub fn empty() -> Self {
        Self::new(NodeKind::Empty, vec![])
    }
}
