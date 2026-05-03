//! Hue/Saturation/Intensity (cylindrical sRGB).
//!
//! Constants and formulas verbatim from culori 4.0.2
//! (`node_modules/culori/src/hsi/`). Mirrors Wikipedia's HSI derivation:
//! intensity is the channel mean, saturation is `1 - 3·min/(r+g+b)`, hue
//! follows the HSV/HSL geometry.

use crate::spaces::{Rgb, Xyz65};
use crate::traits::ColorSpace;
use crate::util::normalize_hue;

/// HSI color. Hue is in degrees; saturation and intensity are in 0..1.
/// Hue is `f64::NAN` for achromatic (max == min) inputs, mirroring
/// culori's `undefined h` sentinel.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hsi {
    /// Hue in degrees.
    pub h: f64,
    /// Saturation in 0..1.
    pub s: f64,
    /// Intensity in 0..1.
    pub i: f64,
    /// Optional alpha in 0..1.
    pub alpha: Option<f64>,
}

impl ColorSpace for Hsi {
    const MODE: &'static str = "hsi";
    const CHANNELS: &'static [&'static str] = &["h", "s", "i"];

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

impl From<Rgb> for Hsi {
    fn from(c: Rgb) -> Self {
        let Rgb { r, g, b, alpha } = c;
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let sum = r + g + b;
        let s = if sum == 0.0 {
            0.0
        } else {
            1.0 - (3.0 * min) / sum
        };
        let i = sum / 3.0;
        let h = if max - min == 0.0 {
            f64::NAN
        } else {
            let span = max - min;
            let raw = if max == r {
                (g - b) / span + if g < b { 6.0 } else { 0.0 }
            } else if max == g {
                (b - r) / span + 2.0
            } else {
                (r - g) / span + 4.0
            };
            raw * 60.0
        };
        Self { h, s, i, alpha }
    }
}

impl From<Hsi> for Rgb {
    fn from(c: Hsi) -> Self {
        let h = normalize_hue(if c.h.is_nan() { 0.0 } else { c.h });
        let s = c.s;
        let i = c.i;
        let f = (((h / 60.0) % 2.0) - 1.0).abs();
        let sextant = (h / 60.0).floor() as i32;
        let (r, g, b) = match sextant {
            0 => (
                i * (1.0 + s * (3.0 / (2.0 - f) - 1.0)),
                i * (1.0 + s * ((3.0 * (1.0 - f)) / (2.0 - f) - 1.0)),
                i * (1.0 - s),
            ),
            1 => (
                i * (1.0 + s * ((3.0 * (1.0 - f)) / (2.0 - f) - 1.0)),
                i * (1.0 + s * (3.0 / (2.0 - f) - 1.0)),
                i * (1.0 - s),
            ),
            2 => (
                i * (1.0 - s),
                i * (1.0 + s * (3.0 / (2.0 - f) - 1.0)),
                i * (1.0 + s * ((3.0 * (1.0 - f)) / (2.0 - f) - 1.0)),
            ),
            3 => (
                i * (1.0 - s),
                i * (1.0 + s * ((3.0 * (1.0 - f)) / (2.0 - f) - 1.0)),
                i * (1.0 + s * (3.0 / (2.0 - f) - 1.0)),
            ),
            4 => (
                i * (1.0 + s * ((3.0 * (1.0 - f)) / (2.0 - f) - 1.0)),
                i * (1.0 - s),
                i * (1.0 + s * (3.0 / (2.0 - f) - 1.0)),
            ),
            5 => (
                i * (1.0 + s * (3.0 / (2.0 - f) - 1.0)),
                i * (1.0 - s),
                i * (1.0 + s * ((3.0 * (1.0 - f)) / (2.0 - f) - 1.0)),
            ),
            _ => (i * (1.0 - s), i * (1.0 - s), i * (1.0 - s)),
        };
        Self {
            r,
            g,
            b,
            alpha: c.alpha,
        }
    }
}
