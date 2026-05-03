//! Tests for the CSS Color Module 5 `color-mix()` parser.
//!
//! culori 4.0.2 does not implement `color-mix()`, so reference values
//! were produced by a hand-rolled W3C-spec port of the algorithm built
//! on top of culori's `converter()` (see
//! `fixtures-gen/color-mix-ref.mjs`). The script applies the spec's
//! premultiplied-alpha interpolation, hue-fixup strategies, and
//! sub-100% alpha multiplier. Cross-checked against `colorjs.io`'s
//! `Color.mix(..., {premultiplied: true})` for the alpha-bearing cases.

use culors::parse;
use culors::spaces::{Hsl, Hwb, Lab, Lch, LinearRgb, Oklab, Oklch, Rgb, Xyz50, Xyz65};
use culors::Color;

const EPS: f64 = 1e-12;

fn approx(a: f64, b: f64) -> bool {
    if a.is_nan() && b.is_nan() {
        return true;
    }
    (a - b).abs() < EPS
}

fn assert_close(label: &str, a: f64, b: f64) {
    assert!(approx(a, b), "{label}: {a} vs {b} (diff {})", (a - b).abs());
}

fn rgb(c: Color) -> Rgb {
    let Color::Rgb(c) = c else {
        panic!("expected Rgb, got {c:?}")
    };
    c
}
fn lrgb(c: Color) -> LinearRgb {
    let Color::LinearRgb(c) = c else {
        panic!("expected LinearRgb, got {c:?}")
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
fn srgb_equal_mix_red_blue() {
    let c = rgb(parse("color-mix(in srgb, red, blue)").unwrap());
    assert_close("r", c.r, 0.5);
    assert_close("g", c.g, 0.0);
    assert_close("b", c.b, 0.5);
    assert_eq!(c.alpha, None);
}

#[test]
fn srgb_weighted_first() {
    // p1=70 means c1 (red) gets 70% weight. t for c2 = 0.3.
    let c = rgb(parse("color-mix(in srgb, red 70%, blue)").unwrap());
    assert_close("r", c.r, 0.7);
    assert_close("g", c.g, 0.0);
    assert_close("b", c.b, 0.3);
    assert_eq!(c.alpha, None);
}

#[test]
fn srgb_weighted_second() {
    let c = rgb(parse("color-mix(in srgb, red, blue 70%)").unwrap());
    assert_close("r", c.r, 0.3);
    assert_close("b", c.b, 0.7);
}

#[test]
fn srgb_both_weighted_summing_100() {
    let c = rgb(parse("color-mix(in srgb, red 30%, blue 70%)").unwrap());
    assert_close("r", c.r, 0.3);
    assert_close("b", c.b, 0.7);
    assert_eq!(c.alpha, None);
}

#[test]
fn lab_equal_mix() {
    let c = lab(parse("color-mix(in lab, red, blue)").unwrap());
    assert_close("l", c.l, 41.92942005020719);
    assert_close("a", c.a, 74.54616349338983);
    assert_close("b", c.b, -21.069364863606836);
}

#[test]
fn lch_equal_mix() {
    let c = lch(parse("color-mix(in lch, red, blue)").unwrap());
    assert_close("l", c.l, 41.92942005020719);
    assert_close("c", c.c, 119.01933412159538);
    assert_close("h", c.h, 351.110975135933);
}

#[test]
fn oklab_equal_mix() {
    let c = oklab(parse("color-mix(in oklab, red, blue)").unwrap());
    assert_close("l", c.l, 0.5399845410479274);
    assert_close("a", c.a, 0.09620304662773833);
    assert_close("b", c.b, -0.09284094417349634);
}

#[test]
fn oklch_equal_mix() {
    let c = oklch(parse("color-mix(in oklch, red, blue)").unwrap());
    assert_close("l", c.l, 0.5399845410479274);
    assert_close("c", c.c, 0.2854488462199228);
    assert_close("h", c.h, 326.6429514479988);
}

#[test]
fn oklch_weighted() {
    let c = oklch(parse("color-mix(in oklch, red 70%, blue)").unwrap());
    assert_close("l", c.l, 0.5751728701973289);
    assert_close("c", c.c, 0.27434262925409797);
    assert_close("h", c.h, 351.67932298065045);
}

#[test]
fn hsl_equal_mix() {
    let c = hsl(parse("color-mix(in hsl, red, blue)").unwrap());
    assert_close("h", c.h, 300.0);
    assert_close("s", c.s, 1.0);
    assert_close("l", c.l, 0.5);
}

#[test]
fn hwb_equal_mix() {
    let c = hwb(parse("color-mix(in hwb, red, blue)").unwrap());
    assert_close("h", c.h, 300.0);
    assert_close("w", c.w, 0.0);
    assert_close("b", c.b, 0.0);
}

#[test]
fn hsl_shorter_hue_explicit() {
    let c = hsl(parse("color-mix(in hsl shorter hue, red, blue)").unwrap());
    assert_close("h", c.h, 300.0);
}

#[test]
fn hsl_longer_hue() {
    let c = hsl(parse("color-mix(in hsl longer hue, red, blue)").unwrap());
    assert_close("h", c.h, 120.0);
}

#[test]
fn hsl_increasing_hue() {
    let c = hsl(parse("color-mix(in hsl increasing hue, red, blue)").unwrap());
    assert_close("h", c.h, 120.0);
}

#[test]
fn hsl_decreasing_hue() {
    let c = hsl(parse("color-mix(in hsl decreasing hue, red, blue)").unwrap());
    assert_close("h", c.h, 300.0);
}

#[test]
fn oklch_longer_hue() {
    let c = oklch(parse("color-mix(in oklch longer hue, red, blue)").unwrap());
    assert_close("h", c.h, 146.64295144799883);
}

#[test]
fn srgb_transparent_with_red() {
    // Premultiplied path: transparent contributes nothing to colour
    // channels, so the result is full red at half alpha.
    let c = rgb(parse("color-mix(in srgb, transparent, red)").unwrap());
    assert_close("r", c.r, 1.0);
    assert_close("g", c.g, 0.0);
    assert_close("b", c.b, 0.0);
    assert_eq!(c.alpha, Some(0.5));
}

#[test]
fn srgb_sum_below_100_scales_alpha() {
    // Both 30%, sum = 60%, alpha multiplier = 0.6.
    let c = rgb(parse("color-mix(in srgb, red 30%, blue 30%)").unwrap());
    assert_close("r", c.r, 0.5);
    assert_close("b", c.b, 0.5);
    assert_eq!(c.alpha, Some(0.6));
}

#[test]
fn srgb_both_zero_returns_none() {
    assert!(parse("color-mix(in srgb, red 0%, blue 0%)").is_none());
}

#[test]
fn oklab_white_black_50_50() {
    let c = oklab(parse("color-mix(in oklab, white, black)").unwrap());
    assert_close("l", c.l, 0.5000000000000001);
    assert_close("a", c.a, 0.0);
    assert_close("b", c.b, 0.0);
}

#[test]
fn oklab_white_black_25_75() {
    let c = oklab(parse("color-mix(in oklab, white 25%, black 75%)").unwrap());
    assert_close("l", c.l, 0.25);
}

#[test]
fn srgb_hex_colors() {
    let c = rgb(parse("color-mix(in srgb, #ff0000, #0000ff)").unwrap());
    assert_close("r", c.r, 0.5);
    assert_close("b", c.b, 0.5);
}

#[test]
fn xyz_alias_and_d65() {
    let c = xyz65(parse("color-mix(in xyz, red, blue)").unwrap());
    assert_close("x", c.x, 0.2964357938338968);
    assert_close("y", c.y, 0.14241566061612193);
    assert_close("z", c.z, 0.48493148548262627);
}

#[test]
fn xyz_d50() {
    let c = xyz50(parse("color-mix(in xyz-d50, red, blue)").unwrap());
    assert_close("x", c.x, 0.28957209862372646);
    assert_close("y", c.y, 0.1415564911462012);
    assert_close("z", c.z, 0.3640116314507295);
}

#[test]
fn srgb_linear() {
    let c = lrgb(parse("color-mix(in srgb-linear, red, blue)").unwrap());
    assert_close("r", c.r, 0.5);
    assert_close("b", c.b, 0.5);
}

#[test]
fn srgb_two_semitransparent_inputs() {
    let c = rgb(parse("color-mix(in srgb, rgba(255 0 0 / 0.5), rgba(0 0 255 / 0.5))").unwrap());
    // Both halves contribute equally to premultiplied colour. Mixed
    // alpha = 0.5; unpremult brings each colour back to full strength.
    assert_close("r", c.r, 0.5);
    assert_close("b", c.b, 0.5);
    assert_eq!(c.alpha, Some(0.5));
}

#[test]
fn lch_longer_hue_with_weights() {
    let c = lch(parse("color-mix(in lch longer hue, red 30%, blue 70%)").unwrap());
    assert_close("l", c.l, 36.9849708915022);
    assert_close("c", c.c, 123.89219135276967);
    assert_close("h", c.h, 223.21229767745388);
}

#[test]
fn whitespace_tolerance() {
    let a = parse("color-mix(in srgb, red, blue)").unwrap();
    let b = parse("  color-mix(  in   srgb  ,   red  ,   blue  )  ").unwrap();
    let ar = rgb(a);
    let br = rgb(b);
    assert_close("r", ar.r, br.r);
    assert_close("g", ar.g, br.g);
    assert_close("b", ar.b, br.b);
}

#[test]
fn case_insensitive_method_keywords() {
    // CSS keywords are case-insensitive; `color-mix`, `in`, and the
    // hue-method words follow that rule. Color space identifiers are
    // matched in lower case.
    let c = rgb(parse("COLOR-MIX(IN srgb, red, blue)").unwrap());
    assert_close("r", c.r, 0.5);
    assert_close("b", c.b, 0.5);
    let c2 = hsl(parse("color-mix(in hsl LONGER hue, red, blue)").unwrap());
    assert_close("h", c2.h, 120.0);
}

#[test]
fn unsupported_space_returns_none() {
    // hsv is a culori extension, not a CSS Color 5 interpolation space;
    // display-p3 etc. are CSS spaces but culors doesn't ship them yet.
    assert!(parse("color-mix(in hsv, red, blue)").is_none());
    assert!(parse("color-mix(in display-p3, red, blue)").is_none());
    assert!(parse("color-mix(in not-a-space, red, blue)").is_none());
}

#[test]
fn malformed_inputs_return_none() {
    // Missing comma between method and colours.
    assert!(parse("color-mix(in srgb red, blue)").is_none());
    // Missing colour.
    assert!(parse("color-mix(in srgb, red)").is_none());
    // Three colours.
    assert!(parse("color-mix(in srgb, red, blue, green)").is_none());
    // Missing `in` keyword.
    assert!(parse("color-mix(srgb, red, blue)").is_none());
    // Hue-method without strategy keyword still requires `hue` literal.
    assert!(parse("color-mix(in hsl shorter, red, blue)").is_none());
    // Negative percentage rejected.
    assert!(parse("color-mix(in srgb, red -10%, blue)").is_none());
}

#[test]
fn percentage_can_appear_before_color() {
    // CSS Color 5 grammar allows the percentage on either side of the
    // <color>. Both placements should give the same result.
    let a = rgb(parse("color-mix(in srgb, 70% red, blue)").unwrap());
    let b = rgb(parse("color-mix(in srgb, red 70%, blue)").unwrap());
    assert_close("r", a.r, b.r);
    assert_close("g", a.g, b.g);
    assert_close("b", a.b, b.b);
}

#[test]
fn nested_color_mix_resolves() {
    // Recursive `parse()` means the inner color-mix() is evaluated
    // first and its result is the second mixing color.
    let inner = "color-mix(in srgb, red, blue)";
    let outer = format!("color-mix(in srgb, white, {inner})");
    let c = rgb(parse(&outer).unwrap());
    // inner is rgb(0.5, 0, 0.5); mixing 50/50 with white yields
    // rgb(0.75, 0.5, 0.75).
    assert_close("r", c.r, 0.75);
    assert_close("g", c.g, 0.5);
    assert_close("b", c.b, 0.75);
}

#[test]
fn color_mix_rejects_hue_method_on_rectangular_space() {
    // Per CSS Color Module 5, hue-interpolation-method is only valid for
    // polar color spaces (hsl, hwb, lch, oklch).
    assert!(parse("color-mix(in srgb shorter hue, red, blue)").is_none());
    assert!(parse("color-mix(in lab longer hue, red, blue)").is_none());
    assert!(parse("color-mix(in oklab increasing hue, red, blue)").is_none());
    assert!(parse("color-mix(in xyz shorter hue, red, blue)").is_none());
    assert!(parse("color-mix(in srgb-linear longer hue, red, blue)").is_none());
}

#[test]
fn color_mix_accepts_hue_method_only_on_polar_space() {
    assert!(parse("color-mix(in hsl shorter hue, red, blue)").is_some());
    assert!(parse("color-mix(in oklch longer hue, red, blue)").is_some());
    assert!(parse("color-mix(in lch increasing hue, red, blue)").is_some());
    assert!(parse("color-mix(in hwb decreasing hue, red, blue)").is_some());
}

#[test]
fn color_mix_rejects_strategy_without_hue_keyword() {
    // Per the grammar, the literal `hue` keyword is required after the strategy.
    assert!(parse("color-mix(in hsl shorter, red, blue)").is_none());
    assert!(parse("color-mix(in oklch longer, red, blue)").is_none());
}
