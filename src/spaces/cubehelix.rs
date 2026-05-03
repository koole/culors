//! Cubehelix — Dave Green's astronomical color scheme as a color space.
//!
//! Constants and formulas verbatim from culori 4.0.2
//! (`node_modules/culori/src/cubehelix/`). Cubehelix is parameterized in
//! HSL-like (h, s, l), where the helix forward conversion is purely
//! algebraic on linear sRGB. culori does not normalize the hue: the value
//! returned by `atan2` minus 120 degrees may be negative, and the matching
//! inverse adds 120 degrees and feeds back into trig — both signs round-trip
//! correctly. We mirror that.
//!
//! When `l == 0` or `l == 1` the saturation is undefined; when saturation is
//! zero the hue is undefined. We encode undefined channels as `f64::NAN`,
//! matching culori's `undefined` sentinel.

#![allow(clippy::excessive_precision)]

use crate::spaces::{Rgb, Xyz65};
use crate::traits::ColorSpace;

const M0: f64 = -0.14861;
const M1: f64 = 1.78277;
const M2: f64 = -0.29227;
const M3: f64 = -0.90649;
const M4: f64 = 1.97294;
// M5 is 0 in culori; the green/blue channel formulas embed it as a
// multiplied constant, so we never need it as a runtime value.

const RAD_TO_DEG: f64 = 180.0 / std::f64::consts::PI;
const DEG_TO_RAD: f64 = std::f64::consts::PI / 180.0;

const DE: f64 = M3 * M4;
const BE: f64 = M1 * M4;
const BCAD: f64 = M1 * M2 - M0 * M3;

/// Cubehelix color. `h` is unnormalized (matches culori); `s` is in
/// `[0, 4.614]` for in-gamut sRGB; `l` is in `[0, 1]`. Either channel may
/// be `f64::NAN` to indicate culori's `undefined`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cubehelix {
    /// Hue in degrees. Not normalized.
    pub h: f64,
    /// Saturation.
    pub s: f64,
    /// Lightness in 0..1.
    pub l: f64,
    /// Optional alpha in 0..1.
    pub alpha: Option<f64>,
}

impl ColorSpace for Cubehelix {
    const MODE: &'static str = "cubehelix";
    const CHANNELS: &'static [&'static str] = &["h", "s", "l"];

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

impl From<Rgb> for Cubehelix {
    fn from(c: Rgb) -> Self {
        let Rgb { r, g, b, alpha } = c;
        let l = (BCAD * b + r * DE - g * BE) / (BCAD + DE - BE);
        let x = b - l;
        let y = (M4 * (g - l) - M2 * x) / M3;

        let s = if l == 0.0 || l == 1.0 {
            f64::NAN
        } else {
            (x * x + y * y).sqrt() / (M4 * l * (1.0 - l))
        };

        let h = if s.is_nan() || s == 0.0 {
            f64::NAN
        } else {
            y.atan2(x) * RAD_TO_DEG - 120.0
        };

        Self { h, s, l, alpha }
    }
}

impl From<Cubehelix> for Rgb {
    fn from(c: Cubehelix) -> Self {
        let Cubehelix { h, s, l, alpha } = c;
        let h_eff = if h.is_nan() { 0.0 } else { h };
        let l_eff = if l.is_nan() { 0.0 } else { l };
        let s_eff = if s.is_nan() { 0.0 } else { s };

        let h_rad = (h_eff + 120.0) * DEG_TO_RAD;
        let amp = s_eff * l_eff * (1.0 - l_eff);
        let cosh = h_rad.cos();
        let sinh = h_rad.sin();

        Self {
            r: l_eff + amp * (M0 * cosh + M1 * sinh),
            g: l_eff + amp * (M2 * cosh + M3 * sinh),
            b: l_eff + amp * (M4 * cosh/* + 0.0 * sinh */),
            alpha,
        }
    }
}
