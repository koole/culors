//! sRGB color space (gamma-encoded, nominal 0..1 channels).
//!
//! The sRGB ↔ linear-sRGB transfer function and the linear-sRGB ↔ XYZ D65
//! matrix are both lifted verbatim from culori 4.0.2
//! (`node_modules/culori/src/lrgb/convertRgbToLrgb.js`,
//! `node_modules/culori/src/lrgb/convertLrgbToRgb.js`,
//! `node_modules/culori/src/xyz65/convertRgbToXyz65.js`,
//! `node_modules/culori/src/xyz65/convertXyz65ToRgb.js`).

// Matrix coefficients are copied verbatim from culori; clippy flags them as
// over-precise but the redundant digits are harmless and preserve a 1:1 trace
// to the JS source.
#![allow(clippy::excessive_precision)]

use crate::spaces::Xyz65;
use crate::traits::ColorSpace;

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

#[inline]
fn srgb_to_linear(c: f64) -> f64 {
    let abs = c.abs();
    if abs <= 0.04045 {
        c / 12.92
    } else {
        let sign = if c < 0.0 { -1.0 } else { 1.0 };
        sign * ((abs + 0.055) / 1.055).powf(2.4)
    }
}

#[inline]
fn linear_to_srgb(c: f64) -> f64 {
    let abs = c.abs();
    if abs > 0.0031308 {
        let sign = if c < 0.0 { -1.0 } else { 1.0 };
        sign * (1.055 * abs.powf(1.0 / 2.4) - 0.055)
    } else {
        c * 12.92
    }
}

impl ColorSpace for Rgb {
    const MODE: &'static str = "rgb";
    const CHANNELS: &'static [&'static str] = &["r", "g", "b", "alpha"];

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
        Xyz65 {
            x: 0.4123907992659593 * r + 0.357584339383878 * g + 0.1804807884018343 * b,
            y: 0.2126390058715102 * r + 0.715168678767756 * g + 0.0721923153607337 * b,
            z: 0.0193308187155918 * r + 0.119194779794626 * g + 0.9505321522496607 * b,
            alpha: self.alpha,
        }
    }

    fn from_xyz65(xyz: Xyz65) -> Self {
        let r =
            xyz.x * 3.2409699419045226 - xyz.y * 1.5373831775700939 - 0.4986107602930034 * xyz.z;
        let g =
            xyz.x * -0.9692436362808796 + xyz.y * 1.8759675015077204 + 0.0415550574071756 * xyz.z;
        let b =
            xyz.x * 0.0556300796969936 - xyz.y * 0.2039769588889765 + 1.0569715142428784 * xyz.z;
        Self {
            r: linear_to_srgb(r),
            g: linear_to_srgb(g),
            b: linear_to_srgb(b),
            alpha: xyz.alpha,
        }
    }
}
