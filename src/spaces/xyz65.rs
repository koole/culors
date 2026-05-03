//! XYZ tristimulus values, D65 illuminant. The conversion hub for `culor`.

use crate::traits::ColorSpace;

/// CIE XYZ color with the D65 illuminant. Channels are in the nominal 0..1
/// range for in-gamut colors but may exceed it for HDR or extended-gamut
/// values.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Xyz65 {
    /// X tristimulus value.
    pub x: f64,
    /// Y tristimulus value (luminance).
    pub y: f64,
    /// Z tristimulus value.
    pub z: f64,
    /// Optional alpha in 0..1.
    pub alpha: Option<f64>,
}

impl ColorSpace for Xyz65 {
    const MODE: &'static str = "xyz65";
    const CHANNELS: &'static [&'static str] = &["x", "y", "z", "alpha"];

    fn alpha(&self) -> Option<f64> {
        self.alpha
    }

    fn with_alpha(self, alpha: Option<f64>) -> Self {
        Self { alpha, ..self }
    }

    fn to_xyz65(&self) -> Xyz65 {
        *self
    }

    fn from_xyz65(xyz: Xyz65) -> Self {
        xyz
    }
}
