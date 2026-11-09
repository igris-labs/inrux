//! Shared shaping projected into egui editor and selection metadata.
pub(crate) use crate::text_image::paint;
use eframe::egui;
/// Use the same font and line metrics for intrinsic measurement and editable text.
pub(crate) fn layout(
    ui: &egui::Ui,
    text: &str,
    font: egui::FontId,
    ink: egui::Color32,
    width: f32,
) -> std::sync::Arc<egui::Galley> {
    let family = match &font.family {
        egui::FontFamily::Name(name) => Some(name.as_ref()),
        _ => None,
    };
    let shaped = inrux_text::shape(text, font.size, family, width);
    // Reuse one invisible native glyph as a metadata template. The content itself
    // is shaped and rasterized only by the shared provider.
    let native_job = egui::text::LayoutJob::simple(" ".into(), font.clone(), ink, f32::INFINITY);
    let native = ui.fonts_mut(|fonts| fonts.layout_job(native_job));
    let mut galley = (*native).clone();
    galley.job = std::sync::Arc::new(egui::text::LayoutJob::simple(text.into(), font, ink, width));
    let template = galley.rows.first().cloned();
    let glyph_template = template
        .as_ref()
        .and_then(|row| row.glyphs.first())
        .copied();
    let mut rows = Vec::new();
    if let Some(template) = template {
        let mut row_top = 0.0;
        for line in shaped.layout.lines() {
            let range = line.text_range();
            let slice = text.get(range.clone()).unwrap_or("");
            let ends_with_newline = slice.ends_with('\n');
            let visible = slice.strip_suffix('\n').unwrap_or(slice);
            let mut placed = template.clone();
            let row = std::sync::Arc::make_mut(&mut placed.row);
            row.glyphs.clear();
            for (offset, ch) in visible.char_indices() {
                let index = range.start + offset;
                if let Some(mut glyph) = glyph_template {
                    glyph.chr = ch;
                    let here = shaped.caret(index);
                    let next = shaped.caret(index + ch.len_utf8());
                    glyph.pos = egui::pos2(here[0], line.metrics().baseline - row_top);
                    glyph.advance_width = if next[1] == here[1] {
                        next[0] - here[0]
                    } else {
                        line.metrics().advance - here[0]
                    };
                    glyph.line_height = here[3];
                    glyph.font_height = here[3];
                    glyph.font_impl_height = here[3];
                    glyph.font_ascent = line.metrics().baseline - row_top;
                    glyph.font_impl_ascent = glyph.font_ascent;
                    row.glyphs.push(glyph);
                }
            }
            row.size = egui::vec2(line.metrics().advance, line.metrics().line_height);
            row.ends_with_newline = ends_with_newline;
            row.visuals = Default::default();
            placed.pos = egui::pos2(0.0, row_top);
            row_top += line.metrics().line_height;
            rows.push(placed);
        }
    }
    galley.rows = rows;
    galley.rect = egui::Rect::from_min_size(
        egui::Pos2::ZERO,
        egui::vec2(shaped.width(), shaped.height()),
    );
    galley.mesh_bounds = egui::Rect::NOTHING;
    galley.num_indices = 0;
    galley.num_vertices = 0;
    std::sync::Arc::new(galley)
}

/// Center font ascent/descent in the authored line box, matching native shaping services.
pub(crate) fn font_data(
    asset: &inrux_adapter::FontAsset,
) -> Result<egui::FontData, inrux_adapter::AdapterError> {
    let face = ttf_parser::Face::parse(&asset.data, 0).map_err(|error| {
        inrux_adapter::AdapterError::Configuration(format!(
            "Cannot load font '{}': {error:?}",
            asset.family
        ))
    })?;
    let em_height =
        (f32::from(face.ascender()) - f32::from(face.descender())) / f32::from(face.units_per_em());
    let mut data = egui::FontData::from_owned(asset.data.to_vec());
    // egui starts glyphs at their ascent; native shaping centers excess leading (including negative leading).
    data.tweak.y_offset_factor = (inrux_adapter::controls::LINE_HEIGHT - em_height) / 2.0;
    Ok(data)
}

/// Configure shared font assets and egui metadata before rendering an embedded Inrux tree.
/// Call on the rendering thread; normal Inrux windows do this during startup.
pub fn configure_fonts(
    context: &egui::Context,
    assets: &[inrux_adapter::FontAsset],
) -> Result<(), inrux_adapter::AdapterError> {
    context.set_fonts(definitions(assets)?);
    Ok(())
}

