//! Tests for the separable blend modes.
//!
//! Reference values come from culori 4.0.2 invoked through Node — see
//! the case tables at the bottom of each section. Each numeric expected
//! value was produced by `culori.blend([...], '<mode>')` and pasted in.

use culor::spaces::Rgb;
use culor::{blend, blend_str, parse, BlendMode, Color};

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
fn assert_rgb(out: Color, r: f64, g: f64, b: f64, alpha: f64) {
    let Color::Rgb(c) = out else {
        panic!("expected Color::Rgb, got {out:?}")
    };
    approx("r", c.r, r);
    approx("g", c.g, g);
    approx("b", c.b, b);
    approx("alpha", c.alpha.expect("blend always sets alpha"), alpha);
}

fn rgb(r: f64, g: f64, b: f64, a: Option<f64>) -> Color {
    Color::Rgb(Rgb { r, g, b, alpha: a })
}

// ----- normal -----------------------------------------------------------

#[test]
fn normal_replaces_backdrop_when_source_is_opaque() {
    let out = blend(
        &[parse("red").unwrap(), parse("blue").unwrap()],
        BlendMode::Normal,
    );
    assert_rgb(out, 0.0, 0.0, 1.0, 1.0);
}

#[test]
fn normal_with_half_alpha_does_porter_duff() {
    let bg = rgb(1.0, 0.0, 0.0, Some(0.5));
    let fg = rgb(0.0, 0.0, 1.0, Some(0.5));
    let out = blend(&[bg, fg], BlendMode::Normal);
    // culori: alpha = 0.75, r = 1/3, g = 0, b = 2/3
    assert_rgb(out, 1.0 / 3.0, 0.0, 2.0 / 3.0, 0.75);
}

// ----- multiply ---------------------------------------------------------

#[test]
fn multiply_red_white_is_red() {
    let out = blend(
        &[parse("red").unwrap(), parse("white").unwrap()],
        BlendMode::Multiply,
    );
    assert_rgb(out, 1.0, 0.0, 0.0, 1.0);
}

#[test]
fn multiply_red_black_is_black() {
    let out = blend(
        &[parse("red").unwrap(), parse("black").unwrap()],
        BlendMode::Multiply,
    );
    assert_rgb(out, 0.0, 0.0, 0.0, 1.0);
}

#[test]
fn multiply_red_blue_is_black() {
    let out = blend(
        &[parse("red").unwrap(), parse("blue").unwrap()],
        BlendMode::Multiply,
    );
    assert_rgb(out, 0.0, 0.0, 0.0, 1.0);
}

#[test]
fn multiply_chains_three_colors() {
    let out = blend(
        &[
            parse("red").unwrap(),
            parse("green").unwrap(),
            parse("blue").unwrap(),
        ],
        BlendMode::Multiply,
    );
    assert_rgb(out, 0.0, 0.0, 0.0, 1.0);
}

#[test]
fn multiply_with_half_alpha() {
    let bg = rgb(1.0, 0.0, 0.0, Some(0.5));
    let fg = rgb(0.0, 0.0, 1.0, Some(0.5));
    let out = blend(&[bg, fg], BlendMode::Multiply);
    assert_rgb(out, 1.0 / 3.0, 0.0, 1.0 / 3.0, 0.75);
}

// ----- screen -----------------------------------------------------------

#[test]
fn screen_red_white_is_white() {
    let out = blend(
        &[parse("red").unwrap(), parse("white").unwrap()],
        BlendMode::Screen,
    );
    assert_rgb(out, 1.0, 1.0, 1.0, 1.0);
}

#[test]
fn screen_red_black_is_red() {
    let out = blend(
        &[parse("red").unwrap(), parse("black").unwrap()],
        BlendMode::Screen,
    );
    assert_rgb(out, 1.0, 0.0, 0.0, 1.0);
}

#[test]
fn screen_red_blue_is_magenta() {
    let out = blend(
        &[parse("red").unwrap(), parse("blue").unwrap()],
        BlendMode::Screen,
    );
    assert_rgb(out, 1.0, 0.0, 1.0, 1.0);
}

// ----- darken / lighten -------------------------------------------------

#[test]
fn darken_red_blue_is_black() {
    let out = blend(
        &[parse("red").unwrap(), parse("blue").unwrap()],
        BlendMode::Darken,
    );
    assert_rgb(out, 0.0, 0.0, 0.0, 1.0);
}

