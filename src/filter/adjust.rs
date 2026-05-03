//! Per-channel adjustments and 3x3 color-matrix filters.
//!
//! Each filter mirrors the corresponding entry in culori 4.0.2's
//! `src/filter.js`. Inputs are converted to sRGB, the matrix or transfer
//! function is applied, alpha is preserved, and a `Color::Rgb` is returned.
//! Channel values are not clipped, matching culori.

use crate::filter::common::to_rgb;
use crate::spaces::Rgb;
use crate::Color;

/// CSS-style brightness filter. `amount = 1` is identity, `0` produces
/// black, values above `1` extend beyond the gamut. Negative inputs are
/// clamped to zero, matching culori's `Math.max(amt, 0)`.
pub fn filter_brightness(amount: f64) -> impl Fn(&Color) -> Color {
    let a = amount.max(0.0);
    move |color: &Color| {
        let rgb = to_rgb(*color);
        Color::Rgb(Rgb {
            r: rgb.r * a,
            g: rgb.g * a,
            b: rgb.b * a,
            alpha: rgb.alpha,
        })
    }
}

/// CSS-style contrast filter. `amount = 1` is identity, `0` collapses to
/// 50% grey, larger values amplify around 0.5. Negative inputs are
/// clamped to zero.
pub fn filter_contrast(amount: f64) -> impl Fn(&Color) -> Color {
    let a = amount.max(0.0);
    let intercept = (1.0 - a) / 2.0;
    move |color: &Color| {
        let rgb = to_rgb(*color);
        Color::Rgb(Rgb {
            r: rgb.r * a + intercept,
            g: rgb.g * a + intercept,
            b: rgb.b * a + intercept,
            alpha: rgb.alpha,
        })
    }
}

/// CSS-style invert filter. `amount = 1` fully inverts each channel,
/// `0` is identity. Inputs are clamped to `[0, 1]`.
pub fn filter_invert(amount: f64) -> impl Fn(&Color) -> Color {
    let a = amount.clamp(0.0, 1.0);
    move |color: &Color| {
        let rgb = to_rgb(*color);
        let lerp = |v: f64| a + (1.0 - a - a) * v;
        Color::Rgb(Rgb {
            r: lerp(rgb.r),
            g: lerp(rgb.g),
            b: lerp(rgb.b),
            alpha: rgb.alpha,
        })
    }
}

#[inline]
fn apply_matrix(rgb: Rgb, m: &[f64; 9]) -> Rgb {
    Rgb {
        r: m[0] * rgb.r + m[1] * rgb.g + m[2] * rgb.b,
        g: m[3] * rgb.r + m[4] * rgb.g + m[5] * rgb.b,
        b: m[6] * rgb.r + m[7] * rgb.g + m[8] * rgb.b,
        alpha: rgb.alpha,
    }
}

fn matrix_sepia(amount: f64) -> [f64; 9] {
    let a = 1.0 - amount.clamp(0.0, 1.0);
    [
        0.393 + 0.607 * a,
        0.769 - 0.769 * a,
        0.189 - 0.189 * a,
        0.349 - 0.349 * a,
        0.686 + 0.314 * a,
        0.168 - 0.168 * a,
        0.272 - 0.272 * a,
        0.534 - 0.534 * a,
        0.131 + 0.869 * a,
    ]
}

fn matrix_saturate(amount: f64) -> [f64; 9] {
    let s = amount.max(0.0);
    [
        0.213 + 0.787 * s,
        0.715 - 0.715 * s,
        0.072 - 0.072 * s,
        0.213 - 0.213 * s,
        0.715 + 0.285 * s,
        0.072 - 0.072 * s,
        0.213 - 0.213 * s,
        0.715 - 0.715 * s,
        0.072 + 0.928 * s,
    ]
}

fn matrix_grayscale(amount: f64) -> [f64; 9] {
    let a = 1.0 - amount.clamp(0.0, 1.0);
    [
        0.2126 + 0.7874 * a,
        0.7152 - 0.7152 * a,
        0.0722 - 0.0722 * a,
        0.2126 - 0.2126 * a,
        0.7152 + 0.2848 * a,
        0.0722 - 0.0722 * a,
        0.2126 - 0.2126 * a,
        0.7152 - 0.7152 * a,
        0.0722 + 0.9278 * a,
    ]
}

fn matrix_hue_rotate(degrees: f64) -> [f64; 9] {
    let rad = std::f64::consts::PI * degrees / 180.0;
    let c = rad.cos();
    let s = rad.sin();
    [
        0.213 + c * 0.787 - s * 0.213,
        0.715 - c * 0.715 - s * 0.715,
        0.072 - c * 0.072 + s * 0.928,
        0.213 - c * 0.213 + s * 0.143,
        0.715 + c * 0.285 + s * 0.140,
        0.072 - c * 0.072 - s * 0.283,
        0.213 - c * 0.213 - s * 0.787,
        0.715 - c * 0.715 + s * 0.715,
        0.072 + c * 0.928 + s * 0.072,
    ]
}

/// CSS-style sepia filter. `amount = 1` is full sepia, `0` is identity.
/// Inputs are clamped to `[0, 1]`.
pub fn filter_sepia(amount: f64) -> impl Fn(&Color) -> Color {
    let m = matrix_sepia(amount);
    move |color: &Color| {
        let rgb = to_rgb(*color);
        Color::Rgb(apply_matrix(rgb, &m))
    }
}

/// CSS-style saturate filter. `amount = 1` is identity, `0` is fully
/// desaturated, larger values exaggerate saturation. Negative inputs are
/// clamped to zero.
pub fn filter_saturate(amount: f64) -> impl Fn(&Color) -> Color {
    let m = matrix_saturate(amount);
    move |color: &Color| {
        let rgb = to_rgb(*color);
        Color::Rgb(apply_matrix(rgb, &m))
    }
}

/// CSS-style grayscale filter. `amount = 1` is fully grey using the BT.709
/// luminance weights, `0` is identity. Inputs are clamped to `[0, 1]`.
pub fn filter_grayscale(amount: f64) -> impl Fn(&Color) -> Color {
    let m = matrix_grayscale(amount);
    move |color: &Color| {
        let rgb = to_rgb(*color);
        Color::Rgb(apply_matrix(rgb, &m))
    }
}

/// CSS-style hue-rotate filter, in degrees. `0` is identity. Uses the
/// SVG/CSS hue-rotate matrix (a luminance-preserving approximation that
/// can produce out-of-gamut channel values).
pub fn filter_hue_rotate(degrees: f64) -> impl Fn(&Color) -> Color {
    let m = matrix_hue_rotate(degrees);
    move |color: &Color| {
        let rgb = to_rgb(*color);
        Color::Rgb(apply_matrix(rgb, &m))
    }
}
