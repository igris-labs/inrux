//! Native input widget behavior.
use crate::editing::Editor;
use crate::paint::style_widgets;
use crate::widgets::Visual;
use eframe::egui;
use inrux_components::{Node, NodeKind as Kind};
use inrux_core::DispatchError;

/// Paint and handle one native input without running application logic on the UI thread.
pub(crate) fn paint(
    node: &Node,
    ui: &mut egui::Ui,
    visual: &Visual<'_>,
) -> Result<(), DispatchError> {
    let Kind::Input {
        value,
        placeholder,
        event,
        multiline,
    } = &node.kind
    else {
        return Ok(());
    };
    let rect = visual.rect;
    let path = visual.path;
    let style = visual.style;
    let radius = visual.radius;
    let ink = visual.ink;
    let font = visual.font.clone();
    let mut layouter = |ui: &egui::Ui, buffer: &dyn egui::TextBuffer, wrap: f32| {
        crate::typography::layout(
            ui,
            buffer.as_str(),
            font.clone(),
            ink,
            if *multiline { wrap } else { f32::INFINITY },
        )
    };
    let mut emit_result = Ok(());
    let mut editor = ui
        .ctx()
        .data_mut(|data| data.get_temp::<Editor>(path))
        .unwrap_or_else(|| Editor::new(value, event.applied()));
    editor.reconcile(value, event.applied());
    let enabled = ui.is_enabled() && !node.disabled;
    if !enabled {
        ui.memory_mut(|memory| memory.surrender_focus(path));
    }
    if ui.memory(|memory| memory.has_focus(path)) {
        // Normalize before native editing computes cursor/selection positions.
        ui.input_mut(|input| {
            for event in &mut input.events {
                normalize_event(event, *multiline);
            }
        });
    }
    let previous = editor.value.clone();
    let response = ui
        .scope_builder(egui::UiBuilder::new().id_salt(path).max_rect(rect), |ui| {
            style_widgets(ui, style, radius);
            if node.disabled {
                ui.disable();
            }
            let margin = style
                .padding
                .map_or(egui::vec2(12.0, 8.0), |p| egui::vec2(p, p));
            let inner = rect.shrink2(margin.min(rect.size() * 0.5));
            let field = ui.interact(rect, path.with("input-frame"), egui::Sense::click());
            if field.clicked() {
                ui.memory_mut(|memory| memory.request_focus(path));
            }
            let visuals = ui.visuals().widgets.inactive;
            ui.painter().rect(
                rect,
                radius,
                ui.visuals().text_edit_bg_color(),
                if ui.memory(|memory| memory.has_focus(path)) && !node.has_focus_decoration() {
                    let c = style
                        .accent_color
                        .unwrap_or(inrux_adapter::controls::ACCENT);
                    egui::Stroke::new(
                        1.0_f32,
                        egui::Color32::from_rgba_unmultiplied(c.0, c.1, c.2, c.3),
                    )
                } else {
                    visuals.bg_stroke
                },
                egui::StrokeKind::Inside,
            );
            ui.scope_builder(egui::UiBuilder::new().max_rect(inner), |ui| {
                ui.set_clip_rect(ui.clip_rect().intersect(inner));
                let salt = path.with("text-scroll");
                let id = ui.make_persistent_id(egui::IdSalt::new(salt));
                let extent =
                    crate::typography::layout(ui, &editor.value, font.clone(), ink, f32::INFINITY)
                        .size()
                        .x;
                let pan = (!*multiline)
                    .then(|| pan_text(ui, inner, id, extent))
                    .flatten();
                let area = (if *multiline {
                    egui::ScrollArea::vertical()
                } else {
                    egui::ScrollArea::horizontal()
                })
                .scroll_source(egui::scroll_area::ScrollSource {
                    mouse_wheel: *multiline,
                    ..Default::default()
                })
                .id_salt(path.with("text-scroll"))
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                .auto_shrink([false, false])
                .animated(false)
                .max_height(inner.height());
                let area = if let Some(offset) = pan {
                    area.horizontal_scroll_offset(offset)
                } else {
                    area
                };
                let scrolling = area.show(ui, |ui| {
                    // add_sized centers oversized widgets, shifting the text/caret by
                    // half its overflow. Let the native editor allocate its full content.
                    let navigation = ui.input(|input| {
                        input.events.iter().any(|event| {
                            matches!(
                                event,
                                egui::Event::Key {
                                    key: egui::Key::Home
                                        | egui::Key::End
                                        | egui::Key::ArrowLeft
                                        | egui::Key::ArrowRight
                                        | egui::Key::ArrowUp
                                        | egui::Key::ArrowDown,
                                    pressed: true,
                                    ..
                                }
                            )
                        })
                    });
                    let edit = if *multiline {
                        egui::TextEdit::multiline(&mut editor.value)
                    } else {
                        egui::TextEdit::singleline(&mut editor.value)
                    };
                    let output = edit
                        .interactive(enabled)
                        .id(path)
                        .frame(egui::Frame::NONE)
                        .clip_text(false)
                        .desired_width(inner.width())
                        .margin(egui::Vec2::ZERO)
                        .font(font.clone())
                        .layouter(&mut layouter)
                        .text_color(ink)
                        .align(if *multiline {
                            egui::Align2::LEFT_TOP
                        } else {
                            egui::Align2::LEFT_CENTER
                        })
                        .min_size(inner.size())
                        .show(ui);
                    let (display, display_ink) = if editor.value.is_empty() {
                        let ink = inrux_adapter::control_parts::placeholder(
                            node,
                            ui.rect_contains_pointer(rect),
                            output.response.has_focus(),
                        );
                        (
                            placeholder.as_str(),
                            egui::Color32::from_rgba_unmultiplied(ink.0, ink.1, ink.2, ink.3),
                        )
                    } else {
                        (editor.value.as_str(), ink)
                    };
                    if editor.value.is_empty() {
                        let placeholder_galley = crate::typography::layout(
                            ui,
                            display,
                            font.clone(),
                            display_ink,
                            if *multiline {
                                inner.width()
                            } else {
                                f32::INFINITY
                            },
                        );
                        ui.painter()
                            .galley(output.galley_pos, placeholder_galley, display_ink);
                    }
                    crate::typography::paint(
                        ui,
                        display,
                        &font,
                        display_ink,
                        if *multiline {
                            inner.width()
                        } else {
                            f32::INFINITY
                        },
                        output.galley_pos,
                    );
                    if output.response.has_focus()
                        && (navigation || output.response.gained_focus())
                        && let Some(cursor) = output.cursor_range
                    {
                        let caret = output
                            .galley
                            .pos_from_cursor(cursor.primary)
                            .translate(output.galley_pos.to_vec2());
                        ui.scroll_to_rect(caret, None);
                    }
                    output.response
                });
                scrolling.inner
            })
            .inner
        })
        .inner;
    if enabled && response.changed() {
        let revision = editor.edited();
        emit_result = event.emit(revision, editor.value.clone());
        if emit_result.is_err() {
            // Rejected edits must not look like accepted model changes.
            editor.reject(previous);
        }
    }
    ui.ctx().data_mut(|data| data.insert_temp(path, editor));
    emit_result
}

