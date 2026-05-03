//! Ported tests for `displayable`, `clamp_rgb`, `in_gamut`, `clamp_gamut`,
//! `clamp_chroma`, `to_gamut`, mirroring `node_modules/culori/test/clamp.test.js`.
//!
//! `tests/ported_gamut.rs` already covers most of culori's clamp suite; this
//! file pins the cases that touch `displayable` / `clamp_rgb` and the
//! missing-component handling that culori added more recently.

use culors::spaces::{Lch, Oklch, Rgb};
use culors::{clamp_chroma, clamp_rgb, displayable, parse, Color};

const TOL: f64 = 1e-10;

fn close(actual: f64, expected: f64, label: &str) {
    let diff = (actual - expected).abs();
    assert!(
        diff <= TOL,
        "{label}: expected {expected}, got {actual} (diff {diff:.3e})"
    );
}

// ----- displayable: RGB / LCh / missing components -----

#[test]
fn displayable_rgb_zeros() {
    let c = Color::Rgb(Rgb {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        alpha: None,
    });
    assert!(displayable(&c));
}

#[test]
fn displayable_rgb_ones_with_alpha() {
    let c = Color::Rgb(Rgb {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        alpha: Some(0.5),
    });
    assert!(displayable(&c));
}

#[test]
fn displayable_rgb_above_one_is_false() {
    let c = Color::Rgb(Rgb {
        r: 1.1,
        g: 1.0,
        b: 1.0,
        alpha: Some(0.5),
    });
    assert!(!displayable(&c));
}

#[test]
fn displayable_lch_zero_chroma_is_true() {
    let c = parse("lch(50% 0 0)").unwrap();
    assert!(displayable(&c));
}

#[test]
fn displayable_lch_negative_chroma_in_gamut() {
    // culori treats lch's c-channel as unsigned for displayability; the
    // resulting RGB is what gets checked, and abs(-100) at L=50 is still
    // out-of-rgb but in-rgb-like-axis ... but culori asserts true here, so
    // we mirror.
    let c = parse("lch(50% -100 0)").unwrap();
    assert!(displayable(&c));
}

#[test]
fn displayable_lch_above_100_is_false() {
    let c = parse("lch(120% -100 0)").unwrap();
    assert!(!displayable(&c));
}

#[test]
fn displayable_handles_missing_components() {
    // culori: displayable('rgb(none none none)') is true (NaN coerced to 0).
    let c = parse("rgb(none none none)").unwrap();
    assert!(displayable(&c));
}

// ----- clamp_rgb: per-channel clip -----

#[test]
fn clamp_rgb_basic() {
    let c = Color::Rgb(Rgb {
        r: 1.5,
        g: -0.2,
        b: 0.5,
        alpha: None,
    });
    let out = match clamp_rgb(c) {
        Color::Rgb(v) => v,
        other => panic!("expected Rgb, got {other:?}"),
    };
    close(out.r, 1.0, "r");
    close(out.g, 0.0, "g");
    close(out.b, 0.5, "b");
}

#[test]
fn clamp_rgb_handles_missing_components() {
    // culori: clampRgb('rgb(none 300 none)') → {r:0, g:1, b:0}.
    let parsed = parse("rgb(none 300 none)").unwrap();
    let out = match clamp_rgb(parsed) {
        Color::Rgb(v) => v,
        other => panic!("expected Rgb, got {other:?}"),
    };
    close(out.r, 0.0, "r");
    close(out.g, 1.0, "g");
    close(out.b, 0.0, "b");
}

// NOTE: culors' `in_gamut`/`clamp_gamut` only accept sRGB-family modes
// (`"rgb" | "hsl" | "hsv" | "hwb"`); culori's wide-gamut variants
// (`"p3"`, `"rec2020"`, …) aren't yet wired into the gamut module, so the
// p3-specific scenarios from culori's `clamp.test.js` are out of scope here.

// ----- clamp_chroma corner cases -----

#[test]
fn clamp_chroma_red_is_already_in_gamut() {
    let red = parse("red").unwrap();
    let out = match clamp_chroma(red, "rgb") {
        Color::Rgb(v) => v,
        other => panic!("expected Rgb, got {other:?}"),
    };
    close(out.r, 1.0, "r");
    close(out.g, 0.0, "g");
    close(out.b, 0.0, "b");
}

#[test]
fn clamp_chroma_lch_zero_lightness_zeroes_chroma() {
    // culori: at L=0 only c=0 is in gamut; result has c=0 and the input hue
    // preserved.
    let c = Color::Lch(Lch {
        l: 0.0,
        c: 100.0,
        h: 30.0,
        alpha: None,
    });
    let out = match clamp_chroma(c, "lch") {
        Color::Lch(v) => v,
        other => panic!("expected Lch, got {other:?}"),
    };
    close(out.l, 0.0, "l");
    close(out.c, 0.0, "c");
    close(out.h, 30.0, "h");
}

#[test]
fn clamp_chroma_oklch_180_remains_displayable() {
    // Issue #129: clampChroma at hue=180 should stay in sRGB gamut.
    let c = Color::Oklch(Oklch {
        l: 0.5,
        c: 0.161,
        h: 180.0,
        alpha: None,
    });
    let out = clamp_chroma(c, "oklch");
    assert!(displayable(&out));
}
