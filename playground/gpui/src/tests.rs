//! Exercise the independently compiled application through published nodes and real event handles.
use inrux::{Adapter, Node, NodeKind};
use inrux_adapter::Session;
use std::{
    sync::{Arc, mpsc},
    time::Duration,
};

/// Coordinate with committed frames rather than guessing worker scheduling with sleeps.
pub struct Harness {
    session: Session<crate::app::State>,
    wake: mpsc::Receiver<()>,
    pub root: Arc<Node>,
}
impl Harness {
    /// Start the actual App.iru graph with its global stylesheet and injected scope.
    pub fn new() -> Self {
        let (sent, wake) = mpsc::channel();
        let styles = crate::inrux_global_styles();
        let initial = crate::app::setup();
        let scope = crate::app::scope().provide(initial.theme.sheet.clone());
        let session = Session::start(initial, scope, styles, move || {
            let _ = sent.send(());
        })
        .unwrap();
        let mut result = Self {
            session,
            wake,
            root: Arc::new(Node::empty()),
        };
        result.wait(|node| has_text(node, "Live preview"));
        result.open("Layout", "Box, Container, Row and Spacer");
        result
    }
    /// Wait until a state transition is visible through the normal publication path.
    pub fn wait(&mut self, predicate: impl Fn(&Node) -> bool) {
        loop {
            self.wake.recv_timeout(Duration::from_secs(8)).unwrap();
            if let Some(node) = self.session.take_frame().unwrap() {
                self.root = node;
                if predicate(&self.root) {
                    return;
                }
            }
        }
    }
    /// Queue an ordinary button event; renderer admission is checked in native harness tests.
    pub fn click(&self, title: &str) {
        let node = find(
            &self.root,
            &|node| matches!(&node.kind, NodeKind::Button(text, _) if text == title),
        )
        .unwrap_or_else(|| panic!("missing button {title}"));
        let NodeKind::Button(_, Some(event)) = &node.kind else {
            panic!("missing event")
        };
        event.emit(()).unwrap();
    }
    /// Select a view and wait for its own state to mount.
    pub fn open(&mut self, tab: &str, title: &str) {
        self.click(tab);
        self.wait(|node| has_text(node, title));
    }
}
/// Search descendants without depending on transparent/layout wrapper depth.
pub fn find<'a>(node: &'a Node, predicate: &impl Fn(&Node) -> bool) -> Option<&'a Node> {
    if predicate(node) {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| find(child, predicate))
}
/// Locate exact text content in the published description.
pub fn has_text(node: &Node, expected: &str) -> bool {
    find(
        node,
        &|node| matches!(&node.kind, NodeKind::Text(text) if text == expected),
    )
    .is_some()
}
/// Get a native controlled input by its current model value.
fn input(node: &Node, value: &str) -> inrux::InputEvent {
    let found = find(
        node,
        &|node| matches!(&node.kind, NodeKind::Input { value: text, .. } if text == value),
    )
    .unwrap();
    let NodeKind::Input { event, .. } = &found.kind else {
        unreachable!()
    };
    event.clone()
}
/// Read one retained child by its displayed local counter.
pub fn row<'a>(node: &'a Node, text: &str) -> &'a Node {
    find(node, &|node| {
        node.children
            .iter()
            .any(|child| matches!(&child.kind, NodeKind::Text(value) if value == text))
    })
    .unwrap()
}

/// Observe key order through actual rendered child labels.
pub fn row_order(node: &Node) -> Vec<String> {
    let mut labels = Vec::new();
    fn collect(node: &Node, labels: &mut Vec<String>) {
        if let NodeKind::Text(text) = &node.kind
            && text.starts_with("Row ")
        {
            labels.push(text.clone());
        }
        for child in &node.children {
            collect(child, labels);
        }
    }
    collect(node, &mut labels);
    labels
}

#[test]
fn layout_controls_resize_and_change_growth_with_the_same_bundled_family() {
    let mut app = Harness::new();
    app.click("Use narrow specimen");
    app.wait(|node| {
        find(node, &|node| {
            node.style().width == Some(inrux::Size::Px(360.0))
        })
        .is_some()
    });
    app.click("Double the first spacer");
    app.wait(|node| find(node, &|node| node.style().flex_grow == Some(2.0)).is_some());
    fn font(node: &Node) {
        assert_eq!(node.style().font_family.as_deref(), Some("Noto Sans"));
        for child in &node.children {
            font(child);
        }
    }
    font(&app.root);
}

