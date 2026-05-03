//! Rounding factory matching culori's `round.js`.
//!
//! ```js
//! const r = (value, precision) =>
//!     Math.round(value * (precision = Math.pow(10, precision))) / precision;
//!
//! const round =
//!     (precision = 4) =>
//!     value =>
//!         typeof value === 'number' ? r(value, precision) : value;
//! ```
//!
//! JavaScript's `Math.round` rounds half-toward-positive-infinity, not
//! Rust's half-away-from-zero. The two differ for exact halves of negative
//! numbers: `Math.round(-0.5) === -0`, whereas `(-0.5_f64).round() ==
//! -1.0`. This module reproduces the JS rule via `(x + 0.5).floor()`.

/// Returns a function that rounds an `f64` to the requested decimal
/// precision, matching culori's `round(precision)`. The default precision
/// in culori is `4`; callers pass the value explicitly here.
///
/// ```rust
/// let r = culor::round(2);
/// assert_eq!(r(0.123), 0.12);
/// assert_eq!(r(1.235), 1.24);
/// ```
pub fn round(places: u32) -> impl Fn(f64) -> f64 {
    let factor = 10f64.powi(places as i32);
    move |value| js_math_round(value * factor) / factor
}

#[inline]
fn js_math_round(x: f64) -> f64 {
    // ECMA-262: returns the Number value that is closest to x and is
    // an integer; if two integer Number values are equally close to x,
    // the result is the one closer to +∞.
    if x.is_nan() || x.is_infinite() {
        return x;
    }
    (x + 0.5).floor()
}
