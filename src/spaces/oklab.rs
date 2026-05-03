//! Oklab color space (Björn Ottosson, 2020).
//!
//! Constants and matrices verbatim from culori 4.0.2
//! (`node_modules/culori/src/oklab/convertLrgbToOklab.js`,
//! `node_modules/culori/src/oklab/convertOklabToLrgb.js`). Oklab is defined
//! relative to LINEAR sRGB; the cube-root non-linearity is on LMS-shaped
//! cone responses, not on perceptually-encoded sRGB.

#![allow(clippy::excessive_precision)]

use crate::spaces::{LinearRgb, Xyz65};
use crate::traits::ColorSpace;

/// Oklab — perceptually uniform color space. `l` is in 0..1 for in-gamut
/// colors, `a` and `b` are signed (roughly -0.5..0.5).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Oklab {
    /// Lightness in 0..1.
    pub l: f64,
    /// Green/red opponent channel.
    pub a: f64,
    /// Blue/yellow opponent channel.
    pub b: f64,
    /// Optional alpha in 0..1.
    pub alpha: Option<f64>,
}

impl ColorSpace for Oklab {
    const MODE: &'static str = "oklab";
    const CHANNELS: &'static [&'static str] = &["l", "a", "b"];

    fn alpha(&self) -> Option<f64> {
        self.alpha
    }

    fn with_alpha(self, alpha: Option<f64>) -> Self {
        Self { alpha, ..self }
    }

    fn to_xyz65(&self) -> Xyz65 {
        LinearRgb::from(*self).to_xyz65()
    }

    fn from_xyz65(xyz: Xyz65) -> Self {
        LinearRgb::from_xyz65(xyz).into()
    }
}

impl From<LinearRgb> for Oklab {
    fn from(c: LinearRgb) -> Self {
        let LinearRgb { r, g, b, alpha } = c;
        let l_ = (0.412221469470763 * r + 0.5363325372617348 * g + 0.0514459932675022 * b).cbrt();
        let m_ = (0.2119034958178252 * r + 0.6806995506452344 * g + 0.1073969535369406 * b).cbrt();
        let s_ = (0.0883024591900564 * r + 0.2817188391361215 * g + 0.6299787016738222 * b).cbrt();
        Self {
            l: 0.210454268309314 * l_ + 0.7936177747023054 * m_ - 0.0040720430116193 * s_,
            a: 1.9779985324311684 * l_ - 2.4285922420485799 * m_ + 0.450593709617411 * s_,
            b: 0.0259040424655478 * l_ + 0.7827717124575296 * m_ - 0.8086757549230774 * s_,
            alpha,
        }
    }
}

impl From<Oklab> for LinearRgb {
    fn from(c: Oklab) -> Self {
        let l_ = c.l + 0.3963377773761749 * c.a + 0.2158037573099136 * c.b;
        let m_ = c.l - 0.1055613458156586 * c.a - 0.0638541728258133 * c.b;
        let s_ = c.l - 0.0894841775298119 * c.a - 1.2914855480194092 * c.b;
        let l3 = l_ * l_ * l_;
        let m3 = m_ * m_ * m_;
        let s3 = s_ * s_ * s_;
        Self {
            r: 4.0767416360759574 * l3 - 3.3077115392580616 * m3 + 0.2309699031821044 * s3,
            g: -1.2684379732850317 * l3 + 2.6097573492876887 * m3 - 0.3413193760026573 * s3,
            b: -0.0041960761386756 * l3 - 0.7034186179359362 * m3 + 1.7076146940746117 * s3,
            alpha: c.alpha,
        }
    }
}
