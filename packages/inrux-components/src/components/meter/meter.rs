//! Bounded measurements remain semantically distinct from work completion.
use crate::{Node, RangeKind, RangeValue};
impl Node {
    /// Describe a finite measurement in min..max; this is read-only, not a slider.
    pub fn meter(label: String, value: f64, min: f64, max: f64) -> Self {
        Self::range_indicator(RangeValue {
            kind: RangeKind::Meter,
            label,
            value,
            min,
            max,
        })
    }
}
