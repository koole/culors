//! Color averaging helpers.
//!
//! Mirrors the helpers from culori 4.0.2's `src/average.js`. The mode-aware
//! [`average`] color reducer that uses these helpers lands alongside in the
//! next commit.
//!
//! Both helpers treat `NaN` as "missing" — culori uses `undefined`, which
//! we model as `NaN` for non-alpha channels.

/// Arithmetic mean of `values`, ignoring `NaN`. Returns `NaN` if every
/// value is `NaN` or if `values` is empty.
pub fn average_number(values: &[f64]) -> f64 {
    let mut sum = 0.0;
    let mut count = 0usize;
    for &v in values {
        if !v.is_nan() {
            sum += v;
            count += 1;
        }
    }
    if count == 0 {
        f64::NAN
    } else {
        sum / count as f64
    }
}

/// Circular mean of `angles` in degrees, ignoring `NaN`.
///
/// Returns a value in `[0, 360]`. The upper bound shows up when atan2
/// underflow lands the raw result just below zero, which the wrap-around
/// branch maps to `360 + angle`. culori behaves identically.
///
/// Empty input or all-NaN input returns `0` because `atan2(0, 0) = 0`,
/// and `0 < 0` is false.
pub fn average_angle(angles: &[f64]) -> f64 {
    let mut sum_sin = 0.0;
    let mut sum_cos = 0.0;
    for &a in angles {
        if !a.is_nan() {
            let rad = a.to_radians();
            sum_sin += rad.sin();
            sum_cos += rad.cos();
        }
    }
    let angle = sum_sin.atan2(sum_cos).to_degrees();
    if angle < 0.0 {
        360.0 + angle
    } else {
        angle
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn average_number_arithmetic() {
        let v = average_number(&[1.0, 2.0, 3.0]);
        assert!((v - 2.0).abs() < 1e-12);
    }

    #[test]
    fn average_angle_circular() {
        let v = average_angle(&[10.0, 350.0]);
        assert!((v - 360.0).abs() < 1e-9 || v.abs() < 1e-9);
    }
}
