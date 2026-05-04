//! Hue cycling strategies for cylindrical color spaces.
//!
//! Mirrors culori's `fixup/hue.js`. The fixup runs over a channel's stop
//! values before interpolation: each non-missing hue is reduced modulo 360,
//! a per-strategy delta function rewrites the gap between consecutive
//! defined stops, then the deltas are accumulated into absolute angles.
//! Missing hues (`NaN` here, `undefined` in culori) pass through and reset
//! the running accumulator on the next defined value.

/// Hue interpolation strategy. Defaults to `Shorter`, the CSS Color Module 4
/// rule for cylindrical spaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HueFixup {
    /// Take the shorter arc between two hues. CSS Color Module 4 default.
    #[default]
    Shorter,
    /// Take the longer arc.
    Longer,
    /// Always rotate counter-clockwise (positive hue direction).
    Increasing,
    /// Always rotate clockwise (negative hue direction).
    Decreasing,
    /// No fixup. Hues are interpolated linearly without normalization.
    Raw,
}

#[inline]
fn normalize_hue(h: f64) -> f64 {
    let m = h % 360.0;
    if m < 0.0 {
        m + 360.0
    } else {
        m
    }
}

fn apply_with<F: Fn(f64) -> f64>(hues: &[f64], delta_fn: F) -> Vec<f64> {
    let mut deltas: Vec<f64> = Vec::with_capacity(hues.len());
    for (idx, &h) in hues.iter().enumerate() {
        if h.is_nan() {
            deltas.push(f64::NAN);
            continue;
        }
        let normalized = normalize_hue(h);
        if idx == 0 || hues[idx - 1].is_nan() {
            deltas.push(normalized);
        } else {
            let prev = normalize_hue(hues[idx - 1]);
            deltas.push(delta_fn(normalized - prev));
        }
    }

    // Reduce: accumulate deltas into absolute hues. NaN resets the chain;
    // the next defined value is treated as a fresh anchor by the next pass.
    let mut acc: Vec<f64> = Vec::with_capacity(deltas.len());
    for d in deltas {
        if acc.is_empty() || d.is_nan() || acc.last().is_some_and(|v| v.is_nan()) {
            acc.push(d);
        } else {
            let prev = *acc.last().expect("non-empty");
            acc.push(d + prev);
        }
    }
    acc
}

pub(crate) fn apply(hues: &[f64], strategy: HueFixup) -> Vec<f64> {
    match strategy {
        HueFixup::Shorter => fixup_hue_shorter(hues),
        HueFixup::Longer => fixup_hue_longer(hues),
        HueFixup::Increasing => fixup_hue_increasing(hues),
        HueFixup::Decreasing => fixup_hue_decreasing(hues),
        HueFixup::Raw => hues.to_vec(),
    }
}

/// Take the shorter arc between consecutive hues. CSS Color Module 4 default.
///
/// Mirrors culori 4.0.2's `fixupHueShorter` (`fixup/hue.js`). Each non-`NaN`
/// hue is reduced to `[0, 360)`, the delta to the previous defined hue is
/// rewritten to lie within `[-180, 180]`, then deltas are accumulated into
/// absolute angles. `NaN` (culori `undefined`) passes through and resets the
/// running anchor on the next defined value.
///
/// ```rust
/// use culors::fixup_hue_shorter;
/// assert_eq!(fixup_hue_shorter(&[0.0, 340.0, 30.0, 0.0, 170.0]),
///            vec![0.0, -20.0, 30.0, 0.0, 170.0]);
/// ```
pub fn fixup_hue_shorter(hues: &[f64]) -> Vec<f64> {
    apply_with(hues, |d| {
        if d.abs() <= 180.0 {
            d
        } else {
            d - 360.0 * d.signum()
        }
    })
}

/// Take the longer arc between consecutive hues.
///
/// Mirrors culori 4.0.2's `fixupHueLonger` (`fixup/hue.js`). The delta is
/// rewritten so that arcs strictly shorter than 180° flip to their long-way
/// counterpart; equal-magnitude arcs (exactly 180°) and zero arcs pass
/// through unchanged.
///
/// ```rust
/// use culors::fixup_hue_longer;
/// assert_eq!(fixup_hue_longer(&[0.0, 340.0, 30.0, 0.0, 170.0]),
///            vec![0.0, 340.0, 30.0, 360.0, 170.0]);
/// ```
pub fn fixup_hue_longer(hues: &[f64]) -> Vec<f64> {
    apply_with(hues, |d| {
        if d.abs() >= 180.0 || d == 0.0 {
            d
        } else {
            d - 360.0 * d.signum()
        }
    })
}

/// Always rotate counter-clockwise (positive hue direction).
///
/// Mirrors culori 4.0.2's `fixupHueIncreasing` (`fixup/hue.js`). Negative
/// deltas are shifted by `+360°` so that every consecutive arc is
/// non-negative; the accumulated output is monotonically non-decreasing.
///
/// ```rust
/// use culors::fixup_hue_increasing;
/// assert_eq!(fixup_hue_increasing(&[0.0, 340.0, 30.0, 0.0, 170.0]),
///            vec![0.0, 340.0, 390.0, 720.0, 890.0]);
/// ```
pub fn fixup_hue_increasing(hues: &[f64]) -> Vec<f64> {
    apply_with(hues, |d| if d >= 0.0 { d } else { d + 360.0 })
}

/// Always rotate clockwise (negative hue direction).
///
/// Mirrors culori 4.0.2's `fixupHueDecreasing` (`fixup/hue.js`). Positive
/// deltas are shifted by `-360°` so that every consecutive arc is
/// non-positive; the accumulated output is monotonically non-increasing.
///
/// ```rust
/// use culors::fixup_hue_decreasing;
/// assert_eq!(fixup_hue_decreasing(&[0.0, 340.0, 30.0, 0.0, 170.0]),
///            vec![0.0, -20.0, -330.0, -360.0, -550.0]);
/// ```
pub fn fixup_hue_decreasing(hues: &[f64]) -> Vec<f64> {
    apply_with(hues, |d| if d <= 0.0 { d } else { d - 360.0 })
}

/// Fix up alpha stops the way culori does: if any alpha is set, missing
/// values become 1; if none are set, leave them missing. The interpolation
/// closure uses NaN as the "missing" marker.
///
/// Mirrors culori 4.0.2's `fixupAlpha`
/// (`node_modules/culori/src/fixup/alpha.js`):
///
/// ```text
/// fixupAlpha([NaN, 0, NaN])     == [1, 0, 1]
/// fixupAlpha([NaN, NaN, NaN])   == [NaN, NaN, NaN]   // unchanged
/// ```
///
/// Exposed publicly so callers building custom interpolation pipelines can
/// reuse the same alpha-handling rule the standard `interpolate` family
/// applies internally.
pub fn fixup_alpha(alphas: &[f64]) -> Vec<f64> {
    let any_defined = alphas.iter().any(|a| !a.is_nan());
    if !any_defined {
        return alphas.to_vec();
    }
    alphas
        .iter()
        .map(|&a| if a.is_nan() { 1.0 } else { a })
        .collect()
}
