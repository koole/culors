//! Ported tests for [`culors::easing`].
//!
//! Expected values come from culori 4.0.2 invoked via `node -e`:
//!
//! ```js
//! easingSmoothstep(0.25)         === 0.15625
//! easingSmoothstep(0.75)         === 0.84375
//! easingSmoothstepInverse(0.25)  === 0.32635182233306964
//! easingSmootherstep(0.25)       === 0.103515625
//! easingInOutSine(0.25)          === 0.1464466094067262
//! easingMidpoint(0.3)(0.5)       === 0.670952880320326
//! easingMidpoint(0)(0.5)         === 1
//! easingMidpoint(1)(0.5)         === 0
//! easingGamma(2.2)(0.5)          === 0.217637640824031
//! ```
//!
//! culori has no test file for these factories in the published package, so
//! the suite is built from the documented formulas plus exact spot checks
//! against the JS implementation.

use culors::{
    easing_gamma, easing_in_out_sine, easing_midpoint, easing_smootherstep, easing_smoothstep,
    easing_smoothstep_inverse,
};

const EPS: f64 = 1e-15;

#[track_caller]
fn close(label: &str, got: f64, expected: f64) {
    let d = (got - expected).abs();
    assert!(d < EPS, "{label}: got {got}, expected {expected}, |Δ|={d}");
}

#[test]
fn smoothstep_endpoints_and_quarters() {
    let s = easing_smoothstep();
    close("smoothstep(0)", s(0.0), 0.0);
    close("smoothstep(1)", s(1.0), 1.0);
    close("smoothstep(0.5)", s(0.5), 0.5);
    close("smoothstep(0.25)", s(0.25), 0.15625);
    close("smoothstep(0.75)", s(0.75), 0.84375);
}

#[test]
fn smoothstep_inverse_round_trips() {
    let s = easing_smoothstep();
    let s_inv = easing_smoothstep_inverse();
    // s_inv(s(t)) ≈ t for t in (0, 1).
    for &t in &[0.05, 0.1, 0.3, 0.5, 0.7, 0.9, 0.95] {
        let round = s_inv(s(t));
        assert!(
            (round - t).abs() < 1e-12,
            "round-trip at t={t}: got {round}"
        );
    }
}

#[test]
fn smoothstep_inverse_spot_checks() {
    let f = easing_smoothstep_inverse();
    close("smoothstepInv(0.25)", f(0.25), 0.326_351_822_333_069_64);
    close("smoothstepInv(0.5)", f(0.5), 0.5);
}

#[test]
fn smootherstep_spot_checks() {
    let s = easing_smootherstep();
    close("smootherstep(0)", s(0.0), 0.0);
    close("smootherstep(0.25)", s(0.25), 0.103_515_625);
    close("smootherstep(0.5)", s(0.5), 0.5);
    close("smootherstep(1)", s(1.0), 1.0);
}

#[test]
fn in_out_sine_spot_checks() {
    let f = easing_in_out_sine();
    close("inOutSine(0)", f(0.0), 0.0);
    close("inOutSine(0.25)", f(0.25), 0.146_446_609_406_726_2);
    close("inOutSine(1)", f(1.0), 1.0);
}

#[test]
fn midpoint_default() {
    // H = 0.5 → exponent ln(0.5)/ln(0.5) = 1, so the curve is the identity.
    let f = easing_midpoint(0.5);
    close("midpoint(0.5)(0)", f(0.0), 0.0);
    close("midpoint(0.5)(0.5)", f(0.5), 0.5);
    close("midpoint(0.5)(1)", f(1.0), 1.0);
}

#[test]
fn midpoint_at_03_passes_through_05_at_t_03() {
    let f = easing_midpoint(0.3);
    // f(0.3) === 0.5 by construction (midpoint definition).
    close("f(0.3) at H=0.3", f(0.3), 0.5);
    close("f(0.5) at H=0.3", f(0.5), 0.670_952_880_320_326);
}

#[test]
fn midpoint_degenerate_edges() {
    // H ≤ 0 collapses to constant 1; H ≥ 1 collapses to constant 0.
    let lo = easing_midpoint(0.0);
    let hi = easing_midpoint(1.0);
    for t in [0.0, 0.25, 0.5, 0.75, 1.0] {
        close("H=0", lo(t), 1.0);
        close("H=1", hi(t), 0.0);
    }
    let neg = easing_midpoint(-0.2);
    close("H=-0.2", neg(0.5), 1.0);
}

#[test]
fn gamma_identity_when_one() {
    let f = easing_gamma(1.0);
    for t in [0.0, 0.25, 0.5, 0.75, 1.0] {
        // Bit-for-bit equal — no Math.pow on the JS side either.
        assert_eq!(f(t), t);
    }
}

#[test]
fn gamma_2_2_spot_check() {
    let f = easing_gamma(2.2);
    close("gamma(2.2)(0.5)", f(0.5), 0.217_637_640_824_031);
    close("gamma(2.2)(0)", f(0.0), 0.0);
    close("gamma(2.2)(1)", f(1.0), 1.0);
}
