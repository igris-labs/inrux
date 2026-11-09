//! Ordinary application handlers and typed equivalents of the view's CSS classes.
use crate::views::content::State;
use inrux::{Color, Context, Size, Style};

/// Save in memory; no I/O or framework-specific component behavior is involved.
pub fn save(state: &mut State, _: &Context<State>) {
    state.saved_name.clone_from(&state.name);
    state.saves += 1;
}

/// Restore the last saved value through the normal controlled input model.
pub fn reset(state: &mut State, _: &Context<State>) {
    state.name.clone_from(&state.saved_name);
}

/// Match the CSS example while leaving the framework's component defaults overridable.
pub fn custom_card() -> Style {
    Style::new()
        .width(Size::Px(250.0))
        .padding(12.0)
        .gap(8.0)
        .rounded(6.0)
        .border_width(1.0)
        .border_color(Color(169, 189, 179))
        .background(Color(237, 244, 240))
}

/// The badge remains plain text, even when it looks like a compact chip.
pub fn custom_badge() -> Style {
    Style::new()
        .padding(6.0)
        .rounded(3.0)
        .background(Color(29, 112, 98))
        .color(Color(255, 255, 255))
}

/// Thicker decorative lines use ordinary dimensions and background paint.
pub fn custom_line() -> Style {
    Style::new()
        .height(Size::Px(2.0))
        .background(Color(169, 189, 179))
}
