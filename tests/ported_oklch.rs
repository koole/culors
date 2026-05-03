//! Ported tests for the `Oklch` color space (polar form of Oklab).
//!
//! Reference values produced by culori 4.0.2 and pasted verbatim.

use culor::spaces::{Oklab, Oklch, Rgb, Xyz65};
use culor::ColorSpace;

#[path = "common/mod.rs"]
mod common;

const EPS: f64 = 1e-12;
const CHAIN_EPS: f64 = 1e-10;

#[test]
fn oklch_channels_excludes_alpha() {
    assert_eq!(Oklch::CHANNELS, &["l", "c", "h"]);
    assert_eq!(Oklch::CHANNELS.len(), 3);
    assert_eq!(Oklch::MODE, "oklch");
}

#[test]
fn oklab_a30_b40_to_oklch() {
    // Same polar form as Lch — magnitudes are smaller in Oklab.
    // c.oklch({mode:'oklab', l:0.5, a:0.03, b:0.04})
    // -> {"mode":"oklch","l":0.5,"c":0.05,"h":53.13010235415598}
    let oklab = Oklab {
        l: 0.5,
        a: 0.03,
        b: 0.04,
        alpha: None,
    };
    let oklch = Oklch::from(oklab);
    common::assert_close(oklch.l, 0.5, EPS);
    common::assert_close(oklch.c, 0.05, EPS);
    common::assert_close(oklch.h, 53.13010235415598, EPS);
}

#[test]
fn oklab_a_neg30_b40_to_oklch() {
    // c.oklch({mode:'oklab', l:0.5, a:-0.03, b:0.04})
    // -> {"l":0.5,"c":0.05,"h":126.86989764584402}
    let oklab = Oklab {
        l: 0.5,
        a: -0.03,
        b: 0.04,
        alpha: None,
    };
    let oklch = Oklch::from(oklab);
    common::assert_close(oklch.l, 0.5, EPS);
    common::assert_close(oklch.c, 0.05, EPS);
    common::assert_close(oklch.h, 126.86989764584402, EPS);
}

#[test]
fn oklab_a30_b_neg40_to_oklch_wraps_negative_atan2() {
    // c.oklch({mode:'oklab', l:0.5, a:0.03, b:-0.04})
    // -> {"l":0.5,"c":0.05,"h":306.86989764584405}
    let oklab = Oklab {
        l: 0.5,
        a: 0.03,
        b: -0.04,
        alpha: None,
    };
    let oklch = Oklch::from(oklab);
    common::assert_close(oklch.l, 0.5, EPS);
    common::assert_close(oklch.c, 0.05, EPS);
    common::assert_close(oklch.h, 306.86989764584405, EPS);
}

#[test]
fn oklab_both_negative_to_oklch() {
    // c.oklch({mode:'oklab', l:0.5, a:-0.03, b:-0.04})
    // -> {"l":0.5,"c":0.05,"h":233.13010235415598}
    let oklab = Oklab {
        l: 0.5,
        a: -0.03,
        b: -0.04,
        alpha: None,
    };
    let oklch = Oklch::from(oklab);
    common::assert_close(oklch.l, 0.5, EPS);
    common::assert_close(oklch.c, 0.05, EPS);
    common::assert_close(oklch.h, 233.13010235415598, EPS);
}

#[test]
fn oklab_achromatic_yields_nan_hue() {
    // c.oklch({mode:'oklab', l:0.5, a:0, b:0}) -> {"l":0.5,"c":0}
    let oklab = Oklab {
        l: 0.5,
        a: 0.0,
        b: 0.0,
        alpha: None,
    };
    let oklch = Oklch::from(oklab);
    common::assert_close(oklch.l, 0.5, EPS);
    common::assert_close(oklch.c, 0.0, EPS);
    assert!(oklch.h.is_nan(), "expected NaN hue, got {}", oklch.h);
}

#[test]
fn oklch_nan_hue_back_to_oklab_no_nan_propagation() {
    // c.oklab({mode:'oklch', l:0.5, c:0}) -> {"mode":"oklab","l":0.5,"a":0,"b":0}
    let oklch = Oklch {
        l: 0.5,
        c: 0.0,
        h: f64::NAN,
        alpha: None,
    };
    let oklab = Oklab::from(oklch);
    common::assert_close(oklab.l, 0.5, EPS);
    common::assert_close(oklab.a, 0.0, EPS);
    common::assert_close(oklab.b, 0.0, EPS);
}

