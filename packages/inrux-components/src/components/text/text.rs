//! Text component construction.
use crate::{Node, NodeKind};

impl Node {
    /// Create wrapping text.
    pub fn text(text: String) -> Self {
        Self::new(NodeKind::Text(text), vec![])
    }
}
