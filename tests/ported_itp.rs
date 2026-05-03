//! Ported tests for ICtCp.

use culor::convert;
use culor::spaces::{Itp, Rgb};
use culor::ColorSpace;

#[path = "common/mod.rs"]
mod common;

const EPS: f64 = 1e-10;

#[test]
fn itp_metadata() {
    assert_eq!(Itp::CHANNELS, &["i", "t", "p"]);
    assert_eq!(Itp::MODE, "itp");
}

#[test]
fn itp_red() {
    let rgb = Rgb {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        alpha: None,
    };
    let v: Itp = convert(rgb);
    common::assert_close(v.i, 0.4278802843622844, EPS);
    common::assert_close(v.t, -0.11570435976969046, EPS);
    common::assert_close(v.p, 0.27872894737532694, EPS);
}

#[test]
fn itp_green() {
    let rgb = Rgb {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        alpha: None,
    };
    let v: Itp = convert(rgb);
    common::assert_close(v.i, 0.5397602802111227, EPS);
    common::assert_close(v.t, -0.28124791880458244, EPS);
    common::assert_close(v.p, -0.04948450651601132, EPS);
}

#[test]
fn itp_blue() {
    let rgb = Rgb {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        alpha: None,
    };
    let v: Itp = convert(rgb);
    common::assert_close(v.i, 0.35598362802705463, EPS);
    common::assert_close(v.t, 0.2693341555682033, EPS);
    common::assert_close(v.p, -0.16137109768327434, EPS);
}

#[test]
fn itp_arbitrary() {
    let rgb = Rgb {
        r: 0.25,
        g: 0.4,
        b: 0.75,
        alpha: None,
    };
    let v: Itp = convert(rgb);
    common::assert_close(v.i, 0.3956335876749574, EPS);
    common::assert_close(v.t, 0.1378085173482485, EPS);
    common::assert_close(v.p, -0.07824793852607184, EPS);
}

#[test]
fn itp_gray() {
    let rgb = Rgb {
        r: 0.5,
        g: 0.5,
        b: 0.5,
        alpha: None,
    };
    let v: Itp = convert(rgb);
    common::assert_close(v.i, 0.42701896671887857, EPS);
    common::assert_close(v.t, 0.0, 1e-12);
    common::assert_close(v.p, 0.0, 1e-12);
}

#[test]
fn itp_round_trip() {
    let original = Itp {
        i: 0.4,
        t: 0.05,
        p: -0.1,
        alpha: Some(0.4),
    };
    let xyz = original.to_xyz65();
    let back = Itp::from_xyz65(xyz);
    common::assert_close(back.i, original.i, 1e-7);
    common::assert_close(back.t, original.t, 1e-7);
    common::assert_close(back.p, original.p, 1e-7);
    assert_eq!(back.alpha, Some(0.4));
}
