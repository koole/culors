//! Higher-order channel-interpolator factory.
//!
//! Mirrors culori 4.0.2's `interpolatorPiecewise` (defined in
//! `node_modules/culori/src/interpolate/piecewise.js`). Given a per-segment
//! interpolator `f(a, b, t)`, returns a factory that builds a sampler over
//! a stop slice. The built sampler partitions `[0, 1]` evenly across the
//! segments, then calls `f` on the active segment with a local `t`.
//!
//! Missing stops (encoded as `NaN`) follow culori's `[a, a]` / `[b, b]`
//! propagation rule: a class with one missing endpoint repeats the present
//! endpoint, a class with both missing returns `NaN`.

use super::spline::{ChannelInterp, ChannelInterpFactory};

/// Build a [`ChannelInterpFactory`] from a per-segment interpolator
/// `f(a, b, t)`. The returned factory takes a stop slice and produces a
/// sampler that, given `t` in `[0, 1]`, picks the active segment and
/// returns `f(stops[i], stops[i+1], local_t)`.
///
/// Mirrors culori's `interpolatorPiecewise(interpolator)`: the partition
/// logic is identical (`idx = floor(t * (n - 1))` clamped to `[0, n - 2]`),
/// and the missing-endpoint rule (`[a, a]` if only `a` is defined,
/// `[b, b]` if only `b` is, `None` if neither) is preserved bit-for-bit.
///
/// Slot the resulting factory into
/// [`crate::InterpolateOptions::channel_interpolator`] to use a custom
/// per-channel interpolator, exactly as culori's
/// `interpolate(stops, mode, { interpolator })` does.
///
/// # Example
///
/// ```rust
/// use culors::interpolator_piecewise;
///
/// // Quadratic ease per segment.
/// let factory = interpolator_piecewise(|a: f64, b: f64, t: f64| {
///     a + (b - a) * t * t
/// });
/// let sampler = factory(&[0.0, 10.0]);
/// assert!((sampler(0.5) - 2.5).abs() < 1e-12);
/// ```
pub fn interpolator_piecewise<F>(channel_fn: F) -> ChannelInterpFactory
where
    F: Fn(f64, f64, f64) -> f64 + Send + Sync + Clone + 'static,
{
    Box::new(move |stops: &[f64]| -> ChannelInterp {
        let classes = build_classes(stops);
        let f = channel_fn.clone();
        Box::new(move |t: f64| {
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
                Some((a, b)) => f(a, b, local_t),
                None => f64::NAN,
            }
        })
    })
}

// Shared with `lerp.rs` — duplicated here to avoid a module-private
// dependency edge. The two are byte-for-byte identical because they both
// mirror culori's `get_classes`.
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
