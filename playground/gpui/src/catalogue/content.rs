//! Presentation primitives retain ordinary interactive child behavior.
use super::{Example, controls, increment};
use crate::views::component_preview::State;
use inrux::{Color, Events, Node, Size, Style};
/// Keep presentation examples small enough to see their behavior and corresponding usage together.
pub fn preview(state: &State, events: &Events<State>) -> Node {
    let action = || {
        Node::button("Update example".into(), Some(events.handler(increment)))
            .styled(controls::secondary())
    };
    let children = match state.props.example {
        Example::Text => vec![
            Node::text("Build a native interface with Rust.".into())
                .styled(Style::new().font_size(22.0)),
            Node::text(format!("Computed from local state: {}", state.count)),
            action(),
        ],
        Example::Badge => vec![
            Node::row(vec![
                Node::badge("Ready".into()),
                Node::badge(format!("Updates: {}", state.count)).styled(
                    Style::new()
                        .background(Color(226, 240, 233))
                        .color(Color(24, 93, 74)),
                ),
            ])
            .styled(Style::new().gap(12.0)),
            action(),
        ],
        Example::Card => vec![Node::card(vec![
            Node::text("Workspace settings".into()).styled(Style::new().font_size(18.0)),
            Node::separator(false),
            Node::model_input(
                state.value.clone(),
                "Workspace name".into(),
                events.model_versioned("component-card", |state, value, _| state.value = value),
            )
            .styled(controls::field()),
            Node::button("Save workspace".into(), Some(events.handler(increment)))
                .classes("primary")
                .styled(controls::primary()),
            Node::text(format!("Saves: {} · {}", state.count, state.value)),
        ])],
        _ => vec![
            Node::text("A horizontal separator divides related content.".into()),
            Node::separator(false),
            Node::row(vec![
                Node::badge("Before".into()),
                Node::separator(true).styled(Style::new().height(Size::Px(28.0))),
                Node::badge("After".into()),
            ])
            .styled(Style::new().gap(16.0)),
            Node::text("Separators are decorative, not focus targets.".into()),
        ],
    };
    Node::column(children).styled(Style::new().gap(16.0))
}
