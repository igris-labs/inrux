//! Shared CSS value validation.
use crate::{Size, StyleError};
/// Parse finite nonnegative logical lengths.
pub(crate) fn length(value: &str) -> Result<f32, StyleError> {
    let raw = if value == "0" {
        "0"
    } else {
        value
            .strip_suffix("px")
            .ok_or_else(|| StyleError(format!("expected logical pixels, got `{value}`")))?
    };
    let number: f32 = raw
        .parse()
        .map_err(|_| StyleError(format!("invalid length `{value}`")))?;
    if !number.is_finite() || number < 0.0 {
        return Err(StyleError("length must be finite and nonnegative".into()));
    }
    Ok(number)
}
/// Parse logical, relative, and intrinsic dimensions.
pub(crate) fn size(value: &str) -> Result<Size, StyleError> {
    if value == "auto" {
        return Ok(Size::Content);
    }
    if let Some(raw) = value.strip_suffix('%') {
        let n: f32 = raw
            .parse()
            .map_err(|_| StyleError("invalid percentage".into()))?;
        if !n.is_finite() || n < 0.0 {
            return Err(StyleError("invalid percentage".into()));
        }
        return Ok(Size::Percent(n));
    }
    Ok(Size::Px(length(value)?))
}

/// Parse the intentionally bounded uniform-grid subset without allocating track vectors.
pub(crate) fn grid_columns(value: &str) -> Result<u16, StyleError> {
    let invalid =
        || StyleError("grid-template-columns requires repeat(N, 1fr), with N in 1..=256".into());
    let body = value
        .strip_prefix("repeat(")
        .and_then(|value| value.strip_suffix(')'))
        .ok_or_else(invalid)?;
    let (count, track) = body.split_once(',').ok_or_else(invalid)?;
    if track.trim() != "1fr" {
        return Err(invalid());
    }
    let count: u16 = count.trim().parse().map_err(|_| invalid())?;
    if !(1..=crate::MAX_GRID_COLUMNS).contains(&count) {
        return Err(invalid());
    }
    Ok(count)
}
/// Parse a positive CSS ratio while rejecting zero denominators and floating-point overflow.
pub(crate) fn aspect_ratio(value: &str) -> Result<f32, StyleError> {
    let invalid =
        || StyleError("aspect-ratio requires finite positive numbers, such as 1 or 16 / 9".into());
    let positive = |value: &str| -> Result<f32, StyleError> {
        let value: f32 = value.trim().parse().map_err(|_| invalid())?;
        if !value.is_finite() || value <= 0.0 {
            return Err(invalid());
        }
        Ok(value)
    };
    let ratio = if let Some((width, height)) = value.split_once('/') {
        positive(width)? / positive(height)?
    } else {
        positive(value)?
    };
    if !ratio.is_finite() || ratio <= 0.0 {
        return Err(invalid());
    }
    Ok(ratio)
}

/// Physical margins may be negative, but never nonfinite or in an unimplemented unit.
pub(crate) fn signed_length(value: &str) -> Result<f32, StyleError> {
    let raw = if value == "0" {
        "0"
    } else {
        value
            .strip_suffix("px")
            .ok_or_else(|| StyleError(format!("expected logical px margin, got `{value}`")))?
    };
    let n: f32 = raw
        .parse()
        .map_err(|_| StyleError("invalid margin length".into()))?;
    if !n.is_finite() {
        return Err(StyleError("margin must be finite".into()));
    }
    Ok(n)
}
/// Expand CSS physical shorthand in top/right/bottom/left order without allocating a vector.
pub(crate) fn margin_edges(value: &str) -> Result<[f32; 4], StyleError> {
    let mut values = [0.0; 4];
    let mut count = 0;
    for token in value.split_whitespace() {
        if count == 4 {
            return Err(StyleError("margin expects one to four px lengths".into()));
        }
        values[count] = signed_length(token)?;
        count += 1;
    }
    match count {
        1 => Ok([values[0]; 4]),
        2 => Ok([values[0], values[1], values[0], values[1]]),
        3 => Ok([values[0], values[1], values[2], values[1]]),
        4 => Ok(values),
        _ => Err(StyleError("margin expects one to four px lengths".into())),
    }
}
