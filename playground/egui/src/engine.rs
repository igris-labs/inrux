//! Application startup is the only engine selection point.
use crate::views::engine::State;
use inrux::{Adapter, Events, Node, Size, Style};
/// Window title.
pub const NAME: &str = "Inrux playground · egui";
/// Header identification.
pub const LABEL: &str = "egui";
/// Truthful primitive coverage for this independently built application.
pub const SUPPORT: &str = "Portable catalogue plus working Select, Dialog and Heartbeat.";
/// Choose only this application's native adapter.
pub fn adapter() -> impl Adapter {
    inrux::backends::Egui
}

/// Render native-only controls with typed model/close events and ordinary child content.
pub fn specimen(state: &State, events: &Events<State>) -> Node {
    use crate::features::engine::{close_dialog, open_dialog};
    Node::box_container(vec![
        Node::text("Select, Dialog and Heartbeat".into()),
        Node::select(
            state.selected.clone(),
            vec!["Balanced".into(), "Performance".into(), "Quiet".into()],
            events.model(|state, value, _| state.selected = value),
        )
        .disabled(state.select_disabled),
        Node::checkbox(
            "Disable Select".into(),
            state.select_disabled,
            events.model(|state, value, _| state.select_disabled = value),
        ),
        Node::text(format!("Selected: {}", state.selected)),
        Node::button("Open dialog".into(), Some(events.handler(open_dialog))),
        Node::checkbox(
            "Show Heartbeat".into(),
            state.heartbeat,
            events.model(|state, value, _| state.heartbeat = value),
        ),
        if state.heartbeat {
            Node::heartbeat()
        } else {
            Node::empty()
        },
        Node::dialog(
            state.show_dialog,
            Node::box_container(vec![
                Node::text("Controlled Dialog".into()),
                Node::text("Close with this button, Escape or the backdrop.".into()),
                Node::button("Close dialog".into(), Some(events.handler(close_dialog))),
            ])
            .styled(Style::new().width(Size::Px(420.0)).padding(20.0).gap(12.0)),
            events.handler(close_dialog),
        ),
    ])
    .styled(
        Style::new()
            .padding(18.0)
            .gap(12.0)
            .background(inrux::Color(245, 247, 246))
            .rounded(10.0),
    )
}
