//! Ported tests for OkHSV.

use culors::spaces::{Okhsv, Oklab, Rgb};
use culors::ColorSpace;

#[path = "common/mod.rs"]
mod common;

const EPS: f64 = 1e-9;

#[test]
fn okhsv_metadata() {
    assert_eq!(Okhsv::CHANNELS, &["h", "s", "v"]);
    assert_eq!(Okhsv::MODE, "okhsv");
}

#[test]
fn okhsv_red() {
    let c: Okhsv = Rgb {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        alpha: None,
    }
    .into();
    common::assert_close(c.s, 0.9995219665357181, EPS);
    common::assert_close(c.v, 0.9999999999999998, EPS);
    common::assert_close(c.h, 29.233880279627854, EPS);
}

#[test]
fn okhsv_green() {
    let c: Okhsv = Rgb {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        alpha: None,
    }
    .into();
    common::assert_close(c.s, 0.9999998946102391, 1e-7);
    common::assert_close(c.v, 1.0, EPS);
    common::assert_close(c.h, 142.4953450414438, EPS);
}

#[test]
fn okhsv_blue() {
    let c: Okhsv = Rgb {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        alpha: None,
    }
    .into();
    common::assert_close(c.s, 0.9999911198554942, EPS);
    common::assert_close(c.v, 1.0000000000000002, EPS);
    common::assert_close(c.h, 264.05202261636987, EPS);
}

#[test]
fn okhsv_gray() {
    let c: Okhsv = Rgb {
        r: 0.5,
        g: 0.5,
        b: 0.5,
        alpha: None,
    }
    .into();
    assert_eq!(c.s, 0.0);
    common::assert_close(c.v, 0.5337598273037281, EPS);
    assert!(c.h.is_nan());
}

#[test]
fn okhsv_arbitrary() {
    let c: Okhsv = Rgb {
        r: 0.25,
        g: 0.4,
        b: 0.75,
        alpha: None,
    }
    .into();
    common::assert_close(c.s, 0.6342731124829709, EPS);
    common::assert_close(c.v, 0.7600945507088643, EPS);
    common::assert_close(c.h, 264.4895037760846, EPS);
}

#[test]
fn okhsv_round_trip_via_oklab() {
    let original = Okhsv {
        h: 200.0,
        s: 0.5,
        v: 0.7,
        alpha: Some(0.6),
    };
    let lab: Oklab = original.into();
    let back: Okhsv = lab.into();
    common::assert_close(back.h, 200.0, 1e-7);
    common::assert_close(back.s, 0.5, 1e-7);
    common::assert_close(back.v, 0.7, 1e-9);
    assert_eq!(back.alpha, Some(0.6));
}
