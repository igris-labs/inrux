//! Actual Masonry harness checks retained control identities, typing and layout.
use crate::widgets::label::Label;
use crate::{
    capabilities,
    widgets::element::{Element, Routes},
};
use inrux_components::Node;
use inrux_style::Style;
use masonry::{
    core::Widget,
    kurbo::Size,
    testing::TestHarness,
    widgets::{ButtonPress, TextAction},
};

/// Build an ordinary native harness through the same descriptor translator used by the host.
fn harness(node: &Node) -> (TestHarness<Element>, Routes) {
    capabilities::validate(node).unwrap();
    let mut routes = Routes::new();
    let tree = Element::build(node, &mut routes);
    (
        TestHarness::create_with_size(crate::theme::defaults(), tree, Size::new(600.0, 400.0)),
        routes,
    )
}
/// Keyboard editing must keep the same native focused text area after a keyed reorder.
#[test]
fn keyed_reorder_preserves_native_focus_and_text_input_identity() {
    let input = || {
        Node::input(
            "hello".into(),
            "Name".into(),
            inrux_core::Emit::new(|_| Ok(())),
        )
        .key("editor")
    };
    let label = || Node::text("Summary".into()).key("summary");
    let (mut harness, mut routes) = harness(&Node::column(vec![input(), label()]));
    let input_id = *routes
        .keys()
        .find(|id| {
            harness
                .get_widget_with_id(**id)
                .short_type_name()
                .contains("TextArea")
        })
        .unwrap();
    harness.focus_on(Some(input_id));
    harness.keyboard_type_chars("!");
    let (TextAction::Changed(value), _) = harness.pop_action::<TextAction>().unwrap() else {
        panic!("expected edit");
    };
    assert!(value.contains('!'));
    routes.clear();
    harness.edit_root_widget(|mut root| {
        Element::reconcile(
            &mut root,
            &Node::column(vec![label(), input()]),
            &mut routes,
        )
    });
    assert!(routes.contains_key(&input_id));
    assert_eq!(harness.focused_widget_id(), Some(input_id));
}
/// A new mount generation must replace native editing identity instead of reusing disposed focus.
#[test]
fn remount_replaces_the_native_control_generation() {
    let mut first =
        Node::input("first".into(), "".into(), inrux_core::Emit::new(|_| Ok(()))).key("editor");
    first.generation = Some("mount:1".into());
    let (mut harness, mut routes) = harness(&Node::column(vec![first.clone()]));
    let old: Vec<_> = routes.keys().copied().collect();
    first.generation = Some("mount:2".into());
    routes.clear();
    harness.edit_root_widget(|mut root| {
        Element::reconcile(&mut root, &Node::column(vec![first]), &mut routes)
    });
    assert!(old.iter().all(|id| !routes.contains_key(id)));
}
/// Real button pointer activation is produced by Masonry rather than fabricated adapter events.
#[test]
fn native_button_emits_an_activation_action() {
    let (mut harness, routes) = harness(&Node::button("Save".into(), None));
    let button = *routes.keys().next().unwrap();
    harness.mouse_click_on(button);
    assert!(harness.pop_action::<ButtonPress>().is_some());
}
/// Row and Column must translate into distinct native placement without recreating controls.
#[test]
fn row_layout_places_children_horizontally_with_the_authored_gap() {
    let tree = Node::row(vec![Node::text("Left".into()), Node::text("Right".into())])
        .styled(Style::new().gap(20.0).padding(8.0));
    let (harness, _) = harness(&tree);
    let root = harness.root_widget();
    let children = root.children_ids();
    let left = harness
        .get_widget_with_id(children[0])
        .ctx()
        .local_layout_rect();
    let right = harness
        .get_widget_with_id(children[1])
        .ctx()
        .local_layout_rect();
    assert!(right.x0 >= left.x1 + 19.0);
}

