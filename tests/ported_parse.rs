//! Ported tests for the CSS Color Module 4 parser.
//!
//! Each expected value was produced by running culori 4.0.2's `parse()`
//! on the same input string (see `node -e` snippets in the Phase E
//! report). Channel-mode mapping: culori's modes `rgb`, `hsl`, `hwb`,
//! `lab`, `lch`, `oklab`, `oklch`, `lrgb`, `xyz65`, `xyz50` correspond
//! to the matching Rust struct in `culor::spaces`.

use culor::parse;
use culor::spaces::{Hsl, Hwb, Lab, Lch, LinearRgb, Oklab, Oklch, Rgb, Xyz50, Xyz65};
use culor::Color;

#[path = "common/mod.rs"]
mod common;

const EPS: f64 = 1e-12;

fn rgb(c: Color) -> Rgb {
    let Color::Rgb(c) = c else {
        panic!("expected Rgb, got {c:?}")
    };
    c
}
fn hsl(c: Color) -> Hsl {
    let Color::Hsl(c) = c else {
        panic!("expected Hsl, got {c:?}")
    };
    c
}
fn hwb(c: Color) -> Hwb {
    let Color::Hwb(c) = c else {
        panic!("expected Hwb, got {c:?}")
    };
    c
}
fn lab(c: Color) -> Lab {
    let Color::Lab(c) = c else {
        panic!("expected Lab, got {c:?}")
    };
    c
}
fn lch(c: Color) -> Lch {
    let Color::Lch(c) = c else {
        panic!("expected Lch, got {c:?}")
    };
    c
}
fn oklab(c: Color) -> Oklab {
    let Color::Oklab(c) = c else {
        panic!("expected Oklab, got {c:?}")
    };
    c
}
fn oklch(c: Color) -> Oklch {
    let Color::Oklch(c) = c else {
        panic!("expected Oklch, got {c:?}")
    };
    c
}
fn lrgb(c: Color) -> LinearRgb {
    let Color::LinearRgb(c) = c else {
        panic!("expected LinearRgb, got {c:?}")
    };
    c
}
fn xyz65(c: Color) -> Xyz65 {
    let Color::Xyz65(c) = c else {
        panic!("expected Xyz65, got {c:?}")
    };
    c
}
fn xyz50(c: Color) -> Xyz50 {
    let Color::Xyz50(c) = c else {
        panic!("expected Xyz50, got {c:?}")
    };
    c
}

#[test]
fn named_red() {
    let c = rgb(parse("red").unwrap());
    common::assert_close(c.r, 1.0, EPS);
    common::assert_close(c.g, 0.0, EPS);
    common::assert_close(c.b, 0.0, EPS);
    assert_eq!(c.alpha, None);
}

#[test]
fn named_cornflowerblue_case_insensitive() {
    // culori: Cornflowerblue -> r=100/255, g=149/255, b=237/255
    let c = rgb(parse("Cornflowerblue").unwrap());
    common::assert_close(c.r, 100.0 / 255.0, EPS);
    common::assert_close(c.g, 149.0 / 255.0, EPS);
    common::assert_close(c.b, 237.0 / 255.0, EPS);
}

#[test]
fn named_uppercase() {
    assert_eq!(parse("RED"), parse("red"));
}

#[test]
fn named_with_surrounding_whitespace_fails() {
    // culori does not trim before parseNamed; matches the strict
    // behavior we mirror here.
    assert!(parse("   red   ").is_none());
    assert!(parse(" red").is_none());
}

#[test]
fn transparent_keyword() {
    let c = rgb(parse("transparent").unwrap());
    assert_eq!(c.r, 0.0);
    assert_eq!(c.g, 0.0);
    assert_eq!(c.b, 0.0);
    assert_eq!(c.alpha, Some(0.0));
}

#[test]
fn currentcolor_unknown() {
    // Per culori, `currentcolor` is not recognized.
    assert!(parse("currentcolor").is_none());
}

#[test]
fn hex_three_digit() {
    assert_eq!(parse("#f00"), parse("red"));
}

