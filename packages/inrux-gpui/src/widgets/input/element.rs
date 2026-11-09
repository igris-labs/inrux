// Adapted from gpui 0.2.2 examples/input.rs, licensed Apache-2.0.
//! Text, selection, caret, clipping and input-service registration.
use super::*;
pub(super) struct TextElement {
    pub(super) input: Entity<TextInput>,
}

pub(super) struct PrepaintState {
    line: Option<super::layout::TextLayout>,
    cursor: Option<PaintQuad>,
    selection: Vec<PaintQuad>,
}

impl IntoElement for TextElement {
    type Element = Self;

    /// Provide native editor into element inside shared framework geometry.
    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for TextElement {
    type RequestLayoutState = ();
    type PrepaintState = PrepaintState;

    /// Provide native editor id inside shared framework geometry.
    fn id(&self) -> Option<ElementId> {
        None
    }

    /// Provide native editor source location inside shared framework geometry.
    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    /// Provide native editor request layout inside shared framework geometry.
    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = if self.input.read(cx).multiline {
            relative(1.).into()
        } else {
            window.line_height().into()
        };
        (window.request_layout(style, [], cx), ())
    }

    /// Provide native editor prepaint inside shared framework geometry.
    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let input = self.input.read(cx);
        let content = input.content.clone();
        let selected_range = input.selected_range.clone();
        let cursor = input.cursor_offset();
        let style = window.text_style();

        let (display_text, text_color) = if content.is_empty() {
            (
                input.placeholder.clone(),
                crate::widgets::surface::color(input.placeholder_ink).into(),
            )
        } else {
            (content, style.color)
        };

        let ink = text_color.to_rgb();
        let line = super::layout::TextLayout {
            shaped: inrux_text::shape(
                display_text.as_ref(),
                input.style.font_size.unwrap_or(15.0),
                input.style.font_family.as_deref(),
                if input.multiline {
                    f32::from(bounds.size.width).max(1.0)
                } else {
                    f32::INFINITY
                },
            ),
            line_height: px(input.style.font_size.unwrap_or(15.0) * inrux_text::LINE_HEIGHT),
            ink: [
                (ink.r * 255.0).round() as u8,
                (ink.g * 255.0).round() as u8,
                (ink.b * 255.0).round() as u8,
                (ink.a * 255.0).round() as u8,
            ],
        };
        let cursor_pos = line.position(cursor);
        let focused = input.has_focus(window);
        let extent = if input.content.is_empty() {
            size(px(0.0), px(0.0))
        } else {
            line.size()
        };
        self.input.update(cx, |input, _| {
            input.viewport.update(
                f32::from(bounds.size.width),
                f32::from(extent.width),
                f32::from(cursor_pos.x),
                focused,
            );
            if input.multiline {
                input.vertical.update_range(
                    f32::from(bounds.size.height),
                    f32::from(extent.height),
                    f32::from(cursor_pos.y),
                    f32::from(cursor_pos.y + line.line_height),
                    focused,
                );
            }
        });
        let input = self.input.read(cx);
        let origin =
            bounds.origin - point(px(input.viewport.offset()), px(input.vertical.offset()));
        let selection = line
            .selection(selected_range.clone(), bounds.size.width)
            .into_iter()
            .map(|rect| fill(Bounds::new(origin + rect.origin, rect.size), rgb(0xd0ebe2)))
            .collect();
        let cursor = selected_range.is_empty().then(|| {
            fill(
                Bounds::new(origin + cursor_pos, size(px(2.), line.line_height)),
                rgb(0x1d7062),
            )
        });
        PrepaintState {
            line: Some(line),
            cursor,
            selection,
        }
    }

    /// Provide native editor paint inside shared framework geometry.
    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let focus_handle = self.input.read(cx).focus_handle.clone();
        if !self.input.read(cx).disabled {
            window.handle_input(
                &focus_handle,
                ElementInputHandler::new(bounds, self.input.clone()),
                cx,
            );
        }
        window.with_content_mask(Some(gpui::ContentMask { bounds }), |window| {
            if self.input.read(cx).has_focus(window) && window.is_window_active() {
                for selection in prepaint.selection.drain(..) {
                    window.paint_quad(selection);
                }
            }
            let Some(line) = prepaint.line.take() else {
                return;
            };
            let input = self.input.read(cx);
            let origin =
                bounds.origin - point(px(input.viewport.offset()), px(input.vertical.offset()));
            if let Err(message) =
                crate::widgets::text_image::paint(&line.shaped, origin, line.ink, window)
            {
                *self.input.read(cx).failure.borrow_mut() =
                    Some(inrux_adapter::AdapterError::Native {
                        backend: "gpui",
                        message,
                    });
            }

            if let Some(range) = input.marked_range.clone() {
                let color = crate::widgets::surface::color(
                    input.style.color.unwrap_or(inrux_style::Color(27, 30, 33)),
                );
                for rect in inrux_text::underline_rects(&line.shaped.layout, range) {
                    window.paint_quad(fill(
                        Bounds::new(
                            origin + point(px(rect[0]), px(rect[1])),
                            size(px(rect[2]), px(rect[3])),
                        ),
                        color,
                    ));
                }
            }

            if self.input.read(cx).has_focus(window)
                && window.is_window_active()
                && self.input.read(cx).caret_visible
                && let Some(cursor) = prepaint.cursor.take()
            {
                window.paint_quad(cursor);
            }

            self.input.update(cx, |input, _cx| {
                input.last_layout = Some(line.clone());
                input.last_bounds = Some(Bounds::new(origin, line.size()));
            });
        });
    }
}
