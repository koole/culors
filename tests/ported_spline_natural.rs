//! Ported tests for `interpolator_spline_natural` and
//! `interpolator_spline_natural_closed`. Reference values produced from
//! culori 4.0.2 via `node -e`.

use culors::{interpolator_spline_natural, interpolator_spline_natural_closed};

const TOL: f64 = 1e-10;

fn close(actual: f64, expected: f64, label: &str) {
    let diff = (actual - expected).abs();
    assert!(
        diff <= TOL,
        "{label}: expected {expected}, got {actual} (diff {diff:.3e})"
    );
}

#[test]
fn natural_four_stops() {
    let arr = [0.0, 1.0, 0.5, 0.7];
    let f = interpolator_spline_natural()(&arr);
    let cases = [
        (0.0, 0.0),
        (0.1, 0.42194000000000015),
        (0.25, 0.8965625),
        (0.5, 0.81),
        (0.75, 0.45593749999999994),
        (0.9, 0.56174),
        (1.0, 0.6999999999999998),
    ];
    for (t, expected) in cases {
        close(f(t), expected, &format!("natural t={t}"));
    }
}

#[test]
fn natural_three_stops() {
    let arr = [0.0, 0.5, 1.0];
    let f = interpolator_spline_natural()(&arr);
    let cases = [
        (0.0, 0.0),
        (0.1, 0.1493333333333334),
        (0.25, 0.3645833333333333),
        (0.5, 0.6666666666666666),
        (0.75, 0.8645833333333334),
        (0.9, 0.9493333333333333),
        (1.0, 1.0),
    ];
    for (t, expected) in cases {
        close(f(t), expected, &format!("natural-3 t={t}"));
    }
}

#[test]
fn natural_passes_through_stops() {
    let arr = [0.1, 0.4, 0.9, 0.2];
    let f = interpolator_spline_natural()(&arr);
    let n = (arr.len() - 1) as f64;
    for (i, &expected) in arr.iter().enumerate() {
        let t = i as f64 / n;
        close(f(t), expected, &format!("stop {i}"));
    }
}

#[test]
fn natural_five_stops_symmetric() {
    let arr = [0.0, 0.25, 1.0, 0.25, 0.0];
    let f = interpolator_spline_natural()(&arr);
    let cases = [
        (0.0, 0.0),
        (0.1, 0.016000000000000004),
        (0.25, 0.25),
        (0.5, 1.0),
        (0.75, 0.25),
        (0.9, 0.015999999999999986),
        (1.0, 0.0),
    ];
    for (t, expected) in cases {
        close(f(t), expected, &format!("natural-5 t={t}"));
    }
}

#[test]
fn natural_closed_four_stops() {
    let arr = [0.0, 1.0, 0.5, 0.7];
    let f = interpolator_spline_natural_closed()(&arr);
    let cases = [
        (0.0, 0.35777777777777775),
        (0.1, 0.5446577777777779),
        (0.25, 0.9021527777777778),
        (0.5, 0.81),
        (0.75, 0.4528472222222222),
        (0.9, 0.4939022222222222),
        (1.0, 0.5022222222222222),
    ];
    for (t, expected) in cases {
        close(f(t), expected, &format!("naturalClosed t={t}"));
    }
}

#[test]
fn natural_closed_symmetric_input_is_periodic() {
    // For a symmetric stop list (same first and last value) the closed
    // natural spline collapses to the open natural spline at every t.
    let arr = [0.0, 0.25, 1.0, 0.25, 0.0];
    let f = interpolator_spline_natural_closed()(&arr);
    let cases = [(0.0, 0.0), (0.5, 1.0), (1.0, 0.0)];
    for (t, expected) in cases {
        close(f(t), expected, &format!("naturalClosed-5 t={t}"));
    }
}
