//! Bounded layout controls avoid depending on a scroll service.
use crate::views::layout::State;
use inrux::{Color, Context, Size, Style};
/// Toggle narrow/wide parent geometry without remounting the specimen.
pub fn toggle_width(state: &mut State, _: &Context<State>) {
    state.narrow = !state.narrow;
}
/// Demonstrate weighted sharing between two sibling Spacers.
pub fn toggle_weight(state: &mut State, _: &Context<State>) {
    state.weighted = !state.weighted;
}
/// Keep both examples inside the minimum supported application width.
pub fn specimen_style(narrow: bool) -> Style {
    Style::new()
        .width(Size::Px(if narrow { 360.0 } else { 720.0 }))
        .max_width(Size::Percent(100.0))
        .padding(12.0)
        .background(Color(235, 239, 237))
}
