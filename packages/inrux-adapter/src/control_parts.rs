//! Shared headless-control geometry and paint resolved from public parts.
use crate::controls;
use inrux_components::{Node, NodeKind, ToggleKind};
use inrux_style::{Color, Part, Size, Style};
/// Backend-neutral boolean-control visual data; changing it never replaces behavior or native identity.
#[derive(Clone, Debug, PartialEq)]
pub struct ToggleVisual {
    /// Track/control width.
    pub width: f32,
    /// Track/control height.
    pub height: f32,
    /// Track/control radius.
    pub radius: f32,
    /// Inside border width.
    pub border_width: f32,
    /// Track/control paint.
    pub fill: Color,
    /// Border paint.
    pub border: Color,
    /// Thumb or checkmark paint.
    pub mark: Color,
    /// Switch thumb width.
    pub thumb_width: f32,
    /// Switch thumb height.
    pub thumb_height: f32,
    /// Switch thumb radius.
    pub thumb_radius: f32,
    /// Horizontal label gap.
    pub gap: f32,
    /// Label typography/ink after inheritance and public-part overrides.
    pub label: Style,
}
/// Resolve one controlled boolean component; fallback values remain shared and minimal.
pub fn toggle(node: &Node, style: &Style, hovered: bool, focused: bool) -> ToggleVisual {
    let (kind, checked) = match node.kind {
        NodeKind::Checkbox { kind, checked, .. } => (kind, checked),
        _ => (ToggleKind::Checkbox, false),
    };
    let switch = kind == ToggleKind::Switch;
    let part = node.part_style(
        if switch { Part::Track } else { Part::Control },
        hovered,
        focused,
    );
    let mark = node.part_style(
        if switch { Part::Thumb } else { Part::Indicator },
        hovered,
        focused,
    );
    let palette = controls::toggle_colors(kind, checked, style);
    let fallback = controls::toggle_size(kind);
    let mut label = style.clone();
    label.overlay(&node.part_style(Part::Label, hovered, focused));
    let px = |value, fallback| match value {
        Some(Size::Px(n)) => n,
        _ => fallback,
    };
    ToggleVisual {
        width: px(part.width, fallback.0),
        height: px(part.height, fallback.1),
        radius: part.rounded.unwrap_or(if switch { 10.0 } else { 4.0 }),
        border_width: part.border_width.unwrap_or(1.0),
        fill: part.background.unwrap_or(palette.fill),
        border: part.border_color.unwrap_or(palette.border),
        mark: mark.background.or(mark.color).unwrap_or(palette.mark),
        thumb_width: if switch { px(mark.width, 16.0) } else { 0.0 },
        thumb_height: if switch { px(mark.height, 16.0) } else { 0.0 },
        thumb_radius: mark.rounded.unwrap_or(8.0),
        gap: style
            .column_gap
            .or(style.gap)
            .unwrap_or(controls::CHECK_GAP),
        label,
    }
}
/// Resolve the range fill independently of its root/track surface.
pub fn range(node: &Node, style: &Style, hovered: bool, focused: bool) -> Style {
    let mut fill = Style::new()
        .background(style.accent_color.unwrap_or(controls::ACCENT))
        .rounded(style.rounded.unwrap_or(0.0));
    let part = node.part_style(Part::Indicator, hovered, focused);
    fill.overlay(&part);
    if part.background.is_none() && part.color.is_some() {
        fill.background = part.color;
    }
    fill
}
/// Resolve editor hint ink without modifying native editing, caret or accessibility ownership.
pub fn placeholder(node: &Node, hovered: bool, focused: bool) -> Color {
    node.part_style(Part::Placeholder, hovered, focused)
        .color
        .unwrap_or(controls::PLACEHOLDER)
}

impl ToggleVisual {
    /// Keep thumb travel and centering identical across native painters, including non-square thumbs.
    pub fn thumb_rect(&self, checked: bool) -> (f32, f32, f32, f32) {
        let inset = ((self.height - self.thumb_height) / 2.0)
            .min((self.width - self.thumb_width) / 2.0)
            .max(0.0);
        (
            if checked {
                self.width - inset - self.thumb_width
            } else {
                inset
            },
            (self.height - self.thumb_height) / 2.0,
            self.thumb_width,
            self.thumb_height,
        )
    }
}
