//! Ported tests for [`culors::lerp`] / [`culors::unlerp`] /
//! [`culors::blerp`] / [`culors::trilerp`].
//!
//! Spot checks taken from culori 4.0.2 via `node -e`:
//!
//! ```js
//! lerp(2, 8, 0.25) === 3.5
//! unlerp(2, 8, 4)  === 0.3333333333333333
//! blerp(0, 10, 20, 30, 0.25, 0.75) === 17.5
//! trilerp(0,1,2,3,4,5,6,7, 0.25, 0.5, 0.75) === 4.25
//! ```

use culors::{blerp, lerp, trilerp, unlerp};

const EPS: f64 = 1e-15;

#[track_caller]
fn close(label: &str, got: f64, expected: f64) {
    let d = (got - expected).abs();
    assert!(d < EPS, "{label}: got {got}, expected {expected}, |Δ|={d}");
}

#[test]
fn lerp_endpoints_and_midpoint() {
    close("lerp(2,8,0)", lerp(2.0, 8.0, 0.0), 2.0);
    close("lerp(2,8,1)", lerp(2.0, 8.0, 1.0), 8.0);
    close("lerp(2,8,0.5)", lerp(2.0, 8.0, 0.5), 5.0);
    close("lerp(2,8,0.25)", lerp(2.0, 8.0, 0.25), 3.5);
    close("lerp(0,1,0.7)", lerp(0.0, 1.0, 0.7), 0.7);
}

#[test]
fn unlerp_inverts_lerp() {
    close("unlerp(2,8,5)", unlerp(2.0, 8.0, 5.0), 0.5);
    close(
        "unlerp(2,8,4)",
        unlerp(2.0, 8.0, 4.0),
        0.333_333_333_333_333_3,
    );
    close("unlerp(0,1,0.7)", unlerp(0.0, 1.0, 0.7), 0.7);
    // Round-trip property.
    for t in [0.1, 0.3, 0.5, 0.9] {
        close(
            "round-trip",
            unlerp(-3.0, 7.0, lerp(-3.0, 7.0, t)),
            t,
        );
    }
}

#[test]
fn unlerp_collapsed_range_is_nan() {
    let v = unlerp(5.0, 5.0, 5.0);
    assert!(v.is_nan(), "unlerp on a collapsed range must be NaN, got {v}");
}

#[test]
fn blerp_corners() {
    // a00=a01=0 → tx-row → 0; a10=a11=1 → tx-row → 1; ty interpolates.
    close("blerp corners ty=0", blerp(0.0, 0.0, 1.0, 1.0, 0.5, 0.0), 0.0);
    close("blerp corners ty=1", blerp(0.0, 0.0, 1.0, 1.0, 0.5, 1.0), 1.0);
}

#[test]
fn blerp_spot_checks() {
    close("blerp center", blerp(0.0, 10.0, 20.0, 30.0, 0.5, 0.5), 15.0);
    close(
        "blerp(0,10,20,30,0.25,0.75)",
        blerp(0.0, 10.0, 20.0, 30.0, 0.25, 0.75),
        17.5,
    );
}

#[test]
fn trilerp_center() {
    // 8 unit-cube corners 0..7 averaged at the center is 3.5.
    close(
        "trilerp center",
        trilerp(
            0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 0.5, 0.5, 0.5,
        ),
        3.5,
    );
}

#[test]
fn trilerp_spot_check() {
    close(
        "trilerp(0..7, 0.25, 0.5, 0.75)",
        trilerp(
            0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 0.25, 0.5, 0.75,
        ),
        4.25,
    );
}
