//! Ported tests for the `Lab` color space (CIE Lab D50).
//!
//! Reference values produced by culori 4.0.2 and pasted verbatim.

use culor::spaces::{Lab, Rgb, Xyz50, Xyz65};
use culor::ColorSpace;

#[path = "common/mod.rs"]
mod common;

const EPS: f64 = 1e-12;
const CHAIN_EPS: f64 = 1e-10;

#[test]
fn lab_channels_excludes_alpha() {
    assert_eq!(Lab::CHANNELS, &["l", "a", "b"]);
    assert_eq!(Lab::CHANNELS.len(), 3);
    assert_eq!(Lab::MODE, "lab");
}

#[test]
fn xyz50_d50_white_to_lab() {
    // D50 white point -> L=100, a=0, b=0.
    let xyz = Xyz50 {
        x: 0.9642956764295677,
        y: 1.0,
        z: 0.8251046025104602,
        alpha: None,
    };
    let lab = Lab::from(xyz);
    common::assert_close(lab.l, 100.0, EPS);
    common::assert_close(lab.a, 0.0, EPS);
    common::assert_close(lab.b, 0.0, EPS);
}

#[test]
fn xyz50_black_to_lab() {
    let xyz = Xyz50 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        alpha: None,
    };
    let lab = Lab::from(xyz);
    common::assert_close(lab.l, 0.0, EPS);
    common::assert_close(lab.a, 0.0, EPS);
    common::assert_close(lab.b, 0.0, EPS);
}

#[test]
fn lab_l50_to_xyz50() {
    // c.xyz50({mode:'lab', l:50, a:0, b:0})
    let lab = Lab {
        l: 50.0,
        a: 0.0,
        b: 0.0,
        alpha: None,
    };
    let xyz = Xyz50::from(lab);
    common::assert_close(xyz.x, 0.17761026345816444, EPS);
    common::assert_close(xyz.y, 0.18418651851244416, EPS);
    common::assert_close(xyz.z, 0.15197314414499577, EPS);
}

#[test]
fn lab_l50_a20_b_neg20_to_xyz50() {
    // c.xyz50({mode:'lab', l:50, a:20, b:-20})
    let lab = Lab {
        l: 50.0,
        a: 20.0,
        b: -20.0,
        alpha: None,
    };
    let xyz = Xyz50::from(lab);
    common::assert_close(xyz.x, 0.21776512232744577, EPS);
    common::assert_close(xyz.y, 0.18418651851244416, EPS);
    common::assert_close(xyz.z, 0.24701322494141761, EPS);
}

#[test]
fn srgb_red_to_lab_via_xyz65() {
    // Hub path: Lab::from_xyz65 ( xyz65 of pure red )
    // c.lab({mode:'rgb', r:1, g:0, b:0})
    // -> {"l":54.29054294696968,"a":80.80492033462417,"b":69.89098825896278}
    let rgb = Rgb {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        alpha: None,
    };
    let xyz65 = rgb.to_xyz65();
    let lab = Lab::from_xyz65(xyz65);
    common::assert_close(lab.l, 54.29054294696968, CHAIN_EPS);
    common::assert_close(lab.a, 80.80492033462417, CHAIN_EPS);
    common::assert_close(lab.b, 69.89098825896278, CHAIN_EPS);
}

#[test]
fn srgb_green_to_lab_via_xyz65() {
    // c.lab({mode:'rgb', r:0, g:1, b:0})
    let rgb = Rgb {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        alpha: None,
    };
    let lab = Lab::from_xyz65(rgb.to_xyz65());
    common::assert_close(lab.l, 87.81853633115202, CHAIN_EPS);
    common::assert_close(lab.a, -79.27108223854806, CHAIN_EPS);
    common::assert_close(lab.b, 80.99459785152247, CHAIN_EPS);
}

#[test]
fn srgb_blue_to_lab_via_xyz65() {
    // c.lab({mode:'rgb', r:0, g:0, b:1})
    let rgb = Rgb {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        alpha: None,
    };
    let lab = Lab::from_xyz65(rgb.to_xyz65());
    common::assert_close(lab.l, 29.568297153444703, CHAIN_EPS);
    common::assert_close(lab.a, 68.2874066521555, CHAIN_EPS);
    common::assert_close(lab.b, -112.02971798617645, CHAIN_EPS);
}

#[test]
fn srgb_arbitrary_to_lab_via_xyz65() {
    // c.lab({mode:'rgb', r:0.3, g:0.6, b:0.9})
    let rgb = Rgb {
        r: 0.3,
        g: 0.6,
        b: 0.9,
        alpha: None,
    };
    let lab = Lab::from_xyz65(rgb.to_xyz65());
    common::assert_close(lab.l, 60.991980887973384, CHAIN_EPS);
    common::assert_close(lab.a, -5.855573502634215, CHAIN_EPS);
    common::assert_close(lab.b, -46.52748655858361, CHAIN_EPS);
}

#[test]
fn lab_to_xyz65_round_trip() {
    let lab = Lab {
        l: 60.991980887973384,
        a: -5.855573502634215,
        b: -46.52748655858361,
        alpha: None,
    };
    let xyz = lab.to_xyz65();
    // Recompute by direct xyz50 -> xyz65 path.
    let x50 = Xyz50::from(lab);
    let expected = x50.to_xyz65();
    common::assert_close(xyz.x, expected.x, EPS);
    common::assert_close(xyz.y, expected.y, EPS);
    common::assert_close(xyz.z, expected.z, EPS);
}

#[test]
fn xyz65_round_trip_through_lab() {
    // Round trip drift matches culori's own (~1e-7) due to Bradford
    // adaptation matrices that aren't exact inverses.
    let xyz = Xyz65 {
        x: 0.5,
        y: 0.4,
        z: 0.3,
        alpha: None,
    };
    let lab = Lab::from_xyz65(xyz);
    let back = lab.to_xyz65();
    // Reference: culori roundtrip yields x=0.4999999902965942,
    // y=0.4000000345910334, z=0.29999994290464727.
    common::assert_close(back.x, 0.4999999902965942, CHAIN_EPS);
    common::assert_close(back.y, 0.4000000345910334, CHAIN_EPS);
    common::assert_close(back.z, 0.29999994290464727, CHAIN_EPS);
}

#[test]
fn lab_alpha_preserved() {
    let lab = Lab {
        l: 50.0,
        a: 10.0,
        b: -10.0,
        alpha: Some(0.42),
    };
    let xyz: Xyz50 = lab.into();
    assert_eq!(xyz.alpha, Some(0.42));
    let back: Lab = xyz.into();
    assert_eq!(back.alpha, Some(0.42));
}