#[test]
fn hex_six_digit() {
    assert_eq!(parse("#ff0000"), parse("#f00"));
}

#[test]
fn hex_eight_digit_with_alpha() {
    let c = rgb(parse("#ff0000ff").unwrap());
    common::assert_close(c.r, 1.0, EPS);
    assert_eq!(c.alpha, Some(1.0));
}

#[test]
fn hex_four_digit_zero_alpha() {
    let c = rgb(parse("#0000").unwrap());
    assert_eq!(c.r, 0.0);
    assert_eq!(c.alpha, Some(0.0));
}

#[test]
fn hex_with_whitespace_fails() {
    assert!(parse(" #f00 ").is_none());
    assert!(parse("# f00").is_none());
}

#[test]
fn rgb_modern() {
    let c = rgb(parse("rgb(255 0 0)").unwrap());
    common::assert_close(c.r, 1.0, EPS);
    common::assert_close(c.g, 0.0, EPS);
    common::assert_close(c.b, 0.0, EPS);
    assert_eq!(c.alpha, None);
}

#[test]
fn rgb_modern_with_surrounding_whitespace() {
    // Outer whitespace is trimmed before dispatch, matching culori.
    let c = rgb(parse(" rgb(255 0 0) ").unwrap());
    common::assert_close(c.r, 1.0, EPS);
    common::assert_close(c.g, 0.0, EPS);
    common::assert_close(c.b, 0.0, EPS);
    assert_eq!(c.alpha, None);
}

#[test]
fn rgb_modern_with_alpha() {
    let c = rgb(parse("rgb(255 0 0 / 0.5)").unwrap());
    assert_eq!(c.alpha, Some(0.5));
}

#[test]
fn rgb_modern_with_pct_alpha() {
    let c = rgb(parse("rgb(255 0 0 / 50%)").unwrap());
    assert_eq!(c.alpha, Some(0.5));
}

#[test]
fn rgb_legacy_numbers() {
    let c = rgb(parse("rgb(255, 0, 0)").unwrap());
    common::assert_close(c.r, 1.0, EPS);
    assert_eq!(c.alpha, None);
}

#[test]
fn rgb_legacy_percentages() {
    let c = rgb(parse("rgb(100%, 0%, 0%)").unwrap());
    common::assert_close(c.r, 1.0, EPS);
    common::assert_close(c.g, 0.0, EPS);
    common::assert_close(c.b, 0.0, EPS);
}

#[test]
fn rgba_legacy() {
    let c = rgb(parse("rgba(255, 0, 0, 0.5)").unwrap());
    common::assert_close(c.r, 1.0, EPS);
    assert_eq!(c.alpha, Some(0.5));
}

#[test]
fn rgb_function_case_sensitive() {
    // Per culori: function names are not lowercased before tokenizing.
    assert!(parse("RGB(255 0 0)").is_none());
    assert!(parse("Rgb(255 0 0)").is_none());
}

#[test]
fn rgb_none_channel_is_nan() {
    let c = rgb(parse("rgb(none 0 0)").unwrap());
    assert!(c.r.is_nan());
    assert_eq!(c.g, 0.0);
    assert_eq!(c.b, 0.0);
    assert_eq!(c.alpha, None);
}

#[test]
fn rgb_none_alpha_is_none() {
    let c = rgb(parse("rgb(255 0 0 / none)").unwrap());
    assert_eq!(c.r, 1.0);
    assert_eq!(c.alpha, None);
}

#[test]
fn rgb_oor_passes_through() {
    // Out-of-range values are not clamped during parsing.
    let c = rgb(parse("rgb(300 0 0)").unwrap());
    common::assert_close(c.r, 300.0 / 255.0, EPS);
}

#[test]
fn rgb_too_few_args() {
    assert!(parse("rgb(255 0)").is_none());
    assert!(parse("rgb(not enough)").is_none());
}

#[test]
fn rgb_round_trip_named_vs_function() {
    // Named -> alpha=None; rgb() with three args -> alpha=None.
    assert_eq!(parse("red"), parse("rgb(255 0 0)"));
    assert_eq!(parse("red"), parse("#ff0000"));
}

