//! Ported tests for DIN99o LCh.

use culor::spaces::{Dlch, Rgb};
use culor::ColorSpace;

#[path = "common/mod.rs"]
mod common;

const EPS: f64 = 1e-10;

#[test]
fn dlch_metadata() {
    assert_eq!(Dlch::CHANNELS, &["l", "c", "h"]);
    assert_eq!(Dlch::MODE, "dlch");
}

#[test]
fn dlch_red() {
    let rgb = Rgb {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        alpha: None,
    };
    let d: Dlch = rgb.into();
    common::assert_close(d.l, 57.28917941426676, EPS);
    common::assert_close(d.c, 49.914581534832, EPS);
    common::assert_close(d.h, 37.691765574369924, EPS);
}

#[test]
fn dlch_green() {
    let rgb = Rgb {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        alpha: None,
    };
    let d: Dlch = rgb.into();
    common::assert_close(d.l, 89.36631009006445, EPS);
    common::assert_close(d.c, 49.60162444138398, EPS);
    common::assert_close(d.h, 139.69328788328542, EPS);
}

#[test]
fn dlch_blue() {
    let rgb = Rgb {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        alpha: None,
    };
    let d: Dlch = rgb.into();
    common::assert_close(d.l, 36.029930076331965, EPS);
    common::assert_close(d.c, 51.48452910485975, EPS);
    common::assert_close(d.h, 308.33687592694776, EPS);
}

#[test]
fn dlch_arbitrary() {
    let rgb = Rgb {
        r: 0.25,
        g: 0.4,
        b: 0.75,
        alpha: None,
    };
    let d: Dlch = rgb.into();
    common::assert_close(d.l, 48.83929889376355, EPS);
    common::assert_close(d.c, 33.73641100159493, EPS);
    common::assert_close(d.h, 285.55735915839347, EPS);
}

#[test]
fn dlch_gray_hue_nan() {
    let rgb = Rgb {
        r: 0.5,
        g: 0.5,
        b: 0.5,
        alpha: None,
    };
    let d: Dlch = rgb.into();
    common::assert_close(d.l, 57.43806128457824, EPS);
    common::assert_close(d.c, 0.0, EPS);
    assert!(d.h.is_nan());
}

#[test]
fn dlch_round_trip() {
    let original = Dlch {
        l: 50.0,
        c: 27.120345907653437,
        h: 305.240228793963,
        alpha: Some(0.5),
    };
    let xyz = original.to_xyz65();
    let back = Dlch::from_xyz65(xyz);
    common::assert_close(back.l, original.l, 1e-9);
    common::assert_close(back.c, original.c, 1e-9);
    common::assert_close(back.h, original.h, 1e-8);
    assert_eq!(back.alpha, Some(0.5));
}
