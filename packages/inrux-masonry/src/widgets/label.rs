// Copyright 2019 the Xilem Authors and the Druid Authors
// SPDX-License-Identifier: Apache-2.0

//! Private native label adaptation: shared glyph coverage and fractional logical layout.

use std::any::TypeId;

use masonry::accesskit::{Node, NodeId, Role};
use masonry::core::{HasProperty, NoAction};
use masonry::parley::LayoutAccessibility;
use masonry::vello::Scene;
use masonry::vello::kurbo::{Affine, Point, Size};
use masonry::vello::peniko::BlendMode;
use tracing::{Span, trace_span};

use masonry::core::{
    AccessCtx, ArcStr, BoxConstraints, BrushIndex, ChildrenIds, LayoutCtx, PaintCtx, PropertiesMut,
    PropertiesRef, RegisterCtx, StyleProperty, StyleSet, Update, UpdateCtx, Widget, WidgetId,
    WidgetMut,
};
use masonry::properties::{ContentColor, DisabledContentColor, LineBreaking, Padding};
use masonry::theme::default_text_styles;

use masonry::theme;

/// A widget displaying non-interactive text.
///
/// This is useful for creating interactive widgets which internally
/// need support for displaying text, such as a button.
///
/// You can customize the look of this label with the
/// [`Padding`], [`LineBreaking`], [`ContentColor`] and [`DisabledContentColor`] properties.
///
pub struct Label {
    shaped: std::sync::Arc<inrux_text::Shaped>,
    accessibility: LayoutAccessibility,
    text: ArcStr,
    styles: StyleSet,
    centered_insets: Option<Size>,
    single_line: bool,
    paint_shift: masonry::kurbo::Vec2,
}

// --- MARK: BUILDERS
impl Label {
    /// Create a new label with the given text.
    ///
    // This is written out fully to appease rust-analyzer; StyleProperty is imported but not recognised.
    /// To change the font size, use `with_style`, setting [`StyleProperty::FontSize`](parley::StyleProperty::FontSize).
    pub fn new(text: impl Into<ArcStr>) -> Self {
        let mut styles = StyleSet::new(theme::TEXT_SIZE_NORMAL);
        default_text_styles(&mut styles);
        Self {
            shaped: inrux_text::shape("", 15.0, None, f32::INFINITY),
            accessibility: LayoutAccessibility::default(),
            text: text.into(),
            styles,
            centered_insets: None,
            single_line: false,
            paint_shift: masonry::kurbo::Vec2::ZERO,
        }
    }

    /// Button and boolean labels share the other bindings' single-line control policy.
    pub(crate) fn single_line(mut self) -> Self {
        self.single_line = true;
        self
    }
    /// Preserve visual centering when a native button constrains or rounds its label box.
    pub(crate) fn centered_insets(mut self, insets: Size) -> Self {
        self.centered_insets = Some(insets);
        self
    }
    /// Apply changed native container insets without invalidating unchanged text metrics.
    pub(crate) fn set_centered_insets(this: &mut WidgetMut<'_, Self>, insets: Size) {
        if this.widget.centered_insets != Some(insets) {
            this.widget.centered_insets = Some(insets);
            this.ctx.request_layout();
        }
    }
    /// Inspect the glyph offset independently of native layout-origin rounding.
    #[cfg(test)]
    pub(crate) fn paint_shift(&self) -> masonry::kurbo::Vec2 {
        self.paint_shift
    }
    /// Get the current text of this label.
    ///
    /// To update the text of an active label, use [`set_text`](Self::set_text).
    pub fn text(&self) -> &ArcStr {
        &self.text
    }

    /// Set a style property for the new label.
    ///
    /// Setting [`StyleProperty::Brush`](parley::StyleProperty::Brush) is not supported.
    /// Use [`ContentColor`] and [`DisabledContentColor`] properties instead.
    ///
    /// To set a style property on an active label, use [`insert_style`](Self::insert_style).
    pub fn with_style(mut self, property: impl Into<StyleProperty>) -> Self {
        self.insert_style_inner(property.into());
        self
    }

    /// Shared logic between `with_style` and `insert_style`
    fn insert_style_inner(&mut self, property: StyleProperty) -> Option<StyleProperty> {
        if let StyleProperty::Brush(idx @ BrushIndex(1..))
        | StyleProperty::UnderlineBrush(Some(idx @ BrushIndex(1..)))
        | StyleProperty::StrikethroughBrush(Some(idx @ BrushIndex(1..))) = &property
        {
            panic!(
                "Can't set a non-zero brush index ({idx:?}) on a `Label`, as it only supports global styling."
            );
        }
        self.styles.insert(property)
    }
}

// --- MARK: WIDGETMUT
impl Label {
    // Note: These docs are lazy, but also have a decreased likelihood of going out of date.
    /// The runtime equivalent of [`with_style`](Self::with_style).
    ///
    /// Setting [`StyleProperty::Brush`](parley::StyleProperty::Brush) is not supported.
    /// Use [`ContentColor`] and [`DisabledContentColor`] properties instead.
    pub fn insert_style(
        this: &mut WidgetMut<'_, Self>,
        property: impl Into<StyleProperty>,
    ) -> Option<StyleProperty> {
        let old = this.widget.insert_style_inner(property.into());

        this.ctx.request_layout();
        old
    }

