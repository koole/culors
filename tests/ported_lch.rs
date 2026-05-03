//! Ported tests for the `Lch` color space (CIE Lch, D50).
//!
//! Reference values produced by culori 4.0.2 and pasted verbatim.

use culors::spaces::{Lab, Lch, Rgb, Xyz65};
use culors::ColorSpace;

#[path = "common/mod.rs"]
mod common;

const EPS: f64 = 1e-12;
const CHAIN_EPS: f64 = 1e-10;

#[test]
fn lch_channels_excludes_alpha() {
    assert_eq!(Lch::CHANNELS, &["l", "c", "h"]);
    assert_eq!(Lch::CHANNELS.len(), 3);
    assert_eq!(Lch::MODE, "lch");
}

#[test]
fn lab_a30_b40_to_lch() {
    // c.lch({mode:'lab', l:50, a:30, b:40})
    // -> {"mode":"lch","l":50,"c":50,"h":53.13010235415598}
    let lab = Lab {
        l: 50.0,
        a: 30.0,
        b: 40.0,
        alpha: None,
    };
    let lch = Lch::from(lab);
    common::assert_close(lch.l, 50.0, EPS);
    common::assert_close(lch.c, 50.0, EPS);
    common::assert_close(lch.h, 53.13010235415598, EPS);
}

#[test]
fn lab_a_neg30_b40_to_lch() {
    // c.lch({mode:'lab', l:50, a:-30, b:40})
    // -> {"l":50,"c":50,"h":126.86989764584402}
    let lab = Lab {
        l: 50.0,
        a: -30.0,
        b: 40.0,
        alpha: None,
    };
    let lch = Lch::from(lab);
    common::assert_close(lch.l, 50.0, EPS);
    common::assert_close(lch.c, 50.0, EPS);
    common::assert_close(lch.h, 126.86989764584402, EPS);
}

#[test]
fn lab_a30_b_neg40_to_lch_wraps_negative_atan2() {
    // c.lch({mode:'lab', l:50, a:30, b:-40})
    // -> {"l":50,"c":50,"h":306.86989764584405}
    // atan2(-40, 30) is negative in radians; normalizeHue wraps to [0,360).
    let lab = Lab {
        l: 50.0,
        a: 30.0,
        b: -40.0,
        alpha: None,
    };
    let lch = Lch::from(lab);
    common::assert_close(lch.l, 50.0, EPS);
    common::assert_close(lch.c, 50.0, EPS);
    common::assert_close(lch.h, 306.86989764584405, EPS);
}

#[test]
fn lab_both_negative_to_lch() {
    // c.lch({mode:'lab', l:50, a:-30, b:-40})
    // -> {"l":50,"c":50,"h":233.13010235415598}
    let lab = Lab {
        l: 50.0,
        a: -30.0,
        b: -40.0,
        alpha: None,
    };
    let lch = Lch::from(lab);
    common::assert_close(lch.l, 50.0, EPS);
    common::assert_close(lch.c, 50.0, EPS);
    common::assert_close(lch.h, 233.13010235415598, EPS);
}

#[test]
fn lab_achromatic_yields_nan_hue() {
    // c.lch({mode:'lab', l:50, a:0, b:0}) -> {"l":50,"c":0}
    // culori omits `h`; we use NaN sentinel.
    let lab = Lab {
        l: 50.0,
        a: 0.0,
        b: 0.0,
        alpha: None,
    };
    let lch = Lch::from(lab);
    common::assert_close(lch.l, 50.0, EPS);
    common::assert_close(lch.c, 0.0, EPS);
    assert!(lch.h.is_nan(), "expected NaN hue, got {}", lch.h);
}

#[test]
fn lch_nan_hue_back_to_lab_no_nan_propagation() {
    // c.lab({mode:'lch', l:50, c:0}) -> {"mode":"lab","l":50,"a":0,"b":0}
    let lch = Lch {
        l: 50.0,
        c: 0.0,
        h: f64::NAN,
        alpha: None,
    };
    let lab = Lab::from(lch);
    common::assert_close(lab.l, 50.0, EPS);
    common::assert_close(lab.a, 0.0, EPS);
    common::assert_close(lab.b, 0.0, EPS);
}

