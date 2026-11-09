//! Errors stay expressed in framework terms instead of leaking a backend's public types.
use std::fmt;
/// Failure to start a host, support a requested feature, or consume application output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdapterError {
    /// Application startup must select exactly one linked engine or an explicit adapter.
    Configuration(String),
    /// A selected backend is not linked or implemented in this build.
    Unavailable(&'static str),
    /// A backend cannot represent a requested primitive or styling feature.
    Unsupported {
        /// Name of the backend rejecting the request.
        backend: &'static str,
        /// Unsupported operation/property/control with actionable context.
        feature: String,
    },
    /// A native platform or renderer failed.
    Native {
        /// Backend that encountered the platform failure.
        backend: &'static str,
        /// Native error rendered at the boundary.
        message: String,
    },
    /// Application startup, tree validation or the coordinator failed.
    Application(String),
}
impl fmt::Display for AdapterError {
    /// Render a concise error suitable for a CLI or native error surface.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Configuration(message) => write!(f, "Adapter configuration: {message}"),
            Self::Unavailable(name) => write!(f, "Backend `{name}` is not available in this build"),
            Self::Unsupported { backend, feature } => {
                write!(f, "Backend `{backend}` does not support {feature}")
            }
            Self::Native { backend, message } => write!(f, "Backend `{backend}` failed: {message}"),
            Self::Application(message) => write!(f, "Application failed: {message}"),
        }
    }
}
impl std::error::Error for AdapterError {}
