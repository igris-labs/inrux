//! Exact logical metrics and cluster geometry shared by layout, paint and editor bridges.
use super::*;
use parley::{Cursor, FontFamily, FontStack, Layout, LineHeight, StyleProperty};
/// Immutable text layout. Native handles and display scale never enter logical geometry.
pub struct Shaped {
    /// Stable identity used by private texture caches.
    pub id: u64,
    /// Original UTF-8 content.
    pub text: String,
    /// Exact shaped glyphs and cluster geometry.
    pub layout: Layout<usize>,
    /// Requested logical font size.
    pub font_size: f32,
}
impl Shaped {
    /// Natural logical width, including trailing whitespace.
    pub fn width(&self) -> f32 {
        self.layout.full_width()
    }
    /// Logical height including all wrapped lines.
    pub fn height(&self) -> f32 {
        self.layout.height()
    }
    /// Caret rectangle for a UTF-8 byte boundary: x, y, width, height.
    pub fn caret(&self, index: usize) -> [f32; 4] {
        super::caret_rect(&self.layout, index, 1.0)
    }
    /// Visual selection rectangles for a UTF-8 byte range, including bidirectional runs.
    pub fn selection(&self, range: std::ops::Range<usize>) -> Vec<[f32; 4]> {
        super::selection_rects(&self.layout, range)
    }
    /// Hit-test logical coordinates to a valid UTF-8 cluster boundary.
    pub fn hit(&self, x: f32, y: f32) -> usize {
        Cursor::from_point(&self.layout, x, y).index()
    }
}
#[derive(Clone, Hash, Eq, PartialEq)]
pub(super) struct Key {
    text: String,
    size: u32,
    family: String,
    width: u32,
}
/// Shape without device-pixel quantization; scale is applied only during rasterization.
pub fn shape(text: &str, size: f32, family: Option<&str>, width: f32) -> Arc<Shaped> {
    let family = family.unwrap_or(DEFAULT_FAMILY);
    let size = if size.is_finite() {
        size.max(0.0)
    } else {
        15.0
    };
    let width = if width.is_nan() {
        f32::INFINITY
    } else {
        width.max(0.0)
    };
    let key = Key {
        text: text.into(),
        size: size.to_bits(),
        family: family.into(),
        width: width.to_bits(),
    };
    CONTEXT.with_borrow_mut(|context| {
        if let Some(shaped) = context.shapes.get(&key) {
            return shaped.clone();
        }
        let mut layout = Layout::new();
        let mut builder = context
            .layouts
            .ranged_builder(&mut context.fonts, text, 1.0, false);
        builder.push_default(StyleProperty::FontStack(FontStack::Single(
            FontFamily::Named(family.into()),
        )));
        builder.push_default(StyleProperty::FontSize(size));
        builder.push_default(StyleProperty::LineHeight(LineHeight::FontSizeRelative(
            LINE_HEIGHT,
        )));
        builder.build_into(&mut layout, text);
        layout.break_all_lines(width.is_finite().then_some(width));
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        let result = Arc::new(Shaped {
            id: NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            text: text.into(),
            layout,
            font_size: size,
        });
        // Bound history rather than retaining every visited document or keystroke.
        let bytes = text.len().saturating_mul(128).saturating_add(512);
        if context.shapes.len() >= 512 || context.shape_bytes + bytes > 8 * 1024 * 1024 {
            context.shapes.clear();
            context.shape_bytes = 0;
        }
        if bytes <= 8 * 1024 * 1024 {
            context.shape_bytes += bytes;
            context.shapes.insert(key, result.clone());
        }
        result
    })
}
