//! Cached native uploads for shared device-scale glyph tiles.
use eframe::egui;
/// Paint common glyph pixels while native widgets retain their input/accessibility semantics.
pub(crate) fn paint(
    ui: &egui::Ui,
    text: &str,
    font: &egui::FontId,
    ink: egui::Color32,
    width: f32,
    origin: egui::Pos2,
) {
    let family = match &font.family {
        egui::FontFamily::Name(name) => Some(name.as_ref()),
        _ => None,
    };
    let shaped = inrux_text::shape(text, font.size, family, width);
    let clip = ui.clip_rect();
    let region = [
        clip.left() - origin.x,
        clip.top() - origin.y,
        clip.width(),
        clip.height(),
    ];
    for bitmap in inrux_text::rasterize_region(
        &shaped,
        ui.ctx().pixels_per_point(),
        ink.to_srgba_unmultiplied(),
        region,
    ) {
        let id = egui::Id::new("inrux-shared-text-textures");
        let existing = ui.ctx().data_mut(|data| {
            data.get_temp_mut_or_default::<std::collections::HashMap<u64, egui::TextureHandle>>(id)
                .get(&bitmap.id)
                .cloned()
        });
        let texture = existing.unwrap_or_else(|| {
            let texture = ui.ctx().load_texture(
                format!("inrux-text-{}", bitmap.id),
                egui::ColorImage::from_rgba_unmultiplied(
                    [bitmap.width as usize, bitmap.height as usize],
                    &bitmap.rgba,
                ),
                egui::TextureOptions::LINEAR,
            );
            ui.ctx().data_mut(|data| {
                let cache = data
                    .get_temp_mut_or_default::<std::collections::HashMap<u64, egui::TextureHandle>>(
                        id,
                    );
                if cache
                    .values()
                    .map(|image| image.size()[0] * image.size()[1] * 4)
                    .sum::<usize>()
                    + bitmap.rgba.len()
                    > 16 * 1024 * 1024
                {
                    cache.clear();
                }
                cache.insert(bitmap.id, texture.clone());
            });
            texture
        });
        let rect = egui::Rect::from_min_size(
            origin + egui::vec2(bitmap.offset[0], bitmap.offset[1]),
            egui::vec2(
                bitmap.width as f32 / bitmap.scale,
                bitmap.height as f32 / bitmap.scale,
            ),
        );
        ui.painter().image(
            texture.id(),
            rect,
            egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        );
    }
}
