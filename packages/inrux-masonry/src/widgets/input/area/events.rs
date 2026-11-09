// Copyright 2018 the Xilem Authors and the Druid Authors
// SPDX-License-Identifier: Apache-2.0
// Adapted from Masonry 0.4.0; private horizontal viewport support.
use super::*;
impl<const EDITABLE: bool> TextArea<EDITABLE> {
    /// Preserve native caret blink scheduling and its idle timeout.
    pub(super) fn handle_on_anim_frame(
        &mut self,
        ctx: &mut UpdateCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        interval: u64,
    ) {
        /// The time for a complete blink cycle, in milliseconds.
        /// For the first half (i.e. currently 0.5s) the cursor is shown, and for the second half,
        /// it is hidden.
        const CURSOR_BLINK_TIME: u64 = 1000;

        /// The timeout, in milliseconds, after which the cursor will stop blinking (i.e. stay
        /// solid).
        const CURSOR_BLINK_TIMEOUT: u64 = 10_000; // 10 seconds

        // Preserve the pinned native editor's bounded caret timing.

        if ctx.is_window_focused() && ctx.is_focus_target() {
            if self.anim_elapsed < CURSOR_BLINK_TIMEOUT {
                let interval_ms = interval / 1_000_000; // ns to ms
                self.anim_prev_interval += interval_ms;
                self.anim_elapsed += interval_ms;

                if self.anim_prev_interval >= CURSOR_BLINK_TIME {
                    self.anim_prev_interval = self.anim_prev_interval.rem_euclid(CURSOR_BLINK_TIME);
                }

                ctx.request_anim_frame();

                // Request paint only if changed.
                if self.anim_prev_interval < CURSOR_BLINK_TIME / 2 && !self.anim_cursor_visible {
                    self.anim_cursor_visible = true;
                    ctx.request_paint_only();
                } else if self.anim_prev_interval >= CURSOR_BLINK_TIME / 2
                    && self.anim_cursor_visible
                {
                    self.anim_cursor_visible = false;
                    ctx.request_paint_only();
                }
            } else if !self.anim_cursor_visible {
                // Request paint only if changed.
                self.anim_cursor_visible = true;
                ctx.request_paint_only();
            }
        }
    }

