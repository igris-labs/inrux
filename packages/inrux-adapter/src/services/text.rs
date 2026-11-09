//! Fallible text measurement with framework units and provider-owned native context.
use super::ProviderInfo;
use crate::AdapterError;
pub use crate::layout::TextMeasure;
/// Natural text bounds in logical pixels; unbreakable content may exceed its width constraint.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextSize {
    /// Natural text width, excluding framework control padding.
    pub width: f32,
    /// Natural text height, using the authored line-height policy.
    pub height: f32,
}
/// A native or fallback measurer; no Send/Sync bound moves UI-affine resources across threads.
pub trait TextMeasurer {
    /// Return natural bounds or the original provider failure.
    fn measure(&mut self, request: TextMeasure<'_>) -> Result<TextSize, AdapterError>;
}
/// Statically dispatched callback adapter that keeps native shaping context private.
pub struct TextService<F> {
    provider: ProviderInfo,
    measure: F,
}
impl<F> TextService<F> {
    /// Bind a provider once in the owning native context; no automatic runtime selection.
    pub fn new(provider: ProviderInfo, measure: F) -> Self {
        Self { provider, measure }
    }
    /// Report the implementation selected by the private binding.
    pub fn provider(&self) -> ProviderInfo {
        self.provider
    }
}
impl<F> TextMeasurer for TextService<F>
where
    F: FnMut(TextMeasure<'_>) -> Result<TextSize, AdapterError>,
{
    /// Check the native result before geometry or paint can consume invalid dimensions.
    fn measure(&mut self, request: TextMeasure<'_>) -> Result<TextSize, AdapterError> {
        validate_request(&request)?;
        let size = (self.measure)(request)?;
        validate_size(size, self.provider.name)?;
        Ok(size)
    }
}
/// Reject malformed native requests even when a provider is called outside the layout service.
pub(crate) fn validate_request(request: &TextMeasure<'_>) -> Result<(), AdapterError> {
    if !request.font_size.is_finite()
        || request.font_size < 0.0
        || request.max_width.is_nan()
        || request.max_width < 0.0
    {
        return Err(AdapterError::Application("Text measurement requires a finite nonnegative font size and nonnegative width (infinity permits unwrapped text)".into()));
    }
    Ok(())
}
/// Validate results from custom trait implementations as well as callback services.
pub(crate) fn validate_size(size: TextSize, provider: &str) -> Result<(), AdapterError> {
    if !size.width.is_finite() || !size.height.is_finite() || size.width < 0.0 || size.height < 0.0
    {
        return Err(AdapterError::Application(format!(
            "Text provider `{provider}` returned nonfinite or negative bounds"
        )));
    }
    Ok(())
}

impl<M: TextMeasurer + ?Sized> TextMeasurer for &mut M {
    /// Borrow a retained native measurer for one layout request without boxing or cloning its context.
    fn measure(&mut self, request: TextMeasure<'_>) -> Result<TextSize, AdapterError> {
        (**self).measure(request)
    }
}
