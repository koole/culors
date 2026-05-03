//! Ported tests for the `Hsv` color space.
//!
//! Reference values produced by culori 4.0.2 and pasted verbatim.

use culor::spaces::{Hsv, Rgb, Xyz65};
use culor::ColorSpace;

#[path = "common/mod.rs"]
mod common;

const EPS: f64 = 1e-12;
const CHAIN_EPS: f64 = 1e-10;

#[test]
fn hsv_channels_excludes_alpha() {
    assert_eq!(Hsv::CHANNELS, &["h", "s", "v"]);
    assert_eq!(Hsv::CHANNELS.len(), 3);
    assert_eq!(Hsv::MODE, "hsv");
}

#[test]
fn rgb_red_to_hsv() {
    let rgb = Rgb {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        alpha: None,
    };
    let hsv: Hsv = rgb.into();
    common::assert_close(hsv.h, 0.0, EPS);
    common::assert_close(hsv.s, 1.0, EPS);
    common::assert_close(hsv.v, 1.0, EPS);
}

#[test]
fn rgb_green_to_hsv() {
    let rgb = Rgb {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        alpha: None,
    };
    let hsv: Hsv = rgb.into();
    common::assert_close(hsv.h, 120.0, EPS);
    common::assert_close(hsv.s, 1.0, EPS);
    common::assert_close(hsv.v, 1.0, EPS);
}

#[test]
fn rgb_blue_to_hsv() {
    let rgb = Rgb {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        alpha: None,
    };
    let hsv: Hsv = rgb.into();
    common::assert_close(hsv.h, 240.0, EPS);
    common::assert_close(hsv.s, 1.0, EPS);
    common::assert_close(hsv.v, 1.0, EPS);
}

#[test]
fn rgb_orange_to_hsv() {
    let rgb = Rgb {
        r: 1.0,
        g: 0.5,
        b: 0.25,
        alpha: None,
    };
    let hsv: Hsv = rgb.into();
    common::assert_close(hsv.h, 20.0, EPS);
    common::assert_close(hsv.s, 0.75, EPS);
    common::assert_close(hsv.v, 1.0, EPS);
}

#[test]
fn rgb_g_lt_b_wraps_hue() {
    let rgb = Rgb {
        r: 1.0,
        g: 0.0,
        b: 0.5,
        alpha: None,
    };
    let hsv: Hsv = rgb.into();
    common::assert_close(hsv.h, 330.0, EPS);
    common::assert_close(hsv.s, 1.0, EPS);
    common::assert_close(hsv.v, 1.0, EPS);
}

#[test]
fn rgb_grey_produces_nan_hue() {
    let rgb = Rgb {
        r: 0.5,
        g: 0.5,
        b: 0.5,
        alpha: None,
    };
    let hsv: Hsv = rgb.into();
    assert!(hsv.h.is_nan(), "expected NaN hue for grey, got {}", hsv.h);
    common::assert_close(hsv.s, 0.0, EPS);
    common::assert_close(hsv.v, 0.5, EPS);
}

#[test]
fn rgb_black_produces_nan_hue_and_zero_s() {
    let rgb = Rgb {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        alpha: None,
    };
    let hsv: Hsv = rgb.into();
    assert!(hsv.h.is_nan());
    common::assert_close(hsv.s, 0.0, EPS);
    common::assert_close(hsv.v, 0.0, EPS);
}

#[test]
fn hsv_to_rgb_red() {
    let hsv = Hsv {
        h: 0.0,
        s: 1.0,
        v: 1.0,
        alpha: None,
    };
    let rgb: Rgb = hsv.into();
    common::assert_close(rgb.r, 1.0, EPS);
    common::assert_close(rgb.g, 0.0, EPS);
    common::assert_close(rgb.b, 0.0, EPS);
}

#[test]
fn hsv_to_rgb_yellow() {
    let hsv = Hsv {
        h: 60.0,
        s: 1.0,
        v: 1.0,
        alpha: None,
    };
    let rgb: Rgb = hsv.into();
    common::assert_close(rgb.r, 1.0, EPS);
    common::assert_close(rgb.g, 1.0, EPS);
    common::assert_close(rgb.b, 0.0, EPS);
}

