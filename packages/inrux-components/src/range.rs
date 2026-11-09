//! Backend-independent determinate range semantics and validation.
/// Range purpose determines native accessibility semantics, not its paint style.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RangeKind {
    /// Completion of work from zero to a positive maximum.
    Progress,
    /// A measurement within an explicit bounded interval.
    Meter,
}
/// Controlled, read-only range data shared by every native binding.
#[derive(Clone, Debug, PartialEq)]
pub struct RangeValue {
    /// Completion or measurement semantics.
    pub kind: RangeKind,
    /// Accessible name supplied by the developer.
    pub label: String,
    /// Current finite value.
    pub value: f64,
    /// Inclusive minimum (zero for Progress).
    pub min: f64,
    /// Inclusive maximum, strictly greater than min.
    pub max: f64,
}
impl RangeValue {
    /// Validate before native translation and return the bounded paint fraction.
    pub fn fraction(&self) -> Result<f32, &'static str> {
        if self.label.trim().is_empty() {
            return Err("Progress and Meter require a nonempty accessible label");
        }
        if !self.min.is_finite()
            || !self.max.is_finite()
            || !self.value.is_finite()
            || !(self.max - self.min).is_finite()
            || self.max <= self.min
        {
            return Err("Range bounds/value must be finite, with max greater than min");
        }
        if self.kind == RangeKind::Progress && self.min != 0.0 {
            return Err("Progress minimum must be zero");
        }
        if self.value < self.min || self.value > self.max {
            return Err("Range value must be between min and max");
        }
        Ok(((self.value - self.min) / (self.max - self.min)) as f32)
    }
}
/// Distinct native boolean-control presentation and semantic role.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToggleKind {
    /// A checked/unchecked box.
    #[default]
    Checkbox,
    /// An on/off switch.
    Switch,
}

impl crate::Node {
    /// Construct common range defaults once without cloning labels through another component.
    pub(crate) fn range_indicator(range: RangeValue) -> Self {
        use inrux_style::{Color, Size, Style};
        let name = match range.kind {
            RangeKind::Progress => "Progress",
            RangeKind::Meter => "Meter",
        };
        Self::new(crate::NodeKind::Range(range), vec![])
            .component_type(name)
            .with_defaults(
                Style::new()
                    .width(Size::Fill)
                    .height(Size::Px(8.0))
                    .rounded(4.0)
                    .background(Color(228, 235, 231)),
            )
    }
}
