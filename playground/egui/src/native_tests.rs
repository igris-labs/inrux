//! Drive actual egui widgets around the compiled catalogue, not only queued event handles.
use crate::tests::{Harness, find, has_text};
use eframe::egui::{self, Event, Modifiers, PointerButton};
use inrux::NodeKind;

struct Native {
    app: Harness,
    context: egui::Context,
}

/// Native hit testing, editing and button events continue through the shared Card composition.
#[test]
fn native_card_children_edit_and_save_at_compact_size() {
    let mut app = Native::new();
    app.click("Content");
    app.app
        .wait(|node| has_text(node, "Card, Badge and Separator"));
    app.click("Inrux Workspace");
    app.frame(vec![Event::Text(" edited".into())]);
    app.app.wait(|node| has_text(node, "Unsaved changes"));
    app.click("Save card");
    app.app
        .wait(|node| has_text(node, "Saved") && has_text(node, "Saves: 1"));
    let output = app.frame(vec![]);
    assert!(output.shapes.iter().any(|shape| matches!(&shape.shape,egui::epaint::Shape::Text(text) if text.galley.text().starts_with("Noto Sans ·") && text.pos.y + text.galley.size().y <= 820.0)));
}
impl Native {
    /// Register the same bundled face; platform metric calibration is tested by the adapter itself.
    fn new() -> Self {
        let context = egui::Context::default();
        inrux_egui::configure_fonts(
            &context,
            &[
                inrux::FontAsset::new(
                    "Noto Sans",
                    include_bytes!("assets/fonts/NotoSans-Regular.ttf").as_slice(),
                ),
                inrux::FontAsset::new(
                    "Noto Sans Mono",
                    include_bytes!("assets/fonts/NotoSansMono.ttf").as_slice(),
                ),
            ],
        )
        .unwrap();
        Self {
            app: Harness::new(),
            context,
        }
    }
    /// Render at the actual minimum window size with native text measurement and input handling.
    fn frame(&self, events: Vec<Event>) -> egui::FullOutput {
        self.context.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(900.0, 820.0),
                )),
                events,
                ..Default::default()
            },
            |context| {
                egui::CentralPanel::default().show(context, |ui| {
                    assert!(
                        inrux_egui::render_ref(&self.app.root, ui)
                            .unwrap()
                            .is_none()
                    );
                });
            },
        )
    }
    /// Activate by painted label position so real widget admission, focus and identity participate.
    fn click(&self, label: &str) {
        fn locate(shape: &egui::epaint::Shape, label: &str) -> Option<egui::Pos2> {
            match shape {
                egui::epaint::Shape::Text(text) if text.galley.text() == label => {
                    Some(text.pos + text.galley.size() * 0.5)
                }
                egui::epaint::Shape::Vec(shapes) => {
                    shapes.iter().find_map(|shape| locate(shape, label))
                }
                _ => None,
            }
        }
        let point_in = |output: &egui::FullOutput| {
            output.shapes.iter().find_map(|shape| {
                locate(&shape.shape, label).filter(|point| shape.clip_rect.contains(*point))
            })
        };
        let mut point = point_in(&self.frame(vec![]));
        if point.is_none() {
            // The long sidebar is a real viewport; hidden labels must be revealed before clicking.
            let hover = egui::pos2(110.0, 350.0);
            for delta in [2000.0, -180.0, -180.0, -180.0, -180.0, -180.0, -180.0] {
                self.frame(vec![
                    Event::PointerMoved(hover),
                    Event::MouseWheel {
                        unit: egui::MouseWheelUnit::Point,
                        delta: egui::vec2(0.0, delta),
                        modifiers: Modifiers::NONE,
                    },
                ]);
                for _ in 0..12 {
                    self.frame(vec![]);
                }
                point = point_in(&self.frame(vec![]));
                if point.is_some() {
                    break;
                }
            }
        }
        let point = point.unwrap_or_else(|| panic!("missing visible widget {label}"));
        for pressed in [true, false] {
            self.frame(vec![
                Event::PointerMoved(point),
                Event::PointerButton {
                    pos: point,
                    button: PointerButton::Primary,
                    pressed,
                    modifiers: Modifiers::NONE,
                },
            ]);
        }
    }
}

