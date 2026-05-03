//! Ported tests for the `LinearRgb` (linear-sRGB) color space.
//!
//! Reference values produced by culori 4.0.2 and pasted verbatim.

use culors::spaces::{LinearRgb, Rgb, Xyz65};
use culors::ColorSpace;

#[path = "common/mod.rs"]
mod common;

const EPS: f64 = 1e-12;
// Slightly looser eps for chained sRGB ↔ XYZ conversions that go through both
// the matrix and the transfer function.
const CHAIN_EPS: f64 = 1e-10;

#[test]
fn lrgb_channels_excludes_alpha() {
    assert_eq!(LinearRgb::CHANNELS, &["r", "g", "b"]);
    assert_eq!(LinearRgb::CHANNELS.len(), 3);
    assert_eq!(LinearRgb::MODE, "lrgb");
}

#[test]
fn lrgb_white_to_xyz65() {
    let white = LinearRgb {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        alpha: None,
    };
    let xyz = white.to_xyz65();
    common::assert_close(xyz.x, 0.9504559270516715, EPS);
    common::assert_close(xyz.y, 0.9999999999999999, EPS);
    common::assert_close(xyz.z, 1.0890577507598784, EPS);
    assert_eq!(xyz.alpha, None);
}

#[test]
fn lrgb_black_to_xyz65() {
    let black = LinearRgb {
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
fn lrgb_red_to_xyz65() {
    let red = LinearRgb {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        alpha: None,
    };
    let xyz = red.to_xyz65();
    common::assert_close(xyz.x, 0.4123907992659593, EPS);
    common::assert_close(xyz.y, 0.2126390058715102, EPS);
    common::assert_close(xyz.z, 0.0193308187155918, EPS);
}

#[test]
fn lrgb_mid_to_xyz65() {
    let mid = LinearRgb {
        r: 0.5,
        g: 0.5,
        b: 0.5,
        alpha: None,
    };
    let xyz = mid.to_xyz65();
    common::assert_close(xyz.x, 0.47522796352583574, EPS);
    common::assert_close(xyz.y, 0.49999999999999994, EPS);
    common::assert_close(xyz.z, 0.5445288753799392, EPS);
}

#[test]
fn lrgb_round_trip_through_xyz65() {
    let c = LinearRgb {
        r: 0.25,
        g: 0.4,
        b: 0.75,
        alpha: None,
    };
    let back = LinearRgb::from_xyz65(c.to_xyz65());
    common::assert_close(back.r, 0.25, EPS);
    common::assert_close(back.g, 0.4, EPS);
    common::assert_close(back.b, 0.75, EPS);
}

#[test]
fn lrgb_alpha_preserved() {
    let c = LinearRgb {
        r: 0.5,
        g: 0.5,
        b: 0.5,
        alpha: Some(0.7),
    };
    let xyz = c.to_xyz65();
    assert_eq!(xyz.alpha, Some(0.7));
    let back = LinearRgb::from_xyz65(xyz);
    assert_eq!(back.alpha, Some(0.7));
}

#[test]
fn rgb_to_lrgb_direct_white() {
    let rgb = Rgb {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        alpha: None,
    };
    let lrgb: LinearRgb = rgb.into();
    common::assert_close(lrgb.r, 1.0, EPS);
    common::assert_close(lrgb.g, 1.0, EPS);
    common::assert_close(lrgb.b, 1.0, EPS);
}

#[test]
fn rgb_to_lrgb_direct_mid() {
    let rgb = Rgb {
        r: 0.5,
        g: 0.5,
        b: 0.5,
        alpha: None,
    };
    let lrgb: LinearRgb = rgb.into();
    common::assert_close(lrgb.r, 0.21404114048223255, EPS);
    common::assert_close(lrgb.g, 0.21404114048223255, EPS);
    common::assert_close(lrgb.b, 0.21404114048223255, EPS);
}

#[test]
fn rgb_to_lrgb_threshold() {
    // 0.04045 sits on the piecewise boundary; below it the linear branch
    // applies, on it the linear branch still applies (≤ in culori).
    let rgb = Rgb {
        r: 0.04045,
        g: 0.04045,
        b: 0.04045,
        alpha: None,
    };
    let lrgb: LinearRgb = rgb.into();
    common::assert_close(lrgb.r, 0.0031308049535603713, EPS);
    common::assert_close(lrgb.g, 0.0031308049535603713, EPS);
    common::assert_close(lrgb.b, 0.0031308049535603713, EPS);
}

#[test]
fn lrgb_to_rgb_direct_negative() {
    // sRGB transfer is sign-preserving, matching culori.
    let lrgb = LinearRgb {
        r: -0.1,
        g: 0.5,
        b: 0.2,
        alpha: None,
    };
    let rgb: Rgb = lrgb.into();
    common::assert_close(rgb.r, -0.3491902126282938, EPS);
    common::assert_close(rgb.g, 0.7353569830524495, EPS);
    common::assert_close(rgb.b, 0.48452920448170694, EPS);
}

#[test]
fn rgb_lrgb_direct_alpha_preserved() {
    let rgb = Rgb {
        r: 0.5,
        g: 0.5,
        b: 0.5,
        alpha: Some(0.7),
    };
    let lrgb: LinearRgb = rgb.into();
    assert_eq!(lrgb.alpha, Some(0.7));
    let back: Rgb = lrgb.into();
    assert_eq!(back.alpha, Some(0.7));
    common::assert_close(back.r, 0.5, CHAIN_EPS);
}

#[test]
fn xyz65_to_lrgb_white() {
    let xyz = Xyz65 {
        x: 0.9504559270516715,
        y: 0.9999999999999999,
        z: 1.0890577507598784,
        alpha: None,
    };
    let lrgb = LinearRgb::from_xyz65(xyz);
    common::assert_close(lrgb.r, 1.0, CHAIN_EPS);
    common::assert_close(lrgb.g, 1.0, CHAIN_EPS);
    common::assert_close(lrgb.b, 0.9999999999999994, EPS);
}
