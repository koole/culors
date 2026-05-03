//! Legacy comma-form `hsl()` / `hsla()` serializer.
//!
//! Mirrors culori 4.0.2's `serializeHsl`
//! (`node_modules/culori/src/formatter.js`):
//!
//! ```js
//! const h = twoDecimals(color.h || 0);
//! const s = twoDecimals(clamp(color.s) * 100) + '%';
//! const l = twoDecimals(clamp(color.l) * 100) + '%';
//! ```
//!
//! Hue is taken as-is when finite (no clamping or wrapping; negatives and
//! values >360 round-trip), and replaced with `0` when missing or NaN. S and
//! L are clamped to [0, 1], scaled to a percent, and rounded to two decimal
//! places. Alpha is clamped and rounded the same way as in
//! [`crate::format_rgb`]; `hsl(...)` is used when alpha is absent or exactly
//! 1, otherwise `hsla(...)`.

use crate::color::Color;
use crate::spaces::Hsl;

fn hsl_of(color: &Color) -> Hsl {
    if let Color::Hsl(c) = color {
        return *c;
    }
    match color.convert_to("hsl") {
        Some(Color::Hsl(c)) => c,
        _ => unreachable!("convert_to(\"hsl\") returns Color::Hsl for any known mode"),
    }
}

/// Round to two decimal places (`Math.round(v*100)/100`). NaN collapses to
/// 0, matching culori's `value || 0` semantics on its hue input and
/// `clamp(...)` on saturation / lightness.
fn two_decimals(value: f64) -> f64 {
    if value.is_nan() {
        0.0
    } else {
        (value * 100.0).round() / 100.0
    }
}

fn clamp01(value: f64) -> f64 {
    if value.is_nan() {
        0.0
    } else {
        value.clamp(0.0, 1.0)
    }
}

/// Serialize a color as legacy `hsl(H, S%, L%)` or `hsla(H, S%, L%, A)`.
///
/// The input is converted to HSL. Hue is rendered with up to two decimals
/// (NaN → 0); saturation and lightness are clamped to [0, 100]% with
/// rounding; alpha is rendered like in [`crate::format_rgb`].
///
/// ```rust
/// use culors::{format_hsl, parse};
/// let red = parse("red").unwrap();
/// assert_eq!(format_hsl(&red), "hsl(0, 100%, 50%)");
/// ```
pub fn format_hsl(color: &Color) -> String {
    let hsl = hsl_of(color);
    let h = two_decimals(if hsl.h.is_nan() { 0.0 } else { hsl.h });
    let s = two_decimals(clamp01(hsl.s) * 100.0);
    let l = two_decimals(clamp01(hsl.l) * 100.0);
    match hsl.alpha {
        None | Some(1.0) => format!("hsl({}, {}%, {}%)", h, s, l),
        Some(a) => {
            let clamped = if a.is_nan() { 0.0 } else { a.clamp(0.0, 1.0) };
            let rounded = two_decimals(clamped);
            format!("hsla({}, {}%, {}%, {})", h, s, l, rounded)
        }
    }
}
