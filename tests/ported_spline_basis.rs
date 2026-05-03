//! Ported tests for `interpolator_spline_basis` and
//! `interpolator_spline_basis_closed`. Reference values produced with
//! `node -e "const c = require('culori'); …"` against the vendored
//! culori 4.0.2.

use culors::{interpolator_spline_basis, interpolator_spline_basis_closed};

const TOL: f64 = 1e-10;

fn close(actual: f64, expected: f64, label: &str) {
    let diff = (actual - expected).abs();
    assert!(
        diff <= TOL,
        "{label}: expected {expected}, got {actual} (diff {diff:.3e})"
    );
}

#[test]
fn basis_four_stops() {
    let arr = [0.0, 1.0, 0.5, 0.7];
    let f = interpolator_spline_basis()(&arr);
    let cases = [
        (0.0, 0.0),
        (0.1, 0.29325000000000007),
        (0.25, 0.64453125),
        (0.5, 0.7333333333333334),
        (0.75, 0.59921875),
        (0.9, 0.6431499999999999),
        (1.0, 0.6999999999999998),
    ];
    for (t, expected) in cases {
        close(f(t), expected, &format!("basis t={t}"));
    }
}

#[test]
fn basis_three_stops_is_linear() {
    // Three evenly-spaced control points produce a linear curve under the
    // open uniform B-spline because the boundary extrapolation cancels the
    // curvature at the midpoint.
    let arr = [0.0, 0.5, 1.0];
    let f = interpolator_spline_basis()(&arr);
    for t in [0.0, 0.1, 0.5, 0.9, 1.0] {
        close(f(t), t, &format!("basis-3 t={t}"));
    }
}

#[test]
fn basis_five_stops_symmetric() {
    let arr = [0.0, 0.25, 1.0, 0.25, 0.0];
    let f = interpolator_spline_basis()(&arr);
    let cases = [
        (0.0, 0.0),
        (0.1, 0.10533333333333335),
        (0.25, 0.3333333333333333),
        (0.5, 0.75),
        (0.75, 0.3333333333333333),
        (0.9, 0.10533333333333329),
        (1.0, 0.0),
    ];
    for (t, expected) in cases {
        close(f(t), expected, &format!("basis-5 t={t}"));
    }
}

#[test]
fn basis_two_stops_is_linear() {
    let arr = [0.2, 0.8];
    let f = interpolator_spline_basis()(&arr);
    for t in [0.0_f64, 0.1, 0.5, 0.9, 1.0] {
        let expected = 0.2 + t * 0.6;
        close(f(t), expected, &format!("basis-2 t={t}"));
    }
}

#[test]
fn basis_closed_four_stops() {
    let arr = [0.0, 1.0, 0.5, 0.7];
    let f = interpolator_spline_basis_closed()(&arr);
    let cases = [
        (0.0, 0.2833333333333333),
        (0.1, 0.3904333333333334),
        (0.25, 0.6489583333333333),
        (0.5, 0.7333333333333334),
        (0.75, 0.5968749999999999),
        (0.9, 0.5916999999999999),
        (1.0, 0.5499999999999999),
    ];
    for (t, expected) in cases {
        close(f(t), expected, &format!("basisClosed t={t}"));
    }
}

#[test]
fn basis_closed_periodicity() {
    // closed B-spline values at t=0 and t=1 differ in general because the
    // input array is treated as control points (open cycle). Pin the
    // observed values from culori instead of asserting equality.
    let arr = [0.0, 0.25, 1.0, 0.25, 0.0];
    let f = interpolator_spline_basis_closed()(&arr);
    let cases = [
        (0.0, 0.041666666666666664),
        (0.5, 0.75),
        (1.0, 0.041666666666666664),
    ];
    for (t, expected) in cases {
        close(f(t), expected, &format!("basisClosed-5 t={t}"));
    }
}