    /// Inspect or filter style entries in native regression fixtures.
    #[cfg(test)]
    pub fn retain_styles(this: &mut WidgetMut<'_, Self>, f: impl FnMut(&StyleProperty) -> bool) {
        this.widget.styles.retain(f);
        this.ctx.request_layout();
    }
    /// Replace the text of this widget.
    pub fn set_text(this: &mut WidgetMut<'_, Self>, new_text: impl Into<ArcStr>) {
        this.widget.text = new_text.into();

        this.ctx.request_layout();
    }
}

impl HasProperty<ContentColor> for Label {}
impl HasProperty<DisabledContentColor> for Label {}
impl HasProperty<LineBreaking> for Label {}

// --- MARK: IMPL WIDGET
impl Widget for Label {
    type Action = NoAction;

    fn accepts_pointer_interaction(&self) -> bool {
        false
    }

    fn register_children(&mut self, _ctx: &mut RegisterCtx<'_>) {}

    fn property_changed(&mut self, ctx: &mut UpdateCtx<'_>, property_type: TypeId) {
        LineBreaking::prop_changed(ctx, property_type);
        ContentColor::prop_changed(ctx, property_type);
        DisabledContentColor::prop_changed(ctx, property_type);
        Padding::prop_changed(ctx, property_type);
    }

    fn update(&mut self, ctx: &mut UpdateCtx<'_>, _props: &mut PropertiesMut<'_>, event: &Update) {
        if let Update::DisabledChanged(_) = event {
            ctx.request_paint_only();
        }
    }

    /// Native constraints supply available width; the common provider owns glyphs and line boxes.
    fn layout(
        &mut self,
        ctx: &mut LayoutCtx<'_>,
        props: &mut PropertiesMut<'_>,
        bc: &BoxConstraints,
    ) -> Size {
        let padding = *props.get::<Padding>();
        let content_bc = padding.layout_down(*bc);
        let mut size = 15.0;
        let mut family = None;
        for property in self.styles.inner().values() {
            match property {
                StyleProperty::FontSize(value) => size = *value,
                StyleProperty::FontStack(masonry::parley::FontStack::Single(
                    masonry::parley::FontFamily::Named(name),
                )) => family = Some(name.as_ref()),
                _ => {}
            }
        }
        let width = if !self.single_line && *props.get::<LineBreaking>() == LineBreaking::WordWrap {
            content_bc.max().width as f32
        } else {
            f32::INFINITY
        };
        self.shaped = inrux_text::shape(&self.text, size, family, width);
        let natural = Size::new(self.shaped.width().into(), self.shaped.height().into());
        let measured = content_bc.constrain(natural);
        self.paint_shift = if let Some(insets) = self.centered_insets {
            let parent = content_bc.max() + insets;
            let placed = ((parent - measured).to_vec2() / 2.0)
                .to_point()
                .round()
                .to_vec2();
            (parent - natural).to_vec2() / 2.0 - placed
        } else {
            masonry::kurbo::Vec2::ZERO
        };
        let (size, baseline) = padding.layout_up(measured, 0.0);
        ctx.set_baseline_offset(baseline);
        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx<'_>, props: &PropertiesRef<'_>, scene: &mut Scene) {
        let padding = *props.get::<Padding>();
        let line_break_mode = *props.get::<LineBreaking>();

        if line_break_mode == LineBreaking::Clip {
            let clip_rect = ctx.size().to_rect();
            scene.push_layer(BlendMode::default(), 1., Affine::IDENTITY, &clip_rect);
        }
        let text_origin = padding.place_down(Point::ZERO).to_vec2() + self.paint_shift;
        let transform = Affine::translate(text_origin);

        let text_color = if ctx.is_disabled() {
            &props.get::<DisabledContentColor>().0
        } else {
            props.get::<ContentColor>()
        };

        super::text_image::paint_layout(
            scene,
            transform,
            &self.shaped.layout,
            ctx.get_scale_factor() as f32,
            text_color.color,
            None,
        );

        if line_break_mode == LineBreaking::Clip {
            scene.pop_layer();
        }
    }

    fn accessibility_role(&self) -> Role {
        Role::Label
    }

    fn accessibility(
        &mut self,
        ctx: &mut AccessCtx<'_>,
        props: &PropertiesRef<'_>,
        node: &mut Node,
    ) {
        let padding = *props.get::<Padding>();

        let text_origin = padding.place_down(Point::ZERO).to_vec2() + self.paint_shift;
        let first_node = ctx.tree_update().nodes.len();
        self.accessibility.build_nodes(
            self.text.as_ref(),
            &self.shaped.layout,
            ctx.tree_update(),
            node,
            || NodeId::from(WidgetId::next()),
            text_origin.x,
            text_origin.y,
        );
        // Wrapped trailing whitespace can extend beyond the visible content width.
        if *props.get::<LineBreaking>() == LineBreaking::WordWrap {
            let width = ctx.size().width;
            for (_, child) in &mut ctx.tree_update().nodes[first_node..] {
                if let Some(mut bounds) = child.bounds() {
                    bounds.x1 = bounds.x1.min(width);
                    child.set_bounds(bounds);
                }
            }
        }
    }

    fn children_ids(&self) -> ChildrenIds {
        ChildrenIds::new()
    }

    fn make_trace_span(&self, id: WidgetId) -> Span {
        trace_span!("Label", id = id.trace())
    }

    fn get_debug_text(&self) -> Option<String> {
        Some(self.text.to_string())
    }
}
