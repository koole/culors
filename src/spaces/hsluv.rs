//! HSLuv — perceptually uniform HSL.
//!
//! Constants and algorithm verbatim from the official HSLuv reference
//! implementation (<https://github.com/hsluv/hsluv-javascript> v1.0.1,
//! `hsluv.cjs`). HSLuv is a hue-preserving cylinder over CIELuv,
//! parameterized so that the saturation reaches `100` at the sRGB gamut
//! boundary for any (`h`, `l`).
//!
//! The HSLuv algorithm uses its own sRGB ↔ linear-sRGB ↔ XYZ matrix.
//! Keeping these constants verbatim is necessary to reproduce
//! `hsluv-javascript`'s output bit-for-bit.

#![allow(clippy::excessive_precision)]

use crate::spaces::{Rgb, Xyz65};
use crate::traits::ColorSpace;

const REF_Y: f64 = 1.0;
const REF_U: f64 = 0.19783000664283;
const REF_V: f64 = 0.46831999493879;
const KAPPA: f64 = 903.2962962;
const EPSILON: f64 = 0.0088564516;

const M_R0: f64 = 3.240969941904521;
const M_R1: f64 = -1.537383177570093;
const M_R2: f64 = -0.498610760293;
const M_G0: f64 = -0.96924363628087;
const M_G1: f64 = 1.87596750150772;
const M_G2: f64 = 0.041555057407175;
const M_B0: f64 = 0.055630079696993;
const M_B1: f64 = -0.20397695888897;
const M_B2: f64 = 1.056971514242878;

/// HSLuv color. Hue in 0..360, saturation in 0..100, lightness in 0..100.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hsluv {
    /// Hue in degrees (0..360).
    pub h: f64,
    /// Saturation in 0..100.
    pub s: f64,
    /// Lightness in 0..100.
    pub l: f64,
    /// Optional alpha in 0..1.
    pub alpha: Option<f64>,
}

