//! Ported tests for JzCzHz.

use culors::spaces::{Jch, Rgb};
use culors::ColorSpace;

#[path = "common/mod.rs"]
mod common;

const EPS: f64 = 1e-10;

#[test]
fn jch_metadata() {
    assert_eq!(Jch::CHANNELS, &["j", "c", "h"]);
    assert_eq!(Jch::MODE, "jch");
}

#[test]
fn jch_red() {
    let rgb = Rgb {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        alpha: None,
    };
    let j: Jch = rgb.into();
    common::assert_close(j.j, 0.13438473104350068, EPS);
    common::assert_close(j.c, 0.16252275661123236, EPS);
    common::assert_close(j.h, 43.502345460331135, EPS);
}

#[test]
fn jch_green() {
    let rgb = Rgb {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        alpha: None,
    };
    let j: Jch = rgb.into();
    common::assert_close(j.j, 0.17680712813178287, EPS);
    common::assert_close(j.c, 0.16139686507633647, EPS);
    common::assert_close(j.h, 132.50253010398265, EPS);
}

#[test]
fn jch_blue() {
    let rgb = Rgb {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        alpha: None,
    };
    let j: Jch = rgb.into();
    common::assert_close(j.j, 0.09577429215304557, EPS);
    common::assert_close(j.c, 0.1902896996290237, EPS);
    common::assert_close(j.h, 257.60506354312355, EPS);
}

#[test]
fn jch_arbitrary() {
    let rgb = Rgb {
        r: 0.25,
        g: 0.4,
        b: 0.75,
        alpha: None,
    };
    let j: Jch = rgb.into();
    common::assert_close(j.j, 0.10398883320693651, EPS);
    common::assert_close(j.c, 0.09959140685976327, EPS);
    common::assert_close(j.h, 257.8208212032284, EPS);
}

#[test]
fn jch_gray_hue_nan() {
    let rgb = Rgb {
        r: 0.5,
        g: 0.5,
        b: 0.5,
        alpha: None,
    };
    let j: Jch = rgb.into();
    common::assert_close(j.j, 0.11783950641678198, EPS);
    assert_eq!(j.c, 0.0);
    assert!(j.h.is_nan());
}

#[test]
fn jch_round_trip_via_xyz() {
    let original = Jch {
        j: 0.13,
        c: 0.05,
        h: 200.0,
        alpha: Some(0.5),
    };
    let xyz = original.to_xyz65();
    let back = Jch::from_xyz65(xyz);
    common::assert_close(back.j, original.j, 1e-7);
    common::assert_close(back.c, original.c, 1e-7);
    common::assert_close(back.h, original.h, 1e-5);
    assert_eq!(back.alpha, Some(0.5));
}

/// PQ_inv negative-value regression. culori's `convertJabToXyz65.js`
/// guards `jabPqDecode(v)` with `if (v < 0) return 0` because the
/// inverse PQ formula contains `(C1 - vp) / (C3 * vp - C2)` raised to a
/// fractional power; without the clamp a negative `v` (which arises
/// naturally for `j < 0` Jab/Jch inputs) yields NaN through `Math.pow`.
///
/// This test feeds a negative `j` Jch through the XYZ65 hub and asserts
/// the output is the all-zero XYZ tristimulus — the exact value culori
/// 4.0.2 produces — rather than NaN.
#[test]
fn jch_pq_inv_negative_value_regression() {
    let j = Jch {
        j: -0.05,
        c: 0.05,
        h: 200.0,
        alpha: None,
    };
    let xyz = j.to_xyz65();
    assert!(
        !xyz.x.is_nan() && !xyz.y.is_nan() && !xyz.z.is_nan(),
        "PQ_inv produced NaN for negative input: x={}, y={}, z={}",
        xyz.x,
        xyz.y,
        xyz.z
    );
    common::assert_close(xyz.x, 0.0, 1e-15);
    common::assert_close(xyz.y, 0.0, 1e-15);
    common::assert_close(xyz.z, 0.0, 1e-15);
}

/// Variant of the regression with both negative `j` and a non-trivial
/// chroma. Without the clamp the LMS triple goes through PQ_inv with
/// negative arguments on each channel; the all-zero output still holds
/// because each PQ_inv component clamps to 0 independently.
#[test]
fn jch_pq_inv_negative_with_chroma() {
    let j = Jch {
        j: -0.1,
        c: 0.15,
        h: 30.0,
        alpha: None,
    };
    let xyz = j.to_xyz65();
    assert!(!xyz.x.is_nan() && !xyz.y.is_nan() && !xyz.z.is_nan());
    common::assert_close(xyz.x, 0.0, 1e-15);
    common::assert_close(xyz.y, 0.0, 1e-15);
    common::assert_close(xyz.z, 0.0, 1e-15);
}
