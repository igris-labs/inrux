//! Native text editing uses the shared versioned model protocol.
use crate::widgets::text;
mod area;
mod field;
pub(crate) use area::TextArea;
pub(crate) use field::TextInput;
use inrux_style::Style;
use masonry::{
    core::{NewWidget, WidgetMut},
    parley::StyleProperty,
    properties::ContentColor,
};
/// Build a native editor with a known action-producing text-area identity.
pub(crate) fn build(
    value: &str,
    placeholder: &str,
    style: &Style,
    multiline: bool,
) -> NewWidget<TextInput> {
    let area = TextArea::new_editable(value)
        .with_word_wrap(multiline)
        .with_style(StyleProperty::LineHeight(
            masonry::parley::LineHeight::FontSizeRelative(inrux_adapter::controls::LINE_HEIGHT),
        ))
        .with_style(StyleProperty::FontSize(style.font_size.unwrap_or(15.0)));
    let area = if let Some(family) = &style.font_family {
        area.with_style(StyleProperty::FontStack(crate::widgets::text::family(
            family,
        )))
    } else {
        area
    };
    let area = NewWidget::new_with_props(area, text::properties(style));
    NewWidget::new_with_props(
        TextInput::from_text_area(area)
            .with_clip(true)
            .with_placeholder(placeholder.to_owned()),
        super::control_style::properties(style),
    )
}
/// Reconcile text only when it differs, preserving the native selection during acknowledgements.
pub(crate) fn update(
    input: &mut WidgetMut<'_, TextInput>,
    value: &str,
    placeholder: &str,
    style: &Style,
    placeholder_ink: inrux_style::Color,
    previous: Option<&Style>,
    previous_placeholder: Option<inrux_style::Color>,
) {
    super::control_style::update(input, style);
    if TextInput::placeholder_mut(input).widget.text().as_ref() != placeholder {
        TextInput::set_placeholder(input, placeholder.to_owned());
    }
    // The native placeholder is a separate Label and does not inherit TextArea typography.
    let previous_hint = previous.map(|old| {
        old.clone()
            .color(previous_placeholder.unwrap_or(inrux_adapter::controls::PLACEHOLDER))
    });
    let multiline = TextInput::text_mut(input).widget.multiline();
    let hint_style = if multiline {
        style.clone().color(placeholder_ink)
    } else {
        style
            .clone()
            .color(placeholder_ink)
            .white_space(inrux_style::WhiteSpace::Pre)
    };
    text::update(
        &mut TextInput::placeholder_mut(input),
        placeholder,
        &hint_style,
        previous_hint.as_ref(),
    );
    let mut area = TextInput::text_mut(input);
    if previous.is_none_or(|old| old.font_family != style.font_family) {
        TextArea::insert_style(
            &mut area,
            StyleProperty::FontStack(text::resolved_family(style)),
        );
    }
    if area.widget.text() != value {
        TextArea::reset_text(&mut area, value);
    }
    if previous.is_none_or(|old| old.font_size.unwrap_or(15.0) != style.font_size.unwrap_or(15.0)) {
        TextArea::insert_style(
            &mut area,
            StyleProperty::FontSize(style.font_size.unwrap_or(15.0)),
        );
    }
    super::property::set(
        &mut area,
        style.color.map(|ink| ContentColor {
            color: text::color(ink),
        }),
    );
}
