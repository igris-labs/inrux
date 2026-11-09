//! Backend-independent native host contract and application coordinator integration.
#![deny(missing_docs)]
mod capabilities;
pub mod control_parts;
pub mod controls;
pub mod editing;
mod error;
mod focus;
mod part_validation;
pub use focus::FocusVisibility;
mod font;
pub use font::FontAsset;
pub mod layout;
#[cfg(feature = "taffy-layout")]
mod layout_policy;
pub mod services;
mod session;
mod viewport;
pub use viewport::ViewportState;
pub mod surface;
mod window;
pub use capabilities::{Capabilities, Primitive};
pub use error::AdapterError;
use inrux_components::Component;
use inrux_core::Scope;
use inrux_style::Rule;
pub use session::Session;
pub use window::WindowOptions;

/// A native host. Backends own their event loop; the framework owns application state.
///
/// Select one host per window/application launch. This contract does not migrate live windows.
/// Implementations may use UI-thread-only types internally; no backend types cross this API.
pub trait Adapter: Sized {
    /// Report the supported primitive and interaction surface before mounting.
    fn capabilities(&self) -> Capabilities;
    /// Report operational framework service contracts; omitted services are unavailable.
    /// Older/custom hosting-only adapters may leave this empty without claiming native service coverage.
    fn services(&self) -> services::ServiceSupport {
        services::ServiceSupport::default()
    }
    /// Check a component family's service prerequisites before constructing its native resources.
    fn require_services(&self, required: &[services::Service]) -> Result<(), AdapterError> {
        self.services().require(self.capabilities().name, required)
    }
    /// Open a native host and run until it closes, keeping application code on its coordinator.
    fn run<S: Component>(
        self,
        window: WindowOptions,
        initial: S,
        scope: Scope,
        styles: Vec<Rule>,
    ) -> Result<(), AdapterError>;
}

#[cfg(test)]
mod tests;
