//! Ported tests for [`culors::round`], matching culori 4.0.2's `round`.

#![allow(clippy::approx_constant)]

use culors::round;

const EPS: f64 = 1e-15;

#[track_caller]
fn assert_close(label: &str, got: f64, expected: f64) {
    if expected.is_nan() {
        assert!(got.is_nan(), "{label}: expected NaN, got {got}");
        return;
    }
    let diff = (got - expected).abs();
    assert!(
        diff <= EPS,
        "{label}: got {got}, expected {expected} (diff {diff})"
    );
}

#[test]
fn round_4_basic() {
    let r = round(4);
    // node: c.round(4)(0.123456789)
    assert_close("0.123456789 -> 4dp", r(0.123456789), 0.1235);
}

#[test]
fn round_2_negative_value() {
    let r = round(2);
    // node: c.round(2)(-1.235)
    assert_close("-1.235 -> 2dp", r(-1.235), -1.24);
}

#[test]
fn round_0_positive_half_up() {
    // JS Math.round(0.5) === 1.
    let r = round(0);
    assert_close("0.5 -> 0dp", r(0.5), 1.0);
    assert_close("1.5 -> 0dp", r(1.5), 2.0);
}

#[test]
fn round_0_negative_half_to_pos_inf() {
    // JS Math.round(-0.5) === -0 (not -1). The standard says ties go
    // toward +∞.
    let r = round(0);
    assert_close("-0.5 -> 0dp", r(-0.5), 0.0);
    assert_close("-1.5 -> 0dp", r(-1.5), -1.0);
}

#[test]
fn round_pi_default_4() {
    // Mirrors culori's default precision (4) called explicitly.
    let r = round(4);
    assert_close("pi -> 4dp", r(std::f64::consts::PI), 3.1416);
}