/// Register application faces while preserving the native host's missing-glyph fallback chain.
pub(crate) fn definitions(
    assets: &[inrux_adapter::FontAsset],
) -> Result<egui::FontDefinitions, inrux_adapter::AdapterError> {
    inrux_text::register_fonts(assets.iter().map(|asset| asset.data.as_ref()))
        .map_err(inrux_adapter::AdapterError::Configuration)?;
    let mut fonts = egui::FontDefinitions::default();
    let fallback = fonts
        .families
        .get(&egui::FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();
    let default =
        inrux_adapter::FontAsset::new(inrux_text::DEFAULT_FAMILY, inrux_text::DEFAULT_FONT);
    for asset in std::iter::once(&default).chain(assets) {
        fonts
            .font_data
            .insert(asset.family.clone(), std::sync::Arc::new(font_data(asset)?));
        let mut faces = vec![asset.family.clone()];
        faces.extend(
            fallback
                .iter()
                .filter(|name| **name != asset.family)
                .cloned(),
        );
        fonts
            .families
            .insert(egui::FontFamily::Name(asset.family.clone().into()), faces);
    }
    fonts.families.insert(
        egui::FontFamily::Proportional,
        vec![inrux_text::DEFAULT_FAMILY.into()],
    );
    Ok(fonts)
}

/// Diagnose unresolved named families before native measurement or fixed-size paint can panic.
pub(crate) fn validate_families(
    node: &inrux_components::Node,
    ui: &egui::Ui,
) -> Result<(), inrux_adapter::AdapterError> {
    /// Inspect one resolved subtree against the native host's registered family names.
    fn walk(
        node: &inrux_components::Node,
        families: &[egui::FontFamily],
    ) -> Result<(), inrux_adapter::AdapterError> {
        for family in node
            .style()
            .font_family
            .into_iter()
            .chain(
                node.part_rules
                    .iter()
                    .filter_map(|(_, _, style)| style.font_family.clone()),
            )
            .chain(
                node.part_inline
                    .iter()
                    .filter_map(|(_, style)| style.font_family.clone()),
            )
        {
            if !families.contains(&egui::FontFamily::Name(family.clone().into())) {
                return Err(inrux_adapter::AdapterError::Unsupported {
                    backend: "egui",
                    feature: format!(
                        "unregistered font family `{family}`; add its font bytes to WindowOptions.fonts"
                    ),
                });
            }
        }
        for child in &node.children {
            walk(child, families)?;
        }
        Ok(())
    }
    walk(node, &ui.fonts_mut(|fonts| fonts.families()))
}

#[cfg(test)]
mod tests {
    use super::*;
    /// Native editor rows carry shared caret metrics and no independently rasterized content mesh.
    #[test]
    fn editor_metadata_matches_shared_utf8_geometry() {
        let context = egui::Context::default();
        let text = "Rust café 🦀";
        let _ = context.run(egui::RawInput::default(), |context| {
            egui::CentralPanel::default().show(context, |ui| {
                let galley = layout(
                    ui,
                    text,
                    egui::FontId::proportional(15.0),
                    egui::Color32::BLACK,
                    f32::INFINITY,
                );
                let shared = inrux_text::shape(text, 15.0, None, f32::INFINITY);
                assert_eq!(galley.text(), text);
                assert_eq!(galley.size(), egui::vec2(shared.width(), shared.height()));
                assert_eq!(galley.num_vertices, 0);
                for (character, (byte, _)) in text.char_indices().enumerate() {
                    let native = galley.pos_from_cursor(egui::text::CCursor::new(character));
                    let expected = shared.caret(byte);
                    assert!((native.left() - expected[0]).abs() < 0.001);
                    assert!((native.height() - expected[3]).abs() < 0.001);
                }
            });
        });
    }
    /// The shared scene's tall Noto metrics need negative leading, not a hard-coded pixel offset.
    #[test]
    fn bundled_font_uses_font_relative_leading() {
        let asset = inrux_adapter::FontAsset::new(
            "Noto Sans",
            include_bytes!("../tests/assets/fonts/NotoSans-Regular.ttf").as_slice(),
        );
        let data = font_data(&asset).unwrap();
        assert!((data.tweak.y_offset_factor + 0.081).abs() < 0.001);
    }
    /// Report a malformed font before passing it to egui's infallible font loader.
    #[test]
    fn malformed_font_returns_configuration_error() {
        let asset = inrux_adapter::FontAsset::new("Broken", [1, 2, 3].as_slice());
        assert!(matches!(
            font_data(&asset),
            Err(inrux_adapter::AdapterError::Configuration(_))
        ));
    }
    /// A named application face must not hide glyphs available in the host fallback fonts.
    #[test]
    fn application_font_preserves_missing_glyph_fallback() {
        let asset = inrux_adapter::FontAsset::new(
            "Noto Sans",
            include_bytes!("../tests/assets/fonts/NotoSans-Regular.ttf").as_slice(),
        );
        let primary = ttf_parser::Face::parse(&asset.data, 0).unwrap();
        let defaults = egui::FontDefinitions::default();
        let glyph = ['😀', '🦀', '🚀', '🙂', '❤', '☆']
            .into_iter()
            .find(|ch| {
                primary.glyph_index(*ch).is_none()
                    && defaults.font_data.values().any(|data| {
                        ttf_parser::Face::parse(&data.font, data.index)
                            .unwrap()
                            .glyph_index(*ch)
                            .is_some()
                    })
            })
            .expect("expected a glyph supplied only by a bundled host fallback");
        let context = egui::Context::default();
        context.set_fonts(definitions(&[asset]).unwrap());
        let _ = context.run(egui::RawInput::default(), |context| {
            assert!(context.fonts_mut(|fonts| fonts.has_glyph(
                &egui::FontId::new(15.0, egui::FontFamily::Name("Noto Sans".into())),
                glyph
            )));
        });
    }
}
