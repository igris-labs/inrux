//! Native select widget behavior.
use crate::paint::style_widgets;
use crate::widgets::Visual;
use eframe::egui;
use inrux_components::{Node, NodeKind as Kind};
use inrux_core::DispatchError;

/// Paint and handle one native select without running application logic on the UI thread.
pub(crate) fn paint(
    node: &Node,
    ui: &mut egui::Ui,
    visual: &Visual<'_>,
) -> Result<(), DispatchError> {
    let Kind::Select {
        value,
        options,
        event,
    } = &node.kind
    else {
        return Ok(());
    };
    let rect = visual.rect;
    let path = visual.path;
    let style = visual.style;
    let radius = visual.radius;
    let mut emit_result = Ok(());
    ui.scope_builder(egui::UiBuilder::new().id_salt(path).max_rect(rect), |ui| {
        style_widgets(ui, style, radius);
        ui.spacing_mut().interact_size.y = rect.height();
        let response = ui
            .add_enabled_ui(!node.disabled, |ui| {
                egui::ComboBox::from_id_salt(path)
                    .width(rect.width())
                    .selected_text(crate::typography::layout(
                        ui,
                        value,
                        visual.font.clone(),
                        visual.ink,
                        f32::INFINITY,
                    ))
                    .show_ui(ui, |ui| {
                        for option in options {
                            let galley = crate::typography::layout(
                                ui,
                                option,
                                visual.font.clone(),
                                visual.ink,
                                f32::INFINITY,
                            );
                            let height = galley.size().y;
                            let item = ui.selectable_label(option == value, galley);
                            crate::typography::paint(
                                ui,
                                option,
                                &visual.font,
                                visual.ink,
                                f32::INFINITY,
                                egui::pos2(
                                    item.rect.left() + ui.spacing().button_padding.x,
                                    item.rect.center().y - height / 2.0,
                                ),
                            );
                            if item.clicked() {
                                emit_result = event.emit(option.clone());
                            }
                        }
                    })
            })
            .inner;
        let shaped = inrux_text::shape(
            value,
            visual.font.size,
            style.font_family.as_deref(),
            f32::INFINITY,
        );
        crate::typography::paint(
            ui,
            value,
            &visual.font,
            visual.ink,
            f32::INFINITY,
            egui::pos2(
                response.response.rect.left() + ui.spacing().button_padding.x,
                response.response.rect.center().y - shaped.height() / 2.0,
            ),
        );
        ui.ctx()
            .data_mut(|data| data.insert_temp(path.with("widget-id"), response.response.id));
    });
    emit_result
}
