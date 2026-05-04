//! Ported tests for the linear piecewise channel interpolator.
//!
//! Mirrors culori 4.0.2's `test/interpolatorLinear.test.js`. culori's
//! `interpolatorLinear(data)` is the linear specialisation of
//! `interpolatorPiecewise((a, b, t) => a + (b - a) * t)`. culors does
//! not ship a separate `interpolator_linear`; the same composition
//! through [`culors::interpolator_piecewise`] reproduces the JS function
//! byte-for-byte, and that is what we test here.
//!
//! Each expected value comes from running culori 4.0.2's
//! `interpolatorLinear` against the same input array; the
//! `samples(10)`-driven sweep matches culori's published test output
//! (with the trailing 1ULP `2.5000000000000002`-style values preserved
//! verbatim).

use culors::{interpolator_piecewise, samples};

const EPS: f64 = 1e-12;

fn lerp_factory() -> impl Fn(&[f64]) -> Box<dyn Fn(f64) -> f64 + Send + Sync> {
    move |stops: &[f64]| interpolator_piecewise(|a: f64, b: f64, t: f64| a + (b - a) * t)(stops)
}

fn close(actual: f64, expected: f64, label: &str) {
    let diff = (actual - expected).abs();
    assert!(
        diff <= EPS,
        "{label}: expected {expected}, got {actual} (diff {diff:.3e})"
    );
}

#[test]
fn culori_fixture_nine_stops_sampled_at_ten_points() {
    // culori reference (samples(10).map(interpolatorLinear(data))):
    //   [ 3, 2.822222222222222, 2.5666666666666664, 1.5000000000000002,
    //     0.9722222222222222, 0.8833333333333333, 0.7000000000000002,
    //     0.4111111111111111, 0.09444444444444447, 0.05 ]
    let data = [3.0, 2.8, 2.5, 1.0, 0.95, 0.8, 0.5, 0.1, 0.05];
    let f = lerp_factory()(&data);
    let actual: Vec<f64> = samples(10).into_iter().map(&f).collect();
    let expected = [
        3.0,
        2.822_222_222_222_222,
        2.566_666_666_666_666_4,
        1.500_000_000_000_000_2,
        0.972_222_222_222_222_2,
        0.883_333_333_333_333_3,
        0.700_000_000_000_000_2,
        0.411_111_111_111_111_1,
        0.094_444_444_444_444_47,
        0.05,
    ];
    assert_eq!(actual.len(), expected.len());
    for (i, (a, e)) in actual.iter().zip(expected.iter()).enumerate() {
        close(*a, *e, &format!("sample[{i}]"));
    }
}

#[test]
fn outside_unit_range_extrapolates_linearly() {
    // culori test: `interpolatorLinear([3, 10, 1])`:
    //   it(-0.5) === -4, it(-1) === -11, it(1.5) === -8, it(2) === -17
    // Below 0 the factory clamps to segment 0; above 1 it clamps to the
    // last segment. Within each segment lerp's `t` is *not* clamped, so
    // the value extrapolates past the endpoint linearly.
    let f = lerp_factory()(&[3.0, 10.0, 1.0]);
    close(f(-0.5), -4.0, "t=-0.5");
    close(f(-1.0), -11.0, "t=-1");
    close(f(1.5), -8.0, "t=1.5");
    close(f(2.0), -17.0, "t=2");
}

#[test]
fn boundary_values_match_endpoints() {
    let f = lerp_factory()(&[3.0, 10.0, 1.0]);
    close(f(0.0), 3.0, "t=0");
    close(f(1.0), 1.0, "t=1");
}

#[test]
fn quarter_points_match_culori() {
    // culori reference: f(0.25) === 6.5, f(0.5) === 10, f(0.75) === 5.5
    let f = lerp_factory()(&[3.0, 10.0, 1.0]);
    close(f(0.25), 6.5, "t=0.25");
    close(f(0.5), 10.0, "t=0.5");
    close(f(0.75), 5.5, "t=0.75");
}

#[test]
fn two_stops_collapse_to_single_segment() {
    // With only two stops the partition has one class; samples(11) should
    // hit every tenth between the endpoints.
    let f = lerp_factory()(&[0.0, 10.0]);
    for (i, t) in samples(11).into_iter().enumerate() {
        close(f(t), i as f64, &format!("samples(11)[{i}]"));
    }
}

#[test]
fn empty_stops_returns_nan() {
    // culori returns `undefined`; culors uses NaN as the missing marker.
    let f = lerp_factory()(&[]);
    assert!(f(0.0).is_nan());
    assert!(f(0.5).is_nan());
    assert!(f(1.0).is_nan());
}

#[test]
fn single_stop_returns_nan_no_segments_to_interpolate() {
    // One stop produces zero classes; the result is NaN at every t.
    let f = lerp_factory()(&[7.0]);
    assert!(f(0.0).is_nan());
    assert!(f(0.5).is_nan());
}

#[test]
fn missing_endpoint_propagates_present_value() {
    // culors's NaN-as-missing rule (matching culori's `[a, a]` / `[b, b]`
    // propagation): a class with one missing endpoint repeats the present
    // endpoint, so an array with NaN holes still yields finite values
    // inside those segments. Each consecutive pair becomes its own class,
    // so `[NaN, 10, 5, NaN]` produces classes (10,10), (10,5), (5,5).
    let data = [f64::NAN, 10.0, 5.0, f64::NAN];
    let f = lerp_factory()(&data);
    // Segment 0: classes [(10, 10)] — 10 throughout.
    close(f(0.0), 10.0, "seg 0 start");
    close(f(0.125), 10.0, "seg 0 mid");
    // Segment 1 covers [1/3, 2/3]. local_t=0.5 at t = 0.5.
    close(f(0.5), 7.5, "seg 1 midpoint t=0.5");
    // Segment 2: classes [(5, 5)] — 5 throughout.
    close(f(0.875), 5.0, "seg 2 mid");
}

#[test]
fn all_nan_stops_yield_nan() {
    let f = lerp_factory()(&[f64::NAN, f64::NAN, f64::NAN]);
    assert!(f(0.0).is_nan());
    assert!(f(0.5).is_nan());
    assert!(f(1.0).is_nan());
}
