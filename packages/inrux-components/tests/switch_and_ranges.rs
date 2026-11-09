//! Public descriptions keep events, roles and numeric semantics independent of native engines.
use inrux_components::{Node, NodeKind, RangeKind, ToggleKind};
use inrux_core::Emit;
use inrux_style::{Color, parse_css};
/// Switch is an explicitly typed boolean control, never inferred from its class name.
#[test]
fn switch_preserves_controlled_model_event_and_semantic_kind() {
    let (sent, received) = std::sync::mpsc::channel();
    let node = Node::switch(
        "Automatic sync".into(),
        false,
        Emit::new(move |value| {
            sent.send(value).unwrap();
            Ok(())
        }),
    );
    let NodeKind::Checkbox {
        kind,
        checked,
        event,
        ..
    } = &node.kind
    else {
        panic!("expected boolean descriptor")
    };
    assert_eq!(*kind, ToggleKind::Switch);
    assert!(!checked);
    event.emit(true).unwrap();
    assert!(received.recv().unwrap());
    assert_eq!(node.component_type, "Switch");
}
/// Read-only ranges keep their distinct purpose and theme defaults below CSS.
#[test]
fn progress_and_meter_normalize_distinct_bounds_and_accept_stylesheet_overrides() {
    let mut progress = Node::progress("Download".into(), 25.0, 100.0).classes("brand");
    progress.apply_globals(&parse_css(".brand { height: 12px; accent-color: #123456; }").unwrap());
    assert_eq!(progress.style().accent_color, Some(Color(18, 52, 86)));
    assert_eq!(progress.style().height, Some(inrux_style::Size::Px(12.0)));
    let NodeKind::Range(range) = progress.kind else {
        panic!("expected range")
    };
    assert_eq!(range.kind, RangeKind::Progress);
    assert_eq!(range.fraction(), Ok(0.25));
    let meter = Node::meter("Temperature".into(), 0.0, -10.0, 30.0);
    let NodeKind::Range(range) = meter.kind else {
        panic!("expected range")
    };
    assert_eq!(range.kind, RangeKind::Meter);
    assert_eq!(range.fraction(), Ok(0.25));
    assert!(meter.children.is_empty());
}
