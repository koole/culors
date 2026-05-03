//! Per-channel linear interpolation.
//!
//! Mirrors culori's `interpolate/lerp.js` and `interpolate/piecewise.js`. A
//! channel's stop list may contain `NaN` (culori's `undefined`) for powerless
//! values; the piecewise builder turns each consecutive pair into one
//! "class". Pairs with both endpoints missing produce `NaN` outputs; pairs
//! with only one missing endpoint propagate the present endpoint's value to
//! the missing side, matching culori's `[a, a]` / `[b, b]` rule.

#[inline]
pub(crate) fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + t * (b - a)
}

/// Linear piecewise interpolator over a channel's stop values. The returned
/// closure takes `t` in `[0, 1]` (already clamped by the caller) and returns
/// the interpolated value, or `NaN` when both endpoints of the active segment
/// are missing.
pub(crate) fn linear_interpolator(stops: Vec<f64>) -> impl Fn(f64) -> f64 {
    let classes = build_classes(&stops);
    move |t: f64| {
        let n = classes.len();
        if n == 0 {
            return f64::NAN;
        }
        let cls = t * n as f64;
        let idx = if t >= 1.0 {
            n - 1
        } else {
            (cls.floor() as isize).max(0) as usize
        };
        let local_t = cls - idx as f64;
        match classes[idx] {
            Some((a, b)) => lerp(a, b, local_t),
            None => f64::NAN,
        }
    }
}

fn build_classes(arr: &[f64]) -> Vec<Option<(f64, f64)>> {
    if arr.len() < 2 {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(arr.len() - 1);
    for i in 0..arr.len() - 1 {
        let a = arr[i];
        let b = arr[i + 1];
        let a_def = !a.is_nan();
        let b_def = !b.is_nan();
        let pair = match (a_def, b_def) {
            (false, false) => None,
            (true, true) => Some((a, b)),
            (true, false) => Some((a, a)),
            (false, true) => Some((b, b)),
        };
        out.push(pair);
    }
    out
}
