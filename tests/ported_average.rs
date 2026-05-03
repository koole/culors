//! Tests for `average_number`, `average_angle`, and the mode-aware
//! `average` color reducer.
//!
//! Reference values come from culori 4.0.2 invoked through Node — see
//! the `node -e` snippets in the wave brief. Each numeric expected value
//! was produced by `culori.average([...], '<mode>')` and pasted in.

use culors::spaces::{
    Cubehelix, Hsi, Hsl, Itp, Jab, Jch, Lab, Lab65, Lch, Lch65, Lchuv, Oklab, Oklch, Prismatic,
    Rgb, Xyb, Yiq, P3,
};
use culors::{average, average_angle, average_number, parse, Color};

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

// ----- v0.4 long-tail modes (culori reference values) ------------------

const TOL: f64 = 1e-9;

#[track_caller]
fn approx_tol(label: &str, a: f64, b: f64) {
    let diff = (a - b).abs();
    assert!(
        diff <= TOL,
        "{label}: actual={a}, expected={b}, diff={diff}"
    );
}

#[test]
fn average_p3_red_blue_matches_culori() {
    let out = average(&[red(), blue()], "p3");
    let Color::P3(P3 { r, g, b, .. }) = out else {
        panic!("expected P3")
    };
    approx_tol("r", r, 0.4587437786625832);
    approx_tol("g", g, 0.1001434038704231);
    approx_tol("b", b, 0.5490743089434619);
}

#[test]
fn average_cubehelix_red_blue_matches_culori() {
    let out = average(&[red(), blue()], "cubehelix");
    let Color::Cubehelix(Cubehelix { h, s, l, .. }) = out else {
        panic!("expected Cubehelix")
    };
    // Hue is a circular mean over -90 and 678.7524... → wraps near 294.376°.
    approx_tol("h", h, 294.3762167240816);
    approx_tol("s", s, 3.281642256705994);
    approx_tol("l", l, 0.20499949744362608);
}

#[test]
fn average_hsi_red_blue_matches_culori() {
    let out = average(&[red(), blue()], "hsi");
    let Color::Hsi(Hsi { h, s, i, .. }) = out else {
        panic!("expected Hsi")
    };
    // Red h=0, blue h=240 — circular mean = 300°.
    approx_tol("h", h, 300.0);
    approx_tol("s", s, 1.0);
    approx_tol("i", i, 1.0 / 3.0);
}

#[test]
fn average_lchuv_red_green_blue_matches_culori() {
    let out = average(&[red(), green(), blue()], "lchuv");
    let Color::Lchuv(Lchuv { l, c, h, .. }) = out else {
        panic!("expected Lchuv")
    };
    approx_tol("l", l, 43.378849709298215);
    approx_tol("c", c, 121.20646152188696);
    approx_tol("h", h, 326.16018341786383);
}

#[test]
fn average_jab_red_blue_matches_culori() {
    let out = average(&[red(), blue()], "jab");
    let Color::Jab(Jab { j, a, b, .. }) = out else {
        panic!("expected Jab")
    };
    approx_tol("j", j, 0.11507951159827312);
    approx_tol("a", a, 0.03851988325365877);
    approx_tol("b", b, -0.03698808796696061);
}

#[test]
fn average_jch_red_blue_matches_culori() {
    let out = average(&[red(), blue()], "jch");
    let Color::Jch(Jch { j, c, h, .. }) = out else {
        panic!("expected Jch")
    };
    approx_tol("j", j, 0.11507951159827312);
    approx_tol("c", c, 0.17640622812012802);
    approx_tol("h", h, 330.55370450172734);
}

#[test]
fn average_itp_red_blue_matches_culori() {
    let out = average(&[red(), blue()], "itp");
    let Color::Itp(Itp { i, t, p, .. }) = out else {
        panic!("expected Itp")
    };
    approx_tol("i", i, 0.39193195619466953);
    approx_tol("t", t, 0.07681489789925641);
    approx_tol("p", p, 0.0586789248460263);
}

