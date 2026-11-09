//! Inrux: Igris Native Rust UI eXperience.
#![deny(missing_docs)]
pub use inrux_adapter::services;
pub use inrux_adapter::{
    Adapter, AdapterError, Capabilities, FontAsset, Primitive, ViewportState, WindowOptions,
};
pub use inrux_components::{
    Component, ComponentHost, Events, Handler, InputEvent, ModelEdit, ModelValue, NoPayload, Node,
    NodeKind, RangeKind, RangeValue, ScrollMetrics, ScrollOffset, ScrollRequest, Stylesheet,
    ToggleKind, WithPayload, component_depth, component_key, component_scope, render_child,
};
pub use inrux_core::reactive::{Computed, ReactiveError, Signal};
pub use inrux_core::tasks::{Cancellation, TaskError, TaskHandle, TaskQueueError};
pub use inrux_core::{
    Context, DispatchError, Emit, InstanceHandle, Instances, Scope, Sender, Worker,
};
/// Optional parsed Markdown documents, independent of native engines.
#[cfg(feature = "markdown")]
pub use inrux_markdown as markdown;
mod mount;
pub use mount::{mount, mount_with, mount_with_styles};
/// Engine implementations are selected only at application startup.
pub mod backends {
    #[cfg(feature = "egui")]
    pub use inrux_egui::Egui;
    #[cfg(feature = "gpui")]
    pub use inrux_gpui::Gpui;
    #[cfg(feature = "masonry")]
    pub use inrux_masonry::Masonry;
}
pub use inrux_style::{
    Align, Color, CompiledStyles, Display, FlexDirection, Justify, MAX_GRID_COLUMNS, Media,
    Overflow, Part, Position, Rgba, Rule, Selector, Size, State as StyleState, Style, StyleError,
    Viewport, WhiteSpace, parse_css,
};
/// Common component-authoring imports.
pub mod prelude {
    pub use crate::{
        Align, Cancellation, Color, Component, Computed, Context, Display, Emit, Events,
        FlexDirection, InstanceHandle, Instances, Justify, Node, Overflow, Part, Position, Scope,
        ScrollMetrics, ScrollOffset, ScrollRequest, Signal, Size, Style, Stylesheet, TaskHandle,
        WhiteSpace,
    };
}
