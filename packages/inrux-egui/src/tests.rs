//! Layout and native widget integration tests.
use eframe::egui;
use inrux_components::Node;
use inrux_core::Emit;
use inrux_style::{Size, Style};

use crate::render;

#[test]
fn taffy_applies_padding_gap_and_row_layout() {
    let node = Node::column(vec![
        Node::text("A".into()).styled(Style::new().height(Size::Px(20.0))),
        Node::text("B".into()).styled(Style::new().height(Size::Px(20.0))),
    ])
    .styled(Style::new().padding(24.0).gap(16.0).width(Size::Px(300.0)));
    let root = inrux_adapter::layout::compute(&node, 300.0, None, |_| (0.0, 0.0)).unwrap();
    assert_eq!(root.height, 104.0);
    assert_eq!(root.children[1].y, 60.0);
}
#[test]
fn headless_frame_renders_all_widget_types() {
    let context = egui::Context::default();
    let mut called = false;
    let _ = context.run(egui::RawInput::default(), |context| {
        egui::CentralPanel::default().show(context, |ui| {
            let event = Emit::new(|_: String| Ok(()));
            let node = Node::column(vec![
                Node::text("Hello".into()),
                Node::input("Rust".into(), "Name".into(), event),
                Node::row(vec![Node::button("Save".into(), None)]),
                Node::heartbeat(),
                Node::empty(),
            ])
            .styled(Style::new().gap(12.0));
            assert!(render(node, ui).unwrap().is_none());
            called = true;
        });
    });
    assert!(called);
}

/// A supported border-width declaration produces visible paint without requiring an explicit border color.
#[test]
fn border_width_alone_paints_using_the_current_text_color() {
    let context = egui::Context::default();
    let output = context.run(egui::RawInput::default(), |context| {
        egui::CentralPanel::default().show(context, |ui| {
            let node = Node::column(vec![]).styled(
                Style::new()
                    .width(Size::Px(100.0))
                    .height(Size::Px(50.0))
                    .border_width(2.0)
                    .color(inrux_style::Color(10, 20, 30)),
            );
            assert!(render(node, ui).unwrap().is_none());
        });
    });
    assert!(output.shapes.iter().any(|shape| matches!(&shape.shape, egui::epaint::Shape::Rect(rect) if rect.stroke.width == 2.0 && rect.stroke.color == egui::Color32::from_rgb(10,20,30))));
}

/// A focused editor must stop admitting text as soon as its controlled disabled flag changes.
#[test]
fn disabling_a_focused_editor_rejects_text_and_accepts_external_reset() {
    let context = egui::Context::default();
    let (sent, received) = std::sync::mpsc::channel();
    let callback = Emit::new(move |value| {
        sent.send(value).unwrap();
        Ok(())
    });
    let mut node = Node::input("original".into(), "Name".into(), callback)
        .styled(Style::new().width(Size::Px(240.0)).height(Size::Px(44.0)));
    let frame = |node: &Node, events: Vec<egui::Event>| {
        context.run(
            egui::RawInput {
                events,
                ..Default::default()
            },
            |context| {
                egui::CentralPanel::default().show(context, |ui| {
                    render(node.clone(), ui).unwrap();
                });
            },
        )
    };
    let _ = frame(&node, vec![]);
    for pressed in [true, false] {
        let _ = frame(
            &node,
            vec![
                egui::Event::PointerMoved(egui::pos2(30.0, 25.0)),
                egui::Event::PointerButton {
                    pos: egui::pos2(30.0, 25.0),
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: Default::default(),
                },
            ],
        );
    }
    assert!(context.memory(|memory| memory.focused().is_some()));
    node.disabled = true;
    let _ = frame(&node, vec![egui::Event::Text("blocked".into())]);
    assert!(
        received.try_recv().is_err(),
        "disabled editor emitted a model change"
    );
    assert!(context.memory(|memory| memory.focused().is_none()));
    if let inrux_components::NodeKind::Input { value, .. } = &mut node.kind {
        *value = "reset".into();
    }
    let output = frame(&node, vec![]);
    assert!(output.shapes.iter().any(|shape| matches!(&shape.shape, egui::epaint::Shape::Text(text) if text.galley.text() == "reset")));
}

