//! JzAzBz — HDR-friendly perceptually-uniform Lab variant.
//!
//! Constants and formulas verbatim from culori 4.0.2
//! (`node_modules/culori/src/jab/`, `node_modules/culori/src/hdr/transfer.js`).
//! The JzAzBz space sits on absolute XYZ (Y=203 cd/m² mapping) with a
//! Perceptual-Quantizer-derived non-linearity.

#![allow(clippy::excessive_precision)]

use crate::spaces::{Rgb, Xyz65};
use crate::traits::ColorSpace;

// PQ-based transfer constants verbatim from culori `hdr/transfer.js`.
const M1: f64 = 0.1593017578125;
const C1: f64 = 0.8359375;
const C2: f64 = 18.8515625;
const C3: f64 = 18.6875;

// JzAzBz-specific PQ exponent and Jz offset (`jab/convertXyz65ToJab.js`).
const P: f64 = 134.03437499999998; // 1.7 * 2523 / 2^5
const D0: f64 = 1.6295499532821566e-11;

#[inline]
fn jab_pq_encode(v: f64) -> f64 {
    if v < 0.0 {
        0.0
    } else {
        let vn = (v / 10000.0).powf(M1);
        ((C1 + C2 * vn) / (1.0 + C3 * vn)).powf(P)
    }
}

#[inline]
fn jab_pq_decode(v: f64) -> f64 {
    if v < 0.0 {
        0.0
    } else {
        let vp = v.powf(1.0 / P);
        10000.0 * ((C1 - vp) / (C3 * vp - C2)).powf(1.0 / M1)
    }
}

/// JzAzBz color. `j` is in 0..~0.222 for in-gamut sRGB; `a`/`b` are
/// signed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Jab {
    /// Lightness (Jz).
    pub j: f64,
    /// Green/red opponent (Az).
    pub a: f64,
    /// Blue/yellow opponent (Bz).
    pub b: f64,
    /// Optional alpha in 0..1.
    pub alpha: Option<f64>,
}

impl ColorSpace for Jab {
    const MODE: &'static str = "jab";
    const CHANNELS: &'static [&'static str] = &["j", "a", "b"];

    fn alpha(&self) -> Option<f64> {
        self.alpha
    }

    fn with_alpha(self, alpha: Option<f64>) -> Self {
        Self { alpha, ..self }
    }

    fn to_xyz65(&self) -> Xyz65 {
        let j = self.j;
        let a = self.a;
        let b = self.b;
        let i = (j + D0) / (0.44 + 0.56 * (j + D0));

        let l = jab_pq_decode(i + 0.13860504 * a + 0.058047316 * b);
        let m = jab_pq_decode(i - 0.13860504 * a - 0.058047316 * b);
        let s = jab_pq_decode(i - 0.096019242 * a - 0.8118919 * b);

        let rel = |v: f64| v / 203.0;
        Xyz65 {
            x: rel(1.661373024652174 * l - 0.914523081304348 * m + 0.23136208173913045 * s),
            y: rel(-0.3250758611844533 * l + 1.571847026732543 * m - 0.21825383453227928 * s),
            z: rel(-0.090982811 * l - 0.31272829 * m + 1.5227666 * s),
            alpha: self.alpha,
        }
    }

    fn from_xyz65(xyz: Xyz65) -> Self {
        let abs = |v: f64| (v * 203.0).max(0.0);
        let x = abs(xyz.x);
        let y = abs(xyz.y);
        let z = abs(xyz.z);

        let xp = 1.15 * x - 0.15 * z;
        let yp = 0.66 * y + 0.34 * x;

        let l = jab_pq_encode(0.41478972 * xp + 0.579999 * yp + 0.014648 * z);
        let m = jab_pq_encode(-0.20151 * xp + 1.120649 * yp + 0.0531008 * z);
        let s = jab_pq_encode(-0.0166008 * xp + 0.2648 * yp + 0.6684799 * z);

        let i = (l + m) / 2.0;

        Self {
            j: (0.44 * i) / (1.0 - 0.56 * i) - D0,
            a: 3.524 * l - 4.066708 * m + 0.542708 * s,
            b: 0.199076 * l + 1.096799 * m - 1.295875 * s,
            alpha: xyz.alpha,
        }
    }
}

/// Direct `Rgb` → `Jab` mirroring culori's `convertRgbToJab.js`: route
/// through xyz65 then snap `a` and `b` to zero for achromatic sRGB inputs.
impl From<Rgb> for Jab {
    fn from(c: Rgb) -> Self {
        let mut jab = Jab::from_xyz65(c.to_xyz65());
        if c.r == c.g && c.g == c.b {
            jab.a = 0.0;
            jab.b = 0.0;
        }
        jab
    }
}
