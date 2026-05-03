//! Ported tests for the `Lch65` color space (CIE Lch D65, polar `Lab65`).
//!
//! Reference values produced by culori 4.0.2 and pasted verbatim.

use culor::spaces::{Lab65, Lch65, Rgb};
use culor::ColorSpace;

#[path = "common/mod.rs"]
mod common;

const EPS: f64 = 1e-12;
const CHAIN_EPS: f64 = 1e-10;

#[test]
fn lch65_metadata() {
    assert_eq!(Lch65::CHANNELS, &["l", "c", "h"]);
    assert_eq!(Lch65::MODE, "lch65");
}

#[test]
fn rgb_white_to_lch65_nan_hue() {
    let lch = Lch65::from(Rgb {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        alpha: None,
    });
    common::assert_close(lch.l, 100.0, CHAIN_EPS);
    assert_eq!(lch.c, 0.0);
    assert!(
        lch.h.is_nan(),
        "achromatic hue should be NaN, got {}",
        lch.h
    );
}

#[test]
fn rgb_black_to_lch65_nan_hue() {
    let lch = Lch65::from(Rgb {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        alpha: None,
    });
    common::assert_close(lch.l, 0.0, EPS);
    assert_eq!(lch.c, 0.0);
    assert!(lch.h.is_nan());
}

#[test]
fn rgb_gray_to_lch65_nan_hue() {
    let lch = Lch65::from(Rgb {
        r: 0.5,
        g: 0.5,
        b: 0.5,
        alpha: None,
    });
    common::assert_close(lch.l, 53.38896474111431, CHAIN_EPS);
    assert_eq!(lch.c, 0.0);
    assert!(lch.h.is_nan());
}

#[test]
fn rgb_red_to_lch65() {
    let lch = Lch65::from(Rgb {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        alpha: None,
    });
    common::assert_close(lch.l, 53.237115595429344, CHAIN_EPS);
    common::assert_close(lch.c, 104.55001152926587, CHAIN_EPS);
    common::assert_close(lch.h, 39.99986515439813, CHAIN_EPS);
}

#[test]
fn rgb_green_to_lch65() {
    let lch = Lch65::from(Rgb {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        alpha: None,
    });
    common::assert_close(lch.l, 87.73551910966, CHAIN_EPS);
    common::assert_close(lch.c, 119.78013789910383, CHAIN_EPS);
    common::assert_close(lch.h, 136.01306868501493, CHAIN_EPS);
}

#[test]
fn rgb_blue_to_lch65() {
    let lch = Lch65::from(Rgb {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        alpha: None,
    });
    common::assert_close(lch.l, 32.30087290398018, CHAIN_EPS);
    common::assert_close(lch.c, 133.8084163491125, CHAIN_EPS);
    common::assert_close(lch.h, 306.28880325729324, CHAIN_EPS);
}

#[test]
fn rgb_alpha_passthrough_to_lch65() {
    let lch = Lch65::from(Rgb {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        alpha: Some(0.5),
    });
    assert_eq!(lch.alpha, Some(0.5));
}

#[test]
fn lab65_to_lch65_roundtrip() {
    // c.lch65({mode:'lab65', l:50, a:25, b:-25})
    let lab = Lab65 {
        l: 50.0,
        a: 25.0,
        b: -25.0,
        alpha: None,
    };
    let lch = Lch65::from(lab);
    common::assert_close(lch.l, 50.0, EPS);
    common::assert_close(lch.c, 35.35533905932738, EPS);
    common::assert_close(lch.h, 315.0, EPS);
}

#[test]
fn lch65_back_to_lab65() {
    // c.lab65({mode:'lch65', l:50, c:40, h:30})
    let lch = Lch65 {
        l: 50.0,
        c: 40.0,
        h: 30.0,
        alpha: None,
    };
    let lab = Lab65::from(lch);
    common::assert_close(lab.l, 50.0, EPS);
    common::assert_close(lab.a, 34.64101615137755, EPS);
    common::assert_close(lab.b, 19.999999999999996, EPS);
}

#[test]
fn lch65_to_rgb_full_chain() {
    // c.rgb({mode:'lch65', l:50, c:40, h:30})
    let lch = Lch65 {
        l: 50.0,
        c: 40.0,
        h: 30.0,
        alpha: None,
    };
    let rgb: Rgb = Rgb::from_xyz65(lch.to_xyz65());
    common::assert_close(rgb.r, 0.7095587475594507, CHAIN_EPS);
    common::assert_close(rgb.g, 0.36664712113671, CHAIN_EPS);
    common::assert_close(rgb.b, 0.3404712340981788, CHAIN_EPS);
}
