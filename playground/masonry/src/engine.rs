//! Application startup is the only engine selection point.
use crate::views::engine::State;
use inrux::{Adapter, Events, Node, Size, Style};
/// Window title.
pub const NAME: &str = "Inrux playground · masonry";
/// Header identification.
pub const LABEL: &str = "masonry";
/// Truthful primitive coverage for this independently built application.
pub const SUPPORT: &str =
    "Portable catalogue supported. Select, Dialog and Heartbeat are not implemented here.";
/// Choose only this application's native adapter.
pub fn adapter() -> impl Adapter {
    inrux::backends::Masonry
}

/// Unsupported primitives are disclosed rather than imitated or sent to the adapter.
pub fn specimen(state: &State, _: &Events<State>) -> Node {
    // Keep the shared model contract available while the native service gates remain closed.
    let _ = (
        &state.props,
        &state.selected,
        state.select_disabled,
        state.show_dialog,
        state.heartbeat,
    );
    let _ = (
        crate::features::engine::open_dialog,
        crate::features::engine::close_dialog,
    );
    Node::box_container(vec![
        Node::text("Select · Not supported by this adapter yet".into()),
        Node::text("Dialog · Portable overlay/focus services are still required".into()),
        Node::text("Heartbeat · egui-only diagnostic, not a portable animation API".into()),
        Node::text("Use the egui playground to exercise these native-only examples.".into()),
    ])
    .styled(
        Style::new()
            .width(Size::Fill)
            .padding(18.0)
            .gap(16.0)
            .background(inrux::Color(245, 247, 246))
            .rounded(10.0),
    )
}
