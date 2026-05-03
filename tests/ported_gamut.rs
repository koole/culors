//! Tests for gamut mapping (`in_gamut`, `clamp_gamut`), ported from culori
//! 4.0.2.
//!
//! Each expected output was produced with `node -e "import('culori').then(c =>
//! { console.log(JSON.stringify(c.clampGamut('rgb')(c.parse('oklch(70% 1
//! 30deg)')))); })"` against the version of culori vendored in
//! `node_modules/`.

use culor::spaces::{Oklab, Oklch, Rgb};
use culor::{clamp_gamut, in_gamut, parse, Color};

const TOL_RGB: f64 = 1e-12;

fn rgb(r: f64, g: f64, b: f64) -> Color {
    Color::Rgb(Rgb {
        r,
        g,
        b,
        alpha: None,
    })
}

fn unwrap_rgb(c: Color) -> Rgb {
    match c {
        Color::Rgb(v) => v,
        other => panic!("expected Rgb, got {other:?}"),
    }
}
fn unwrap_oklch(c: Color) -> Oklch {
    match c {
        Color::Oklch(v) => v,
        other => panic!("expected Oklch, got {other:?}"),
    }
}
fn unwrap_oklab(c: Color) -> Oklab {
    match c {
        Color::Oklab(v) => v,
        other => panic!("expected Oklab, got {other:?}"),
    }
}

fn close(actual: f64, expected: f64, tol: f64, label: &str) {
    let diff = (actual - expected).abs();
    assert!(
        diff <= tol,
        "{label}: expected {expected}, got {actual} (diff {diff:.3e})"
    );
}

// ----- in_gamut -----

#[test]
fn in_gamut_rgb_in_range_sample() {
    assert!(in_gamut(&rgb(0.5, 0.5, 0.5), "rgb"));
}

#[test]
fn in_gamut_rgb_above_one_is_false() {
    assert!(!in_gamut(&rgb(1.5, 0.5, 0.5), "rgb"));
}

#[test]
fn in_gamut_rgb_negative_is_false() {
    assert!(!in_gamut(&rgb(-0.1, 0.5, 0.5), "rgb"));
}

#[test]
fn in_gamut_rgb_boundaries() {
    assert!(in_gamut(&rgb(0.0, 0.0, 0.0), "rgb"));
    assert!(in_gamut(&rgb(1.0, 1.0, 1.0), "rgb"));
}

#[test]
fn in_gamut_rgb_in_range_oklch() {
    let c = parse("oklch(70% 0.15 30deg)").unwrap();
    assert!(in_gamut(&c, "rgb"));
}

#[test]
fn in_gamut_rgb_out_of_range_oklch() {
    let c = parse("oklch(70% 1 30deg)").unwrap();
    assert!(!in_gamut(&c, "rgb"));
}

#[test]
fn in_gamut_lab_returns_true_unconditionally() {
    // Lab has no `gamut` field in culori; everything is in gamut.
    let c = rgb(5.0, -2.0, 0.0);
    assert!(in_gamut(&c, "lab"));
}

#[test]
fn in_gamut_oklab_returns_true_unconditionally() {
    let c = parse("oklch(70% 1 30deg)").unwrap();
    assert!(in_gamut(&c, "oklab"));
}

#[test]
fn in_gamut_hsl_proxies_to_rgb() {
    let in_range = parse("oklch(70% 0.15 30deg)").unwrap();
    let out_of_range = parse("oklch(70% 1 30deg)").unwrap();
    assert!(in_gamut(&in_range, "hsl"));
    assert!(!in_gamut(&out_of_range, "hsl"));
}

// ----- clamp_gamut -----

#[test]
fn clamp_gamut_passes_through_in_gamut_color() {
    // White Rgb is already in gamut; output should equal input.
    let c = rgb(1.0, 1.0, 1.0);
    let out = unwrap_rgb(clamp_gamut(c, "rgb"));
    close(out.r, 1.0, TOL_RGB, "r");
    close(out.g, 1.0, TOL_RGB, "g");
    close(out.b, 1.0, TOL_RGB, "b");
}

#[test]
fn clamp_gamut_clips_rgb_above_one() {
    let c = rgb(1.5, 0.5, 0.2);
    let out = unwrap_rgb(clamp_gamut(c, "rgb"));
    close(out.r, 1.0, TOL_RGB, "r");
    close(out.g, 0.5, TOL_RGB, "g");
    close(out.b, 0.2, TOL_RGB, "b");
}

#[test]
fn clamp_gamut_clips_rgb_negative() {
    let c = rgb(-0.3, 0.4, 0.5);
    let out = unwrap_rgb(clamp_gamut(c, "rgb"));
    close(out.r, 0.0, TOL_RGB, "r");
    close(out.g, 0.4, TOL_RGB, "g");
    close(out.b, 0.5, TOL_RGB, "b");
}

#[test]
fn clamp_gamut_returns_in_source_mode_for_oklch() {
    // culori output:
    // {"mode":"oklch","l":0.6279553639214311,"c":0.2576833038053608,
    //  "h":29.233880279627854}
    let c = parse("oklch(70% 1 30deg)").unwrap();
    let out = unwrap_oklch(clamp_gamut(c, "rgb"));
    close(out.l, 0.6279553639214311, 1e-12, "l");
    close(out.c, 0.2576833038053608, 1e-12, "c");
    close(out.h, 29.233880279627854, 1e-10, "h");
}

#[test]
fn clamp_gamut_returns_in_source_mode_for_oklab() {
    // culori output:
    // {"mode":"oklab","l":0.6508732761002483,"a":0.2654229125470914,
    //  "b":-0.014194695246013245}
    let c = Color::Oklab(Oklab {
        l: 0.7,
        a: 0.5,
        b: 0.0,
        alpha: None,
    });
    let out = unwrap_oklab(clamp_gamut(c, "rgb"));
    close(out.l, 0.6508732761002483, 1e-12, "l");
    close(out.a, 0.2654229125470914, 1e-12, "a");
    close(out.b, -0.014194695246013245, 1e-12, "b");
}
