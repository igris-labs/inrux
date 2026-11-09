//! Form events run on the application coordinator and preserve atomic model ordering.
use crate::views::controls::State;
use inrux::{Color, Context, Style};
/// A model update marks the same committed frame dirty.
pub fn mark_dirty(state: &mut State, _: &Context<State>) {
    state.saved = false;
}
/// Save only in memory, never to user files.
pub fn save(state: &mut State, _: &Context<State>) {
    state.saved = true;
}
/// Programmatic replacement also updates a focused or disabled native editor.
pub fn reset(state: &mut State, _: &Context<State>) {
    state.name = "Inrux Workspace".into();
    state.saved = true;
}
/// Empty input reveals the native placeholder.
pub fn clear(state: &mut State, _: &Context<State>) {
    state.name.clear();
    state.saved = false;
}
/// This deliberately wired handler makes disabled admission observable.
pub fn forbidden(state: &mut State, _: &Context<State>) {
    state.forbidden += 1;
}
/// Explicit shared enabled/disabled paint avoids relying on toolkit palettes.
pub fn primary_style(disabled: bool) -> Style {
    Style::new()
        .padding(8.0)
        .height(inrux::Size::Px(38.0))
        .rounded(6.0)
        .border_width(0.0)
        .background(if disabled {
            Color(218, 226, 221)
        } else {
            Color(29, 112, 98)
        })
        .color(if disabled {
            Color(88, 97, 94)
        } else {
            Color(255, 255, 255)
        })
}
