// Adapted from gpui 0.2.2 examples/input.rs, licensed Apache-2.0.
//! Native committed/preedit text, selection and IME geometry.
use super::*;
impl EntityInputHandler for TextInput {
    /// Bridge native text for range using UTF-16 platform ranges.
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        actual_range: &mut Option<Range<usize>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<String> {
        let range = self.range_from_utf16(&range_utf16);
        actual_range.replace(self.range_to_utf16(&range));
        Some(self.content[range].to_string())
    }

    /// Bridge native selected text range using UTF-16 platform ranges.
    fn selected_text_range(
        &mut self,
        ignore_disabled_input: bool,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        if self.disabled && !ignore_disabled_input {
            return None;
        }
        Some(UTF16Selection {
            range: self.range_to_utf16(&self.selected_range),
            reversed: self.selection_reversed,
        })
    }

    /// Bridge native marked text range using UTF-16 platform ranges.
    fn marked_text_range(
        &self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Range<usize>> {
        self.marked_range
            .as_ref()
            .map(|range| self.range_to_utf16(range))
    }

    /// Bridge native unmark text using UTF-16 platform ranges.
    fn unmark_text(&mut self, _window: &mut Window, _cx: &mut Context<Self>) {
        self.marked_range = None;
    }

    /// Bridge native replace text in range using UTF-16 platform ranges.
    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.disabled {
            return;
        }
        let normalized = inrux_components::normalize_text(new_text, self.multiline);
        let new_text = normalized.as_ref();
        let range = range_utf16
            .as_ref()
            .map(|range_utf16| self.range_from_utf16(range_utf16))
            .or(self.marked_range.clone())
            .unwrap_or(self.selected_range.clone());

        self.content =
            (self.content[0..range.start].to_owned() + new_text + &self.content[range.end..])
                .into();
        self.selected_range = range.start + new_text.len()..range.start + new_text.len();
        self.marked_range.take();
        self.publish(cx);
        cx.notify();
    }

    /// Bridge native replace and mark text in range using UTF-16 platform ranges.
    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selected_range_utf16: Option<Range<usize>>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.disabled {
            return;
        }
        let original_text = new_text;
        let normalized = inrux_components::normalize_text(new_text, self.multiline);
        let new_text = normalized.as_ref();
        let range = range_utf16
            .as_ref()
            .map(|range_utf16| self.range_from_utf16(range_utf16))
            .or(self.marked_range.clone())
            .unwrap_or(self.selected_range.clone());

        self.content =
            (self.content[0..range.start].to_owned() + new_text + &self.content[range.end..])
                .into();
        if !new_text.is_empty() {
            self.marked_range = Some(range.start..range.start + new_text.len());
        } else {
            self.marked_range = None;
        }
        self.selected_range = new_selected_range_utf16
            .as_ref()
            .map(|selection| {
                // Platform selections are relative to the replacement, not the full buffer.
                let byte_offset = |offset: usize| {
                    let mut utf16 = 0;
                    for (byte, ch) in original_text.char_indices() {
                        if utf16 >= offset {
                            return inrux_components::normalize_text(
                                &original_text[..byte],
                                self.multiline,
                            )
                            .len();
                        }
                        utf16 += ch.len_utf16();
                    }
                    new_text.len()
                };
                range.start + byte_offset(selection.start)..range.start + byte_offset(selection.end)
            })
            .unwrap_or_else(|| range.start + new_text.len()..range.start + new_text.len());

        self.restart_caret();
        cx.notify();
    }

    /// Bridge native bounds for range using UTF-16 platform ranges.
    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        bounds: Bounds<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let last_layout = self.last_layout.as_ref()?;
        let bounds = self.last_bounds.unwrap_or(bounds);
        let range = self.range_from_utf16(&range_utf16);
        let start = last_layout.position(range.start);
        let end = last_layout.position(range.end);
        Some(Bounds::new(
            bounds.origin + start,
            size(
                if start.y == end.y {
                    (end.x - start.x).max(px(2.0))
                } else {
                    px(2.0)
                },
                last_layout.line_height,
            ),
        ))
    }

    /// Bridge native character index for point using UTF-16 platform ranges.
    fn character_index_for_point(
        &mut self,
        point: gpui::Point<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        let bounds = self.last_bounds?;
        let last_layout = self.last_layout.as_ref()?;

        let utf8_index = last_layout.index(point - bounds.origin);
        Some(self.offset_to_utf16(utf8_index))
    }
}
