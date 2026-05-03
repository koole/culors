//! Evenly spaced ramp samples.
//!
//! Mirrors culori 4.0.2's `samples.js`:
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
//! [`samples`] keeps the linear (γ = 1) shorthand for backwards compatibility.
//! [`samples_with_easing`] takes any `Fn(f64) -> f64`, letting callers plug in
//! [`crate::easing_gamma`], smoothstep, or a custom curve.

/// Returns `n` evenly spaced values in `[0, 1]` (linear, γ = 1).
///
/// Edge cases (matching culori exactly):
/// - `n == 0` returns an empty vector.
/// - `n == 1` returns `[0.5]`.
/// - `n >= 2` returns `[0, 1/(n-1), 2/(n-1), …, 1]`.
///
/// Pair with [`crate::interpolate()`] to drive evenly spaced ramp generation:
///
/// ```rust
/// use culors::{interpolate, parse, samples};
/// let a = parse("oklch(70% 0.15 30deg)").unwrap();
/// let b = parse("oklch(70% 0.15 200deg)").unwrap();
/// let ramp = interpolate(&[a, b], "oklab");
/// let stops: Vec<_> = samples(11).into_iter().map(ramp).collect();
/// assert_eq!(stops.len(), 11);
/// ```
pub fn samples(n: usize) -> Vec<f64> {
    samples_with_easing(n, |t| t)
}

/// `n` evenly spaced ramp positions, each transformed by `easing`.
///
/// Equivalent to culori's `samples(n, γ)` when `easing` is
/// [`crate::easing_gamma(γ)`](crate::easing_gamma); the broader signature
/// accepts any easing curve including [`crate::easing_smoothstep`].
///
/// ```rust
/// use culors::{easing_gamma, samples_with_easing};
/// // culori: samples(5, 2.2)
/// let v = samples_with_easing(5, easing_gamma(2.2));
/// assert!((v[2] - 0.217637640824031).abs() < 1e-12);
/// ```
pub fn samples_with_easing<F: Fn(f64) -> f64>(n: usize, easing: F) -> Vec<f64> {
    if n < 2 {
        return if n == 0 { Vec::new() } else { vec![easing(0.5)] };
    }
    let denom = (n - 1) as f64;
    (0..n).map(|i| easing(i as f64 / denom)).collect()
}