/// Native text must wrap within the allocated content width, including Badge padding.
#[test]
fn badge_text_uses_native_wrapped_lines_within_allocated_width() {
    use inrux_style::Size as LogicalSize;
    for (width, wraps) in [(100.0, true), (400.0, false)] {
        let node = Node::badge("A reusable status label with several words".into())
            .styled(Style::new().width(LogicalSize::Px(width)));
        let (harness, _) = harness(&node);
        let text_id = harness.root_widget().children_ids()[0];
        let text = harness.access_node(text_id).unwrap();
        let mut lines = Vec::new();
        for run in text.children() {
            let bounds = run.raw_bounds().unwrap();
            assert!(
                bounds.x1 <= f64::from(width - 8.0),
                "width={width}, bounds={bounds:?}"
            );
            lines.push(bounds.y0);
        }
        lines.sort_by(f64::total_cmp);
        lines.dedup();
        assert_eq!(lines.len() > 1, wraps);
    }
}
/// Unsupported controls and state styles are explicit rather than silently ignored.
#[test]
fn unsupported_controls_return_capability_errors() {
    let select = Node::select(
        "a".into(),
        vec!["a".into()],
        inrux_core::Emit::new(|_| Ok(())),
    );
    assert!(capabilities::validate(&select).is_err());
    assert!(capabilities::validate(&Node::row(vec![]).styled(Style::new().wrap(true))).is_ok());
}

/// The shared checkbox skin keeps native pointer, keyboard and disabled admission behavior.
#[test]
fn shared_checkbox_emits_controlled_actions_and_honors_disabled() {
    use masonry::widgets::CheckboxToggled;
    let checkbox = Node::checkbox(
        "Notifications".into(),
        false,
        inrux_core::Emit::new(|_| Ok(())),
    );
    let (mut harness, mut routes) = harness(&checkbox);
    let id = *routes.keys().next().unwrap();
    harness.mouse_click_on(id);
    assert!(harness.pop_action::<CheckboxToggled>().unwrap().0.0);
    harness.focus_on(Some(id));
    harness.process_text_event(masonry::core::TextEvent::Keyboard(
        masonry::ui_events::keyboard::KeyboardEvent {
            state: masonry::ui_events::keyboard::KeyState::Up,
            key: masonry::ui_events::keyboard::Key::Character(" ".into()),
            ..Default::default()
        },
    ));
    assert!(harness.pop_action::<CheckboxToggled>().unwrap().0.0);
    let disabled = checkbox.disabled(true);
    routes.clear();
    harness.edit_root_widget(|mut root| Element::reconcile(&mut root, &disabled, &mut routes));
    harness.mouse_click_on(id);
    assert!(harness.pop_action::<CheckboxToggled>().is_none());
}

/// The native editor rejects input after disabling but still displays an external reset.
#[test]
fn disabling_a_focused_editor_rejects_text_and_accepts_external_reset() {
    let mut node = Node::input(
        "original".into(),
        "Name".into(),
        inrux_core::Emit::new(|_| Ok(())),
    );
    let (mut harness, mut routes) = harness(&node);
    let id = *routes
        .keys()
        .find(|id| {
            harness
                .get_widget_with_id(**id)
                .short_type_name()
                .contains("TextArea")
        })
        .unwrap();
    harness.focus_on(Some(id));
    node.disabled = true;
    routes.clear();
    harness.edit_root_widget(|mut root| Element::reconcile(&mut root, &node, &mut routes));
    assert_eq!(harness.focused_widget_id(), None);
    harness.keyboard_type_chars("blocked");
    assert!(harness.pop_action::<TextAction>().is_none());
    if let inrux_components::NodeKind::Input { value, .. } = &mut node.kind {
        *value = "reset".into();
    }
    routes.clear();
    harness.edit_root_widget(|mut root| Element::reconcile(&mut root, &node, &mut routes));
    let area = harness
        .get_widget_with_id(id)
        .downcast::<crate::widgets::input::TextArea<true>>()
        .unwrap();
    assert_eq!(area.text(), "reset");
}

/// Secondary pointer input must not toggle the shared checkbox or steal focus.
#[test]
fn secondary_click_does_not_activate_checkbox() {
    use masonry::{ui_events::pointer::PointerButton, widgets::CheckboxToggled};
    let (mut harness, routes) = harness(&Node::checkbox(
        "Notifications".into(),
        false,
        inrux_core::Emit::new(|_| Ok(())),
    ));
    let id = *routes.keys().next().unwrap();
    harness.mouse_move_to(id);
    harness.mouse_button_press(PointerButton::Secondary);
    harness.mouse_button_release(PointerButton::Secondary);
    assert!(harness.pop_action::<CheckboxToggled>().is_none());
    assert_eq!(harness.focused_widget_id(), None);
}

