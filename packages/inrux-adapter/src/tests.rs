//! Shared contract regressions use deterministic coordinator notifications.
use super::*;
use inrux_components::{Component, Events, Node, NodeKind};
use inrux_core::{Context, Scope};
use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
        mpsc,
    },
    thread::ThreadId,
    time::Duration,
};
#[derive(Clone)]
struct Counter {
    value: usize,
    renders: Arc<AtomicUsize>,
    views: mpsc::Sender<ThreadId>,
}
impl Component for Counter {
    type Action = ();
    /// Increment on the application coordinator.
    fn update(&mut self, _: (), _: &Context<Self>) {
        self.value += 1;
    }
    /// Report which thread constructed the description and expose a queued event.
    fn view(&self, events: &Events<Self>, _: &Scope) -> Node {
        self.renders.fetch_add(1, Ordering::SeqCst);
        self.views.send(std::thread::current().id()).unwrap();
        Node::button(self.value.to_string(), Some(events.action(())))
    }
}
/// Every backend consumes the same frame/event flow; UI reads do not rerun application views.
#[test]
fn session_retains_descriptions_and_routes_updates_off_the_ui_thread() {
    let (wake, awakened) = mpsc::channel();
    let (views, viewed) = mpsc::channel();
    let renders = Arc::new(AtomicUsize::new(0));
    let session = Session::start(
        Counter {
            value: 0,
            renders: renders.clone(),
            views,
        },
        Scope::default(),
        vec![],
        move || {
            let _ = wake.send(());
        },
    )
    .unwrap();
    awakened.recv_timeout(Duration::from_secs(3)).unwrap();
    assert_ne!(viewed.recv().unwrap(), std::thread::current().id());
    let frame = session.take_frame().unwrap().unwrap();
    let NodeKind::Button(title, Some(event)) = &frame.kind else {
        panic!("expected a button");
    };
    assert_eq!(title, "0");
    for _ in 0..20 {
        assert!(session.take_frame().unwrap().is_none());
    }
    assert_eq!(renders.load(Ordering::SeqCst), 1);
    event.emit(()).unwrap();
    awakened.recv_timeout(Duration::from_secs(3)).unwrap();
    let next = session.take_frame().unwrap().unwrap();
    assert!(matches!(&next.kind, NodeKind::Button(title, _) if title == "1"));
    assert_eq!(renders.load(Ordering::SeqCst), 2);
}
/// Unavailable controls must fail before translation instead of silently changing semantics.
#[test]
fn capability_validation_rejects_nested_unsupported_primitives_and_duplicate_keys() {
    let capabilities = Capabilities {
        name: "test",
        primitives: &[Primitive::Column, Primitive::Text],
        interaction_styles: false,
    };
    assert!(matches!(
        capabilities.validate(&Node::column(vec![Node::button("Save".into(), None)])),
        Err(AdapterError::Unsupported { .. })
    ));
    let invalid = Node::column(vec![
        Node::text("a".into()).key("same"),
        Node::text("b".into()).key("same"),
    ]);
    assert!(matches!(
        capabilities.validate(&invalid),
        Err(AdapterError::Application(_))
    ));
}
/// Platform dimensions must not receive nonfinite or contradictory sizes.
#[test]
fn window_configuration_rejects_invalid_dimensions() {
    let mut window = WindowOptions::new("Test");
    window.width = f64::NAN;
    assert!(window.validate().is_err());
    window.width = 100.0;
    assert!(window.validate().is_err());
}

/// Explicit boxes wrap with one policy regardless of the native text shaper.
#[cfg(feature = "taffy-layout")]
#[test]
fn shared_layout_wraps_fixed_boxes_and_rejects_invalid_rust_styles() {
    use inrux_style::{Size, Style};
    let child = || {
        Node::text("box".into()).styled(Style::new().width(Size::Px(80.0)).height(Size::Px(20.0)))
    };
    let row = Node::row(vec![child(), child(), child()]).styled(Style::new().gap(10.0).wrap(true));
    let boxes = layout::compute(&row, 180.0, None, |_| (1.0, 1.0)).unwrap();
    assert_eq!(boxes.children[0].y, 0.0);
    assert_eq!(boxes.children[1].x, 90.0);
    assert_eq!(boxes.children[2].y, 30.0);
    assert!(layout::compute(&row, f32::NAN, None, |_| (0.0, 0.0)).is_err());
    let invalid = Node::text("invalid".into()).styled(Style::new().padding(f32::NAN));
    assert!(layout::validate(&invalid).is_err());
}
/// Border width alone uses the inherited foreground and the same radius on both hosts.
#[test]
fn common_surface_resolves_border_defaults() {
    use inrux_style::{Color, Style};
    let surface = surface::Surface::resolve(&Style::new().border_width(2.0).color(Color(1, 2, 3)));
    assert_eq!(surface.radius, 0.0);
    assert_eq!(
        surface::Surface::resolve(&Style::new().rounded(6.0)).radius,
        6.0
    );
    assert_eq!(surface.border, Some((2.0, Color(1, 2, 3))));
}

/// Fractional intrinsic metrics must not round down and wrap a word in the native text box.
#[cfg(feature = "taffy-layout")]
#[test]
fn intrinsic_text_bounds_preserve_fractional_measurements() {
    let boxes = layout::compute(&Node::text("Fractional".into()), 100.0, None, |_| {
        (10.2, 18.2)
    })
    .unwrap();
    assert_eq!((boxes.width, boxes.height), (11.0, 19.0));
}

/// Font configuration is checked before entering any platform event loop.
#[test]
fn bundled_font_metadata_rejects_empty_assets_without_logging_binary_data() {
    let font = FontAsset::new("Example", [1_u8, 2, 3].as_slice());
    assert_eq!(
        format!("{font:?}"),
        "FontAsset { family: \"Example\", bytes: 3 }"
    );
    let mut window = WindowOptions::new("Fonts");
    window.fonts.push(font);
    assert!(window.validate().is_ok());
    window.fonts[0] = FontAsset::new("Example", [].as_slice());
    assert!(window.validate().is_err());
    window.fonts[0] = FontAsset::new(" ", [1_u8].as_slice());
    assert!(window.validate().is_err());
}

/// Both very dark and light fills receive bounded, visible hover feedback.
#[test]
fn button_hover_palette_has_contrast_at_both_color_extremes() {
    use inrux_style::Color;
    assert_eq!(
        controls::hovered_button_color(Color(0, 0, 0)),
        Color(30, 30, 30)
    );
    assert_eq!(
        controls::hovered_button_color(Color(255, 255, 255)),
        Color(224, 224, 224)
    );
    assert_ne!(
        controls::hovered_button_color(controls::ACCENT),
        controls::ACCENT
    );
}