/// Pan only the horizontal gesture; vertical wheel input must reach the containing viewport.
fn pan_text(ui: &mut egui::Ui, rect: egui::Rect, id: egui::Id, extent: f32) -> Option<f32> {
    if !ui.is_enabled() || !ui.rect_contains_pointer(rect) {
        return None;
    }
    let delta = ui.input(|input| input.smooth_scroll_delta.x);
    if delta == 0.0 {
        return None;
    }
    let state = egui::scroll_area::State::load(ui.ctx(), id).unwrap_or_default();
    let current = state.clamped_offset().x;
    let offset = (current - delta).clamp(0.0, (extent - rect.width()).max(0.0));
    if offset == current {
        return None;
    }
    ui.input_mut(|input| input.smooth_scroll_delta.x -= current - offset);
    ui.ctx().request_repaint();
    Some(offset)
}

/// Keep IME character ranges aligned when normalization collapses CRLF pairs.
fn normalize_event(event: &mut egui::Event, multiline: bool) {
    if let egui::Event::Ime(egui::ImeEvent::Preedit {
        text,
        active_range_chars,
    }) = event
    {
        let normalized = inrux_components::normalize_text(text, multiline);
        if let std::borrow::Cow::Owned(normalized) = normalized {
            if let Some(range) = active_range_chars {
                let mapped = |index| {
                    let bytes = text.chars().take(index).map(char::len_utf8).sum::<usize>();
                    inrux_components::normalize_text(&text[..bytes], multiline)
                        .chars()
                        .count()
                };
                *range = mapped(range.start)..mapped(range.end);
            }
            *text = normalized;
        }
    } else if let egui::Event::Paste(text)
    | egui::Event::Text(text)
    | egui::Event::Ime(egui::ImeEvent::Commit(text)) = event
    {
        *text = inrux_components::normalize_text(text, multiline).into_owned();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preedit_selection_tracks_normalized_unicode_characters() {
        let mut event = egui::Event::Ime(egui::ImeEvent::Preedit {
            text: "é\r\n🦀".into(),
            active_range_chars: Some(3..4),
        });
        normalize_event(&mut event, false);
        assert_eq!(
            event,
            egui::Event::Ime(egui::ImeEvent::Preedit {
                text: "é 🦀".into(),
                active_range_chars: Some(2..3),
            })
        );
    }
}
