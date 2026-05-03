//! Tests for the WCAG luminance and contrast helpers.
//!
//! Reference values come from culori 4.0.2 via Node:
//!
//! ```bash
//! node --input-type=module -e "import * as c from 'culori'; \
//!   console.log(c.wcagLuminance(c.parse('red')))"
//! ```
//!
//! culori exposes `wcagLuminance(color)` and `wcagContrast(a, b)` as
//! plain functions, not factories. Both take a parsed color object and
//! route through `converter('lrgb')`, which means alpha is ignored.

#![allow(clippy::approx_constant, clippy::excessive_precision)]

use culors::{parse, wcag_contrast, wcag_luminance};

const EPS: f64 = 1e-10;

fn approx(a: f64, b: f64) -> bool {
    (a - b).abs() <= EPS
}

#[test]
fn luminance_white_is_one() {
    let c = parse("white").unwrap();
    assert!(approx(wcag_luminance(&c), 1.0));
}

#[test]
fn luminance_black_is_zero() {
    let c = parse("black").unwrap();
    assert_eq!(wcag_luminance(&c), 0.0);
}

#[test]
fn luminance_red_is_coefficient() {
    // Pure red in linear sRGB is (1, 0, 0), so luminance = 0.2126.
    let c = parse("red").unwrap();
    assert!(approx(wcag_luminance(&c), 0.2126));
}

#[test]
fn luminance_blue_is_coefficient() {
    let c = parse("blue").unwrap();
    assert!(approx(wcag_luminance(&c), 0.0722));
}

#[test]
fn luminance_named_green() {
    // CSS "green" is #008000, not #00ff00.
    let c = parse("green").unwrap();
    assert!(approx(wcag_luminance(&c), 0.1543834296814607));
}

#[test]
fn luminance_yellow() {
    let c = parse("yellow").unwrap();
    assert!(approx(wcag_luminance(&c), 0.9278));
}

#[test]
fn luminance_cyan() {
    let c = parse("cyan").unwrap();
    assert!(approx(wcag_luminance(&c), 0.7874));
}

#[test]
fn luminance_magenta() {
    let c = parse("magenta").unwrap();
    assert!(approx(wcag_luminance(&c), 0.2848));
}

#[test]
fn luminance_grey_808080() {
    let c = parse("#808080").unwrap();
    assert!(approx(wcag_luminance(&c), 0.2158605001138992));
}

#[test]
fn luminance_grey_777777() {
    let c = parse("#777777").unwrap();
    assert!(approx(wcag_luminance(&c), 0.184474994500441));
}

#[test]
fn luminance_rebeccapurple() {
    let c = parse("rebeccapurple").unwrap();
    assert!(approx(wcag_luminance(&c), 0.07492341159447033));
}

#[test]
fn luminance_ignores_alpha() {
    // culori's wcagLuminance passes through `converter('lrgb')` which
    // does not weight by alpha. Two reds at different alphas must
    // produce identical luminance.
    let opaque = parse("red").unwrap();
    let translucent = parse("rgba(255,0,0,0.5)").unwrap();
    assert_eq!(wcag_luminance(&opaque), wcag_luminance(&translucent));
}

#[test]
fn luminance_accepts_oklch_input() {
    // Anything convertible to lrgb must work; the wcag module routes
    // through the same path as culori's `converter('lrgb')`.
    let c = parse("oklch(0.7 0.15 30)").unwrap();
    assert!(approx(wcag_luminance(&c), 0.31883068247371255));
}

#[test]
fn contrast_white_black_is_21() {
    let w = parse("white").unwrap();
    let k = parse("black").unwrap();
    assert!(approx(wcag_contrast(&w, &k), 21.0));
}

#[test]
fn contrast_same_color_is_one() {
    let k = parse("black").unwrap();
    assert_eq!(wcag_contrast(&k, &k), 1.0);
    let w = parse("white").unwrap();
    assert!(approx(wcag_contrast(&w, &w), 1.0));
}

#[test]
fn contrast_red_blue() {
    let r = parse("red").unwrap();
    let b = parse("blue").unwrap();
    assert!(approx(wcag_contrast(&r, &b), 2.148936170212766));
}

#[test]
fn contrast_is_symmetric() {
    let r = parse("red").unwrap();
    let b = parse("blue").unwrap();
    assert_eq!(wcag_contrast(&r, &b), wcag_contrast(&b, &r));
}

#[test]
fn contrast_rebeccapurple_white() {
    let p = parse("rebeccapurple").unwrap();
    let w = parse("white").unwrap();
    assert!(approx(wcag_contrast(&p, &w), 8.405149896230322));
}

#[test]
fn contrast_close_greys() {
    let a = parse("#777").unwrap();
    let b = parse("#888").unwrap();
    assert!(approx(wcag_contrast(&a, &b), 1.2632533688247014));
}

#[test]
fn contrast_red_green() {
    let r = parse("red").unwrap();
    let g = parse("green").unwrap();
    assert!(approx(wcag_contrast(&r, &g), 1.28483997166146));
}

#[test]
fn contrast_cyan_magenta() {
    let c = parse("cyan").unwrap();
    let m = parse("magenta").unwrap();
    assert!(approx(wcag_contrast(&c, &m), 2.501194743130227));
}

#[test]
fn contrast_ignores_alpha() {
    // Translucent white over black still reads as 21:1 because alpha
    // is not part of the luminance calculation.
    let w_half = parse("rgba(255,255,255,0.5)").unwrap();
    let k = parse("black").unwrap();
    assert!(approx(wcag_contrast(&w_half, &k), 21.0));
}

#[test]
fn contrast_oklch_against_white() {
    let o = parse("oklch(0.7 0.15 30)").unwrap();
    let w = parse("white").unwrap();
    assert!(approx(wcag_contrast(&o, &w), 2.8468347398805034));
}
