//! Cached, straight-alpha sRGBA glyph tiles shared by every native upload bridge.
use super::*;
use parley::{Brush, Layout, PositionedLayoutItem};
use std::hash::{Hash, Hasher};
use swash::{
    scale::{Render, ScaleContext, Source, StrikeWith, image::Content},
    zeno::{Format, Vector},
};
/// One bounded device-resolution glyph tile, with logical positioning preserved.
pub struct Bitmap {
    /// Stable cache identity for native texture uploads.
    pub id: u64,
    /// Physical width.
    pub width: u32,
    /// Physical height.
    pub height: u32,
    /// Logical offset from the shaped line box origin.
    pub offset: [f32; 2],
    /// Device pixels per logical pixel.
    pub scale: f32,
    /// Straight-alpha sRGBA pixels.
    pub rgba: Arc<[u8]>,
}
type Tiles = Vec<Arc<Bitmap>>;
type RasterKey = (u64, u32, [u8; 4], [u32; 4]);
#[derive(Default)]
pub(super) struct Cache {
    scaler: ScaleContext,
    fonts: HashMap<u64, swash::CacheKey>,
    bitmaps: HashMap<RasterKey, Tiles>,
    bytes: usize,
}
impl Cache {
    pub(super) fn clear(&mut self) {
        *self = Self::default();
    }
}
/// Draw the complete shape as bounded tiles, including glyph overhang.
pub fn rasterize(shaped: &Shaped, scale: f32, ink: [u8; 4]) -> Tiles {
    let pad = shaped.font_size;
    rasterize_region(
        shaped,
        scale,
        ink,
        [
            -pad,
            -pad,
            shaped.width() + pad * 2.0,
            shaped.height() + pad * 2.0,
        ],
    )
}
/// Draw only a visible logical region, allowing long editors to pan without enormous textures.
pub fn rasterize_region(shaped: &Shaped, scale: f32, ink: [u8; 4], region: [f32; 4]) -> Tiles {
    CONTEXT.with_borrow_mut(|context| {
        cached(
            &shaped.layout,
            shaped.id,
            scale,
            ink,
            region,
            &mut context.raster,
        )
    })
}
/// Rasterize a native Parley editor/label layout with the common coverage policy.
pub fn rasterize_layout<B: Brush>(
    layout: &Layout<B>,
    scale: f32,
    ink: [u8; 4],
    region: Option<[f32; 4]>,
) -> Tiles {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    layout.full_width().to_bits().hash(&mut hasher);
    layout.height().to_bits().hash(&mut hasher);
    let mut pad = 15.0_f32;
    for line in layout.lines() {
        for item in line.items() {
            if let PositionedLayoutItem::GlyphRun(glyphs) = item {
                pad = pad.max(glyphs.run().font_size());
                glyphs.run().font().data.id().hash(&mut hasher);
                glyphs.run().font_size().to_bits().hash(&mut hasher);
                glyphs.run().normalized_coords().hash(&mut hasher);
                glyphs.offset().to_bits().hash(&mut hasher);
                glyphs.baseline().to_bits().hash(&mut hasher);
                for glyph in glyphs.glyphs() {
                    glyph.id.hash(&mut hasher);
                    glyph.x.to_bits().hash(&mut hasher);
                    glyph.y.to_bits().hash(&mut hasher);
                    glyph.advance.to_bits().hash(&mut hasher);
                }
            }
        }
    }
    let identity = hasher.finish() | (1 << 63);
    let region = region.unwrap_or([
        -pad,
        -pad,
        layout.full_width() + pad * 2.0,
        layout.height() + pad * 2.0,
    ]);
    CONTEXT.with_borrow_mut(|context| {
        cached(layout, identity, scale, ink, region, &mut context.raster)
    })
}
/// Cache viewport-aligned tiles; the byte budget also bounds retained page/edit history.
fn cached<B: Brush>(
    layout: &Layout<B>,
    identity: u64,
    scale: f32,
    ink: [u8; 4],
    region: [f32; 4],
    cache: &mut Cache,
) -> Tiles {
    let scale = if scale.is_finite() {
        scale.clamp(0.25, 8.0)
    } else {
        1.0
    };
    if region.iter().any(|v| !v.is_finite()) || region[2] <= 0.0 || region[3] <= 0.0 {
        return vec![];
    }
    let mut pad = 1.0_f32;
    for line in layout.lines() {
        for item in line.items() {
            if let PositionedLayoutItem::GlyphRun(glyphs) = item {
                pad = pad.max(glyphs.run().font_size() * 2.0);
            }
        }
    }
    let x = region[0].max(-pad);
    let y = region[1].max(-pad);
    let right = (region[0] + region[2]).min(layout.full_width() + pad);
    let bottom = (region[1] + region[3]).min(layout.height() + pad);
    if right <= x || bottom <= y {
        return vec![];
    }
    let region = [x, y, right - x, bottom - y];
    let left = (region[0] * scale).floor() as i32;
    let top = (region[1] * scale).floor() as i32;
    let right = ((region[0] + region[2]) * scale).ceil() as i32;
    let bottom = ((region[1] + region[3]) * scale).ceil() as i32;
    let key = (
        identity,
        scale.to_bits(),
        ink,
        [left as u32, top as u32, right as u32, bottom as u32],
    );
    if let Some(tiles) = cache.bitmaps.get(&key) {
        return tiles.clone();
    }
    let mut tiles = Vec::new();
    let mut y = top;
    while y < bottom {
        let mut x = left;
        let height = (bottom - y).min(1024) as u32;
        while x < right {
            let width = (right - x).min(2048) as u32;
            if let Some(tile) = draw(
                layout,
                scale,
                ink,
                [x, y, width as i32, height as i32],
                cache,
            ) {
                tiles.push(Arc::new(tile));
            }
            x += width as i32;
        }
        y += height as i32;
    }
    let bytes = tiles.iter().map(|tile| tile.rgba.len()).sum::<usize>();
    if cache.bytes + bytes > 16 * 1024 * 1024 {
        cache.bitmaps.clear();
        cache.bytes = 0;
    }
    if bytes <= 16 * 1024 * 1024 {
        cache.bytes += bytes;
        cache.bitmaps.insert(key, tiles.clone());
    }
    tiles
}
/// Draw and tightly crop one tile. Subpixel positions and color-glyph alpha are engine-independent.
fn draw<B: Brush>(
    layout: &Layout<B>,
    scale: f32,
    ink: [u8; 4],
    tile: [i32; 4],
    cache: &mut Cache,
) -> Option<Bitmap> {
    let [left, top, width, height] = tile;
    let mut rgba = vec![0u8; width as usize * height as usize * 4];
    let mut bounds = [width, height, 0, 0];
    for line in layout.lines() {
        if line.metrics().max_coord * scale + 128.0 < top as f32
            || line.metrics().min_coord * scale - 128.0 > (top + height) as f32
        {
            continue;
        }
        for item in line.items() {
            let PositionedLayoutItem::GlyphRun(glyphs) = item else {
                continue;
            };
            let run = glyphs.run();
            let font = run.font();
            let Some(mut face) =
                swash::FontRef::from_index(font.data.as_ref(), font.index as usize)
            else {
                continue;
            };
            face.key = *cache.fonts.entry(font.data.id()).or_default();
            let mut scaler = cache
                .scaler
                .builder(face)
                .size(run.font_size() * scale)
                .hint(false)
                .normalized_coords(run.normalized_coords())
                .build();
            let mut x = glyphs.offset();
            for glyph in glyphs.glyphs() {
                let gx = (x + glyph.x) * scale - left as f32;
                let gy = (glyphs.baseline() - glyph.y) * scale - top as f32;
                x += glyph.advance;
                let em = run.font_size() * scale * 2.0;
                if gx + em < 0.0
                    || gx - em > width as f32
                    || gy + em < 0.0
                    || gy - em > height as f32
                {
                    continue;
                }
                let Some(image) = Render::new(&[
                    Source::ColorOutline(0),
                    Source::ColorBitmap(StrikeWith::BestFit),
                    Source::Outline,
                ])
                .format(Format::Alpha)
                .offset(Vector::new(gx - gx.floor(), gy - gy.floor()))
                .render(&mut scaler, glyph.id as u16) else {
                    continue;
                };
                let bx = gx.floor() as i32 + image.placement.left;
                let by = gy.floor() as i32 - image.placement.top;
                for row in 0..image.placement.height {
                    for col in 0..image.placement.width {
                        let (px, py) = (bx + col as i32, by + row as i32);
                        if px < 0 || py < 0 || px >= width || py >= height {
                            continue;
                        }
                        let src = (row * image.placement.width + col) as usize;
                        let color = if image.content == Content::Color {
                            let p = &image.data[src * 4..src * 4 + 4];
                            [
                                p[0],
                                p[1],
                                p[2],
                                ((u16::from(p[3]) * u16::from(ink[3])) / 255) as u8,
                            ]
                        } else {
                            [
                                ink[0],
                                ink[1],
                                ink[2],
                                ((u16::from(image.data[src]) * u16::from(ink[3])) / 255) as u8,
                            ]
                        };
                        let a = u32::from(color[3]);
                        if a == 0 {
                            continue;
                        }
                        bounds[0] = bounds[0].min(px);
                        bounds[1] = bounds[1].min(py);
                        bounds[2] = bounds[2].max(px + 1);
                        bounds[3] = bounds[3].max(py + 1);
                        let dst = ((py * width + px) * 4) as usize;
                        let old = u32::from(rgba[dst + 3]);
                        let out = a * 255 + old * (255 - a);
                        for c in 0..3 {
                            rgba[dst + c] = ((u32::from(color[c]) * a * 255
                                + u32::from(rgba[dst + c]) * old * (255 - a))
                                / out) as u8;
                        }
                        rgba[dst + 3] = (out / 255) as u8;
                    }
                }
            }
        }
    }
    let [x0, y0, x1, y1] = bounds;
    if x1 <= x0 || y1 <= y0 {
        return None;
    }
    // Native image primitives antialias their rectangle edges. Keep those edges
    // transparent so glyph coverage is not multiplied by a second edge mask.
    let (x0, y0, x1, y1) = (
        (x0 - 1).max(0),
        (y0 - 1).max(0),
        (x1 + 1).min(width),
        (y1 + 1).min(height),
    );
    let mut cropped = Vec::with_capacity(((x1 - x0) * (y1 - y0) * 4) as usize);
    for y in y0..y1 {
        cropped.extend_from_slice(
            &rgba[((y * width + x0) * 4) as usize..((y * width + x1) * 4) as usize],
        );
    }
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    Some(Bitmap {
        id: NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
        width: (x1 - x0) as u32,
        height: (y1 - y0) as u32,
        offset: [(left + x0) as f32 / scale, (top + y0) as f32 / scale],
        scale,
        rgba: cropped.into(),
    })
}