#[test]
fn native_input_save_reset_and_disabled_button_work_in_the_new_view() {
    let mut app = Native::new();
    app.click("Controls");
    app.app
        .wait(|node| has_text(node, "Input models and Button states"));
    app.click("Inrux Workspace");
    app.frame(vec![Event::Text(" edited".into())]);
    app.app
        .wait(|node| has_text(node, "You have unsaved changes."));
    app.click("Save changes");
    app.app
        .wait(|node| has_text(node, "All changes saved in memory."));
    app.click("Disabled button");
    app.click("Reset name");
    app.app.wait(|node| {
        find(
            node,
            &|node| matches!(&node.kind,NodeKind::Input { value,.. } if value=="Inrux Workspace"),
        )
        .is_some()
    });
    assert!(has_text(
        &app.app.root,
        "Notifications: false | Choice: true | Disabled activations: 0"
    ));
}

#[test]
fn native_keyed_editor_keeps_focus_and_text_after_reordering() {
    let mut app = Native::new();
    app.app
        .open("Composition", "Props, emitted events and computed state");
    app.click("This edit follows its keyed row");
    app.frame(vec![Event::Text("owned".into())]);
    app.app.wait(|node| {
        find(
            node,
            &|node| matches!(&node.kind,NodeKind::Input { value,.. } if value=="owned"),
        )
        .is_some()
    });
    // Dispatch reorder without clicking elsewhere: that would intentionally transfer focus.
    app.app.click("Reverse rows");
    app.app.wait(|node| {
        crate::tests::row_order(node)
            .first()
            .is_some_and(|label| label == "Row 2: 0")
    });
    app.frame(vec![]);
    app.frame(vec![Event::Text("-more".into())]);
    app.app.wait(|node| {
        find(
            node,
            &|node| matches!(&node.kind,NodeKind::Input { value,.. } if value=="owned-more"),
        )
        .is_some()
    });
    let row = crate::tests::row(&app.app.root, "Row 1: 0");
    assert!(
        find(
            row,
            &|node| matches!(&node.kind,NodeKind::Input { value,.. } if value=="owned-more")
        )
        .is_some()
    );
}

#[test]
fn native_modal_close_and_custom_payload_dispatch_in_the_showcase() {
    let mut app = Native::new();
    app.app
        .open("Composition", "Props, emitted events and computed state");
    app.click("Confirm name");
    app.app
        .wait(|node| has_text(node, "Confirmed: Inrux Workspace"));
    app.app.open("Engine", "Engine support");
    app.click("Balanced");
    app.frame(vec![]);
    app.click("Performance");
    app.app.wait(|node| has_text(node, "Selected: Performance"));
    app.click("Disable Select");
    app.app.wait(|node| {
        find(node, &|node| {
            matches!(node.kind, NodeKind::Select { .. }) && node.disabled
        })
        .is_some()
    });
    app.click("Show Heartbeat");
    app.app
        .wait(|node| find(node, &|node| matches!(node.kind, NodeKind::Heartbeat)).is_none());
    app.click("Open dialog");
    app.app.wait(|node| {
        find(node, &|node| {
            matches!(node.kind, NodeKind::Dialog { open: true, .. })
        })
        .is_some()
    });
    app.frame(vec![]);
    app.click("Close dialog");
    app.app.wait(|node| {
        find(node, &|node| {
            matches!(node.kind, NodeKind::Dialog { open: false, .. })
        })
        .is_some()
    });
}

/// Native editor identity and focus survive an axis change without a layout-specific widget.
#[test]
fn native_input_focus_survives_flex_direction_change() {
    let mut app = Native::new();
    app.app
        .open("More layout", "Flex direction and retained input");
    app.click("This edit survives direction changes");
    app.frame(vec![Event::Text("owned".into())]);
    app.app.wait(|node| {
        find(
            node,
            &|node| matches!(&node.kind,NodeKind::Input { value,.. } if value=="owned"),
        )
        .is_some()
    });
    // Queue the layout change without moving pointer focus to a different button.
    app.app.click("Use column direction");
    app.app.wait(|node| {
        find(node, &|node| {
            node.style().flex_direction == Some(inrux::FlexDirection::Column)
        })
        .is_some()
    });
    app.frame(vec![]);
    app.frame(vec![Event::Text("-more".into())]);
    app.app.wait(|node| {
        find(
            node,
            &|node| matches!(&node.kind,NodeKind::Input { value,.. } if value=="owned-more"),
        )
        .is_some()
    });
}

