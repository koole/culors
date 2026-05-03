//! Ported tests for CIELUV.

use culors::spaces::{Luv, Rgb};
use culors::ColorSpace;

#[path = "common/mod.rs"]
mod common;

const EPS: f64 = 1e-9;

#[test]
fn luv_metadata() {
    assert_eq!(Luv::CHANNELS, &["l", "u", "v"]);
    assert_eq!(Luv::MODE, "luv");
}

#[test]
fn luv_red() {
    let c: Luv = Rgb {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        alpha: None,
    }
    .into();
    common::assert_close(c.l, 54.29054294696968, EPS);
    common::assert_close(c.u, 175.03580817106865, EPS);
    common::assert_close(c.v, 25.95390361533953, EPS);
}

#[test]
fn luv_green() {
    let c: Luv = Rgb {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        alpha: None,
    }
    .into();
    common::assert_close(c.l, 87.81853633115202, EPS);
    common::assert_close(c.u, -84.92610887494735, EPS);
    common::assert_close(c.v, 87.2362168129889, EPS);
}

#[test]
fn luv_blue() {
    let c: Luv = Rgb {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        alpha: None,
    }
    .into();
    common::assert_close(c.l, 29.568297153444703, EPS);
    common::assert_close(c.u, -11.544293077670499, EPS);
    common::assert_close(c.v, -121.96716049834208, EPS);
}

#[test]
fn luv_arbitrary() {
    let c: Luv = Rgb {
        r: 0.25,
        g: 0.4,
        b: 0.75,
        alpha: None,
    }
    .into();
    common::assert_close(c.l, 43.9803743461382, EPS);
    common::assert_close(c.u, -19.435421061362963, EPS);
    common::assert_close(c.v, -68.71789398447993, EPS);
}

#[test]
fn luv_zero() {
    let c: Luv = Rgb {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        alpha: None,
    }
    .into();
    assert_eq!(c.l, 0.0);
    assert_eq!(c.u, 0.0);
    assert_eq!(c.v, 0.0);
}

#[test]
fn luv_round_trip() {
    let original = Luv {
        l: 50.0,
        u: 20.0,
        v: -30.0,
        alpha: Some(0.6),
    };
    let xyz = original.to_xyz65();
    let back = Luv::from_xyz65(xyz);
    // Round-trip through XYZ65 hub (Bradford ↔ XYZ50) loses ~1e-6 of
    // precision, similar to Lab.
    common::assert_close(back.l, original.l, 1e-5);
    common::assert_close(back.u, original.u, 1e-5);
    common::assert_close(back.v, original.v, 1e-5);
    assert_eq!(back.alpha, Some(0.6));
}
