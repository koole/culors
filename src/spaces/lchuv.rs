//! CIELChuv — polar form of CIELUV.
//!
//! Constants and formulas verbatim from culori 4.0.2
//! (`node_modules/culori/src/lchuv/`).

use crate::spaces::{Luv, Rgb, Xyz65};
use crate::traits::ColorSpace;
use crate::util::normalize_hue;

/// CIELChuv polar coordinates of [`Luv`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Lchuv {
    /// Lightness in 0..100.
    pub l: f64,
    /// Chroma.
    pub c: f64,
    /// Hue in degrees.
    pub h: f64,
    /// Optional alpha in 0..1.
    pub alpha: Option<f64>,
}

impl ColorSpace for Lchuv {
    const MODE: &'static str = "lchuv";
    const CHANNELS: &'static [&'static str] = &["l", "c", "h"];

    fn alpha(&self) -> Option<f64> {
        self.alpha
    }

    fn with_alpha(self, alpha: Option<f64>) -> Self {
        Self { alpha, ..self }
    }

    fn to_xyz65(&self) -> Xyz65 {
        Luv::from(*self).to_xyz65()
    }

    fn from_xyz65(xyz: Xyz65) -> Self {
        Luv::from_xyz65(xyz).into()
    }
}

impl From<Luv> for Lchuv {
    fn from(luv: Luv) -> Self {
        let c = (luv.u * luv.u + luv.v * luv.v).sqrt();
        let h = if c == 0.0 {
            f64::NAN
        } else {
            normalize_hue(luv.v.atan2(luv.u).to_degrees())
        };
        Self {
            l: luv.l,
            c,
            h,
            alpha: luv.alpha,
        }
    }
}

impl From<Lchuv> for Luv {
    fn from(c: Lchuv) -> Self {
        let h = if c.h.is_nan() { 0.0 } else { c.h };
        let (u, v) = if c.c == 0.0 {
            (0.0, 0.0)
        } else {
            let hr = h.to_radians();
            (c.c * hr.cos(), c.c * hr.sin())
        };
        Self {
            l: c.l,
            u,
            v,
            alpha: c.alpha,
        }
    }
}

/// Direct `Rgb` → `Lchuv` mirroring culori's chained conversion.
/// Reuses the achromatic-RGB snap from [`Luv::from`].
impl From<Rgb> for Lchuv {
    fn from(c: Rgb) -> Self {
        Luv::from(c).into()
    }
}
