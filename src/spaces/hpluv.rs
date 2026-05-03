//! HPLuv — perceptually uniform HSL, pastel-only.
//!
//! HPLuv is HSLuv's sibling: instead of stretching saturation to the
//! full sRGB gamut boundary, it stretches to the *largest circle*
//! inscribed in the gamut shape at the given lightness. Saturation can
//! exceed 100 for vivid hues that fall outside that inscribed circle.
//!
//! Constants and algorithm verbatim from the official HSLuv JS
//! reference (`hsluv-javascript` v1.0.1, `hsluv.cjs`). The shared
//! Luv / LCh / bounding-line helpers live in [`crate::spaces::hsluv`].

use crate::spaces::hsluv::{
    calc_max_chroma_hpluv, calculate_bounding_lines, lch_to_luv, luv_to_lch, luv_to_xyz,
    rgb_to_xyz_hsluv, xyz_to_luv, xyz_to_rgb_hsluv,
};
use crate::spaces::{Rgb, Xyz65};
use crate::traits::ColorSpace;

/// HPLuv color. Hue in 0..360, saturation may exceed 100 for vivid
/// hues, lightness in 0..100.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hpluv {
    /// Hue in degrees.
    pub h: f64,
    /// Saturation (≥ 0; may exceed 100 outside the inscribed circle).
    pub s: f64,
    /// Lightness in 0..100.
    pub l: f64,
    /// Optional alpha in 0..1.
    pub alpha: Option<f64>,
}

impl ColorSpace for Hpluv {
    const MODE: &'static str = "hpluv";
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

impl From<Rgb> for Hpluv {
    fn from(c: Rgb) -> Self {
        let (x, y, z) = rgb_to_xyz_hsluv(c.r, c.g, c.b);
        let (l, u, v) = xyz_to_luv(x, y, z);
        let (lch_l, lch_c, lch_h) = luv_to_lch(l, u, v);
        let (h, s, l) = lch_to_hpluv(lch_l, lch_c, lch_h);
        Self {
            h,
            s,
            l,
            alpha: c.alpha,
        }
    }
}

impl From<Hpluv> for Rgb {
    fn from(c: Hpluv) -> Self {
        let (lch_l, lch_c, lch_h) = hpluv_to_lch(c.h, c.s, c.l);
        let (l, u, v) = lch_to_luv(lch_l, lch_c, lch_h);
        let (x, y, z) = luv_to_xyz(l, u, v);
        let (r, g, b) = xyz_to_rgb_hsluv(x, y, z);
        Self {
            r,
            g,
            b,
            alpha: c.alpha,
        }
    }
}

fn hpluv_to_lch(h: f64, s: f64, l: f64) -> (f64, f64, f64) {
    let (lch_l, lch_c) = if l > 99.9999999 {
        (100.0, 0.0)
    } else if l < 0.00000001 {
        (0.0, 0.0)
    } else {
        let bl = calculate_bounding_lines(l);
        let max = calc_max_chroma_hpluv(&bl);
        (l, max / 100.0 * s)
    };
    (lch_l, lch_c, h)
}

fn lch_to_hpluv(l: f64, c: f64, h: f64) -> (f64, f64, f64) {
    let (s, l_out) = if l > 99.9999999 {
        (0.0, 100.0)
    } else if l < 0.00000001 {
        (0.0, 0.0)
    } else {
        let bl = calculate_bounding_lines(l);
        let max = calc_max_chroma_hpluv(&bl);
        (c / max * 100.0, l)
    };
    (h, s, l_out)
}