#[test]
fn lighten_red_blue_is_magenta() {
    let out = blend(
        &[parse("red").unwrap(), parse("blue").unwrap()],
        BlendMode::Lighten,
    );
    assert_rgb(out, 1.0, 0.0, 1.0, 1.0);
}

#[test]
fn lighten_chains_three_colors_to_white_ish() {
    let out = blend(
        &[
            parse("red").unwrap(),
            parse("green").unwrap(),
            parse("blue").unwrap(),
        ],
        BlendMode::Lighten,
    );
    // green = 0, 128/255, 0
    assert_rgb(out, 1.0, 128.0 / 255.0, 1.0, 1.0);
}

// ----- difference / exclusion ------------------------------------------

#[test]
fn difference_red_minus_red_is_black() {
    let out = blend(
        &[parse("red").unwrap(), parse("red").unwrap()],
        BlendMode::Difference,
    );
    assert_rgb(out, 0.0, 0.0, 0.0, 1.0);
}

#[test]
fn difference_known_values() {
    let bg = rgb(0.3, 0.6, 0.9, None);
    let fg = rgb(0.7, 0.4, 0.1, None);
    let out = blend(&[bg, fg], BlendMode::Difference);
    // culori output, allowing for fp error in 0.6 - 0.4 = 0.19999...
    assert_rgb(out, 0.39999999999999997, 0.19999999999999996, 0.8, 1.0);
}

#[test]
fn exclusion_red_minus_red_is_black() {
    let out = blend(
        &[parse("red").unwrap(), parse("red").unwrap()],
        BlendMode::Exclusion,
    );
    assert_rgb(out, 0.0, 0.0, 0.0, 1.0);
}

#[test]
fn exclusion_known_values() {
    let bg = rgb(0.3, 0.6, 0.9, None);
    let fg = rgb(0.7, 0.4, 0.1, None);
    let out = blend(&[bg, fg], BlendMode::Exclusion);
    assert_rgb(out, 0.5800000000000001, 0.52, 0.82, 1.0);
}

// ----- overlay ----------------------------------------------------------

#[test]
fn overlay_grey_grey_is_zero() {
    // Both inputs = 0.5; for b<0.5 branch never fires (b==0.5), so falls
    // into 2*b*(1-s)-1 = 2*0.5*0.5 - 1 = -0.5, clipped to 0.
    let bg = rgb(0.5, 0.5, 0.5, None);
    let fg = rgb(0.5, 0.5, 0.5, None);
    let out = blend(&[bg, fg], BlendMode::Overlay);
    assert_rgb(out, 0.0, 0.0, 0.0, 1.0);
}

#[test]
fn overlay_known_values() {
    let bg = rgb(0.3, 0.6, 0.9, None);
    let fg = rgb(0.7, 0.4, 0.1, None);
    let out = blend(&[bg, fg], BlendMode::Overlay);
    // r: b<0.5, s*2*b = 0.7*2*0.3 = 0.42
    // g: b>=0.5, 2*b*(1-s)-1 = 1.2*0.6-1 = -0.28 → clipped 0
    // b: b>=0.5, 2*0.9*0.9 - 1 = 0.62
    assert_rgb(out, 0.42, 0.0, 0.6200000000000001, 1.0);
}

// ----- hard-light -------------------------------------------------------

#[test]
fn hard_light_known_values() {
    let bg = rgb(0.3, 0.6, 0.9, None);
    let fg = rgb(0.7, 0.4, 0.1, None);
    let out = blend(&[bg, fg], BlendMode::HardLight);
    // r: s>=0.5, 2*0.7*(1-0.3)-1 = 0.98-1 = -0.02 → 0
    // g: s<0.5, b*2*s = 0.6*0.8 = 0.48
    // b: s<0.5, 0.9*0.2 = 0.18
    assert_rgb(out, 0.0, 0.48, 0.18000000000000002, 1.0);
}

// ----- soft-light -------------------------------------------------------

#[test]
fn soft_light_known_values() {
    let bg = rgb(0.3, 0.6, 0.9, None);
    let fg = rgb(0.7, 0.4, 0.1, None);
    let out = blend(&[bg, fg], BlendMode::SoftLight);
    assert_rgb(out, 0.3990890230020664, 0.552, 0.8280000000000001, 1.0);
}

