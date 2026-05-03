//! DIN99o Lab — rectangular form of DIN99o LCh.
//!
//! Constants and formulas verbatim from culori 4.0.2
//! (`node_modules/culori/src/dlab/`, `dlch/`, `lab65/`). The rectangular
//! form is obtained by going Dlab → Dlch (Lab → Lch) → Lab65 (and back),
//! exactly as `dlab/definition.js` defines `convertDlabToLab65` /
//! `convertLab65ToDlab`.

#![allow(clippy::excessive_precision)]

use crate::spaces::Xyz65;
use crate::traits::ColorSpace;
use crate::util::{lab65_to_xyz65, normalize_hue, xyz65_to_lab65};

const K_E: f64 = 1.0;
const K_CH: f64 = 1.0;
const THETA: f64 = 26.0_f64 / 180.0 * std::f64::consts::PI;
// `factor = 100 / Math.log(139 / 100)` — kept as a runtime const, since
// `f64::ln` is not const.
fn factor() -> f64 {
    100.0 / (139.0_f64 / 100.0).ln()
}

/// DIN99o Lab. `l` ranges in 0..100, `a`/`b` are signed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Dlab {
    /// Lightness in 0..100.
    pub l: f64,
    /// Green/red opponent.
    pub a: f64,
    /// Blue/yellow opponent.
    pub b: f64,
    /// Optional alpha in 0..1.
    pub alpha: Option<f64>,
}

impl ColorSpace for Dlab {
    const MODE: &'static str = "dlab";
    const CHANNELS: &'static [&'static str] = &["l", "a", "b"];

    fn alpha(&self) -> Option<f64> {
        self.alpha
    }

    fn with_alpha(self, alpha: Option<f64>) -> Self {
        Self { alpha, ..self }
    }

    fn to_xyz65(&self) -> Xyz65 {
        // Dlab → Dlch (rect→polar) → Lab65 → XYZ65.
        let c = (self.a * self.a + self.b * self.b).sqrt();
        let h = if c == 0.0 {
            0.0
        } else {
            normalize_hue(self.b.atan2(self.a).to_degrees())
        };
        let (l65, a65, b65) = dlch_to_lab65(self.l, c, h);
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
        let (dl, dc, dh) = lab65_to_dlch(l65, a65, b65);
        // Dlch → Dlab (polar→rect): culori uses `convertLchToLab(c, 'dlab')`
        // which sets a = c * cos(h°), b = c * sin(h°). Hue NaN means
        // achromatic, so a = b = 0.
        let (a, b) = if dh.is_nan() {
            (0.0, 0.0)
        } else {
            let hr = dh.to_radians();
            (dc * hr.cos(), dc * hr.sin())
        };
        Self {
            l: dl,
            a,
            b,
            alpha: xyz.alpha,
        }
    }
}

/// DIN99o LCh → CIELab D65. Mirrors culori's `convertDlchToLab65`.
pub(crate) fn dlch_to_lab65(l: f64, c: f64, h: f64) -> (f64, f64, f64) {
    let l_lab = ((l * K_E / factor()).exp() - 1.0) / 0.0039;
    let g = ((0.0435 * c * K_CH * K_E).exp() - 1.0) / 0.075;
    let h_rad = h.to_radians() - THETA;
    let e = g * h_rad.cos();
    let f = g * h_rad.sin();
    let cos_t = THETA.cos();
    let sin_t = THETA.sin();
    let a = e * cos_t - (f / 0.83) * sin_t;
    let b = e * sin_t + (f / 0.83) * cos_t;
    (l_lab, a, b)
}

/// CIELab D65 → DIN99o LCh. Mirrors culori's `convertLab65ToDlch`. When
/// the chroma is zero the hue is undefined; we encode that as `f64::NAN`.
pub(crate) fn lab65_to_dlch(l: f64, a: f64, b: f64) -> (f64, f64, f64) {
    let cos_t = THETA.cos();
    let sin_t = THETA.sin();
    let e = a * cos_t + b * sin_t;
    let f = 0.83 * (b * cos_t - a * sin_t);
    let g = (e * e + f * f).sqrt();
    let dl = (factor() / K_E) * (1.0 + 0.0039 * l).ln();
    let dc = (1.0 + 0.075 * g).ln() / (0.0435 * K_CH * K_E);
    let dh = if dc == 0.0 {
        f64::NAN
    } else {
        normalize_hue(((f.atan2(e) + THETA) / std::f64::consts::PI) * 180.0)
    };
    (dl, dc, dh)
}
