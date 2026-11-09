//! Native egui input observation; framework/application state is not involved.
use eframe::egui;
use inrux_adapter::FocusVisibility;
/// Read only the last modality; never clone text/IME payloads just to decide focus paint.
pub(crate) fn observe(context: &egui::Context) {
    let keyboard = context.input(|input| {
        input.events.iter().rev().find_map(|event| match event {
            egui::Event::PointerButton { pressed: true, .. } => Some(false),
            egui::Event::Key { pressed: true, .. } => Some(true),
            _ => None,
        })
    });
    if let Some(keyboard) = keyboard {
        context.data_mut(|data| {
            let source = data
                .get_temp_mut_or_default::<FocusVisibility>(egui::Id::new("inrux-focus-visible"));
            if keyboard {
                source.keyboard();
            } else {
                source.pointer();
            }
        });
    }
}
/// Retain pointer focus for Enter/Space; only its outline is suppressed.
pub(crate) fn visible(context: &egui::Context) -> bool {
    context.data(|data| {
        data.get_temp::<FocusVisibility>(egui::Id::new("inrux-focus-visible"))
            .unwrap_or_default()
            .visible()
    })
}