/// The native placeholder must inherit the same font family and size on its very first frame.
#[test]
fn placeholder_inherits_typography_on_initial_mount() {
    use masonry::parley::StyleProperty;
    let node = Node::input("".into(), "Name".into(), inrux_core::Emit::new(|_| Ok(())))
        .styled(Style::new().font_family("Noto Sans").font_size(24.0));
    let (mut harness, _) = harness(&node);
    let input = harness.root_widget().children_ids()[0];
    let placeholder = harness.get_widget_with_id(input).children_ids()[1];
    let mut size = None;
    let mut family = None;
    harness.edit_widget_with_id(placeholder, |mut label| {
        Label::retain_styles(&mut label.downcast::<Label>(), |property| {
            match property {
                StyleProperty::FontSize(value) => size = Some(*value),
                StyleProperty::FontStack(value) => family = Some(value.clone()),
                _ => {}
            }
            true
        });
    });
    assert_eq!(size, Some(24.0));
    assert_eq!(family, Some(crate::widgets::text::family("Noto Sans")));
}

/// Native hover updates the actual button background, restores it, and skips disabled controls.
#[test]
fn button_hover_updates_native_paint_and_respects_disabled_state() {
    use inrux_style::{Color, Size};
    use masonry::properties::Background;
    let make = |disabled| {
        Node::button("Save".into(), None).disabled(disabled).styled(
            Style::new()
                .background(Color(29, 112, 98))
                .width(Size::Px(120.0))
                .height(Size::Px(40.0)),
        )
    };
    let (mut harness, mut routes) = harness(&make(false));
    let id = *routes.keys().next().unwrap();
    let base = harness
        .get_widget_with_id(id)
        .get_prop::<Background>()
        .clone();
    harness.mouse_move_to(id);
    assert_ne!(
        harness.get_widget_with_id(id).get_prop::<Background>(),
        &base
    );
    harness.mouse_move((500.0, 300.0));
    assert_eq!(
        harness.get_widget_with_id(id).get_prop::<Background>(),
        &base
    );
    harness.edit_root_widget(|mut root| Element::reconcile(&mut root, &make(true), &mut routes));
    harness.mouse_move_to_unchecked(id);
    assert_eq!(
        harness.get_widget_with_id(id).get_prop::<Background>(),
        &base
    );
}

/// Real native clipping excludes an offscreen button until wheel movement reveals it.
#[test]
fn viewport_clips_hit_testing_and_wheel_reveals_retained_controls() {
    use inrux_style::Size as LogicalSize;
    let buttons = (0..8)
        .map(|i| {
            Node::button(format!("Row {i}"), None)
                .key(format!("row-{i}"))
                .styled(Style::new().height(LogicalSize::Px(40.0)))
        })
        .collect();
    let node = Node::scroll_area(buttons).styled(
        Style::new()
            .height(LogicalSize::Px(100.0))
            .width(LogicalSize::Px(200.0)),
    );
    let (mut harness, routes) = harness(&node);
    let last_wrapper = harness.root_widget().children_ids()[7];
    let last = *routes
        .iter()
        .find(|(_, wrapper)| **wrapper == last_wrapper)
        .unwrap()
        .0;
    harness.mouse_move((20.0, 60.0));
    harness.mouse_wheel(masonry::kurbo::Vec2::new(0.0, -1000.0));
    assert_eq!(
        harness
            .root_widget()
            .scroll
            .as_ref()
            .unwrap()
            .borrow()
            .metrics()
            .offset
            .y,
        220.0
    );
    harness.mouse_click_on(last);
    assert!(harness.pop_action::<ButtonPress>().is_some());
    harness.mouse_move((20.0, 60.0));
    harness.mouse_wheel(masonry::kurbo::Vec2::new(0.0, 1000.0));
    assert_eq!(
        harness
            .root_widget()
            .scroll
            .as_ref()
            .unwrap()
            .borrow()
            .metrics()
            .offset
            .y,
        0.0
    );
    // A click at the hidden button's old content coordinate cannot reach it.
    harness.mouse_move((20.0, 300.0));
    harness.mouse_button_press(masonry::ui_events::pointer::PointerButton::Primary);
    harness.mouse_button_release(masonry::ui_events::pointer::PointerButton::Primary);
    assert!(harness.pop_action::<ButtonPress>().is_none());
    harness.focus_on(Some(last));
    assert!(
        harness
            .root_widget()
            .scroll
            .as_ref()
            .unwrap()
            .borrow()
            .metrics()
            .offset
            .y
            > 0.0
    );
}

