//! Tests for the generic `convert` function.
//!
//! Reference values produced by culori 4.0.2 and pasted verbatim. The
//! generic `convert` always routes through XYZ D65, so chained-transcendental
//! pairs (e.g. Oklch ↔ Rgb) need a looser epsilon than direct-impl pairs.

use culor::convert;
use culor::spaces::{Hsl, Lab, Lch, LinearRgb, Oklch, Rgb};

#[path = "common/mod.rs"]
mod common;

const EPS: f64 = 1e-12;
const CHAIN_EPS: f64 = 1e-10;

#[test]
fn convert_rgb_red_to_lab() {
    // c.lab({mode:'rgb', r:1, g:0, b:0})
    // -> {"l":54.29054294696968,"a":80.80492033462417,"b":69.89098825896278}
    let rgb = Rgb {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        alpha: None,
    };
    let lab: Lab = convert(rgb);
    common::assert_close(lab.l, 54.29054294696968, CHAIN_EPS);
    common::assert_close(lab.a, 80.80492033462417, CHAIN_EPS);
    common::assert_close(lab.b, 69.89098825896278, CHAIN_EPS);
}

#[test]
fn convert_rgb_green_to_lab() {
    // c.lab({mode:'rgb', r:0, g:1, b:0})
    // -> {"l":87.81853633115202,"a":-79.27108223854806,"b":80.99459785152247}
    let rgb = Rgb {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        alpha: None,
    };
    let lab: Lab = convert(rgb);
    common::assert_close(lab.l, 87.81853633115202, CHAIN_EPS);
    common::assert_close(lab.a, -79.27108223854806, CHAIN_EPS);
    common::assert_close(lab.b, 80.99459785152247, CHAIN_EPS);
}

#[test]
fn convert_rgb_blue_to_lab() {
    // c.lab({mode:'rgb', r:0, g:0, b:1})
    // -> {"l":29.568297153444703,"a":68.2874066521555,"b":-112.02971798617645}
    let rgb = Rgb {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        alpha: None,
    };
    let lab: Lab = convert(rgb);
    common::assert_close(lab.l, 29.568297153444703, CHAIN_EPS);
    common::assert_close(lab.a, 68.2874066521555, CHAIN_EPS);
    common::assert_close(lab.b, -112.02971798617645, CHAIN_EPS);
}

#[test]
fn convert_oklch_to_rgb_round_trip() {
    // Oklch -> Rgb via XYZ65, then Rgb -> Oklch via XYZ65; chained
    // transcendentals (Oklab cube roots and Lab cube roots) accumulate float
    // error, so we use 1e-9.
    let original = Oklch {
        l: 0.7,
        c: 0.15,
        h: 240.0,
        alpha: None,
    };
    let rgb: Rgb = convert(original);
    let back: Oklch = convert(rgb);
    common::assert_close(back.l, original.l, 1e-9);
    common::assert_close(back.c, original.c, 1e-9);
    common::assert_close(back.h, original.h, 1e-9);
}

#[test]
fn convert_rgb_to_hsl_takes_xyz_detour() {
    // `convert::<Rgb, Hsl>` routes Rgb -> Xyz65 -> Rgb -> Hsl, while
    // `Hsl::from(rgb)` is a direct path. The intermediate sRGB transfer
    // round-trip is not bit-exact, so the two paths agree only within float
    // precision. Verify they agree to CHAIN_EPS.
    let rgb = Rgb {
        r: 0.3,
        g: 0.6,
        b: 0.9,
        alpha: None,
    };
    let direct: Hsl = Hsl::from(rgb);
    let via: Hsl = convert(rgb);
    common::assert_close(via.h, direct.h, CHAIN_EPS);
    common::assert_close(via.s, direct.s, CHAIN_EPS);
    common::assert_close(via.l, direct.l, CHAIN_EPS);
}

#[test]
fn convert_lab_to_lch_vs_direct() {
    // `convert::<Lab, Lch>` routes Lab -> Xyz50 -> Xyz65 -> Xyz50 -> Lab ->
    // Lch (the Lab hub already goes via Xyz50/Xyz65). The Bradford round-trip
    // and the f_forward/f_inverse cube-root pair are not bit-exact, so the
    // generic path differs from the direct `Lch::from(lab)` impl by ~5e-7 on
    // L (and proportionally on C/H). The direct impl preserves Lab
    // identically; the generic impl drifts. This is the precision tradeoff
    // documented on `convert`.
    let lab = Lab {
        l: 50.0,
        a: 30.0,
        b: -40.0,
        alpha: None,
    };
    let direct: Lch = Lch::from(lab);
    let via: Lch = convert(lab);
    // Empirically the delta is ~8e-6 on L (and proportional on c/h) on
    // aarch64 darwin. Allow 1e-4 for headroom across architectures.
    common::assert_close(via.l, direct.l, 1e-4);
    common::assert_close(via.c, direct.c, 1e-4);
    common::assert_close(via.h, direct.h, 1e-4);
}

#[test]
fn convert_preserves_alpha() {
    let rgb = Rgb {
        r: 0.25,
        g: 0.5,
        b: 0.75,
        alpha: Some(0.42),
    };
    let lab: Lab = convert(rgb);
    assert_eq!(lab.alpha, Some(0.42));
    let back: Rgb = convert(lab);
    assert_eq!(back.alpha, Some(0.42));
}

#[test]
fn convert_preserves_none_alpha() {
    let rgb = Rgb {
        r: 0.1,
        g: 0.2,
        b: 0.3,
        alpha: None,
    };
    let oklch: Oklch = convert(rgb);
    assert_eq!(oklch.alpha, None);
}

#[test]
fn convert_lrgb_to_rgb_matches_direct_within_eps() {
    // Rgb <-> LinearRgb has a direct impl that simply applies the sRGB
    // transfer function. The generic path goes LinearRgb -> Xyz65 -> Rgb,
    // which composes the matrix round-trip. Result should agree to EPS.
    let lrgb = LinearRgb {
        r: 0.4,
        g: 0.5,
        b: 0.6,
        alpha: None,
    };
    let direct: Rgb = Rgb::from(lrgb);
    let via: Rgb = convert(lrgb);
    common::assert_close(via.r, direct.r, EPS);
    common::assert_close(via.g, direct.g, EPS);
    common::assert_close(via.b, direct.b, EPS);
}
