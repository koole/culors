//! Ported tests for XYB (JPEG XL).

use culor::spaces::{Rgb, Xyb};
use culor::ColorSpace;

#[path = "common/mod.rs"]
mod common;

const EPS: f64 = 1e-12;

#[test]
fn xyb_metadata() {
    assert_eq!(Xyb::CHANNELS, &["x", "y", "b"]);
    assert_eq!(Xyb::MODE, "xyb");
}

#[test]
fn xyb_red() {
    let c: Xyb = Rgb {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        alpha: None,
    }
    .into();
    common::assert_close(c.x, 0.028100083161277323, EPS);
    common::assert_close(c.y, 0.4881882010413151, EPS);
    common::assert_close(c.b, -0.01652922538774071, EPS);
}

#[test]
fn xyb_green() {
    let c: Xyb = Rgb {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        alpha: None,
    }
    .into();
    common::assert_close(c.x, -0.015386116472573375, EPS);
    common::assert_close(c.y, 0.714781372724691, EPS);
    common::assert_close(c.b, -0.2777046155146864, EPS);
}

#[test]
fn xyb_blue() {
    let c: Xyb = Rgb {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        alpha: None,
    }
    .into();
    common::assert_close(c.x, 0.0, 1e-15);
    common::assert_close(c.y, 0.27812819734781813, EPS);
    common::assert_close(c.b, 0.3880116647837879, EPS);
}

#[test]
fn xyb_gray() {
    let c: Xyb = Rgb {
        r: 0.5,
        g: 0.5,
        b: 0.5,
        alpha: None,
    }
    .into();
    common::assert_close(c.x, 0.0, 1e-15);
    common::assert_close(c.y, 0.4457393607565907, EPS);
    common::assert_close(c.b, 0.0, 1e-15);
}

#[test]
fn xyb_arbitrary() {
    let c: Xyb = Rgb {
        r: 0.25,
        g: 0.4,
        b: 0.75,
        alpha: None,
    }
    .into();
    common::assert_close(c.x, -0.0034608904370407867, EPS);
    common::assert_close(c.y, 0.36977571554842403, EPS);
    common::assert_close(c.b, 0.16650892456612276, EPS);
}

#[test]
fn xyb_round_trip() {
    let original = Rgb {
        r: 0.3,
        g: 0.6,
        b: 0.9,
        alpha: Some(0.7),
    };
    let xyb: Xyb = original.into();
    let back: Rgb = xyb.into();
    common::assert_close(back.r, 0.3, 1e-9);
    common::assert_close(back.g, 0.6, 1e-9);
    common::assert_close(back.b, 0.9, 1e-9);
    assert_eq!(back.alpha, Some(0.7));
}