/// A child at its lower edge leaves the next wheel event for its parent.
#[test]
fn nested_viewports_route_wheel_to_the_deepest_movable_target() {
    use inrux_style::Size as LogicalSize;
    let inner = Node::scroll_area(vec![
        Node::box_container(vec![]).styled(Style::new().height(LogicalSize::Px(400.0))),
    ])
    .styled(Style::new().height(LogicalSize::Px(100.0)));
    let root = Node::scroll_area(vec![
        inner,
        Node::box_container(vec![]).styled(Style::new().height(LogicalSize::Px(400.0))),
    ])
    .styled(
        Style::new()
            .height(LogicalSize::Px(140.0))
            .width(LogicalSize::Px(200.0)),
    );
    let (mut harness, _) = harness(&root);
    let inner = harness.root_widget().children_ids()[0];
    harness.mouse_move((30.0, 40.0));
    harness.mouse_wheel(masonry::kurbo::Vec2::new(0.0, -1000.0));
    assert_eq!(
        harness
            .root_widget()
            .scroll
            .as_ref()
            .unwrap()
            .borrow()
            .metrics()
            .offset
            .y,
        0.0
    );
    assert_eq!(
        harness
            .get_widget_with_id(inner)
            .downcast::<Element>()
            .unwrap()
            .scroll
            .as_ref()
            .unwrap()
            .borrow()
            .metrics()
            .offset
            .y,
        300.0
    );
    harness.mouse_wheel(masonry::kurbo::Vec2::new(0.0, -40.0));
    assert_eq!(
        harness
            .root_widget()
            .scroll
            .as_ref()
            .unwrap()
            .borrow()
            .metrics()
            .offset
            .y,
        40.0
    );
}

/// Native capture keeps a thumb drag active beyond its viewport and releases on pointer-up.
#[test]
fn viewport_scrollbar_captures_pointer_outside_its_bounds() {
    use inrux_style::Size as LogicalSize;
    use masonry::ui_events::pointer::PointerButton;
    let node = Node::scroll_area(vec![
        Node::box_container(vec![]).styled(Style::new().height(LogicalSize::Px(400.0))),
    ])
    .styled(
        Style::new()
            .width(LogicalSize::Px(200.0))
            .height(LogicalSize::Px(100.0)),
    );
    let (mut harness, _) = harness(&node);
    harness.mouse_move((195.0, 10.0));
    harness.mouse_button_press(PointerButton::Primary);
    assert!(harness.pointer_capture_target_id().is_some());
    harness.mouse_move((500.0, 300.0));
    harness.mouse_button_release(PointerButton::Primary);
    assert!(harness.pointer_capture_target_id().is_none());
    assert_eq!(
        harness
            .root_widget()
            .scroll
            .as_ref()
            .unwrap()
            .borrow()
            .metrics()
            .offset
            .y,
        300.0
    );
}

/// Only the viewport's own focus admits navigation keys, through native keyboard dispatch.
#[test]
fn viewport_keyboard_pages_and_home_end_follow_shared_bounds() {
    use inrux_style::Size as LogicalSize;
    use masonry::ui_events::keyboard::{Key, KeyState, KeyboardEvent, NamedKey};
    let node = Node::scroll_area(vec![
        Node::box_container(vec![]).styled(Style::new().height(LogicalSize::Px(400.0))),
    ])
    .styled(Style::new().height(LogicalSize::Px(100.0)));
    let (mut harness, _) = harness(&node);
    let id = harness.root_widget().ctx().widget_id();
    harness.focus_on(Some(id));
    for (key, expected) in [
        (NamedKey::PageDown, 60.0),
        (NamedKey::End, 300.0),
        (NamedKey::Home, 0.0),
    ] {
        harness.process_text_event(masonry::core::TextEvent::Keyboard(KeyboardEvent {
            state: KeyState::Down,
            key: Key::Named(key),
            ..Default::default()
        }));
        assert_eq!(
            harness
                .root_widget()
                .scroll
                .as_ref()
                .unwrap()
                .borrow()
                .metrics()
                .offset
                .y,
            expected
        );
    }
}

