//! Evenly spaced ramp samples.
//!
//! Mirrors culori 4.0.2's `samples.js` with the default linear gamma:
//!
//! ```js
//! const samples = (n = 2, γ = 1) => {
//!     let ease = gamma(γ);
//!     if (n < 2) {
//!         return n < 1 ? [] : [ease(0.5)];
//!     }
//!     let res = [];
//!     for (let i = 0; i < n; i++) {
//!         res.push(ease(i / (n - 1)));
//!     }
//!     return res;
//! };
//! ```
//!
//! With γ = 1 the easing is the identity, so the output is `[0]` and `[1]`
//! for the endpoints and `i / (n - 1)` for the interior.

/// Returns `n` evenly spaced values in `[0, 1]`.
///
/// Edge cases (matching culori exactly):
/// - `n == 0` returns an empty vector.
/// - `n == 1` returns `[0.5]`.
/// - `n >= 2` returns `[0, 1/(n-1), 2/(n-1), …, 1]`.
///
/// Pair with [`crate::interpolate()`] to drive evenly spaced ramp generation:
///
/// ```rust
/// use culor::{interpolate, parse, samples};
/// let a = parse("oklch(70% 0.15 30deg)").unwrap();
/// let b = parse("oklch(70% 0.15 200deg)").unwrap();
/// let ramp = interpolate(&[a, b], "oklab");
/// let stops: Vec<_> = samples(11).into_iter().map(ramp).collect();
/// assert_eq!(stops.len(), 11);
/// ```
pub fn samples(n: usize) -> Vec<f64> {
    if n < 2 {
        return if n == 0 { Vec::new() } else { vec![0.5] };
    }
    let denom = (n - 1) as f64;
    (0..n).map(|i| i as f64 / denom).collect()
}
