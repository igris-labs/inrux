//! Native example state remains owned by the framework coordinator.
use crate::views::engine::State;
use inrux::Context;
/// Open the egui-only modal specimen.
pub fn open_dialog(state: &mut State, _: &Context<State>) {
    state.show_dialog = true;
}
/// Close through the button, Escape or native backdrop dismissal.
pub fn close_dialog(state: &mut State, _: &Context<State>) {
    state.show_dialog = false;
}