/// The native editor keeps one line, supports manual panning, and reveals Home/End selections.
#[test]
fn long_input_pans_and_reveals_caret_without_changing_control_width() {
    use crate::widgets::input::TextArea;
    use masonry::ui_events::keyboard::{Key, KeyState, KeyboardEvent, NamedKey};
    let value = "Rust 界 workspace text that is wider than this field ".repeat(8);
    let node = Node::input(
        value.clone(),
        "Name".into(),
        inrux_core::Emit::new(|_| Ok(())),
    )
    .styled(
        Style::new()
            .width(inrux_style::Size::Px(140.0))
            .height(inrux_style::Size::Px(40.0)),
    );
    let (mut harness, routes) = harness(&node);
    let area = *routes.keys().next().unwrap();
    harness.focus_on(Some(area));
    harness.mouse_move((35.0, 20.0));
    harness.mouse_wheel(masonry::kurbo::Vec2::new(-90.0, 0.0));
    assert!(
        harness
            .get_widget_with_id(area)
            .downcast::<TextArea<true>>()
            .unwrap()
            .horizontal_offset()
            > 0.0
    );
    for key in [NamedKey::End, NamedKey::Home] {
        harness.process_text_event(masonry::core::TextEvent::Keyboard(KeyboardEvent {
            state: KeyState::Down,
            key: Key::Named(key),
            ..Default::default()
        }));
        let editor = harness
            .get_widget_with_id(area)
            .downcast::<TextArea<true>>()
            .unwrap();
        assert!(
            editor.ime_area().y1 <= editor.ctx().size().height,
            "single-line glyphs must fit vertically inside the clip"
        );
        assert!(editor.ime_area().x0 >= 0.0);
        assert!(editor.ime_area().x1 <= editor.ctx().size().width + 2.0);
        assert_eq!(editor.text(), value.as_str());
        let field = harness.root_widget().children_ids()[0];
        assert_eq!(harness.get_widget_with_id(field).ctx().size().width, 140.0);
    }
    harness.mouse_move((35.0, 20.0));
    harness.mouse_wheel(masonry::kurbo::Vec2::new(-70.0, 0.0));
    assert!(
        harness
            .get_widget_with_id(area)
            .downcast::<TextArea<true>>()
            .unwrap()
            .horizontal_offset()
            > 0.0
    );
    harness.process_text_event(masonry::core::TextEvent::Keyboard(KeyboardEvent {
        state: KeyState::Down,
        key: Key::Named(NamedKey::Home),
        ..Default::default()
    }));
    assert_eq!(
        harness
            .get_widget_with_id(area)
            .downcast::<TextArea<true>>()
            .unwrap()
            .horizontal_offset(),
        0.0
    );
    harness.edit_widget_with_id(area, |mut widget| {
        TextArea::select_byte_range(&mut widget.downcast::<TextArea<true>>(), 0, 0);
    });
    assert_eq!(
        harness
            .get_widget_with_id(area)
            .downcast::<TextArea<true>>()
            .unwrap()
            .horizontal_offset(),
        0.0
    );
}

