//! Determinate progress and bounded measurement exercise typed data, theme accents and gap axes.
use super::Example;
use crate::views::component_preview::State;
use inrux::{Context, Events, Node, Style};
/// Render real controlled values; no animation timer or fake background work is involved.
pub fn preview(state: &State, events: &Events<State>) -> Node {
    let progress = state.props.example == Example::Progress;
    let value = if progress {
        f64::from(state.count) * 10.0
    } else {
        f64::from(state.count) * 5.0 - 10.0
    };
    let indicator = if progress {
        Node::progress("Build progress".into(), value, 100.0)
    } else {
        Node::meter("Temperature".into(), value, -10.0, 40.0)
    };
    Node::column(vec![
        Node::text(if progress {
            format!("Completion: {value:.0}%")
        } else {
            format!("Temperature: {value:.0} °C (−10 to 40)")
        }),
        indicator,
        Node::row(vec![
            Node::button("Decrease value".into(), Some(events.handler(decrease)))
                .classes("button")
                .disabled(state.count == 0),
            Node::button("Increase value".into(), Some(events.handler(increase)))
                .classes("button")
                .disabled(state.count == 10),
        ])
        .styled(Style::new().column_gap(12.0).row_gap(8.0).wrap(true)),
        Node::checkbox(
            "Use violet accent".into(),
            state.checked,
            events.model(|state, value, _| state.checked = value),
        ),
    ])
    .classes(if state.checked {
        "range-preview custom-accent"
    } else {
        "range-preview"
    })
}
/// Respect range bounds while exercising ordinary native button event admission.
fn increase(state: &mut State, _: &Context<State>) {
    state.count = state.count.saturating_add(1).min(10);
}
/// A bounded measurement cannot decrease past its minimum.
fn decrease(state: &mut State, _: &Context<State>) {
    state.count = state.count.saturating_sub(1);
}
