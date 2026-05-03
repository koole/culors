//! Math helpers shared across color-space implementations.
//!
//! The sRGB transfer functions and the linear-sRGB ↔ XYZ D65 matrix are lifted
//! verbatim from culori 4.0.2 (`node_modules/culori/src/lrgb/` and
//! `node_modules/culori/src/xyz65/`). Numeric constants preserve a 1:1 trace to
//! the JS source.

#![allow(clippy::excessive_precision)]

#[allow(dead_code)]
#[inline]
pub(crate) fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

#[allow(dead_code)]
#[inline]
pub(crate) fn clamp(x: f64, lo: f64, hi: f64) -> f64 {
    x.max(lo).min(hi)
}

/// sRGB → linear-sRGB transfer (sign-preserving, matches culori).
#[inline]
pub(crate) fn srgb_to_linear(c: f64) -> f64 {
    let abs = c.abs();
    if abs <= 0.04045 {
        c / 12.92
    } else {
        let sign = if c < 0.0 { -1.0 } else { 1.0 };
        sign * ((abs + 0.055) / 1.055).powf(2.4)
    }
}

/// linear-sRGB → sRGB transfer (sign-preserving, matches culori).
#[inline]
pub(crate) fn linear_to_srgb(c: f64) -> f64 {
    let abs = c.abs();
    if abs > 0.0031308 {
        let sign = if c < 0.0 { -1.0 } else { 1.0 };
        sign * (1.055 * abs.powf(1.0 / 2.4) - 0.055)
    } else {
        c * 12.92
    }
}

/// linear-sRGB → XYZ D65 matrix multiplication.
#[inline]
pub(crate) fn lrgb_to_xyz65(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    let x = 0.4123907992659593 * r + 0.357584339383878 * g + 0.1804807884018343 * b;
    let y = 0.2126390058715102 * r + 0.715168678767756 * g + 0.0721923153607337 * b;
    let z = 0.0193308187155918 * r + 0.119194779794626 * g + 0.9505321522496607 * b;
    (x, y, z)
}

/// XYZ D65 → linear-sRGB matrix multiplication.
#[inline]
pub(crate) fn xyz65_to_lrgb(x: f64, y: f64, z: f64) -> (f64, f64, f64) {
    let r = x * 3.2409699419045226 - y * 1.5373831775700939 - 0.4986107602930034 * z;
    let g = x * -0.9692436362808796 + y * 1.8759675015077204 + 0.0415550574071756 * z;
    let b = x * 0.0556300796969936 - y * 0.2039769588889765 + 1.0569715142428784 * z;
    (r, g, b)
}