#[test]
fn soft_light_b_below_quarter() {
    // Triggers the `b < 0.25` branch in the s>=0.5 arm.
    let bg = rgb(0.1, 0.2, 0.05, None);
    let fg = rgb(0.9, 0.5, 0.7, None);
    let out = blend(&[bg, fg], BlendMode::SoftLight);
    assert_rgb(out, 0.25680000000000003, 0.2, 0.0988, 1.0);
}

// ----- color-dodge / color-burn ----------------------------------------

#[test]
fn color_dodge_b_zero_stays_zero() {
    let bg = rgb(0.0, 0.0, 0.0, None);
    let fg = rgb(0.5, 0.5, 0.5, None);
    let out = blend(&[bg, fg], BlendMode::ColorDodge);
    assert_rgb(out, 0.0, 0.0, 0.0, 1.0);
}

#[test]
fn color_dodge_s_one_clamps_to_one() {
    let bg = rgb(0.5, 0.5, 0.5, None);
    let fg = rgb(1.0, 1.0, 1.0, None);
    let out = blend(&[bg, fg], BlendMode::ColorDodge);
    assert_rgb(out, 1.0, 1.0, 1.0, 1.0);
}

#[test]
fn color_burn_b_one_stays_one() {
    let bg = rgb(1.0, 1.0, 1.0, None);
    let fg = rgb(0.5, 0.5, 0.5, None);
    let out = blend(&[bg, fg], BlendMode::ColorBurn);
    assert_rgb(out, 1.0, 1.0, 1.0, 1.0);
}

#[test]
fn color_burn_s_zero_clamps_to_zero() {
    let bg = rgb(0.5, 0.5, 0.5, None);
    let fg = rgb(0.0, 0.0, 0.0, None);
    let out = blend(&[bg, fg], BlendMode::ColorBurn);
    assert_rgb(out, 0.0, 0.0, 0.0, 1.0);
}

// ----- single-color & alpha edge cases ---------------------------------

#[test]
fn single_color_returns_self_in_rgb() {
    let out = blend(&[parse("red").unwrap()], BlendMode::Multiply);
    assert_rgb(out, 1.0, 0.0, 0.0, 1.0);
}

#[test]
fn single_color_preserves_alpha() {
    let c = rgb(0.2, 0.4, 0.6, Some(0.7));
    let out = blend(&[c], BlendMode::Multiply);
    assert_rgb(out, 0.2, 0.4, 0.6, 0.7);
}

#[test]
fn both_alphas_zero_yields_zero_channels() {
    let bg = rgb(0.5, 0.5, 0.5, Some(0.0));
    let fg = rgb(1.0, 0.0, 0.0, Some(0.0));
    let out = blend(&[bg, fg], BlendMode::Multiply);
    assert_rgb(out, 0.0, 0.0, 0.0, 0.0);
}

#[test]
fn opaque_backdrop_transparent_source_returns_backdrop() {
    let bg = rgb(0.5, 0.5, 0.5, Some(1.0));
    let fg = rgb(1.0, 0.0, 0.0, Some(0.0));
    let out = blend(&[bg, fg], BlendMode::Multiply);
    assert_rgb(out, 0.5, 0.5, 0.5, 1.0);
}

#[test]
#[should_panic(expected = "at least one color")]
fn empty_input_panics() {
    let _ = blend(&[], BlendMode::Multiply);
}

// ----- string-keyed convenience ----------------------------------------

#[test]
fn blend_str_dispatches_to_modes() {
    let out = blend_str(&[parse("red").unwrap(), parse("blue").unwrap()], "multiply")
        .expect("multiply is a known mode");
    assert_rgb(out, 0.0, 0.0, 0.0, 1.0);
}

#[test]
fn blend_str_returns_none_for_unknown() {
    let out = blend_str(&[parse("red").unwrap(), parse("blue").unwrap()], "screened");
    assert!(out.is_none());
}

#[test]
fn blend_str_handles_hyphenated_keywords() {
    let bg = rgb(0.5, 0.5, 0.5, None);
    let fg = rgb(1.0, 1.0, 1.0, None);
    let out = blend_str(&[bg, fg], "color-dodge").unwrap();
    assert_rgb(out, 1.0, 1.0, 1.0, 1.0);
}
