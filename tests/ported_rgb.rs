//! Ported tests for the `Rgb` (sRGB) color space.
//!
//! Expected XYZ D65 values are produced by running culori 4.0.2 directly
//! (see the project README + plan) and pasted verbatim.

use culor::spaces::{Rgb, Xyz65};
use culor::ColorSpace;

#[path = "common/mod.rs"]
mod common;

const EPS: f64 = 1e-12;

#[test]
fn rgb_channels_excludes_alpha() {
    assert_eq!(Rgb::CHANNELS, &["r", "g", "b"]);
    assert_eq!(Rgb::CHANNELS.len(), 3);
}

#[test]
fn rgb_white_to_xyz65() {
    let white = Rgb {
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
fn rgb_black_to_xyz65() {
    let black = Rgb {
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
fn rgb_red_to_xyz65() {
    let red = Rgb {
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
fn rgb_green_to_xyz65() {
    let green = Rgb {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        alpha: None,
    };
    let xyz = green.to_xyz65();
    common::assert_close(xyz.x, 0.357584339383878, EPS);
    common::assert_close(xyz.y, 0.715168678767756, EPS);
    common::assert_close(xyz.z, 0.119194779794626, EPS);
}

#[test]
fn rgb_blue_to_xyz65() {
    let blue = Rgb {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        alpha: None,
    };
    let xyz = blue.to_xyz65();
    common::assert_close(xyz.x, 0.1804807884018343, EPS);
    common::assert_close(xyz.y, 0.0721923153607337, EPS);
    common::assert_close(xyz.z, 0.9505321522496607, EPS);
}

#[test]
fn rgb_mid_grey_to_xyz65() {
    let mid = Rgb {
        r: 0.5,
        g: 0.5,
        b: 0.5,
        alpha: None,
    };
    let xyz = mid.to_xyz65();
    common::assert_close(xyz.x, 0.20343667060423742, EPS);
    common::assert_close(xyz.y, 0.21404114048223252, EPS);
    common::assert_close(xyz.z, 0.23310316302365935, EPS);
}

#[test]
fn rgb_mid_grey_round_trip() {
    let mid = Rgb {
        r: 0.5,
        g: 0.5,
        b: 0.5,
        alpha: None,
    };
    let back = Rgb::from_xyz65(mid.to_xyz65());
    common::assert_close(back.r, 0.5, EPS);
    common::assert_close(back.g, 0.5, EPS);
    common::assert_close(back.b, 0.5, EPS);
}

#[test]
fn rgb_alpha_preserved() {
    let c = Rgb {
        r: 0.5,
        g: 0.5,
        b: 0.5,
        alpha: Some(0.7),
    };
    let xyz = c.to_xyz65();
    assert_eq!(xyz.alpha, Some(0.7));
    let back = Rgb::from_xyz65(xyz);
    assert_eq!(back.alpha, Some(0.7));
}

#[test]
fn rgb_negative_channel_signum() {
    // culori applies the sRGB transfer with sign preservation, so negative
    // channels survive the round-trip (HDR-style extended range).
    // Reference values come from culori's converter('xyz65') applied to
    // {r:-0.1, g:0.5, b:0.2}.
    let c = Rgb {
        r: -0.1,
        g: 0.5,
        b: 0.2,
        alpha: None,
    };
    let xyz = c.to_xyz65();
    common::assert_close(xyz.x, 0.07837921314115943, EPS);
    common::assert_close(xyz.y, 0.15333418572258703, EPS);
    common::assert_close(xyz.z, 0.05678598220091052, EPS);

    let back = Rgb::from_xyz65(xyz);
    common::assert_close(back.r, -0.1, 1e-12);
    common::assert_close(back.g, 0.5, 1e-12);
    common::assert_close(back.b, 0.2, 1e-12);
}