#[test]
fn oklab_oklch_round_trip() {
    let oklab = Oklab {
        l: 0.6,
        a: 0.1,
        b: -0.05,
        alpha: None,
    };
    let oklch = Oklch::from(oklab);
    let back = Oklab::from(oklch);
    common::assert_close(back.l, oklab.l, EPS);
    common::assert_close(back.a, oklab.a, EPS);
    common::assert_close(back.b, oklab.b, EPS);
}

#[test]
fn srgb_red_via_xyz65_to_oklch() {
    // c.oklch({mode:'rgb', r:1, g:0, b:0})
    // -> {"l":0.6279553639214311,"c":0.2576833038053608,"h":29.233880279627854}
    let rgb = Rgb {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        alpha: None,
    };
    let oklch = Oklch::from_xyz65(rgb.to_xyz65());
    common::assert_close(oklch.l, 0.6279553639214311, CHAIN_EPS);
    common::assert_close(oklch.c, 0.2576833038053608, CHAIN_EPS);
    common::assert_close(oklch.h, 29.233880279627854, CHAIN_EPS);
}

#[test]
fn srgb_green_via_xyz65_to_oklch() {
    // c.oklch({mode:'rgb', r:0, g:1, b:0})
    // -> {"l":0.8664396175234368,"c":0.29482722454269544,"h":142.4953450414438}
    let rgb = Rgb {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        alpha: None,
    };
    let oklch = Oklch::from_xyz65(rgb.to_xyz65());
    common::assert_close(oklch.l, 0.8664396175234368, CHAIN_EPS);
    common::assert_close(oklch.c, 0.29482722454269544, CHAIN_EPS);
    common::assert_close(oklch.h, 142.4953450414438, CHAIN_EPS);
}

#[test]
fn srgb_arbitrary_via_xyz65_to_oklch() {
    // c.oklch({mode:'rgb', r:0.3, g:0.6, b:0.9})
    // -> {"l":0.6687264800232976,"c":0.13708177791741408,"h":250.60273895296604}
    let rgb = Rgb {
        r: 0.3,
        g: 0.6,
        b: 0.9,
        alpha: None,
    };
    let oklch = Oklch::from_xyz65(rgb.to_xyz65());
    common::assert_close(oklch.l, 0.6687264800232976, CHAIN_EPS);
    common::assert_close(oklch.c, 0.13708177791741408, CHAIN_EPS);
    common::assert_close(oklch.h, 250.60273895296604, CHAIN_EPS);
}

#[test]
fn oklch_to_xyz65_via_hub_matches_oklab_path() {
    let oklch = Oklch {
        l: 0.6687264800232976,
        c: 0.13708177791741408,
        h: 250.60273895296604,
        alpha: None,
    };
    let via_hub: Xyz65 = oklch.to_xyz65();
    let via_oklab = Oklab::from(oklch).to_xyz65();
    common::assert_close(via_hub.x, via_oklab.x, EPS);
    common::assert_close(via_hub.y, via_oklab.y, EPS);
    common::assert_close(via_hub.z, via_oklab.z, EPS);
}

#[test]
fn oklch_alpha_preserved_through_oklab() {
    let oklch = Oklch {
        l: 0.5,
        c: 0.1,
        h: 120.0,
        alpha: Some(0.42),
    };
    let oklab: Oklab = oklch.into();
    assert_eq!(oklab.alpha, Some(0.42));
    let back: Oklch = oklab.into();
    assert_eq!(back.alpha, Some(0.42));
}

#[test]
fn oklch_alpha_preserved_with_nan_hue() {
    let oklab = Oklab {
        l: 0.3,
        a: 0.0,
        b: 0.0,
        alpha: Some(0.7),
    };
    let oklch = Oklch::from(oklab);
    assert!(oklch.h.is_nan());
    assert_eq!(oklch.alpha, Some(0.7));
    let back = Oklab::from(oklch);
    assert_eq!(back.alpha, Some(0.7));
}
