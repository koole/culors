//! Ported tests for NTSC Y'IQ.

use culor::spaces::{Rgb, Yiq};
use culor::ColorSpace;

#[path = "common/mod.rs"]
mod common;

const EPS: f64 = 1e-12;

#[test]
fn yiq_metadata() {
    assert_eq!(Yiq::CHANNELS, &["y", "i", "q"]);
    assert_eq!(Yiq::MODE, "yiq");
}

#[test]
fn yiq_red() {
    let y: Yiq = Rgb {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        alpha: None,
    }
    .into();
    common::assert_close(y.y, 0.29889531, EPS);
    common::assert_close(y.i, 0.59597799, EPS);
    common::assert_close(y.q, 0.21147017, EPS);
}

#[test]
fn yiq_green() {
    let y: Yiq = Rgb {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        alpha: None,
    }
    .into();
    common::assert_close(y.y, 0.58662247, EPS);
    common::assert_close(y.i, -0.2741761, EPS);
    common::assert_close(y.q, -0.52261711, EPS);
}

#[test]
fn yiq_blue() {
    let y: Yiq = Rgb {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        alpha: None,
    }
    .into();
    common::assert_close(y.y, 0.11448223, EPS);
    common::assert_close(y.i, -0.32180189, EPS);
    common::assert_close(y.q, 0.31114694, EPS);
}

#[test]
fn yiq_gray() {
    let y: Yiq = Rgb {
        r: 0.5,
        g: 0.5,
        b: 0.5,
        alpha: None,
    }
    .into();
    common::assert_close(y.y, 0.500000005, EPS);
    common::assert_close(y.i, 0.0, EPS);
    common::assert_close(y.q, 0.0, EPS);
}

#[test]
fn yiq_arbitrary() {
    let y: Yiq = Rgb {
        r: 0.25,
        g: 0.4,
        b: 0.75,
        alpha: None,
    }
    .into();
    common::assert_close(y.y, 0.395234488, EPS);
    common::assert_close(y.i, -0.20202736, EPS);
    common::assert_close(y.q, 0.07718090349999998, EPS);
}

#[test]
fn yiq_round_trip() {
    let original = Rgb {
        r: 0.3,
        g: 0.6,
        b: 0.9,
        alpha: Some(0.8),
    };
    let y: Yiq = original.into();
    let back: Rgb = y.into();
    // culori's forward and inverse matrices aren't exact inverses
    // (mid-1980s rounding) — round-trip drift is on the order of 1e-7.
    common::assert_close(back.r, 0.3, 1e-7);
    common::assert_close(back.g, 0.6, 1e-7);
    common::assert_close(back.b, 0.9, 1e-7);
    assert_eq!(back.alpha, Some(0.8));
}
