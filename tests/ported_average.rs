//! Tests for `average_number` and `average_angle`.
//!
//! Reference values come from culori 4.0.2 invoked through Node — see
//! the `node -e` snippets in the wave brief. The mode-aware `average`
//! function (and its color tests) lands in the next commit.

use culor::{average_angle, average_number};

const EPS: f64 = 1e-12;

#[track_caller]
fn approx(label: &str, a: f64, b: f64) {
    let diff = (a - b).abs();
    assert!(
        diff <= EPS,
        "{label}: actual={a}, expected={b}, diff={diff}"
    );
}

// ----- average_number ---------------------------------------------------

#[test]
fn average_number_basic_mean() {
    approx("[1,2,3]", average_number(&[1.0, 2.0, 3.0]), 2.0);
}

#[test]
fn average_number_ignores_nan() {
    approx("[1, NaN, 3]", average_number(&[1.0, f64::NAN, 3.0]), 2.0);
}

#[test]
fn average_number_all_nan_returns_nan() {
    let v = average_number(&[f64::NAN, f64::NAN]);
    assert!(v.is_nan(), "expected NaN, got {v}");
}

#[test]
fn average_number_empty_returns_nan() {
    let v = average_number(&[]);
    assert!(v.is_nan(), "expected NaN, got {v}");
}

#[test]
fn average_number_singleton() {
    approx("[5]", average_number(&[5.0]), 5.0);
}

#[test]
fn average_number_negatives_and_fractions() {
    approx(
        "[-1, 0.5, 2]",
        average_number(&[-1.0, 0.5, 2.0]),
        (-1.0 + 0.5 + 2.0) / 3.0,
    );
}

// ----- average_angle ----------------------------------------------------

#[test]
fn average_angle_singleton_zero() {
    approx("[0]", average_angle(&[0.0]), 0.0);
}

#[test]
fn average_angle_wraps_low_high_pair_to_360() {
    let v = average_angle(&[10.0, 350.0]);
    let near_360 = (v - 360.0).abs() < 1e-9;
    let near_0 = v.abs() < 1e-9;
    assert!(near_360 || near_0, "expected ~360 (or ~0), got {v}");
}

#[test]
fn average_angle_wraps_high_low_pair_to_360() {
    let v = average_angle(&[350.0, 10.0]);
    let near_360 = (v - 360.0).abs() < 1e-9;
    let near_0 = v.abs() < 1e-9;
    assert!(near_360 || near_0, "expected ~360 (or ~0), got {v}");
}

#[test]
fn average_angle_quadrant_midpoint() {
    approx("[90, 180]", average_angle(&[90.0, 180.0]), 135.0);
}

#[test]
fn average_angle_orthogonal_pair() {
    approx("[0, 180]", average_angle(&[0.0, 180.0]), 90.0);
}

#[test]
fn average_angle_close_to_180() {
    approx("[170, 190]", average_angle(&[170.0, 190.0]), 180.0);
}

#[test]
fn average_angle_empty_returns_zero() {
    approx("[]", average_angle(&[]), 0.0);
}

#[test]
fn average_angle_all_nan_returns_zero() {
    approx("[NaN]", average_angle(&[f64::NAN]), 0.0);
    approx("[NaN, NaN]", average_angle(&[f64::NAN, f64::NAN]), 0.0);
}

#[test]
fn average_angle_skips_nan() {
    let v = average_angle(&[10.0, f64::NAN, 350.0]);
    let near_360 = (v - 360.0).abs() < 1e-9;
    let near_0 = v.abs() < 1e-9;
    assert!(near_360 || near_0, "expected ~360 (or ~0), got {v}");
}
