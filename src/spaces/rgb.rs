//! sRGB color space (gamma-encoded, nominal 0..1 channels).
//!
//! The sRGB ↔ linear-sRGB transfer function and the linear-sRGB ↔ XYZ D65
//! matrix are both lifted verbatim from culori 4.0.2; the constants live in
//! [`crate::util`].

use crate::spaces::{LinearRgb, Xyz65};
use crate::traits::ColorSpace;
use crate::util::{linear_to_srgb, lrgb_to_xyz65, srgb_to_linear, xyz65_to_lrgb};

/// sRGB color with channels in the nominal 0..1 range. Values outside that
/// range are valid HDR-style extended-range colors; the transfer function
/// preserves sign, matching culori's behavior.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rgb {
    /// Red channel.
    pub r: f64,
    /// Green channel.
    pub g: f64,
    /// Blue channel.
    pub b: f64,
    /// Optional alpha in 0..1.
    pub alpha: Option<f64>,
}

impl ColorSpace for Rgb {
    const MODE: &'static str = "rgb";
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
        let (x, y, z) = lrgb_to_xyz65(r, g, b);
        Xyz65 {
            x,
            y,
            z,
            alpha: self.alpha,
        }
    }

    fn from_xyz65(xyz: Xyz65) -> Self {
        let (r, g, b) = xyz65_to_lrgb(xyz.x, xyz.y, xyz.z);
        Self {
            r: linear_to_srgb(r),
            g: linear_to_srgb(g),
            b: linear_to_srgb(b),
            alpha: xyz.alpha,
        }
    }
}

impl From<LinearRgb> for Rgb {
    fn from(c: LinearRgb) -> Self {
        Self {
            r: linear_to_srgb(c.r),
            g: linear_to_srgb(c.g),
            b: linear_to_srgb(c.b),
            alpha: c.alpha,
        }
    }
}

impl From<Rgb> for LinearRgb {
    fn from(c: Rgb) -> Self {
        Self {
            r: srgb_to_linear(c.r),
            g: srgb_to_linear(c.g),
            b: srgb_to_linear(c.b),
            alpha: c.alpha,
        }
    }
}
