//! Euclidean color-distance factories — the `differenceEuclidean` family
//! from culori 4.0.2 (`node_modules/culori/src/difference.js`).
//!
//! [`difference_euclidean`] produces the generic distance in any supported
//! mode, with optional per-channel weights. [`difference_ok`] and
//! [`difference_euclidean_xyz`] are short-hands that fix the mode to
//! `oklab` and `xyz65` respectively.

use crate::difference::extract::{extract, mode_shape, normalize_hue, HueDiffKind, ModeShape};
use crate::Color;

/// Per-channel weights, ordered to match `mode_shape(mode).channels`. Plus
/// an alpha weight at index 3 (always 0 in culori's defaults; we accept it
/// but ignore it as alpha is handled separately).
pub(crate) type Weights = [f64; 4];

/// `differenceEuclidean(mode, weights)` — generic Euclidean distance in
/// any supported mode, with optional per-channel weights.
///
/// Defaults match culori: `weights = [1, 1, 1, 0]`. For LCh-like modes
/// (`lch`, `oklch`) the hue channel uses the polar-distance operator
/// `differenceHueChroma`; for HSx modes it uses `differenceHueSaturation`;
/// for HWB it uses the signed-wrap `differenceHueNaive`. For non-cylindrical
/// modes every channel is plain numeric subtraction.
///
/// Per-channel deltas that come back as NaN (e.g. an undefined hue on an
/// achromatic LCh color) are treated as 0, matching culori's
/// `isNaN(delta) ? 0 : delta` guard.
///
/// # Panics
///
/// Panics if `mode` is not one of `rgb`, `lrgb`, `hsl`, `hsv`, `hwb`,
/// `lab`, `lab65`, `lch`, `oklab`, `oklch`, `xyz50`, `xyz65`.
pub fn difference_euclidean(mode: &str) -> impl Fn(&Color, &Color) -> f64 {
    difference_euclidean_with(mode, [1.0, 1.0, 1.0, 0.0])
}

/// Like [`difference_euclidean`] but with explicit channel weights.
/// `weights[0..3]` are the per-channel weights in the order returned by
/// `mode_shape(mode).channels`; `weights[3]` is reserved for alpha
/// (currently ignored, mirroring the alpha-zero default in culori).
pub fn difference_euclidean_with(mode: &str, weights: Weights) -> impl Fn(&Color, &Color) -> f64 {
    let mode = mode.to_string();
    let shape = mode_shape(&mode);
    move |std, smp| euclidean(*std, *smp, &mode, shape, weights)
}

/// `differenceOk()` — Euclidean distance in Oklab.
pub fn difference_ok() -> impl Fn(&Color, &Color) -> f64 {
    difference_euclidean("oklab")
}

/// `differenceEuclidean('xyz65')` — Euclidean distance in XYZ D65.
pub fn difference_euclidean_xyz() -> impl Fn(&Color, &Color) -> f64 {
    difference_euclidean("xyz65")
}

pub(crate) fn euclidean(
    std: Color,
    smp: Color,
    mode: &str,
    shape: ModeShape,
    weights: Weights,
) -> f64 {
    let s = extract(std, mode);
    let t = extract(smp, mode);
    let mut sum = 0.0;
    for (idx, ch) in shape.channels.iter().enumerate() {
        let raw_delta = if ch.is_hue {
            hue_delta(shape.hue_diff, &s, &t, idx)
        } else {
            s[idx] - t[idx]
        };
        let delta = if raw_delta.is_nan() { 0.0 } else { raw_delta };
        sum += weights[idx] * delta * delta;
    }
    sum.sqrt()
}

/// Apply the configured polar-distance operator on the hue channel. The
/// hue index is supplied because LCh-likes have hue at index 2 and HSx
/// have it at index 0, so the partner-channel index differs.
fn hue_delta(kind: Option<HueDiffKind>, s: &[f64; 3], t: &[f64; 3], hue_idx: usize) -> f64 {
    let kind = match kind {
        Some(k) => k,
        None => return s[hue_idx] - t[hue_idx],
    };
    let h_std = s[hue_idx];
    let h_smp = t[hue_idx];
    if h_std.is_nan() || h_smp.is_nan() {
        return 0.0;
    }
    match kind {
        HueDiffKind::Chroma => {
            // LCh-like: chroma is at index 1 (l, c, h).
            let c_std = s[1];
            let c_smp = t[1];
            // culori: `if (!std.c || !smp.c) return 0;`
            if c_std == 0.0 || c_smp == 0.0 {
                return 0.0;
            }
            polar_sin_term(h_std, h_smp) * 2.0 * (c_std * c_smp).sqrt()
        }
        HueDiffKind::Saturation => {
            // HSx-like: saturation is at index 1 (h, s, l|v).
            let s_std = s[1];
            let s_smp = t[1];
            if s_std == 0.0 || s_smp == 0.0 {
                return 0.0;
            }
            polar_sin_term(h_std, h_smp) * 2.0 * (s_std * s_smp).sqrt()
        }
        HueDiffKind::Naive => {
            // culori: signed wrap to (-180, 180].
            let std_h = normalize_hue(h_std);
            let smp_h = normalize_hue(h_smp);
            if (smp_h - std_h).abs() > 180.0 {
                std_h - (smp_h - 360.0 * (smp_h - std_h).signum())
            } else {
                smp_h - std_h
            }
        }
    }
}

#[inline]
fn polar_sin_term(h_std: f64, h_smp: f64) -> f64 {
    let std_h = normalize_hue(h_std);
    let smp_h = normalize_hue(h_smp);
    let theta_deg = (smp_h - std_h + 360.0) / 2.0;
    theta_deg.to_radians().sin()
}
