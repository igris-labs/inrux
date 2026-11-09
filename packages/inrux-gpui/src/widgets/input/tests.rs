//! Native editing regressions through GPUI text services and keyboard dispatch.
use super::*;
use inrux_core::Emit;
use std::prelude::v1::test;
/// Native input must commit Unicode text, delete graphemes and accept external resets.
#[gpui::test]
fn native_typing_selection_and_external_reset(cx: &mut gpui::TestAppContext) {
    cx.update(bind_keys);
    let (sent, received) = std::sync::mpsc::channel();
    let callback = Emit::new(move |value| {
        sent.send(value).unwrap();
        Ok(())
    });
    let node = Node::input("original".into(), "Name".into(), callback.clone());
    let (view, cx) = cx.add_window_view(|_, cx| TextInput::new(&node, Default::default(), cx));
    // Focus through the native engine; normal shortcuts and text service input follow.
    cx.update(|window, cx| window.focus(&view.read(cx).focus_handle));
    cx.simulate_keystrokes(if cfg!(target_os = "macos") {
        "cmd-a"
    } else {
        "ctrl-a"
    });
    cx.simulate_input("Rust 🦀");
    assert_eq!(received.try_iter().last().unwrap(), "Rust 🦀");
    cx.simulate_keystrokes("backspace");
    assert_eq!(received.try_recv().unwrap(), "Rust ");
    let changed = Node::input("reset".into(), "Name".into(), callback);
    view.update(cx, |input, cx| input.reconcile(&changed, cx));
    assert_eq!(
        view.read_with(cx, |input, _| input.content.to_string()),
        "reset"
    );
}
/// Native UTF-16 preedit is kept local until commit, including surrogate-pair selections.
#[gpui::test]
fn preedit_survives_acknowledgement_and_commits_once(cx: &mut gpui::TestAppContext) {
    let (sent, received) = std::sync::mpsc::channel();
    let node = Node::input(
        "a".into(),
        "Name".into(),
        Emit::new(move |value| {
            sent.send(value).unwrap();
            Ok(())
        }),
    );
    let (view, cx) = cx.add_window_view(|_, cx| TextInput::new(&node, Default::default(), cx));
    cx.update(|window, cx| {
        view.update(cx, |input, cx| {
            input.replace_and_mark_text_in_range(Some(1..1), "😀", Some(2..2), window, cx);
            input.reconcile(&node, cx);
            assert_eq!(input.content.as_ref(), "a😀");
            assert_eq!(input.selected_range, 5..5);
            input.replace_text_in_range(None, "界", window, cx);
        })
    });
    assert_eq!(received.try_recv().unwrap(), "a界");
    assert!(received.try_recv().is_err());
}

/// Disabling a focused native input clears focus and rejects queued text without blocking resets.
#[gpui::test]
fn disabled_editor_releases_focus_and_accepts_external_model(cx: &mut TestAppContext) {
    cx.update(bind_keys);
    let (sent, received) = std::sync::mpsc::channel();
    let mut node = Node::input(
        "original".into(),
        "Name".into(),
        Emit::new(move |value| {
            sent.send(value).unwrap();
            Ok(())
        }),
    );
    let (view, cx) = cx.add_window_view(|_, cx| TextInput::new(&node, Default::default(), cx));
    cx.update(|window, cx| window.focus(&view.read(cx).focus_handle));
    node.disabled = true;
    view.update(cx, |input, cx| input.reconcile(&node, cx));
    cx.run_until_parked();
    cx.update(|window, cx| assert!(!view.read(cx).focus_handle.is_focused(window)));
    cx.simulate_input("blocked");
    assert!(received.try_recv().is_err());
    if let NodeKind::Input { value, .. } = &mut node.kind {
        *value = "reset".into();
    }
    view.update(cx, |input, cx| input.reconcile(&node, cx));
    assert_eq!(
        view.read_with(cx, |input, _| input.content.to_string()),
        "reset"
    );
}

