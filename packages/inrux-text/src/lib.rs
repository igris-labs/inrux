//! Common logical text shaping and device-scale glyph coverage, independent of GUI engines.
#![deny(missing_docs)]
mod raster;
mod shape;
use parley::{FontContext, LayoutContext};
pub use raster::{Bitmap, rasterize, rasterize_layout, rasterize_region};
pub use shape::{Shaped, shape};
use std::{cell::RefCell, collections::HashMap, sync::Arc};
/// Portable default face; applications can override it through font-family.
pub const DEFAULT_FAMILY: &str = "Noto Sans";
/// Default face bytes, registered by native editor bridges as well as the shared provider.
pub const DEFAULT_FONT: &[u8] = include_bytes!("../assets/NotoSans-Regular.ttf");
/// Common font-relative line height.
pub const LINE_HEIGHT: f32 = inrux_style::DEFAULT_LINE_HEIGHT;
struct Context {
    fonts: FontContext,
    layouts: LayoutContext<usize>,
    shapes: HashMap<shape::Key, Arc<Shaped>>,
    shape_bytes: usize,
    raster: raster::Cache,
    registered: Vec<Vec<u8>>,
}
impl Context {
    /// Font discovery and the bundled face are initialized once per owning UI thread.
    fn new() -> Self {
        let mut fonts = FontContext::new();
        fonts
            .collection
            .register_fonts(DEFAULT_FONT.to_vec().into(), None);
        Self {
            fonts,
            layouts: LayoutContext::new(),
            shapes: HashMap::new(),
            shape_bytes: 0,
            raster: Default::default(),
            registered: vec![DEFAULT_FONT.to_vec()],
        }
    }
}
thread_local! { static CONTEXT: RefCell<Context> = RefCell::new(Context::new()); }
/// Register font bytes before mounting. Identical registrations preserve cached shapes and rasters.
pub fn register_fonts<'a>(assets: impl IntoIterator<Item = &'a [u8]>) -> Result<(), String> {
    CONTEXT.with_borrow_mut(|context| {
        for bytes in assets {
            if context.registered.iter().any(|known| known == bytes) {
                continue;
            }
            if context
                .fonts
                .collection
                .register_fonts(bytes.to_vec().into(), None)
                .is_empty()
            {
                return Err(
                    "Cannot register font: the asset contains no valid OpenType/TrueType face"
                        .into(),
                );
            }
            context.registered.push(bytes.to_vec());
            context.shapes.clear();
            context.shape_bytes = 0;
            context.raster.clear();
        }
        Ok(())
    })
}

/// Clone the common registry for a retained native editor; face data remains shared.
pub fn font_context() -> FontContext {
    CONTEXT.with_borrow(|context| context.fonts.clone())
}

mod geometry;
pub use geometry::{caret_rect, selection_rects, underline_rects};

#[cfg(test)]
mod tests;
