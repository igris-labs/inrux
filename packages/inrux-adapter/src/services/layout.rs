//! Replaceable layout execution; shared behavior does not require one shared algorithm.
use super::ProviderInfo;
use crate::AdapterError;
pub use crate::layout::LayoutBox;
use inrux_components::Node;
/// Immutable inputs in logical pixels; absence of height means intrinsic content height.
#[derive(Clone, Copy)]
pub struct LayoutRequest<'a> {
    /// Styled/inherited description; output children preserve its order and identity scopes.
    pub root: &'a Node,
    /// Finite, nonnegative available width.
    pub width: f32,
    /// Optional finite, nonnegative available height.
    pub height: Option<f32>,
}
/// Provider-owned execution, including its measurement context when native services share a mutable window.
/// Implementations may retain caches/resources privately and do not require Send/Sync.
pub trait LayoutProvider {
    /// Report the selected implementation without exposing its native types.
    fn provider(&self) -> ProviderInfo;
    /// Compute geometry once for this request, using fallible intrinsic text services as needed.
    /// Implementations preserve authored sizing semantics and description child order.
    fn layout(&mut self, request: LayoutRequest<'_>) -> Result<LayoutBox, AdapterError>;
}
/// Framework validation around a statically selected layout implementation.
pub struct LayoutService<P> {
    provider: P,
}
impl<P: LayoutProvider> LayoutService<P> {
    /// Keep provider-owned state across requests when the private binding retains this service.
    pub fn new(provider: P) -> Self {
        Self { provider }
    }
    /// Inspect the bound implementation; this does not select an engine by name.
    pub fn provider(&self) -> ProviderInfo {
        self.provider.provider()
    }
    /// Validate inputs, propagate text/provider failures and reject malformed geometry before paint.
    pub fn compute(&mut self, request: LayoutRequest<'_>) -> Result<LayoutBox, AdapterError> {
        crate::layout::validate(request.root)?;
        if !request.width.is_finite()
            || request.width < 0.0
            || request
                .height
                .is_some_and(|value| !value.is_finite() || value < 0.0)
        {
            return Err(AdapterError::Application(
                "Available layout dimensions must be finite and nonnegative".into(),
            ));
        }
        let geometry = self.provider.layout(request)?;
        validate_geometry(request.root, &geometry, self.provider.provider().name)?;
        Ok(geometry)
    }
}
/// Permit overflow/negative finite offsets; reject invalid bounds and mismatched tree structure.
fn validate_geometry(
    node: &Node,
    geometry: &LayoutBox,
    provider: &str,
) -> Result<(), AdapterError> {
    if !geometry.content_width.is_finite()
        || !geometry.content_height.is_finite()
        || geometry.content_width < 0.0
        || geometry.content_height < 0.0
        || !geometry.x.is_finite()
        || !geometry.y.is_finite()
        || !geometry.width.is_finite()
        || !geometry.height.is_finite()
        || geometry.width < 0.0
        || geometry.height < 0.0
        || node.children.len() != geometry.children.len()
    {
        return Err(AdapterError::Application(format!(
            "Layout provider `{provider}` returned invalid bounds or a mismatched child tree"
        )));
    }
    for (child, bounds) in node.children.iter().zip(&geometry.children) {
        validate_geometry(child, bounds, provider)?;
    }
    Ok(())
}

#[cfg(feature = "taffy-layout")]
mod taffy;
#[cfg(feature = "taffy-layout")]
pub use taffy::{TAFFY_PROVIDER, TaffyLayout};
