//! Engine-independent viewport behavior and real selected-provider geometry.
use inrux_adapter::{
    layout::LayoutBox,
    services::{
        clipping::ClipRect,
        scroll::{ScrollKey, ScrollState},
    },
};
use inrux_components::{Node, ScrollOffset, ScrollRequest};
#[cfg(feature = "taffy-layout")]
use inrux_style::Size;
use inrux_style::{Overflow, Style};
fn viewport() -> (Node, LayoutBox, ScrollState) {
    let node = Node::scroll_area(vec![]);
    let bounds = LayoutBox {
        width: 200.0,
        height: 100.0,
        content_width: 200.0,
        content_height: 500.0,
        ..Default::default()
    };
    let mut state = ScrollState::default();
    state.update(&node, &bounds);
    (node, bounds, state)
}
#[test]
fn blocked_nested_events_bubble_and_offsets_clamp_after_resize() {
    let (_, _, mut child) = viewport();
    let (_, mut bounds, mut parent) = viewport();
    assert!(child.wheel(0.0, 1000.0, false));
    assert_eq!(child.metrics().offset.y, 400.0);
    assert!(!child.wheel(0.0, 40.0, false));
    assert!(parent.wheel(0.0, 40.0, false));
    bounds.content_height = 110.0;
    child.update(&Node::scroll_area(vec![]), &bounds);
    assert_eq!(child.metrics().offset.y, 10.0);
    assert!(!child.set_offset(ScrollOffset {
        x: f32::NAN,
        y: 0.0
    }));
}
#[test]
fn requests_apply_once_and_focus_reveal_does_not_fight_wheel_input() {
    let (node, bounds, mut state) = viewport();
    let node = node.scroll_to(ScrollRequest {
        revision: 1,
        offset: ScrollOffset { x: 0.0, y: 200.0 },
    });
    state.update(&node, &bounds);
    assert_eq!(state.metrics().offset.y, 200.0);
    state.wheel(0.0, 40.0, false);
    state.update(&node, &bounds);
    assert_eq!(state.metrics().offset.y, 240.0);
    let target = Some((
        "editor".into(),
        ClipRect {
            x: 0.0,
            y: 440.0,
            width: 30.0,
            height: 20.0,
        },
    ));
    assert!(state.focus(target.clone()));
    assert_eq!(state.metrics().offset.y, 360.0);
    state.wheel(0.0, -40.0, false);
    assert!(!state.focus(target));
    assert_eq!(state.metrics().offset.y, 320.0);
}
#[test]
fn horizontal_keyboard_track_and_capture_use_shared_geometry() {
    let (node, mut bounds, mut state) = viewport();
    let node = node.styled(
        Style::new()
            .overflow_x(Overflow::Auto)
            .overflow_y(Overflow::Hidden),
    );
    bounds.content_width = 600.0;
    state.update(&node, &bounds);
    assert!(state.key(ScrollKey::PageDown));
    assert_eq!(state.metrics().offset.x, 160.0);
    let bar = state.bars()[0];
    assert!(state.press(bar.thumb.x + 2.0, bar.thumb.y + 2.0));
    assert!(state.drag_to(1000.0, bar.thumb.y + 2.0));
    assert_eq!(state.metrics().offset.x, 400.0);
    state.release();
    assert!(!state.dragging());
    state.key(ScrollKey::Home);
    assert_eq!(state.metrics().offset.x, 0.0);
    state.key(ScrollKey::End);
    assert_eq!(state.metrics().offset.x, 400.0);
}
#[test]
fn clipping_intersects_ancestors_and_tiny_viewports_have_finite_bars() {
    let a = ClipRect {
        x: 10.0,
        y: 10.0,
        width: 100.0,
        height: 100.0,
    };
    let b = ClipRect {
        x: 90.0,
        y: 0.0,
        width: 100.0,
        height: 20.0,
    };
    assert_eq!(
        a.intersect(b),
        ClipRect {
            x: 90.0,
            y: 10.0,
            width: 20.0,
            height: 10.0
        }
    );
    assert!(!a.intersect(b).contains(110.0, 15.0));
    let mut state = ScrollState::default();
    state.update(
        &Node::scroll_area(vec![]).styled(Style::new().overflow(Overflow::Scroll)),
        &LayoutBox::default(),
    );
    assert!(state.bars().is_empty());
}
#[test]
fn observation_only_emits_changed_metrics_and_reports_rejection() {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    let calls = Arc::new(AtomicUsize::new(0));
    let count = calls.clone();
    let (node, _, mut state) = viewport();
    let node = node.on_scroll(inrux_core::Emit::new(move |_| {
        count.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }));
    state.notify(&node).unwrap();
    state.notify(&node).unwrap();
    state.wheel(0.0, 40.0, false);
    state.notify(&node).unwrap();
    assert_eq!(calls.load(Ordering::Relaxed), 2);
    let rejected = Node::scroll_area(vec![]).on_scroll(inrux_core::Emit::new(|_| {
        Err(inrux_core::DispatchError::Full)
    }));
    let (_, _, mut fresh) = viewport();
    assert!(
        fresh
            .notify(&rejected)
            .unwrap_err()
            .to_string()
            .contains("queue is full")
    );
}
#[test]
fn overflow_normalizes_axes_and_invalid_leaf_requests_are_rejected() {
    assert_eq!(
        Style::new().overflow_y(Overflow::Hidden).overflow_axes(),
        (Overflow::Auto, Overflow::Hidden)
    );
    assert!(
        inrux_adapter::layout::validate(
            &Node::text("Invalid".into()).styled(Style::new().overflow(Overflow::Hidden))
        )
        .is_err()
    );
    let node = Node::scroll_area(vec![]).scroll_to(ScrollRequest {
        revision: 0,
        offset: ScrollOffset {
            x: 0.0,
            y: f32::INFINITY,
        },
    });
    assert!(inrux_adapter::layout::validate(&node).is_err());
}
#[cfg(feature = "taffy-layout")]
#[test]
fn selected_layout_keeps_content_natural_size_and_reports_extent() {
    let node = Node::scroll_area(
        (0..8)
            .map(|_| Node::text("Row".into()).styled(Style::new().height(Size::Px(40.0))))
            .collect(),
    )
    .styled(Style::new().height(Size::Px(100.0)).gap(4.0));
    let bounds = inrux_adapter::layout::compute(&node, 200.0, None, |_| (20.0, 16.0)).unwrap();
    assert_eq!(bounds.height, 100.0);
    assert_eq!(bounds.children[0].height, 40.0);
    assert_eq!(bounds.content_height, 348.0);
}

