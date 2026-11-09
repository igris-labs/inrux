//! Native button composition preserves Masonry keyboard and pointer semantics.
use crate::widgets::label::Label;
use crate::widgets::text;
use inrux_style::Style;
use masonry::{
    core::{NewWidget, WidgetMut},
    properties::{Background, CornerRadius},
    widgets::Button,
};
/// Construct a real native button with a styled text child.
pub(crate) fn build(title: &str, style: &Style) -> NewWidget<Button> {
    let label = NewWidget::new_with_props(
        text::label(title, style)
            .single_line()
            .centered_insets(label_insets(style)),
        text::properties(style),
    );
    let mut props = super::control_style::properties(style);
    if let Some(fill) = style.background {
        props.insert(Background::Color(text::color(fill)));
        props.insert(masonry::properties::ActiveBackground(Background::Color(
            text::color(fill),
        )));
    }
    if let Some(radius) = style.rounded {
        props.insert(CornerRadius::all(radius.into()));
    }
    NewWidget::new_with_props(Button::new(label), props)
}
/// Replace properties and label in place so focused buttons keep their identity.
pub(crate) fn update(
    button: &mut WidgetMut<'_, Button>,
    title: &str,
    style: &Style,
    previous: Option<&Style>,
) {
    super::control_style::update(button, style);
    super::property::set(
        button,
        style.background.map(|fill| {
            masonry::properties::ActiveBackground(Background::Color(text::color(fill)))
        }),
    );
    let mut child = Button::child_mut(button);
    let mut label = child.downcast::<Label>();
    Label::set_centered_insets(&mut label, label_insets(style));
    text::update(&mut label, title, style, previous);
}

/// Mirror the native button's content constraints; authored border paint remains inside the control.
fn label_insets(style: &Style) -> masonry::kurbo::Size {
    let border = f64::from(style.border_width.unwrap_or(1.0));
    masonry::kurbo::Size::new(
        (f64::from(style.padding.unwrap_or(16.0)) + border) * 2.0,
        (f64::from(style.padding.unwrap_or(6.0)) + border) * 2.0,
    )
}