#[test]
fn controls_edit_save_reset_clear_and_publish_disabled_states() {
    let mut app = Harness::new();
    app.open("Controls", "Input models and Button states");
    input(&app.root, "Inrux Workspace")
        .emit(1, "Community workspace".into())
        .unwrap();
    app.wait(|node| has_text(node, "You have unsaved changes."));
    app.click("Save changes");
    app.wait(|node| has_text(node, "All changes saved in memory."));
    app.click("Reset name");
    app.wait(|node| find(node, &|node| matches!(&node.kind, NodeKind::Input { value, .. } if value == "Inrux Workspace")).is_some());
    let checkbox = find(&app.root, &|node| matches!(&node.kind, NodeKind::Checkbox { label, .. } if label == "Disable the name input")).unwrap();
    if let NodeKind::Checkbox { event, .. } = &checkbox.kind {
        event.emit(true).unwrap();
    }
    app.wait(|node| {
        find(node, &|node| {
            matches!(node.kind, NodeKind::Input { .. }) && node.disabled
        })
        .is_some()
    });
    app.click("Clear name");
    app.wait(|node| {
        find(
            node,
            &|node| matches!(&node.kind, NodeKind::Input { value, .. } if value.is_empty()),
        )
        .is_some()
    });
    assert!(has_text(
        &app.root,
        "Notifications: false | Choice: true | Disabled activations: 0"
    ));
}

#[test]
fn custom_models_payloads_and_independent_computed_values_work() {
    let mut app = Harness::new();
    app.open("Composition", "Props, emitted events and computed state");
    input(&app.root, "Inrux Workspace")
        .emit(1, "Custom model".into())
        .unwrap();
    app.wait(|node| {
        has_text(
            node,
            "Custom model changed; confirmation emits its String payload.",
        )
    });
    app.click("Confirm name");
    app.wait(|node| has_text(node, "Confirmed: Custom model"));
    app.click("Increment");
    app.wait(|node| has_text(node, "First counter: 1"));
    assert!(has_text(&app.root, "Double: 2"));
    assert!(has_text(&app.root, "Second counter: 0"));
    assert!(
        find(&app.root, &|node| node.style().border_color
            == Some(inrux::Color(29, 112, 98)))
        .is_some()
    );
}

#[test]
fn keyed_children_keep_notes_on_reorder_and_reset_on_remount() {
    let mut app = Harness::new();
    app.open("Composition", "Props, emitted events and computed state");
    let first = row(&app.root, "Row 1: 0");
    let event = input(first, "");
    event.emit(1, "owned".into()).unwrap();
    app.wait(|node| {
        find(
            row(node, "Row 1: 0"),
            &|node| matches!(&node.kind, NodeKind::Input { value, .. } if value == "owned"),
        )
        .is_some()
    });
    let retained_key = row(&app.root, "Row 1: 0").generation.clone();
    app.click("Reverse rows");
    app.wait(|node| row_order(node) == ["Row 2: 0", "Row 1: 0"]);
    assert_eq!(row(&app.root, "Row 1: 0").generation, retained_key);
    assert!(
        find(
            row(&app.root, "Row 1: 0"),
            &|node| matches!(&node.kind, NodeKind::Input { value, .. } if value == "owned")
        )
        .is_some()
    );
    app.click("Remove first");
    app.wait(|node| !has_text(node, "Row 2: 0"));
    app.click("Reverse rows");
    app.wait(|node| has_text(node, "Row 1: 0"));
    app.click("Remove first");
    app.wait(|node| !has_text(node, "Row 1: 0"));
    assert!(event.emit(2, "stale".into()).is_err());
    app.click("Restore rows");
    app.wait(|node| has_text(node, "Row 1: 0") && has_text(node, "Row 2: 0"));
    assert_ne!(row(&app.root, "Row 1: 0").generation, retained_key);
    assert!(
        find(
            row(&app.root, "Row 1: 0"),
            &|node| matches!(&node.kind, NodeKind::Input { value, .. } if value.is_empty())
        )
        .is_some()
    );
}

