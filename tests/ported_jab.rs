//! Ported tests for JzAzBz.

use culor::convert;
use culor::spaces::{Jab, Rgb};
use culor::ColorSpace;

#[path = "common/mod.rs"]
mod common;

const EPS: f64 = 1e-10;

#[test]
fn jab_metadata() {
    assert_eq!(Jab::CHANNELS, &["j", "a", "b"]);
    assert_eq!(Jab::MODE, "jab");
}

#[test]
fn jab_red() {
    let rgb = Rgb {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        alpha: None,
    };
    let j: Jab = convert(rgb);
    common::assert_close(j.j, 0.13438473104350068, EPS);
    common::assert_close(j.a, 0.11788526260797229, EPS);
    common::assert_close(j.b, 0.11187810901317238, EPS);
}

#[test]
fn jab_green() {
    let rgb = Rgb {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        alpha: None,
    };
    let j: Jab = convert(rgb);
    common::assert_close(j.j, 0.17680712813178287, EPS);
    common::assert_close(j.a, -0.10904339610399605, EPS);
    common::assert_close(j.b, 0.11898943576039084, EPS);
}

#[test]
fn jab_blue() {
    let rgb = Rgb {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        alpha: None,
    };
    let j: Jab = convert(rgb);
    common::assert_close(j.j, 0.09577429215304557, EPS);
    common::assert_close(j.a, -0.04084549610065474, EPS);
    common::assert_close(j.b, -0.1858542849470936, EPS);
}

#[test]
fn jab_arbitrary() {
    let rgb = Rgb {
        r: 0.25,
        g: 0.4,
        b: 0.75,
        alpha: None,
    };
    let j: Jab = convert(rgb);
    common::assert_close(j.j, 0.10398883320693651, EPS);
    common::assert_close(j.a, -0.021010758371353633, EPS);
    common::assert_close(j.b, -0.09734986570595511, EPS);
}

#[test]
fn jab_white() {
    let rgb = Rgb {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        alpha: None,
    };
    let j: Jab = rgb.into();
    common::assert_close(j.j, 0.222065249535743, EPS);
    assert_eq!(j.a, 0.0);
    assert_eq!(j.b, 0.0);
}

#[test]
fn jab_round_trip() {
    let original = Jab {
        j: 0.13,
        a: 0.05,
        b: -0.08,
        alpha: Some(0.7),
    };
    let xyz = original.to_xyz65();
    let back = Jab::from_xyz65(xyz);
    // PQ encode/decode is sensitive at the precision level; 1e-7 is
    // representative of culori's own round-trip drift.
    common::assert_close(back.j, original.j, 1e-7);
    common::assert_close(back.a, original.a, 1e-7);
    common::assert_close(back.b, original.b, 1e-7);
    assert_eq!(back.alpha, Some(0.7));
}
