//! HSV color space (cylindrical sRGB).
//!
//! Conversions track culori 4.0.2 (`node_modules/culori/src/hsv/`). HSV is a
//! direct cylindrical reparameterization of sRGB; no XYZ trip is involved.
//! `to_xyz65` / `from_xyz65` simply compose with the [`Rgb`] hub conversion.
//!
//! Like Hsl, achromatic colors get `f64::NAN` for hue (mirroring culori's
//! omission of the property). On the reverse path NaN hue is coerced to 0
//! before normalization.

use crate::spaces::{Rgb, Xyz65};
use crate::traits::ColorSpace;

/// HSV — hue (degrees, 0..360), saturation (0..1), value (0..1). For
/// achromatic colors `h` is NaN.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hsv {
    /// Hue in degrees, NaN for achromatic colors.
    pub h: f64,
    /// Saturation in 0..1.
    pub s: f64,
    /// Value (max channel) in 0..1.
    pub v: f64,
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

impl ColorSpace for Hsv {
    const MODE: &'static str = "hsv";
    const CHANNELS: &'static [&'static str] = &["h", "s", "v"];

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

impl From<Rgb> for Hsv {
    fn from(c: Rgb) -> Self {
        let Rgb { r, g, b, alpha } = c;
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let v = max;
        let s = if max == 0.0 { 0.0 } else { 1.0 - min / max };
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
        Self { h, s, v, alpha }
    }
}

impl From<Hsv> for Rgb {
    fn from(c: Hsv) -> Self {
        let h_in = if c.h.is_nan() { 0.0 } else { c.h };
        let h = normalize_hue(h_in);
        let s = c.s;
        let v = c.v;
        let f = (((h / 60.0) % 2.0) - 1.0).abs();
        let (r, g, b) = match (h / 60.0).floor() as i32 {
            0 => (v, v * (1.0 - s * f), v * (1.0 - s)),
            1 => (v * (1.0 - s * f), v, v * (1.0 - s)),
            2 => (v * (1.0 - s), v, v * (1.0 - s * f)),
            3 => (v * (1.0 - s), v * (1.0 - s * f), v),
            4 => (v * (1.0 - s * f), v * (1.0 - s), v),
            5 => (v, v * (1.0 - s), v * (1.0 - s * f)),
            _ => (v * (1.0 - s), v * (1.0 - s), v * (1.0 - s)),
        };
        Self {
            r,
            g,
            b,
            alpha: c.alpha,
        }
    }
}
