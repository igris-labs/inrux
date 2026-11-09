//! Headless UI descriptions and component contracts for Inrux.
#![deny(missing_docs)]
mod component;
mod components;
mod node;
mod parts;
mod range;
pub use range::{RangeKind, RangeValue, ToggleKind};
mod text_edit;
pub use text_edit::normalize_text;
mod scroll;
pub use component::{Component, Events};
pub use node::{Node, NodeKind};
pub use scroll::{ScrollMetrics, ScrollOffset, ScrollRequest};

mod model;
pub use model::{InputEvent, ModelEdit};

mod stylesheet;
pub use stylesheet::Stylesheet;

mod host;
pub use host::{ComponentHost, component_depth, component_key, component_scope, render_child};

mod handler;
pub use handler::{Handler, NoPayload, WithPayload};
mod binding;
pub use binding::ModelValue;
