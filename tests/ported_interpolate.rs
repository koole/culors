//! Tests for color interpolation, ported from culori 4.0.2.
//!
//! Each expected output was produced with `node -e "import('culori').then(c =>
//! { const f = c.interpolate([...], 'mode'); console.log(JSON.stringify(f(t)));
//! })"` against the version of culori vendored in `node_modules/`.

use culor::spaces::{Hsl, Hwb, Lab, Lch, Oklab, Oklch, Rgb};
use culor::{interpolate, interpolate_with, Color, HueFixup, InterpolateOptions};

const TOL: f64 = 1e-10;

fn red() -> Color {
    Color::Rgb(Rgb {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        alpha: None,
    })
}

fn green_named() -> Color {
    // CSS named "green" is rgb(0, 128, 0) = 0/0.50196.../0.
    Color::Rgb(Rgb {
        r: 0.0,
        g: 128.0 / 255.0,
        b: 0.0,
        alpha: None,
    })
}

fn blue() -> Color {
    Color::Rgb(Rgb {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        alpha: None,
    })
}

fn assert_close(actual: f64, expected: f64, label: &str) {
    if expected.is_nan() {
        assert!(actual.is_nan(), "{label}: expected NaN, got {actual}");
        return;
    }
    let diff = (actual - expected).abs();
    assert!(
        diff <= TOL,
        "{label}: expected {expected}, got {actual} (diff {diff:.3e})"
    );
}

fn unwrap_rgb(c: Color) -> Rgb {
    match c {
        Color::Rgb(r) => r,
        other => panic!("expected Rgb, got {other:?}"),
    }
}
fn unwrap_lab(c: Color) -> Lab {
    match c {
        Color::Lab(v) => v,
        other => panic!("expected Lab, got {other:?}"),
    }
}
fn unwrap_oklab(c: Color) -> Oklab {
    match c {
        Color::Oklab(v) => v,
        other => panic!("expected Oklab, got {other:?}"),
    }
}
fn unwrap_oklch(c: Color) -> Oklch {
    match c {
        Color::Oklch(v) => v,
        other => panic!("expected Oklch, got {other:?}"),
    }
}
fn unwrap_lch(c: Color) -> Lch {
    match c {
        Color::Lch(v) => v,
        other => panic!("expected Lch, got {other:?}"),
    }
}
fn unwrap_hsl(c: Color) -> Hsl {
    match c {
        Color::Hsl(v) => v,
        other => panic!("expected Hsl, got {other:?}"),
    }
}
fn unwrap_hwb(c: Color) -> Hwb {
    match c {
        Color::Hwb(v) => v,
        other => panic!("expected Hwb, got {other:?}"),
    }
}

#[test]
fn rgb_two_stop_midpoint() {
    let f = interpolate(&[red(), blue()], "rgb");
    let out = unwrap_rgb(f(0.5));
    assert_close(out.r, 0.5, "r");
    assert_close(out.g, 0.0, "g");
    assert_close(out.b, 0.5, "b");
}

#[test]
fn rgb_two_stop_quarter() {
    let f = interpolate(&[red(), blue()], "rgb");
    let out = unwrap_rgb(f(0.25));
    assert_close(out.r, 0.75, "r");
    assert_close(out.g, 0.0, "g");
    assert_close(out.b, 0.25, "b");
}

#[test]
fn rgb_two_stop_three_quarter() {
    let f = interpolate(&[red(), blue()], "rgb");
    let out = unwrap_rgb(f(0.75));
    assert_close(out.r, 0.25, "r");
    assert_close(out.g, 0.0, "g");
    assert_close(out.b, 0.75, "b");
}

#[test]
fn rgb_two_stop_t_zero_returns_first() {
    let f = interpolate(&[red(), blue()], "rgb");
    let out = unwrap_rgb(f(0.0));
    assert_close(out.r, 1.0, "r");
    assert_close(out.g, 0.0, "g");
    assert_close(out.b, 0.0, "b");
}

#[test]
fn rgb_two_stop_t_one_returns_last() {
    let f = interpolate(&[red(), blue()], "rgb");
    let out = unwrap_rgb(f(1.0));
    assert_close(out.r, 0.0, "r");
    assert_close(out.g, 0.0, "g");
    assert_close(out.b, 1.0, "b");
}

#[test]
fn rgb_clamps_negative_t_to_zero() {
    let f = interpolate(&[red(), blue()], "rgb");
    let out = unwrap_rgb(f(-0.5));
    assert_close(out.r, 1.0, "r");
    assert_close(out.b, 0.0, "b");
}

#[test]
fn rgb_clamps_t_above_one() {
    let f = interpolate(&[red(), blue()], "rgb");
    let out = unwrap_rgb(f(1.5));
    assert_close(out.b, 1.0, "b");
    assert_close(out.r, 0.0, "r");
}

