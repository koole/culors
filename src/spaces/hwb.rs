//! HWB color space — hue / whiteness / blackness.
//!
//! Conversions track culori 4.0.2 (`node_modules/culori/src/hwb/`). HWB is
//! a direct reparameterization of HSV (Smith 1996); no XYZ trip is
//! involved. `to_xyz65` / `from_xyz65` compose with the [`Hsv`] hub
//! conversion (which itself goes via [`Rgb`]).
//!
//! When `w + b > 1` the color is achromatic and culori normalizes by
//! dividing both by their sum. Hue is powerless under that condition; the
//! NaN sentinel mirrors Hsv/Hsl when the source color is achromatic.

use crate::spaces::{Hsv, Rgb, Xyz65};
use crate::traits::ColorSpace;

/// HWB — hue (degrees, 0..360), whiteness (0..1), blackness (0..1). For
/// achromatic colors `h` is NaN.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hwb {
    /// Hue in degrees, NaN for achromatic colors.
    pub h: f64,
    /// Whiteness in 0..1.
    pub w: f64,
    /// Blackness in 0..1.
    pub b: f64,
    /// Optional alpha in 0..1.
    pub alpha: Option<f64>,
}

impl ColorSpace for Hwb {
    const MODE: &'static str = "hwb";
    const CHANNELS: &'static [&'static str] = &["h", "w", "b"];

    fn alpha(&self) -> Option<f64> {
        self.alpha
    }

    fn with_alpha(self, alpha: Option<f64>) -> Self {
        Self { alpha, ..self }
    }

    fn to_xyz65(&self) -> Xyz65 {
        Hsv::from(*self).to_xyz65()
    }

    fn from_xyz65(xyz: Xyz65) -> Self {
        Hsv::from_xyz65(xyz).into()
    }
}

impl From<Hsv> for Hwb {
    fn from(c: Hsv) -> Self {
        // culori: w = (1 - s) * v, b = 1 - v. Hue passes through, including
        // a NaN sentinel for achromatic colors (where culori omits `h`).
        Self {
            h: c.h,
            w: (1.0 - c.s) * c.v,
            b: 1.0 - c.v,
            alpha: c.alpha,
        }
    }
}

impl From<Rgb> for Hwb {
    fn from(c: Rgb) -> Self {
        Hsv::from(c).into()
    }
}

impl From<Hwb> for Rgb {
    fn from(c: Hwb) -> Self {
        Hsv::from(c).into()
    }
}

impl From<Hwb> for Hsv {
    fn from(c: Hwb) -> Self {
        let mut w = c.w;
        let mut b = c.b;
        if w + b > 1.0 {
            let sum = w + b;
            w /= sum;
            b /= sum;
        }
        let s = if b == 1.0 { 1.0 } else { 1.0 - w / (1.0 - b) };
        Self {
            h: c.h,
            s,
            v: 1.0 - b,
            alpha: c.alpha,
        }
    }
}
