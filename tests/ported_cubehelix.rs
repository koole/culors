//! Ported tests for Cubehelix.

use culor::spaces::{Cubehelix, Rgb};
use culor::ColorSpace;

#[path = "common/mod.rs"]
mod common;

const EPS: f64 = 1e-12;

#[test]
fn cubehelix_metadata() {
    assert_eq!(Cubehelix::CHANNELS, &["h", "s", "l"]);
    assert_eq!(Cubehelix::MODE, "cubehelix");
}

#[test]
fn cubehelix_red() {
    let c: Cubehelix = Rgb {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        alpha: None,
    }
    .into();
    common::assert_close(c.l, 0.2999994453152424, EPS);
    common::assert_close(c.s, 1.9488976453722686, EPS);
    common::assert_close(c.h, -8.18973822915791, EPS);
}

#[test]
fn cubehelix_green() {
    let c: Cubehelix = Rgb {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        alpha: None,
    }
    .into();
    common::assert_close(c.l, 0.5900010051127478, EPS);
    common::assert_close(c.s, 1.9216187417323771, EPS);
    common::assert_close(c.h, -250.04083478116877, EPS);
}

#[test]
fn cubehelix_blue() {
    let c: Cubehelix = Rgb {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        alpha: None,
    }
    .into();
    common::assert_close(c.l, 0.10999954957200976, EPS);
    common::assert_close(c.s, 4.614386868039719, EPS);
    common::assert_close(c.h, -123.05782832267896, EPS);
}

#[test]
fn cubehelix_gray_undefined_saturation_and_hue() {
    let c: Cubehelix = Rgb {
        r: 0.5,
        g: 0.5,
        b: 0.5,
        alpha: None,
    }
    .into();
    common::assert_close(c.l, 0.5, EPS);
    assert_eq!(c.s, 0.0);
    assert!(c.h.is_nan());
}

#[test]
fn cubehelix_arbitrary() {
    let c: Cubehelix = Rgb {
        r: 0.25,
        g: 0.4,
        b: 0.75,
        alpha: None,
    }
    .into();
    common::assert_close(c.l, 0.3934999255529171, EPS);
    common::assert_close(c.s, 0.8052380516828442, EPS);
    common::assert_close(c.h, -139.90546874098982, EPS);
}

#[test]
fn cubehelix_to_rgb_known() {
    let r: Rgb = Cubehelix {
        h: 0.0,
        s: 0.5,
        l: 0.5,
        alpha: None,
    }
    .into();
    common::assert_close(r.r, 0.7022786386380979, EPS);
    common::assert_close(r.g, 0.42013645396543053, EPS);
    common::assert_close(r.b, 0.37669125000000003, EPS);
}

#[test]
fn cubehelix_to_rgb_zero_saturation() {
    let r: Rgb = Cubehelix {
        h: 240.0,
        s: 0.0,
        l: 0.3,
        alpha: None,
    }
    .into();
    common::assert_close(r.r, 0.3, EPS);
    common::assert_close(r.g, 0.3, EPS);
    common::assert_close(r.b, 0.3, EPS);
}

#[test]
fn cubehelix_round_trip_through_rgb() {
    let original = Cubehelix {
        h: 90.0,
        s: 0.6,
        l: 0.5,
        alpha: Some(0.7),
    };
    let rgb: Rgb = original.into();
    let back: Cubehelix = rgb.into();
    common::assert_close(back.l, original.l, 1e-10);
    common::assert_close(back.s, original.s, 1e-10);
    let dh = (back.h - original.h).rem_euclid(360.0);
    let dh = dh.min(360.0 - dh);
    assert!(dh < 1e-10, "hue mismatch dh={dh}");
    assert_eq!(back.alpha, Some(0.7));
}
