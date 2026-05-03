//! Ported tests for `interpolator_spline_monotone`,
//! `interpolator_spline_monotone_2`, and
//! `interpolator_spline_monotone_closed`. Reference values produced from
//! culori 4.0.2 via `node -e`.

use culors::{
    interpolator_spline_monotone, interpolator_spline_monotone_2,
    interpolator_spline_monotone_closed,
};

const TOL: f64 = 1e-10;

fn close(actual: f64, expected: f64, label: &str) {
    let diff = (actual - expected).abs();
    assert!(
        diff <= TOL,
        "{label}: expected {expected}, got {actual} (diff {diff:.3e})"
    );
}

#[test]
fn monotone_four_stops() {
    let arr = [0.0, 1.0, 0.5, 0.7];
    let f = interpolator_spline_monotone()(&arr);
    let cases = [
        (0.0, 0.0),
        (0.1, 0.36300000000000004),
        (0.25, 0.890625),
        (0.5, 0.75),
        (0.75, 0.521875),
        (0.9, 0.6274),
        (1.0, 0.7),
    ];
    for (t, expected) in cases {
        close(f(t), expected, &format!("monotone t={t}"));
    }
}

#[test]
fn monotone_preserves_monotonicity() {
    // Steffen-style monotone spline must not overshoot for monotonic data.
    let arr = [0.1, 0.3, 0.5, 0.9];
    let f = interpolator_spline_monotone()(&arr);
    let cases = [
        (0.0, 0.1),
        (0.1, 0.16),
        (0.25, 0.25),
        (0.5, 0.3875),
        (0.75, 0.5859375),
        (0.9, 0.7737000000000002),
        (1.0, 0.9000000000000001),
    ];
    for (t, expected) in cases {
        close(f(t), expected, &format!("mono-monotonic t={t}"));
    }
    // Empirical: walk t in 100 steps; values must be non-decreasing.
    let mut prev = f(0.0);
    for i in 1..=100 {
        let v = f(i as f64 / 100.0);
        assert!(v >= prev - 1e-12, "non-monotonic at i={i}: {prev} -> {v}");
        prev = v;
    }
}

#[test]
fn monotone_two_stops_falls_back_to_linear() {
    let arr = [0.2, 0.8];
    let f = interpolator_spline_monotone()(&arr);
    for t in [0.0_f64, 0.1, 0.5, 0.9, 1.0] {
        let expected = 0.2 + t * 0.6;
        close(f(t), expected, &format!("mono-2 t={t}"));
    }
}

#[test]
fn monotone_2_four_stops() {
    let arr = [0.0, 1.0, 0.5, 0.7];
    let f = interpolator_spline_monotone_2()(&arr);
    let cases = [
        (0.0, 0.0),
        (0.1, 0.47325),
        (0.25, 0.92578125),
        (0.5, 0.75),
        (0.75, 0.5125),
        (0.9, 0.598),
        (1.0, 0.7),
    ];
    for (t, expected) in cases {
        close(f(t), expected, &format!("monotone2 t={t}"));
    }
}

#[test]
fn monotone_closed_four_stops() {
    let arr = [0.0, 1.0, 0.5, 0.7];
    let f = interpolator_spline_monotone_closed()(&arr);
    let cases = [
        (0.0, 0.0),
        (0.1, 0.21600000000000005),
        (0.25, 0.84375),
        (0.5, 0.75),
        (0.75, 0.53125),
        (0.9, 0.6568),
        (1.0, 0.7),
    ];
    for (t, expected) in cases {
        close(f(t), expected, &format!("monotoneClosed t={t}"));
    }
}

#[test]
fn monotone_closed_two_stops_linear_fallback() {
    let arr = [0.2, 0.8];
    let f = interpolator_spline_monotone_closed()(&arr);
    for t in [0.0_f64, 0.5, 1.0] {
        let expected = 0.2 + t * 0.6;
        close(f(t), expected, &format!("monoClosed-2 t={t}"));
    }
}