/// Even fixed-size text that skips intrinsic measurement must diagnose an unknown font before paint.
#[test]
fn unknown_font_returns_a_diagnostic_before_native_layout_or_paint() {
    let context = egui::Context::default();
    let _ = context.run(egui::RawInput::default(), |context| {
        egui::CentralPanel::default().show(context, |ui| {
            let node = Node::text("Hello".into()).styled(
                Style::new()
                    .font_family("UnregisteredFamily")
                    .width(Size::Px(100.0))
                    .height(Size::Px(40.0)),
            );
            let error = render(node, ui).unwrap_err();
            assert!(matches!(
                error,
                inrux_adapter::AdapterError::Unsupported {
                    backend: "egui",
                    ..
                }
            ));
            assert!(error.to_string().contains("WindowOptions.fonts"));
        });
    });
}

/// Authored button fills still show hover, while disabled buttons keep their base paint.
#[test]
fn button_hover_changes_paint_without_recoloring_disabled_controls() {
    use inrux_style::Color;
    let context = egui::Context::default();
    let frame = |disabled, position| {
        context.run(
            egui::RawInput {
                events: vec![egui::Event::PointerMoved(position)],
                ..Default::default()
            },
            |context| {
                egui::CentralPanel::default().show(context, |ui| {
                    render(
                        Node::button("Save".into(), None).disabled(disabled).styled(
                            Style::new()
                                .background(Color(29, 112, 98))
                                .border_width(0.0)
                                .width(Size::Px(120.0)),
                        ),
                        ui,
                    )
                    .unwrap();
                });
            },
        )
    };
    let hovered = inrux_adapter::controls::hovered_button_color(Color(29, 112, 98));
    let ink = egui::Color32::from_rgba_unmultiplied(hovered.0, hovered.1, hovered.2, hovered.3);
    frame(false, egui::pos2(300.0, 300.0));
    frame(false, egui::pos2(30.0, 20.0));
    let output = frame(false, egui::pos2(30.0, 20.0));
    assert!(
        output.shapes.iter().any(
            |shape| matches!(&shape.shape, egui::epaint::Shape::Rect(rect) if rect.fill == ink)
        )
    );
    let disabled = frame(true, egui::pos2(30.0, 20.0));
    assert!(
        !disabled.shapes.iter().any(
            |shape| matches!(&shape.shape, egui::epaint::Shape::Rect(rect) if rect.fill == ink)
        )
    );
}

/// Keyboard focus remains visible on the square; pointer selection does not leave an outline.
#[test]
fn checkbox_focus_paints_only_a_compact_square_indicator() {
    let context = egui::Context::default();
    let frame = |events| {
        context.run(
            egui::RawInput {
                events,
                ..Default::default()
            },
            |context| {
                egui::CentralPanel::default().show(context, |ui| {
                    render(
                        Node::checkbox("Notifications".into(), false, Emit::new(|_| Ok(())))
                            .styled(Style::new().width(Size::Px(400.0))),
                        ui,
                    )
                    .unwrap();
                });
            },
        )
    };
    frame(vec![]);
    for pressed in [true, false] {
        frame(vec![
            egui::Event::PointerMoved(egui::pos2(30.0, 20.0)),
            egui::Event::PointerButton {
                pos: egui::pos2(30.0, 20.0),
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::NONE,
            },
        ]);
    }
    assert!(!crate::focus::visible(&context));
    let output = frame(vec![egui::Event::Key {
        key: egui::Key::Space,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::NONE,
    }]);
    let accent = inrux_adapter::controls::ACCENT;
    let rings: Vec<_> = output
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::epaint::Shape::Rect(rect)
                if rect.stroke.color
                    == egui::Color32::from_rgba_unmultiplied(
                        accent.0, accent.1, accent.2, accent.3,
                    ) =>
            {
                Some(rect)
            }
            _ => None,
        })
        .collect();
    assert!(!rings.is_empty());
    assert!(rings.iter().all(|rect| rect.rect.width() <= 18.0));
}

