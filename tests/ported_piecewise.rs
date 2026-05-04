//! Tests for `interpolator_piecewise` — the higher-order channel-interpolator
//! factory that mirrors culori 4.0.2's `interpolatorPiecewise`
//! (`node_modules/culori/src/interpolate/piecewise.js`).
//!
//! Each expected value comes from running culori's `interpolatorPiecewise`
//! against the same inputs in node, e.g.:
//!
//! ```text
//! node -e "import('culori').then(c => {
//!   const f = c.interpolatorPiecewise((a, b, t) => a + (b - a) * t)([0, 10, 100]);
//!   console.log(f(0.25));  // 5
//! })"
//! ```

use culors::interpolator_piecewise;

const EPS: f64 = 1e-12;

fn close(actual: f64, expected: f64, label: &str) {
    let diff = (actual - expected).abs();
    assert!(
        diff <= EPS,
        "{label}: expected {expected}, got {actual} (diff {diff:.3e})"
    );
}

#[test]
fn piecewise_linear_two_stops() {
    // Linear lerp; one segment.
    let factory = interpolator_piecewise(|a: f64, b: f64, t: f64| a + (b - a) * t);
    let f = factory(&[0.0, 10.0]);
    close(f(0.0), 0.0, "t=0");
    close(f(0.5), 5.0, "t=0.5");
    close(f(1.0), 10.0, "t=1");
}

#[test]
fn piecewise_linear_three_stops() {
    // Two segments: [0..10] then [10..100]. At t=0.25 we are halfway through
    // segment 0 (-> 5); at t=0.75 we are halfway through segment 1 (-> 55).
    let factory = interpolator_piecewise(|a: f64, b: f64, t: f64| a + (b - a) * t);
    let f = factory(&[0.0, 10.0, 100.0]);
    close(f(0.0), 0.0, "t=0");
    close(f(0.25), 5.0, "t=0.25");
    close(f(0.5), 10.0, "t=0.5");
    close(f(0.75), 55.0, "t=0.75");
    close(f(1.0), 100.0, "t=1");
}

#[test]
fn piecewise_custom_midpoint_interpolator() {
    // Custom per-segment function that ignores `t` and returns the midpoint.
    let factory = interpolator_piecewise(|a: f64, b: f64, _t: f64| (a + b) * 0.5);
    let f = factory(&[0.0, 10.0, 100.0]);
    close(f(0.25), 5.0, "seg0 midpoint");
    close(f(0.75), 55.0, "seg1 midpoint");
}

#[test]
fn piecewise_quadratic() {
    // Custom non-linear interpolator.
    let factory = interpolator_piecewise(|a: f64, b: f64, t: f64| a + (b - a) * t * t);
    let f = factory(&[0.0, 10.0]);
    close(f(0.0), 0.0, "t=0");
    close(f(0.5), 2.5, "t=0.5");
    close(f(1.0), 10.0, "t=1");
}

#[test]
fn piecewise_propagates_present_endpoint_when_one_missing() {
    // culori's `get_classes` rule: if exactly one endpoint of a pair is
    // defined, both endpoints in the class become that defined value; if
    // both are undefined, the class is undefined and the result is NaN.
    // Our `f64`-based API uses NaN as the "undefined" marker.
    let factory = interpolator_piecewise(|a: f64, b: f64, t: f64| a + (b - a) * t);
    let f = factory(&[f64::NAN, 10.0, f64::NAN]);
    // Both segments resolve to (10, 10) — the present endpoint propagates.
    close(f(0.0), 10.0, "t=0");
    close(f(0.25), 10.0, "t=0.25");
    close(f(0.5), 10.0, "t=0.5");
    close(f(0.75), 10.0, "t=0.75");
    close(f(1.0), 10.0, "t=1");
}

#[test]
fn piecewise_both_endpoints_missing_produces_nan() {
    let factory = interpolator_piecewise(|a: f64, b: f64, t: f64| a + (b - a) * t);
    let f = factory(&[f64::NAN, f64::NAN]);
    assert!(f(0.0).is_nan());
    assert!(f(0.5).is_nan());
    assert!(f(1.0).is_nan());
}

#[test]
fn piecewise_clamps_t_at_boundaries() {
    // culori's `idx = t >= 1 ? classes.length - 1 : Math.max(Math.floor(cls), 0)`.
    // We mirror that: t outside [0, 1] still resolves to the boundary class.
    let factory = interpolator_piecewise(|a: f64, b: f64, t: f64| a + (b - a) * t);
    let f = factory(&[0.0, 10.0, 100.0]);
    // t = 1 lands on the last class with local_t = 1.
    close(f(1.0), 100.0, "t=1");
}

// Wire-through test: the factory plugs into InterpolateOptions::channel_interpolator
// just like the spline factories do.
#[test]
fn piecewise_channel_interpolator_wires_into_interpolate() {
    use culors::interpolate::ChannelInterpFactory;
    use culors::spaces::Rgb;
    use culors::{interpolate_with, Color, InterpolateOptions};

    // Build a piecewise factory with a custom (non-linear) per-segment fn.
    // It squares t inside each segment, so r at t=0.5 is 0.25 not 0.5.
    let factory: ChannelInterpFactory = Box::new(|stops: &[f64]| {
        let inner = interpolator_piecewise(|a: f64, b: f64, t: f64| a + (b - a) * t * t);
        inner(stops)
    });

    let stops = [
        Color::Rgb(Rgb {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            alpha: None,
        }),
        Color::Rgb(Rgb {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            alpha: None,
        }),
    ];
    let options = InterpolateOptions::default().channel_interpolator("r", factory);
    let f = interpolate_with(&stops, "rgb", options);
    let mid = f(0.5);
    if let Color::Rgb(c) = mid {
        close(c.r, 0.25, "r squared at midpoint");
        // g and b still use the default linear interpolator.
        close(c.g, 0.5, "g linear at midpoint");
        close(c.b, 0.5, "b linear at midpoint");
    } else {
        panic!("expected Rgb");
    }
}