/// Standard macOS line navigation must select the same Unicode text as a native single-line editor.
#[cfg(target_os = "macos")]
#[gpui::test]
fn platform_line_shortcuts_preserve_unicode_selection(cx: &mut TestAppContext) {
    cx.update(bind_keys);
    let (sent, received) = std::sync::mpsc::channel();
    let node = Node::input(
        "A😀B".into(),
        "Name".into(),
        Emit::new(move |value| {
            sent.send(value).unwrap();
            Ok(())
        }),
    );
    let (view, cx) = cx.add_window_view(|_, cx| TextInput::new(&node, Default::default(), cx));
    cx.update(|window, cx| window.focus(&view.read(cx).focus_handle));
    cx.simulate_keystrokes("cmd-left cmd-shift-right");
    cx.simulate_input("replacement");
    assert_eq!(received.try_iter().last().unwrap(), "replacement");
}

/// Native caret work is bounded, stays stopped after timeout, restarts on editing and cancels on blur.
#[gpui::test]
fn caret_timer_stops_after_idle_and_restarts_on_edit(cx: &mut TestAppContext) {
    let node = Node::input("original".into(), "Name".into(), Emit::new(|_| Ok(())));
    let (view, cx) = cx.add_window_view(|_, cx| TextInput::new(&node, Default::default(), cx));
    cx.update(|window, cx| {
        window.activate_window();
        window.focus(&view.read(cx).focus_handle);
    });
    cx.run_until_parked();
    assert!(view.read_with(cx, |input, _| input.caret_task.is_some()));
    cx.background_executor
        .advance_clock(std::time::Duration::from_millis(500));
    cx.run_until_parked();
    assert!(!view.read_with(cx, |input, _| input.caret_visible));
    for _ in 1..20 {
        cx.background_executor
            .advance_clock(std::time::Duration::from_millis(500));
        cx.run_until_parked();
    }
    assert!(view.read_with(cx, |input, _| input.caret_visible));
    cx.background_executor
        .advance_clock(std::time::Duration::from_secs(20));
    cx.run_until_parked();
    assert!(view.read_with(cx, |input, _| input.caret_visible));
    cx.simulate_input("!");
    cx.background_executor
        .advance_clock(std::time::Duration::from_millis(500));
    cx.run_until_parked();
    assert!(!view.read_with(cx, |input, _| input.caret_visible));
    cx.update(|window, _| window.blur());
    cx.run_until_parked();
    assert!(view.read_with(cx, |input, _| input.caret_task.is_none()));
}

/// A model replacement cancels active preedit instead of retaining invisible uncommitted content.
#[gpui::test]
fn external_reset_cancels_preedit_without_emitting_a_commit(cx: &mut TestAppContext) {
    let (sent, received) = std::sync::mpsc::channel();
    let mut node = Node::input(
        "a".into(),
        "Name".into(),
        Emit::new(move |value| {
            sent.send(value).unwrap();
            Ok(())
        }),
    );
    let (view, cx) = cx.add_window_view(|_, cx| TextInput::new(&node, Default::default(), cx));
    cx.update(|window, cx| {
        view.update(cx, |input, cx| {
            input.replace_and_mark_text_in_range(Some(1..1), "😀", Some(2..2), window, cx)
        })
    });
    if let NodeKind::Input { value, .. } = &mut node.kind {
        *value = "reset".into();
    }
    view.update(cx, |input, cx| {
        input.reconcile(&node, cx);
        assert_eq!(input.content.as_ref(), "reset");
        assert!(input.marked_range.is_none());
    });
    assert!(received.try_recv().is_err());
}

/// Clipboard commands go through native GPUI services and preserve a Unicode payload.
#[gpui::test]
fn clipboard_cut_and_paste_preserve_unicode(cx: &mut TestAppContext) {
    cx.update(bind_keys);
    let (sent, received) = std::sync::mpsc::channel();
    let node = Node::input(
        "Rust 界😀".into(),
        "Name".into(),
        Emit::new(move |value| {
            sent.send(value).unwrap();
            Ok(())
        }),
    );
    let (view, cx) = cx.add_window_view(|_, cx| TextInput::new(&node, Default::default(), cx));
    cx.update(|window, cx| window.focus(&view.read(cx).focus_handle));
    cx.simulate_keystrokes(if cfg!(target_os = "macos") {
        "cmd-a cmd-x"
    } else {
        "ctrl-a ctrl-x"
    });
    assert_eq!(received.try_recv().unwrap(), "");
    cx.simulate_keystrokes(if cfg!(target_os = "macos") {
        "cmd-v"
    } else {
        "ctrl-v"
    });
    assert_eq!(received.try_recv().unwrap(), "Rust 界😀");
}

