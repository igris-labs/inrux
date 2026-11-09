//! Real interactive controls, their model bindings and readable state output.
use super::{Example, increment};
use crate::views::component_preview::State;
use inrux::{Events, Node, Size, Style};
/// Every edit uses the same model/event protocol as application-authored components.
pub fn preview(state: &State, events: &Events<State>) -> Node {
    let mut children = match state.props.example {
        Example::Button => vec![
            Node::row(vec![
                Node::button("Add to count".into(), Some(events.handler(increment)))
                    .classes("primary")
                    .styled(primary())
                    .disabled(state.disabled),
                Node::button("Disabled button".into(), Some(events.handler(increment)))
                    .disabled(true)
                    .styled(secondary()),
            ])
            .styled(Style::new().gap(10.0).wrap(true)),
            Node::text(format!("Count: {}", state.count)),
        ],
        Example::Switch => vec![
            Node::switch(
                "Automatic sync".into(),
                state.checked,
                events.model(|state, value, _| state.checked = value),
            )
            .classes("headless-switch")
            .disabled(state.disabled),
            Node::text(format!(
                "Automatic sync: {}",
                if state.checked { "on" } else { "off" }
            )),
        ],
        Example::Checkbox => vec![
            Node::checkbox(
                "Enable notifications".into(),
                state.checked,
                events.model(|state, value, _| state.checked = value),
            )
            .classes("headless-checkbox")
            .disabled(state.disabled),
            Node::text(format!(
                "Notifications: {}",
                if state.checked { "enabled" } else { "disabled" }
            )),
        ],
        Example::TextArea => vec![
            Node::model_text_area(
                state.notes.clone(),
                "Write several lines".into(),
                events.model_versioned("component-textarea", |state, value, _| state.notes = value),
            )
            .disabled(state.disabled)
            .styled(field().height(Size::Px(160.0))),
            Node::text(format!(
                "{} characters · {} lines",
                state.notes.chars().count(),
                state.notes.lines().count()
            )),
        ],
        _ => vec![
            Node::text("Workspace name".into()),
            Node::model_input(
                state.value.clone(),
                "Workspace name".into(),
                events.model_versioned("component-input", |state, value, _| state.value = value),
            )
            .disabled(state.disabled)
            .styled(field()),
            Node::text(format!("Value: {}", state.value)),
        ],
    };
    children.push(Node::checkbox(
        "Disable preview control".into(),
        state.disabled,
        events.model(|state, value, _| state.disabled = value),
    ));
    Node::column(children).styled(Style::new().gap(16.0))
}
/// Match ordinary catalogue actions without requiring engine-specific control skins.
pub(super) fn primary() -> Style {
    secondary()
}
/// A restrained neutral action surface, shared by the demonstration factories.
pub(super) fn secondary() -> Style {
    Style::new()
        .height(Size::Px(38.0))
        .padding(8.0)
        .rounded(5.0)
        .border_width(0.0)
}
/// Text controls keep an explicit bounded surface while their text scrolls internally.
pub(super) fn field() -> Style {
    Style::new()
        .width(Size::Fill)
        .height(Size::Px(42.0))
        .border_width(1.0)
        .rounded(5.0)
}
