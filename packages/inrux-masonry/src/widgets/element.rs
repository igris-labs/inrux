//! Retained wrapper nodes preserve framework identity while Masonry owns native widgets.
use crate::widgets::checkbox::NativeCheckbox as Checkbox;
use crate::widgets::input::TextInput;
use crate::widgets::label::Label;
use crate::widgets::{button, checkbox, input, text};
use inrux_adapter::{AdapterError, editing::Editor};
use inrux_components::{Node, NodeKind};
use inrux_style::Style;
use masonry::{
    accesskit,
    core::{
        AccessCtx, ChildrenIds, ErasedAction, NewWidget, PropertiesMut, PropertiesRef, QueryCtx,
        RegisterCtx, Update, UpdateCtx, Widget, WidgetId, WidgetMut, WidgetPod, WidgetRef,
    },
    widgets::{Button, ButtonPress, CheckboxToggled, TextAction},
};
use std::{cell::RefCell, collections::HashMap, rc::Rc, sync::Arc};
/// Routes actual native action IDs (including an input's text area) to retained wrappers.
pub(crate) type Routes = HashMap<WidgetId, WidgetId>;
#[derive(Hash, PartialEq, Eq)]
enum ChildIdentity {
    Key(String),
    Position(usize),
}
/// Backend-only metadata for one logical primitive; children are owned by Masonry pods.
pub(crate) struct Element {
    pub(crate) geometry: Rc<RefCell<crate::layout::LayoutState>>,
    pub(crate) path: Vec<usize>,
    pub(crate) kind: NodeKind,
    pub(crate) style: Style,
    pub(crate) paint_owner: Node,
    applied_style: Option<Style>,
    applied_placeholder: Option<inrux_style::Color>,
    pub(crate) disabled: bool,
    pub(crate) focus_visibility: inrux_adapter::FocusVisibility,
    pub(crate) key: Option<String>,
    pub(crate) generation: Option<String>,
    pub(crate) children: Vec<WidgetPod<Element>>,
    keys: Vec<Option<String>>,
    pub(crate) content: Option<WidgetPod<dyn Widget>>,
    pub(crate) action_id: Option<WidgetId>,
    pub(crate) editor: Option<Editor>,
    pub(crate) scroll: Option<Rc<RefCell<inrux_adapter::services::scroll::ScrollState>>>,
    pub(crate) focused: Option<(String, inrux_adapter::services::clipping::ClipRect)>,
    pub(crate) bars: Vec<WidgetPod<super::viewport::ViewportBar>>,
}
impl Element {
    /// Build a widget subtree once; snapshots never hold native handles.
    pub(crate) fn build(node: &Node, routes: &mut Routes) -> NewWidget<Self> {
        let geometry = Rc::new(RefCell::new(crate::layout::LayoutState::new(Arc::new(
            node.clone(),
        ))));
        Self::build_at(node, routes, geometry, vec![])
    }
    /// Associate private native nodes with one shared logical layout tree.
    fn build_at(
        node: &Node,
        routes: &mut Routes,
        geometry: Rc<RefCell<crate::layout::LayoutState>>,
        path: Vec<usize>,
    ) -> NewWidget<Self> {
        let id = WidgetId::next();
        let style = node.style();
        let content = match &node.kind {
            NodeKind::Text(title) => Some(
                NewWidget::new_with_props(
                    text::label(title, &style),
                    // Shared Text geometry already wraps; native glyph placement must agree.
                    text::properties(&style).with(text::line_breaking(&style)),
                )
                .erased(),
            ),
            NodeKind::Button(title, _) => Some(button::build(title, &style).erased()),
            NodeKind::Checkbox {
                kind,
                checked,
                label,
                ..
            } => Some(
                checkbox::build(
                    *kind,
                    *checked,
                    label,
                    &inrux_adapter::control_parts::toggle(
                        node,
                        &node.paint_style(false, false),
                        false,
                        false,
                    ),
                )
                .erased(),
            ),
            NodeKind::Range(range) => Some(
                super::range::build(
                    range,
                    &inrux_adapter::control_parts::range(
                        node,
                        &node.paint_style(false, false),
                        false,
                        false,
                    ),
                )
                .erased(),
            ),
            NodeKind::Input {
                value,
                placeholder,
                multiline,
                ..
            } => Some(input::build(value, placeholder, &style, *multiline).erased()),
            _ => None,
        };
        let action_id = content.as_ref().map(|widget| {
            if let Some(input) =
                (widget.widget.as_ref() as &dyn std::any::Any).downcast_ref::<TextInput>()
            {
                input.area_pod().id()
            } else {
                widget.id()
            }
        });
        if let Some(action_id) = action_id {
            routes.insert(action_id, id);
        }
        let editor = if let NodeKind::Input { value, event, .. } = &node.kind {
            Some(Editor::new(value, event.applied()))
        } else {
            None
        };
        let children = node
            .children
            .iter()
            .enumerate()
            .map(|(index, child)| {
                let mut child_path = path.clone();
                child_path.push(index);
                Self::build_at(child, routes, geometry.clone(), child_path).to_pod()
            })
            .collect();
        let scroll = inrux_adapter::services::scroll::clips(&style)
            .then(|| Rc::new(RefCell::new(Default::default())));
        let bars = super::viewport::bars(&style, scroll.as_ref(), id);
        NewWidget::new_with_id(
            Self {
                geometry,
                path,
                keys: node
                    .children
                    .iter()
                    .map(|child| child.key.clone())
                    .collect(),
                kind: node.kind.clone(),
                style: style.clone(),
                paint_owner: node.paint_snapshot(),
                applied_style: if matches!(node.kind, NodeKind::Input { .. }) {
                    None
                } else {
                    Some(style.clone())
                },
                applied_placeholder: None,
                disabled: node.disabled,
                focus_visibility: Default::default(),
                key: node.key.clone(),
                generation: node.generation.clone(),
                children,
                content: content.map(NewWidget::to_pod),
                action_id,
                editor,
                scroll,
                focused: None,
                bars,
            },
            id,
        )
    }
    /// Keep matching keyed/generation-bearing pods during reorder; dispose absent widgets.
    pub(crate) fn reconcile(this: &mut WidgetMut<'_, Self>, node: &Node, routes: &mut Routes) {
        this.widget
            .geometry
            .borrow_mut()
            .replace_descriptor(Arc::new(node.clone()));
        Self::reconcile_at(this, node, routes, vec![]);
    }
    /// Reorder private engine pods while updating their position in the shared geometry tree.
    fn reconcile_at(
        this: &mut WidgetMut<'_, Self>,
        node: &Node,
        routes: &mut Routes,
        path: Vec<usize>,
    ) {
        this.widget.path = path.clone();
        let reset = this.widget.generation != node.generation
            || this.widget.key != node.key
            || std::mem::discriminant(&this.widget.kind) != std::mem::discriminant(&node.kind)
            || matches!((&this.widget.kind, &node.kind), (NodeKind::Input { multiline: old, .. }, NodeKind::Input { multiline: new, .. }) if old != new);
        if reset {
            if let Some(content) = this.widget.content.take() {
                this.ctx.remove_child(content);
            }
            for child in std::mem::take(&mut this.widget.children) {
                this.ctx.remove_child(child);
            }
            for bar in std::mem::take(&mut this.widget.bars) {
                this.ctx.remove_child(bar);
            }
            let replacement =
                Self::build_at(node, routes, this.widget.geometry.clone(), path.clone());
            *this.widget = *replacement.widget;
            // The wrapper keeps its engine WidgetId on reset; newly allocated bars target that ID.
            this.widget.bars = super::viewport::bars(
                &this.widget.style,
                this.widget.scroll.as_ref(),
                this.ctx.widget_id(),
            );
            this.ctx.children_changed();
        } else {
            let old_ids: Vec<_> = this.widget.children.iter().map(WidgetPod::id).collect();
            let old_keys = std::mem::take(&mut this.widget.keys);
            let mut previous: HashMap<_, _> = std::mem::take(&mut this.widget.children)
                .into_iter()
                .zip(old_keys)
                .enumerate()
                .map(|(index, (pod, key))| {
                    (
                        key.map_or(ChildIdentity::Position(index), ChildIdentity::Key),
                        pod,
                    )
                })
                .collect();
            let mut children = Vec::with_capacity(node.children.len());
            for (index, child) in node.children.iter().enumerate() {
                let identity = child
                    .key
                    .clone()
                    .map_or(ChildIdentity::Position(index), ChildIdentity::Key);
                let mut child_path = path.clone();
                child_path.push(index);
                let pod = if let Some(mut pod) = previous.remove(&identity) {
                    Self::reconcile_at(&mut this.ctx.get_mut(&mut pod), child, routes, child_path);
                    pod
                } else {
                    Self::build_at(child, routes, this.widget.geometry.clone(), child_path).to_pod()
                };
                children.push(pod);
            }
            for child in previous.into_values() {
                this.ctx.remove_child(child);
            }
            let changed = old_ids != children.iter().map(WidgetPod::id).collect::<Vec<_>>();
            this.widget.children = children;
            this.widget.keys = node
                .children
                .iter()
                .map(|child| child.key.clone())
                .collect();
            if changed {
                this.ctx.children_changed();
            }
            this.widget.kind = node.kind.clone();
            let style = node.style();
            let clipped = inrux_adapter::services::scroll::clips(&style);
            if clipped && this.widget.scroll.is_none() {
                this.widget.scroll = Some(Rc::new(RefCell::new(Default::default())));
            }
            let wants_bars = inrux_adapter::services::scroll::scrollable(&style);
            if wants_bars != !this.widget.bars.is_empty() {
                for bar in std::mem::take(&mut this.widget.bars) {
                    this.ctx.remove_child(bar);
                }
                this.widget.bars = super::viewport::bars(
                    &style,
                    this.widget.scroll.as_ref(),
                    this.ctx.widget_id(),
                );
                this.ctx.children_changed();
            }
            if !clipped {
                this.widget.scroll = None;
            }
            for bar in &mut this.widget.bars {
                let mut bar = this.ctx.get_mut(bar);
                let colors = style.resolved_scrollbar_colors();
                if bar.widget.colors != colors {
                    bar.widget.colors = colors;
                    bar.ctx.request_render();
                }
            }
            this.widget.style = style;
            this.widget.paint_owner = node.paint_snapshot();
            this.widget.disabled = node.disabled;
            this.widget.key = node.key.clone();
            this.widget.update_content(&mut this.ctx);
        }
        if let Some(action) = this.widget.action_id {
            routes.insert(action, this.ctx.widget_id());
        }
        this.ctx.set_disabled(node.disabled);
        if this.widget.scroll.is_some() {
            this.ctx.request_anim_frame();
        }
        this.ctx.request_layout();
        this.ctx.request_render();
    }
    /// Apply props/model acknowledgements to native leaves without rebuilding them.
    fn update_content(&mut self, ctx: &mut masonry::core::MutateCtx<'_>) {
        let Some(content) = &mut self.content else {
            return;
        };
        let hovered = ctx.has_hovered();
        let focused = ctx.has_focus_target()
            && (matches!(self.kind, NodeKind::Input { .. }) || self.focus_visibility.visible());
        let paint = self.paint_owner.paint_style(hovered, focused);
        let placeholder_ink = matches!(self.kind, NodeKind::Input { .. }).then(|| {
            inrux_adapter::control_parts::placeholder(&self.paint_owner, hovered, focused)
        });
        let mut widget = ctx.get_mut(content);
        match &self.kind {
            NodeKind::Text(title) => text::update(
                &mut widget.downcast::<Label>(),
                title,
                &paint,
                self.applied_style.as_ref(),
            ),
            NodeKind::Button(title, _) => {
                let mut style = paint.clone();
                if hovered
                    && !self.disabled
                    && !self.paint_owner.states.iter().any(|(state, layer)| {
                        *state == inrux_style::State::Hover && layer.background.is_some()
                    })
                {
                    style.background = Some(inrux_adapter::controls::hovered_button_color(
                        style
                            .background
                            .unwrap_or(inrux_style::Color(235, 235, 235)),
                    ));
                }
                button::update(
                    &mut widget.downcast::<Button>(),
                    title,
                    &style,
                    self.applied_style.as_ref(),
                );
                self.applied_style = Some(style);
                return;
            }
            NodeKind::Checkbox {
                kind,
                checked,
                label,
                ..
            } => checkbox::update(
                &mut widget.downcast::<Checkbox>(),
                *kind,
                *checked,
                label,
                &inrux_adapter::control_parts::toggle(&self.paint_owner, &paint, hovered, focused),
            ),
            NodeKind::Range(range) => super::range::update(
                &mut widget.downcast::<super::range::NativeRange>(),
                range,
                &inrux_adapter::control_parts::range(&self.paint_owner, &paint, hovered, focused),
            ),
            NodeKind::Input {
                value,
                placeholder,
                event,
                ..
            } => {
                if let Some(editor) = &mut self.editor {
                    editor.reconcile(value, event.applied());
                    input::update(
                        &mut widget.downcast::<TextInput>(),
                        &editor.value,
                        placeholder,
                        &paint,
                        placeholder_ink.unwrap_or(inrux_adapter::controls::PLACEHOLDER),
                        self.applied_style.as_ref(),
                        self.applied_placeholder,
                    );
                }
            }
            _ => {}
        }
        self.applied_style = Some(paint);
        self.applied_placeholder = placeholder_ink;
    }
    /// Forward native actions through the framework's bounded command queue.
    pub(crate) fn action(
        this: &mut WidgetMut<'_, Self>,
        action: ErasedAction,
    ) -> Result<(), AdapterError> {
        if this.widget.disabled {
            return Ok(());
        }
        let result = match &this.widget.kind {
            NodeKind::Button(_, Some(event)) if action.is::<ButtonPress>() => event.emit(()),
            NodeKind::Checkbox { event, .. } if action.is::<CheckboxToggled>() => {
                match action.downcast::<CheckboxToggled>() {
                    Ok(value) => event.emit(value.0),
                    Err(_) => return Ok(()),
                }
            }
            NodeKind::Input { event, .. } if action.is::<TextAction>() => {
                let Ok(action) = action.downcast::<TextAction>() else {
                    return Ok(());
                };
                let TextAction::Changed(value) = *action else {
                    return Ok(());
                };
                let Some(editor) = &mut this.widget.editor else {
                    return Ok(());
                };
                let previous = std::mem::replace(&mut editor.value, value);
                let revision = editor.edited();
                let result = event.emit(revision, editor.value.clone());
                if result.is_err() {
                    editor.reject(previous);
                }
                this.widget.update_content(&mut this.ctx);
                result
            }
            _ => return Ok(()),
        };
        result.map_err(|error| AdapterError::Application(error.to_string()))
    }
}
impl Widget for Element {
    type Action = AdapterError;
    /// Register the owned retained pods with the engine.
    fn register_children(&mut self, ctx: &mut RegisterCtx<'_>) {
        if let Some(content) = &mut self.content {
            ctx.register_child(content);
        }
        for child in &mut self.children {
            ctx.register_child(child);
        }
        for bar in &mut self.bars {
            ctx.register_child(bar);
        }
    }
    /// Stop indicator work after the finite idle deadline; caret work remains native and independent.
    fn on_anim_frame(&mut self, ctx: &mut UpdateCtx<'_>, _: &mut PropertiesMut<'_>, _: u64) {
        super::viewport::animate(self, ctx);
    }
    /// Disabled metadata must apply on the first mount as well as later snapshots.
    fn update(&mut self, ctx: &mut UpdateCtx<'_>, _: &mut PropertiesMut<'_>, event: &Update) {
        ctx.set_disabled(self.disabled);
        match event {
            Update::ChildActiveChanged(true) | Update::ActiveChanged(true) => {
                self.focus_visibility.pointer();
                ctx.request_render();
            }
            Update::ChildFocusChanged(false) | Update::FocusChanged(false) => {
                self.focus_visibility.keyboard();
                ctx.request_render();
            }
            _ => {}
        }
        if matches!(
            event,
            Update::ChildFocusChanged(_) | Update::FocusChanged(_)
        ) {
            ctx.request_layout();
        }
        if matches!(
            event,
            Update::ChildHoveredChanged(_)
                | Update::HoveredChanged(_)
                | Update::DisabledChanged(_)
                | Update::FocusChanged(_)
                | Update::ChildFocusChanged(_)
                | Update::ActiveChanged(_)
                | Update::ChildActiveChanged(_)
        ) {
            ctx.request_render();
            // Refresh authored native paint without replacing controls, labels or event admission.
            ctx.mutate_self_later(|mut element| {
                let mut element = element.downcast::<Self>();
                element.widget.update_content(&mut element.ctx);
            });
        }
        if matches!(event, Update::WidgetAdded) {
            // Apply the placeholder's native styles before the first paint, not only after a model update.
            ctx.mutate_self_later(|mut element| {
                let mut element = element.downcast::<Self>();
                element.widget.update_content(&mut element.ctx);
            });
        }
    }
    /// Pointer admission is cached by Masonry; keep it stable and filter actual owner targets below.
    fn accepts_pointer_interaction(&self) -> bool {
        true
    }
    /// Preserve native child hit testing while observing authored hover on passive or restyled owners.
    fn find_widget_under_pointer<'c>(
        &'c self,
        ctx: QueryCtx<'c>,
        pos: masonry::kurbo::Point,
    ) -> Option<WidgetRef<'c, dyn Widget>> {
        let own = ctx.widget_id();
        let local = ctx.window_transform().inverse() * pos;
        let target = masonry::core::find_widget_under_pointer(self, ctx, pos)?;
        if target.id() != own {
            return Some(target);
        }
        let geometry = self.geometry.borrow();
        let bounds = geometry.at(&self.path)?;
        let inside = masonry::kurbo::Rect::new(0.0, 0.0, bounds.width.into(), bounds.height.into())
            .contains(local);
        (inside
            && (inrux_adapter::services::scroll::scrollable(&self.style)
                || self.paint_owner.has_hover_styles()))
        .then_some(target)
    }
    fn accepts_focus(&self) -> bool {
        inrux_adapter::services::scroll::scrollable(&self.style)
    }
    fn on_pointer_event(
        &mut self,
        ctx: &mut masonry::core::EventCtx<'_>,
        _: &mut PropertiesMut<'_>,
        event: &masonry::core::PointerEvent,
    ) {
        if matches!(event, masonry::core::PointerEvent::Down(_)) {
            self.focus_visibility.pointer();
            ctx.request_render();
        }
        super::viewport::pointer(self, ctx, event);
    }
    fn on_text_event(
        &mut self,
        ctx: &mut masonry::core::EventCtx<'_>,
        _: &mut PropertiesMut<'_>,
        event: &masonry::core::TextEvent,
    ) {
        if matches!(event, masonry::core::TextEvent::Keyboard(event) if event.state.is_down()) {
            self.focus_visibility.keyboard();
            ctx.request_render();
        }
        super::viewport::keyboard(self, ctx, event);
    }
    /// Wrappers participate in the tree without inventing a control role.
    fn accessibility_role(&self) -> accesskit::Role {
        accesskit::Role::GenericContainer
    }
    /// Native leaves supply their own accessible labels and interaction semantics.
    fn accessibility(
        &mut self,
        _: &mut AccessCtx<'_>,
        _: &PropertiesRef<'_>,
        _: &mut accesskit::Node,
    ) {
    }
    /// Stable order is shared by layout, keyboard traversal and accessibility.
    fn children_ids(&self) -> ChildrenIds {
        self.content
            .iter()
            .map(WidgetPod::id)
            .chain(self.children.iter().map(WidgetPod::id))
            .chain(self.bars.iter().map(WidgetPod::id))
            .collect()
    }
    /// Delegate geometry to the shared layout policy and native placement bridge.
    fn layout(
        &mut self,
        ctx: &mut masonry::core::LayoutCtx<'_>,
        _: &mut PropertiesMut<'_>,
        bc: &masonry::core::BoxConstraints,
    ) -> masonry::kurbo::Size {
        crate::layout::layout(self, ctx, bc)
    }
    /// Native composition moves scroll content while preserving cached layout and text.
    fn compose(&mut self, ctx: &mut masonry::core::ComposeCtx<'_>) {
        super::viewport::compose(self, ctx);
    }
    /// Draw an engine-independent focus outline after native child painting.
    fn post_paint(
        &mut self,
        ctx: &mut masonry::core::PaintCtx<'_>,
        _: &PropertiesRef<'_>,
        scene: &mut masonry::vello::Scene,
    ) {
        let size = self
            .geometry
            .borrow()
            .at(&self.path)
            .map_or(ctx.size(), |bounds| {
                masonry::kurbo::Size::new(bounds.width.into(), bounds.height.into())
            });
        let editor = matches!(self.kind, NodeKind::Input { .. });
        if !self.paint_owner.has_focus_decoration()
            && (editor || self.focus_visibility.visible())
            && ((ctx.has_focus_target()
                && matches!(
                    self.kind,
                    NodeKind::Button(..) | NodeKind::Checkbox { .. } | NodeKind::Input { .. }
                ))
                || (ctx.is_focus_target()
                    && inrux_adapter::services::scroll::scrollable(&self.style)))
        {
            let checkbox = matches!(self.kind, NodeKind::Checkbox { .. });
            let visual =
                inrux_adapter::control_parts::toggle(&self.paint_owner, &self.style, false, true);
            let rect = if checkbox {
                let (width, height) = (visual.width, visual.height);
                let y = (size.height - f64::from(height)) / 2.0;
                masonry::kurbo::RoundedRect::new(
                    1.0,
                    y + 1.0,
                    f64::from(width) - 1.0,
                    y + f64::from(height) - 1.0,
                    f64::from(visual.radius),
                )
            } else {
                masonry::kurbo::RoundedRect::from_rect(
                    size.to_rect().inset(-0.5),
                    f64::from(self.style.rounded.unwrap_or(6.0)),
                )
            };
            scene.stroke(
                &masonry::kurbo::Stroke::new(if checkbox { 2.0 } else { 1.0 }),
                masonry::kurbo::Affine::IDENTITY,
                text::color(
                    self.style
                        .accent_color
                        .unwrap_or(inrux_adapter::controls::ACCENT),
                ),
                None,
                &rect,
            );
        }
    }
    /// Render style surfaces with the backend's vector painter.
    fn paint(
        &mut self,
        ctx: &mut masonry::core::PaintCtx<'_>,
        _: &PropertiesRef<'_>,
        scene: &mut masonry::vello::Scene,
    ) {
        crate::paint::paint(self, ctx, scene);
    }
}
