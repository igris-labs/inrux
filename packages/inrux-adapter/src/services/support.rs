//! Explicit readiness and provenance, separate from individual primitive coverage.
use crate::AdapterError;
/// Independent framework service boundaries required by component families.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Service {
    /// Authored layout semantics and framework geometry.
    Layout,
    /// Intrinsic text measurement through the selected shared or native provider.
    TextMeasurement,
    /// Rectangular native clipping shared by paint and pointer hit testing.
    Clipping,
    /// UI-owned scrolling, revision-bearing commands and viewport queries.
    Scrolling,
    /// Generation-safe focus commands/queries, beyond primitive native focus.
    FocusCommands,
    /// Overlay placement/lifetime, dismissal and focus ownership.
    Overlays,
    /// Framework-managed asset resources.
    Assets,
    /// Framework semantic-tree integration beyond individual native roles.
    Accessibility,
}
/// Implementation provenance, not a performance ranking.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProviderKind {
    /// Delegates execution to the selected engine's native service.
    Native,
    /// Supplies missing functionality with a library or focused implementation.
    Fallback,
}
/// Human-readable implementation metadata; no engine handles cross the boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProviderInfo {
    /// Provider name used only for inspection/diagnostics, never string-based dispatch.
    pub name: &'static str,
    /// Whether this service uses native delegation or a fallback.
    pub kind: ProviderKind,
}
/// A service this adapter actually implements through the common contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ServiceDescriptor {
    /// Available service boundary.
    pub service: Service,
    /// Selected provider for that boundary.
    pub provider: ProviderInfo,
}
/// Read-only startup readiness. An absent entry is unsupported, not silently emulated.
#[derive(Clone, Copy, Debug, Default)]
pub struct ServiceSupport(pub &'static [ServiceDescriptor]);
impl ServiceSupport {
    /// Inspect a selected provider without constructing native objects.
    pub fn provider(&self, service: Service) -> Option<ProviderInfo> {
        self.0
            .iter()
            .find(|entry| entry.service == service)
            .map(|entry| entry.provider)
    }
    /// Reject missing prerequisites before attempting to construct a component family.
    pub fn require(&self, backend: &'static str, required: &[Service]) -> Result<(), AdapterError> {
        for service in required {
            if self.provider(*service).is_none() {
                return Err(AdapterError::Unsupported {
                    backend,
                    feature: format!("{service:?} service contract"),
                });
            }
        }
        Ok(())
    }
}