#[test]
fn hsl_modern() {
    let c = hsl(parse("hsl(120deg 50% 50%)").unwrap());
    assert_eq!(c.h, 120.0);
    assert_eq!(c.s, 0.5);
    assert_eq!(c.l, 0.5);
}

#[test]
fn hsl_legacy() {
    let c = hsl(parse("hsl(120, 50%, 50%)").unwrap());
    assert_eq!(c.h, 120.0);
    assert_eq!(c.s, 0.5);
    assert_eq!(c.l, 0.5);
}

#[test]
fn hsl_turn_unit_converts_to_degrees() {
    let c = hsl(parse("hsl(0.5turn 100% 50%)").unwrap());
    assert_eq!(c.h, 180.0);
    assert_eq!(c.s, 1.0);
    assert_eq!(c.l, 0.5);
}

#[test]
fn hsl_with_alpha() {
    let c = hsl(parse("hsl(120 50% 50% / 0.5)").unwrap());
    assert_eq!(c.alpha, Some(0.5));
}

#[test]
fn hwb_modern() {
    let c = hwb(parse("hwb(120 30% 30%)").unwrap());
    assert_eq!(c.h, 120.0);
    assert_eq!(c.w, 0.3);
    assert_eq!(c.b, 0.3);
}

#[test]
fn hwb_legacy_form_unsupported() {
    // hwb has no legacy comma form; commas are an error.
    assert!(parse("hwb(120, 30%, 30%)").is_none());
}

#[test]
fn lab_pct_l() {
    let c = lab(parse("lab(50% 40 -30)").unwrap());
    assert_eq!(c.l, 50.0);
    assert_eq!(c.a, 40.0);
    assert_eq!(c.b, -30.0);
}

#[test]
fn lab_number_l() {
    let c = lab(parse("lab(50 40 -30)").unwrap());
    assert_eq!(c.l, 50.0);
}

#[test]
fn lab_pct_ab_scales_to_125() {
    let c = lab(parse("lab(50% 50% -50% / 50%)").unwrap());
    assert_eq!(c.a, 62.5);
    assert_eq!(c.b, -62.5);
    assert_eq!(c.alpha, Some(0.5));
}

#[test]
fn lch_basic() {
    let c = lch(parse("lch(50% 40 30deg)").unwrap());
    assert_eq!(c.l, 50.0);
    assert_eq!(c.c, 40.0);
    assert_eq!(c.h, 30.0);
}

#[test]
fn oklab_number_l() {
    let c = oklab(parse("oklab(0.5 0.1 -0.1)").unwrap());
    assert_eq!(c.l, 0.5);
    assert_eq!(c.a, 0.1);
    assert_eq!(c.b, -0.1);
}

#[test]
fn oklab_pct_l_maps_to_unit() {
    let c = oklab(parse("oklab(50% 0.1 -0.1)").unwrap());
    assert_eq!(c.l, 0.5);
}

#[test]
fn oklch_basic() {
    let c = oklch(parse("oklch(70% 0.15 30deg)").unwrap());
    assert_eq!(c.l, 0.7);
    assert_eq!(c.c, 0.15);
    assert_eq!(c.h, 30.0);
}

#[test]
fn color_srgb() {
    let c = rgb(parse("color(srgb 1 0 0)").unwrap());
    assert_eq!(c.r, 1.0);
    assert_eq!(c.g, 0.0);
    assert_eq!(c.b, 0.0);
}

#[test]
fn color_srgb_linear() {
    let c = lrgb(parse("color(srgb-linear 1 0 0)").unwrap());
    assert_eq!(c.r, 1.0);
    assert_eq!(c.g, 0.0);
    assert_eq!(c.b, 0.0);
}

#[test]
fn color_xyz_aliases_to_d65() {
    let c = xyz65(parse("color(xyz 0.5 0.5 0.5)").unwrap());
    assert_eq!(c.x, 0.5);
    let c2 = xyz65(parse("color(xyz-d65 0.5 0.5 0.5)").unwrap());
    assert_eq!(c2.x, 0.5);
}

