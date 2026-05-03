//! Oklch color space (polar form of Oklab).
//!
//! Conversions track culori 4.0.2 (`node_modules/culori/src/oklch/`). Oklch
//! reuses culori's `convertLabToLch` / `convertLchToLab` directly with
//! `mode = 'oklch'` / `'oklab'`, so the polar math is identical to Lch —
//! only the Cartesian space differs. `to_xyz65` / `from_xyz65` compose
//! with the [`Oklab`] hub conversion (which itself goes via
//! [`LinearRgb`](crate::spaces::LinearRgb)).
//!
//! NaN-hue handling matches Lch: chroma exactly zero produces a NaN hue
//! sentinel; on the reverse path NaN coerces to 0.

use crate::spaces::{Oklab, Rgb, Xyz65};
use crate::traits::ColorSpace;

/// Oklch — polar Oklab. `l` is in 0..1, `c` (chroma) is non-negative,
/// `h` (hue, degrees) is normalized to 0..360. For achromatic colors
/// `h` is NaN.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Oklch {
    /// Lightness (0..1).
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

impl ColorSpace for Oklch {
    const MODE: &'static str = "oklch";
    const CHANNELS: &'static [&'static str] = &["l", "c", "h"];

    fn alpha(&self) -> Option<f64> {
        self.alpha
    }

    fn with_alpha(self, alpha: Option<f64>) -> Self {
        Self { alpha, ..self }
    }

    fn to_xyz65(&self) -> Xyz65 {
        Oklab::from(*self).to_xyz65()
    }

    fn from_xyz65(xyz: Xyz65) -> Self {
        Oklab::from_xyz65(xyz).into()
    }
}

impl From<Oklab> for Oklch {
    fn from(c: Oklab) -> Self {
        // culori's `convertLabToLch.js` (shared via oklch/definition.js):
        //   let c = Math.sqrt(a*a + b*b);
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

/// Direct `Rgb` -> `Oklch` conversion that picks up the achromatic snap
/// from `Oklab::from(Rgb)` so `r == g == b` produces `c = 0` and `h = NaN`,
/// matching culori's public `oklch({mode:'rgb', ...})` output.
impl From<Rgb> for Oklch {
    fn from(c: Rgb) -> Self {
        Oklch::from(Oklab::from(c))
    }
}

impl From<Oklch> for Oklab {
    fn from(c: Oklch) -> Self {
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