/// Native external commands remain clickable when offscreen child geometry crosses their bounds.
#[test]
fn native_scroll_commands_resize_without_activating_hidden_rows() {
    let mut app = Native::new();
    app.click("Scrolling");
    app.app
        .wait(|node| has_text(node, "ScrollArea and clipped viewports"));
    app.frame(vec![]);
    app.click("Scroll to end");
    app.app.wait(|node| {
        find(node, &|node| node.key.as_deref() == Some("outer-viewport"))
            .unwrap()
            .scroll_request
            .unwrap()
            .revision
            == 1
    });
    app.frame(vec![]);
    app.app.wait(|node|find(node,&|node|matches!(&node.kind,NodeKind::Text(text) if text.starts_with("Offset: ") && !text.starts_with("Offset: 0 /"))).is_some());
    app.click("Scroll to top");
    app.app.wait(|node| {
        find(node, &|node| node.key.as_deref() == Some("outer-viewport"))
            .unwrap()
            .scroll_request
            .unwrap()
            .revision
            == 2
    });
    app.frame(vec![]);
    app.app.wait(|node| {
        find(
            node,
            &|node| matches!(&node.kind,NodeKind::Text(text) if text.starts_with("Offset: 0 /")),
        )
        .is_some()
    });
    app.click("Resize viewport");
    app.app.wait(|node| {
        find(node, &|node| node.key.as_deref() == Some("outer-viewport"))
            .unwrap()
            .style()
            .height
            == Some(inrux::Size::Px(240.0))
    });
    assert!(has_text(&app.app.root, "Actions: 0"));
}

/// Sidebar selection and a specimen's click use native hit testing inside independent viewports.
#[test]
fn native_dashboard_sidebar_and_component_action_are_clickable() {
    let mut app = Native::new();
    app.click("Button");
    app.app.wait(|node| has_text(node, "Live preview"));
    app.click("Add to count");
    app.app.wait(|node| has_text(node, "Count: 1"));
    app.click("Input");
    app.app
        .wait(|node| has_text(node, "Value: Inrux Workspace"));
    app.click("Inrux Workspace");
    app.frame(vec![Event::Text(" edited".into())]);
    app.app
        .wait(|node| find(node, &|node| matches!(&node.kind, NodeKind::Input { value, .. } if value.contains(" edited"))).is_some());
}

/// Center must lay out its intrinsic, padded text child inside the actual dashboard viewport.
#[test]
fn native_center_preview_renders_and_resizes_its_centered_child() {
    let mut app = Native::new();
    app.app
        .open("Center", crate::catalogue::Example::Center.usage());
    app.frame(vec![]);
    let control = find(&app.app.root, &|node| matches!(&node.kind, NodeKind::Checkbox { label, .. } if label == "Use alternate layout")).unwrap();
    if let NodeKind::Checkbox { event, .. } = &control.kind {
        event.emit(true).unwrap();
    }
    app.app
        .wait(|node| has_text(node, "Alternate: true · Child actions: 0"));
    app.frame(vec![]);
}

/// The embedded code face has equal advances, rather than falling back to the body font.
#[test]
fn native_code_font_uses_monospace_advances() {
    let app = Native::new();
    let node = inrux::Node::column(vec![
        inrux::Node::text("iiii".into()),
        inrux::Node::text("WWWW".into()),
    ])
    .styled(
        inrux::Style::new()
            .font_family("Noto Sans Mono")
            .font_size(13.0),
    );
    let output = app.context.run(egui::RawInput::default(), |context| {
        egui::CentralPanel::default().show(context, |ui| {
            inrux_egui::render(node.clone(), ui).unwrap();
        });
    });
    let width = |value: &str| {
        output
            .shapes
            .iter()
            .find_map(|shape| match &shape.shape {
                egui::Shape::Text(text) if text.galley.text() == value => {
                    Some(text.galley.size().x)
                }
                _ => None,
            })
            .unwrap()
    };
    assert!((width("iiii") - width("WWWW")).abs() < 0.01);
}

/// Range pages use geometry-only leaves and survive real font measurement at compact width.
#[test]
fn native_switch_and_range_catalogue_pages_render_and_update() {
    let mut app = Native::new();
    app.app.open("Switch", "Automatic sync: off");
    app.click("Automatic sync");
    app.app.wait(|node| has_text(node, "Automatic sync: on"));
    for (page, initial, updated) in [
        ("Progress", "Completion: 0%", "Completion: 10%"),
        (
            "Meter",
            "Temperature: -10 °C (−10 to 40)",
            "Temperature: -5 °C (−10 to 40)",
        ),
    ] {
        app.app.open(page, initial);
        app.click("Increase value");
        app.app.wait(|node| has_text(node, updated));
        app.click("Use violet accent");
        app.app.wait(|node| {
            find(node, &|node| {
                matches!(node.kind, NodeKind::Range(_))
                    && node.style().accent_color == Some(inrux::Color(112, 91, 234))
            })
            .is_some()
        });
        app.frame(vec![]);
    }
}
