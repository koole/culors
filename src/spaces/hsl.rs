//! HSL color space (cylindrical sRGB).
//!
//! Conversions track culori 4.0.2 (`node_modules/culori/src/hsl/`). HSL is a
//! direct cylindrical reparameterization of sRGB; no XYZ trip is involved.
//! `to_xyz65` / `from_xyz65` simply compose with the [`Rgb`] hub conversion.
//!
//! culori omits the `h` property when chroma is zero (achromatic colors). We
//! mirror that with `f64::NAN` since our struct stores `h` as `f64`. On the
//! reverse path, NaN hue is treated as 0 (matching culori's `h ?? 0`
//! fallback inside `convertHslToRgb`).

use crate::spaces::{Rgb, Xyz65};
use crate::traits::ColorSpace;

/// HSL — hue (degrees, 0..360), saturation (0..1), lightness (0..1). For
/// achromatic colors `h` is NaN.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hsl {
    /// Hue in degrees, NaN for achromatic colors.
    pub h: f64,
    /// Saturation in 0..1.
    pub s: f64,
    /// Lightness in 0..1.
    pub l: f64,
    /// Optional alpha in 0..1.
    pub alpha: Option<f64>,
}

#[inline]
fn normalize_hue(h: f64) -> f64 {
    let h = h % 360.0;
    if h < 0.0 {
        h + 360.0
    } else {
        h
    }
}

impl ColorSpace for Hsl {
    const MODE: &'static str = "hsl";
    const CHANNELS: &'static [&'static str] = &["h", "s", "l"];

    fn alpha(&self) -> Option<f64> {
        self.alpha
    }

    fn with_alpha(self, alpha: Option<f64>) -> Self {
        Self { alpha, ..self }
    }

    fn to_xyz65(&self) -> Xyz65 {
        Rgb::from(*self).to_xyz65()
    }

    fn from_xyz65(xyz: Xyz65) -> Self {
        Rgb::from_xyz65(xyz).into()
    }
}

impl From<Rgb> for Hsl {
    fn from(c: Rgb) -> Self {
        let Rgb { r, g, b, alpha } = c;
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let l = 0.5 * (max + min);
        let s = if max == min {
            0.0
        } else {
            (max - min) / (1.0 - (max + min - 1.0).abs())
        };
        let h = if max == min {
            f64::NAN
        } else if max == r {
            let mut h = (g - b) / (max - min);
            if g < b {
                h += 6.0;
            }
            h * 60.0
        } else if max == g {
            ((b - r) / (max - min) + 2.0) * 60.0
        } else {
            ((r - g) / (max - min) + 4.0) * 60.0
        };
        Self { h, s, l, alpha }
    }
}

impl From<Hsl> for Rgb {
    fn from(c: Hsl) -> Self {
        // culori normalizes h via `h !== undefined ? h : 0`; for our NaN
        // sentinel we coerce to 0 before normalizing.
        let h_in = if c.h.is_nan() { 0.0 } else { c.h };
        let h = normalize_hue(h_in);
        let s = c.s;
        let l = c.l;
        let m1 = l + s * (if l < 0.5 { l } else { 1.0 - l });
        let m2 = m1 - (m1 - l) * 2.0 * (((h / 60.0) % 2.0) - 1.0).abs();
        let (r, g, b) = match (h / 60.0).floor() as i32 {
            0 => (m1, m2, 2.0 * l - m1),
            1 => (m2, m1, 2.0 * l - m1),
            2 => (2.0 * l - m1, m1, m2),
            3 => (2.0 * l - m1, m2, m1),
            4 => (m2, 2.0 * l - m1, m1),
            5 => (m1, 2.0 * l - m1, m2),
            _ => (2.0 * l - m1, 2.0 * l - m1, 2.0 * l - m1),
        };
        Self {
            r,
            g,
            b,
            alpha: c.alpha,
        }
    }
}