#[test]
fn work_completes_cancels_and_keeps_inputs_and_other_events_available() {
    let mut app = Harness::new();
    app.open("Background", "Cancellable work without blocking input");
    app.click("Run background task");
    app.wait(|node| has_text(node, "Running. Name editing and counters remain available."));
    input(&app.root, "Inrux Workspace")
        .emit(1, "Still editing".into())
        .unwrap();
    app.click("Independent count: 0");
    app.wait(|node| find(node, &|node| matches!(&node.kind, NodeKind::Button(text, _) if text == "Independent count: 1")).is_some());
    assert!(
        find(
            &app.root,
            &|node| matches!(&node.kind, NodeKind::Input { value, .. } if value == "Still editing")
        )
        .is_some()
    );
    app.click("Cancel task");
    app.wait(|node| has_text(node, "Cancelled. Start again whenever you are ready."));
    app.click("Run background task");
    app.wait(|node| has_text(node, "Progress: 100% | Completed tasks: 1"));
}

#[test]
fn navigating_away_disposes_view_events_and_owned_work() {
    let mut app = Harness::new();
    app.open("Background", "Cancellable work without blocking input");
    let event = input(&app.root, "Inrux Workspace");
    app.click("Run background task");
    app.wait(|node| has_text(node, "Running. Name editing and counters remain available."));
    app.open("Controls", "Input models and Button states");
    assert!(event.emit(1, "stale".into()).is_err());
    app.open("Background", "Cancellable work without blocking input");
    assert!(has_text(&app.root, "Progress: 0% | Completed tasks: 0"));
}

#[test]
fn engine_view_never_sends_unsupported_primitives_to_its_adapter() {
    let mut app = Harness::new();
    app.open("Engine", "Engine support");
    assert!(
        crate::engine::adapter()
            .capabilities()
            .validate(&app.root)
            .is_ok()
    );
    if crate::engine::LABEL == "egui" {
        let select = find(&app.root, &|node| {
            matches!(node.kind, NodeKind::Select { .. })
        })
        .unwrap();
        if let NodeKind::Select { event, .. } = &select.kind {
            event.emit("Quiet".into()).unwrap();
        }
        app.wait(|node| has_text(node, "Selected: Quiet"));
        let disabled = find(&app.root, &|node| matches!(&node.kind, NodeKind::Checkbox { label, .. } if label == "Disable Select")).unwrap();
        if let NodeKind::Checkbox { event, .. } = &disabled.kind {
            event.emit(true).unwrap();
        }
        app.wait(|node| {
            find(node, &|node| {
                matches!(node.kind, NodeKind::Select { .. }) && node.disabled
            })
            .is_some()
        });
        let heartbeat = find(&app.root, &|node| matches!(&node.kind, NodeKind::Checkbox { label, .. } if label == "Show Heartbeat")).unwrap();
        if let NodeKind::Checkbox { event, .. } = &heartbeat.kind {
            event.emit(false).unwrap();
        }
        app.wait(|node| find(node, &|node| matches!(node.kind, NodeKind::Heartbeat)).is_none());
        app.click("Open dialog");
        app.wait(|node| {
            find(node, &|node| {
                matches!(node.kind, NodeKind::Dialog { open: true, .. })
            })
            .is_some()
        });
        app.click("Close dialog");
        app.wait(|node| {
            find(node, &|node| {
                matches!(node.kind, NodeKind::Dialog { open: false, .. })
            })
            .is_some()
        });
    } else {
        assert!(has_text(
            &app.root,
            "Select · Not supported by this adapter yet"
        ));
        assert!(
            find(&app.root, &|node| matches!(
                node.kind,
                NodeKind::Select { .. } | NodeKind::Dialog { .. } | NodeKind::Heartbeat
            ))
            .is_none()
        );
    }
}

