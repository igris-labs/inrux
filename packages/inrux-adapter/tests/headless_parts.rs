//! Shared native metrics and diagnostics for public headless parts.
use inrux_components::Node;
use inrux_style::{Color, Part, Size, Style};
/// Custom tracks/thumbs share travel geometry; labels and indicators remain independent.
#[test]
fn parts_drive_shared_geometry_and_invalid_overrides_are_rejected() {
    let node = Node::switch("Sync".into(), true, inrux_core::Emit::new(|_| Ok(())))
        .styled_part(
            Part::Track,
            Style::new()
                .width(Size::Px(48.0))
                .height(Size::Px(28.0))
                .rounded(14.0)
                .background(Color(3, 4, 5)),
        )
        .styled_part(
            Part::Thumb,
            Style::new()
                .width(Size::Px(20.0))
                .height(Size::Px(20.0))
                .rounded(5.0)
                .background(Color(8, 9, 10)),
        )
        .styled_part(
            Part::Label,
            Style::new().font_size(18.0).color(Color(11, 12, 13)),
        );
    inrux_adapter::layout::validate(&node).unwrap();
    let visual = inrux_adapter::control_parts::toggle(&node, &node.style(), false, false);
    assert_eq!(
        (visual.width, visual.height, visual.radius),
        (48.0, 28.0, 14.0)
    );
    assert_eq!(visual.thumb_rect(true), (24.0, 4.0, 20.0, 20.0));
    assert_eq!(visual.thumb_rect(false), (4.0, 4.0, 20.0, 20.0));
    assert_eq!(visual.label.font_size, Some(18.0));
    for invalid in [
        node.clone()
            .styled_part(Part::Thumb, Style::new().width(Size::Px(100.0))),
        node.clone()
            .styled_part(Part::Track, Style::new().width(Size::Percent(100.0))),
        node.clone()
            .styled_part(Part::Label, Style::new().padding(8.0)),
        Node::text("Wrong part".into())
            .styled_part(Part::Track, Style::new().background(Color(0, 0, 0))),
    ] {
        assert!(inrux_adapter::layout::validate(&invalid).is_err());
    }
}
/// New control metrics must participate in layout/text requests rather than only decorative paint.
#[cfg(feature = "taffy-layout")]
#[test]
fn label_metrics_and_track_width_change_intrinsic_measurement() {
    let node = Node::checkbox("Sync".into(), false, inrux_core::Emit::new(|_| Ok(())))
        .styled_part(Part::Control, Style::new().width(Size::Px(30.0)))
        .styled_part(Part::Label, Style::new().font_size(18.0));
    let mut measured = false;
    let boxes = inrux_adapter::layout::compute(&node, 300.0, None, |request| {
        measured = true;
        assert_eq!(request.font_size, 18.0);
        (72.0, 22.0)
    })
    .unwrap();
    assert!(measured);
    assert_eq!(boxes.width, 116.0); // Label + control + shared gap and existing control allowance.
}

/// Margins change shared child placement without introducing backend-specific layout algorithms.
#[cfg(feature = "taffy-layout")]
#[test]
fn physical_margins_affect_shared_child_layout() {
    let child = || {
        Node::box_container(vec![]).styled(
            Style::new()
                .width(Size::Px(50.0))
                .height(Size::Px(20.0))
                .margin_top(5.0)
                .margin_left(10.0)
                .margin_bottom(7.0),
        )
    };
    let boxes =
        inrux_adapter::layout::compute(&Node::column(vec![child(), child()]), 200.0, None, |_| {
            (0.0, 0.0)
        })
        .unwrap();
    assert_eq!((boxes.children[0].x, boxes.children[0].y), (10.0, 5.0));
    assert_eq!(boxes.children[1].y, 37.0);
    assert!(
        inrux_adapter::layout::validate(
            &Node::column(vec![]).styled(Style::new().margin_top(f32::INFINITY))
        )
        .is_err()
    );
}
