//! Prismatic — a 4-channel decomposition of sRGB into intensity plus a
//! barycentric chromaticity simplex.
//!
//! # Definition
//!
//! Prismatic separates intensity (the maximum sRGB component) from a
//! barycentric chromaticity formed by normalising the same RGB triple to
//! sum to one. Forward transform from sRGB:
//!
//! ```text
//! l = max(R, G, B)
//! s = R + G + B
//! if s > 0: (r, g, b) = (R / s, G / s, B / s)
//! else:     (r, g, b) = (0, 0, 0)
//! ```
//!
//! The inverse multiplies the chromaticity by `l / max(r, g, b)` to
//! recover the original RGB:
//!
//! ```text
//! m = max(r, g, b)
//! if m > 0: (R, G, B) = (l * r / m, l * g / m, l * b / m)
//! else:     (R, G, B) = (0, 0, 0)
//! ```
//!
//! The chromaticity satisfies `r + g + b = 1` for any non-black input,
//! which makes it useful for hue/saturation work where the intensity
//! variable should be orthogonal to the colour direction.
//!
//! # Provenance and divergence
//!
//! Several published "prismatic" decompositions exist; this file picks
//! the formulation Hauke describes in *"The Prismatic Color Space"*
//! (2009): intensity = max channel, chromatic part = normalised triple.
//! Other authors use sum-normalisation paired with intensity = sum/3
//! instead. Pick this one explicitly so behaviour is well-defined.
//!
//! culori 4.0.2 has no Prismatic mode; this is a culor extension. CSS
//! has no registered identifier for it either, so the formatter and
//! parser treat the space using the non-standard `color(--prismatic
//! ...)` keyword.

use crate::spaces::{Rgb, Xyz65};
use crate::traits::ColorSpace;

/// Prismatic colour: intensity `l` and a barycentric `(r, g, b)`
/// chromaticity that sums to 1 for non-black inputs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Prismatic {
    /// Intensity — the maximum of the source sRGB triple.
    pub l: f64,
    /// Normalised red component of the chromaticity simplex.
    pub r: f64,
    /// Normalised green component.
    pub g: f64,
    /// Normalised blue component.
    pub b: f64,
    /// Optional alpha in 0..1.
    pub alpha: Option<f64>,
}

impl ColorSpace for Prismatic {
    const MODE: &'static str = "prismatic";
    const CHANNELS: &'static [&'static str] = &["l", "r", "g", "b"];

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

impl From<Rgb> for Prismatic {
    fn from(c: Rgb) -> Self {
        let s = c.r + c.g + c.b;
        let l = c.r.max(c.g).max(c.b);
        if s > 0.0 {
            Self {
                l,
                r: c.r / s,
                g: c.g / s,
                b: c.b / s,
                alpha: c.alpha,
            }
        } else {
            Self {
                l,
                r: 0.0,
                g: 0.0,
                b: 0.0,
                alpha: c.alpha,
            }
        }
    }
}

impl From<Prismatic> for Rgb {
    fn from(c: Prismatic) -> Self {
        let m = c.r.max(c.g).max(c.b);
        if m > 0.0 {
            let scale = c.l / m;
            Rgb {
                r: c.r * scale,
                g: c.g * scale,
                b: c.b * scale,
                alpha: c.alpha,
            }
        } else {
            Rgb {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                alpha: c.alpha,
            }
        }
    }
}
