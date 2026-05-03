//! Spline-based per-channel interpolators.
//!
//! Mirrors culori 4.0.2's `interpolate/splineBasis.js`,
//! `interpolate/splineNatural.js`, and `interpolate/splineMonotone.js`.
//! Each public factory returns a closure that, given a channel-stop slice
//! `&[f64]`, builds a sampler `Fn(f64) -> f64` over `[0, 1]`.
//!
//! These factories slot into [`crate::InterpolateOptions::channel_interpolator`]
//! to swap out culori's default linear-per-channel sampling.

use crate::interpolate::lerp::linear_interpolator;

/// Channel sampler returned by spline factories. Maps `t` in `[0, 1]` to a
/// channel value.
pub type ChannelInterp = Box<dyn Fn(f64) -> f64 + Send + Sync>;

/// Factory type used by [`crate::InterpolateOptions::channel_interpolator`].
/// Given a channel's stop list, builds a per-channel sampler. The factories
/// in this module return `impl Fn(&[f64]) -> ChannelInterp` and convert
/// transparently to this trait object.
pub type ChannelInterpFactory = Box<dyn Fn(&[f64]) -> ChannelInterp + Send + Sync>;

#[inline]
fn modulo(v: i64, l: i64) -> usize {
    (((v % l) + l) % l) as usize
}

#[inline]
fn bspline(v_im2: f64, v_im1: f64, v_i: f64, v_ip1: f64, t: f64) -> f64 {
    let t2 = t * t;
    let t3 = t2 * t;
    ((1.0 - 3.0 * t + 3.0 * t2 - t3) * v_im2
        + (4.0 - 6.0 * t2 + 3.0 * t3) * v_im1
        + (1.0 + 3.0 * t + 3.0 * t2 - 3.0 * t3) * v_i
        + t3 * v_ip1)
        / 6.0
}

/// Uniform B-spline interpolator factory. Mirrors culori's
/// `interpolatorSplineBasis`.
///
/// Returns a closure `Fn(&[f64]) -> ChannelInterp`. The resulting sampler
/// uses one-sided extrapolation at the boundaries: the virtual control
/// point before stop 0 is `2*arr[0] - arr[1]`, and the one past stop n is
/// `2*arr[n] - arr[n-1]`. With fewer than two stops the sampler returns the
/// single value (or `NaN` for an empty input).
pub fn interpolator_spline_basis() -> ChannelInterpFactory {
    Box::new(|arr: &[f64]| -> ChannelInterp {
        let arr: Vec<f64> = arr.to_vec();
        Box::new(move |t: f64| {
            let n = arr.len();
            if n == 0 {
                return f64::NAN;
            }
            if n == 1 {
                return arr[0];
            }
            let classes = (n - 1) as i64;
            let i = if t >= 1.0 {
                classes - 1
            } else {
                ((t * classes as f64).floor() as i64).max(0)
            };
            let i_us = i as usize;
            let v_im2 = if i > 0 {
                arr[i_us - 1]
            } else {
                2.0 * arr[i_us] - arr[i_us + 1]
            };
            let v_im1 = arr[i_us];
            let v_i = arr[i_us + 1];
            let v_ip1 = if i < classes - 1 {
                arr[i_us + 2]
            } else {
                2.0 * arr[i_us + 1] - arr[i_us]
            };
            let local_t = (t - i as f64 / classes as f64) * classes as f64;
            bspline(v_im2, v_im1, v_i, v_ip1, local_t)
        })
    })
}

/// Closed-loop uniform B-spline interpolator factory. Mirrors culori's
/// `interpolatorSplineBasisClosed`.
///
/// The control polygon wraps around: stop `-1 mod n` is the last stop, stop
/// `n mod n` is the first stop. This produces a periodic ramp suitable for
/// hue rings.
pub fn interpolator_spline_basis_closed() -> ChannelInterpFactory {
    Box::new(|arr: &[f64]| -> ChannelInterp {
        let arr: Vec<f64> = arr.to_vec();
        Box::new(move |t: f64| {
            let n = arr.len();
            if n == 0 {
                return f64::NAN;
            }
            if n == 1 {
                return arr[0];
            }
            let classes = (n - 1) as i64;
            // culori: i = floor(t * classes); no clamping. With t == 1
            // and classes >= 1 this yields i == classes; the modulo
            // handles the wrap.
            let i = (t * classes as f64).floor() as i64;
            let len = arr.len() as i64;
            let v_im2 = arr[modulo(i - 1, len)];
            let v_im1 = arr[modulo(i, len)];
            let v_i = arr[modulo(i + 1, len)];
            let v_ip1 = arr[modulo(i + 2, len)];
            let local_t = (t - i as f64 / classes as f64) * classes as f64;
            bspline(v_im2, v_im1, v_i, v_ip1, local_t)
        })
    })
}

