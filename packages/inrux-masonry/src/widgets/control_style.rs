//! Explicit Inrux paint/spacing overrides for native controls.
use super::text;
use inrux_style::Style;
use masonry::{
    core::{HasProperty, Properties, Widget, WidgetMut},
    properties::*,
};
/// Native property storage is private; shared styles remain authoritative.
pub(crate) fn properties(style: &Style) -> Properties {
    let mut props = text::properties(style);
    if let Some(fill) = style.background {
        props.insert(Background::Color(text::color(fill)));
        props.insert(DisabledBackground(Background::Color(text::color(fill))));
    }
    if let Some(radius) = style.rounded {
        props.insert(CornerRadius::all(radius.into()));
    }
    if let Some(width) = style.border_width {
        props.insert(BorderWidth {
            width: width.into(),
        });
    }
    if let Some(ink) = style.border_color {
        props.insert(BorderColor {
            color: text::color(ink),
        });
    }
    if let Some(padding) = style.padding {
        props.insert(Padding::all(padding.into()));
    }
    props
}
/// Update an existing native control rather than recreating focused state.
pub(crate) fn update<W>(widget: &mut WidgetMut<'_, W>, style: &Style)
where
    W: Widget
        + HasProperty<Background>
        + HasProperty<DisabledBackground>
        + HasProperty<BorderWidth>
        + HasProperty<BorderColor>
        + HasProperty<CornerRadius>
        + HasProperty<Padding>,
{
    super::property::set(
        widget,
        style
            .background
            .map(|fill| Background::Color(text::color(fill))),
    );
    super::property::set(
        widget,
        style
            .background
            .map(|fill| DisabledBackground(Background::Color(text::color(fill)))),
    );
    super::property::set(
        widget,
        style.rounded.map(|radius| CornerRadius::all(radius.into())),
    );
    super::property::set(
        widget,
        style.border_width.map(|width| BorderWidth {
            width: width.into(),
        }),
    );
    super::property::set(
        widget,
        style.border_color.map(|ink| BorderColor {
            color: text::color(ink),
        }),
    );
    super::property::set(
        widget,
        style.padding.map(|padding| Padding::all(padding.into())),
    );
}
