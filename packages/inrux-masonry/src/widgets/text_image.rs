//! Device-scale shared glyph surfaces uploaded through Vello's native image primitive.
use masonry::{
    kurbo::Affine,
    parley::Layout,
    peniko::{Blob, ImageAlphaType, ImageData, ImageFormat},
    vello::Scene,
};
use std::{cell::RefCell, collections::HashMap, sync::Arc};
thread_local! { static IMAGES: RefCell<HashMap<u64, ImageData>> = RefCell::new(HashMap::new()); }
/// Paint one immutable layout while preserving the editor's native clip, caret and selection transform.
pub(crate) fn paint_layout<B: masonry::parley::Brush>(
    scene: &mut Scene,
    transform: Affine,
    layout: &Layout<B>,
    scale: f32,
    ink: masonry::peniko::Color,
    region: Option<[f32; 4]>,
) {
    let ink = ink.to_rgba8();
    for bitmap in inrux_text::rasterize_layout(layout, scale, [ink.r, ink.g, ink.b, ink.a], region)
    {
        IMAGES.with_borrow_mut(|images| {
            if images.values().map(|image| image.data.len()).sum::<usize>() + bitmap.rgba.len()
                > 16 * 1024 * 1024
            {
                images.clear();
            }
            let image = images.entry(bitmap.id).or_insert_with(|| ImageData {
                data: Blob::new(Arc::new(bitmap.rgba.clone())),
                format: ImageFormat::Rgba8,
                alpha_type: ImageAlphaType::Alpha,
                width: bitmap.width,
                height: bitmap.height,
            });
            scene.draw_image(
                &masonry::peniko::ImageBrush::new(image.clone()),
                image_transform(transform, bitmap.offset, bitmap.scale),
            );
        });
    }
}

/// Vello 0.6 samples images at integer device coordinates, then subtracts half a texel.
/// Offset the bitmap mapping by half a device pixel to match the other image primitives.
fn image_transform(transform: Affine, offset: [f32; 2], scale: f32) -> Affine {
    let pixel = 1.0 / f64::from(scale);
    transform
        * Affine::translate((
            f64::from(offset[0]) - 0.5 * pixel,
            f64::from(offset[1]) - 0.5 * pixel,
        ))
        * Affine::scale(pixel)
}
#[cfg(test)]
mod tests {
    use super::*;
    /// A source texel center maps to the common glyph origin at every physical scale.
    #[test]
    fn device_texel_centers_preserve_the_common_glyph_origin() {
        for scale in [1.0, 1.25, 2.0, 3.0] {
            let transform = Affine::scale(f64::from(scale))
                * image_transform(Affine::translate((13.125, 7.5)), [2.0, 3.0], scale);
            let origin = transform * masonry::kurbo::Point::new(0.5, 0.5);
            assert!((origin.x - 15.125 * f64::from(scale)).abs() < 0.0001);
            assert!((origin.y - 10.5 * f64::from(scale)).abs() < 0.0001);
        }
    }
}