#[test]
fn color_xyz_d50() {
    let c = xyz50(parse("color(xyz-d50 0.5 0.5 0.5)").unwrap());
    assert_eq!(c.x, 0.5);
}

#[test]
fn color_unsupported_profiles_return_none() {
    assert!(parse("color(display-p3 1 0 0)").is_none());
    assert!(parse("color(rec2020 1 0 0)").is_none());
    assert!(parse("color(prophoto-rgb 1 0 0)").is_none());
    assert!(parse("color(a98-rgb 1 0 0)").is_none());
}

#[test]
fn unknown_input_returns_none() {
    assert!(parse("not a color").is_none());
    assert!(parse("").is_none());
}

#[test]
fn lab_none_channels_become_nan() {
    let c = lab(parse("lab(none none none / 0.5)").unwrap());
    assert!(c.l.is_nan());
    assert!(c.a.is_nan());
    assert!(c.b.is_nan());
    assert_eq!(c.alpha, Some(0.5));
}

// Discovered while building the parse fixture against culori's
// `parse()` (Phase E.2). The legacy comma-form parser used to stripe
// out commas without verifying the input had legal legacy structure —
// it accepted modern-style 4-positional forms (`rgb(255 0 0 0)`),
// trailing/leading commas, and mixed comma/space separators.
#[test]
fn rgb_modern_four_positional_no_slash_rejected() {
    assert!(parse("rgb(255 0 0 0)").is_none());
    assert!(parse("rgb(255 0 0 0.5)").is_none());
    assert!(parse("hsl(180 50% 50% 0.5)").is_none());
}

#[test]
fn rgb_legacy_trailing_or_misplaced_commas_rejected() {
    assert!(parse("rgb(255, 0, 0,)").is_none());
    assert!(parse("rgb(255 0 0,)").is_none());
}

#[test]
fn rgb_legacy_mixed_separators_rejected() {
    assert!(parse("hsl(180 50%, 50%)").is_none());
    assert!(parse("hsl(180, 50% 50%)").is_none());
}

// Culori's `parseRgbLegacy.js` uses two regexes (`rgb_num_old` and
// `rgb_per_old`) so legacy form requires all three RGB channels to
// share the same type. Mixed forms like `rgb(50%, 50, 0%)` are
// rejected.
#[test]
fn rgb_legacy_requires_uniform_channel_types() {
    assert!(parse("rgb(50%, 50, 0%)").is_none());
    assert!(parse("rgb(50%, 50%, 0)").is_none());
    assert!(parse("rgb(50, 50%, 0)").is_none());
    // All-num and all-per stay valid.
    assert_eq!(rgb(parse("rgb(255, 0, 0)").unwrap()).r, 1.0);
    assert_eq!(rgb(parse("rgb(100%, 0%, 0%)").unwrap()).r, 1.0);
}

// Culori's `parseHslLegacy.js` clamps S and L to [0, 1] (legacy form
// only) and requires both to be percentages. Modern syntax preserves
// out-of-range values and accepts bare numbers.
#[test]
fn hsl_legacy_clamps_saturation_lightness() {
    let neg = hsl(parse("hsl(180, -50%, 50%)").unwrap());
    assert_eq!(neg.s, 0.0);
    let big = hsl(parse("hsl(180, 150%, 50%)").unwrap());
    assert_eq!(big.s, 1.0);
    // Modern form does NOT clamp.
    let neg_modern = hsl(parse("hsl(180 -50% 50%)").unwrap());
    assert_eq!(neg_modern.s, -0.5);
    let big_modern = hsl(parse("hsl(180 150% 50%)").unwrap());
    assert_eq!(big_modern.s, 1.5);
}

#[test]
fn hsl_legacy_requires_percentage_sl() {
    // Bare numbers in legacy form are rejected.
    assert!(parse("hsl(180, 50, 50)").is_none());
    assert!(parse("hsl(180, 0.5, 0.5)").is_none());
    // Modern still accepts bare numbers (treated as percentages).
    let bare = hsl(parse("hsl(180 50 50)").unwrap());
    assert_eq!(bare.s, 0.5);
    assert_eq!(bare.l, 0.5);
}
