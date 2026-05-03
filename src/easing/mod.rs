//! Scalar easing curves.
//!
//! Each function mirrors a counterpart in culori 4.0.2's `src/easing/` and
//! `src/interpolate/` modules. Curve factories return a closure
//! `Fn(f64) -> f64`; constant-curve helpers return the same shape so the
//! call sites compose uniformly.
//!
//! ```rust
//! use culors::easing_smoothstep;
//! let s = easing_smoothstep();
//! assert!((s(0.5) - 0.5).abs() < 1e-12);
//! ```

/// Color-interpolation hint exponential. Matches culori's `midpoint(H)`:
///
/// ```js
/// const midpoint = (H = 0.5) => t =>
///     H <= 0 ? 1 : H >= 1 ? 0 : Math.pow(t, Math.log(0.5) / Math.log(H));
/// ```
///
/// `H` is the parameter value at which the curve passes through `0.5`. The
/// degenerate cases `H <= 0` and `H >= 1` collapse to the constants `1` and
/// `0` respectively, identical to culori.
pub fn easing_midpoint(midpoint: f64) -> impl Fn(f64) -> f64 {
    let exponent = if midpoint <= 0.0 || midpoint >= 1.0 {
        f64::NAN
    } else {
        (0.5f64).ln() / midpoint.ln()
    };
    let edge: Option<f64> = if midpoint <= 0.0 {
        Some(1.0)
    } else if midpoint >= 1.0 {
        Some(0.0)
    } else {
        None
    };
    move |t: f64| match edge {
        Some(v) => v,
        None => t.powf(exponent),
    }
}

/// Cubic smoothstep `t * t * (3 - 2 * t)`.
pub fn easing_smoothstep() -> impl Fn(f64) -> f64 {
    |t: f64| t * t * (3.0 - 2.0 * t)
}

/// Inverse of [`easing_smoothstep`]: `0.5 - sin(asin(1 - 2t) / 3)`.
pub fn easing_smoothstep_inverse() -> impl Fn(f64) -> f64 {
    |t: f64| 0.5 - ((1.0 - 2.0 * t).asin() / 3.0).sin()
}

/// Quintic smootherstep proposed by K. Perlin:
/// `t^3 * (t * (t * 6 - 15) + 10)`.
pub fn easing_smootherstep() -> impl Fn(f64) -> f64 {
    |t: f64| t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

/// Sinusoidal in-out easing: `(1 - cos(t * π)) / 2`.
pub fn easing_in_out_sine() -> impl Fn(f64) -> f64 {
    |t: f64| (1.0 - (t * std::f64::consts::PI).cos()) / 2.0
}

/// Power curve `t^γ`. With `γ == 1` returns the identity (matching culori,
/// which short-circuits the `Math.pow` call to avoid unneeded denorm work).
pub fn easing_gamma(gamma: f64) -> impl Fn(f64) -> f64 {
    move |t: f64| {
        if gamma == 1.0 {
            t
        } else {
            t.powf(gamma)
        }
    }
}
