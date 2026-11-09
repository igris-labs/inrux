//! Stable author-facing component part names, independent of native widget structure.
use crate::StyleError;
/// A public styling target inside a semantic component.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Part {
    /// Checkbox's box surface.
    Control,
    /// Switch's track surface.
    Track,
    /// Switch's moving thumb.
    Thumb,
    /// Checkbox mark or range fill.
    Indicator,
    /// Boolean control's text label.
    Label,
    /// Text editor's empty-value hint.
    Placeholder,
}
impl Part {
    /// Parse only stable public names; native implementation identifiers are never selectors.
    pub fn parse(name: &str) -> Result<Self, StyleError> {
        match name {
            "control" => Ok(Self::Control),
            "track" => Ok(Self::Track),
            "thumb" => Ok(Self::Thumb),
            "indicator" => Ok(Self::Indicator),
            "label" => Ok(Self::Label),
            "placeholder" => Ok(Self::Placeholder),
            _ => Err(StyleError(format!("unknown component part `{name}`"))),
        }
    }
}
