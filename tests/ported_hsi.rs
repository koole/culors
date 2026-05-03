//! Ported tests for HSI.

use culor::spaces::{Hsi, Rgb};
use culor::ColorSpace;

#[path = "common/mod.rs"]
mod common;

const EPS: f64 = 1e-12;

#[test]
fn hsi_metadata() {
    assert_eq!(Hsi::CHANNELS, &["h", "s", "i"]);
    assert_eq!(Hsi::MODE, "hsi");
}

#[test]
fn hsi_red() {
    let c: Hsi = Rgb {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        alpha: None,
    }
    .into();
    common::assert_close(c.h, 0.0, EPS);
    common::assert_close(c.s, 1.0, EPS);
    common::assert_close(c.i, 1.0 / 3.0, EPS);
}

#[test]
fn hsi_green() {
    let c: Hsi = Rgb {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        alpha: None,
    }
    .into();
    common::assert_close(c.h, 120.0, EPS);
    common::assert_close(c.s, 1.0, EPS);
    common::assert_close(c.i, 1.0 / 3.0, EPS);
}

#[test]
fn hsi_blue() {
    let c: Hsi = Rgb {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        alpha: None,
    }
    .into();
    common::assert_close(c.h, 240.0, EPS);
    common::assert_close(c.s, 1.0, EPS);
    common::assert_close(c.i, 1.0 / 3.0, EPS);
}

#[test]
fn hsi_gray_hue_nan() {
    let c: Hsi = Rgb {
        r: 0.5,
        g: 0.5,
        b: 0.5,
        alpha: None,
    }
    .into();
    assert!(c.h.is_nan());
    common::assert_close(c.s, 0.0, EPS);
    common::assert_close(c.i, 0.5, EPS);
}

#[test]
fn hsi_arbitrary() {
    let c: Hsi = Rgb {
        r: 0.25,
        g: 0.4,
        b: 0.75,
        alpha: None,
    }
    .into();
    common::assert_close(c.h, 222.0, EPS);
    common::assert_close(c.s, 0.4642857142857143, EPS);
    common::assert_close(c.i, 0.4666666666666666, EPS);
}

#[test]
fn hsi_to_rgb_round_trip() {
    let original = Hsi {
        h: 200.0,
        s: 0.5,
        i: 0.6,
        alpha: Some(0.4),
    };
    let rgb: Rgb = original.into();
    let back: Hsi = rgb.into();
    common::assert_close(back.h, 200.0, 1e-9);
    common::assert_close(back.s, 0.5, 1e-9);
    common::assert_close(back.i, 0.6, 1e-9);
    assert_eq!(back.alpha, Some(0.4));
}

#[test]
fn hsi_zero() {
    let c: Hsi = Rgb {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        alpha: None,
    }
    .into();
    assert!(c.h.is_nan());
    assert_eq!(c.s, 0.0);
    assert_eq!(c.i, 0.0);
}