#[test]
fn lab_lch_round_trip() {
    // Forward and inverse around a non-achromatic point.
    let lab = Lab {
        l: 60.0,
        a: 25.0,
        b: -15.0,
        alpha: None,
    };
    let lch = Lch::from(lab);
    let back = Lab::from(lch);
    common::assert_close(back.l, lab.l, EPS);
    common::assert_close(back.a, lab.a, EPS);
    common::assert_close(back.b, lab.b, EPS);
}

#[test]
fn srgb_red_via_xyz65_to_lch() {
    // c.lch({mode:'rgb', r:1, g:0, b:0})
    // -> {"l":54.29054294696968,"c":106.83719104365966,"h":40.85766878213079}
    let rgb = Rgb {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        alpha: None,
    };
    let lch = Lch::from_xyz65(rgb.to_xyz65());
    common::assert_close(lch.l, 54.29054294696968, CHAIN_EPS);
    common::assert_close(lch.c, 106.83719104365966, CHAIN_EPS);
    common::assert_close(lch.h, 40.85766878213079, CHAIN_EPS);
}

#[test]
fn srgb_blue_via_xyz65_to_lch() {
    // c.lch({mode:'rgb', r:0, g:0, b:1})
    // -> {"l":29.568297153444703,"c":131.2014771995311,"h":301.36428148973533}
    let rgb = Rgb {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        alpha: None,
    };
    let lch = Lch::from_xyz65(rgb.to_xyz65());
    common::assert_close(lch.l, 29.568297153444703, CHAIN_EPS);
    common::assert_close(lch.c, 131.2014771995311, CHAIN_EPS);
    common::assert_close(lch.h, 301.36428148973533, CHAIN_EPS);
}

#[test]
fn srgb_arbitrary_via_xyz65_to_lch() {
    // c.lch({mode:'rgb', r:0.3, g:0.6, b:0.9})
    // -> {"l":60.991980887973384,"c":46.89450657064141,"h":262.8269282419191}
    let rgb = Rgb {
        r: 0.3,
        g: 0.6,
        b: 0.9,
        alpha: None,
    };
    let lch = Lch::from_xyz65(rgb.to_xyz65());
    common::assert_close(lch.l, 60.991980887973384, CHAIN_EPS);
    common::assert_close(lch.c, 46.89450657064141, CHAIN_EPS);
    common::assert_close(lch.h, 262.8269282419191, CHAIN_EPS);
}

#[test]
fn lch_to_xyz65_via_hub_matches_lab_path() {
    let lch = Lch {
        l: 60.991980887973384,
        c: 46.89450657064141,
        h: 262.8269282419191,
        alpha: None,
    };
    let via_hub: Xyz65 = lch.to_xyz65();
    let via_lab = Lab::from(lch).to_xyz65();
    common::assert_close(via_hub.x, via_lab.x, EPS);
    common::assert_close(via_hub.y, via_lab.y, EPS);
    common::assert_close(via_hub.z, via_lab.z, EPS);
}

#[test]
fn lch_alpha_preserved_through_lab() {
    let lch = Lch {
        l: 50.0,
        c: 30.0,
        h: 120.0,
        alpha: Some(0.42),
    };
    let lab: Lab = lch.into();
    assert_eq!(lab.alpha, Some(0.42));
    let back: Lch = lab.into();
    assert_eq!(back.alpha, Some(0.42));
}

#[test]
fn lch_alpha_preserved_with_nan_hue() {
    // Achromatic input must keep alpha through the NaN-hue branch.
    let lab = Lab {
        l: 30.0,
        a: 0.0,
        b: 0.0,
        alpha: Some(0.7),
    };
    let lch = Lch::from(lab);
    assert!(lch.h.is_nan());
    assert_eq!(lch.alpha, Some(0.7));
    let back = Lab::from(lch);
    assert_eq!(back.alpha, Some(0.7));
}
