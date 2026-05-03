//! ProPhoto RGB color space.
//!
//! ProPhoto's reference white is D50, so its native matrix targets XYZ D50;
//! routing into the XYZ D65 hub goes through the existing
//! [`Xyz50`](crate::spaces::Xyz50) Bradford adaptation. Transfer constants
//! and matrix lifted verbatim from culori 4.0.2
//! (`node_modules/culori/src/prophoto/`).
//!
//! The transfer function uses a 1.8 gamma above `16/512` and a 16× linear
//! segment near zero (sign-preserving).

#![allow(clippy::excessive_precision)]

use crate::spaces::{Xyz50, Xyz65};
use crate::traits::ColorSpace;

#[inline]
fn linearize(v: f64) -> f64 {
    let abs = v.abs();
    if abs >= 16.0 / 512.0 {
        let sign = if v < 0.0 { -1.0 } else { 1.0 };
        sign * abs.powf(1.8)
    } else {
        v / 16.0
    }
}

#[inline]
fn gamma(v: f64) -> f64 {
    let abs = v.abs();
    if abs >= 1.0 / 512.0 {
        let sign = if v < 0.0 { -1.0 } else { 1.0 };
        sign * abs.powf(1.0 / 1.8)
    } else {
        16.0 * v
    }
}

/// ProPhoto RGB color with channels in the nominal 0..1 range.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProphotoRgb {
    /// Red channel (gamma-encoded).
    pub r: f64,
    /// Green channel (gamma-encoded).
    pub g: f64,
    /// Blue channel (gamma-encoded).
    pub b: f64,
    /// Optional alpha in 0..1.
    pub alpha: Option<f64>,
}

impl ProphotoRgb {
    /// Convert directly into XYZ D50, matching culori's native ProPhoto
    /// → XYZ50 path.
    pub fn to_xyz50(&self) -> Xyz50 {
        let r = linearize(self.r);
        let g = linearize(self.g);
        let b = linearize(self.b);
        let x = 0.7977666449006423 * r + 0.1351812974005331 * g + 0.0313477341283922 * b;
        let y = 0.2880748288194013 * r + 0.7118352342418731 * g + 0.0000899369387256 * b;
        let z = 0.0 * r + 0.0 * g + 0.8251046025104602 * b;
        Xyz50 {
            x,
            y,
            z,
            alpha: self.alpha,
        }
    }

    /// Construct a ProPhoto RGB color from XYZ D50, matching culori's
    /// native XYZ50 → ProPhoto path.
    pub fn from_xyz50(xyz: Xyz50) -> Self {
        let r =
            xyz.x * 1.3457868816471585 - xyz.y * 0.2555720873797946 - 0.0511018649755453 * xyz.z;
        let g =
            xyz.x * -0.5446307051249019 + xyz.y * 1.5082477428451466 + 0.0205274474364214 * xyz.z;
        let b = xyz.x * 0.0 + xyz.y * 0.0 + 1.2119675456389452 * xyz.z;
        Self {
            r: gamma(r),
            g: gamma(g),
            b: gamma(b),
            alpha: xyz.alpha,
        }
    }
}

impl ColorSpace for ProphotoRgb {
    const MODE: &'static str = "prophoto";
    const CHANNELS: &'static [&'static str] = &["r", "g", "b"];

    fn alpha(&self) -> Option<f64> {
        self.alpha
    }

    fn with_alpha(self, alpha: Option<f64>) -> Self {
        Self { alpha, ..self }
    }

    fn to_xyz65(&self) -> Xyz65 {
        self.to_xyz50().to_xyz65()
    }

    fn from_xyz65(xyz: Xyz65) -> Self {
        Self::from_xyz50(Xyz50::from_xyz65(xyz))
    }
}
