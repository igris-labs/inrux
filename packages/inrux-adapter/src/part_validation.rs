//! Reject unsupported part targets/declarations before they can silently disappear in native paint.
use crate::AdapterError;
use inrux_components::Node;
use inrux_style::{Part, Size, Style};
/// Every accepted part declaration must have an implemented effect on all current engines.
pub(crate) fn validate(node: &Node) -> Result<(), AdapterError> {
    if node
        .states
        .iter()
        .any(|(state, style)| *state != inrux_style::State::Base && !style.is_paint_only())
        || node
            .part_rules
            .iter()
            .any(|(_, state, style)| *state != inrux_style::State::Base && !style.is_paint_only())
    {
        return Err(AdapterError::Application(
            "conditional styles support paint properties only".into(),
        ));
    }

    for (part, style) in node
        .part_rules
        .iter()
        .map(|(part, _, style)| (part, style))
        .chain(node.part_inline.iter().map(|(part, style)| (part, style)))
    {
        if !node.supports_part(*part) {
            return Err(AdapterError::Application(format!(
                "{} does not expose part {part:?}",
                node.component_type
            )));
        }
        let mut accepted = Style::new();
        match part {
            Part::Track | Part::Control => {
                accepted.width = style.width;
                accepted.height = style.height;
                accepted.background = style.background;
                accepted.border_color = style.border_color;
                accepted.border_width = style.border_width;
                accepted.rounded = style.rounded;
            }
            Part::Thumb => {
                accepted.width = style.width;
                accepted.height = style.height;
                accepted.background = style.background;
                accepted.rounded = style.rounded;
            }
            Part::Indicator => {
                accepted.background = style.background;
                accepted.color = style.color;
                accepted.rounded = if matches!(node.kind, inrux_components::NodeKind::Range(_)) {
                    style.rounded
                } else {
                    None
                };
            }
            Part::Label => {
                accepted.color = style.color;
                accepted.font_size = style.font_size;
                accepted.font_family.clone_from(&style.font_family);
            }
            Part::Placeholder => accepted.color = style.color,
        }
        if accepted != *style {
            return Err(AdapterError::Application(format!(
                "unsupported property for {} part {part:?}",
                node.component_type
            )));
        }
        for value in [style.width, style.height].into_iter().flatten() {
            if !matches!(value, Size::Px(n) if n.is_finite() && n > 0.0) {
                return Err(AdapterError::Application(
                    "part dimensions require finite positive logical px".into(),
                ));
            }
        }
        for value in [style.border_width, style.rounded, style.font_size]
            .into_iter()
            .flatten()
        {
            if !value.is_finite() || value < 0.0 {
                return Err(AdapterError::Application(
                    "part metrics must be finite and nonnegative".into(),
                ));
            }
        }
    }
    if matches!(node.kind, inrux_components::NodeKind::Checkbox { .. }) {
        let visual = crate::control_parts::toggle(node, &node.style(), false, false);
        if visual.thumb_width > visual.width || visual.thumb_height > visual.height {
            return Err(AdapterError::Application(
                "Switch thumb must fit inside its track".into(),
            ));
        }
    }
    Ok(())
}