    /// Pan horizontal gestures and translate native selection coordinates.
    pub(super) fn handle_on_pointer_event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        event: &PointerEvent,
    ) {
        if self.editor.is_composing() {
            return;
        }

        if let PointerEvent::Scroll(event) = event {
            let horizontal = match event.delta {
                masonry::core::ScrollDelta::PixelDelta(delta) => {
                    if event.state.modifiers.shift() && delta.x == 0.0 {
                        -delta.y / ctx.get_scale_factor()
                    } else {
                        -delta.x / ctx.get_scale_factor()
                    }
                }
                masonry::core::ScrollDelta::LineDelta(x, y) => {
                    f64::from(if event.state.modifiers.shift() && x == 0.0 {
                        -y
                    } else {
                        -x
                    }) * f64::from(inrux_adapter::services::scroll::LINE_STEP)
                }
                _ => 0.0,
            };
            let vertical = if self.word_wrap && !event.state.modifiers.shift() {
                match event.delta {
                    masonry::core::ScrollDelta::PixelDelta(delta) => {
                        -delta.y / ctx.get_scale_factor()
                    }
                    masonry::core::ScrollDelta::LineDelta(_, y) => {
                        -f64::from(y) * f64::from(inrux_adapter::services::scroll::LINE_STEP)
                    }
                    _ => 0.0,
                }
            } else {
                0.0
            };
            if !ctx.is_disabled()
                && (self.viewport.pan(horizontal as f32) | self.vertical.pan(vertical as f32))
            {
                ctx.set_handled();
                ctx.request_layout();
                ctx.request_render();
            }
            return;
        }
        match event {
            PointerEvent::Down(PointerButtonEvent {
                button: None | Some(PointerButton::Primary),
                state,
                ..
            }) => {
                let cursor_pos = ctx.local_position(state.position)
                    + masonry::kurbo::Vec2::new(
                        self.viewport.offset().into(),
                        self.vertical.offset().into(),
                    );
                let (_, lctx) = ctx.text_contexts();
                let fctx = &mut self.fonts;
                let mut drv = self.editor.driver(fctx, lctx);
                match state.count {
                    2 => drv.select_word_at_point(cursor_pos.x as f32, cursor_pos.y as f32),
                    3 => {
                        drv.select_hard_line_at_point(cursor_pos.x as f32, cursor_pos.y as f32);
                    }
                    _ => {
                        if state.modifiers.shift() {
                            drv.shift_click_extension(cursor_pos.x as f32, cursor_pos.y as f32);
                        } else {
                            drv.move_to_point(cursor_pos.x as f32, cursor_pos.y as f32);
                        }
                    }
                }
                let new_generation = self.editor.generation();
                if new_generation != self.rendered_generation {
                    ctx.request_layout();
                    ctx.request_render();
                    ctx.set_ime_area(self.ime_area());
                    self.rendered_generation = new_generation;
                }
                ctx.request_focus();
                ctx.capture_pointer();
            }
            PointerEvent::Move(PointerUpdate { current, .. }) if ctx.is_active() => {
                let cursor_pos = ctx.local_position(current.position)
                    + masonry::kurbo::Vec2::new(
                        self.viewport.offset().into(),
                        self.vertical.offset().into(),
                    );
                let (_, lctx) = ctx.text_contexts();
                let fctx = &mut self.fonts;
                self.editor
                    .driver(fctx, lctx)
                    .extend_selection_to_point(cursor_pos.x as f32, cursor_pos.y as f32);
                let new_generation = self.editor.generation();
                if new_generation != self.rendered_generation {
                    ctx.request_layout();
                    ctx.request_render();
                    ctx.set_ime_area(self.ime_area());
                    self.rendered_generation = new_generation;
                }
            }
            _ => {}
        }
    }

    /// Dispatch native keyboard, clipboard and IME edits without replacing editing semantics.
    pub(super) fn handle_on_text_event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        event: &TextEvent,
    ) {
        // Reset the blink animation.
        self.anim_prev_interval = 0;
        self.anim_elapsed = 0;
        ctx.request_anim_frame();

        match event {
            TextEvent::Keyboard(key_event) => {
                if key_event.state != KeyState::Down || self.editor.is_composing() {
                    return;
                }
                if matches!(
                    key_event.key,
                    Key::Named(
                        NamedKey::Home
                            | NamedKey::End
                            | NamedKey::ArrowLeft
                            | NamedKey::ArrowRight
                            | NamedKey::ArrowUp
                            | NamedKey::ArrowDown
                    )
                ) {
                    self.viewport.request_reveal();
                    self.vertical.request_reveal();
                    ctx.request_layout();
                    ctx.request_render();
                }
                let (shift, action_mod) = (
                    key_event.modifiers.shift(),
                    if cfg!(target_os = "macos") {
                        key_event.modifiers.meta()
                    } else {
                        key_event.modifiers.ctrl()
                    },
                );
                let (_, lctx) = ctx.text_contexts();
                let fctx = &mut self.fonts;
                // Whether the text was changed.
                let mut edited = false;
                match &key_event.key {
                    // Cut
                    #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
                    Key::Character(x)
                        if EDITABLE && action_mod && x.as_str().eq_ignore_ascii_case("x") =>
                    {
                        if let Some(text) = self.editor.selected_text()
                            && !text.is_empty()
                        {
                            let text = text.to_string();
                            self.editor.driver(fctx, lctx).delete_selection();
                            edited = true;
                            ctx.set_clipboard(text);
                        }
                    }
                    // Copy
                    #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
                    Key::Character(c) if action_mod && c.as_str().eq_ignore_ascii_case("c") => {
                        if let Some(text) = self.editor.selected_text()
                            && !text.is_empty()
                        {
                            ctx.set_clipboard(text.to_string());
                        }
                    }
                    Key::Character(a) if action_mod && a.as_str().eq_ignore_ascii_case("a") => {
                        let mut drv = self.editor.driver(fctx, lctx);

                        if shift {
                            drv.collapse_selection();
                        } else {
                            drv.select_all();
                        }
                    }
                    Key::Named(NamedKey::ArrowLeft) => {
                        let mut drv = self.editor.driver(fctx, lctx);
                        if action_mod {
                            if shift {
                                drv.select_word_left();
                            } else {
                                drv.move_word_left();
                            }
                        } else if shift {
                            drv.select_left();
                        } else {
                            drv.move_left();
                        }
                    }
                    Key::Named(NamedKey::ArrowRight) => {
                        let mut drv = self.editor.driver(fctx, lctx);
                        if action_mod {
                            if shift {
                                drv.select_word_right();
                            } else {
                                drv.move_word_right();
                            }
                        } else if shift {
                            drv.select_right();
                        } else {
                            drv.move_right();
                        }
                    }
                    Key::Named(NamedKey::ArrowUp) => {
                        let mut drv = self.editor.driver(fctx, lctx);
                        if cfg!(target_os = "macos") && action_mod {
                            if shift {
                                drv.select_to_text_start();
                            } else {
                                drv.move_to_text_start();
                            }
                        } else if shift {
                            drv.select_up();
                        } else {
                            drv.move_up();
                        }
                    }
                    Key::Named(NamedKey::ArrowDown) => {
                        let mut drv = self.editor.driver(fctx, lctx);
                        if cfg!(target_os = "macos") && action_mod {
                            if shift {
                                drv.select_to_text_end();
                            } else {
                                drv.move_to_text_end();
                            }
                        } else if shift {
                            drv.select_down();
                        } else {
                            drv.move_down();
                        }
                    }
                    Key::Named(NamedKey::Home) => {
                        let mut drv = self.editor.driver(fctx, lctx);
                        if action_mod {
                            if shift {
                                drv.select_to_text_start();
                            } else {
                                drv.move_to_text_start();
                            }
                        } else if shift {
                            drv.select_to_line_start();
                        } else {
                            drv.move_to_line_start();
                        }
                    }
                    Key::Named(NamedKey::End) => {
                        let mut drv = self.editor.driver(fctx, lctx);
                        if action_mod {
                            if shift {
                                drv.select_to_text_end();
                            } else {
                                drv.move_to_text_end();
                            }
                        } else if shift {
                            drv.select_to_line_end();
                        } else {
                            drv.move_to_line_end();
                        }
                    }
                    Key::Named(NamedKey::Delete) if EDITABLE => {
                        let mut drv = self.editor.driver(fctx, lctx);
                        if action_mod {
                            drv.delete_word();
                        } else {
                            drv.delete();
                        }

                        edited = true;
                    }
                    Key::Named(NamedKey::Backspace) if EDITABLE => {
                        let mut drv = self.editor.driver(fctx, lctx);
                        if action_mod {
                            drv.backdelete_word();
                        } else {
                            drv.backdelete();
                        }

                        edited = true;
                    }
                    Key::Character(sp) if EDITABLE && sp.as_str() == " " => {
                        self.editor
                            .driver(fctx, lctx)
                            .insert_or_replace_selection(" ");
                        edited = true;
                    }
                    Key::Named(NamedKey::Enter) => {
                        let insert_newline = match self.insert_newline {
                            InsertNewline::OnEnter => true,
                            InsertNewline::OnShiftEnter => shift,
                            InsertNewline::Never => false,
                        };
                        if insert_newline {
                            let (_, lctx) = ctx.text_contexts();
                            let fctx = &mut self.fonts;
                            self.editor
                                .driver(fctx, lctx)
                                .insert_or_replace_selection("\n");
                            edited = true;
                        } else {
                            ctx.submit_action::<TextAction>(TextAction::Entered(
                                self.text().to_string(),
                            ));
                        }
                    }

                    Key::Named(NamedKey::Tab) => {
                        // Intentionally do nothing so that tabbing from a TextInput/Prose works.
                        // Note that this doesn't allow input of the tab character; we need to be more clever here at some point
                        return;
                    }
                    Key::Character(text) if EDITABLE => {
                        self.editor.driver(fctx, lctx).insert_or_replace_selection(
                            &inrux_components::normalize_text(text, self.word_wrap),
                        );
                        edited = true;
                    }
                    _ => {
                        // Do nothing, don't set as handled.
                        return;
                    }
                }
                ctx.set_handled();
                let new_generation = self.editor.generation();
                if new_generation != self.rendered_generation {
                    if edited {
                        ctx.submit_action::<TextAction>(TextAction::Changed(
                            self.text().into_iter().collect(),
                        ));
                        ctx.request_layout();
                    } else {
                        ctx.request_layout();
                        ctx.request_render();
                        ctx.set_ime_area(self.ime_area());
                    }
                    self.rendered_generation = new_generation;
                }
            }

            TextEvent::WindowFocusChange(_) => {
                // To use a different selection color when unfocused.
                ctx.request_paint_only();
            }

            TextEvent::Ime(e) => {
                let (_, lctx) = ctx.text_contexts();
                let fctx = &mut self.fonts;

                // Whether the returned text has changed.
                // We don't send a TextChanged when the preedit changes
                let mut edited = false;
                match e {
                    Ime::Disabled => {
                        self.editor.driver(fctx, lctx).clear_compose();
                    }
                    Ime::Preedit(text, cursor) => {
                        if text.is_empty() {
                            self.editor.driver(fctx, lctx).clear_compose();
                        } else {
                            let normalized = inrux_components::normalize_text(text, self.word_wrap);
                            let selection = cursor.map(|(start, end)| {
                                let boundary = |n: usize| {
                                    let mut n = n.min(text.len());
                                    while !text.is_char_boundary(n) {
                                        n -= 1;
                                    }
                                    inrux_components::normalize_text(&text[..n], self.word_wrap)
                                        .len()
                                };
                                (boundary(start), boundary(end))
                            });
                            self.editor
                                .driver(fctx, lctx)
                                .set_compose(&normalized, selection);
                            edited = true;
                        }
                    }
                    Ime::Commit(text) => {
                        self.editor.driver(fctx, lctx).insert_or_replace_selection(
                            &inrux_components::normalize_text(text, self.word_wrap),
                        );
                        edited = true;
                    }
                    Ime::Enabled => {}
                }

                ctx.set_handled();
                if edited {
                    let text = self.text().into_iter().collect();
                    ctx.submit_action::<TextAction>(TextAction::Changed(text));
                }

                let new_generation = self.editor.generation();
                if new_generation != self.rendered_generation {
                    ctx.request_layout();
                    self.rendered_generation = new_generation;
                }
            }

            TextEvent::ClipboardPaste(text) => {
                if EDITABLE {
                    let (_, lctx) = ctx.text_contexts();
                    let fctx = &mut self.fonts;
                    self.editor.driver(fctx, lctx).insert_or_replace_selection(
                        &inrux_components::normalize_text(text, self.word_wrap),
                    );

                    let new_generation = self.editor.generation();
                    if new_generation != self.rendered_generation {
                        ctx.submit_action::<TextAction>(TextAction::Changed(
                            self.text().into_iter().collect(),
                        ));
                        ctx.request_layout();
                        self.rendered_generation = new_generation;
                    }
                }
            }
        }
    }

    /// Preserve native accessible selection actions and refresh caret geometry.
    pub(super) fn handle_on_access_event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        event: &AccessEvent,
    ) {
        if event.action == accesskit::Action::SetTextSelection {
            if self.editor.is_composing() {
                return;
            }

            if let Some(accesskit::ActionData::SetTextSelection(selection)) = &event.data {
                let (_, lctx) = ctx.text_contexts();
                let fctx = &mut self.fonts;
                self.editor
                    .driver(fctx, lctx)
                    .select_from_accesskit(selection);
                let new_generation = self.editor.generation();
                if new_generation != self.rendered_generation {
                    ctx.request_layout();
                    ctx.request_render();
                    ctx.set_ime_area(self.ime_area());
                    self.rendered_generation = new_generation;
                }
            }
        }
    }
}