#[test]
fn hsv_to_rgb_arbitrary() {
    let hsv = Hsv {
        h: 30.0,
        s: 0.4,
        v: 0.7,
        alpha: None,
    };
    let rgb: Rgb = hsv.into();
    common::assert_close(rgb.r, 0.7, EPS);
    common::assert_close(rgb.g, 0.5599999999999999, EPS);
    common::assert_close(rgb.b, 0.42, EPS);
}

#[test]
fn hsv_negative_hue_normalizes() {
    let hsv = Hsv {
        h: -10.0,
        s: 1.0,
        v: 1.0,
        alpha: None,
    };
    let rgb: Rgb = hsv.into();
    common::assert_close(rgb.r, 1.0, EPS);
    common::assert_close(rgb.g, 0.0, EPS);
    common::assert_close(rgb.b, 0.16666666666666696, EPS);
}

#[test]
fn hsv_360_normalizes_to_zero() {
    let hsv = Hsv {
        h: 360.0,
        s: 1.0,
        v: 1.0,
        alpha: None,
    };
    let rgb: Rgb = hsv.into();
    common::assert_close(rgb.r, 1.0, EPS);
    common::assert_close(rgb.g, 0.0, EPS);
    common::assert_close(rgb.b, 0.0, EPS);
}

#[test]
fn hsv_nan_hue_treated_as_zero() {
    let hsv = Hsv {
        h: f64::NAN,
        s: 0.0,
        v: 0.5,
        alpha: None,
    };
    let rgb: Rgb = hsv.into();
    common::assert_close(rgb.r, 0.5, EPS);
    common::assert_close(rgb.g, 0.5, EPS);
    common::assert_close(rgb.b, 0.5, EPS);
}

#[test]
fn rgb_round_trip_through_hsv() {
    let rgb = Rgb {
        r: 0.3,
        g: 0.6,
        b: 0.9,
        alpha: None,
    };
    let hsv: Hsv = rgb.into();
    common::assert_close(hsv.h, 210.0, EPS);
    common::assert_close(hsv.s, 0.6666666666666667, EPS);
    common::assert_close(hsv.v, 0.9, EPS);
    let back: Rgb = hsv.into();
    common::assert_close(back.r, 0.29999999999999993, EPS);
    common::assert_close(back.g, 0.6, EPS);
    common::assert_close(back.b, 0.9, EPS);
}

#[test]
fn hsv_alpha_preserved() {
    let hsv = Hsv {
        h: 30.0,
        s: 0.4,
        v: 0.7,
        alpha: Some(0.7),
    };
    let rgb: Rgb = hsv.into();
    assert_eq!(rgb.alpha, Some(0.7));
    let back: Hsv = rgb.into();
    assert_eq!(back.alpha, Some(0.7));
}

#[test]
fn hsv_to_xyz65() {
    let hsv = Hsv {
        h: 30.0,
        s: 0.4,
        v: 0.7,
        alpha: None,
    };
    let xyz = hsv.to_xyz65();
    common::assert_close(xyz.x, 0.3092548778514898, CHAIN_EPS);
    common::assert_close(xyz.y, 0.30173576882063885, CHAIN_EPS);
    common::assert_close(xyz.z, 0.18133153795756435, CHAIN_EPS);
}

#[test]
fn hsv_from_xyz65_white() {
    // Same matrix imprecision as Hsl: D65 white round-tripped through the
    // sRGB matrix produces a tiny non-zero saturation.
    let xyz = Xyz65 {
        x: 0.9504559270516715,
        y: 0.9999999999999999,
        z: 1.0890577507598784,
        alpha: None,
    };
    let hsv = Hsv::from_xyz65(xyz);
    common::assert_close(hsv.h, 60.0, CHAIN_EPS);
    common::assert_close(hsv.s, 2.220446049250313e-16, CHAIN_EPS);
    common::assert_close(hsv.v, 0.9999999999999999, CHAIN_EPS);
}
