//! Native window and application coordinator integration.
use eframe::egui;
use inrux_adapter::{Adapter, AdapterError, Capabilities, Primitive, Session, WindowOptions};
use inrux_components::{Component, Node};
use inrux_core::Scope;
use inrux_style::Rule;
use std::sync::Arc;

use crate::render_ref;
struct NativeApp<S: Component> {
    session: Session<S>,
    frame: Option<Arc<Node>>,
    error: Option<String>,
    #[cfg(feature = "devtools")]
    inspector: bool,
}
impl<S: Component> eframe::App for NativeApp<S> {
    /// Consume the latest description and paint it on the native main thread.
    fn update(&mut self, context: &egui::Context, _: &mut eframe::Frame) {
        match self.session.take_frame() {
            Ok(Some(frame)) => self.frame = Some(frame),
            Ok(None) => {}
            Err(error) => self.error = Some(error.to_string()),
        }
        #[cfg(feature = "devtools")]
        {
            if context.input(|input| input.key_pressed(egui::Key::F12)) {
                self.inspector = !self.inspector;
            }
            egui::Window::new("Inrux Inspector")
                .open(&mut self.inspector)
                .show(context, |ui| {
                    if let Some(frame) = &self.frame {
                        inrux_devtools::inspect(frame, ui);
                    }
                });
        }
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(context.style().visuals.panel_fill))
            .show(context, |ui| {
                self.session
                    .viewport()
                    .set(ui.available_width(), ui.available_height());
                if let Some(error) = &self.error {
                    ui.colored_label(egui::Color32::DARK_RED, error);
                    if ui.button("Dismiss").clicked() {
                        self.error = None;
                    }
                }
                // Applications explicitly compose ScrollArea; the host must not scroll a second tree.
                let failure = match self
                    .frame
                    .as_ref()
                    .map(|frame| render_ref(frame, ui))
                    .transpose()
                {
                    Ok(Some(Some(error))) => Some(error),
                    Err(error) => Some(format!("Layout failed: {error}")),
                    _ => None,
                };
                if let Some(failure) = failure
                    && self.error.as_ref() != Some(&failure)
                {
                    self.error = Some(failure);
                    context.request_repaint();
                }
            });
    }
}
/// Mount a compiled root component in a native window on the main thread.
///
/// The component owns its initial state; application actions execute on the worker.
pub fn mount<S: Component>(title: &str, initial: S, scope: Scope) -> Result<(), eframe::Error> {
    mount_with_styles(title, initial, scope, vec![])
}
/// Mount with application-global CSS rules loaded by main.rs before the first view.
pub fn mount_with_styles<S: Component>(
    title: &str,
    initial: S,
    scope: Scope,
    rules: Vec<Rule>,
) -> Result<(), eframe::Error> {
    mount_with_options(WindowOptions::new(title), initial, scope, rules)
}

/// Open the egui host after translating only backend-independent window configuration.
fn mount_with_options<S: Component>(
    window: WindowOptions,
    initial: S,
    scope: Scope,
    rules: Vec<Rule>,
) -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([window.width as f32, window.height as f32])
            .with_min_inner_size([window.min_width as f32, window.min_height as f32]),
        ..Default::default()
    };
    eframe::run_native(
        &window.title,
        options,
        Box::new(move |creation| {
            crate::platform::configure_colors();
            creation.egui_ctx.options_mut(|options| {
                options.input_options.line_scroll_speed = inrux_adapter::services::scroll::LINE_STEP
            });
            let mut visuals = egui::Visuals::light();
            visuals.selection.bg_fill = egui::Color32::from_rgb(208, 235, 226);
            visuals.selection.stroke =
                egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(29, 112, 98));
            visuals.text_cursor.stroke =
                egui::Stroke::new(2.0_f32, egui::Color32::from_rgb(29, 112, 98));
            visuals.panel_fill = egui::Color32::WHITE;
            creation.egui_ctx.set_visuals(visuals);
            crate::configure_fonts(&creation.egui_ctx, &window.fonts)?;
            let context = creation.egui_ctx.clone();
            let session = Session::start(initial, scope, rules, move || context.request_repaint())?;
            Ok(Box::new(NativeApp {
                session,
                frame: None,
                error: None,
                #[cfg(feature = "devtools")]
                inspector: false,
            }))
        }),
    )
}

/// Existing egui/Taffy implementation of the common native adapter contract.
#[derive(Clone, Copy, Debug, Default)]
pub struct Egui;
impl Adapter for Egui {
    /// Report the concrete service providers implemented by this private binding.
    fn services(&self) -> inrux_adapter::services::ServiceSupport {
        crate::services::SUPPORT
    }
    /// Report every primitive supported by the existing native renderer.
    fn capabilities(&self) -> Capabilities {
        Capabilities {
            name: "egui",
            primitives: &[
                Primitive::Column,
                Primitive::Row,
                Primitive::Text,
                Primitive::Button,
                Primitive::Input,
                Primitive::TextArea,
                Primitive::Checkbox,
                Primitive::Switch,
                Primitive::Progress,
                Primitive::Meter,
                Primitive::Select,
                Primitive::Dialog,
                Primitive::Heartbeat,
                Primitive::Empty,
            ],
            interaction_styles: true,
        }
    }
    /// Run the same coordinator/session contract used by other native backends.
    fn run<S: Component>(
        self,
        window: WindowOptions,
        initial: S,
        scope: Scope,
        styles: Vec<Rule>,
    ) -> Result<(), AdapterError> {
        window.validate()?;
        mount_with_options(window, initial, scope, styles).map_err(|e| AdapterError::Native {
            backend: "egui",
            message: e.to_string(),
        })
    }
}
