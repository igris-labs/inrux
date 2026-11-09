//! Native defaults for the prototype's light surfaces; application styles still override them.
use crate::widgets::input::{TextArea, TextInput};
use crate::widgets::label::Label;
use masonry::{
    core::DefaultProperties,
    peniko::Color,
    properties::{
        ActiveBackground, Background, BorderColor, BorderWidth, CaretColor, ContentColor,
        CornerRadius, Padding, PlaceholderColor, SelectionColor, UnfocusedSelectionColor,
    },
    widgets::{Button, SizedBox},
};
/// Start with native accessibility/interaction properties and replace dark-theme colors.
pub(crate) fn defaults() -> DefaultProperties {
    let mut properties = masonry::theme::default_property_set();
    let ink = Color::from_rgb8(27, 30, 33);
    let accent = crate::widgets::text::color(inrux_adapter::controls::ACCENT);
    let placeholder = crate::widgets::text::color(inrux_adapter::controls::PLACEHOLDER);
    properties.insert::<Label, _>(ContentColor::new(ink));
    properties.insert::<Label, _>(Padding::all(0.0));
    // Leave room for the native line box now that the text viewport clips its own bounds.
    properties.insert::<TextInput, _>(Padding::from_vh(8.0, 12.0));
    properties.insert::<TextInput, _>(BorderWidth { width: 1.0 });
    properties.insert::<TextInput, _>(BorderColor {
        color: masonry::theme::ZYNC_600,
    });
    properties.insert::<TextInput, _>(CornerRadius { radius: 4.0 });
    properties.insert::<TextArea<true>, _>(ContentColor::new(ink));
    properties.insert::<TextArea<false>, _>(ContentColor::new(ink));
    properties.insert::<TextInput, _>(UnfocusedSelectionColor(SelectionColor {
        color: masonry::theme::DISABLED_TEXT_COLOR,
    }));
    properties.insert::<TextInput, _>(CaretColor { color: accent });
    properties.insert::<TextInput, _>(PlaceholderColor::new(placeholder));
    properties.insert::<TextArea<true>, _>(CaretColor { color: accent });
    properties.insert::<TextArea<false>, _>(CaretColor { color: accent });
    properties.insert::<Button, _>(Background::Color(Color::from_rgb8(235, 235, 235)));
    properties.insert::<Button, _>(ActiveBackground(Background::Color(Color::from_rgb8(
        215, 215, 215,
    ))));
    properties.insert::<TextInput, _>(SelectionColor {
        color: Color::from_rgb8(208, 235, 226),
    });
    properties.insert::<TextArea<true>, _>(SelectionColor {
        color: Color::from_rgb8(208, 235, 226),
    });
    properties.insert::<SizedBox, _>(Background::Color(Color::WHITE));
    properties
}
