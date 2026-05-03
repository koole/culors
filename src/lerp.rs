//! Scalar linear, bilinear, and trilinear interpolation utilities.
//!
//! Mirrors culori 4.0.2's `interpolate/lerp.js`:
//!
//! ```js
//! const lerp = (a, b, t) => a + t * (b - a);
//! const unlerp = (a, b, v) => (v - a) / (b - a);
//!
//! const blerp = (a00, a01, a10, a11, tx, ty) =>
//!     lerp(lerp(a00, a01, tx), lerp(a10, a11, tx), ty);
//!
//! const trilerp = (a000, a010, a100, a110, a001, a011, a101, a111,
//!                  tx, ty, tz) =>
//!     lerp(blerp(a000, a010, a100, a110, tx, ty),
//!          blerp(a001, a011, a101, a111, tx, ty),
//!          tz);
//! ```
//!
//! Argument ordering for [`blerp`] and [`trilerp`] follows culori's positional
//! convention so per-bit reproduction across platforms holds.

/// 1-D linear interpolation: `a + t * (b - a)`.
#[inline]
pub fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + t * (b - a)
}

/// Inverse of [`lerp`]: returns `t` such that `lerp(a, b, t) == value`.
///
/// When `a == b` the result is `NaN` (`(v - a) / 0`), matching culori.
#[inline]
pub fn unlerp(a: f64, b: f64, value: f64) -> f64 {
    (value - a) / (b - a)
}

/// Bilinear interpolation across a 2x2 grid.
///
/// Argument order matches culori's `blerp(a00, a01, a10, a11, tx, ty)`:
/// the first dimension (`tx`) lerps `a00→a01` and `a10→a11`; the second
/// (`ty`) lerps the two resulting values.
#[inline]
pub fn blerp(a00: f64, a01: f64, a10: f64, a11: f64, tx: f64, ty: f64) -> f64 {
    lerp(lerp(a00, a01, tx), lerp(a10, a11, tx), ty)
}

/// Trilinear interpolation across a 2x2x2 grid.
///
/// Argument order matches culori's `trilerp(a000, a010, a100, a110, a001,
/// a011, a101, a111, tx, ty, tz)`. The two `z`-slices are bilinearly
/// interpolated then lerped along `tz`.
#[inline]
#[allow(clippy::too_many_arguments)]
pub fn trilerp(
    a000: f64,
    a010: f64,
    a100: f64,
    a110: f64,
    a001: f64,
    a011: f64,
    a101: f64,
    a111: f64,
    tx: f64,
    ty: f64,
    tz: f64,
) -> f64 {
    lerp(
        blerp(a000, a010, a100, a110, tx, ty),
        blerp(a001, a011, a101, a111, tx, ty),
        tz,
    )
}
