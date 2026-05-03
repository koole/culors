//! CIE Lch color space (polar form of Lab65, D65 illuminant).
//!
//! Conversions track culori 4.0.2 (`node_modules/culori/src/lch65/`,
//! which delegates to `lch/convertLabToLch.js` and `lch/convertLchToLab.js`).
//! Lch65 is a direct polar reparameterization of [`Lab65`]; no XYZ trip is
//! involved on the polar leg. `to_xyz65` / `from_xyz65` compose with the
//! [`Lab65`] hub conversion.
//!
//! culori omits the `h` property when chroma is exactly zero (the strict
//! `if (c)` truthy check in `convertLabToLch.js`). We mirror that with
//! `f64::NAN`. On the reverse path, NaN hue is treated as 0 (matching
//! culori's `h === undefined` fallback inside `convertLchToLab`).

use crate::spaces::{Lab65, Rgb, Xyz65};
use crate::traits::ColorSpace;

/// CIE Lch (D65). `l` is in 0..100, `c` (chroma) is non-negative, `h`
/// (hue, degrees) is normalized to 0..360. For achromatic colors `h` is
/// NaN.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Lch65 {
    /// Lightness (0..100).
    pub l: f64,
    /// Chroma (>= 0).
    pub c: f64,
    /// Hue in degrees (0..360), NaN for achromatic colors.
    pub h: f64,
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

impl ColorSpace for Lch65 {
    const MODE: &'static str = "lch65";
    const CHANNELS: &'static [&'static str] = &["l", "c", "h"];

    fn alpha(&self) -> Option<f64> {
        self.alpha
    }

    fn with_alpha(self, alpha: Option<f64>) -> Self {
        Self { alpha, ..self }
    }

    fn to_xyz65(&self) -> Xyz65 {
        Lab65::from(*self).to_xyz65()
    }

    fn from_xyz65(xyz: Xyz65) -> Self {
        Lab65::from_xyz65(xyz).into()
    }
}

impl From<Lab65> for Lch65 {
    fn from(c: Lab65) -> Self {
        let chroma = (c.a * c.a + c.b * c.b).sqrt();
        let h = if chroma == 0.0 {
            f64::NAN
        } else {
            normalize_hue(c.b.atan2(c.a).to_degrees())
        };
        Self {
            l: c.l,
            c: chroma,
            h,
            alpha: c.alpha,
        }
    }
}

/// Direct `Rgb` -> `Lch65` conversion that picks up the achromatic snap
/// from `Lab65::from(Rgb)` so `r == g == b` produces `c = 0` and `h = NaN`,
/// matching culori's public `lch65({mode:'rgb', ...})` output.
impl From<Rgb> for Lch65 {
    fn from(c: Rgb) -> Self {
        Lch65::from(Lab65::from(c))
    }
}

impl From<Lch65> for Lab65 {
    fn from(c: Lch65) -> Self {
        let h = if c.h.is_nan() { 0.0 } else { c.h };
        let (a, b) = if c.c == 0.0 {
            (0.0, 0.0)
        } else {
            let theta = h.to_radians();
            (c.c * theta.cos(), c.c * theta.sin())
        };
        Self {
            l: c.l,
            a,
            b,
            alpha: c.alpha,
        }
    }
}
