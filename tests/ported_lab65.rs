//! Ported tests for the `Lab65` color space (CIE Lab D65).
//!
//! Reference values produced by culori 4.0.2 and pasted verbatim.

use culors::spaces::{Lab65, Rgb, Xyz65};
use culors::ColorSpace;

#[path = "common/mod.rs"]
mod common;

const EPS: f64 = 1e-12;
const CHAIN_EPS: f64 = 1e-10;

#[test]
fn lab65_metadata() {
    assert_eq!(Lab65::CHANNELS, &["l", "a", "b"]);
    assert_eq!(Lab65::MODE, "lab65");
}

/// Regression for the JS-parity D65 white point: feed the Lab65 inverse
/// the exact `(0.3127 / 0.329, 1, (1 - 0.3127 - 0.329) / 0.329)` produced
/// at runtime — exactly culori's `D65.X / .Y / .Z` — and require the
/// result `L=100, a=0, b=0`. A precomputed-literal regression in lab65
/// would land 1 ULP away on `X` in some environments and would surface
/// here as a non-zero `a` channel.
#[test]
fn lab65_d65_white_point_matches_runtime_division() {
    let white = Xyz65 {
        x: 0.3127 / 0.329,
        y: 1.0,
        z: (1.0 - 0.3127 - 0.329) / 0.329,
        alpha: None,
    };
    let lab = Lab65::from(white);
    common::assert_close(lab.l, 100.0, 1e-15);
    common::assert_close(lab.a, 0.0, 1e-15);
    common::assert_close(lab.b, 0.0, 1e-15);
}

#[test]
fn xyz65_d65_white_to_lab65() {
    // D65 white point -> L=100, a=0, b=0.
    let xyz = Xyz65 {
        x: 0.9504559270516716,
        y: 1.0,
        z: 1.0890577507598784,
        alpha: None,
    };
    let lab = Lab65::from(xyz);
    common::assert_close(lab.l, 100.0, EPS);
    common::assert_close(lab.a, 0.0, EPS);
    common::assert_close(lab.b, 0.0, EPS);
}

#[test]
fn xyz65_black_to_lab65() {
    let xyz = Xyz65 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        alpha: None,
    };
    let lab = Lab65::from(xyz);
    common::assert_close(lab.l, 0.0, EPS);
    common::assert_close(lab.a, 0.0, EPS);
    common::assert_close(lab.b, 0.0, EPS);
}

#[test]
fn rgb_red_to_lab65() {
    let lab = Lab65::from(Rgb {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        alpha: None,
    });
    common::assert_close(lab.l, 53.237115595429344, CHAIN_EPS);
    common::assert_close(lab.a, 80.09011352310385, CHAIN_EPS);
    common::assert_close(lab.b, 67.20326351172214, CHAIN_EPS);
}

#[test]
fn rgb_green_to_lab65() {
    let lab = Lab65::from(Rgb {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        alpha: None,
    });
    common::assert_close(lab.l, 87.73551910966, CHAIN_EPS);
    common::assert_close(lab.a, -86.18159689039895, CHAIN_EPS);
    common::assert_close(lab.b, 83.18662027362998, CHAIN_EPS);
}

#[test]
fn rgb_blue_to_lab65() {
    let lab = Lab65::from(Rgb {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        alpha: None,
    });
    common::assert_close(lab.l, 32.30087290398018, CHAIN_EPS);
    common::assert_close(lab.a, 79.1952703074042, CHAIN_EPS);
    common::assert_close(lab.b, -107.85546553974265, CHAIN_EPS);
}

#[test]
fn rgb_achromatic_snap_to_lab65() {
    // r == g == b — culori zeroes the residual a/b that the floating-point
    // matrix multiply leaves behind.
    let lab = Lab65::from(Rgb {
        r: 0.5,
        g: 0.5,
        b: 0.5,
        alpha: None,
    });
    common::assert_close(lab.l, 53.38896474111431, CHAIN_EPS);
    assert_eq!(lab.a, 0.0);
    assert_eq!(lab.b, 0.0);
}

#[test]
fn rgb_alpha_passthrough_to_lab65() {
    let lab = Lab65::from(Rgb {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        alpha: Some(0.5),
    });
    common::assert_close(lab.l, 53.237115595429344, CHAIN_EPS);
    common::assert_close(lab.a, 80.09011352310385, CHAIN_EPS);
    common::assert_close(lab.b, 67.20326351172214, CHAIN_EPS);
    assert_eq!(lab.alpha, Some(0.5));
}

#[test]
fn lab65_to_xyz65_roundtrip() {
    // c.xyz65({mode:'lab65', l:50, a:25, b:-25})
    let lab = Lab65 {
        l: 50.0,
        a: 25.0,
        b: -25.0,
        alpha: None,
    };
    let xyz = lab.to_xyz65();
    common::assert_close(xyz.x, 0.22538828985418838, EPS);
    common::assert_close(xyz.y, 0.18418651851244416, EPS);
    common::assert_close(xyz.z, 0.3639691577104542, EPS);
}

#[test]
fn lab65_chained_from_rgb_to_rgb() {
    // c.rgb({mode:'lab65', l:50, a:25, b:-25})
    let lab = Lab65 {
        l: 50.0,
        a: 25.0,
        b: -25.0,
        alpha: None,
    };
    let xyz = lab.to_xyz65();
    let rgb: Rgb = Rgb::from_xyz65(xyz);
    common::assert_close(rgb.r, 0.5524433895522728, CHAIN_EPS);
    common::assert_close(rgb.g, 0.41304706391355944, CHAIN_EPS);
    common::assert_close(rgb.b, 0.6339922828542454, CHAIN_EPS);
}