/// Raw native wheel events are normalized once, and clipped controls cannot activate.
#[test]
fn viewport_wheel_clips_and_reveals_native_button() {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    let calls = Arc::new(AtomicUsize::new(0));
    let count = calls.clone();
    let button = Node::button(
        "Last".into(),
        Some(Emit::new(move |_| {
            count.fetch_add(1, Ordering::Relaxed);
            Ok(())
        })),
    )
    .styled(Style::new().height(Size::Px(40.0)));
    let node = Node::scroll_area(vec![
        Node::box_container(vec![]).styled(Style::new().height(Size::Px(280.0))),
        button,
    ])
    .styled(Style::new().width(Size::Px(200.0)).height(Size::Px(100.0)));
    let context = egui::Context::default();
    let frame = |events| {
        context.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(400.0, 400.0),
                )),
                events,
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    assert!(render(node.clone(), ui).unwrap().is_none());
                });
            },
        )
    };
    frame(vec![]);
    let click = |point: egui::Pos2| {
        frame(vec![
            egui::Event::PointerMoved(point),
            egui::Event::PointerButton {
                pos: point,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ]);
        frame(vec![egui::Event::PointerButton {
            pos: point,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }]);
    };
    click(egui::pos2(30.0, 300.0));
    assert_eq!(calls.load(Ordering::Relaxed), 0);
    frame(vec![
        egui::Event::PointerMoved(egui::pos2(30.0, 50.0)),
        egui::Event::MouseWheel {
            unit: egui::MouseWheelUnit::Point,
            delta: egui::vec2(0.0, -1000.0),
            modifiers: egui::Modifiers::NONE,
        },
    ]);
    frame(vec![]);
    click(egui::pos2(30.0, 85.0));
    assert_eq!(calls.load(Ordering::Relaxed), 1);
}

/// Real nested egui hit regions route wheel input once; a blocked child bubbles to the parent.
#[test]
fn nested_viewports_route_wheel_to_the_deepest_movable_target() {
    let (sent, received) = std::sync::mpsc::channel();
    let inner_sent = sent.clone();
    let content = || Node::box_container(vec![]).styled(Style::new().height(Size::Px(400.0)));
    let inner = Node::scroll_area(vec![content()])
        .styled(Style::new().height(Size::Px(100.0)))
        .on_scroll(Emit::new(
            move |metrics: inrux_components::ScrollMetrics| {
                inner_sent.send((true, metrics.offset.y)).unwrap();
                Ok(())
            },
        ));
    let node = Node::scroll_area(vec![inner, content()])
        .styled(Style::new().width(Size::Px(200.0)).height(Size::Px(140.0)))
        .on_scroll(Emit::new(
            move |metrics: inrux_components::ScrollMetrics| {
                sent.send((false, metrics.offset.y)).unwrap();
                Ok(())
            },
        ));
    let context = egui::Context::default();
    let frame = |events| {
        let _ = context.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(400.0, 400.0),
                )),
                events,
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    render(node.clone(), ui).unwrap();
                });
            },
        );
    };
    frame(vec![]);
    received.try_iter().for_each(drop);
    let wheel = |y| {
        vec![
            egui::Event::PointerMoved(egui::pos2(30.0, 40.0)),
            egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: egui::vec2(0.0, y),
                modifiers: egui::Modifiers::NONE,
            },
        ]
    };
    // Fine trackpad samples bypass native smoothing and must never move both viewports.
    for _ in 0..60 {
        frame(wheel(-5.0));
    }
    frame(vec![]);
    let observations = received.try_iter().collect::<Vec<_>>();
    assert!(observations.iter().all(|(inner, _)| *inner));
    assert_eq!(observations.last(), Some(&(true, 300.0)));
    frame(wheel(-5.0));
    frame(vec![]);
    assert_eq!(received.try_iter().collect::<Vec<_>>(), vec![(false, 5.0)]);
}

