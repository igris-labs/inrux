//! Upload shared glyph pixels through GPUI's ordinary image primitive.
use gpui::{Bounds, Corners, Pixels, Point, RenderImage, Window, point, px, size};
use std::{cell::RefCell, collections::HashMap, sync::Arc};
thread_local! { static IMAGES: RefCell<HashMap<u64, Arc<RenderImage>>> = RefCell::new(HashMap::new()); }
/// All glyphs, controls and editor text use the same straight-alpha sRGBA pixels.
pub(crate) fn paint(
    shaped: &inrux_text::Shaped,
    origin: Point<Pixels>,
    ink: [u8; 4],
    window: &mut Window,
) -> Result<(), String> {
    let clip = window.content_mask().bounds;
    let region = [
        f32::from(clip.origin.x - origin.x),
        f32::from(clip.origin.y - origin.y),
        f32::from(clip.size.width),
        f32::from(clip.size.height),
    ];
    for bitmap in inrux_text::rasterize_region(shaped, window.scale_factor(), ink, region) {
        IMAGES.with_borrow_mut(|images| {
            if images
                .values()
                .map(|image| image.as_bytes(0).map_or(0, |data| data.len()))
                .sum::<usize>()
                + bitmap.rgba.len()
                > 16 * 1024 * 1024
            {
                for (_, image) in images.drain() {
                    window
                        .drop_image(image)
                        .map_err(|error| error.to_string())?;
                }
            }
            let image = images.entry(bitmap.id).or_insert_with(|| {
                let mut pixels = bitmap.rgba.to_vec();
                for pixel in pixels.as_chunks_mut::<4>().0 {
                    pixel.swap(0, 2);
                }
                let data = image::RgbaImage::from_raw(bitmap.width, bitmap.height, pixels)
                    .expect("shared raster dimensions match its pixel buffer");
                Arc::new(RenderImage::new(vec![image::Frame::new(data)]))
            });
            window
                .paint_image(
                    Bounds::new(
                        origin + point(px(bitmap.offset[0]), px(bitmap.offset[1])),
                        size(
                            px(bitmap.width as f32 / bitmap.scale),
                            px(bitmap.height as f32 / bitmap.scale),
                        ),
                    ),
                    Corners::default(),
                    image.clone(),
                    0,
                    false,
                )
                .map_err(|error| error.to_string())
        })?;
    }
    Ok(())
}