#[test]
fn lab_two_stop_midpoint() {
    let f = interpolate(&[red(), blue()], "lab");
    let out = unwrap_lab(f(0.5));
    assert_close(out.l, 41.92942005020719, "l");
    assert_close(out.a, 74.54616349338983, "a");
    assert_close(out.b, -21.069364863606836, "b");
}

#[test]
fn lab_two_stop_quarter() {
    let f = interpolate(&[red(), blue()], "lab");
    let out = unwrap_lab(f(0.25));
    assert_close(out.l, 48.10998149858844, "l");
    assert_close(out.a, 77.675541914007, "a");
    assert_close(out.b, 24.41081169767797, "b");
}

#[test]
fn lab_two_stop_t_zero_is_red_in_lab() {
    let f = interpolate(&[red(), blue()], "lab");
    let out = unwrap_lab(f(0.0));
    assert_close(out.l, 54.29054294696968, "l");
    assert_close(out.a, 80.80492033462417, "a");
    assert_close(out.b, 69.89098825896278, "b");
}

#[test]
fn oklab_two_stop_midpoint() {
    let f = interpolate(&[red(), blue()], "oklab");
    let out = unwrap_oklab(f(0.5));
    assert_close(out.l, 0.5399845410479274, "l");
    assert_close(out.a, 0.09620304662773835, "a");
    assert_close(out.b, -0.09284094417349634, "b");
}

#[test]
fn oklab_two_stop_quarter() {
    let f = interpolate(&[red(), blue()], "oklab");
    let out = unwrap_oklab(f(0.25));
    assert_close(out.l, 0.5839699524846793, "l");
    assert_close(out.a, 0.1605330575270064, "a");
    assert_close(out.b, 0.01650266657854431, "b");
}

#[test]
fn lch_two_stop_midpoint() {
    let f = interpolate(&[red(), blue()], "lch");
    let out = unwrap_lch(f(0.5));
    assert_close(out.l, 41.92942005020719, "l");
    assert_close(out.c, 119.01933412159538, "c");
    assert_close(out.h, -8.889024864066954, "h");
}

#[test]
fn oklch_two_stop_midpoint() {
    let f = interpolate(&[red(), blue()], "oklch");
    let out = unwrap_oklch(f(0.5));
    assert_close(out.l, 0.5399845410479274, "l");
    assert_close(out.c, 0.2854488462199228, "c");
    assert_close(out.h, -33.35704855200113, "h");
}

#[test]
fn hsl_shorter_default_midpoint() {
    let f = interpolate(&[red(), blue()], "hsl");
    let out = unwrap_hsl(f(0.5));
    assert_close(out.h, -60.0, "h");
    assert_close(out.s, 1.0, "s");
    assert_close(out.l, 0.5, "l");
}

#[test]
fn hsl_shorter_quarter() {
    let f = interpolate(&[red(), blue()], "hsl");
    let out = unwrap_hsl(f(0.25));
    assert_close(out.h, -30.0, "h");
}

#[test]
fn hsl_shorter_three_quarter() {
    let f = interpolate(&[red(), blue()], "hsl");
    let out = unwrap_hsl(f(0.75));
    assert_close(out.h, -90.0, "h");
}

#[test]
fn hsl_longer_midpoint() {
    let opts = InterpolateOptions::new().hue_fixup(HueFixup::Longer);
    let f = interpolate_with(&[red(), blue()], "hsl", opts);
    let out = unwrap_hsl(f(0.5));
    assert_close(out.h, 120.0, "h");
}

#[test]
fn hsl_longer_quarter() {
    let opts = InterpolateOptions::new().hue_fixup(HueFixup::Longer);
    let f = interpolate_with(&[red(), blue()], "hsl", opts);
    let out = unwrap_hsl(f(0.25));
    assert_close(out.h, 60.0, "h");
}

#[test]
fn hsl_increasing_midpoint() {
    let opts = InterpolateOptions::new().hue_fixup(HueFixup::Increasing);
    let f = interpolate_with(&[red(), blue()], "hsl", opts);
    let out = unwrap_hsl(f(0.5));
    assert_close(out.h, 120.0, "h");
}

#[test]
fn hsl_decreasing_midpoint() {
    let opts = InterpolateOptions::new().hue_fixup(HueFixup::Decreasing);
    let f = interpolate_with(&[red(), blue()], "hsl", opts);
    let out = unwrap_hsl(f(0.5));
    assert_close(out.h, -60.0, "h");
}

#[test]
fn hwb_shorter_midpoint() {
    let f = interpolate(&[red(), blue()], "hwb");
    let out = unwrap_hwb(f(0.5));
    assert_close(out.h, -60.0, "h");
    assert_close(out.w, 0.0, "w");
    assert_close(out.b, 0.0, "b");
}