/// Offscreen native focus reveals once; keyboard and captured scrollbar dragging share offsets.
#[test]
fn viewport_focus_keyboard_and_drag_use_the_same_state() {
    let (sent, received) = std::sync::mpsc::channel();
    let node = Node::scroll_area(vec![
        Node::box_container(vec![]).styled(Style::new().height(Size::Px(280.0))),
        Node::input("kept".into(), "".into(), Emit::new(|_| Ok(())))
            .styled(Style::new().height(Size::Px(40.0))),
    ])
    .styled(Style::new().width(Size::Px(200.0)).height(Size::Px(100.0)))
    .on_scroll(Emit::new(
        move |metrics: inrux_components::ScrollMetrics| {
            sent.send(metrics.offset.y).unwrap();
            Ok(())
        },
    ));
    let context = egui::Context::default();
    let frame = |events, focus| {
        let _ = context.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(400.0, 400.0),
                )),
                events,
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    let root = ui.id().with(("inrux-root", &node.key, &node.generation));
                    if focus == 1 {
                        ui.memory_mut(|m| m.request_focus(root.with(("index", 1_usize))));
                    }
                    if focus == 2 {
                        ui.memory_mut(|m| m.request_focus(root.with("viewport-focus")));
                    }
                    render(node.clone(), ui).unwrap();
                });
            },
        );
    };
    frame(vec![], 0);
    received.try_iter().for_each(drop);
    frame(vec![], 1);
    frame(vec![], 0);
    assert_eq!(received.try_iter().last(), Some(220.0));
    frame(
        vec![
            egui::Event::PointerMoved(egui::pos2(40.0, 50.0)),
            egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: egui::vec2(0.0, 40.0),
                modifiers: egui::Modifiers::NONE,
            },
        ],
        0,
    );
    for _ in 0..90 {
        frame(vec![], 0);
    }
    assert!((received.try_iter().last().unwrap() - 180.0).abs() < 0.1);
    frame(
        vec![egui::Event::Key {
            key: egui::Key::Home,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        }],
        2,
    );
    frame(vec![], 0);
    assert_eq!(received.try_iter().last(), Some(0.0));
    let pointer = |point, pressed| egui::Event::PointerButton {
        pos: point,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: egui::Modifiers::NONE,
    };
    frame(
        vec![
            egui::Event::PointerMoved(egui::pos2(202.0, 15.0)),
            pointer(egui::pos2(202.0, 15.0), true),
        ],
        0,
    );
    frame(vec![egui::Event::PointerMoved(egui::pos2(350.0, 300.0))], 0);
    frame(vec![pointer(egui::pos2(350.0, 300.0), false)], 0);
    assert_eq!(received.try_iter().last(), Some(220.0));
}

/// Long single-line edits must translate the galley while retaining the authored control width.
#[test]
fn long_input_reveals_caret_inside_fixed_width() {
    let context = egui::Context::default();
    let node = Node::input(String::new(), "Name".into(), Emit::new(|_| Ok(())))
        .styled(Style::new().width(Size::Px(140.0)).height(Size::Px(40.0)));
    let frame = |events| {
        context.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(400.0, 150.0),
                )),
                events,
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    render(node.clone(), ui).unwrap();
                });
            },
        )
    };
    frame(vec![]);
    let pos = egui::pos2(30.0, 25.0);
    frame(vec![
        egui::Event::PointerMoved(pos),
        egui::Event::PointerButton {
            pos,
            pressed: true,
            button: egui::PointerButton::Primary,
            modifiers: egui::Modifiers::NONE,
        },
    ]);
    frame(vec![egui::Event::PointerButton {
        pos,
        pressed: false,
        button: egui::PointerButton::Primary,
        modifiers: egui::Modifiers::NONE,
    }]);
    let text = "a long workspace name with a final visible suffix";
    frame(vec![egui::Event::Text(text.into())]);
    let output = frame(vec![]);
    let text_shape = output
        .shapes
        .iter()
        .find_map(|shape| match &shape.shape {
            egui::epaint::Shape::Text(t) if t.galley.text() == text => Some((shape.clip_rect, t)),
            _ => None,
        })
        .unwrap();
    assert!(
        text_shape.1.pos.x < 20.0,
        "text origin {:?}, clip {:?}, width {}",
        text_shape.1.pos,
        text_shape.0,
        text_shape.1.galley.size().x
    );
    assert!(
        text_shape.1.pos.x + text_shape.1.galley.size().x <= 150.0,
        "caret end outside field: {:?}",
        text_shape.1.pos
    );
    let origin = text_shape.1.pos.x;
    let text_x = |output: &egui::FullOutput| {
        output
            .shapes
            .iter()
            .find_map(|shape| match &shape.shape {
                egui::epaint::Shape::Text(t) if t.galley.text() == text => Some(t.pos.x),
                _ => None,
            })
            .unwrap()
    };
    for _ in 0..8 {
        frame(vec![
            egui::Event::PointerMoved(pos),
            egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: egui::vec2(5.0, 0.0),
                modifiers: egui::Modifiers::NONE,
            },
        ]);
    }
    assert!(
        text_x(&frame(vec![])) > origin + 30.0,
        "manual horizontal scroll did not persist"
    );
    frame(vec![egui::Event::Key {
        key: egui::Key::Home,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::NONE,
    }]);
    assert!(
        text_x(&frame(vec![])) >= 19.0,
        "Home did not reveal text start: x {}, focus {:?}, cursor {:?}",
        text_x(&frame(vec![])),
        context.memory(|m| m.focused()),
        context
            .memory(|m| m.focused())
            .and_then(|id| egui::TextEdit::load_state(&context, id))
            .and_then(|s| s.cursor.char_range())
    );
    for _ in 0..8 {
        frame(vec![egui::Event::MouseWheel {
            unit: egui::MouseWheelUnit::Point,
            delta: egui::vec2(-5.0, 0.0),
            modifiers: egui::Modifiers::NONE,
        }]);
    }
    assert!(text_x(&frame(vec![])) < 19.0);
    frame(vec![egui::Event::Key {
        key: egui::Key::Home,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::NONE,
    }]);
    assert!(
        text_x(&frame(vec![])) >= 19.0,
        "repeated Home should reveal a manually hidden stationary caret"
    );
}