/// Live layout changes preserve model state and source identity, and compile all new CSS properties.
#[test]
fn more_layout_reflows_without_discarding_controlled_values() {
    use inrux::{Display, FlexDirection};
    let mut app = Harness::new();
    app.open("More layout", "Flex direction and retained input");
    let native=find(&app.root,&|node| matches!(&node.kind,NodeKind::Input { placeholder,.. } if placeholder=="This edit survives direction changes")).unwrap();
    let identity = (native.key.clone(), native.generation.clone());
    if let NodeKind::Input { event, .. } = &native.kind {
        event.emit(1, "Retained edit".into()).unwrap();
    }
    app.wait(|node| {
        find(
            node,
            &|node| matches!(&node.kind,NodeKind::Input { value,.. } if value=="Retained edit"),
        )
        .is_some()
    });
    app.click("Use column direction");
    app.wait(|node| {
        find(node, &|node| {
            node.style().flex_direction == Some(FlexDirection::Column)
        })
        .is_some()
    });
    let native = find(
        &app.root,
        &|node| matches!(&node.kind,NodeKind::Input { value,.. } if value=="Retained edit"),
    )
    .unwrap();
    assert_eq!((native.key.clone(), native.generation.clone()), identity);
    app.click("Count: 0");
    app.wait(|node| {
        find(
            node,
            &|node| matches!(&node.kind,NodeKind::Button(text,_) if text=="Count: 1"),
        )
        .is_some()
    });
    app.click("Use three columns");
    app.wait(|node| {
        find(node, &|node| {
            node.style().display == Some(Display::Grid) && node.style().grid_columns == Some(3)
        })
        .is_some()
    });
    app.click("Use landscape ratio");
    app.wait(|node| find(node, &|node| node.style().aspect_ratio == Some(16.0 / 9.0)).is_some());
    assert!(has_text(&app.root, "Alpha") && has_text(&app.root, "Zeta"));
    assert!(
        crate::engine::adapter()
            .capabilities()
            .validate(&app.root)
            .is_ok()
    );
}

/// New surfaces compose normal controls and preserve identity across saved/dirty presentation.
#[test]
fn content_card_edits_saves_resets_and_disposes_owned_events() {
    let mut app = Harness::new();
    app.open("Content", "Card, Badge and Separator");
    let current = find(&app.root, &|node| {
        node.key.as_deref() == Some("content-name")
    })
    .unwrap();
    let identity = (current.key.clone(), current.generation.clone());
    let event = input(&app.root, "Inrux Workspace");
    event.emit(1, "Community workspace".into()).unwrap();
    app.wait(|node| has_text(node, "Unsaved changes"));
    let pending = find(
        &app.root,
        &|node| matches!(&node.kind,NodeKind::Text(text) if text=="Unsaved changes"),
    )
    .unwrap();
    assert_eq!(
        pending.style().background,
        Some(inrux::Color(255, 240, 209))
    );
    app.click("Reset card");
    app.wait(|node| has_text(node, "Saved"));
    assert!(
        find(
            &app.root,
            &|node| matches!(&node.kind,NodeKind::Input { value,.. } if value=="Inrux Workspace")
        )
        .is_some()
    );
    event.emit(2, "Saved workspace".into()).unwrap();
    app.wait(|node| has_text(node, "Unsaved changes"));
    app.click("Save card");
    app.wait(|node| has_text(node, "Saves: 1") && has_text(node, "Saved"));
    event.emit(3, "Discard this".into()).unwrap();
    app.wait(|node| has_text(node, "Unsaved changes"));
    app.click("Reset card");
    app.wait(|node| has_text(node, "Saved"));
    let current = find(&app.root, &|node| {
        node.key.as_deref() == Some("content-name")
    })
    .unwrap();
    assert_eq!((current.key.clone(), current.generation.clone()), identity);
    assert!(matches!(&current.kind, NodeKind::Input { value,.. } if value=="Saved workspace"));
    assert!(
        crate::engine::adapter()
            .capabilities()
            .validate(&app.root)
            .is_ok()
    );
    app.open("Layout", "Box, Container, Row and Spacer");
    assert!(event.emit(4, "stale".into()).is_err());
    app.open("Content", "Card, Badge and Separator");
    assert!(has_text(&app.root, "Saves: 0"));
    assert!(
        find(
            &app.root,
            &|node| matches!(&node.kind,NodeKind::Input { value,.. } if value=="Inrux Workspace")
        )
        .is_some()
    );
}

