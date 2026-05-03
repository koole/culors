//! OkHSL — Oklab-derived HSL.
//!
//! Constants and algorithm verbatim from culori 4.0.2
//! (`node_modules/culori/src/okhsl/`), itself adapted from Björn
//! Ottosson's reference. Channels: `h` (degrees), `s` (0..1), `l`
//! (perceptual 0..1, computed via [`toe`](super::okhsl_helpers::toe)).

use crate::spaces::okhsl_helpers::{get_cs, toe, toe_inv};
use crate::spaces::{Oklab, Rgb, Xyz65};
use crate::traits::ColorSpace;
use crate::util::normalize_hue;

/// OkHSL color.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Okhsl {
    /// Hue in degrees.
    pub h: f64,
    /// Saturation in 0..1.
    pub s: f64,
    /// Perceptual lightness in 0..1.
    pub l: f64,
    /// Optional alpha in 0..1.
    pub alpha: Option<f64>,
}

impl ColorSpace for Okhsl {
    const MODE: &'static str = "okhsl";
    const CHANNELS: &'static [&'static str] = &["h", "s", "l"];

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

impl From<Okhsl> for Oklab {
    fn from(c: Okhsl) -> Self {
        let h = c.h;
        let s = c.s;
        let l = c.l;

        let mut out = Oklab {
            l: toe_inv(l),
            a: 0.0,
            b: 0.0,
            alpha: c.alpha,
        };
        if s == 0.0 || l == 1.0 {
            return out;
        }

        let a_ = (h.to_radians()).cos();
        let b_ = (h.to_radians()).sin();
        let (c_0, c_mid, c_max) = get_cs(out.l, a_, b_);

        let (t, k_0, k_1, k_2) = if s < 0.8 {
            let t = 1.25 * s;
            let k_1 = 0.8 * c_0;
            let k_2 = 1.0 - k_1 / c_mid;
            (t, 0.0_f64, k_1, k_2)
        } else {
            let t = 5.0 * (s - 0.8);
            let k_0 = c_mid;
            let k_1 = (0.2 * c_mid * c_mid * 1.25 * 1.25) / c_0;
            let k_2 = 1.0 - k_1 / (c_max - c_mid);
            (t, k_0, k_1, k_2)
        };

        let chroma = k_0 + (t * k_1) / (1.0 - k_2 * t);
        out.a = chroma * a_;
        out.b = chroma * b_;
        out
    }
}

impl From<Oklab> for Okhsl {
    fn from(lab: Oklab) -> Self {
        let l = lab.l;
        let a = lab.a;
        let b = lab.b;
        let mut out = Okhsl {
            h: f64::NAN,
            s: 0.0,
            l: toe(l),
            alpha: lab.alpha,
        };
        let c = (a * a + b * b).sqrt();
        if c == 0.0 {
            return out;
        }
        let (c_0, c_mid, c_max) = get_cs(l, a / c, b / c);
        let s = if c < c_mid {
            let k_1 = 0.8 * c_0;
            let k_2 = 1.0 - k_1 / c_mid;
            let t = c / (k_1 + k_2 * c);
            t * 0.8
        } else {
            let k_0 = c_mid;
            let k_1 = (0.2 * c_mid * c_mid * 1.25 * 1.25) / c_0;
            let k_2 = 1.0 - k_1 / (c_max - c_mid);
            let t = (c - k_0) / (k_1 + k_2 * (c - k_0));
            0.8 + 0.2 * t
        };
        if s != 0.0 {
            out.s = s;
            out.h = normalize_hue(b.atan2(a).to_degrees());
        }
        out
    }
}

/// Direct `Rgb` → `Okhsl` mirroring culori's
/// `convertOklabToOkhsl ∘ convertRgbToOklab`. Picks up the achromatic
/// snap from [`Oklab::from`](crate::spaces::Oklab::from).
impl From<Rgb> for Okhsl {
    fn from(c: Rgb) -> Self {
        Oklab::from(c).into()
    }
}
