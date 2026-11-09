//! Native window options expressed without platform-specific handle types.
/// Initial logical window dimensions. Each backend translates these at its boundary.
#[derive(Clone, Debug)]
pub struct WindowOptions {
    /// Window title.
    pub title: String,
    /// Initial client width in logical pixels.
    pub width: f64,
    /// Initial client height in logical pixels.
    pub height: f64,
    /// Minimum client width in logical pixels.
    pub min_width: f64,
    /// Minimum client height in logical pixels.
    pub min_height: f64,
    /// Application fonts registered before the first component frame.
    pub fonts: Vec<crate::FontAsset>,
}
impl WindowOptions {
    /// Use the framework's initial prototype dimensions with an owned title.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            width: 860.0,
            height: 820.0,
            min_width: 360.0,
            min_height: 480.0,
            fonts: vec![],
        }
    }
    /// Reject invalid sizes before passing them to platform APIs.
    pub fn validate(&self) -> Result<(), crate::AdapterError> {
        if self
            .fonts
            .iter()
            .any(|font| font.family.trim().is_empty() || font.data.is_empty())
        {
            return Err(crate::AdapterError::Configuration(
                "A font needs its intrinsic family name and nonempty OpenType/TrueType data".into(),
            ));
        }
        let sizes = [self.width, self.height, self.min_width, self.min_height];
        if sizes
            .iter()
            .any(|value| !value.is_finite() || *value <= 0.0)
            || self.width < self.min_width
            || self.height < self.min_height
        {
            return Err(crate::AdapterError::Application(
                "Window dimensions must be finite, positive, and no smaller than their minimums"
                    .into(),
            ));
        }
        Ok(())
    }
}
