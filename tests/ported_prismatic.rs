//! Tests for the Prismatic color space.
//!
//! culori 4.0.2 has no Prismatic mode, so reference values come from the
//! Hauke 2009 definition computed by hand:
//!
//! ```text
//! l = max(R, G, B);  s = R + G + B
//! if s > 0: (r, g, b) = (R/s, G/s, B/s) else (0, 0, 0)
//! ```
//!
//! and the inverse `(R, G, B) = (l*r, l*g, l*b) / max(r, g, b)`.

use culor::convert;
use culor::format_css;
use culor::spaces::{Prismatic, Rgb};
use culor::Color;

const TOL: f64 = 1e-12;

#[track_caller]
fn approx(label: &str, actual: f64, expected: f64) {
    let diff = (actual - expected).abs();
    assert!(
        diff <= TOL,
        "{label}: actual={actual}, expected={expected}, diff={diff}"
    );
}

fn rgb(r: f64, g: f64, b: f64) -> Rgb {
    Rgb {
        r,
        g,
        b,
        alpha: None,
    }
}

#[test]
fn white_to_prismatic() {
    // White: l = 1, normalized = (1/3, 1/3, 1/3).
    let p: Prismatic = rgb(1.0, 1.0, 1.0).into();
    approx("l", p.l, 1.0);
    approx("r", p.r, 1.0 / 3.0);
    approx("g", p.g, 1.0 / 3.0);
    approx("b", p.b, 1.0 / 3.0);
}

#[test]
fn black_to_prismatic_pins_chroma_to_zero() {
    // Black has s = 0; spec: (r, g, b) = (0, 0, 0).
    let p: Prismatic = rgb(0.0, 0.0, 0.0).into();
    approx("l", p.l, 0.0);
    approx("r", p.r, 0.0);
    approx("g", p.g, 0.0);
    approx("b", p.b, 0.0);
}

#[test]
fn red_to_prismatic_full_intensity_red_corner() {
    // Pure red lands at (1, 1, 0, 0).
    let p: Prismatic = rgb(1.0, 0.0, 0.0).into();
    approx("l", p.l, 1.0);
    approx("r", p.r, 1.0);
    approx("g", p.g, 0.0);
    approx("b", p.b, 0.0);
}

#[test]
fn yellow_to_prismatic_split_between_r_and_g() {
    // (1, 1, 0): l = 1, s = 2, normalized = (0.5, 0.5, 0).
    let p: Prismatic = rgb(1.0, 1.0, 0.0).into();
    approx("l", p.l, 1.0);
    approx("r", p.r, 0.5);
    approx("g", p.g, 0.5);
    approx("b", p.b, 0.0);
}

#[test]
fn arbitrary_color_to_prismatic() {
    // (0.6, 0.3, 0.9): l = 0.9, s = 1.8 → (1/3, 1/6, 1/2).
    let p: Prismatic = rgb(0.6, 0.3, 0.9).into();
    approx("l", p.l, 0.9);
    approx("r", p.r, 1.0 / 3.0);
    approx("g", p.g, 1.0 / 6.0);
    approx("b", p.b, 1.0 / 2.0);
}

#[test]
fn round_trip_arbitrary_color_through_prismatic() {
    let original = rgb(0.6, 0.3, 0.9);
    let p: Prismatic = original.into();
    let back: Rgb = p.into();
    approx("r", back.r, original.r);
    approx("g", back.g, original.g);
    approx("b", back.b, original.b);
}

#[test]
fn round_trip_grey_preserves_luminance() {
    // Mid-grey: l = 0.5, normalized = (1/3, 1/3, 1/3); inverse scales by
    // 0.5 / (1/3) = 1.5 to land back on (0.5, 0.5, 0.5).
    let original = rgb(0.5, 0.5, 0.5);
    let back: Rgb = Prismatic::from(original).into();
    approx("r", back.r, 0.5);
    approx("g", back.g, 0.5);
    approx("b", back.b, 0.5);
}

#[test]
fn alpha_passes_through_both_directions() {
    let src = Rgb {
        r: 0.4,
        g: 0.6,
        b: 0.2,
        alpha: Some(0.5),
    };
    let p: Prismatic = src.into();
    assert_eq!(p.alpha, Some(0.5));
    let back: Rgb = p.into();
    assert_eq!(back.alpha, Some(0.5));
}

#[test]
fn convert_via_xyz_hub_matches_direct_path_within_tolerance() {
    // The hub round-trip (Rgb → Xyz65 → Prismatic → Xyz65 → Rgb) should
    // match the direct conversion within numerical precision.
    let src = rgb(0.7, 0.3, 0.5);
    let p: Prismatic = convert(src);
    let back: Rgb = convert(p);
    let direct: Prismatic = src.into();
    approx("hub vs direct l", p.l, direct.l);
    approx("hub vs direct r", p.r, direct.r);
    approx("hub vs direct g", p.g, direct.g);
    approx("hub vs direct b", p.b, direct.b);
    // Hub round-trip degrades a bit through XYZ; loosen the tolerance.
    let hub_tol = 1e-9;
    assert!((back.r - 0.7).abs() < hub_tol, "back.r={}", back.r);
    assert!((back.g - 0.3).abs() < hub_tol, "back.g={}", back.g);
    assert!((back.b - 0.5).abs() < hub_tol, "back.b={}", back.b);
}

#[test]
fn format_css_uses_dashed_prismatic_id() {
    // CSS Color 4 reserves `--<custom>` for non-standard profiles.
    let c = Color::Prismatic(Prismatic {
        l: 0.5,
        r: 0.25,
        g: 0.25,
        b: 0.5,
        alpha: None,
    });
    let css = format_css(&c);
    assert!(
        css.starts_with("color(--prismatic"),
        "expected `color(--prismatic ...)`, got `{css}`"
    );
    assert!(css.contains(" 0.5 "), "missing l channel: `{css}`");
}