#[test]
fn three_stop_rgb_midpoint_is_middle() {
    let f = interpolate(&[red(), green_named(), blue()], "rgb");
    let out = unwrap_rgb(f(0.5));
    assert_close(out.r, 0.0, "r");
    assert_close(out.g, 128.0 / 255.0, "g");
    assert_close(out.b, 0.0, "b");
}

#[test]
fn three_stop_rgb_quarter() {
    let f = interpolate(&[red(), green_named(), blue()], "rgb");
    let out = unwrap_rgb(f(0.25));
    assert_close(out.r, 0.5, "r");
    assert_close(out.g, 0.5 * 128.0 / 255.0, "g");
    assert_close(out.b, 0.0, "b");
}

#[test]
fn three_stop_rgb_three_quarter() {
    let f = interpolate(&[red(), green_named(), blue()], "rgb");
    let out = unwrap_rgb(f(0.75));
    assert_close(out.r, 0.0, "r");
    assert_close(out.g, 0.5 * 128.0 / 255.0, "g");
    assert_close(out.b, 0.5, "b");
}

#[test]
fn powerless_hue_propagates_to_grey_endpoint() {
    // Grey: HSL with NaN h, s=0, l=0.5. Red: h=0, s=1, l=0.5.
    let grey = Color::Hsl(Hsl {
        h: f64::NAN,
        s: 0.0,
        l: 0.5,
        alpha: None,
    });
    let red_hsl = Color::Hsl(Hsl {
        h: 0.0,
        s: 1.0,
        l: 0.5,
        alpha: None,
    });
    let f = interpolate(&[grey, red_hsl], "hsl");
    let out = unwrap_hsl(f(0.5));
    // [a,a] rule: grey's NaN h becomes [red.h, red.h] = [0, 0].
    assert_close(out.h, 0.0, "h");
    assert_close(out.s, 0.5, "s");
}

#[test]
fn powerless_hue_at_t_zero_stays_nan() {
    let grey = Color::Hsl(Hsl {
        h: f64::NAN,
        s: 0.0,
        l: 0.5,
        alpha: None,
    });
    let red_hsl = Color::Hsl(Hsl {
        h: 0.0,
        s: 1.0,
        l: 0.5,
        alpha: None,
    });
    let f = interpolate(&[grey, red_hsl], "hsl");
    let out = unwrap_hsl(f(0.0));
    // Boundary short-circuit: at t=0, return first stop's literal channels.
    assert!(out.h.is_nan(), "expected NaN h, got {}", out.h);
    assert_close(out.s, 0.0, "s");
}

#[test]
fn alpha_interpolates_linearly() {
    let a = Color::Rgb(Rgb {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        alpha: Some(1.0),
    });
    let b = Color::Rgb(Rgb {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        alpha: Some(0.0),
    });
    let f = interpolate(&[a, b], "rgb");
    let out = unwrap_rgb(f(0.5));
    assert_eq!(out.alpha, Some(0.5));
    let q = unwrap_rgb(f(0.25));
    assert_eq!(q.alpha, Some(0.75));
}

#[test]
fn alpha_missing_stays_missing_when_none_defined() {
    let f = interpolate(&[red(), blue()], "rgb");
    let out = unwrap_rgb(f(0.5));
    assert_eq!(out.alpha, None);
}

#[test]
fn alpha_missing_filled_to_one_when_other_endpoint_defined() {
    let a = Color::Rgb(Rgb {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        alpha: None,
    });
    let b = Color::Rgb(Rgb {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        alpha: Some(0.5),
    });
    let f = interpolate(&[a, b], "rgb");
    // Boundary at t=0: returns first stop literally → alpha None.
    assert_eq!(unwrap_rgb(f(0.0)).alpha, None);
    // Mid: filled-in 1.0 lerps with 0.5 → 0.75.
    let mid = unwrap_rgb(f(0.5));
    assert_eq!(mid.alpha, Some(0.75));
}

#[test]
fn global_easing_quadratic() {
    // Quadratic easing: t -> t^2. At t=0.5, eased = 0.25, so the
    // interpolation in rgb yields the t=0.25 linear color.
    let opts = InterpolateOptions::new().easing(|t| t * t);
    let f = interpolate_with(&[red(), blue()], "rgb", opts);
    let out = unwrap_rgb(f(0.5));
    assert_close(out.r, 0.75, "r");
    assert_close(out.b, 0.25, "b");
}

#[test]
fn per_channel_easing_only_affects_that_channel() {
    // Ease only the L channel of Lab; a/b stay linear.
    let opts = InterpolateOptions::new().channel_easing("l", |t| t * t);
    let f = interpolate_with(&[red(), blue()], "lab", opts);
    let out = unwrap_lab(f(0.5));
    // a/b at t=0.5 should match the linear case.
    assert_close(out.a, 74.54616349338983, "a (linear)");
    assert_close(out.b, -21.069364863606836, "b (linear)");
    // L at eased t=0.25 should match the linear-quarter L value.
    assert_close(out.l, 48.10998149858844, "l (eased to quarter)");
}
