//! JzCzHz — polar form of JzAzBz.
//!
//! Constants and formulas verbatim from culori 4.0.2
//! (`node_modules/culori/src/jch/`).

use crate::spaces::{Jab, Rgb, Xyz65};
use crate::traits::ColorSpace;
use crate::util::normalize_hue;

/// JzCzHz polar coordinates of [`Jab`]. Hue is undefined (`f64::NAN`)
/// when the chroma is zero.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Jch {
    /// Lightness (Jz).
    pub j: f64,
    /// Chroma.
    pub c: f64,
    /// Hue in degrees.
    pub h: f64,
    /// Optional alpha in 0..1.
    pub alpha: Option<f64>,
}

impl ColorSpace for Jch {
    const MODE: &'static str = "jch";
    const CHANNELS: &'static [&'static str] = &["j", "c", "h"];

    fn alpha(&self) -> Option<f64> {
        self.alpha
    }

    fn with_alpha(self, alpha: Option<f64>) -> Self {
        Self { alpha, ..self }
    }

    fn to_xyz65(&self) -> Xyz65 {
        Jab::from(*self).to_xyz65()
    }

    fn from_xyz65(xyz: Xyz65) -> Self {
        Jab::from_xyz65(xyz).into()
    }
}

impl From<Jab> for Jch {
    fn from(j: Jab) -> Self {
        let c = (j.a * j.a + j.b * j.b).sqrt();
        let h = if c == 0.0 {
            f64::NAN
        } else {
            normalize_hue(j.b.atan2(j.a).to_degrees())
        };
        Self {
            j: j.j,
            c,
            h,
            alpha: j.alpha,
        }
    }
}

impl From<Jch> for Jab {
    fn from(j: Jch) -> Self {
        let h = if j.h.is_nan() { 0.0 } else { j.h };
        let (a, b) = if j.c == 0.0 {
            (0.0, 0.0)
        } else {
            let hr = h.to_radians();
            (j.c * hr.cos(), j.c * hr.sin())
        };
        Self {
            j: j.j,
            a,
            b,
            alpha: j.alpha,
        }
    }
}

/// Direct `Rgb` → `Jch` mirroring culori's `convertJabToJch ∘
/// convertRgbToJab` chain. Picks up the achromatic snap from
/// [`Jab::from`].
impl From<Rgb> for Jch {
    fn from(c: Rgb) -> Self {
        Jab::from(c).into()
    }
}
