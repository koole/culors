//! CIE XYZ tristimulus values, D50 illuminant.
//!
//! Bradford chromatic adaptation links D50 and D65. The matrix coefficients
//! are lifted verbatim from culori 4.0.2
//! (`node_modules/culori/src/xyz65/convertXyz65ToXyz50.js`,
//! `node_modules/culori/src/xyz65/convertXyz50ToXyz65.js`).

#![allow(clippy::excessive_precision)]

use crate::spaces::Xyz65;
use crate::traits::ColorSpace;

/// CIE XYZ color with the D50 illuminant. Channels are nominally in 0..1
/// for in-gamut colors but may exceed it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Xyz50 {
    /// X tristimulus value.
    pub x: f64,
    /// Y tristimulus value (luminance).
    pub y: f64,
    /// Z tristimulus value.
    pub z: f64,
    /// Optional alpha in 0..1.
    pub alpha: Option<f64>,
}

impl ColorSpace for Xyz50 {
    const MODE: &'static str = "xyz50";
    const CHANNELS: &'static [&'static str] = &["x", "y", "z"];

    fn alpha(&self) -> Option<f64> {
        self.alpha
    }

    fn with_alpha(self, alpha: Option<f64>) -> Self {
        Self { alpha, ..self }
    }

    fn to_xyz65(&self) -> Xyz65 {
        let x =
            0.9554734527042182 * self.x - 0.0230985368742614 * self.y + 0.0632593086610217 * self.z;
        let y =
            -0.0283697069632081 * self.x + 1.0099954580058226 * self.y + 0.021041398966943 * self.z;
        let z =
            0.0123140016883199 * self.x - 0.0205076964334779 * self.y + 1.3303659366080753 * self.z;
        Xyz65 {
            x,
            y,
            z,
            alpha: self.alpha,
        }
    }

    fn from_xyz65(xyz: Xyz65) -> Self {
        let x =
            1.0479298208405488 * xyz.x + 0.0229467933410191 * xyz.y - 0.0501922295431356 * xyz.z;
        let y = 0.0296278156881593 * xyz.x + 0.990434484573249 * xyz.y - 0.0170738250293851 * xyz.z;
        let z =
            -0.0092430581525912 * xyz.x + 0.0150551448965779 * xyz.y + 0.7518742899580008 * xyz.z;
        Self {
            x,
            y,
            z,
            alpha: xyz.alpha,
        }
    }
}
