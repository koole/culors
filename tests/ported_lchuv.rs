//! Ported tests for CIELChuv.

use culors::spaces::{Lchuv, Rgb};
use culors::ColorSpace;

#[path = "common/mod.rs"]
mod common;

const EPS: f64 = 1e-9;

#[test]
fn lchuv_metadata() {
    assert_eq!(Lchuv::CHANNELS, &["l", "c", "h"]);
    assert_eq!(Lchuv::MODE, "lchuv");
}

#[test]
fn lchuv_red() {
    let c: Lchuv = Rgb {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        alpha: None,
    }
    .into();
    common::assert_close(c.l, 54.29054294696968, EPS);
    common::assert_close(c.c, 176.94953872495253, EPS);
    common::assert_close(c.h, 8.434231142939021, EPS);
}

#[test]
fn lchuv_green() {
    let c: Lchuv = Rgb {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        alpha: None,
    }
    .into();
    common::assert_close(c.l, 87.81853633115202, EPS);
    common::assert_close(c.c, 121.74810673058624, EPS);
    common::assert_close(c.h, 134.23124010020928, EPS);
}

#[test]
fn lchuv_blue() {
    let c: Lchuv = Rgb {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        alpha: None,
    }
    .into();
    common::assert_close(c.l, 29.568297153444703, EPS);
    common::assert_close(c.c, 122.51228078315859, EPS);
    common::assert_close(c.h, 264.5930151325516, EPS);
}

#[test]
fn lchuv_arbitrary() {
    let c: Lchuv = Rgb {
        r: 0.25,
        g: 0.4,
        b: 0.75,
        alpha: None,
    }
    .into();
    common::assert_close(c.l, 43.9803743461382, EPS);
    common::assert_close(c.c, 71.41347593763165, EPS);
    common::assert_close(c.h, 254.20755238367497, EPS);
}

#[test]
fn lchuv_gray_hue_nan() {
    let c: Lchuv = Rgb {
        r: 0.5,
        g: 0.5,
        b: 0.5,
        alpha: None,
    }
    .into();
    common::assert_close(c.l, 53.388965576471506, EPS);
    assert_eq!(c.c, 0.0);
    assert!(c.h.is_nan());
}

#[test]
fn lchuv_round_trip() {
    let original = Lchuv {
        l: 50.0,
        c: 30.0,
        h: 200.0,
        alpha: Some(0.5),
    };
    let xyz = original.to_xyz65();
    let back = Lchuv::from_xyz65(xyz);
    common::assert_close(back.l, original.l, 1e-5);
    common::assert_close(back.c, original.c, 1e-5);
    common::assert_close(back.h, original.h, 1e-4);
    assert_eq!(back.alpha, Some(0.5));
}
