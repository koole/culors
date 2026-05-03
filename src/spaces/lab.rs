//! CIE Lab color space, D50 illuminant.
//!
//! Lifted verbatim from culori 4.0.2 (`node_modules/culori/src/lab/`,
//! `node_modules/culori/src/xyz50/constants.js`,
//! `node_modules/culori/src/constants.js`). Lab uses the D50 white point;
//! the hub conversion goes through Xyz50 and then Bradford-adapts to Xyz65.

#![allow(clippy::excessive_precision)]

use crate::spaces::{Xyz50, Xyz65};
use crate::traits::ColorSpace;

/// Reference white point (D50, CIE 1931 2°) — culori's `D50.X`, `D50.Y`,
/// `D50.Z`. Computed as `0.3457 / 0.3585`, `1`, `(1 - 0.3457 - 0.3585) /
/// 0.3585`.
const D50_X: f64 = 0.9642956764295677;
const D50_Y: f64 = 1.0;
const D50_Z: f64 = 0.8251046025104602;

/// Constants from the CIE Lab specification, matching culori's
/// `xyz50/constants.js`. `K = 29^3 / 3^3`, `E = 6^3 / 29^3`.
const K: f64 = 903.2962962962963;
const E: f64 = 0.008856451679035631;

/// CIE Lab color, D50 illuminant. `l` is in 0..100 for in-gamut colors,
/// `a` and `b` are signed (roughly -128..127 for sRGB inputs).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Lab {
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

impl ColorSpace for Lab {
    const MODE: &'static str = "lab";
    const CHANNELS: &'static [&'static str] = &["l", "a", "b"];

    fn alpha(&self) -> Option<f64> {
        self.alpha
    }

    fn with_alpha(self, alpha: Option<f64>) -> Self {
        Self { alpha, ..self }
    }

    fn to_xyz65(&self) -> Xyz65 {
        Xyz50::from(*self).to_xyz65()
    }

    fn from_xyz65(xyz: Xyz65) -> Self {
        Xyz50::from_xyz65(xyz).into()
    }
}

impl From<Xyz50> for Lab {
    fn from(xyz: Xyz50) -> Self {
        let f0 = f_forward(xyz.x / D50_X);
        let f1 = f_forward(xyz.y / D50_Y);
        let f2 = f_forward(xyz.z / D50_Z);
        Self {
            l: 116.0 * f1 - 16.0,
            a: 500.0 * (f0 - f1),
            b: 200.0 * (f1 - f2),
            alpha: xyz.alpha,
        }
    }
}

impl From<Lab> for Xyz50 {
    fn from(lab: Lab) -> Self {
        let fy = (lab.l + 16.0) / 116.0;
        let fx = lab.a / 500.0 + fy;
        let fz = fy - lab.b / 200.0;
        Self {
            x: f_inverse(fx) * D50_X,
            y: f_inverse(fy) * D50_Y,
            z: f_inverse(fz) * D50_Z,
            alpha: lab.alpha,
        }
    }
}