/// The actual scoped CSS and equivalent Rust helpers resolve to the same public styles.
#[test]
fn content_css_and_rust_examples_share_resolved_styles_and_font() {
    let mut app = Harness::new();
    app.open("Content", "Card, Badge and Separator");
    let css = find(&app.root, &|node| {
        node.classes
            .iter()
            .any(|class| class == "custom-content-card")
    })
    .unwrap();
    let rust = find(&app.root, &|node| {
        node.inline == crate::features::content::custom_card()
    })
    .unwrap();
    assert_eq!(css.style(), rust.style());
    for (left, right) in css.children.iter().zip(&rust.children) {
        assert_eq!(left.style(), right.style());
    }
    assert_eq!(
        css.children[0].children[0].style(),
        rust.children[0].children[0].style()
    );
    assert_eq!(css.style().font_family.as_deref(), Some("Noto Sans"));
}

/// The scrolling screen uses the same typed observer/command path on every selected engine.
#[test]
fn scrolling_commands_resize_and_child_events_use_the_portable_application_graph() {
    let mut app = Harness::new();
    app.open("Scrolling", "ScrollArea and clipped viewports");
    let scroll = find(&app.root, &|node| {
        node.key.as_deref() == Some("outer-viewport")
    })
    .unwrap();
    assert!(scroll.on_scroll.is_some());
    let initial = scroll.scroll_request.unwrap();
    app.click("Scroll to end");
    app.wait(|node| {
        find(node, &|node| node.key.as_deref() == Some("outer-viewport"))
            .unwrap()
            .scroll_request
            .unwrap()
            .revision
            > initial.revision
    });
    let scroll = find(&app.root, &|node| {
        node.key.as_deref() == Some("outer-viewport")
    })
    .unwrap();
    assert_eq!(scroll.scroll_request.unwrap().offset.y, f32::MAX);
    scroll
        .on_scroll
        .as_ref()
        .unwrap()
        .emit(inrux::ScrollMetrics {
            offset: inrux::ScrollOffset { x: 0.0, y: 80.0 },
            width: 700.0,
            height: 410.0,
            content_width: 700.0,
            content_height: 1000.0,
        })
        .unwrap();
    app.wait(|node| has_text(node, "Offset: 80 / 590"));
    app.click("Resize viewport");
    app.wait(|node| {
        find(node, &|node| node.key.as_deref() == Some("outer-viewport"))
            .unwrap()
            .style()
            .height
            == Some(inrux::Size::Px(240.0))
    });
    app.click("Nested row 1");
    app.wait(|node| has_text(node, "Actions: 1"));
}

/// Every sidebar destination compiles and mounts a real portable specimen and its usage text.
#[test]
fn dashboard_catalogue_mounts_every_component_and_its_usage() {
    let mut app = Harness::new();
    for example in crate::catalogue::EXAMPLES {
        app.click(example.name());
        app.wait(|node| has_text(node, example.usage()) && has_text(node, "Live preview"));
        assert!(has_text(&app.root, example.description()));
        let code = find(
            &app.root,
            &|node| matches!(&node.kind,NodeKind::Text(text) if text==example.usage()),
        )
        .unwrap();
        assert_eq!(code.style().font_family.as_deref(), Some("Noto Sans Mono"));
        assert_eq!(code.style().white_space, Some(inrux::WhiteSpace::Pre));
        assert!(
            find(&app.root, &|node| node.key.as_deref()
                == Some("catalogue-navigation"))
            .is_some()
        );
        assert!(has_text(&app.root, "Components"));
    }
}

/// Navigation disposes old edit handles while each new preview owns working model state.
#[test]
fn dashboard_previews_edit_reset_and_dispose_state_on_navigation() {
    let mut app = Harness::new();
    app.open("Button", "Live preview");
    app.click("Add to count");
    app.wait(|node| has_text(node, "Count: 1"));
    app.open("Input", "Workspace name");
    let stale = input(&app.root, "Inrux Workspace");
    stale.emit(1, "Dashboard workspace".into()).unwrap();
    app.wait(|node| has_text(node, "Value: Dashboard workspace"));
    app.open("Checkbox", "Notifications: disabled");
    let checkbox = find(&app.root, &|node| matches!(&node.kind, NodeKind::Checkbox { label, .. } if label == "Enable notifications")).unwrap();
    if let NodeKind::Checkbox { event, .. } = &checkbox.kind {
        event.emit(true).unwrap();
    }
    app.wait(|node| has_text(node, "Notifications: enabled"));
    assert!(stale.emit(2, "disposed".into()).is_err());
    app.open("Input", "Value: Inrux Workspace");
    app.open("TextArea", crate::catalogue::Example::TextArea.usage());
    let editor = find(&app.root, &|node| {
        matches!(
            node.kind,
            NodeKind::Input {
                multiline: true,
                ..
            }
        )
    })
    .unwrap();
    if let NodeKind::Input { event, .. } = &editor.kind {
        event.emit(1, "One\nTwo\nThree".into()).unwrap();
    }
    app.wait(|node| has_text(node, "13 characters · 3 lines"));
    app.click("Reset example");
    app.wait(|node| find(node, &|node| matches!(&node.kind, NodeKind::Input { value, multiline: true, .. } if value == "First line\nA native multiline editor.\n")).is_some());
}

