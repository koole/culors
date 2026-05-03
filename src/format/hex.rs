//! Legacy hex serializers `format_hex` and `format_hex8`.
//!
//! Mirrors culori 4.0.2's `formatHex` / `formatHex8`
//! (`node_modules/culori/src/formatter.js`):
//!
//! ```js
//! const clamp = value => Math.max(0, Math.min(1, value || 0));
//! const fixup = value => Math.round(clamp(value) * 255);
//! ```
//!
//! Steps culori's pipeline takes:
//!   1. Convert input to `rgb` (sRGB).
//!   2. Per channel: replace NaN/undefined with 0 (`value || 0`), clamp to
//!      [0, 1], multiply by 255, round half-away-from-zero.
//!   3. Pack into `#rrggbb` (or `#rrggbbaa`) lowercase hex.
//!
//! For `format_hex8`, an absent alpha is treated as 1 (matching culori's
//! `color.alpha !== undefined ? color.alpha : 1`).

use crate::color::Color;
use crate::spaces::Rgb;

/// Map an f64 channel to its 0..=255 byte using culori's `fixup` rule:
/// NaN → 0, clamp to [0, 1], `Math.round(x * 255)`.
///
/// Rust's `f64::round()` rounds half away from zero, which matches JS
/// `Math.round` for non-negative inputs (and the input is always >= 0
/// here because we clamp first).
pub(super) fn fixup(value: f64) -> u8 {
    let v = if value.is_nan() { 0.0 } else { value };
    let clamped = v.clamp(0.0, 1.0);
    (clamped * 255.0).round() as u8
}

pub(super) fn rgb_of(color: &Color) -> Rgb {
    if let Color::Rgb(c) = color {
        return *c;
    }
    match color.convert_to("rgb") {
        Some(Color::Rgb(c)) => c,
        _ => unreachable!("convert_to(\"rgb\") returns Color::Rgb for any known mode"),
    }
}

/// Serialize a color as a legacy `#rrggbb` hex string.
///
/// Input is converted to sRGB; channels are clamped to 0..=255 with rounding
/// (NaN → 0). Alpha is ignored. Output is lowercase.
///
/// ```rust
/// use culors::{format_hex, parse};
/// let red = parse("tomato").unwrap();
/// assert_eq!(format_hex(&red), "#ff6347");
/// ```
pub fn format_hex(color: &Color) -> String {
    let rgb = rgb_of(color);
    let r = fixup(rgb.r);
    let g = fixup(rgb.g);
    let b = fixup(rgb.b);
    format!("#{:02x}{:02x}{:02x}", r, g, b)
}

/// Serialize a color as a legacy `#rrggbbaa` hex string.
///
/// Input is converted to sRGB; channels and alpha are clamped to 0..=255 with
/// rounding (NaN → 0). Absent alpha is treated as 1, yielding `…ff`.
///
/// ```rust
/// use culors::{format_hex8, parse};
/// let red = parse("rgb(255 255 255 / 0.5)").unwrap();
/// assert_eq!(format_hex8(&red), "#ffffff80");
/// ```
pub fn format_hex8(color: &Color) -> String {
    let rgb = rgb_of(color);
    let r = fixup(rgb.r);
    let g = fixup(rgb.g);
    let b = fixup(rgb.b);
    let a = fixup(rgb.alpha.unwrap_or(1.0));
    format!("#{:02x}{:02x}{:02x}{:02x}", r, g, b, a)
}