/// Use egui's native coarse-wheel interpolation while fine trackpad samples stay immediate.
#[test]
fn coarse_wheel_is_smoothed_and_settles_without_losing_distance() {
    let (sent, received) = std::sync::mpsc::channel();
    let node = Node::scroll_area(vec![
        Node::box_container(vec![]).styled(Style::new().height(Size::Px(700.0))),
    ])
    .styled(Style::new().height(Size::Px(120.0)))
    .on_scroll(Emit::new(
        move |metrics: inrux_components::ScrollMetrics| {
            sent.send(metrics.offset.y).unwrap();
            Ok(())
        },
    ));
    let context = egui::Context::default();
    let frame = |events| {
        let _ = context.run(
            egui::RawInput {
                events,
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    render(node.clone(), ui).unwrap();
                });
            },
        );
    };
    frame(vec![]);
    received.try_iter().for_each(drop);
    frame(vec![
        egui::Event::PointerMoved(egui::pos2(30.0, 40.0)),
        egui::Event::MouseWheel {
            unit: egui::MouseWheelUnit::Point,
            delta: egui::vec2(0.0, -120.0),
            modifiers: egui::Modifiers::NONE,
        },
    ]);
    let first = received.try_iter().last().unwrap();
    assert!(first > 0.0 && first < 120.0);
    for _ in 0..90 {
        frame(vec![]);
    }
    assert!((received.try_iter().last().unwrap() - 120.0).abs() < 0.1);
    frame(vec![egui::Event::MouseWheel {
        unit: egui::MouseWheelUnit::Point,
        delta: egui::vec2(0.0, -3.0),
        modifiers: egui::Modifiers::NONE,
    }]);
    assert!((received.try_iter().last().unwrap() - 123.0).abs() < 0.1);
}

/// Paste must never create hidden extra rows in Input; TextArea wraps and accepts Enter.
#[test]
fn native_editing_modes_handle_paste_enter_and_wrapping() {
    for multiline in [false, true] {
        let context = egui::Context::default();
        let (sent, received) = std::sync::mpsc::channel();
        let event = Emit::new(move |value| {
            sent.send(value).unwrap();
            Ok(())
        });
        let node = if multiline {
            Node::text_area(String::new(), "Notes".into(), event)
        } else {
            Node::input(String::new(), "Name".into(), event)
        }
        .styled(Style::new().width(Size::Px(180.0)).height(Size::Px(100.0)));
        let frame = |events| {
            context.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(400.0, 200.0),
                    )),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        render(node.clone(), ui).unwrap();
                    });
                },
            )
        };
        frame(vec![]);
        let pos = egui::pos2(30.0, 25.0);
        for pressed in [true, false] {
            frame(vec![
                egui::Event::PointerMoved(pos),
                egui::Event::PointerButton {
                    pos,
                    pressed,
                    button: egui::PointerButton::Primary,
                    modifiers: egui::Modifiers::NONE,
                },
            ]);
        }
        frame(vec![egui::Event::Paste(
            "first\r\nsecond\u{2028}third".into(),
        )]);
        assert_eq!(
            received.try_iter().last().unwrap(),
            if multiline {
                "first\nsecond\nthird"
            } else {
                "first second third"
            }
        );
        frame(vec![egui::Event::Key {
            key: egui::Key::Enter,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        }]);
        let edits: Vec<_> = received.try_iter().collect();
        assert_eq!(!edits.is_empty(), multiline);
        if multiline {
            assert!(edits.last().unwrap().contains('\n'));
        }
    }
}

