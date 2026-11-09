//! Shared default metrics and colors for native control bindings.
use inrux_style::Color;
/// Font-size-relative line height used for text measurement and painting.
pub const LINE_HEIGHT: f32 = inrux_style::DEFAULT_LINE_HEIGHT;
/// Checkbox square in logical pixels.
pub const CHECK_SIZE: f32 = 18.0;
/// Gap between checkbox square and label.
pub const CHECK_GAP: f32 = 4.0;
/// Default interactive accent.
pub const ACCENT: Color = Color(29, 112, 98);
/// Default neutral control border.
pub const BORDER: Color = Color(142, 154, 149);

/// Muted text color for empty native input hints.
pub const PLACEHOLDER: Color = Color(104, 113, 109);

/// Derive enabled-button hover feedback without changing layout or authored base colors.
/// Very dark fills brighten; other fills darken so black also has visible feedback.
pub fn hovered_button_color(base: Color) -> Color {
    let (r, g, b) = (base.0, base.1, base.2);
    let dark = (u32::from(r) * 299 + u32::from(g) * 587 + u32::from(b) * 114) < 80_000;
    let channel = |value: u8| {
        let value = u16::from(value);
        if dark {
            (value + (255 - value) * 12 / 100) as u8
        } else {
            (value * 88 / 100) as u8
        }
    };
    Color::rgba(channel(r), channel(g), channel(b), base.3)
}

/// Indicator geometry shared by checkbox/switch painting, layout and focus decoration.
pub fn toggle_size(kind: inrux_components::ToggleKind) -> (f32, f32) {
    match kind {
        inrux_components::ToggleKind::Checkbox => (CHECK_SIZE, CHECK_SIZE),
        inrux_components::ToggleKind::Switch => (34.0, 20.0),
    }
}
/// Paint roles for a boolean indicator; label ink is inherited separately.
#[derive(Clone, Copy, Debug)]
pub struct ToggleColors {
    /// Track or box background.
    pub fill: Color,
    /// Indicator outline.
    pub border: Color,
    /// Checkmark or thumb.
    pub mark: Color,
}
/// Choose dark/light indicator ink with stable contrast on opaque authored accents.
pub fn contrasting_ink(color: Color) -> Color {
    let (r, g, b) = (color.0, color.1, color.2);
    if u32::from(r) * 299 + u32::from(g) * 587 + u32::from(b) * 114 >= 150_000 {
        Color(22, 22, 22)
    } else {
        Color(255, 255, 255)
    }
}
/// Preserve legacy unthemed checkboxes, while authored accents use filled selection marks.
pub fn toggle_colors(
    kind: inrux_components::ToggleKind,
    checked: bool,
    style: &inrux_style::Style,
) -> ToggleColors {
    let accent = style.accent_color.unwrap_or(ACCENT);
    if kind == inrux_components::ToggleKind::Switch {
        ToggleColors {
            fill: if checked { accent } else { BORDER },
            border: if checked { accent } else { BORDER },
            mark: if checked {
                contrasting_ink(accent)
            } else {
                Color(255, 255, 255)
            },
        }
    } else {
        let filled = checked && style.accent_color.is_some();
        ToggleColors {
            fill: if filled { accent } else { Color(255, 255, 255) },
            border: if checked { accent } else { BORDER },
            mark: if filled {
                contrasting_ink(accent)
            } else {
                accent
            },
        }
    }
}
