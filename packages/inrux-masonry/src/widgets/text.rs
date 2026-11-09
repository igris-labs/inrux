//! Text shaping and content-color translation, owned by the Masonry boundary.
use crate::widgets::label::Label;
use inrux_style::{Color, Style};
use masonry::{
    core::{Properties, WidgetMut},
    parley::StyleProperty,
    peniko::color::AlphaColor,
    properties::{ContentColor, DisabledContentColor},
};
/// Convert opaque framework sRGB without exporting a painter color type.
pub(crate) fn color(value: Color) -> AlphaColor<masonry::peniko::color::Srgb> {
    AlphaColor::from_rgba8(value.0, value.1, value.2, value.3)
}
/// Create text using the resolved inherited font size.
pub(crate) fn label(text: &str, style: &Style) -> Label {
    Label::new(text.to_owned())
        .with_style(StyleProperty::FontStack(resolved_family(style)))
        .with_style(StyleProperty::LineHeight(
            masonry::parley::LineHeight::FontSizeRelative(inrux_adapter::controls::LINE_HEIGHT),
        ))
        .with_style(StyleProperty::FontSize(style.font_size.unwrap_or(15.0)))
}
/// Use framework text color while allowing native defaults when none was supplied.
pub(crate) fn properties(style: &Style) -> Properties {
    let mut result = Properties::new();
    if let Some(ink) = style.color {
        result.insert(ContentColor { color: color(ink) });
        result.insert(DisabledContentColor(ContentColor { color: color(ink) }));
    }
    result
}
/// Update a retained label without replacing its widget identity.
pub(crate) fn update(
    label: &mut WidgetMut<'_, Label>,
    text: &str,
    style: &Style,
    previous: Option<&Style>,
) {
    if label.widget.text().as_ref() != text {
        Label::set_text(label, text.to_owned());
    }
    super::property::set(label, Some(line_breaking(style)));
    if previous.is_none() {
        Label::insert_style(
            label,
            StyleProperty::LineHeight(masonry::parley::LineHeight::FontSizeRelative(
                inrux_adapter::controls::LINE_HEIGHT,
            )),
        );
    }
    if previous.is_none_or(|old| old.font_family != style.font_family) {
        Label::insert_style(label, StyleProperty::FontStack(resolved_family(style)));
    }
    if previous.is_none_or(|old| old.font_size.unwrap_or(15.0) != style.font_size.unwrap_or(15.0)) {
        Label::insert_style(
            label,
            StyleProperty::FontSize(style.font_size.unwrap_or(15.0)),
        );
    }
    super::property::set(
        label,
        style.color.map(|ink| ContentColor { color: color(ink) }),
    );
    super::property::set(
        label,
        style
            .color
            .map(|ink| DisabledContentColor(ContentColor { color: color(ink) })),
    );
}

/// Own a named family so retained native text does not borrow a render description.
pub(crate) fn family(name: &str) -> masonry::parley::FontStack<'static> {
    masonry::parley::FontStack::Single(masonry::parley::FontFamily::Named(name.to_owned().into()))
}

/// Removing an authored family restores the native default instead of retaining stale typography.
pub(crate) fn resolved_family(style: &Style) -> masonry::parley::FontStack<'static> {
    family(
        style
            .font_family
            .as_deref()
            .unwrap_or(inrux_text::DEFAULT_FAMILY),
    )
}

/// Native glyph placement follows the same preformatted/paragraph policy as shared measurement.
pub(crate) fn line_breaking(style: &Style) -> masonry::properties::LineBreaking {
    if style.white_space == Some(inrux_style::WhiteSpace::Pre) {
        masonry::properties::LineBreaking::Overflow
    } else {
        masonry::properties::LineBreaking::WordWrap
    }
}