/// Pointer activation retains Enter/Space focus without drawing a focus-visible ring.
#[test]
fn button_pointer_activation_has_no_outline_and_keyboard_activation_restores_it() {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    let count = Arc::new(AtomicUsize::new(0));
    let observed = count.clone();
    let node = Node::button(
        "Save".into(),
        Some(Emit::new(move |_| {
            observed.fetch_add(1, Ordering::SeqCst);
            Ok(())
        })),
    )
    .styled(
        Style::new()
            .width(Size::Px(120.0))
            .height(Size::Px(40.0))
            .border_width(0.0),
    );
    let context = egui::Context::default();
    let frame = |events| {
        context.run(
            egui::RawInput {
                events,
                ..Default::default()
            },
            |context| {
                egui::CentralPanel::default().show(context, |ui| {
                    render(node.clone(), ui).unwrap();
                });
            },
        )
    };
    frame(vec![]);
    for pressed in [true, false] {
        frame(vec![
            egui::Event::PointerMoved(egui::pos2(20.0, 20.0)),
            egui::Event::PointerButton {
                pos: egui::pos2(20.0, 20.0),
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::NONE,
            },
        ]);
    }
    let accent = inrux_adapter::controls::ACCENT;
    let has_ring = |output: &egui::FullOutput| {
        output.shapes.iter().any(|shape| matches!(&shape.shape,egui::epaint::Shape::Rect(rect) if rect.stroke.color==egui::Color32::from_rgba_unmultiplied(accent.0,accent.1,accent.2,accent.3) && rect.stroke.width>0.0))
    };
    assert_eq!(count.load(Ordering::SeqCst), 1);
    assert!(!has_ring(&frame(vec![])));
    let keyboard = frame(vec![egui::Event::Key {
        key: egui::Key::Enter,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::NONE,
    }]);
    assert_eq!(count.load(Ordering::SeqCst), 2);
    assert!(has_ring(&keyboard));
}

/// Native switch hit testing and Space use the same controlled boolean event and disabled gate.
#[test]
fn switch_pointer_keyboard_and_disabled_state_work_with_semantic_accessibility() {
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };
    let checked = Arc::new(AtomicBool::new(false));
    let disabled = Arc::new(AtomicBool::new(false));
    let state = checked.clone();
    let event = Emit::new(move |value| {
        state.store(value, Ordering::SeqCst);
        Ok(())
    });
    let context = egui::Context::default();
    context.enable_accesskit();
    let frame = |events| {
        context.run(
            egui::RawInput {
                events,
                ..Default::default()
            },
            |context| {
                egui::CentralPanel::default().show(context, |ui| {
                    render(
                        Node::switch("Sync".into(), checked.load(Ordering::SeqCst), event.clone())
                            .disabled(disabled.load(Ordering::SeqCst))
                            .styled(Style::new().width(Size::Px(200.0))),
                        ui,
                    )
                    .unwrap();
                });
            },
        )
    };
    let first = frame(vec![]);
    assert!(
        first
            .platform_output
            .accesskit_update
            .unwrap()
            .nodes
            .iter()
            .any(|(_, node)| node.role() == egui::accesskit::Role::Switch)
    );
    for pressed in [true, false] {
        frame(vec![
            egui::Event::PointerMoved(egui::pos2(20.0, 20.0)),
            egui::Event::PointerButton {
                pos: egui::pos2(20.0, 20.0),
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::NONE,
            },
        ]);
    }
    assert!(checked.load(Ordering::SeqCst));
    frame(vec![egui::Event::Key {
        key: egui::Key::Space,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::NONE,
    }]);
    assert!(!checked.load(Ordering::SeqCst));
    disabled.store(true, Ordering::SeqCst);
    for pressed in [true, false] {
        frame(vec![egui::Event::PointerButton {
            pos: egui::pos2(20.0, 20.0),
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        }]);
    }
    assert!(!checked.load(Ordering::SeqCst));
}

