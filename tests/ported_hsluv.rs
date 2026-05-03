//! Ported tests for HSLuv. Reference values from the official
//! `hsluv-javascript` package (v1.0.1).

use culors::spaces::{Hsluv, Rgb};
use culors::ColorSpace;

#[path = "common/mod.rs"]
mod common;

const EPS: f64 = 1e-9;

#[test]
fn hsluv_metadata() {
    assert_eq!(Hsluv::CHANNELS, &["h", "s", "l"]);
    assert_eq!(Hsluv::MODE, "hsluv");
}

#[test]
fn hsluv_red() {
    let c: Hsluv = Rgb {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        alpha: None,
    }
    .into();
    common::assert_close(c.h, 12.177050630061776, EPS);
    common::assert_close(c.s, 100.0, 1e-7);
    common::assert_close(c.l, 53.23711559542933, EPS);
}

#[test]
fn hsluv_green() {
    let c: Hsluv = Rgb {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        alpha: None,
    }
    .into();
    common::assert_close(c.h, 127.71501294924047, EPS);
    common::assert_close(c.s, 100.0, 1e-7);
    common::assert_close(c.l, 87.73551910965973, EPS);
}

#[test]
fn hsluv_blue() {
    let c: Hsluv = Rgb {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        alpha: None,
    }
    .into();
    common::assert_close(c.h, 265.8743202181779, EPS);
    common::assert_close(c.s, 100.0, 1e-7);
    common::assert_close(c.l, 32.30087290398002, EPS);
}

#[test]
fn hsluv_gray() {
    let c: Hsluv = Rgb {
        r: 0.5,
        g: 0.5,
        b: 0.5,
        alpha: None,
    }
    .into();
    common::assert_close(c.h, 0.0, EPS);
    common::assert_close(c.s, 1.9030441739252076e-12, 1e-9);
    common::assert_close(c.l, 53.38896474111415, EPS);
}

#[test]
fn hsluv_arbitrary() {
    let c: Hsluv = Rgb {
        r: 0.25,
        g: 0.4,
        b: 0.75,
        alpha: None,
    }
    .into();
    common::assert_close(c.h, 257.7327212716839, EPS);
    common::assert_close(c.s, 74.64906962004846, 1e-9);
    common::assert_close(c.l, 44.73970976823455, EPS);
}

#[test]
fn hsluv_to_rgb() {
    let rgb: Rgb = Hsluv {
        h: 200.0,
        s: 50.0,
        l: 40.0,
        alpha: None,
    }
    .into();
    common::assert_close(rgb.r, 0.26610158702738623, EPS);
    common::assert_close(rgb.g, 0.39100431850077594, EPS);
    common::assert_close(rgb.b, 0.40199149495256387, EPS);
}

#[test]
fn hsluv_round_trip_via_rgb() {
    let original = Hsluv {
        h: 100.0,
        s: 60.0,
        l: 50.0,
        alpha: Some(0.5),
    };
    let rgb: Rgb = original.into();
    let back: Hsluv = rgb.into();
    common::assert_close(back.h, 100.0, 1e-7);
    common::assert_close(back.s, 60.0, 1e-7);
    common::assert_close(back.l, 50.0, 1e-9);
    assert_eq!(back.alpha, Some(0.5));
}