// Solver for the natural cubic spline. Mirrors culori's `solve` in
// `splineNatural.js`. Input is the stop array; output is the array of
// transformed control points fed back through the basis interpolator.
fn natural_solve(v: &[f64]) -> Vec<f64> {
    let n = v.len() - 1;
    let mut c = vec![0.0_f64; n];
    let mut _v = vec![0.0_f64; n];
    let mut sol = vec![0.0_f64; n + 1];

    if n >= 2 {
        c[1] = 0.25;
        _v[1] = (6.0 * v[1] - v[0]) / 4.0;
        for i in 2..n {
            c[i] = 1.0 / (4.0 - c[i - 1]);
            let term_n = if i == n - 1 { v[n] } else { 0.0 };
            _v[i] = (6.0 * v[i] - term_n - _v[i - 1]) * c[i];
        }
    }

    sol[0] = v[0];
    sol[n] = v[n];
    if n >= 2 {
        sol[n - 1] = _v[n - 1];
        for i in (1..=(n - 2)).rev() {
            sol[i] = _v[i] - c[i] * sol[i + 1];
        }
    }

    sol
}

/// Natural cubic spline interpolator factory. Mirrors culori's
/// `interpolatorSplineNatural`: solves the tridiagonal system to derive
/// B-spline control points whose curve passes exactly through the stops,
/// then evaluates with the uniform B-spline.
pub fn interpolator_spline_natural() -> ChannelInterpFactory {
    let basis = interpolator_spline_basis();
    Box::new(move |arr: &[f64]| -> ChannelInterp {
        if arr.len() < 2 {
            return basis(arr);
        }
        let solved = natural_solve(arr);
        basis(&solved)
    })
}

/// Closed-loop natural cubic spline factory. Mirrors culori's
/// `interpolatorSplineNaturalClosed`.
pub fn interpolator_spline_natural_closed() -> ChannelInterpFactory {
    let basis_closed = interpolator_spline_basis_closed();
    Box::new(move |arr: &[f64]| -> ChannelInterp {
        if arr.len() < 2 {
            return basis_closed(arr);
        }
        let solved = natural_solve(arr);
        basis_closed(&solved)
    })
}

#[inline]
fn sgn(v: f64) -> f64 {
    if v > 0.0 {
        1.0
    } else if v < 0.0 {
        -1.0
    } else {
        0.0
    }
}

// Returns `(s, p, yp)`. Mirrors culori's `mono` helper. `s` has length `n`
// (`= arr.len() - 1`); culori's loop populates `p` and `yp` only at indices
// `1..n`, leaving `[0]` and `[n]` as `undefined`. Each caller then writes
// those boundary entries before the inner interpolator runs, which reads
// `yp[i + 1]` up to `yp[n]`. We allocate `p` and `yp` with length `n + 1`
// and pre-fill the boundary slots with `NaN` so the callers can overwrite
// in place.
fn mono(arr: &[f64]) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
    let n = (arr.len() - 1) as f64;
    let len = arr.len() - 1;
    let mut s = Vec::with_capacity(len);
    let mut p = vec![f64::NAN; len + 1];
    let mut yp = vec![f64::NAN; len + 1];
    for i in 0..len {
        s.push((arr[i + 1] - arr[i]) * n);
        if i > 0 {
            let p_i = 0.5 * (arr[i + 1] - arr[i - 1]) * n;
            p[i] = p_i;
            yp[i] =
                (sgn(s[i - 1]) + sgn(s[i])) * s[i - 1].abs().min(s[i].abs()).min(0.5 * p_i.abs());
        }
    }
    (s, p, yp)
}