/// Clipboard text obeys the public editing mode; Enter is reserved for TextArea.
#[test]
fn native_single_line_and_multiline_paste_and_enter() {
    use crate::widgets::input::TextArea;
    use masonry::ui_events::keyboard::{Key, KeyState, KeyboardEvent, NamedKey};
    for multiline in [false, true] {
        let event = inrux_core::Emit::new(|_| Ok(()));
        let node = if multiline {
            Node::text_area(String::new(), "Notes".into(), event)
        } else {
            Node::input(String::new(), "Name".into(), event)
        }
        .styled(
            Style::new()
                .width(inrux_style::Size::Px(180.0))
                .height(inrux_style::Size::Px(100.0)),
        );
        let (mut harness, routes) = harness(&node);
        let area = *routes.keys().next().unwrap();
        harness.focus_on(Some(area));
        harness.process_text_event(masonry::core::TextEvent::ClipboardPaste(
            "first\r\nsecond\u{2028}third".into(),
        ));
        let expected = if multiline {
            "first\nsecond\nthird"
        } else {
            "first second third"
        };
        assert_eq!(
            harness
                .get_widget_with_id(area)
                .downcast::<TextArea<true>>()
                .unwrap()
                .text(),
            expected
        );
        harness.process_text_event(masonry::core::TextEvent::Keyboard(KeyboardEvent {
            state: KeyState::Down,
            key: Key::Named(NamedKey::Enter),
            ..Default::default()
        }));
        let editor = harness
            .get_widget_with_id(area)
            .downcast::<TextArea<true>>()
            .unwrap();
        assert_eq!(
            editor
                .text()
                .into_iter()
                .collect::<String>()
                .ends_with('\n'),
            multiline
        );
        harness.process_text_event(masonry::core::TextEvent::ClipboardPaste(
            "a long paragraph that should wrap onto another visual row. ".repeat(40),
        ));
        let editor = harness
            .get_widget_with_id(area)
            .downcast::<TextArea<true>>()
            .unwrap();
        let caret = editor.ime_area();
        assert!(caret.y0 >= -0.1, "{caret:?}");
        assert!(caret.y1 <= editor.ctx().size().height + 0.1, "{caret:?}");

        if multiline {
            use masonry::ui_events::keyboard::Modifiers;
            harness.process_text_event(masonry::core::TextEvent::Keyboard(KeyboardEvent {
                state: KeyState::Down,
                key: Key::Named(if cfg!(target_os = "macos") {
                    NamedKey::ArrowUp
                } else {
                    NamedKey::Home
                }),
                modifiers: if cfg!(target_os = "macos") {
                    Modifiers::META
                } else {
                    Modifiers::CONTROL
                },
                ..Default::default()
            }));
            let editor = harness
                .get_widget_with_id(area)
                .downcast::<TextArea<true>>()
                .unwrap();
            assert!(
                editor.ime_area().y0 < 1.0,
                "document start must reveal first line"
            );
        }
    }
}

/// Parley-backed Center layout keeps padded intrinsic text centered at narrow and ordinary widths.
#[test]
fn native_center_with_padded_text_uses_valid_content_widths() {
    for width in [100.0, 400.0] {
        let node = Node::center(vec![
            Node::text("Centered on both axes".into()).styled(Style::new().padding(10.0)),
        ])
        .styled(
            Style::new()
                .width(inrux_style::Size::Px(width))
                .height(inrux_style::Size::Px(150.0))
                .padding(12.0),
        );
        let (harness, _) = harness(&node);
        let child = harness.root_widget().children_ids()[0];
        let rect = harness.get_widget_with_id(child).ctx().local_layout_rect();
        assert!((rect.x0 + rect.x1 - f64::from(width)).abs() <= 1.0);
        assert!((rect.y0 + rect.y1 - 150.0).abs() <= 1.0);
    }
}

/// Real native pointer capture suppresses decoration, then keyboard interaction restores it.
#[test]
fn button_focus_visibility_tracks_native_pointer_and_keyboard_without_replacing_focus() {
    let (mut harness, routes) = harness(&Node::button("Save".into(), None));
    let id = *routes.keys().next().unwrap();
    harness.mouse_click_on(id);
    assert!(!harness.root_widget().focus_visibility.visible());
    harness.focus_on(Some(id));
    harness.process_text_event(masonry::core::TextEvent::Keyboard(
        masonry::ui_events::keyboard::KeyboardEvent {
            state: masonry::ui_events::keyboard::KeyState::Down,
            key: masonry::ui_events::keyboard::Key::Named(
                masonry::ui_events::keyboard::NamedKey::Enter,
            ),
            ..Default::default()
        },
    ));
    assert!(harness.root_widget().focus_visibility.visible());
    assert_eq!(harness.focused_widget_id(), Some(id));
}

