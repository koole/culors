//! Ported tests for the `Xyz50` color space.
//!
//! Reference values produced by culori 4.0.2 and pasted verbatim.

use culors::spaces::{Xyz50, Xyz65};
use culors::ColorSpace;

#[path = "common/mod.rs"]
mod common;

const EPS: f64 = 1e-12;

#[test]
fn xyz50_channels_excludes_alpha() {
    assert_eq!(Xyz50::CHANNELS, &["x", "y", "z"]);
    assert_eq!(Xyz50::CHANNELS.len(), 3);
    assert_eq!(Xyz50::MODE, "xyz50");
}

#[test]
fn xyz50_from_xyz65_white() {
    // Adapting the sRGB-derived D65 white {0.9504559270516715, ~1, 1.0890577507598784}
    // to D50 yields culori's reference D50 white.
    let xyz65 = Xyz65 {
        x: 0.9504559270516715,
        y: 0.9999999999999999,
        z: 1.0890577507598784,
        alpha: None,
    };
    let xyz50 = Xyz50::from_xyz65(xyz65);
    common::assert_close(xyz50.x, 0.9642956660812441, EPS);
    common::assert_close(xyz50.y, 1.0000000361162846, EPS);
    common::assert_close(xyz50.z, 0.8251045485672053, EPS);
}

#[test]
fn xyz50_from_xyz65_black() {
    let xyz65 = Xyz65 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        alpha: None,
    };
    let xyz50 = Xyz50::from_xyz65(xyz65);
    common::assert_close(xyz50.x, 0.0, 1e-15);
    common::assert_close(xyz50.y, 0.0, 1e-15);
    common::assert_close(xyz50.z, 0.0, 1e-15);
}

#[test]
fn xyz50_from_xyz65_red() {
    let xyz65 = Xyz65 {
        x: 0.4123907992659593,
        y: 0.2126390058715102,
        z: 0.0193308187155918,
        alpha: None,
    };
    let xyz50 = Xyz50::from_xyz65(xyz65);
    common::assert_close(xyz50.x, 0.436065742824811, EPS);
    common::assert_close(xyz50.y, 0.22249319175623702, EPS);
    common::assert_close(xyz50.z, 0.013923904500943456, EPS);
}

#[test]
fn xyz50_to_xyz65_round_trip_through_self() {
    // The Bradford matrices in culori are not exact inverses; using the
    // values that culori's chained converter produces verbatim.
    let xyz65 = Xyz65 {
        x: 0.5,
        y: 0.5,
        z: 0.5,
        alpha: None,
    };
    let xyz50 = Xyz50::from_xyz65(xyz65);
    let back = xyz50.to_xyz65();
    common::assert_close(back.x, 0.49999999160209957, EPS);
    common::assert_close(back.y, 0.500000034433209, EPS);
    common::assert_close(back.z, 0.4999999361278314, EPS);
}

#[test]
fn xyz50_alpha_preserved() {
    let xyz50 = Xyz50 {
        x: 0.5,
        y: 0.5,
        z: 0.5,
        alpha: Some(0.4),
    };
    let xyz65 = xyz50.to_xyz65();
    assert_eq!(xyz65.alpha, Some(0.4));
    common::assert_close(xyz65.x, 0.49781711224548925, EPS);
    common::assert_close(xyz65.y, 0.5013335750047787, EPS);
    common::assert_close(xyz65.z, 0.6610861209314587, EPS);
}

#[test]
fn xyz65_to_xyz50_mid() {
    let xyz65 = Xyz65 {
        x: 0.5,
        y: 0.5,
        z: 0.5,
        alpha: None,
    };
    let xyz50 = Xyz50::from_xyz65(xyz65);
    common::assert_close(xyz50.x, 0.5103421923192161, EPS);
    common::assert_close(xyz50.y, 0.5014942376160115, EPS);
    common::assert_close(xyz50.z, 0.37884318835099373, EPS);
}
