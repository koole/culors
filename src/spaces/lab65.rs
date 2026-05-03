//! CIE Lab color space, D65 illuminant.
//!
//! Lifted verbatim from culori 4.0.2 (`node_modules/culori/src/lab65/`,
//! `node_modules/culori/src/xyz65/constants.js`,
//! `node_modules/culori/src/constants.js`). [`Lab65`] uses the D65 white
//! point and so the hub conversion is a direct trip through [`Xyz65`] —
//! no Bradford detour, unlike [`Lab`](crate::spaces::Lab) which is D50.

#![allow(clippy::excessive_precision)]

use crate::spaces::{Rgb, Xyz65};
use crate::traits::ColorSpace;

/// Reference white point (D65, CIE 1931 2°) — culori's `D65.X`, `D65.Y`,
/// `D65.Z` from `node_modules/culori/src/constants.js`. Kept as runtime
/// divisions (rather than precomputed literals) to match JS bit-for-bit
/// regardless of how the host f64 parser rounds the intermediate
/// constants.
const D65_X: f64 = 0.3127 / 0.329;
const D65_Y: f64 = 1.0;
const D65_Z: f64 = (1.0 - 0.3127 - 0.329) / 0.329;

/// Constants from the CIE Lab specification, matching culori's
/// `xyz65/constants.js`. `K = 29^3 / 3^3`, `E = 6^3 / 29^3`.
const K: f64 = 903.2962962962963;
const E: f64 = 0.008856451679035631;

/// CIE Lab color, D65 illuminant. `l` is in 0..100 for in-gamut colors,
/// `a` and `b` are signed (roughly -125..125 for sRGB inputs, matching
/// culori's range hints).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Lab65 {
    /// Lightness (0..100).
    pub l: f64,
    /// Green/red opponent channel.
    pub a: f64,
    /// Blue/yellow opponent channel.
    pub b: f64,
    /// Optional alpha in 0..1.
    pub alpha: Option<f64>,
}

#[inline]
fn f_forward(value: f64) -> f64 {
    if value > E {
        value.cbrt()
    } else {
        (K * value + 16.0) / 116.0
    }
}

#[inline]
fn f_inverse(v: f64) -> f64 {
    let v3 = v * v * v;
    if v3 > E {
        v3
    } else {
        (116.0 * v - 16.0) / K
    }
}

impl ColorSpace for Lab65 {
    const MODE: &'static str = "lab65";
    const CHANNELS: &'static [&'static str] = &["l", "a", "b"];

    fn alpha(&self) -> Option<f64> {
        self.alpha
    }

    fn with_alpha(self, alpha: Option<f64>) -> Self {
        Self { alpha, ..self }
    }

    fn to_xyz65(&self) -> Xyz65 {
        let fy = (self.l + 16.0) / 116.0;
        let fx = self.a / 500.0 + fy;
        let fz = fy - self.b / 200.0;
        Xyz65 {
            x: f_inverse(fx) * D65_X,
            y: f_inverse(fy) * D65_Y,
            z: f_inverse(fz) * D65_Z,
            alpha: self.alpha,
        }
    }

    fn from_xyz65(xyz: Xyz65) -> Self {
        Self::from(xyz)
    }
}

impl From<Xyz65> for Lab65 {
    fn from(xyz: Xyz65) -> Self {
        let f0 = f_forward(xyz.x / D65_X);
        let f1 = f_forward(xyz.y / D65_Y);
        let f2 = f_forward(xyz.z / D65_Z);
        Self {
            l: 116.0 * f1 - 16.0,
            a: 500.0 * (f0 - f1),
            b: 200.0 * (f1 - f2),
            alpha: xyz.alpha,
        }
    }
}

/// Direct `Rgb` -> `Lab65` conversion mirroring culori's
/// `convertRgbToLab65.js`: route through Xyz65 and snap `a` and `b` to
/// exactly zero when the input is achromatic (`r == g == b`). Without the
/// snap the matrix multiply leaves a residual on the order of 1e-6 in both
/// opponent channels, which feeds a phantom hue into [`Lch65`](crate::spaces::Lch65).
impl From<Rgb> for Lab65 {
    fn from(c: Rgb) -> Self {
        let mut lab = Lab65::from(c.to_xyz65());
        if c.r == c.g && c.g == c.b {
            lab.a = 0.0;
            lab.b = 0.0;
        }
        lab
    }
}
