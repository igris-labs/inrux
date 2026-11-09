//! GPUI main-thread hosting with coalesced coordinator notifications.
use crate::{
    capabilities, layout,
    widgets::{Failure, Widgets},
};
use futures::{StreamExt, channel::mpsc};
use gpui::{
    App, Application, Bounds, Context, IntoElement, Render, TitlebarOptions, Window, WindowBounds,
    div, prelude::*, px, size,
};
use inrux_adapter::{Adapter, AdapterError, Capabilities, Session, WindowOptions};
use inrux_components::{Component, Node};
use inrux_core::Scope;
use inrux_style::Rule;
use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
};
/// Experimental GPUI infrastructure behind the same framework-facing host contract.
#[derive(Clone, Copy, Debug, Default)]
pub struct Gpui;
struct Host<S: Component> {
    session: Session<S>,
    frame: Option<Arc<Node>>,
    widgets: Widgets,
    error: Failure,
}
impl<S: Component> Host<S> {
    /// Wake on the native executor; the bounded channel coalesces notifications without polling.
    fn new(session: Session<S>, mut wakes: mpsc::Receiver<()>, cx: &mut Context<Self>) -> Self {
        cx.spawn(async move |this, cx| {
            while wakes.next().await.is_some() {
                if this.update(cx, |_, cx| cx.notify()).is_err() {
                    break;
                }
            }
        })
        .detach();
        Self {
            session,
            frame: None,
            widgets: Widgets::default(),
            error: Rc::default(),
        }
    }
}
impl<S: Component> Render for Host<S> {
    /// Native frames use immutable descriptions and never call user views or state handlers.
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let size = window.viewport_size();
        self.session
            .viewport()
            .set(size.width.into(), size.height.into());
        match self.session.take_frame() {
            Ok(Some(frame)) => match capabilities::validate(&frame) {
                Ok(()) => {
                    self.frame = Some(frame);
                    *self.error.borrow_mut() = None;
                }
                Err(error) => {
                    self.frame = None;
                    *self.error.borrow_mut() = Some(error);
                    self.widgets = Widgets::default();
                }
            },
            Ok(None) => {}
            Err(error) => {
                self.frame = None;
                *self.error.borrow_mut() = Some(error);
                self.widgets = Widgets::default();
            }
        }
        let keyboard = self.widgets.focus_visibility();
        let mut root = div()
            .id("inrux-host")
            .relative()
            .size_full()
            .bg(gpui::white())
            .text_color(gpui::black())
            .on_key_down(move |event, window, cx| {
                let mut visibility = keyboard.get();
                if visibility.keyboard() {
                    keyboard.set(visibility);
                    window.refresh();
                }
                if event.keystroke.key == "tab" {
                    cx.stop_propagation();
                    if event.keystroke.modifiers.shift {
                        window.focus_prev();
                    } else {
                        window.focus_next();
                    }
                }
            });
        if let Some(frame) = &self.frame {
            match layout::compute(frame, window.viewport_size().width.into(), window) {
                Ok(boxes) => {
                    root = root.child(self.widgets.frame(
                        frame,
                        &boxes,
                        self.error.clone(),
                        window,
                        cx,
                    ))
                }
                Err(error) => *self.error.borrow_mut() = Some(error),
            }
        }
        if let Some(error) = self.error.borrow().as_ref() {
            root = root.child(error.to_string());
        }
        root
    }
}
impl Adapter for Gpui {
    /// Report the concrete service providers implemented by this private binding.
    fn services(&self) -> inrux_adapter::services::ServiceSupport {
        crate::services::SUPPORT
    }
    /// Advertise only the native surface implemented by this binding.
    fn capabilities(&self) -> Capabilities {
        capabilities::CAPABILITIES
    }
    /// Open a native window and connect a shared coordinator without exposing engine types.
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
        let (wake, wakes) = mpsc::channel(1);
        let wake = Mutex::new(wake);
        let session = Session::start(initial, scope, styles, move || {
            if let Ok(mut sender) = wake.lock() {
                let _ = sender.try_send(());
            }
        })?;
        let failure = Rc::new(RefCell::new(None));
        let result = failure.clone();
        Application::new().run(move |cx: &mut App| {
            crate::widgets::input::bind_keys(cx);
            cx.on_window_closed(|cx| {
                if cx.windows().is_empty() {
                    cx.quit();
                }
            })
            .detach();
            let bounds = Bounds::centered(
                None,
                size(px(options.width as f32), px(options.height as f32)),
                cx,
            );
            let window = cx.open_window(
                gpui::WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    window_min_size: Some(size(
                        px(options.min_width as f32),
                        px(options.min_height as f32),
                    )),
                    titlebar: Some(TitlebarOptions {
                        title: Some(options.title.into()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                |_, cx| cx.new(|cx| Host::new(session, wakes, cx)),
            );
            if let Err(error) = window {
                *failure.borrow_mut() = Some(AdapterError::Native {
                    backend: "gpui",
                    message: error.to_string(),
                });
                cx.quit();
            } else {
                cx.activate(true);
            }
        });
        result.borrow_mut().take().map_or(Ok(()), Err)
    }
}
