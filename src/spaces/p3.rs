//! Display P3 color space.
//!
//! Display P3 shares sRGB's transfer function but uses the DCI-P3 primaries
//! and the D65 illuminant. Matrix coefficients and the (sign-preserving)
//! sRGB transfer are lifted verbatim from culori 4.0.2
//! (`node_modules/culori/src/p3/`).

#![allow(clippy::excessive_precision)]

use crate::spaces::Xyz65;
use crate::traits::ColorSpace;
use crate::util::{linear_to_srgb, srgb_to_linear};

/// Display P3 color with channels in the nominal 0..1 range. Same transfer
/// function as sRGB; different primaries.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct P3 {
    /// Red channel (gamma-encoded).
    pub r: f64,
    /// Green channel (gamma-encoded).
    pub g: f64,
    /// Blue channel (gamma-encoded).
    pub b: f64,
    /// Optional alpha in 0..1.
    pub alpha: Option<f64>,
}

#[inline]
fn p3_linear_to_xyz65(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    let x = 0.486570948648216 * r + 0.265667693169093 * g + 0.1982172852343625 * b;
    let y = 0.2289745640697487 * r + 0.6917385218365062 * g + 0.079286914093745 * b;
    let z = 0.0 * r + 0.0451133818589026 * g + 1.043944368900976 * b;
    (x, y, z)
}

#[inline]
fn xyz65_to_p3_linear(x: f64, y: f64, z: f64) -> (f64, f64, f64) {
    let r = x * 2.4934969119414263 - y * 0.9313836179191242 - 0.402710784450717 * z;
    let g = x * -0.8294889695615749 + y * 1.7626640603183465 + 0.0236246858419436 * z;
    let b = x * 0.0358458302437845 - y * 0.0761723892680418 + 0.9568845240076871 * z;
    (r, g, b)
}

impl ColorSpace for P3 {
    const MODE: &'static str = "p3";
    const CHANNELS: &'static [&'static str] = &["r", "g", "b"];

    fn alpha(&self) -> Option<f64> {
        self.alpha
    }

    fn with_alpha(self, alpha: Option<f64>) -> Self {
        Self { alpha, ..self }
    }

    fn to_xyz65(&self) -> Xyz65 {
        let r = srgb_to_linear(self.r);
        let g = srgb_to_linear(self.g);
        let b = srgb_to_linear(self.b);
        let (x, y, z) = p3_linear_to_xyz65(r, g, b);
        Xyz65 {
            x,
            y,
            z,
            alpha: self.alpha,
        }
    }

    fn from_xyz65(xyz: Xyz65) -> Self {
        let (r, g, b) = xyz65_to_p3_linear(xyz.x, xyz.y, xyz.z);
        Self {
            r: linear_to_srgb(r),
            g: linear_to_srgb(g),
            b: linear_to_srgb(b),
            alpha: xyz.alpha,
        }
    }
}
