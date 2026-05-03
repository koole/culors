//! Tests for the CSS-style image filters.
//!
//! Reference values come from culori 4.0.2 invoked through Node — each
//! expected number was produced by `culori.filter*(amount)(c.parse(...))`
//! and pasted in.

use culor::{
    filter_brightness, filter_contrast, filter_grayscale, filter_hue_rotate, filter_invert,
    filter_saturate, filter_sepia, parse, Color,
};

const EPS: f64 = 1e-12;

#[track_caller]
fn approx(label: &str, a: f64, b: f64) {
    let diff = (a - b).abs();
    assert!(
        diff <= EPS,
        "{label}: actual={a}, expected={b}, diff={diff}"
    );
}

#[track_caller]
fn assert_rgb(out: Color, r: f64, g: f64, b: f64) {
    let Color::Rgb(c) = out else {
        panic!("expected Color::Rgb, got {out:?}")
    };
    approx("r", c.r, r);
    approx("g", c.g, g);
    approx("b", c.b, b);
}

#[track_caller]
fn assert_rgb_alpha(out: Color, r: f64, g: f64, b: f64, alpha: Option<f64>) {
    let Color::Rgb(c) = out else {
        panic!("expected Color::Rgb, got {out:?}")
    };
    approx("r", c.r, r);
    approx("g", c.g, g);
    approx("b", c.b, b);
    assert_eq!(c.alpha, alpha, "alpha mismatch");
}

fn red() -> Color {
    parse("red").unwrap()
}
fn lime() -> Color {
    parse("lime").unwrap()
}
fn blue() -> Color {
    parse("blue").unwrap()
}
fn black() -> Color {
    parse("black").unwrap()
}
fn white() -> Color {
    parse("white").unwrap()
}

// ----- brightness -------------------------------------------------------

#[test]
fn brightness_identity() {
    assert_rgb(filter_brightness(1.0)(&red()), 1.0, 0.0, 0.0);
}

#[test]
fn brightness_half() {
    assert_rgb(filter_brightness(0.5)(&red()), 0.5, 0.0, 0.0);
    assert_rgb(filter_brightness(0.5)(&lime()), 0.0, 0.5, 0.0);
}

#[test]
fn brightness_zero_yields_black() {
    assert_rgb(filter_brightness(0.0)(&red()), 0.0, 0.0, 0.0);
    assert_rgb(filter_brightness(0.0)(&white()), 0.0, 0.0, 0.0);
}

#[test]
fn brightness_above_one_extends_gamut() {
    assert_rgb(filter_brightness(2.0)(&red()), 2.0, 0.0, 0.0);
    assert_rgb(filter_brightness(2.0)(&lime()), 0.0, 2.0, 0.0);
}

#[test]
fn brightness_negative_is_clamped_to_zero() {
    assert_rgb(filter_brightness(-0.5)(&red()), 0.0, 0.0, 0.0);
}

#[test]
fn brightness_preserves_alpha() {
    let with_alpha = parse("rgba(255, 0, 0, 0.5)").unwrap();
    assert_rgb_alpha(
        filter_brightness(0.5)(&with_alpha),
        0.5,
        0.0,
        0.0,
        Some(0.5),
    );
}

// ----- contrast ---------------------------------------------------------

#[test]
fn contrast_identity() {
    assert_rgb(filter_contrast(1.0)(&red()), 1.0, 0.0, 0.0);
}

#[test]
fn contrast_low_pushes_toward_grey() {
    assert_rgb(filter_contrast(0.5)(&red()), 0.75, 0.25, 0.25);
}

#[test]
fn contrast_zero_collapses_to_grey() {
    assert_rgb(filter_contrast(0.0)(&red()), 0.5, 0.5, 0.5);
    assert_rgb(filter_contrast(0.0)(&lime()), 0.5, 0.5, 0.5);
    assert_rgb(filter_contrast(0.0)(&white()), 0.5, 0.5, 0.5);
}

#[test]
fn contrast_high_amplifies_around_half() {
    assert_rgb(filter_contrast(2.0)(&red()), 1.5, -0.5, -0.5);
    assert_rgb(filter_contrast(2.0)(&white()), 1.5, 1.5, 1.5);
}

#[test]
fn contrast_negative_is_clamped() {
    assert_rgb(filter_contrast(-1.0)(&red()), 0.5, 0.5, 0.5);
}

#[test]
fn contrast_lime_zero_seven() {
    assert_rgb(
        filter_contrast(0.7)(&lime()),
        0.150_000_000_000_000_02,
        0.85,
        0.150_000_000_000_000_02,
    );
}

// ----- invert -----------------------------------------------------------

#[test]
fn invert_full_swaps_channels() {
    assert_rgb(filter_invert(1.0)(&red()), 0.0, 1.0, 1.0);
    assert_rgb(filter_invert(1.0)(&white()), 0.0, 0.0, 0.0);
    assert_rgb(filter_invert(1.0)(&black()), 1.0, 1.0, 1.0);
}

#[test]
fn invert_zero_is_identity() {
    assert_rgb(filter_invert(0.0)(&red()), 1.0, 0.0, 0.0);
}

