//! XYB — JPEG XL's perceptual color space.
//!
//! Constants and matrices verbatim from culori 4.0.2
//! (`node_modules/culori/src/xyb/`). XYB is defined on linear sRGB:
//! apply a biased cube-root non-linearity to LMS-shaped tristimulus
//! values, then mix into (X, Y, B) where X is L-M chroma, Y is L+M
//! luma, and B is S minus the L+M luma (chroma-from-luma).

#![allow(clippy::excessive_precision)]

use crate::spaces::{LinearRgb, Rgb, Xyz65};
use crate::traits::ColorSpace;

const BIAS: f64 = 0.00379307325527544933;
fn bias_cbrt() -> f64 {
    BIAS.cbrt()
}

#[inline]
fn fwd(v: f64) -> f64 {
    v.cbrt() - bias_cbrt()
}

#[inline]
fn inv(v: f64) -> f64 {
    (v + bias_cbrt()).powi(3)
}

/// XYB color (JPEG XL).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Xyb {
    /// Red/green opponent.
    pub x: f64,
    /// Luma.
    pub y: f64,
    /// Blue (chroma from luma — culori subtracts Y).
    pub b: f64,
    /// Optional alpha in 0..1.
    pub alpha: Option<f64>,
}

impl ColorSpace for Xyb {
    const MODE: &'static str = "xyb";
    const CHANNELS: &'static [&'static str] = &["x", "y", "b"];

    fn alpha(&self) -> Option<f64> {
        self.alpha
    }

    fn with_alpha(self, alpha: Option<f64>) -> Self {
        Self { alpha, ..self }
    }

    fn to_xyz65(&self) -> Xyz65 {
        Rgb::from(*self).to_xyz65()
    }

    fn from_xyz65(xyz: Xyz65) -> Self {
        Rgb::from_xyz65(xyz).into()
    }
}

impl From<Rgb> for Xyb {
    fn from(c: Rgb) -> Self {
        let lrgb = LinearRgb::from(c);
        let r = lrgb.r;
        let g = lrgb.g;
        let b = lrgb.b;
        let l = fwd(0.3 * r + 0.622 * g + 0.078 * b + BIAS);
        let m = fwd(0.23 * r + 0.692 * g + 0.078 * b + BIAS);
        let s =
            fwd(0.24342268924547819 * r + 0.20476744424496821 * g + 0.5518098665095536 * b + BIAS);
        Self {
            x: (l - m) / 2.0,
            y: (l + m) / 2.0,
            b: s - (l + m) / 2.0,
            alpha: lrgb.alpha,
        }
    }
}

impl From<Xyb> for Rgb {
    fn from(c: Xyb) -> Self {
        let l = inv(c.x + c.y) - BIAS;
        let m = inv(c.y - c.x) - BIAS;
        let s = inv(c.b + c.y) - BIAS;
        let r = 11.031566904639861 * l - 9.866943908131562 * m - 0.16462299650829934 * s;
        let g = -3.2541473810744237 * l + 4.418770377582723 * m - 0.16462299650829934 * s;
        let b_ = -3.6588512867136815 * l + 2.7129230459360922 * m + 1.9459282407775895 * s;
        Rgb::from(LinearRgb {
            r,
            g,
            b: b_,
            alpha: c.alpha,
        })
    }
}