/// Theme swaps and responsive width changes preserve the keyed specimen and its edited state.
#[test]
fn compiled_themes_and_viewport_rules_preserve_live_state() {
    let mut app = Harness::new();
    app.open("Button", "Live preview");
    app.click("Add to count");
    app.wait(|node| has_text(node, "Count: 1"));
    app.click("Dark theme");
    app.wait(|node| node.style().background == Some(inrux::Color(11, 11, 11)));
    assert!(has_text(&app.root, "Count: 1"));
    app.session.viewport().set(1400.0, 860.0);
    app.wait(|node| has_text(node, "On this page"));
    let scroll = find(&app.root, &|node| {
        node.classes.iter().any(|class| class == "content-scroll")
    })
    .unwrap();
    assert_eq!(scroll.style().padding, Some(48.0));
    app.session.viewport().set(900.0, 820.0);
    app.wait(|node| !has_text(node, "On this page"));
    let scroll = find(&app.root, &|node| {
        node.classes.iter().any(|class| class == "content-scroll")
    })
    .unwrap();
    assert_eq!(scroll.style().padding, Some(24.0));
    assert!(has_text(&app.root, "Count: 1"));
    app.click("Light theme");
    app.wait(|node| node.style().background == Some(inrux::Color(255, 255, 255)));
    assert!(has_text(&app.root, "Count: 1"));
}

/// The fixed rail sits outside scrolling flow, with the main viewport reaching the window edge.
#[test]
fn wide_document_column_and_rail_do_not_overlap() {
    let mut app = Harness::new();
    app.open("Button", "Live preview");
    app.session.viewport().set(1200.0, 860.0);
    app.wait(|node| has_text(node, "On this page"));
    let geometry = inrux_adapter::layout::compute(&app.root, 1200.0, Some(860.0), |request| {
        let width = (request.text.chars().count() as f32 * request.font_size * 0.55)
            .min(request.max_width.max(1.0));
        (width, request.font_size * 1.4)
    })
    .unwrap();
    let body = &geometry.children[1];
    assert!(
        body.children[1].width >= 700.0,
        "document collapsed: {}",
        body.children[1].width
    );
    assert_eq!(body.children.len(), 2);
    let sheet = &body.children[1];
    let scroll = &sheet.children[0];
    let rail = &sheet.children[1];
    assert_eq!(scroll.width, sheet.width);
    assert_eq!(rail.width, 160.0);
    assert!(rail.x + rail.width <= sheet.width);
    assert!(rail.x > 700.0);
    let request = find(&app.root, &|node| {
        node.classes.iter().any(|class| class == "content-scroll")
    })
    .unwrap()
    .scroll_request
    .unwrap();
    app.click("End of page");
    app.wait(|node| {
        find(node, &|node| {
            node.classes.iter().any(|class| class == "content-scroll")
        })
        .is_some_and(|node| {
            node.scroll_request
                .is_some_and(|next| next.revision > request.revision)
        })
    });
    let after =
        inrux_adapter::layout::compute(&app.root, 1200.0, Some(860.0), |_| (100.0, 20.0)).unwrap();
    assert_eq!(after.children[1].children[1].children[1].y, rail.y);
}

