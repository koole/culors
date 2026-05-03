//! CIE Lch color space (polar form of Lab, D50 illuminant).
//!
//! Conversions track culori 4.0.2 (`node_modules/culori/src/lch/`). Lch is
//! a direct polar reparameterization of Lab; no XYZ trip is involved.
//! `to_xyz65` / `from_xyz65` compose with the [`Lab`] hub conversion (which
//! itself goes via [`Xyz50`](crate::spaces::Xyz50)).
//!
//! culori omits the `h` property when chroma is exactly zero (the strict
//! `if (c)` truthy check in `convertLabToLch.js`). We mirror that with
//! `f64::NAN` since our struct stores `h` as `f64`. On the reverse path,
//! NaN hue is treated as 0 (matching culori's `h === undefined` fallback
//! inside `convertLchToLab`).

use crate::spaces::{Lab, Xyz65};
use crate::traits::ColorSpace;

/// CIE Lch (D50). `l` is in 0..100, `c` (chroma) is non-negative, `h`
/// (hue, degrees) is normalized to 0..360. For achromatic colors `h` is
/// NaN.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Lch {
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
    // culori's `util/normalizeHue.js`:
    //   const normalizeHue = hue => ((hue = hue % 360) < 0 ? hue + 360 : hue);
    let h = h % 360.0;
    if h < 0.0 {
        h + 360.0
    } else {
        h
    }
}

impl ColorSpace for Lch {
    const MODE: &'static str = "lch";
    const CHANNELS: &'static [&'static str] = &["l", "c", "h"];

    fn alpha(&self) -> Option<f64> {
        self.alpha
    }

    fn with_alpha(self, alpha: Option<f64>) -> Self {
        Self { alpha, ..self }
    }

    fn to_xyz65(&self) -> Xyz65 {
        Lab::from(*self).to_xyz65()
    }

    fn from_xyz65(xyz: Xyz65) -> Self {
        Lab::from_xyz65(xyz).into()
    }
}

impl From<Lab> for Lch {
    fn from(c: Lab) -> Self {
        // culori's `convertLabToLch.js`:
        //   let c = Math.sqrt(a*a + b*b);
        //   let res = { mode, l, c };
        //   if (c) res.h = normalizeHue((Math.atan2(b, a) * 180) / Math.PI);
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

impl From<Lch> for Lab {
    fn from(c: Lch) -> Self {
        // culori's `convertLchToLab.js`:
        //   if (h === undefined) h = 0;
        //   a: c ? c * Math.cos((h / 180) * Math.PI) : 0,
        //   b: c ? c * Math.sin((h / 180) * Math.PI) : 0
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