impl ColorSpace for Hsluv {
    const MODE: &'static str = "hsluv";
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

impl From<Rgb> for Hsluv {
    fn from(c: Rgb) -> Self {
        let (x, y, z) = rgb_to_xyz_hsluv(c.r, c.g, c.b);
        let (l, u, v) = xyz_to_luv(x, y, z);
        let (lch_l, lch_c, lch_h) = luv_to_lch(l, u, v);
        let (h, s, l) = lch_to_hsluv(lch_l, lch_c, lch_h);
        Self {
            h,
            s,
            l,
            alpha: c.alpha,
        }
    }
}

impl From<Hsluv> for Rgb {
    fn from(c: Hsluv) -> Self {
        let (lch_l, lch_c, lch_h) = hsluv_to_lch(c.h, c.s, c.l);
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

// --- Helpers (verbatim port of hsluv.cjs) ----------------------------------

#[inline]
fn from_linear(c: f64) -> f64 {
    if c <= 0.0031308 {
        12.92 * c
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

#[inline]
fn to_linear(c: f64) -> f64 {
    if c > 0.04045 {
        ((c + 0.055) / 1.055).powf(2.4)
    } else {
        c / 12.92
    }
}

#[inline]
fn y_to_l(y: f64) -> f64 {
    if y <= EPSILON {
        y / REF_Y * KAPPA
    } else {
        116.0 * (y / REF_Y).powf(1.0 / 3.0) - 16.0
    }
}

#[inline]
fn l_to_y(l: f64) -> f64 {
    if l <= 8.0 {
        REF_Y * l / KAPPA
    } else {
        REF_Y * ((l + 16.0) / 116.0).powi(3)
    }
}

pub(crate) fn rgb_to_xyz_hsluv(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    let lr = to_linear(r);
    let lg = to_linear(g);
    let lb = to_linear(b);
    let x = 0.41239079926595 * lr + 0.35758433938387 * lg + 0.18048078840183 * lb;
    let y = 0.21263900587151 * lr + 0.71516867876775 * lg + 0.072192315360733 * lb;
    let z = 0.019330818715591 * lr + 0.11919477979462 * lg + 0.95053215224966 * lb;
    (x, y, z)
}

pub(crate) fn xyz_to_rgb_hsluv(x: f64, y: f64, z: f64) -> (f64, f64, f64) {
    (
        from_linear(M_R0 * x + M_R1 * y + M_R2 * z),
        from_linear(M_G0 * x + M_G1 * y + M_G2 * z),
        from_linear(M_B0 * x + M_B1 * y + M_B2 * z),
    )
}

pub(crate) fn xyz_to_luv(x: f64, y: f64, z: f64) -> (f64, f64, f64) {
    let divider = x + 15.0 * y + 3.0 * z;
    let (var_u, var_v) = if divider != 0.0 {
        (4.0 * x / divider, 9.0 * y / divider)
    } else {
        (f64::NAN, f64::NAN)
    };
    let l = y_to_l(y);
    if l == 0.0 {
        (l, 0.0, 0.0)
    } else {
        (l, 13.0 * l * (var_u - REF_U), 13.0 * l * (var_v - REF_V))
    }
}

pub(crate) fn luv_to_xyz(l: f64, u: f64, v: f64) -> (f64, f64, f64) {
    if l == 0.0 {
        return (0.0, 0.0, 0.0);
    }
    let var_u = u / (13.0 * l) + REF_U;
    let var_v = v / (13.0 * l) + REF_V;
    let y = l_to_y(l);
    let x = 0.0 - 9.0 * y * var_u / ((var_u - 4.0) * var_v - var_u * var_v);
    let z = (9.0 * y - 15.0 * var_v * y - var_v * x) / (3.0 * var_v);
    (x, y, z)
}

pub(crate) fn luv_to_lch(l: f64, u: f64, v: f64) -> (f64, f64, f64) {
    let c = (u * u + v * v).sqrt();
    let h = if c < 0.00000001 {
        0.0
    } else {
        let hrad = v.atan2(u);
        let mut h = hrad * 180.0 / std::f64::consts::PI;
        if h < 0.0 {
            h += 360.0;
        }
        h
    };
    (l, c, h)
}

pub(crate) fn lch_to_luv(l: f64, c: f64, h: f64) -> (f64, f64, f64) {
    let hrad = h / 180.0 * std::f64::consts::PI;
    (l, hrad.cos() * c, hrad.sin() * c)
}

#[derive(Clone, Copy)]
pub(crate) struct BoundingLines {
    pub r0s: f64,
    pub r0i: f64,
    pub r1s: f64,
    pub r1i: f64,
    pub g0s: f64,
    pub g0i: f64,
    pub g1s: f64,
    pub g1i: f64,
    pub b0s: f64,
    pub b0i: f64,
    pub b1s: f64,
    pub b1i: f64,
}

pub(crate) fn calculate_bounding_lines(l: f64) -> BoundingLines {
    let sub1 = (l + 16.0).powi(3) / 1560896.0;
    let sub2 = if sub1 > EPSILON { sub1 } else { l / KAPPA };

    let s1r = sub2 * (284517.0 * M_R0 - 94839.0 * M_R2);
    let s2r = sub2 * (838422.0 * M_R2 + 769860.0 * M_R1 + 731718.0 * M_R0);
    let s3r = sub2 * (632260.0 * M_R2 - 126452.0 * M_R1);

    let s1g = sub2 * (284517.0 * M_G0 - 94839.0 * M_G2);
    let s2g = sub2 * (838422.0 * M_G2 + 769860.0 * M_G1 + 731718.0 * M_G0);
    let s3g = sub2 * (632260.0 * M_G2 - 126452.0 * M_G1);

    let s1b = sub2 * (284517.0 * M_B0 - 94839.0 * M_B2);
    let s2b = sub2 * (838422.0 * M_B2 + 769860.0 * M_B1 + 731718.0 * M_B0);
    let s3b = sub2 * (632260.0 * M_B2 - 126452.0 * M_B1);

    BoundingLines {
        r0s: s1r / s3r,
        r0i: s2r * l / s3r,
        r1s: s1r / (s3r + 126452.0),
        r1i: (s2r - 769860.0) * l / (s3r + 126452.0),
        g0s: s1g / s3g,
        g0i: s2g * l / s3g,
        g1s: s1g / (s3g + 126452.0),
        g1i: (s2g - 769860.0) * l / (s3g + 126452.0),
        b0s: s1b / s3b,
        b0i: s2b * l / s3b,
        b1s: s1b / (s3b + 126452.0),
        b1i: (s2b - 769860.0) * l / (s3b + 126452.0),
    }
}

#[inline]
fn distance_from_origin(slope: f64, intercept: f64) -> f64 {
    intercept.abs() / (slope.powi(2) + 1.0).sqrt()
}

#[inline]
fn distance_from_origin_angle(slope: f64, intercept: f64, angle: f64) -> f64 {
    let d = intercept / (angle.sin() - slope * angle.cos());
    if d < 0.0 {
        f64::INFINITY
    } else {
        d
    }
}

#[inline]
fn min6(a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) -> f64 {
    a.min(b).min(c).min(d).min(e).min(f)
}

pub(crate) fn calc_max_chroma_hpluv(bl: &BoundingLines) -> f64 {
    min6(
        distance_from_origin(bl.r0s, bl.r0i),
        distance_from_origin(bl.r1s, bl.r1i),
        distance_from_origin(bl.g0s, bl.g0i),
        distance_from_origin(bl.g1s, bl.g1i),
        distance_from_origin(bl.b0s, bl.b0i),
        distance_from_origin(bl.b1s, bl.b1i),
    )
}

pub(crate) fn calc_max_chroma_hsluv(bl: &BoundingLines, h: f64) -> f64 {
    let hue_rad = h / 360.0 * std::f64::consts::PI * 2.0;
    min6(
        distance_from_origin_angle(bl.r0s, bl.r0i, hue_rad),
        distance_from_origin_angle(bl.r1s, bl.r1i, hue_rad),
        distance_from_origin_angle(bl.g0s, bl.g0i, hue_rad),
        distance_from_origin_angle(bl.g1s, bl.g1i, hue_rad),
        distance_from_origin_angle(bl.b0s, bl.b0i, hue_rad),
        distance_from_origin_angle(bl.b1s, bl.b1i, hue_rad),
    )
}

fn hsluv_to_lch(h: f64, s: f64, l: f64) -> (f64, f64, f64) {
    let (lch_l, lch_c) = if l > 99.9999999 {
        (100.0, 0.0)
    } else if l < 0.00000001 {
        (0.0, 0.0)
    } else {
        let bl = calculate_bounding_lines(l);
        let max = calc_max_chroma_hsluv(&bl, h);
        (l, max / 100.0 * s)
    };
    (lch_l, lch_c, h)
}

fn lch_to_hsluv(l: f64, c: f64, h: f64) -> (f64, f64, f64) {
    let (s, l_out) = if l > 99.9999999 {
        (0.0, 100.0)
    } else if l < 0.00000001 {
        (0.0, 0.0)
    } else {
        let bl = calculate_bounding_lines(l);
        let max = calc_max_chroma_hsluv(&bl, h);
        (c / max * 100.0, l)
    };
    (h, s, l_out)
}
