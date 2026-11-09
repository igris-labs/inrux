//! Optional reference provider; no Taffy handles appear in the service contract.
use super::*;
use crate::services::{ProviderKind, text::TextMeasurer};
/// Provenance for the optional reference fallback.
pub const TAFFY_PROVIDER: ProviderInfo = ProviderInfo {
    name: "taffy",
    kind: ProviderKind::Fallback,
};
/// Fallback with a provider-owned fallible text measurer, borrowing native context only for its lifetime.
pub struct TaffyLayout<M> {
    text: M,
}
impl<M: TextMeasurer> TaffyLayout<M> {
    /// Bind the native measurer without forcing two mutable borrows of an engine context.
    pub fn new(text: M) -> Self {
        Self { text }
    }
}
impl<M: TextMeasurer> LayoutProvider for TaffyLayout<M> {
    /// Identify the library fallback without implying native layout or performance results.
    fn provider(&self) -> ProviderInfo {
        TAFFY_PROVIDER
    }
    /// Execute one pass and propagate measurement failure before returning geometry.
    fn layout(&mut self, request: LayoutRequest<'_>) -> Result<LayoutBox, AdapterError> {
        crate::layout::fallback::compute(
            request.root,
            request.width,
            request.height,
            &mut self.text,
        )
    }
}
