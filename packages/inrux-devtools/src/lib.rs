//! Optional native render-tree inspection; no server or browser runtime is required.
#![deny(missing_docs)]
mod tree;
pub use tree::{inspect, node_count};