/// The scroll track spans the pane; only its scrolling content owns the navigation gutter.
#[test]
fn sidebar_scrollport_reaches_pane_edge_and_buttons_stay_clear_of_overlay_track() {
    let app = Harness::new();
    let layout =
        inrux_adapter::layout::compute(&app.root, 1200.0, Some(860.0), |_| (80.0, 20.0)).unwrap();
    let sidebar = &layout.children[1].children[0];
    let scroll = &sidebar.children[1];
    assert_eq!(scroll.x, 0.0);
    assert_eq!(scroll.width, sidebar.width);
    let items = &scroll.children[0];
    let list = &items.children[0];
    let button = &list.children[1];
    let right = items.x + list.x + button.x + button.width;
    assert!(right <= scroll.width - inrux_adapter::services::scroll::BAR_SIZE);
}

/// Real catalogue publications carry controlled values and inherited compiled accent styling.
#[test]
fn switch_and_ranges_update_through_the_application_coordinator() {
    let mut app = Harness::new();
    app.open("Switch", "Automatic sync: off");
    let switch = find(&app.root, &|node| {
        matches!(
            &node.kind,
            NodeKind::Checkbox {
                kind: inrux::ToggleKind::Switch,
                ..
            }
        )
    })
    .unwrap();
    if let NodeKind::Checkbox { event, .. } = &switch.kind {
        event.emit(true).unwrap();
    }
    app.wait(|node| has_text(node, "Automatic sync: on"));
    app.open("Progress", "Completion: 0%");
    app.click("Increase value");
    app.wait(|node| has_text(node, "Completion: 10%"));
    let accent = find(&app.root, &|node| matches!(&node.kind, NodeKind::Checkbox { label, .. } if label == "Use violet accent")).unwrap();
    if let NodeKind::Checkbox { event, .. } = &accent.kind {
        event.emit(true).unwrap();
    }
    app.wait(|node| find(node, &|node| matches!(&node.kind, NodeKind::Range(range) if range.kind == inrux::RangeKind::Progress && range.value == 10.0) && node.style().accent_color == Some(inrux::Color(112,91,234))).is_some());
    app.click("Reset example");
    app.wait(|node| has_text(node, "Completion: 0%"));
    app.open("Meter", "Temperature: -10 °C (−10 to 40)");
    app.click("Increase value");
    app.wait(|node| has_text(node, "Temperature: -5 °C (−10 to 40)"));
    let meter = find(&app.root, &|node| matches!(&node.kind, NodeKind::Range(range) if range.kind == inrux::RangeKind::Meter)).unwrap();
    if let NodeKind::Range(range) = &meter.kind {
        assert_eq!(range.min, -10.0);
        assert_eq!(range.max, 40.0);
        assert_eq!(range.fraction().unwrap(), 0.1);
    }
}

/// Application-owned recipes reach public parts and survive state/theme replacement on every engine.
#[test]
fn headless_recipes_resolve_in_all_catalogue_publications() {
    let mut app = Harness::new();
    app.open("Switch", "Automatic sync: off");
    let find_switch = |node: &Node| {
        matches!(
            node.kind,
            NodeKind::Checkbox {
                kind: inrux::ToggleKind::Switch,
                ..
            }
        )
    };
    let switch = find(&app.root, &find_switch).unwrap();
    assert_eq!(
        switch.part_style(inrux::Part::Track, false, false).width,
        Some(inrux::Size::Px(42.0))
    );
    assert_eq!(
        switch.part_style(inrux::Part::Thumb, false, false).width,
        Some(inrux::Size::Px(18.0))
    );
    assert_eq!(switch.style().margin_top, Some(4.0));
    if let NodeKind::Checkbox { event, .. } = &switch.kind {
        event.emit(true).unwrap();
    }
    app.wait(|node| has_text(node, "Automatic sync: on"));
    let before = find(&app.root, &find_switch)
        .unwrap()
        .part_style(inrux::Part::Track, false, false)
        .background;
    app.click("Dark theme");
    app.wait(|node| {
        find(
            node,
            &|node| matches!(&node.kind,NodeKind::Button(label,_) if label=="Light theme"),
        )
        .is_some()
    });
    let switch = find(&app.root, &find_switch).unwrap();
    assert!(has_text(&app.root, "Automatic sync: on"));
    assert_ne!(
        switch
            .part_style(inrux::Part::Track, false, false)
            .background,
        before
    );
    assert!(switch.supports_part(inrux::Part::Track));
}
