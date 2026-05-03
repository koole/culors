//! Ported tests for JzCzHz.

use culor::spaces::{Jch, Rgb};
use culor::ColorSpace;

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
