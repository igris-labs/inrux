//! Headless egui frames intentionally inspect shapes without uploading GPU textures.
use eframe::egui;
pub(crate) fn frame(
    context: &egui::Context,
    input: egui::RawInput,
    render: impl FnMut(&mut egui::Ui),
) -> egui::FullOutput {
    let mut output = context.run_ui(input, render);
    output.textures_delta.clear();
    output
}
