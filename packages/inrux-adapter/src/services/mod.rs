//! Engine-neutral service contracts; providers are selected by each private native binding.
pub mod clipping;
mod indicators;
pub mod layout;
pub mod scroll;
mod support;
pub mod text;
pub use support::{ProviderInfo, ProviderKind, Service, ServiceDescriptor, ServiceSupport};
