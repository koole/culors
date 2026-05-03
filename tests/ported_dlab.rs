//! Ported tests for DIN99o Lab.

use culor::convert;
use culor::spaces::{Dlab, Rgb};
use culor::ColorSpace;

#[path = "common/mod.rs"]
mod common;

const EPS: f64 = 1e-10;

#[test]
fn dlab_metadata() {
    assert_eq!(Dlab::CHANNELS, &["l", "a", "b"]);
    assert_eq!(Dlab::MODE, "dlab");
}

#[test]
fn dlab_red() {
    let rgb = Rgb {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        alpha: None,
    };
    let d: Dlab = convert(rgb);
    common::assert_close(d.l, 57.28917941426676, EPS);
    common::assert_close(d.a, 39.49797800074304, EPS);
    common::assert_close(d.b, 30.518440059252875, EPS);
}

#[test]
fn dlab_green() {
    let rgb = Rgb {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        alpha: None,
    };
    let d: Dlab = convert(rgb);
    common::assert_close(d.l, 89.36631009006445, EPS);
    common::assert_close(d.a, -37.82582946463206, EPS);
    common::assert_close(d.b, 32.08625519652728, EPS);
}

#[test]
fn dlab_blue() {
    let rgb = Rgb {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        alpha: None,
    };
    let d: Dlab = convert(rgb);
    common::assert_close(d.l, 36.029930076331965, EPS);
    common::assert_close(d.a, 31.935029159217553, EPS);
    common::assert_close(d.b, -40.38329666767036, EPS);
}

#[test]
fn dlab_gray() {
    let rgb = Rgb {
        r: 0.5,
        g: 0.5,
        b: 0.5,
        alpha: None,
    };
    let d: Dlab = convert(rgb);
    common::assert_close(d.l, 57.43806128457824, EPS);
    common::assert_close(d.a, 0.0, EPS);
    common::assert_close(d.b, 0.0, EPS);
}

#[test]
fn dlab_arbitrary() {
    let rgb = Rgb {
        r: 0.25,
        g: 0.4,
        b: 0.75,
        alpha: None,
    };
    let d: Dlab = convert(rgb);
    common::assert_close(d.l, 48.83929889376355, EPS);
    common::assert_close(d.a, 9.048204561515925, EPS);
    common::assert_close(d.b, -32.50039109736216, EPS);
}

#[test]
fn dlab_round_trip() {
    let original = Dlab {
        l: 50.0,
        a: 12.0,
        b: -28.0,
        alpha: Some(0.6),
    };
    let xyz = original.to_xyz65();
    let back = Dlab::from_xyz65(xyz);
    common::assert_close(back.l, 50.0, 1e-9);
    common::assert_close(back.a, 12.0, 1e-9);
    common::assert_close(back.b, -28.0, 1e-9);
    assert_eq!(back.alpha, Some(0.6));
}

#[test]
fn dlab_zero() {
    let d = Dlab {
        l: 0.0,
        a: 0.0,
        b: 0.0,
        alpha: None,
    };
    let rgb: Rgb = convert(d);
    common::assert_close(rgb.r, 0.0, 1e-9);
    common::assert_close(rgb.g, 0.0, 1e-9);
    common::assert_close(rgb.b, 0.0, 1e-9);
}

#[test]
fn dlab_white() {
    let d = Dlab {
        l: 100.0,
        a: 0.0,
        b: 0.0,
        alpha: None,
    };
    let rgb: Rgb = convert(d);
    common::assert_close(rgb.r, 1.0, 1e-9);
    common::assert_close(rgb.g, 1.0, 1e-9);
    common::assert_close(rgb.b, 1.0, 1e-9);
}
