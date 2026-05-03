//! Tests for `average_number`, `average_angle`, and the mode-aware
//! `average` color reducer.
//!
//! Reference values come from culori 4.0.2 invoked through Node — see
//! the `node -e` snippets in the wave brief. Each numeric expected value
//! was produced by `culori.average([...], '<mode>')` and pasted in.

use culor::spaces::{Hsl, Lab, Lch, Oklab, Oklch, Rgb};
use culor::{average, average_angle, average_number, parse, Color};

const EPS: f64 = 1e-12;

#[track_caller]
fn approx(label: &str, a: f64, b: f64) {
    let diff = (a - b).abs();
    assert!(
        diff <= EPS,
        "{label}: actual={a}, expected={b}, diff={diff}"
    );
}

// ----- average_number ---------------------------------------------------

#[test]
fn average_number_basic_mean() {
    approx("[1,2,3]", average_number(&[1.0, 2.0, 3.0]), 2.0);
}

#[test]
fn average_number_ignores_nan() {
    approx("[1, NaN, 3]", average_number(&[1.0, f64::NAN, 3.0]), 2.0);
}

#[test]
fn average_number_all_nan_returns_nan() {
    let v = average_number(&[f64::NAN, f64::NAN]);
    assert!(v.is_nan(), "expected NaN, got {v}");
}

#[test]
fn average_number_empty_returns_nan() {
    let v = average_number(&[]);
    assert!(v.is_nan(), "expected NaN, got {v}");
}

#[test]
fn average_number_singleton() {
    approx("[5]", average_number(&[5.0]), 5.0);
}

#[test]
fn average_number_negatives_and_fractions() {
    approx(
        "[-1, 0.5, 2]",
        average_number(&[-1.0, 0.5, 2.0]),
        (-1.0 + 0.5 + 2.0) / 3.0,
    );
}

// ----- average_angle ----------------------------------------------------

#[test]
fn average_angle_singleton_zero() {
    approx("[0]", average_angle(&[0.0]), 0.0);
}

#[test]
fn average_angle_wraps_low_high_pair_to_360() {
    let v = average_angle(&[10.0, 350.0]);
    let near_360 = (v - 360.0).abs() < 1e-9;
    let near_0 = v.abs() < 1e-9;
    assert!(near_360 || near_0, "expected ~360 (or ~0), got {v}");
}

#[test]
fn average_angle_wraps_high_low_pair_to_360() {
    let v = average_angle(&[350.0, 10.0]);
    let near_360 = (v - 360.0).abs() < 1e-9;
    let near_0 = v.abs() < 1e-9;
    assert!(near_360 || near_0, "expected ~360 (or ~0), got {v}");
}

#[test]
fn average_angle_quadrant_midpoint() {
    approx("[90, 180]", average_angle(&[90.0, 180.0]), 135.0);
}

#[test]
fn average_angle_orthogonal_pair() {
    approx("[0, 180]", average_angle(&[0.0, 180.0]), 90.0);
}

#[test]
fn average_angle_close_to_180() {
    approx("[170, 190]", average_angle(&[170.0, 190.0]), 180.0);
}

#[test]
fn average_angle_empty_returns_zero() {
    approx("[]", average_angle(&[]), 0.0);
}

#[test]
fn average_angle_all_nan_returns_zero() {
    approx("[NaN]", average_angle(&[f64::NAN]), 0.0);
    approx("[NaN, NaN]", average_angle(&[f64::NAN, f64::NAN]), 0.0);
}

#[test]
fn average_angle_skips_nan() {
    let v = average_angle(&[10.0, f64::NAN, 350.0]);
    let near_360 = (v - 360.0).abs() < 1e-9;
    let near_0 = v.abs() < 1e-9;
    assert!(near_360 || near_0, "expected ~360 (or ~0), got {v}");
}

// ----- average (color) --------------------------------------------------

fn red() -> Color {
    parse("red").expect("parse red")
}

fn blue() -> Color {
    parse("blue").expect("parse blue")
}

fn green() -> Color {
    parse("green").expect("parse green")
}

fn grey() -> Color {
    parse("grey").expect("parse grey")
}

#[test]
fn average_red_blue_rgb() {
    let out = average(&[red(), blue()], "rgb");
    let Color::Rgb(c) = out else {
        panic!("expected Rgb, got {out:?}")
    };
    approx("r", c.r, 0.5);
    approx("g", c.g, 0.0);
    approx("b", c.b, 0.5);
    assert!(
        c.alpha.is_none(),
        "alpha should be missing, got {:?}",
        c.alpha
    );
}

