//! Compact straight-alpha colors; stylesheet conversion runs before native rendering.
use crate::StyleError;
/// sRGB bytes with straight alpha. Native bindings perform their own premultiplication.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgba(
    /// Red channel.
    pub u8,
    /// Green channel.
    pub u8,
    /// Blue channel.
    pub u8,
    /// Alpha channel: zero is transparent, 255 is opaque.
    pub u8,
);
/// Public color type; existing RGB construction remains available as Color(r,g,b).
pub type Color = Rgba;
/// Construct opaque sRGB without parsing or allocation.
#[allow(
    non_snake_case,
    reason = "preserve the public Color(r,g,b) constructor"
)]
pub const fn Color(r: u8, g: u8, b: u8) -> Color {
    Rgba(r, g, b, 255)
}
impl Rgba {
    /// Construct straight-alpha sRGBA bytes without parsing or allocation.
    #[expect(
        clippy::self_named_constructors,
        reason = "the public type alias exposes the familiar Color::rgba API"
    )]
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self(r, g, b, a)
    }
    /// Parse exactly #rrggbb for callers relying on the original strict helper.
    pub fn hex(value: &str) -> Result<Self, StyleError> {
        let digits = value
            .strip_prefix('#')
            .ok_or_else(|| StyleError("expected #rrggbb".into()))?;
        if digits.len() != 6 || !digits.is_ascii() {
            return Err(StyleError("expected #rrggbb".into()));
        }
        let rgb =
            u32::from_str_radix(digits, 16).map_err(|_| StyleError("invalid hex color".into()))?;
        Ok(Color((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8))
    }
    /// Parse a literal CSS color once, converting/clipping it to the native sRGBA8 paint contract.
    /// Contextual values, color-mix and relative colors are intentionally not accepted here.
    pub fn parse(value: &str) -> Result<Self, StyleError> {
        let parsed = color::parse_color(value).map_err(|error| {
            StyleError(format!(
                "invalid or unsupported CSS color `{value}`: {error}"
            ))
        })?;
        let srgb: color::AlphaColor<color::Srgb> = parsed.to_alpha_color();
        if srgb.components.iter().any(|n| !n.is_finite()) {
            return Err(StyleError("color channels must be finite".into()));
        }
        let rgba = srgb.to_rgba8();
        Ok(Self(rgba.r, rgba.g, rgba.b, rgba.a))
    }
}
