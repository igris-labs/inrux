//! Framework geometry and validation, independent of the selected layout implementation.
use crate::AdapterError;
use inrux_components::Node;
#[cfg(feature = "taffy-layout")]
pub(crate) mod fallback;
/// Intrinsic text request in logical pixels, independent of a font engine.
pub struct TextMeasure<'a> {
    /// Text to shape or measure.
    pub text: &'a str,
    /// Resolved font size.
    pub font_size: f32,
    /// Registered family used for both intrinsic measurement and native painting.
    pub font_family: Option<&'a str>,
    /// Available text width; infinity permits an unwrapped line.
    pub max_width: f32,
}
/// Resolved geometry relative to the parent, preserving descriptor child order.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LayoutBox {
    /// Unclipped content extent reported by the selected provider.
    pub content_width: f32,
    /// Unclipped content extent reported by the selected provider.
    pub content_height: f32,
    /// Horizontal offset in logical pixels.
    pub x: f32,
    /// Vertical offset in logical pixels.
    pub y: f32,
    /// Width in logical pixels.
    pub width: f32,
    /// Height in logical pixels.
    pub height: f32,
    /// Child geometry in description order.
    pub children: Vec<Self>,
}
/// Compatibility entry point selecting the optional Taffy fallback and a tuple-returning text callback.
/// New bindings use services::layout::LayoutService with fallible text measurement instead.
#[cfg(feature = "taffy-layout")]
pub fn compute(
    node: &Node,
    width: f32,
    height: Option<f32>,
    mut measure: impl FnMut(TextMeasure<'_>) -> (f32, f32),
) -> Result<LayoutBox, AdapterError> {
    use crate::services::{
        ProviderInfo, ProviderKind,
        layout::{LayoutRequest, LayoutService, TaffyLayout},
        text::{TextService, TextSize},
    };
    let mut text = TextService::new(
        ProviderInfo {
            name: "legacy text callback",
            kind: ProviderKind::Fallback,
        },
        |request: TextMeasure<'_>| {
            let (width, height) = measure(request);
            Ok(TextSize { width, height })
        },
    );
    LayoutService::new(TaffyLayout::new(&mut text)).compute(LayoutRequest {
        root: node,
        width,
        height,
    })
}
/// Validate Rust-built styles as well as parsed CSS before entering either engine's layout API.
pub fn validate(node: &Node) -> Result<(), AdapterError> {
    node.validate_keys().map_err(AdapterError::Application)?;
    validate_styles(node)
}
/// Walk styles once without repeating sibling identity validation at every node.
fn validate_styles(node: &Node) -> Result<(), AdapterError> {
    let style = node.style();
    crate::part_validation::validate(node)?;
    if let inrux_components::NodeKind::Checkbox {
        kind: inrux_components::ToggleKind::Switch,
        label,
        ..
    } = &node.kind
        && label.trim().is_empty()
    {
        return Err(AdapterError::Application(
            "Switch requires a nonempty accessible label".into(),
        ));
    }
    if let inrux_components::NodeKind::Range(range) = &node.kind {
        range
            .fraction()
            .map_err(|message| AdapterError::Application(format!("{}: {message}", range.label)))?;
    }
    let viewport = style.overflow_axes()
        != (
            inrux_style::Overflow::Visible,
            inrux_style::Overflow::Visible,
        );
    if (viewport || node.on_scroll.is_some() || node.scroll_request.is_some())
        && !matches!(
            node.kind,
            inrux_components::NodeKind::Column | inrux_components::NodeKind::Row
        )
    {
        return Err(AdapterError::Application(
            "Overflow and scroll bindings require a physical container".into(),
        ));
    }
    if let Some(request) = node.scroll_request
        && (!request.offset.x.is_finite() || !request.offset.y.is_finite())
    {
        return Err(AdapterError::Application(
            "Scroll requests must have finite offsets".into(),
        ));
    }
    if (node.on_scroll.is_some() || node.scroll_request.is_some()) && !viewport {
        return Err(AdapterError::Application(
            "Scroll bindings require a non-visible overflow viewport".into(),
        ));
    }
    if let Some(columns) = style.grid_columns
        && !(1..=inrux_style::MAX_GRID_COLUMNS).contains(&columns)
    {
        return Err(AdapterError::Application(
            "Grid columns must be in 1..=256".into(),
        ));
    }
    if let Some(ratio) = style.aspect_ratio
        && (!ratio.is_finite() || ratio <= 0.0)
    {
        return Err(AdapterError::Application(
            "Aspect ratio must be finite and positive".into(),
        ));
    }
    if (style.display == Some(inrux_style::Display::Grid) || style.aspect_ratio.is_some())
        && !matches!(
            node.kind,
            inrux_components::NodeKind::Column | inrux_components::NodeKind::Row
        )
    {
        return Err(AdapterError::Application(
            "Grid display and aspect ratio require a physical container".into(),
        ));
    }
    for value in [
        style.gap,
        style.row_gap,
        style.column_gap,
        style.padding,
        style.font_size,
        style.rounded,
        style.border_width,
    ]
    .into_iter()
    .flatten()
    {
        if !value.is_finite() || value < 0.0 {
            return Err(AdapterError::Application(
                "Style lengths must be finite and nonnegative".into(),
            ));
        }
    }
    for value in [
        style.margin_top,
        style.margin_right,
        style.margin_bottom,
        style.margin_left,
    ]
    .into_iter()
    .flatten()
    {
        if !value.is_finite() {
            return Err(AdapterError::Application("Margins must be finite".into()));
        }
    }
    if let Some(value) = style.flex_grow
        && (!value.is_finite() || value < 0.0)
    {
        return Err(AdapterError::Application(
            "Style flex-grow must be finite and nonnegative".into(),
        ));
    }
    for value in [
        style.top,
        style.right,
        style.bottom,
        style.left,
        style.width,
        style.height,
        style.min_width,
        style.max_width,
        style.min_height,
        style.max_height,
    ]
    .into_iter()
    .flatten()
    {
        if let inrux_style::Size::Px(value) | inrux_style::Size::Percent(value) = value
            && (!value.is_finite() || value < 0.0)
        {
            return Err(AdapterError::Application(
                "Style dimensions must be finite and nonnegative".into(),
            ));
        }
    }
    for child in &node.children {
        validate_styles(child)?;
    }
    Ok(())
}
