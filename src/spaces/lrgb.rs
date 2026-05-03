//! Linear-sRGB color space (gamma-decoded sRGB).
//!
//! Channels are in the nominal 0..1 range, but extended-range values are
//! preserved. The space shares its primaries (and therefore its XYZ D65
//! matrix) with sRGB; only the transfer function differs. Conversions track
//! culori 4.0.2 (`node_modules/culori/src/lrgb/`).

use crate::spaces::Xyz65;
use crate::traits::ColorSpace;
use crate::util::{lrgb_to_xyz65, xyz65_to_lrgb};

/// Linear-sRGB color. The transfer function from sRGB has been undone, so
/// channels are linear in light intensity. Same primaries and white point as
/// sRGB.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LinearRgb {
    /// Red channel (linear light).
    pub r: f64,
    /// Green channel (linear light).
    pub g: f64,
    /// Blue channel (linear light).
    pub b: f64,
    /// Optional alpha in 0..1.
    pub alpha: Option<f64>,
}

impl ColorSpace for LinearRgb {
    const MODE: &'static str = "lrgb";
    const CHANNELS: &'static [&'static str] = &["r", "g", "b"];

    fn alpha(&self) -> Option<f64> {
        self.alpha
    }

    fn with_alpha(self, alpha: Option<f64>) -> Self {
        Self { alpha, ..self }
    }

    fn to_xyz65(&self) -> Xyz65 {
        let (x, y, z) = lrgb_to_xyz65(self.r, self.g, self.b);
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
            r,
            g,
            b,
            alpha: xyz.alpha,
        }
    }
}
