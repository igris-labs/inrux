//! Determinate work completion, shared across native bindings.
use crate::{Node, RangeKind, RangeValue};
impl Node {
    /// Describe completion in 0..max. Invalid values/names are diagnosed before native translation.
    pub fn progress(label: String, value: f64, max: f64) -> Self {
        Self::range_indicator(RangeValue {
            kind: RangeKind::Progress,
            label,
            value,
            min: 0.0,
            max,
        })
    }
}