#[test]
fn average_three_red_green_blue_rgb() {
    let out = average(&[red(), green(), blue()], "rgb");
    let Color::Rgb(c) = out else {
        panic!("expected Rgb, got {out:?}")
    };
    approx("r", c.r, 1.0 / 3.0);
    approx("g", c.g, 0.16732026143790849);
    approx("b", c.b, 1.0 / 3.0);
}

#[test]
fn average_red_blue_lab() {
    let out = average(&[red(), blue()], "lab");
    let Color::Lab(c) = out else {
        panic!("expected Lab, got {out:?}")
    };
    approx("l", c.l, 41.92942005020719);
    approx("a", c.a, 74.54616349338983);
    approx("b", c.b, -21.069364863606836);
}

#[test]
fn average_red_blue_lch() {
    let out = average(&[red(), blue()], "lch");
    let Color::Lch(c) = out else {
        panic!("expected Lch, got {out:?}")
    };
    approx("l", c.l, 41.92942005020719);
    approx("c", c.c, 119.01933412159538);
    approx("h", c.h, 351.1109751359331);
}

#[test]
fn average_red_blue_hsl_circular_hue() {
    // hsl(0, 100%, 50%) and hsl(240, 100%, 50%) — circular mean of hues
    // is 300, not 120.
    let out = average(&[red(), blue()], "hsl");
    let Color::Hsl(c) = out else {
        panic!("expected Hsl, got {out:?}")
    };
    approx("h", c.h, 300.0);
    approx("s", c.s, 1.0);
    approx("l", c.l, 0.5);
}

#[test]
fn average_opposite_hues_hsl() {
    let parsed_a = parse("hsl(0 50% 50%)").expect("a");
    let parsed_b = parse("hsl(180 50% 50%)").expect("b");
    let out = average(&[parsed_a, parsed_b], "hsl");
    let Color::Hsl(c) = out else {
        panic!("expected Hsl")
    };
    approx("h", c.h, 90.0);
    approx("s", c.s, 0.5);
    approx("l", c.l, 0.5);
}

#[test]
fn average_red_grey_hsl_keeps_red_hue() {
    // grey converts to HSL with h = NaN; only red's hue contributes.
    // averageAngle of [0] is 0.
    let out = average(&[red(), grey()], "hsl");
    let Color::Hsl(c) = out else {
        panic!("expected Hsl")
    };
    approx("h", c.h, 0.0);
    approx("s", c.s, 0.5);
    approx("l", c.l, 0.5009803921568627);
}

#[test]
fn average_all_grey_hsl_drops_hue() {
    // Both inputs have NaN h. After filtering, the hue list is empty, so
    // culori does not assign h on the result. We mirror that: h stays NaN.
    let g1 = grey();
    let g2 = parse("#888").expect("888");
    let out = average(&[g1, g2], "hsl");
    let Color::Hsl(c) = out else {
        panic!("expected Hsl")
    };
    assert!(c.h.is_nan(), "h should be NaN, got {}", c.h);
    approx("s", c.s, 0.0);
    approx("l", c.l, 0.5176470588235293);
}

#[test]
fn average_red_blue_hsv() {
    let out = average(&[red(), blue()], "hsv");
    let Color::Hsv(c) = out else {
        panic!("expected Hsv")
    };
    approx("h", c.h, 300.0);
    approx("s", c.s, 1.0);
    approx("v", c.v, 1.0);
}

#[test]
fn average_red_blue_hwb() {
    let out = average(&[red(), blue()], "hwb");
    let Color::Hwb(c) = out else {
        panic!("expected Hwb")
    };
    approx("h", c.h, 300.0);
    approx("w", c.w, 0.0);
    approx("b", c.b, 0.0);
}

#[test]
fn average_red_blue_oklab() {
    let out = average(&[red(), blue()], "oklab");
    let Color::Oklab(c) = out else {
        panic!("expected Oklab")
    };
    approx("l", c.l, 0.5399845410479274);
    approx("a", c.a, 0.09620304662773833);
    approx("b", c.b, -0.09284094417349634);
}

#[test]
fn average_red_blue_oklch() {
    let out = average(&[red(), blue()], "oklch");
    let Color::Oklch(c) = out else {
        panic!("expected Oklch")
    };
    approx("l", c.l, 0.5399845410479274);
    approx("c", c.c, 0.2854488462199228);
    approx("h", c.h, 326.6429514479989);
}