// Evaluate the Hermite cubic for the monotone spline. Mirrors culori's
// inner `interpolator(arr, yp, s)`.
fn evaluate_monotone(arr: Vec<f64>, yp: Vec<f64>, s: Vec<f64>) -> ChannelInterp {
    Box::new(move |t: f64| {
        let n = (arr.len() - 1) as f64;
        let n_us = arr.len() - 1;
        let n2 = n * n;
        let i = if t >= 1.0 {
            n_us - 1
        } else {
            ((t * n).floor() as isize).max(0) as usize
        };
        let t1 = t - i as f64 / n;
        let t2 = t1 * t1;
        let t3 = t2 * t1;
        (yp[i] + yp[i + 1] - 2.0 * s[i]) * n2 * t3
            + (3.0 * s[i] - 2.0 * yp[i] - yp[i + 1]) * n * t2
            + yp[i] * t1
            + arr[i]
    })
}

/// Monotone cubic spline factory using one-sided finite differences at the
/// boundaries. Mirrors culori's `interpolatorSplineMonotone` (Steffen
/// 1990). Falls back to linear interpolation for fewer than three stops.
pub fn interpolator_spline_monotone() -> ChannelInterpFactory {
    Box::new(|arr: &[f64]| -> ChannelInterp {
        if arr.len() < 3 {
            return Box::new(linear_interpolator(arr.to_vec()));
        }
        let (s, _p, mut yp) = mono(arr);
        let n = arr.len() - 1;
        yp[0] = s[0];
        yp[n] = s[n - 1];
        evaluate_monotone(arr.to_vec(), yp, s)
    })
}

/// Clamped monotone cubic spline factory. Mirrors culori's
/// `interpolatorSplineMonotone2`: derives the boundary derivatives by
/// passing a parabola through the three nearest stops.
pub fn interpolator_spline_monotone_2() -> ChannelInterpFactory {
    Box::new(|arr: &[f64]| -> ChannelInterp {
        if arr.len() < 3 {
            return Box::new(linear_interpolator(arr.to_vec()));
        }
        let (s, mut p, mut yp) = mono(arr);
        let n_us = arr.len() - 1;
        let n = n_us as f64;
        p[0] = (arr[1] * 2.0 - arr[0] * 1.5 - arr[2] * 0.5) * n;
        p[n_us] = (arr[n_us] * 1.5 - arr[n_us - 1] * 2.0 + arr[n_us - 2] * 0.5) * n;
        yp[0] = if p[0] * s[0] <= 0.0 {
            0.0
        } else if p[0].abs() > 2.0 * s[0].abs() {
            2.0 * s[0]
        } else {
            p[0]
        };
        yp[n_us] = if p[n_us] * s[n_us - 1] <= 0.0 {
            0.0
        } else if p[n_us].abs() > 2.0 * s[n_us - 1].abs() {
            2.0 * s[n_us - 1]
        } else {
            p[n_us]
        };
        evaluate_monotone(arr.to_vec(), yp, s)
    })
}

/// Closed-loop monotone cubic spline factory. Mirrors culori's
/// `interpolatorSplineMonotoneClosed`: treats the stop list as periodic.
pub fn interpolator_spline_monotone_closed() -> ChannelInterpFactory {
    Box::new(|arr: &[f64]| -> ChannelInterp {
        if arr.len() < 3 {
            return Box::new(linear_interpolator(arr.to_vec()));
        }
        let (s, mut p, mut yp) = mono(arr);
        let n_us = arr.len() - 1;
        let n = n_us as f64;
        p[0] = 0.5 * (arr[1] - arr[n_us]) * n;
        p[n_us] = 0.5 * (arr[0] - arr[n_us - 1]) * n;
        let s_m1 = (arr[0] - arr[n_us]) * n;
        let s_n = s_m1;
        yp[0] = (sgn(s_m1) + sgn(s[0])) * s_m1.abs().min(s[0].abs()).min(0.5 * p[0].abs());
        yp[n_us] = (sgn(s[n_us - 1]) + sgn(s_n))
            * s[n_us - 1].abs().min(s_n.abs()).min(0.5 * p[n_us].abs());
        evaluate_monotone(arr.to_vec(), yp, s)
    })
}
