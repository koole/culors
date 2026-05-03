//! Polar-hue distance helpers.
//!
//! culori exposes `differenceHueChroma` and `differenceHueSaturation` as
//! plain `(std, smp) => number` reducers that operate on the LCh-like or
//! HSx-like channel triple of one fixed mode. We expose factories that
//! take the mode by name and convert both inputs to that mode first.
//!
//! Both helpers read from `differenceEuclidean(mode, [0, 0, 1, 0])`-style
//! invocations in culori's tests, but the dedicated wrappers skip the
//! Euclidean-sum overhead and return the polar `dH` term directly.

use crate::difference::extract::{extract, normalize_hue};
use crate::Color;

/// LCh-style polar distance. `mode` must be `"lch"` or `"oklch"`.
///
/// The formula is `2 · √(c1 · c2) · sin(((h2 − h1 + 360) / 2)°)`, taken
/// verbatim from culori's `differenceHueChroma` in `difference.js`. The
/// result is **signed**: an ascending hue (`h2 > h1` modulo 360) makes
/// the inner sine negative, so the helper returns a negative number.
/// Returns 0 when either chroma is zero or either hue is NaN. To get
/// the magnitude only — what `differenceEuclidean(mode, [0, 0, 1, 0])`
/// produces — square the result before summing.
///
/// # Panics
///
/// Panics if `mode` is not an LCh-like mode.
pub fn difference_hue_chroma(mode: &str) -> impl Fn(&Color, &Color) -> f64 {
    assert!(
        matches!(mode, "lch" | "oklch"),
        "difference_hue_chroma: mode must be 'lch' or 'oklch', got '{mode}'"
    );
    let mode = mode.to_string();
    move |std, smp| {
        // (l, c, h) ordering.
        let s = extract(*std, &mode);
        let t = extract(*smp, &mode);
        polar_hue_distance(s[2], t[2], s[1], t[1])
    }
}

/// HSx-style polar distance. `mode` must be `"hsl"` or `"hsv"`.
///
/// The formula is `2 · √(s1 · s2) · sin(((h2 − h1 + 360) / 2)°)`, taken
/// verbatim from culori's `differenceHueSaturation`. Like
/// [`difference_hue_chroma`] the result is signed; ascending hue
/// returns a negative value. Returns 0 when either saturation is zero
/// or either hue is NaN.
///
/// # Panics
///
/// Panics if `mode` is not an HSx mode.
pub fn difference_hue_saturation(mode: &str) -> impl Fn(&Color, &Color) -> f64 {
    assert!(
        matches!(mode, "hsl" | "hsv"),
        "difference_hue_saturation: mode must be 'hsl' or 'hsv', got '{mode}'"
    );
    let mode = mode.to_string();
    move |std, smp| {
        // (h, s, _) ordering.
        let s = extract(*std, &mode);
        let t = extract(*smp, &mode);
        polar_hue_distance(s[0], t[0], s[1], t[1])
    }
}

/// Naive signed hue distance — culori's `differenceHueNaive`.
///
/// ```text
/// let dh = smp.h - std.h, both normalized to 0..360.
/// if |dh| > 180:  std.h - (smp.h - 360 * sign(dh))
/// else:           dh
/// ```
///
/// Returns the signed shortest angular distance (range `-180..180`).
/// Returns 0 if either hue is NaN. The caller specifies which mode
/// supplies the hue (any cylindrical mode: `lch`, `lch65`, `oklch`,
/// `hsl`, `hsv`, `hwb`).
///
/// # Panics
///
/// Panics if `mode` is not a known cylindrical mode.
pub fn difference_hue_naive(mode: &str) -> impl Fn(&Color, &Color) -> f64 {
    assert!(
        matches!(mode, "lch" | "lch65" | "oklch" | "hsl" | "hsv" | "hwb"),
        "difference_hue_naive: mode must be cylindrical (lch/lch65/oklch/hsl/hsv/hwb), got '{mode}'"
    );
    let mode_str = mode.to_string();
    let hue_idx: usize = if matches!(mode, "lch" | "lch65" | "oklch") {
        2
    } else {
        0
    };
    move |std, smp| {
        let s = extract(*std, &mode_str);
        let t = extract(*smp, &mode_str);
        let h_std = s[hue_idx];
        let h_smp = t[hue_idx];
        if h_std.is_nan() || h_smp.is_nan() {
            return 0.0;
        }
        let std_h = normalize_hue(h_std);
        let smp_h = normalize_hue(h_smp);
        if (smp_h - std_h).abs() > 180.0 {
            std_h - (smp_h - 360.0 * (smp_h - std_h).signum())
        } else {
            smp_h - std_h
        }
    }
}

fn polar_hue_distance(h_std: f64, h_smp: f64, mag_std: f64, mag_smp: f64) -> f64 {
    if h_std.is_nan() || h_smp.is_nan() || mag_std == 0.0 || mag_smp == 0.0 {
        return 0.0;
    }
    let std_h = normalize_hue(h_std);
    let smp_h = normalize_hue(h_smp);
    let theta_deg = (smp_h - std_h + 360.0) / 2.0;
    2.0 * (mag_std * mag_smp).sqrt() * theta_deg.to_radians().sin()
}