#[test]
fn invert_half_yields_grey() {
    assert_rgb(filter_invert(0.5)(&red()), 0.5, 0.5, 0.5);
}

#[test]
fn invert_clamps_above_one() {
    assert_rgb(filter_invert(2.0)(&red()), 0.0, 1.0, 1.0);
}

#[test]
fn invert_clamps_below_zero() {
    assert_rgb(filter_invert(-1.0)(&red()), 1.0, 0.0, 0.0);
}

#[test]
fn invert_preserves_alpha() {
    let with_alpha = parse("rgba(255, 0, 0, 0.5)").unwrap();
    assert_rgb_alpha(filter_invert(0.5)(&with_alpha), 0.5, 0.5, 0.5, Some(0.5));
}

// ----- hue-rotate -------------------------------------------------------

#[test]
fn hue_rotate_zero_is_identity() {
    assert_rgb(filter_hue_rotate(0.0)(&red()), 1.0, 0.0, 0.0);
}

#[test]
fn hue_rotate_red_90() {
    assert_rgb(
        filter_hue_rotate(90.0)(&red()),
        5.551_115_123_125_783e-17,
        0.356,
        -0.574_000_000_000_000_1,
    );
}

#[test]
fn hue_rotate_red_180() {
    assert_rgb(
        filter_hue_rotate(180.0)(&red()),
        -0.574_000_000_000_000_1,
        0.426,
        0.425_999_999_999_999_9,
    );
}

#[test]
fn hue_rotate_lime_90() {
    assert_rgb(filter_hue_rotate(90.0)(&lime()), 0.0, 0.855, 1.43);
}

#[test]
fn hue_rotate_blue_60() {
    assert_rgb(
        filter_hue_rotate(60.0)(&blue()),
        0.839_671_574_711_959_1,
        -0.209_085_189_270_996_12,
        0.598_353_829_072_479_7,
    );
}

// ----- saturate ---------------------------------------------------------

#[test]
fn saturate_zero_desaturates_to_luminance() {
    assert_rgb(filter_saturate(0.0)(&red()), 0.213, 0.213, 0.213);
}

#[test]
fn saturate_one_is_identity() {
    assert_rgb(filter_saturate(1.0)(&red()), 1.0, 0.0, 0.0);
}

#[test]
fn saturate_two_amplifies() {
    assert_rgb(
        filter_saturate(2.0)(&red()),
        1.787_000_000_000_000_1,
        -0.213,
        -0.213,
    );
}

#[test]
fn saturate_neutral_grey_stays_neutral() {
    let grey = parse("#808080").unwrap();
    assert_rgb(
        filter_saturate(5.0)(&grey),
        0.501_960_784_313_725_5,
        0.501_960_784_313_725_5,
        0.501_960_784_313_726_1,
    );
}

// ----- grayscale --------------------------------------------------------

#[test]
fn grayscale_zero_is_identity() {
    assert_rgb(filter_grayscale(0.0)(&red()), 1.0, 0.0, 0.0);
}

#[test]
fn grayscale_one_yields_luminance() {
    assert_rgb(filter_grayscale(1.0)(&red()), 0.2126, 0.2126, 0.2126);
    assert_rgb(filter_grayscale(1.0)(&lime()), 0.7152, 0.7152, 0.7152);
    assert_rgb(filter_grayscale(1.0)(&blue()), 0.0722, 0.0722, 0.0722);
}

#[test]
fn grayscale_half_blends() {
    assert_rgb(
        filter_grayscale(0.5)(&red()),
        0.606_300_000_000_000_1,
        0.1063,
        0.1063,
    );
}

// ----- sepia ------------------------------------------------------------

#[test]
fn sepia_zero_is_identity() {
    assert_rgb(filter_sepia(0.0)(&red()), 1.0, 0.0, 0.0);
}

#[test]
fn sepia_one_full() {
    assert_rgb(filter_sepia(1.0)(&red()), 0.393, 0.349, 0.272);
}

#[test]
fn sepia_half() {
    assert_rgb(filter_sepia(0.5)(&red()), 0.6965, 0.1745, 0.136);
}

#[test]
fn sepia_white() {
    assert_rgb(filter_sepia(1.0)(&white()), 1.351, 1.203, 0.937);
}

#[test]
fn sepia_black_is_black() {
    assert_rgb(filter_sepia(1.0)(&black()), 0.0, 0.0, 0.0);
}

// ----- non-RGB inputs ---------------------------------------------------

#[test]
fn brightness_accepts_hsl_input() {
    // hsl(0 100% 50%) is sRGB red. After brightness 0.5, R = 0.5 in sRGB.
    let red_hsl = parse("hsl(0 100% 50%)").unwrap();
    assert_rgb(filter_brightness(0.5)(&red_hsl), 0.5, 0.0, 0.0);
}

#[test]
fn invert_accepts_hwb_input() {
    let red_hwb = parse("hwb(0 0% 0%)").unwrap();
    assert_rgb(filter_invert(1.0)(&red_hwb), 0.0, 1.0, 1.0);
}