/// Meter/progress have distinct readonly native roles and numeric bounds.
#[test]
fn ranges_expose_numeric_values_and_distinct_native_accessibility_roles() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let output = context.run(Default::default(), |context| {
        egui::CentralPanel::default().show(context, |ui| {
            render(
                Node::column(vec![
                    Node::progress("Work".into(), 25.0, 100.0),
                    Node::meter("Temperature".into(), 0.0, -10.0, 30.0),
                ])
                .styled(Style::new().gap(12.0)),
                ui,
            )
            .unwrap();
        });
    });
    let update = output.platform_output.accesskit_update.unwrap();
    let meter = update
        .nodes
        .iter()
        .find(|(_, node)| node.role() == egui::accesskit::Role::Meter)
        .unwrap();
    assert_eq!(meter.1.numeric_value(), Some(0.0));
    assert_eq!(meter.1.min_numeric_value(), Some(-10.0));
    assert_eq!(meter.1.max_numeric_value(), Some(30.0));
    assert!(update.nodes.iter().any(|(_, node)| node.role()
        == egui::accesskit::Role::ProgressIndicator
        && node.numeric_value() == Some(25.0)));
}

/// Native painting uses authored part bounds/ink and owner hover while retaining controlled activation.
#[test]
fn headless_switch_parts_paint_custom_geometry_and_hover() {
    use inrux_style::{Color, parse_css};
    let context = egui::Context::default();
    let node=Node::switch("Sync".into(),false,Emit::new(|_|Ok(())))
        .class("choice",&parse_css(".choice::part(track) {width:48px;height:28px;background:#112233;border-radius:9px;} .choice::part(thumb) {width:20px;height:20px;background:#ddeeff;} .choice::part(track):hover {background:#778899;}").unwrap())
        .styled(Style::new().width(Size::Px(200.0)));
    let frame = |events| {
        context.run(
            egui::RawInput {
                events,
                ..Default::default()
            },
            |context| {
                egui::CentralPanel::default().show(context, |ui| {
                    render(node.clone(), ui).unwrap();
                });
            },
        )
    };
    let output = frame(vec![]);
    let has_track = |output: &egui::FullOutput, fill: Color| {
        output.shapes.iter().any(|shape|matches!(&shape.shape,egui::epaint::Shape::Rect(rect) if rect.fill==egui::Color32::from_rgba_unmultiplied(fill.0,fill.1,fill.2,fill.3) && rect.rect.size()==egui::vec2(48.0,28.0)))
    };
    assert!(has_track(&output, Color(17, 34, 51)));
    frame(vec![egui::Event::PointerMoved(egui::pos2(20.0, 20.0))]);
    assert!(has_track(&frame(vec![]), Color(119, 136, 153)));
}

/// A translucent native button must not paint the same authored surface in both wrapper and widget.
#[test]
fn native_button_alpha_is_preserved_and_painted_once() {
    let context = egui::Context::default();
    let fill = inrux_style::Color::rgba(20, 40, 200, 128);
    let output = context.run(egui::RawInput::default(), |context| {
        egui::CentralPanel::default().show(context, |ui| {
            let node = Node::button("Alpha".into(), None)
                .styled(Style::new().width(Size::Px(120.0)).background(fill));
            render(node, ui).unwrap();
        });
    });
    let native = egui::Color32::from_rgba_unmultiplied(fill.0, fill.1, fill.2, fill.3);
    assert_eq!(
        output
            .shapes
            .iter()
            .filter(
                |shape| matches!(&shape.shape,egui::epaint::Shape::Rect(rect) if rect.fill==native)
            )
            .count(),
        1
    );
}
