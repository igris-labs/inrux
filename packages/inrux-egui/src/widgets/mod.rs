//! Private native bindings; public component contracts live in inrux-components.
use eframe::egui;
use inrux_style::Style;
/// Resolved native paint inputs.
pub(crate) struct Visual<'a> {
    pub(crate) rect: egui::Rect,
    pub(crate) path: egui::Id,
    pub(crate) style: &'a Style,
    pub(crate) radius: f32,
    pub(crate) ink: egui::Color32,
    pub(crate) font: egui::FontId,
}
pub(crate) mod button;
pub(crate) mod checkbox;
pub(crate) mod dialog;
pub(crate) mod heartbeat;
pub(crate) mod input;
pub(crate) mod select;
pub(crate) mod text;

pub(crate) mod range;
