//! DIN99o LCh — polar form of DIN99o.
//!
//! Constants and formulas verbatim from culori 4.0.2
//! (`node_modules/culori/src/dlch/`).

use crate::spaces::dlab::{dlch_to_lab65, lab65_to_dlch};
use crate::spaces::{Rgb, Xyz65};
use crate::traits::ColorSpace;
use crate::util::{lab65_to_xyz65, xyz65_to_lab65};

/// DIN99o LCh. `l` is in 0..100, `c` is non-negative, `h` is in degrees.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Dlch {
    /// Lightness in 0..100.
    pub l: f64,
    /// Chroma.
    pub c: f64,
    /// Hue in degrees.
    pub h: f64,
    /// Optional alpha in 0..1.
    pub alpha: Option<f64>,
}

impl ColorSpace for Dlch {
    const MODE: &'static str = "dlch";
    const CHANNELS: &'static [&'static str] = &["l", "c", "h"];

    fn alpha(&self) -> Option<f64> {
        self.alpha
    }

    fn with_alpha(self, alpha: Option<f64>) -> Self {
        Self { alpha, ..self }
    }

    fn to_xyz65(&self) -> Xyz65 {
        let h = if self.h.is_nan() { 0.0 } else { self.h };
        let (l65, a65, b65) = dlch_to_lab65(self.l, self.c, h);
        let (x, y, z) = lab65_to_xyz65(l65, a65, b65);
        Xyz65 {
            x,
            y,
            z,
            alpha: self.alpha,
        }
    }

    fn from_xyz65(xyz: Xyz65) -> Self {
        let (l65, a65, b65) = xyz65_to_lab65(xyz.x, xyz.y, xyz.z);
        let (l, c, h) = lab65_to_dlch(l65, a65, b65);
        Self {
            l,
            c,
            h,
            alpha: xyz.alpha,
        }
    }
}

/// Direct `Rgb` → `Dlch` matching culori's achromatic-RGB snap. See the
/// `From<Rgb> for Dlab` doc for why the snap is necessary.
impl From<Rgb> for Dlch {
    fn from(c: Rgb) -> Self {
        use crate::traits::ColorSpace;
        let xyz = c.to_xyz65();
        let (mut l, mut a, mut b) = xyz65_to_lab65(xyz.x, xyz.y, xyz.z);
        if c.r == c.g && c.g == c.b {
            a = 0.0;
            b = 0.0;
            let _ = &mut l;
        }
        let (dl, dc, dh) = lab65_to_dlch(l, a, b);
        Self {
            l: dl,
            c: dc,
            h: dh,
            alpha: c.alpha,
        }
    }
}
