//! OkHSV — Oklab-derived HSV.
//!
//! Constants and algorithm verbatim from culori 4.0.2
//! (`node_modules/culori/src/okhsv/`), itself adapted from Björn
//! Ottosson's reference.

use crate::spaces::okhsl_helpers::{get_st_max, toe, toe_inv};
use crate::spaces::{LinearRgb, Oklab, Rgb, Xyz65};
use crate::traits::ColorSpace;
use crate::util::normalize_hue;

/// OkHSV color.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Okhsv {
    /// Hue in degrees.
    pub h: f64,
    /// Saturation in 0..1.
    pub s: f64,
    /// Value (perceptual) in 0..1.
    pub v: f64,
    /// Optional alpha in 0..1.
    pub alpha: Option<f64>,
}

impl ColorSpace for Okhsv {
    const MODE: &'static str = "okhsv";
    const CHANNELS: &'static [&'static str] = &["h", "s", "v"];

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

impl From<Okhsv> for Oklab {
    fn from(c: Okhsv) -> Self {
        let h = c.h;
        let s = c.s;
        let v = c.v;

        let a_ = h.to_radians().cos();
        let b_ = h.to_radians().sin();

        let (s_max, t) = get_st_max(a_, b_, None);
        let s_0 = 0.5;
        let k = 1.0 - s_0 / s_max;
        let l_v = 1.0 - (s * s_0) / (s_0 + t - t * k * s);
        let c_v = (s * t * s_0) / (s_0 + t - t * k * s);

        let l_vt = toe_inv(l_v);
        let c_vt = (c_v * l_vt) / l_v;
        let rgb_scale = LinearRgb::from(Oklab {
            l: l_vt,
            a: a_ * c_vt,
            b: b_ * c_vt,
            alpha: None,
        });
        let scale_l = (1.0 / rgb_scale.r.max(rgb_scale.g).max(rgb_scale.b).max(0.0)).cbrt();

        let l_new = toe_inv(v * l_v);
        let chroma = (c_v * l_new) / l_v;

        Self {
            l: l_new * scale_l,
            a: chroma * a_ * scale_l,
            b: chroma * b_ * scale_l,
            alpha: c.alpha,
        }
    }
}

impl From<Oklab> for Okhsv {
    fn from(lab: Oklab) -> Self {
        let mut l = lab.l;
        let a = lab.a;
        let b = lab.b;

        let mut c = (a * a + b * b).sqrt();
        let a_ = if c != 0.0 { a / c } else { 1.0 };
        let b_ = if c != 0.0 { b / c } else { 1.0 };

        let (s_max, t) = get_st_max(a_, b_, None);
        let s_0 = 0.5;
        let k = 1.0 - s_0 / s_max;

        let t_split = t / (c + l * t);
        let l_v = t_split * l;
        let c_v = t_split * c;

        let l_vt = toe_inv(l_v);
        let c_vt = (c_v * l_vt) / l_v;
        let rgb_scale = LinearRgb::from(Oklab {
            l: l_vt,
            a: a_ * c_vt,
            b: b_ * c_vt,
            alpha: None,
        });
        let scale_l = (1.0 / rgb_scale.r.max(rgb_scale.g).max(rgb_scale.b).max(0.0)).cbrt();

        l /= scale_l;
        c = ((c / scale_l) * toe(l)) / l;
        l = toe(l);

        let mut out = Okhsv {
            h: f64::NAN,
            s: if c != 0.0 {
                ((s_0 + t) * c_v) / (t * s_0 + t * k * c_v)
            } else {
                0.0
            },
            v: if l != 0.0 { l / l_v } else { 0.0 },
            alpha: lab.alpha,
        };
        if out.s != 0.0 {
            out.h = normalize_hue(b.atan2(a).to_degrees());
        }
        out
    }
}

/// Direct `Rgb` → `Okhsv` mirroring culori's
/// `convertOklabToOkhsv ∘ convertRgbToOklab`.
impl From<Rgb> for Okhsv {
    fn from(c: Rgb) -> Self {
        Oklab::from(c).into()
    }
}