/// Switch changes the native semantic role while reusing controlled boolean event handling.
#[test]
fn native_switch_controls_pointer_keyboard_disabled_and_accessible_value() {
    use masonry::{accesskit, widgets::CheckboxToggled};
    let event = inrux_core::Emit::new(|_| Ok(()));
    let (mut harness, mut routes) = harness(&Node::switch("Sync".into(), false, event.clone()));
    let id = *routes.keys().next().unwrap();
    assert_eq!(
        harness.access_node(id).unwrap().role(),
        accesskit::Role::Switch
    );
    harness.mouse_click_on(id);
    assert!(harness.pop_action::<CheckboxToggled>().unwrap().0.0);
    routes.clear();
    harness.edit_root_widget(|mut root| {
        Element::reconcile(
            &mut root,
            &Node::switch("Sync".into(), true, event),
            &mut routes,
        )
    });
    harness.focus_on(Some(id));
    harness.process_text_event(masonry::core::TextEvent::Keyboard(
        masonry::ui_events::keyboard::KeyboardEvent {
            state: masonry::ui_events::keyboard::KeyState::Up,
            key: masonry::ui_events::keyboard::Key::Character(" ".into()),
            ..Default::default()
        },
    ));
    assert!(!harness.pop_action::<CheckboxToggled>().unwrap().0.0);
    let disabled =
        Node::switch("Sync".into(), true, inrux_core::Emit::new(|_| Ok(()))).disabled(true);
    harness.edit_root_widget(|mut root| Element::reconcile(&mut root, &disabled, &mut routes));
    harness.mouse_click_on(id);
    assert!(harness.pop_action::<CheckboxToggled>().is_none());
}

/// Retained range updates preserve identity while refreshing native numeric/semantic data.
#[test]
fn native_ranges_update_bounds_values_and_roles_in_place() {
    use masonry::accesskit;
    let (mut harness, mut routes) = harness(&Node::progress("Work".into(), 25.0, 100.0));
    let id = *routes.keys().next().unwrap();
    let node = harness.access_node(id).unwrap();
    assert_eq!(node.role(), accesskit::Role::ProgressIndicator);
    assert_eq!(node.numeric_value(), Some(25.0));
    harness.edit_root_widget(|mut root| {
        Element::reconcile(
            &mut root,
            &Node::meter("Temperature".into(), 0.0, -10.0, 30.0),
            &mut routes,
        )
    });
    assert!(routes.contains_key(&id));
    // Accessibility publications are consumed when the native render pass runs.
    harness.render();
    let node = harness.access_node(id).unwrap();
    assert_eq!(node.role(), accesskit::Role::Meter);
    assert_eq!(node.numeric_value(), Some(0.0));
    assert_eq!(node.min_numeric_value(), Some(-10.0));
    assert_eq!(node.max_numeric_value(), Some(30.0));
}

/// Authored native hover paint takes precedence over the default derived hover affordance.
#[test]
fn authored_button_state_paint_updates_native_properties_and_restores() {
    use inrux_style::{Color, Size, parse_css};
    use masonry::properties::Background;
    let node=Node::button("Action".into(),None)
        .class("action",&parse_css(".action {background:#112233;} .action:hover {background:#778899;} .action:disabled {background:#223344;}").unwrap())
        .styled(Style::new().width(Size::Px(120.0)));
    let (mut harness, mut routes) = harness(&node);
    let id = *routes.keys().next().unwrap();
    harness.mouse_move_to(id);
    assert_eq!(
        harness.get_widget_with_id(id).get_prop::<Background>(),
        &Background::Color(crate::widgets::text::color(Color(119, 136, 153)))
    );
    harness.mouse_move((500.0, 300.0));
    assert_eq!(
        harness.get_widget_with_id(id).get_prop::<Background>(),
        &Background::Color(crate::widgets::text::color(Color(17, 34, 51)))
    );
    harness.edit_root_widget(|mut root| {
        Element::reconcile(&mut root, &node.disabled(true), &mut routes)
    });
    assert_eq!(
        harness.get_widget_with_id(id).get_prop::<Background>(),
        &Background::Color(crate::widgets::text::color(Color(34, 51, 68)))
    );
}