#[test]
fn average_xyb_red_blue_matches_culori() {
    let out = average(&[red(), blue()], "xyb");
    let Color::Xyb(Xyb { x, y, b, .. }) = out else {
        panic!("expected Xyb")
    };
    approx_tol("x", x, 0.014050041580638661);
    approx_tol("y", y, 0.3831581991945666);
    approx_tol("b", b, 0.1857412196980236);
}

#[test]
fn average_yiq_red_blue_matches_culori() {
    let out = average(&[red(), blue()], "yiq");
    let Color::Yiq(Yiq { y, i, q, .. }) = out else {
        panic!("expected Yiq")
    };
    approx_tol("y", y, 0.20668877000000002);
    approx_tol("i", i, 0.13708805);
    approx_tol("q", q, 0.261308555);
}

#[test]
fn average_lab65_red_blue_matches_culori() {
    // c.average(['red','blue'],'lab65')
    let out = average(&[red(), blue()], "lab65");
    let Color::Lab65(Lab65 { l, a, b, .. }) = out else {
        panic!("expected Lab65")
    };
    approx_tol("l", l, 42.76899424970476);
    approx_tol("a", a, 79.64269191525403);
    approx_tol("b", b, -20.326101014010256);
}

#[test]
fn average_lab65_singleton_red_round_trips() {
    // c.lab65('red') — averaging a single color is the conversion.
    let out = average(&[red()], "lab65");
    let Color::Lab65(Lab65 { l, a, b, .. }) = out else {
        panic!("expected Lab65")
    };
    approx_tol("l", l, 53.237115595429344);
    approx_tol("a", a, 80.09011352310385);
    approx_tol("b", b, 67.20326351172214);
}

#[test]
fn average_lch65_red_blue_matches_culori_circular() {
    // c.average(['red','blue'],'lch65') — hue uses the circular mean,
    // which lands at 353.144… rather than the lerp-style midpoint
    // produced by `interpolate(...,'lch65')`.
    let out = average(&[red(), blue()], "lch65");
    let Color::Lch65(Lch65 { l, c, h, .. }) = out else {
        panic!("expected Lch65")
    };
    approx_tol("l", l, 42.76899424970476);
    approx_tol("c", c, 119.17921393918918);
    approx_tol("h", h, 353.14433420584567);
}

#[test]
fn average_lch65_singleton_red_round_trips() {
    // c.lch65('red')
    let out = average(&[red()], "lch65");
    let Color::Lch65(Lch65 { l, c, h, .. }) = out else {
        panic!("expected Lch65")
    };
    approx_tol("l", l, 53.237115595429344);
    approx_tol("c", c, 104.55001152926587);
    approx_tol("h", h, 39.99986515439813);
}

#[test]
fn average_prismatic_red_blue_midpoint() {
    // Hand-computed: red→Prismatic = (1, 1, 0, 0); blue→Prismatic =
    // (1, 0, 0, 1). Per-channel arithmetic mean = (1, 0.5, 0, 0.5).
    let out = average(&[red(), blue()], "prismatic");
    let Color::Prismatic(Prismatic { l, r, g, b, .. }) = out else {
        panic!("expected Prismatic")
    };
    approx_tol("l", l, 1.0);
    approx_tol("r", r, 0.5);
    approx_tol("g", g, 0.0);
    approx_tol("b", b, 0.5);
}

#[test]
fn average_prismatic_three_primaries_uniform_chromaticity() {
    // Pure red, pure green (1.0, not CSS "green" = 128/255), pure blue:
    // each maps to Prismatic (1, e_i) with a unit chromaticity vector.
    // The arithmetic mean is (1, 1/3, 1/3, 1/3).
    let pure_green = Color::Rgb(Rgb {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        alpha: None,
    });
    let out = average(&[red(), pure_green, blue()], "prismatic");
    let Color::Prismatic(Prismatic { l, r, g, b, .. }) = out else {
        panic!("expected Prismatic")
    };
    approx_tol("l", l, 1.0);
    approx_tol("r", r, 1.0 / 3.0);
    approx_tol("g", g, 1.0 / 3.0);
    approx_tol("b", b, 1.0 / 3.0);
}
