//! Font, cluster, raster, scale and clipping contracts, independent of any window toolkit.
use super::*;
/// Shaping remains proportional, reuses immutable layouts, and maps caret hit tests to UTF-8 boundaries.
#[test]
fn proportional_metrics_and_utf8_caret_geometry_are_shared() {
    let narrow = shape("iiii", 15.0, None, f32::INFINITY);
    let wide = shape("WWWW", 15.0, None, f32::INFINITY);
    assert!(wide.width() > narrow.width() * 2.0);
    assert!(Arc::ptr_eq(
        &wide,
        &shape("WWWW", 15.0, None, f32::INFINITY)
    ));
    let text = shape("Rust café", 15.0, None, f32::INFINITY);
    for (index, _) in text.text.char_indices() {
        let caret = text.caret(index);
        assert_eq!(text.hit(caret[0] + 0.01, caret[1] + caret[3] * 0.5), index);
        assert_eq!(caret[1], 0.0);
        assert!((caret[3] - 18.0).abs() < 0.001);
    }
}
/// Wrapping uses the same exact line boxes at every device scale, including empty paragraphs.
#[test]
fn multiline_wrapping_and_empty_lines_preserve_line_boxes() {
    let text = shape("alpha beta gamma delta\n\nlast", 15.0, None, 70.0);
    assert!(text.layout.len() > 3);
    assert!((text.height() - text.layout.len() as f32 * 18.0).abs() < 0.001);
    let caret = text.caret(text.text.len());
    assert!(caret[1] + caret[3] <= text.height() + 0.001);
    assert!(text.selection(0..text.text.len()).len() > 1);
}
/// Coverage is cached at physical scale; alpha-zero paint cannot leave visible glyphs.
#[test]
fn physical_rasters_reuse_upload_identity_and_preserve_alpha() {
    let text = shape("office AV café", 18.0, None, f32::INFINITY);
    let one = rasterize(&text, 1.0, [20, 30, 40, 255]);
    let warm = rasterize(&text, 1.0, [20, 30, 40, 255]);
    assert!(!one.is_empty());
    let tile = &one[0];
    let top = &tile.rgba[..tile.width as usize * 4];
    let bottom = &tile.rgba[(tile.height as usize - 1) * tile.width as usize * 4..];
    assert!(top.as_chunks::<4>().0.iter().all(|pixel| pixel[3] == 0));
    assert!(bottom.as_chunks::<4>().0.iter().all(|pixel| pixel[3] == 0));
    assert_eq!(one[0].id, warm[0].id);
    let two = rasterize(&text, 2.0, [20, 30, 40, 255]);
    assert!(two[0].width > one[0].width);
    assert_ne!(one[0].id, two[0].id);
    assert!(rasterize(&text, 1.0, [20, 30, 40, 0]).is_empty());
}
/// Visible tail glyphs remain renderable even when the logical text exceeds GPU texture limits.
#[test]
fn long_editor_regions_do_not_truncate_the_tail_or_allocate_full_width() {
    let text = shape(&"Rust text ".repeat(2500), 15.0, None, f32::INFINITY);
    assert!(text.width() > 16384.0);
    let region = [text.width() - 200.0, 0.0, 200.0, text.height()];
    let tiles = rasterize_region(&text, 2.0, [0, 0, 0, 255], region);
    assert!(!tiles.is_empty());
    assert!(
        tiles
            .iter()
            .all(|tile| tile.width <= 400 && tile.height <= 36)
    );
    assert!(tiles.iter().all(|tile| tile.offset[0] >= region[0] - 1.0));
}
/// Re-clipping glyphs across a tile boundary must preserve identical coverage and subpixel positions.
#[test]
fn tiled_and_viewport_rasterization_agree_pixel_for_pixel() {
    let text = shape(&"AV office café ".repeat(40), 15.0, None, f32::INFINITY);
    let full = rasterize(&text, 2.0, [0, 0, 0, 255]);
    assert!(full.len() > 1);
    let region = [1010.0, 0.0, 80.0, text.height()];
    let clipped = rasterize_region(&text, 2.0, [0, 0, 0, 255], region);
    let pixels = |tiles: &[Arc<Bitmap>]| {
        let mut result = std::collections::BTreeMap::new();
        for tile in tiles {
            for y in 0..tile.height {
                for x in 0..tile.width {
                    let gx = (tile.offset[0] * 2.0).round() as i32 + x as i32;
                    let gy = (tile.offset[1] * 2.0).round() as i32 + y as i32;
                    if (2020..2180).contains(&gx) && (0..36).contains(&gy) {
                        let p = ((y * tile.width + x) * 4) as usize;
                        if tile.rgba[p + 3] > 0 {
                            result.insert((gx, gy), tile.rgba[p..p + 4].to_vec());
                        }
                    }
                }
            }
        }
        result
    };
    assert_eq!(pixels(&full), pixels(&clipped));
}