/// Native properties retain straight alpha instead of flattening against a guessed background.
#[test]
fn native_background_and_text_conversion_preserve_alpha() {
    use masonry::properties::Background;
    let ink = inrux_style::Color::rgba(20, 40, 200, 128);
    let node = Node::button("Alpha".into(), None).styled(Style::new().background(ink));
    let (harness, routes) = harness(&node);
    let id = *routes.keys().next().unwrap();
    let native = crate::widgets::text::color(ink);
    assert_eq!(native.to_rgba8().a, 128);
    assert_eq!(
        harness.get_widget_with_id(id).get_prop::<Background>(),
        &Background::Color(native)
    );
}

/// A stylesheet added after mount can make passive text hoverable without replacing its native ID.
#[test]
fn passive_owner_hover_can_be_enabled_after_mount_and_respects_logical_bounds() {
    use inrux_style::{Color, Size, parse_css};
    use masonry::properties::ContentColor;
    let plain = Node::text("Label".into())
        .styled(Style::new().width(Size::Px(100.0)).color(Color(1, 2, 3)));
    let (mut harness, mut routes) = harness(&plain);
    let id = *routes.keys().next().unwrap();
    let styled = Node::text("Label".into())
        .class(
            "label",
            &parse_css(".label {color:#010203;} .label:hover {color:#778899;}").unwrap(),
        )
        .styled(Style::new().width(Size::Px(100.0)));
    harness.edit_root_widget(|mut root| Element::reconcile(&mut root, &styled, &mut routes));
    assert!(routes.contains_key(&id));
    harness.mouse_move((20.0, 10.0));
    assert_eq!(
        harness
            .get_widget_with_id(id)
            .get_prop::<ContentColor>()
            .color,
        crate::widgets::text::color(Color(119, 136, 153))
    );
    harness.mouse_move((500.0, 300.0));
    assert_eq!(
        harness
            .get_widget_with_id(id)
            .get_prop::<ContentColor>()
            .color,
        crate::widgets::text::color(Color(1, 2, 3))
    );
}

/// Native button constraints must not shift glyphs away from the framework's visual center.
#[test]
fn native_button_label_centers_the_shared_line_box_with_fractional_padding() {
    use inrux_style::Size as LogicalSize;
    let node = Node::button("office AV".into(), None).styled(
        Style::new()
            .width(LogicalSize::Px(137.0))
            .height(LogicalSize::Px(37.0))
            .font_size(13.0)
            .padding(5.25)
            .border_width(1.0),
    );
    let (harness, _) = harness(&node);
    let root = harness.root_widget();
    let content = root.content.as_ref().unwrap().id();
    let button = harness.get_widget_with_id(content);
    let label = harness
        .get_widget_with_id(button.children_ids()[0])
        .downcast::<Label>()
        .unwrap();
    let actual = label.ctx().window_origin() + label.paint_shift();
    let shared = inrux_text::shape("office AV", 13.0, None, f32::INFINITY);
    assert!(
        (actual.x - (137.0 - f64::from(shared.width())) / 2.0).abs() < 0.001,
        "{actual:?}"
    );
    assert!(
        (actual.y - (37.0 - f64::from(shared.height())) / 2.0).abs() < 0.001,
        "{actual:?}"
    );
}
/// Borders are painted inside the field; text padding/centering stays identical to the other bindings.
#[test]
fn native_input_uses_authored_padding_and_shared_vertical_center() {
    use inrux_style::Size as LogicalSize;
    let node = Node::input(
        "office AV".into(),
        "Name".into(),
        inrux_core::Emit::new(|_| Ok(())),
    )
    .styled(
        Style::new()
            .width(LogicalSize::Px(137.0))
            .height(LogicalSize::Px(37.0))
            .font_size(13.0)
            .padding(5.25)
            .border_width(3.0),
    );
    let (harness, routes) = harness(&node);
    let area = *routes
        .iter()
        .find(|(_, owner)| **owner == harness.root_widget().ctx().widget_id())
        .unwrap()
        .0;
    let native = harness.get_widget_with_id(area);
    let origin = native.ctx().window_origin();
    let shared = inrux_text::shape("office AV", 13.0, None, f32::INFINITY);
    assert!((origin.x - 5.25).abs() < 0.001, "{origin:?}");
    assert!(
        (origin.y - (37.0 - f64::from(shared.height())) / 2.0).abs() < 0.001,
        "{origin:?}"
    );
}
