//! Rec. 2020 color space.
//!
//! Rec. 2020 (ITU-R BT.2020) uses wide primaries with a D65 white point. Its
//! transfer function is the BT.1886-derived piecewise curve with constants
//! `α = 1.09929682680944`, `β = 0.018053968510807`, and a 0.45 exponent.
//! Constants and matrix lifted verbatim from culori 4.0.2
//! (`node_modules/culori/src/rec2020/`).

#![allow(clippy::excessive_precision)]

use crate::spaces::Xyz65;
use crate::traits::ColorSpace;

const ALPHA: f64 = 1.09929682680944;
const BETA: f64 = 0.018053968510807;

#[inline]
fn linearize(v: f64) -> f64 {
    let abs = v.abs();
    if abs < BETA * 4.5 {
        v / 4.5
    } else {
        let sign = if v < 0.0 { -1.0 } else { 1.0 };
        sign * ((abs + ALPHA - 1.0) / ALPHA).powf(1.0 / 0.45)
    }
}

#[inline]
fn gamma(v: f64) -> f64 {
    let abs = v.abs();
    if abs > BETA {
        let sign = if v < 0.0 { -1.0 } else { 1.0 };
        sign * (ALPHA * abs.powf(0.45) - (ALPHA - 1.0))
    } else {
        4.5 * v
    }
}

/// Rec. 2020 color with channels in the nominal 0..1 range.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rec2020 {
    /// Red channel (gamma-encoded).
    pub r: f64,
    /// Green channel (gamma-encoded).
    pub g: f64,
    /// Blue channel (gamma-encoded).
    pub b: f64,
    /// Optional alpha in 0..1.
    pub alpha: Option<f64>,
}

impl ColorSpace for Rec2020 {
    const MODE: &'static str = "rec2020";
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
        let x = 0.6369580483012911 * r + 0.1446169035862083 * g + 0.1688809751641721 * b;
        let y = 0.262700212011267 * r + 0.6779980715188708 * g + 0.059301716469862 * b;
        let z = 0.0 * r + 0.0280726930490874 * g + 1.0609850577107909 * b;
        Xyz65 {
            x,
            y,
            z,
            alpha: self.alpha,
        }
    }

    fn from_xyz65(xyz: Xyz65) -> Self {
        let r =
            xyz.x * 1.7166511879712683 - xyz.y * 0.3556707837763925 - 0.2533662813736599 * xyz.z;
        let g =
            xyz.x * -0.6666843518324893 + xyz.y * 1.6164812366349395 + 0.0157685458139111 * xyz.z;
        let b =
            xyz.x * 0.0176398574453108 - xyz.y * 0.0427706132578085 + 0.9421031212354739 * xyz.z;
        Self {
            r: gamma(r),
            g: gamma(g),
            b: gamma(b),
            alpha: xyz.alpha,
        }
    }
}