/// Logical selection boxes share the same height as the caret and do not bleed into adjacent rows.
#[test]
fn selections_follow_logical_lines_without_font_metric_overflow() {
    let text = shape("first\nsecond", 15.0, None, f32::INFINITY);
    let rects = text.selection(0..text.text.len());
    assert_eq!(rects.len(), 2);
    assert_eq!(rects[0][1], 0.0);
    assert_eq!(rects[0][3], 18.0);
    assert_eq!(rects[1][1], 18.0);
    assert_eq!(rects[1][3], 18.0);
}

/// Opt-in isolated provider timing; excludes native upload, event handling and GPU presentation.
#[test]
#[ignore = "release provider timing specimen; run with --release --ignored --nocapture"]
fn shared_text_cache_timing() {
    let text =
        "Inrux shared typography: office AV café, proportional body text and control labels.";
    let start = std::time::Instant::now();
    let shaped = shape(text, 15.0, None, 500.0);
    let cold = start.elapsed();
    let start = std::time::Instant::now();
    let first = rasterize(&shaped, 2.0, [27, 30, 33, 255]);
    let raster = start.elapsed();
    let start = std::time::Instant::now();
    for _ in 0..1000 {
        let layout = shape(text, 15.0, None, 500.0);
        let pixels = rasterize(&layout, 2.0, [27, 30, 33, 255]);
        assert_eq!(pixels[0].id, first[0].id);
        std::hint::black_box(pixels);
    }
    println!(
        "cold_shape_us={} first_raster_us={} warm_shape_and_raster_ns={} retained_rgba_bytes={}",
        cold.as_micros(),
        raster.as_micros(),
        start.elapsed().as_nanos() / 1000,
        first.iter().map(|image| image.rgba.len()).sum::<usize>()
    );
}

/// Invalid font data is rejected consistently before it can enter native glyph loaders.
#[test]
fn malformed_font_registration_preserves_existing_cached_shapes() {
    let before = shape("valid cached text", 15.0, None, f32::INFINITY);
    assert!(register_fonts([&[1, 2, 3][..]]).is_err());
    assert!(Arc::ptr_eq(
        &before,
        &shape("valid cached text", 15.0, None, f32::INFINITY)
    ));
    register_fonts([DEFAULT_FONT]).unwrap();
    assert!(Arc::ptr_eq(
        &before,
        &shape("valid cached text", 15.0, None, f32::INFINITY)
    ));
}

/// Marked text keeps visible underline geometry when native glyph painting is replaced.
#[test]
fn ime_underline_uses_shared_font_metrics_and_visual_ranges() {
    let text = shape("Rust café", 15.0, None, f32::INFINITY);
    let underlines = underline_rects(&text.layout, 5..text.text.len());
    assert_eq!(underlines.len(), 1);
    assert!((underlines[0][0] - text.caret(5)[0]).abs() < 0.001);
    assert!(underlines[0][1] > 0.0 && underlines[0][1] < text.height());
    assert_eq!(underlines[0][3], 1.0);
    assert!(underline_rects(&text.layout, 0..0).is_empty());
}