/// Mount generations dispose UI state even when a native position has no authored key.
#[test]
fn a_new_generation_resets_offset_capture_and_command_history() {
    let (mut node, bounds, mut state) = viewport();
    state.wheel(0.0, 180.0, false);
    assert_eq!(state.metrics().offset.y, 180.0);
    let bar = state.bars()[0];
    state.press(bar.thumb.x + 1.0, bar.thumb.y + 1.0);
    assert!(state.dragging());
    node.generation = Some("next-mount".into());
    state.update(&node, &bounds);
    assert_eq!(state.metrics().offset.y, 0.0);
    assert!(!state.dragging());
}

/// OS-adjusted signed deltas stay signed; Shift must not remap native X twice.
#[test]
fn wheel_preserves_direction_and_already_horizontal_shift_events() {
    let (node, mut bounds, mut state) = viewport();
    let node = node.styled(
        Style::new()
            .overflow_x(Overflow::Auto)
            .overflow_y(Overflow::Auto),
    );
    bounds.content_width = 600.0;
    state.update(&node, &bounds);
    state.set_offset(ScrollOffset { x: 100.0, y: 100.0 });
    assert!(state.wheel(0.0, 30.0, false));
    assert_eq!(state.metrics().offset.y, 130.0);
    assert!(state.wheel(0.0, -30.0, false));
    assert_eq!(state.metrics().offset.y, 100.0);
    assert!(state.wheel(30.0, 0.0, true));
    assert_eq!(state.metrics().offset, ScrollOffset { x: 130.0, y: 100.0 });
    assert!(state.wheel(0.0, -30.0, true));
    assert_eq!(state.metrics().offset, ScrollOffset { x: 100.0, y: 100.0 });
}

/// A native editor's text is internal overflow, never extra content of its parent pane.
#[cfg(feature = "taffy-layout")]
#[test]
fn long_editor_text_does_not_expand_parent_scroll_extent() {
    for multiline in [false, true] {
        let build = |text: &str| {
            let event = inrux_core::Emit::new(|_| Ok(()));
            let input = if multiline {
                Node::text_area(text.into(), String::new(), event)
            } else {
                Node::input(text.into(), String::new(), event)
            }
            .styled(Style::new().width(Size::Px(180.0)).height(Size::Px(120.0)));
            Node::scroll_area(vec![
                input,
                Node::text("after".into()).styled(Style::new().height(Size::Px(40.0))),
            ])
            .styled(Style::new().height(Size::Px(100.0)))
        };
        let measure = |request: inrux_adapter::layout::TextMeasure<'_>| {
            (
                request.text.len() as f32 * 8.0,
                request.text.lines().count().max(1) as f32 * 20.0,
            )
        };
        let short = inrux_adapter::layout::compute(&build("one"), 200.0, None, measure).unwrap();
        let long = inrux_adapter::layout::compute(
            &build(&"many long lines of text\n".repeat(100)),
            200.0,
            None,
            measure,
        )
        .unwrap();
        assert_eq!(long.content_height, short.content_height);
        assert_eq!(long.content_width, short.content_width);
        assert_eq!(long.children[0].height, 120.0);
    }
}

/// Auto bars have discoverable geometry while idle and reveal on movement; explicit scroll stays visible.
#[test]
fn auto_indicators_start_hidden_and_activity_does_not_change_scrollbar_geometry() {
    let (node, bounds, mut state) = viewport();
    let before = state.bars();
    assert!(!before.is_empty());
    assert!(!state.indicator_active());
    assert!(state.wheel(0.0, 30.0, false));
    assert!(state.indicator_active());
    assert_eq!(state.bars()[0].track, before[0].track);
    assert!(state.take_indicator_timer().is_some());
    assert!(state.take_indicator_timer().is_none());
    state.finish_indicator_timer();
    assert!(state.take_indicator_timer().is_some());
    let mut persistent = ScrollState::default();
    persistent.update(
        &node.styled(Style::new().overflow_y(Overflow::Scroll)),
        &bounds,
    );
    assert!(persistent.indicator_active());
}
