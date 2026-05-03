//! Ported tests for HPLuv. Reference values from the official
//! `hsluv-javascript` package (v1.0.1).

use culors::spaces::{Hpluv, Rgb};
use culors::ColorSpace;

#[path = "common/mod.rs"]
mod common;

const EPS: f64 = 1e-9;

#[test]
fn hpluv_metadata() {
    assert_eq!(Hpluv::CHANNELS, &["h", "s", "l"]);
    assert_eq!(Hpluv::MODE, "hpluv");
}

#[test]
fn hpluv_red_pastel_overflow() {
    let c: Hpluv = Rgb {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        alpha: None,
    }
    .into();
    common::assert_close(c.h, 12.177050630061776, EPS);
    common::assert_close(c.s, 426.7467891831252, 1e-7);
    common::assert_close(c.l, 53.23711559542933, EPS);
}

#[test]
fn hpluv_blue() {
    let c: Hpluv = Rgb {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        alpha: None,
    }
    .into();
    common::assert_close(c.h, 265.8743202181779, EPS);
    common::assert_close(c.s, 513.4126968442804, 1e-7);
    common::assert_close(c.l, 32.30087290398002, EPS);
}

#[test]
fn hpluv_arbitrary() {
    let c: Hpluv = Rgb {
        r: 0.25,
        g: 0.4,
        b: 0.75,
        alpha: None,
    }
    .into();
    common::assert_close(c.h, 257.7327212716839, EPS);
    common::assert_close(c.s, 225.036631106538, 1e-7);
    common::assert_close(c.l, 44.73970976823455, EPS);
}

#[test]
fn hpluv_pastel_in_range() {
    // Light pastel blue — saturation in the inscribed-circle range.
    let c: Hpluv = Rgb {
        r: 0.7,
        g: 0.7,
        b: 0.8,
        alpha: None,
    }
    .into();
    common::assert_close(c.h, 265.8743202181871, 1e-7);
    common::assert_close(c.s, 35.591065589799186, 1e-7);
    common::assert_close(c.l, 73.49611348219773, EPS);
}

#[test]
fn hpluv_gray() {
    let c: Hpluv = Rgb {
        r: 0.5,
        g: 0.5,
        b: 0.5,
        alpha: None,
    }
    .into();
    common::assert_close(c.h, 0.0, EPS);
    common::assert_close(c.s, 6.646547317414331e-12, 1e-9);
    common::assert_close(c.l, 53.38896474111415, EPS);
}

#[test]
fn hpluv_round_trip_via_rgb() {
    let original = Hpluv {
        h: 100.0,
        s: 30.0,
        l: 70.0,
        alpha: Some(0.4),
    };
    let rgb: Rgb = original.into();
    let back: Hpluv = rgb.into();
    common::assert_close(back.h, 100.0, 1e-7);
    common::assert_close(back.s, 30.0, 1e-7);
    common::assert_close(back.l, 70.0, 1e-9);
    assert_eq!(back.alpha, Some(0.4));
}
