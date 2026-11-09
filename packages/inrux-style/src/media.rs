//! Compiled viewport conditions are evaluated only when resolving a description.
use crate::StyleError;
/// Logical viewport dimensions, independent of native device scale.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Viewport {
    /// Available logical width.
    pub width: f32,
    /// Available logical height.
    pub height: f32,
}
/// Inclusive min/max width/height media constraints, combined with `and`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Media {
    /// Minimum logical viewport width.
    pub min_width: Option<f32>,
    /// Maximum logical viewport width.
    pub max_width: Option<f32>,
    /// Minimum logical viewport height.
    pub min_height: Option<f32>,
    /// Maximum logical viewport height.
    pub max_height: Option<f32>,
}
impl Media {
    /// Compile a conjunction of pixel-valued viewport conditions.
    pub fn parse(input: &str) -> Result<Self, StyleError> {
        let mut result = Self::default();
        for part in input.split("and") {
            let part = part
                .trim()
                .strip_prefix('(')
                .and_then(|s| s.strip_suffix(')'))
                .ok_or_else(|| {
                    StyleError("media supports (min/max-width/height: Npx) joined by and".into())
                })?;
            let (name, value) = part
                .split_once(':')
                .ok_or_else(|| StyleError("expected media feature: value".into()))?;
            let value: f32 = value
                .trim()
                .strip_suffix("px")
                .ok_or_else(|| StyleError("media dimensions require px".into()))?
                .parse()
                .map_err(|_| StyleError("invalid media dimension".into()))?;
            if !value.is_finite() || value < 0.0 {
                return Err(StyleError(
                    "media dimensions must be finite and nonnegative".into(),
                ));
            }
            let next = match name.trim() {
                "min-width" => Self {
                    min_width: Some(value),
                    ..Self::default()
                },
                "max-width" => Self {
                    max_width: Some(value),
                    ..Self::default()
                },
                "min-height" => Self {
                    min_height: Some(value),
                    ..Self::default()
                },
                "max-height" => Self {
                    max_height: Some(value),
                    ..Self::default()
                },
                _ => return Err(StyleError(format!("unsupported media feature `{name}`"))),
            };
            result = result.intersect(next);
        }
        Ok(result)
    }
    /// Intersect nested media blocks without changing their source order.
    pub fn intersect(self, other: Self) -> Self {
        // Preserve the stricter bound when the same dimension occurs more than once.
        fn bound(a: Option<f32>, b: Option<f32>, minimum: bool) -> Option<f32> {
            match (a, b) {
                (Some(a), Some(b)) => Some(if minimum { a.max(b) } else { a.min(b) }),
                (a, b) => a.or(b),
            }
        }
        Self {
            min_width: bound(self.min_width, other.min_width, true),
            max_width: bound(self.max_width, other.max_width, false),
            min_height: bound(self.min_height, other.min_height, true),
            max_height: bound(self.max_height, other.max_height, false),
        }
    }
    /// Test inclusive boundaries in logical pixels.
    pub fn matches(self, viewport: Viewport) -> bool {
        self.min_width.is_none_or(|n| viewport.width >= n)
            && self.max_width.is_none_or(|n| viewport.width <= n)
            && self.min_height.is_none_or(|n| viewport.height >= n)
            && self.max_height.is_none_or(|n| viewport.height <= n)
    }
}
