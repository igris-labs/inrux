//! Native modal presentation and controlled close requests.
use eframe::egui;
use inrux_components::{Node, NodeKind as Kind};
/// Present modal content outside normal Taffy flow and forward native dismissal to its owner.
pub(crate) fn paint(
    node: &Node,
    ui: &mut egui::Ui,
    path: egui::Id,
    error: &mut Option<String>,
) -> Result<(), inrux_adapter::AdapterError> {
    if let Kind::Dialog { open, on_close } = &node.kind {
        if *open {
            let response = egui::Modal::new(path).show(ui.ctx(), |ui| {
                if let Some(content) = node.children.first() {
                    crate::render_ref(content, ui)
                } else {
                    Ok(None)
                }
            });
            let should_close = response.should_close();
            if let Some(failure) = response.inner? {
                *error = Some(failure);
            }
            if should_close && let Err(failure) = on_close.emit(()) {
                *error = Some(failure.to_string());
            }
        }
        return Ok(());
    }
    Ok(())
}
