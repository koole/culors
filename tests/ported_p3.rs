//! Ported tests for the Display P3 color space.
//!
//! Reference values produced by culori 4.0.2:
//! `c.xyz65({mode:'p3', r, g, b})`.

use culor::spaces::{Xyz65, P3};
use culor::ColorSpace;

#[path = "common/mod.rs"]
mod common;

const EPS: f64 = 1e-12;

#[test]
fn p3_metadata() {
    assert_eq!(P3::CHANNELS, &["r", "g", "b"]);
    assert_eq!(P3::MODE, "p3");
}

#[test]
fn p3_white_to_xyz65() {
    let white = P3 {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        alpha: None,
    };
    let xyz = white.to_xyz65();
    common::assert_close(xyz.x, 0.9504559270516715, EPS);
    common::assert_close(xyz.y, 0.9999999999999999, EPS);
    common::assert_close(xyz.z, 1.0890577507598787, EPS);
}

#[test]
fn p3_black_to_xyz65() {
    let black = P3 {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        alpha: None,
    };
    let xyz = black.to_xyz65();
    common::assert_close(xyz.x, 0.0, 1e-15);
    common::assert_close(xyz.y, 0.0, 1e-15);
    common::assert_close(xyz.z, 0.0, 1e-15);
}

#[test]
fn p3_red_primary() {
    let red = P3 {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        alpha: None,
    };
    let xyz = red.to_xyz65();
    common::assert_close(xyz.x, 0.486570948648216, EPS);
    common::assert_close(xyz.y, 0.2289745640697487, EPS);
    common::assert_close(xyz.z, 0.0, 1e-15);
}

#[test]
fn p3_green_primary() {
    let green = P3 {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        alpha: None,
    };
    let xyz = green.to_xyz65();
    common::assert_close(xyz.x, 0.265667693169093, EPS);
    common::assert_close(xyz.y, 0.6917385218365062, EPS);
    common::assert_close(xyz.z, 0.0451133818589026, EPS);
}

#[test]
fn p3_blue_primary() {
    let blue = P3 {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        alpha: None,
    };
    let xyz = blue.to_xyz65();
    common::assert_close(xyz.x, 0.1982172852343625, EPS);
    common::assert_close(xyz.y, 0.079286914093745, EPS);
    common::assert_close(xyz.z, 1.043944368900976, EPS);
}

#[test]
fn p3_mid_gray() {
    let c = P3 {
        r: 0.5,
        g: 0.5,
        b: 0.5,
        alpha: None,
    };
    let xyz = c.to_xyz65();
    common::assert_close(xyz.x, 0.2034366706042374, EPS);
    common::assert_close(xyz.y, 0.21404114048223252, EPS);
    common::assert_close(xyz.z, 0.23310316302365938, EPS);
}

#[test]
fn p3_arbitrary_value_matches_culori() {
    // c.xyz65({mode:'p3', r:0.25, g:0.4, b:0.75})
    let c = P3 {
        r: 0.25,
        g: 0.4,
        b: 0.75,
        alpha: None,
    };
    let xyz = c.to_xyz65();
    common::assert_close(xyz.x, 0.16362645087165187, EPS);
    common::assert_close(xyz.y, 0.144988588022814, EPS);
    common::assert_close(xyz.z, 0.5514775732218992, EPS);
}

#[test]
fn p3_round_trip_through_xyz65() {
    let c = P3 {
        r: 0.25,
        g: 0.4,
        b: 0.75,
        alpha: None,
    };
    let back = P3::from_xyz65(c.to_xyz65());
    common::assert_close(back.r, 0.25, 1e-10);
    common::assert_close(back.g, 0.4, 1e-10);
    common::assert_close(back.b, 0.75, 1e-10);
}

#[test]
fn p3_alpha_preserved() {
    let c = P3 {
        r: 0.5,
        g: 0.5,
        b: 0.5,
        alpha: Some(0.7),
    };
    let xyz = c.to_xyz65();
    assert_eq!(xyz.alpha, Some(0.7));
    let back = P3::from_xyz65(xyz);
    assert_eq!(back.alpha, Some(0.7));
}

#[test]
fn xyz65_to_p3_white() {
    let xyz = Xyz65 {
        x: 0.9504559270516715,
        y: 1.0,
        z: 1.0890577507598787,
        alpha: None,
    };
    let p3 = P3::from_xyz65(xyz);
    common::assert_close(p3.r, 1.0, 1e-10);
    common::assert_close(p3.g, 1.0, 1e-10);
    common::assert_close(p3.b, 1.0, 1e-10);
}
