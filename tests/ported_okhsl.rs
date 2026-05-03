//! Ported tests for OkHSL.

use culors::spaces::{Okhsl, Rgb};
use culors::ColorSpace;

#[path = "common/mod.rs"]
mod common;

const EPS: f64 = 1e-10;

#[test]
fn okhsl_metadata() {
    assert_eq!(Okhsl::CHANNELS, &["h", "s", "l"]);
    assert_eq!(Okhsl::MODE, "okhsl");
}

#[test]
fn okhsl_red() {
    let c: Okhsl = Rgb {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        alpha: None,
    }
    .into();
    common::assert_close(c.l, 0.5680846563197034, EPS);
    common::assert_close(c.s, 1.0000000007111225, 1e-9);
    common::assert_close(c.h, 29.233880279627854, 1e-9);
}

#[test]
fn okhsl_green() {
    let c: Okhsl = Rgb {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        alpha: None,
    }
    .into();
    common::assert_close(c.l, 0.8445289714936317, EPS);
    common::assert_close(c.s, 1.0000000130542381, 1e-7);
    common::assert_close(c.h, 142.4953450414438, 1e-9);
}

#[test]
fn okhsl_blue() {
    let c: Okhsl = Rgb {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        alpha: None,
    }
    .into();
    common::assert_close(c.l, 0.3665653391870817, EPS);
    common::assert_close(c.s, 1.000000000269043, 1e-9);
    common::assert_close(c.h, 264.05202261636987, 1e-9);
}

#[test]
fn okhsl_gray_hue_nan() {
    let c: Okhsl = Rgb {
        r: 0.5,
        g: 0.5,
        b: 0.5,
        alpha: None,
    }
    .into();
    common::assert_close(c.l, 0.5337598273037278, EPS);
    assert_eq!(c.s, 0.0);
    assert!(c.h.is_nan());
}

#[test]
fn okhsl_arbitrary() {
    let c: Okhsl = Rgb {
        r: 0.25,
        g: 0.4,
        b: 0.75,
        alpha: None,
    }
    .into();
    common::assert_close(c.l, 0.4543696222543252, EPS);
    common::assert_close(c.s, 0.7276779909839312, 1e-9);
    common::assert_close(c.h, 264.4895037760846, 1e-9);
}

#[test]
fn okhsl_round_trip_via_oklab() {
    use culors::spaces::Oklab;
    let original = Okhsl {
        h: 200.0,
        s: 0.5,
        l: 0.6,
        alpha: Some(0.7),
    };
    let lab: Oklab = original.into();
    let back: Okhsl = lab.into();
    common::assert_close(back.h, 200.0, 1e-7);
    common::assert_close(back.s, 0.5, 1e-7);
    common::assert_close(back.l, 0.6, 1e-9);
    assert_eq!(back.alpha, Some(0.7));
}
