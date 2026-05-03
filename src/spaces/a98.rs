//! Adobe RGB (1998) color space.
//!
//! A98 RGB uses Adobe's 1998 primaries with a D65 white point. The transfer
//! function is a single-power gamma of `563/256 ≈ 2.19922` (sign-preserving).
//! Constants and matrix lifted verbatim from culori 4.0.2
//! (`node_modules/culori/src/a98/`).

#![allow(clippy::excessive_precision)]

use crate::spaces::Xyz65;
use crate::traits::ColorSpace;

#[inline]
fn linearize(v: f64) -> f64 {
    let sign = if v < 0.0 { -1.0 } else { 1.0 };
    v.abs().powf(563.0 / 256.0) * sign
}

#[inline]
fn gamma(v: f64) -> f64 {
    let sign = if v < 0.0 { -1.0 } else { 1.0 };
    v.abs().powf(256.0 / 563.0) * sign
}

/// Adobe RGB (1998) color with channels in the nominal 0..1 range.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct A98 {
    /// Red channel (gamma-encoded).
    pub r: f64,
    /// Green channel (gamma-encoded).
    pub g: f64,
    /// Blue channel (gamma-encoded).
    pub b: f64,
    /// Optional alpha in 0..1.
    pub alpha: Option<f64>,
}

impl ColorSpace for A98 {
    const MODE: &'static str = "a98";
    const CHANNELS: &'static [&'static str] = &["r", "g", "b"];

    fn alpha(&self) -> Option<f64> {
        self.alpha
    }

    fn with_alpha(self, alpha: Option<f64>) -> Self {
        Self { alpha, ..self }
    }

    fn to_xyz65(&self) -> Xyz65 {
        let r = linearize(self.r);
        let g = linearize(self.g);
        let b = linearize(self.b);
        let x = 0.5766690429101305 * r + 0.1855582379065463 * g + 0.1882286462349947 * b;
        let y = 0.297344975250536 * r + 0.6273635662554661 * g + 0.0752914584939979 * b;
        let z = 0.0270313613864123 * r + 0.0706888525358272 * g + 0.9913375368376386 * b;
        Xyz65 {
            x,
            y,
            z,
            alpha: self.alpha,
        }
    }

    fn from_xyz65(xyz: Xyz65) -> Self {
        let r =
            xyz.x * 2.0415879038107465 - xyz.y * 0.5650069742788597 - 0.3447313507783297 * xyz.z;
        let g =
            xyz.x * -0.9692436362808798 + xyz.y * 1.8759675015077206 + 0.0415550574071756 * xyz.z;
        let b =
            xyz.x * 0.0134442806320312 - xyz.y * 0.1183623922310184 + 1.0151749943912058 * xyz.z;
        Self {
            r: gamma(r),
            g: gamma(g),
            b: gamma(b),
            alpha: xyz.alpha,
        }
    }
}