#[test]
fn average_alpha_both_present() {
    let with_alpha = Color::Rgb(Rgb {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        alpha: Some(0.5),
    });
    let other = Color::Rgb(Rgb {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        alpha: Some(1.0),
    });
    let out = average(&[with_alpha, other], "rgb");
    let Color::Rgb(c) = out else {
        panic!("expected Rgb")
    };
    approx("r", c.r, 0.5);
    approx("b", c.b, 0.5);
    approx("alpha", c.alpha.expect("alpha set"), 0.75);
}

#[test]
fn average_alpha_one_missing_takes_only_present() {
    // culori treats undefined alpha as missing; it's filtered before the
    // mean. So [0.5, undef] averages to 0.5, not 0.75.
    let with_alpha = Color::Rgb(Rgb {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        alpha: Some(0.5),
    });
    let no_alpha = blue();
    let out = average(&[with_alpha, no_alpha], "rgb");
    let Color::Rgb(c) = out else {
        panic!("expected Rgb")
    };
    approx("alpha", c.alpha.expect("alpha set"), 0.5);
}

#[test]
fn average_single_color_round_trips() {
    let out = average(&[red()], "rgb");
    let Color::Rgb(c) = out else {
        panic!("expected Rgb")
    };
    approx("r", c.r, 1.0);
    approx("g", c.g, 0.0);
    approx("b", c.b, 0.0);
}

#[test]
fn average_lab_struct_input() {
    let a = Color::Lab(Lab {
        l: 50.0,
        a: 10.0,
        b: -10.0,
        alpha: None,
    });
    let b = Color::Lab(Lab {
        l: 70.0,
        a: -10.0,
        b: 10.0,
        alpha: None,
    });
    let out = average(&[a, b], "lab");
    let Color::Lab(c) = out else {
        panic!("expected Lab")
    };
    approx("l", c.l, 60.0);
    approx("a", c.a, 0.0);
    approx("b", c.b, 0.0);
}

#[test]
fn average_lch_hue_uses_circular_mean() {
    let a = Color::Lch(Lch {
        l: 50.0,
        c: 30.0,
        h: 10.0,
        alpha: None,
    });
    let b = Color::Lch(Lch {
        l: 50.0,
        c: 30.0,
        h: 350.0,
        alpha: None,
    });
    let out = average(&[a, b], "lch");
    let Color::Lch(c) = out else {
        panic!("expected Lch")
    };
    let near_360 = (c.h - 360.0).abs() < 1e-9;
    let near_0 = c.h.abs() < 1e-9;
    assert!(near_360 || near_0, "h should be near 0 or 360, got {}", c.h);
}

#[test]
fn average_oklch_hue_circular() {
    let a = Color::Oklch(Oklch {
        l: 0.7,
        c: 0.1,
        h: 10.0,
        alpha: None,
    });
    let b = Color::Oklch(Oklch {
        l: 0.7,
        c: 0.1,
        h: 350.0,
        alpha: None,
    });
    let out = average(&[a, b], "oklch");
    let Color::Oklch(c) = out else {
        panic!("expected Oklch")
    };
    let near_360 = (c.h - 360.0).abs() < 1e-9;
    let near_0 = c.h.abs() < 1e-9;
    assert!(near_360 || near_0, "h should be near 0/360, got {}", c.h);
}

#[test]
fn average_oklab_does_not_use_circular_mean() {
    // a/b channels in oklab are rectangular — plain arithmetic mean.
    let a = Color::Oklab(Oklab {
        l: 0.5,
        a: 0.1,
        b: -0.1,
        alpha: None,
    });
    let b = Color::Oklab(Oklab {
        l: 0.7,
        a: -0.1,
        b: 0.1,
        alpha: None,
    });
    let out = average(&[a, b], "oklab");
    let Color::Oklab(c) = out else {
        panic!("expected Oklab")
    };
    approx("l", c.l, 0.6);
    approx("a", c.a, 0.0);
    approx("b", c.b, 0.0);
}

#[test]
fn average_hsl_struct_with_explicit_nan_hue() {
    let a = Color::Hsl(Hsl {
        h: f64::NAN,
        s: 0.0,
        l: 0.4,
        alpha: None,
    });
    let b = Color::Hsl(Hsl {
        h: 120.0,
        s: 0.5,
        l: 0.6,
        alpha: None,
    });
    let out = average(&[a, b], "hsl");
    let Color::Hsl(c) = out else {
        panic!("expected Hsl")
    };
    approx("h", c.h, 120.0);
    approx("s", c.s, 0.25);
    approx("l", c.l, 0.5);
}

#[test]
#[should_panic(expected = "unknown mode")]
fn average_unknown_mode_panics() {
    let _ = average(&[red()], "nope");
}
