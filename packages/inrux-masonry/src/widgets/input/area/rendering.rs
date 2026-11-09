// Copyright 2018 the Xilem Authors and the Druid Authors
// SPDX-License-Identifier: Apache-2.0
// Adapted from Masonry 0.4.0; private horizontal viewport support.
use super::*;
impl<const EDITABLE: bool> TextArea<EDITABLE> {
    /// Shape using native metrics and reveal the active selection focus.
    pub(super) fn handle_layout(
        &mut self,
        ctx: &mut LayoutCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        bc: &BoxConstraints,
    ) -> Size {
        let available_width = if bc.max().width.is_finite() {
            Some((bc.max().width) as f32)
        } else {
            None
        };
        let max_advance = if self.word_wrap {
            available_width
        } else {
            None
        };
        if self.last_available_width != available_width && self.word_wrap {
            self.editor.set_width(max_advance);
        }
        self.last_available_width = available_width;

        let new_generation = self.editor.generation();
        if new_generation != self.rendered_generation {
            self.rendered_generation = new_generation;
        }

        if ctx.fonts_changed() {
            // Refresh native shaping after a font-system change.
            // We know that the lifecycle of dirty tracking in Parley's
            // editor will need to change eventually anyway...
            let _ = self.editor.edit_styles();
        }
        let (_, lctx) = ctx.text_contexts();
        let layout = self.editor.layout(&mut self.fonts, lctx);
        let extent = layout.full_width();
        let text_height = layout.height();
        let text_width = max_advance.unwrap_or(extent);
        let text_size = Size::new(text_width.into(), layout.height().into());
        let caret = self.editor.try_layout().map_or(0.0, |layout| {
            self.editor.raw_selection().focus().geometry(layout, 1.5).x0 as f32
        });
        self.viewport
            .update(bc.max().width as f32, extent, caret, ctx.is_focus_target());
        let (top, bottom) = self.editor.try_layout().map_or((0.0, 0.0), |layout| {
            let caret =
                inrux_text::caret_rect(layout, self.editor.raw_selection().focus().index(), 1.5);
            (caret[1], caret[1] + caret[3])
        });
        if self.word_wrap {
            self.vertical.update_range(
                bc.max().height as f32,
                text_height,
                top,
                bottom,
                ctx.is_focus_target(),
            );
        }
        ctx.set_ime_area(self.ime_area());

        let area_size = Size {
            height: text_size.height,
            width: text_size.width,
        };
        let size = bc.constrain(area_size);
        ctx.set_clip_path(size.to_rect());
        size
    }

    /// Translate glyphs, selection and caret together inside the native clip.
    pub(super) fn handle_paint(
        &mut self,
        ctx: &mut PaintCtx<'_>,
        props: &PropertiesRef<'_>,
        scene: &mut Scene,
    ) {
        if self.editor.try_layout().is_none() {
            // Rebuild an invalidated native shaping cache before using its glyph geometry.
            let (_, lctx) = ctx.text_contexts();
            self.editor.refresh_layout(&mut self.fonts, lctx);
        }
        let Some(layout) = self.editor.try_layout() else {
            return;
        };

        if ctx.is_focus_target() {
            let caret_color = props.get::<CaretColor>().color;
            let selection_color = if ctx.is_window_focused() {
                props.get::<SelectionColor>().color
            } else {
                props.get::<UnfocusedSelectionColor>().0.color
            };
            for rect in
                inrux_text::selection_rects(layout, self.editor.raw_selection().text_range())
            {
                scene.fill(
                    Fill::NonZero,
                    Affine::translate((
                        -f64::from(self.viewport.offset()),
                        -f64::from(self.vertical.offset()),
                    )),
                    selection_color,
                    None,
                    &Rect::new(
                        rect[0].into(),
                        rect[1].into(),
                        (rect[0] + rect[2]).into(),
                        (rect[1] + rect[3]).into(),
                    ),
                );
            }
            if let Some(_cursor) = self.editor.cursor_geometry(1.5)
                && self.anim_cursor_visible
                && ctx.is_window_focused()
            {
                let caret = inrux_text::caret_rect(
                    layout,
                    self.editor.raw_selection().focus().index(),
                    1.5,
                );
                scene.fill(
                    Fill::NonZero,
                    Affine::translate((
                        -f64::from(self.viewport.offset()),
                        -f64::from(self.vertical.offset()),
                    )),
                    caret_color,
                    None,
                    &Rect::new(
                        caret[0].into(),
                        caret[1].into(),
                        (caret[0] + caret[2]).into(),
                        (caret[1] + caret[3]).into(),
                    ),
                );
            };
        }

        let text_color = if ctx.is_disabled() {
            &props.get::<DisabledContentColor>().0
        } else {
            props.get::<ContentColor>()
        };

        crate::widgets::text_image::paint_layout(
            scene,
            Affine::translate((
                -f64::from(self.viewport.offset()),
                -f64::from(self.vertical.offset()),
            )),
            layout,
            ctx.get_scale_factor() as f32,
            text_color.color,
            Some([
                self.viewport.offset(),
                self.vertical.offset(),
                ctx.size().width as f32,
                ctx.size().height as f32,
            ]),
        );
        if let Some(range) = self.editor.raw_compose().clone() {
            for rect in inrux_text::underline_rects(layout, range) {
                scene.fill(
                    Fill::NonZero,
                    Affine::translate((
                        -f64::from(self.viewport.offset()),
                        -f64::from(self.vertical.offset()),
                    )),
                    text_color.color,
                    None,
                    &Rect::new(
                        rect[0].into(),
                        rect[1].into(),
                        (rect[0] + rect[2]).into(),
                        (rect[1] + rect[3]).into(),
                    ),
                );
            }
        }
    }

    /// Expose native semantic text geometry with the same horizontal translation.
    pub(super) fn handle_accessibility(
        &mut self,
        ctx: &mut AccessCtx<'_>,
        _props: &PropertiesRef<'_>,
        node: &mut Node,
    ) {
        if !EDITABLE {
            node.set_read_only();
        }
        let updated = self.editor.try_accessibility(
            ctx.tree_update(),
            node,
            || NodeId::from(WidgetId::next()),
            -f64::from(self.viewport.offset()),
            -f64::from(self.vertical.offset()),
        );

        let Some(()) = updated else {
            // We always perform layout before accessibility, so this panic should be unreachable.
            debug_panic!("Could not generate accessibility nodes for text area");
            return;
        };
    }
}
