//! Owned font assets shared by native adapters without exposing a font engine.
use std::sync::Arc;
/// A bundled OpenType/TrueType face registered under its intrinsic family name.
#[derive(Clone)]
pub struct FontAsset {
    /// Family recorded inside the font, used by CSS/Rust font-family.
    pub family: String,
    /// Font file bytes, cheaply shared until a native font service consumes them.
    pub data: Arc<[u8]>,
}
impl FontAsset {
    /// Register an owned font asset; include its redistribution license with the application.
    pub fn new(family: impl Into<String>, data: impl Into<Arc<[u8]>>) -> Self {
        Self {
            family: family.into(),
            data: data.into(),
        }
    }
}
impl std::fmt::Debug for FontAsset {
    /// Avoid dumping a binary font into configuration logs.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FontAsset")
            .field("family", &self.family)
            .field("bytes", &self.data.len())
            .finish()
    }
}
