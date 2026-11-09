//! Heartbeat component construction.
use crate::{Node, NodeKind};

impl Node {
    /// Create the animated UI-thread diagnostic.
    pub fn heartbeat() -> Self {
        Self::new(NodeKind::Heartbeat, vec![])
    }
}
