//! Shared logical sizes, colors, and interaction states.
/// Overflow behavior on one physical axis.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Overflow {
    /// Paint outside the box unless the other axis establishes a viewport.
    #[default]
    Visible,
    /// Clip without accepting scroll input.
    Hidden,
    /// Scroll on overflow, showing an indicator only when needed.
    Auto,
    /// Accept scrolling and keep the axis indicator visible.
    Scroll,
}
impl Overflow {
    /// Whether this axis admits user scrolling.
    pub fn scrollable(self) -> bool {
        matches!(self, Self::Auto | Self::Scroll)
    }
}
/// Logical-pixel, parent-relative, or intrinsic sizing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Size {
    /// A fixed number of logical pixels.
    Px(f32),
    /// A percentage of the available parent dimension, from 0 upward.
    Percent(f32),
    /// Use the full available parent dimension.
    Fill,
    /// Size to intrinsic content.
    Content,
}
/// Interaction state selector supported by the prototype.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum State {
    #[default]
    /// Always applied.
    Base,
    /// Applied while the pointer is over an enabled node.
    Hover,
    /// Applied while the widget owns keyboard focus.
    Focus,
    /// Applied when the component is disabled.
    Disabled,
    /// Applied when the owning boolean control is checked.
    Checked,
    /// Applied when the owning boolean control is unchecked.
    Unchecked,
}

/// Cross-axis alignment in a flex container.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Align {
    /// Align to the beginning of the cross axis.
    Start,
    /// Center on the cross axis.
    Center,
    /// Align to the end of the cross axis.
    End,
    /// Stretch children with automatic cross-axis dimensions.
    Stretch,
}
/// Main-axis distribution in a flex container.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Justify {
    /// Pack at the beginning.
    Start,
    /// Center together.
    Center,
    /// Pack at the end.
    End,
    /// Distribute space between children.
    SpaceBetween,
    /// Give each child equal surrounding space.
    SpaceAround,
    /// Distribute all outer and inner spaces equally.
    SpaceEvenly,
}

/// Layout algorithm selected by a physical container.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Display {
    /// Single-axis flexible layout.
    Flex,
    /// Row-major equal-column grid layout.
    Grid,
}
/// Main-axis direction without changing source, keyboard or reading order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlexDirection {
    /// Horizontal main axis.
    Row,
    /// Vertical main axis.
    Column,
}
/// Maximum explicit column count supported by the initial uniform grid contract.
pub const MAX_GRID_COLUMNS: u16 = 256;

/// Whether a box participates in normal flow or is positioned inside its parent.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Position {
    /// Remain in flow; insets offset the box relative to its normal position.
    #[default]
    Relative,
    /// Leave flow; insets anchor the box to the containing physical parent.
    Absolute,
}

/// Text wrapping policy; Pre preserves authored line breaks and never wraps long code lines.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WhiteSpace {
    /// Existing native paragraph wrapping behavior.
    #[default]
    Normal,
    /// Preformatted text with native measurement of unwrapped lines.
    Pre,
}
