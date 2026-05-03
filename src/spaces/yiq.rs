//! NTSC Y'IQ color space.
//!
//! Constants and matrix verbatim from culori 4.0.2
//! (`node_modules/culori/src/yiq/`). Y'IQ operates on gamma-encoded
//! sRGB; the conversion is a single 3x3 matrix with no transfer
//! function.

#![allow(clippy::excessive_precision)]

use crate::spaces::{Rgb, Xyz65};
use crate::traits::ColorSpace;

/// NTSC Y'IQ color. `y` is in 0..1, `i`/`q` are signed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Yiq {
    /// Luma in 0..1.
    pub y: f64,
    /// In-phase chroma.
    pub i: f64,
    /// Quadrature chroma.
    pub q: f64,
    /// Optional alpha in 0..1.
    pub alpha: Option<f64>,
}

impl ColorSpace for Yiq {
    const MODE: &'static str = "yiq";
    const CHANNELS: &'static [&'static str] = &["y", "i", "q"];

    fn alpha(&self) -> Option<f64> {
        self.alpha
    }

    fn with_alpha(self, alpha: Option<f64>) -> Self {
        Self { alpha, ..self }
    }

    fn to_xyz65(&self) -> Xyz65 {
        Rgb::from(*self).to_xyz65()
    }

    fn from_xyz65(xyz: Xyz65) -> Self {
        Rgb::from_xyz65(xyz).into()
    }
}

impl From<Rgb> for Yiq {
    fn from(c: Rgb) -> Self {
        Self {
            y: 0.29889531 * c.r + 0.58662247 * c.g + 0.11448223 * c.b,
            i: 0.59597799 * c.r - 0.2741761 * c.g - 0.32180189 * c.b,
            q: 0.21147017 * c.r - 0.52261711 * c.g + 0.31114694 * c.b,
            alpha: c.alpha,
        }
    }
}

impl From<Yiq> for Rgb {
    fn from(c: Yiq) -> Self {
        Self {
            r: c.y + 0.95608445 * c.i + 0.6208885 * c.q,
            g: c.y - 0.27137664 * c.i - 0.6486059 * c.q,
            b: c.y - 1.10561724 * c.i + 1.70250126 * c.q,
            alpha: c.alpha,
        }
    }
}
