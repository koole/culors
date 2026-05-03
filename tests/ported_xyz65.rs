//! Ported tests for the `Xyz65` hub color space.

use culors::spaces::{Rgb, Xyz65};
use culors::ColorSpace;

#[path = "common/mod.rs"]
mod common;

#[test]
fn xyz65_round_trip_through_self() {
    let c = Xyz65 {
        x: 0.5,
        y: 0.4,
        z: 0.3,
        alpha: None,
    };
    let back = Xyz65::from_xyz65(c.to_xyz65());
    common::assert_close(back.x, 0.5, 1e-15);
    common::assert_close(back.y, 0.4, 1e-15);
    common::assert_close(back.z, 0.3, 1e-15);
}

#[test]
fn xyz65_channels_excludes_alpha() {
    assert_eq!(Xyz65::CHANNELS, &["x", "y", "z"]);
    assert_eq!(Xyz65::CHANNELS.len(), 3);
}

#[test]
fn xyz65_alpha_with_alpha() {
    let c = Xyz65 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        alpha: Some(0.5),
    };
    let with = c.with_alpha(Some(1.0));
    assert_eq!(with.alpha(), Some(1.0));
    let cleared = with.with_alpha(None);
    assert_eq!(cleared.alpha(), None);
}

/// sRGB primary mapped through the linear-sRGB → XYZ D65 matrix.
#[test]
fn rgb_red_to_xyz65() {
    let xyz: Xyz65 = Rgb {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        alpha: None,
    }
    .to_xyz65();
    common::assert_close(xyz.x, 0.4123907992659593, 1e-12);
    common::assert_close(xyz.y, 0.2126390058715102, 1e-12);
    common::assert_close(xyz.z, 0.01933081871559185, 1e-12);
}

/// sRGB white sums to D65 within numeric tolerance.
#[test]
fn rgb_white_to_xyz65() {
    let xyz: Xyz65 = Rgb {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        alpha: None,
    }
    .to_xyz65();
    common::assert_close(xyz.x, 0.3127 / 0.329, 1e-13);
    common::assert_close(xyz.y, 1.0, 1e-13);
    common::assert_close(xyz.z, (1.0 - 0.3127 - 0.329) / 0.329, 1e-13);
}

#[test]
fn xyz65_to_rgb_round_trip_via_self() {
    let xyz = Xyz65 {
        x: 0.25,
        y: 0.30,
        z: 0.45,
        alpha: Some(0.6),
    };
    let rgb = Rgb::from_xyz65(xyz);
    let back = rgb.to_xyz65();
    common::assert_close(back.x, xyz.x, 1e-12);
    common::assert_close(back.y, xyz.y, 1e-12);
    common::assert_close(back.z, xyz.z, 1e-12);
    assert_eq!(back.alpha, Some(0.6));
}

/// `from_xyz65` on the hub itself is identity.
#[test]
fn xyz65_from_self_is_identity() {
    let xyz = Xyz65 {
        x: 0.7,
        y: 0.8,
        z: 0.9,
        alpha: Some(0.25),
    };
    let same = Xyz65::from_xyz65(xyz);
    assert_eq!(same, xyz);
}