/// Horizontal wheel input persists until caret movement, with coordinates shared by pointer/IME paths.
#[gpui::test]
fn long_input_manual_pan_and_caret_reveal(cx: &mut TestAppContext) {
    struct Field {
        input: Entity<TextInput>,
    }
    impl Render for Field {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().w(px(140.0)).h(px(40.0)).child(self.input.clone())
        }
    }
    cx.update(bind_keys);
    let value = "Rust 界 long workspace name ".repeat(20);
    let node = Node::input(value.clone(), "Name".into(), Emit::new(|_| Ok(())));
    let (view, cx) = cx.add_window_view(|_, cx| Field {
        input: cx.new(|cx| TextInput::new(&node, Default::default(), cx)),
    });
    let input = view.read_with(cx, |view, _| view.input.clone());
    cx.update(|window, cx| window.focus(&input.read(cx).focus_handle));
    cx.run_until_parked();
    assert!(input.read_with(cx, |input, _| input.viewport.offset()) > 0.0);
    cx.simulate_keystrokes("home");
    cx.run_until_parked();
    assert_eq!(input.read_with(cx, |input, _| input.viewport.offset()), 0.0);
    cx.simulate_event(ScrollWheelEvent {
        position: point(px(40.0), px(20.0)),
        delta: ScrollDelta::Pixels(point(px(-70.0), px(0.0))),
        ..Default::default()
    });
    cx.run_until_parked();
    let offset = input.read_with(cx, |input, _| input.viewport.offset());
    assert!(offset > 0.0);
    cx.update(|window, _| window.refresh());
    cx.run_until_parked();
    assert_eq!(
        input.read_with(cx, |input, _| input.viewport.offset()),
        offset
    );
    cx.simulate_keystrokes("home");
    cx.run_until_parked();
    assert_eq!(input.read_with(cx, |input, _| input.viewport.offset()), 0.0);
    cx.simulate_keystrokes("end");
    cx.run_until_parked();
    assert!(input.read_with(cx, |input, _| input.viewport.offset()) > offset);
    assert_eq!(
        input.read_with(cx, |input, _| input.content.to_string()),
        value
    );
}

/// Input normalizes paste/IME; TextArea keeps line breaks and reveals a wrapped caret.
#[gpui::test]
fn native_text_modes_keep_models_and_geometry_consistent(cx: &mut TestAppContext) {
    cx.update(bind_keys);
    for multiline in [false, true] {
        let (sent, received) = std::sync::mpsc::channel();
        let event = Emit::new(move |value| {
            sent.send(value).unwrap();
            Ok(())
        });
        let node = if multiline {
            Node::text_area(String::new(), "Notes".into(), event)
        } else {
            Node::input(String::new(), "Name".into(), event)
        };
        let (view, cx) = cx.add_window_view(|_, cx| TextInput::new(&node, Default::default(), cx));
        cx.update(|window, cx| window.focus(&view.read(cx).focus_handle));
        cx.update(|window, cx| {
            view.update(cx, |input, cx| {
                input.replace_text_in_range(None, "first\r\nsecond\u{2028}third", window, cx);
            })
        });
        assert_eq!(
            received.try_iter().last().unwrap(),
            if multiline {
                "first\nsecond\nthird"
            } else {
                "first second third"
            }
        );
        cx.simulate_keystrokes("enter");
        let value = view.read_with(cx, |input, _| input.content.to_string());
        assert_eq!(value.ends_with('\n'), multiline);
        cx.update(|window, cx| {
            view.update(cx, |input, cx| {
                input.replace_text_in_range(
                    None,
                    &"Rust text wraps naturally into multiple visual rows. ".repeat(500),
                    window,
                    cx,
                );
            })
        });
        cx.run_until_parked();
        view.read_with(cx, |input, _| {
            let layout = input.last_layout.as_ref().unwrap();
            let caret = layout.position(input.cursor_offset());
            if multiline {
                assert!(layout.size().height > layout.line_height);
                assert!(
                    input.vertical.offset() > 0.0,
                    "size {:?}, caret {:?}, viewport {} focused {}",
                    layout.size(),
                    caret,
                    input.vertical.width(),
                    input.vertical.offset()
                );
                assert!(caret.y >= px(input.vertical.offset()));
            } else {
                assert_eq!(layout.shaped.layout.len(), 1);
            }
        });
    }
}
