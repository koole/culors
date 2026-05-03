//! Legacy comma-form `rgb()` / `rgba()` serializer.
//!
//! Mirrors culori 4.0.2's `serializeRgb`
//! (`node_modules/culori/src/formatter.js`). The input is first converted to
//! sRGB, then channels are clamped to 0..=255 (NaN → 0) and the output uses
//! the legacy comma-separated form:
//!
//! ```text
//! rgb(R, G, B)             // alpha is undefined or exactly 1
//! rgba(R, G, B, A)         // alpha < 1; A rounded to 2 decimals
//! ```
//!
//! Alpha is clamped to [0, 1] and rounded to two decimal places via the
//! same `Math.round(a*100)/100` rule culori uses.

use crate::color::Color;
use crate::format::hex::{fixup, rgb_of};

/// Round a finite value to two decimals, matching culori's
/// `round(2)`: `Math.round(v*100)/100`. Trailing zeros are dropped on
/// stringification (Rust's default f64 format matches JS for the relevant
/// magnitudes).
fn two_decimals(value: f64) -> f64 {
    if value.is_nan() {
        0.0
    } else {
        (value * 100.0).round() / 100.0
    }
}

/// Serialize a color as legacy `rgb(R, G, B)` or `rgba(R, G, B, A)`.
///
/// Channels are integers in 0..=255, clamped with rounding (NaN → 0). Alpha
/// is rendered with up to two decimals; if alpha is absent or exactly 1, the
/// `rgb(...)` form is used.
///
/// ```rust
/// use culors::{format_rgb, parse};
/// let c = parse("#f0f0f0f0").unwrap();
/// assert_eq!(format_rgb(&c), "rgba(240, 240, 240, 0.94)");
/// ```
pub fn format_rgb(color: &Color) -> String {
    let rgb = rgb_of(color);
    let r = fixup(rgb.r);
    let g = fixup(rgb.g);
    let b = fixup(rgb.b);
    match rgb.alpha {
        None | Some(1.0) => format!("rgb({}, {}, {})", r, g, b),
        Some(a) => {
            // culori's `clamp(value)` is `Math.max(0, Math.min(1, value ||
            // 0))`. The `value || 0` collapses NaN to 0; clamp pins
            // out-of-range alphas to [0, 1].
            let clamped = if a.is_nan() { 0.0 } else { a.clamp(0.0, 1.0) };
            let rounded = two_decimals(clamped);
            format!("rgba({}, {}, {}, {})", r, g, b, rounded)
        }
    }
}
