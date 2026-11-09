//! Winit/Masonry hosting, with coordinator wakeups marshalled onto the UI thread.
use crate::widgets::label::Label;
use crate::{
    capabilities,
    widgets::element::{Element, Routes},
};
use inrux_adapter::{Adapter, AdapterError, Capabilities, Session, WindowOptions};
use inrux_components::Component;
use inrux_core::Scope;
use inrux_style::Rule;
use masonry::{
    core::{ErasedAction, WidgetId},
    dpi::LogicalSize,
    widgets::{Align, SizedBox},
};
use masonry_winit::{
    app::{AppDriver, DriverCtx, EventLoop, MasonryUserEvent, NewWindow, WindowId},
    winit::window::Window,
};

/// A retained native backend; unsupported capabilities return errors rather than approximations.
#[derive(Clone, Copy, Debug, Default)]
pub struct Masonry;
/// Internal wake message carries no borrowed state or backend handles to the coordinator.
#[derive(Debug)]
struct Refresh;
struct Driver<S: Component> {
    session: Session<S>,
    root: WidgetId,
    routes: Routes,
    element_root: Option<WidgetId>,
}
impl<S: Component> AppDriver for Driver<S> {
    /// Apply the newest description or route an action through the shared bounded event queue.
    fn on_action(
        &mut self,
        window: WindowId,
        ctx: &mut DriverCtx<'_, '_>,
        widget: WidgetId,
        action: ErasedAction,
    ) {
        let root = ctx.render_root(window);
        let outcome = if action.is::<AdapterError>() {
            action
                .downcast::<AdapterError>()
                .map_or_else(|_| Ok(()), |error| Err(*error))
        } else if action.is::<Refresh>() {
            match self.session.take_frame() {
                Ok(Some(frame)) => capabilities::validate(&frame).map(|()| {
                    if let Some(element) = self.element_root {
                        root.edit_widget(element, |mut widget| {
                            self.routes.clear();
                            Element::reconcile(
                                &mut widget.downcast::<Element>(),
                                &frame,
                                &mut self.routes,
                            );
                        });
                    } else {
                        self.routes.clear();
                        let tree = Element::build(&frame, &mut self.routes);
                        tree.widget.geometry.borrow_mut().viewport = Some(self.session.viewport());
                        self.element_root = Some(tree.id());
                        root.edit_widget(self.root, |mut widget| {
                            let aligned =
                                Align::new(masonry::properties::types::UnitPoint::TOP_LEFT, tree);
                            SizedBox::set_child(
                                &mut widget.downcast::<SizedBox>(),
                                masonry::core::NewWidget::new(aligned),
                            );
                        });
                    }
                }),
                Ok(None) => Ok(()),
                Err(error) => Err(error),
            }
        } else if let Some(target) = self.routes.get(&widget).copied() {
            root.edit_widget(target, |mut widget| {
                Element::action(&mut widget.downcast::<Element>(), action)
            })
        } else {
            Ok(())
        };
        if let Err(error) = outcome {
            // Keep failures visible in the native surface; do not panic in the platform callback.
            self.routes.clear();
            self.element_root = None;
            root.edit_widget(self.root, |mut widget| {
                SizedBox::set_child(
                    &mut widget.downcast::<SizedBox>(),
                    masonry::core::NewWidget::new(Label::new(error.to_string())),
                )
            });
        }
    }
}
impl Adapter for Masonry {
    /// Report the concrete service providers implemented by this private binding.
    fn services(&self) -> inrux_adapter::services::ServiceSupport {
        crate::services::SUPPORT
    }
    /// Report the initial adapter's intentionally limited supported surface.
    fn capabilities(&self) -> Capabilities {
        capabilities::CAPABILITIES
    }
    /// Start the native main-thread loop and a separate shared application coordinator.
    fn run<S: Component>(
        self,
        options: WindowOptions,
        initial: S,
        scope: Scope,
        styles: Vec<Rule>,
    ) -> Result<(), AdapterError> {
        options.validate()?;
        inrux_text::register_fonts(options.fonts.iter().map(|font| font.data.as_ref()))
            .map_err(AdapterError::Configuration)?;
        let event_loop = EventLoop::with_user_event().build().map_err(native_error)?;
        let proxy = event_loop.create_proxy();
        let window = WindowId::next();
        let root = WidgetId::next();
        let session = Session::start(initial, scope, styles, move || {
            let _ = proxy.send_event(MasonryUserEvent::Action(window, Box::new(Refresh), root));
        })?;
        let attributes = Window::default_attributes()
            .with_title(options.title)
            .with_inner_size(LogicalSize::new(options.width, options.height))
            .with_min_inner_size(LogicalSize::new(options.min_width, options.min_height));
        let host = masonry::core::NewWidget::new_with_id(
            SizedBox::new(masonry::core::NewWidget::new(Label::new("Starting Inrux…"))).expand(),
            root,
        );
        masonry_winit::app::run_with(
            event_loop,
            vec![NewWindow::new_with_id(window, attributes, host.erased())],
            Driver {
                session,
                root,
                routes: Routes::new(),
                element_root: None,
            },
            crate::theme::defaults(),
        )
        .map_err(native_error)
    }
}
/// Convert platform errors at the boundary without exposing their concrete types.
fn native_error(error: impl std::fmt::Display) -> AdapterError {
    AdapterError::Native {
        backend: "masonry",
        message: error.to_string(),
    }
}
