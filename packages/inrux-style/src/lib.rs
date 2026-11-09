//! Shared typed styles used by Rust builders and compile-time CSS translation.
#![deny(missing_docs)]
mod compiled;
mod css;
mod media;
mod part;
mod selector;
pub use part::Part;
mod syntax;
mod tokens;
pub use compiled::CompiledStyles;
pub use media::{Media, Viewport};
pub use selector::Selector;
mod colors;
mod error;
mod style;
mod types;
pub use colors::{Color, Rgba};
mod values;
pub use css::{Rule, parse_css};
pub use error::StyleError;
pub use style::Style;
pub use types::{
    Align, Display, FlexDirection, Justify, MAX_GRID_COLUMNS, Overflow, Position, Size, State,
    WhiteSpace,
};
#[cfg(test)]
mod tests;

#[cfg(test)]
mod compiled_tests;

/// Default font-relative line height used by every text provider and editor bridge.
pub const DEFAULT_LINE_HEIGHT: f32 = 1.2;
