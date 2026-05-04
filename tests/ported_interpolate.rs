//! Tests for color interpolation, ported from culori 4.0.2.
//!
//! Each expected output was produced with `node -e "import('culori').then(c =>
//! { const f = c.interpolate([...], 'mode'); console.log(JSON.stringify(f(t)));
//! })"` against the version of culori vendored in `node_modules/`.

use culors::spaces::{
    Cubehelix, Dlab, Dlch, Hsi, Hsl, Hwb, Itp, Jab, Jch, Lab, Lab65, Lch, Lch65, Lchuv, Luv, Okhsl,
    Okhsv, Oklab, Oklch, Prismatic, ProphotoRgb, Rec2020, Rgb, Xyb, Yiq, A98, P3,
};
use culors::{interpolate, interpolate_with, Color, HueFixup, InterpolateOptions};

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

// ----- v0.4 long-tail modes (culori reference values) ------------------

#[test]
fn p3_red_blue_midpoint() {
    let f = interpolate(&[red(), blue()], "p3");
    let Color::P3(P3 { r, g, b, .. }) = f(0.5) else {
        panic!("expected P3")
    };
    assert_close(r, 0.4587437786625832, "r");
    assert_close(g, 0.1001434038704231, "g");
    assert_close(b, 0.5490743089434619, "b");
}

#[test]
fn rec2020_red_blue_midpoint() {
    let f = interpolate(&[red(), blue()], "rec2020");
    let Color::Rec2020(Rec2020 { r, g, b, .. }) = f(0.5) else {
        panic!("expected Rec2020")
    };
    assert_close(r, 0.4801732411926725, "r");
    assert_close(g, 0.14105305242959412, "g");
    assert_close(b, 0.5102727791253909, "b");
}

#[test]
fn a98_red_blue_midpoint() {
    let f = interpolate(&[red(), blue()], "a98");
    let Color::A98(A98 { r, g, b, .. }) = f(0.5) else {
        panic!("expected A98")
    };
    assert_close(r, 0.42929577333089874, "r");
    assert_close(g, -4.796489868265625e-8, "g");
    assert_close(b, 0.4905343872139412, "b");
}

#[test]
fn prophoto_red_blue_midpoint() {
    let f = interpolate(&[red(), blue()], "prophoto");
    let Color::ProphotoRgb(ProphotoRgb { r, g, b, .. }) = f(0.5) else {
        panic!("expected ProphotoRgb")
    };
    assert_close(r, 0.5192266914172228, "r");
    assert_close(g, 0.2066753874789284, "g");
    assert_close(b, 0.5132068387306806, "b");
}

#[test]
fn cubehelix_red_blue_midpoint() {
    let f = interpolate(&[red(), blue()], "cubehelix");
    let Color::Cubehelix(Cubehelix { h, s, l, .. }) = f(0.5) else {
        panic!("expected Cubehelix")
    };
    assert_close(h, 294.3762167240816, "h");
    assert_close(s, 3.281642256705994, "s");
    assert_close(l, 0.20499949744362608, "l");
}

#[test]
fn dlab_red_blue_midpoint() {
    let f = interpolate(&[red(), blue()], "dlab");
    let Color::Dlab(Dlab { l, a, b, .. }) = f(0.5) else {
        panic!("expected Dlab")
    };
    assert_close(l, 46.65955474529936, "l");
    assert_close(a, 35.7165035799803, "a");
    assert_close(b, -4.932428304208742, "b");
}

#[test]
fn dlch_red_blue_midpoint() {
    let f = interpolate(&[red(), blue()], "dlch");
    let Color::Dlch(Dlch { l, c, h, .. }) = f(0.5) else {
        panic!("expected Dlch")
    };
    assert_close(l, 46.65955474529936, "l");
    assert_close(c, 50.69955531984588, "c");
    assert_close(h, -6.98567924934116, "h");
}

#[test]
fn jab_red_blue_midpoint() {
    let f = interpolate(&[red(), blue()], "jab");
    let Color::Jab(Jab { j, a, b, .. }) = f(0.5) else {
        panic!("expected Jab")
    };
    assert_close(j, 0.11507951159827312, "j");
    assert_close(a, 0.03851988325365878, "a");
    assert_close(b, -0.0369880879669606, "b");
}

#[test]
fn jch_red_blue_midpoint() {
    let f = interpolate(&[red(), blue()], "jch");
    let Color::Jch(Jch { j, c, h, .. }) = f(0.5) else {
        panic!("expected Jch")
    };
    assert_close(j, 0.11507951159827312, "j");
    assert_close(c, 0.17640622812012802, "c");
    assert_close(h, -29.446295498272654, "h");
}

#[test]
fn yiq_red_blue_midpoint() {
    let f = interpolate(&[red(), blue()], "yiq");
    let Color::Yiq(Yiq { y, i, q, .. }) = f(0.5) else {
        panic!("expected Yiq")
    };
    assert_close(y, 0.20668877000000002, "y");
    assert_close(i, 0.13708805000000002, "i");
    assert_close(q, 0.261308555, "q");
}

#[test]
fn hsi_red_blue_midpoint() {
    let f = interpolate(&[red(), blue()], "hsi");
    let Color::Hsi(Hsi { h, s, i, .. }) = f(0.5) else {
        panic!("expected Hsi")
    };
    assert_close(h, -60.0, "h");
    assert_close(s, 1.0, "s");
    assert_close(i, 1.0 / 3.0, "i");
}

#[test]
fn okhsl_red_blue_midpoint() {
    let f = interpolate(&[red(), blue()], "okhsl");
    let Color::Okhsl(Okhsl { h, s, l, .. }) = f(0.5) else {
        panic!("expected Okhsl")
    };
    assert_close(h, -33.35704855200113, "h");
    assert_close(s, 1.0000000004900826, "s");
    assert_close(l, 0.46732499775339253, "l");
}

#[test]
fn okhsv_red_blue_midpoint() {
    let f = interpolate(&[red(), blue()], "okhsv");
    let Color::Okhsv(Okhsv { h, s, v, .. }) = f(0.5) else {
        panic!("expected Okhsv")
    };
    assert_close(h, -33.35704855200113, "h");
    assert_close(s, 0.9997565431956061, "s");
    assert_close(v, 1.0, "v");
}

#[test]
fn itp_red_blue_midpoint() {
    let f = interpolate(&[red(), blue()], "itp");
    let Color::Itp(Itp { i, t, p, .. }) = f(0.5) else {
        panic!("expected Itp")
    };
    assert_close(i, 0.39193195619466953, "i");
    assert_close(t, 0.07681489789925641, "t");
    assert_close(p, 0.0586789248460263, "p");
}

#[test]
fn xyb_red_blue_midpoint() {
    let f = interpolate(&[red(), blue()], "xyb");
    let Color::Xyb(Xyb { x, y, b, .. }) = f(0.5) else {
        panic!("expected Xyb")
    };
    assert_close(x, 0.014050041580638661, "x");
    assert_close(y, 0.3831581991945666, "y");
    assert_close(b, 0.1857412196980236, "b");
}

#[test]
fn luv_red_blue_midpoint() {
    let f = interpolate(&[red(), blue()], "luv");
    let Color::Luv(Luv { l, u, v, .. }) = f(0.5) else {
        panic!("expected Luv")
    };
    assert_close(l, 41.92942005020719, "l");
    assert_close(u, 81.74575754669908, "u");
    assert_close(v, -48.00662844150128, "v");
}

#[test]
fn lchuv_red_blue_midpoint() {
    let f = interpolate(&[red(), blue()], "lchuv");
    let Color::Lchuv(Lchuv { l, c, h, .. }) = f(0.5) else {
        panic!("expected Lchuv")
    };
    assert_close(l, 41.92942005020719, "l");
    assert_close(c, 149.73090975405557, "c");
    assert_close(h, -43.486376862254694, "h");
}

#[test]
fn p3_red_white_midpoint() {
    let white = Color::Rgb(Rgb {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        alpha: None,
    });
    let f = interpolate(&[red(), white], "p3");
    let Color::P3(P3 { r, g, b, .. }) = f(0.5) else {
        panic!("expected P3")
    };
    assert_close(r, 0.9587437786625828, "r");
    assert_close(g, 0.6001434038704232, "g");
    assert_close(b, 0.5692802956055569, "b");
}

#[test]
fn cubehelix_red_blue_quarter() {
    let f = interpolate(&[red(), blue()], "cubehelix");
    let Color::Cubehelix(Cubehelix { h, s, l, .. }) = f(0.25) else {
        panic!("expected Cubehelix")
    };
    assert_close(h, 323.09323924746184, "h");
    assert_close(s, 2.615269951039131, "s");
    assert_close(l, 0.25249947137943424, "l");
}

#[test]
fn lchuv_red_green_midpoint() {
    let green = Color::Rgb(Rgb {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        alpha: None,
    });
    let f = interpolate(&[red(), green], "lchuv");
    let Color::Lchuv(Lchuv { l, c, h, .. }) = f(0.5) else {
        panic!("expected Lchuv")
    };
    assert_close(l, 71.05453963906085, "l");
    assert_close(c, 149.3488227277694, "c");
    assert_close(h, 71.33273562157414, "h");
}

#[test]
fn lab65_red_blue_midpoint_matches_culori() {
    // c.interpolate(['red','blue'],'lab65')(0.5)
    let f = interpolate(&[red(), blue()], "lab65");
    let Color::Lab65(Lab65 { l, a, b, .. }) = f(0.5) else {
        panic!("expected Lab65")
    };
    assert_close(l, 42.76899424970476, "l");
    assert_close(a, 79.64269191525403, "a");
    assert_close(b, -20.326101014010263, "b");
}

#[test]
fn lab65_red_blue_t_zero_matches_culori() {
    // c.interpolate(['red','blue'],'lab65')(0) == c.lab65('red')
    let f = interpolate(&[red(), blue()], "lab65");
    let Color::Lab65(Lab65 { l, a, b, .. }) = f(0.0) else {
        panic!("expected Lab65")
    };
    assert_close(l, 53.237115595429344, "l");
    assert_close(a, 80.09011352310385, "a");
    assert_close(b, 67.20326351172214, "b");
}

#[test]
fn lch65_red_blue_midpoint_matches_culori() {
    // c.interpolate(['red','blue'],'lch65')(0.5) — shorter-fixup wraps
    // blue's hue (306.288…) into negative territory so the lerp lands
    // at -6.855…, matching culori's output exactly.
    let f = interpolate(&[red(), blue()], "lch65");
    let Color::Lch65(Lch65 { l, c, h, .. }) = f(0.5) else {
        panic!("expected Lch65")
    };
    assert_close(l, 42.76899424970476, "l");
    assert_close(c, 119.17921393918918, "c");
    assert_close(h, -6.855665794154326, "h");
}

#[test]
fn lch65_red_blue_t_zero_matches_culori() {
    // c.interpolate(['red','blue'],'lch65')(0) == c.lch65('red')
    let f = interpolate(&[red(), blue()], "lch65");
    let Color::Lch65(Lch65 { l, c, h, .. }) = f(0.0) else {
        panic!("expected Lch65")
    };
    assert_close(l, 53.237115595429344, "l");
    assert_close(c, 104.55001152926587, "c");
    assert_close(h, 39.99986515439813, "h");
}

#[test]
fn prismatic_red_blue_midpoint_handcomputed() {
    // Hand-computed: red→Prismatic = (1, 1, 0, 0); blue→Prismatic =
    // (1, 0, 0, 1); per-channel lerp at t=0.5 gives (1, 0.5, 0, 0.5).
    // culori 4.0.2 has no `prismatic` mode, so the reference is
    // derived from culors's own conversion (Hauke 2009).
    let f = interpolate(&[red(), blue()], "prismatic");
    let Color::Prismatic(Prismatic { l, r, g, b, .. }) = f(0.5) else {
        panic!("expected Prismatic")
    };
    assert_close(l, 1.0, "l");
    assert_close(r, 0.5, "r");
    assert_close(g, 0.0, "g");
    assert_close(b, 0.5, "b");
}

#[test]
fn prismatic_red_blue_t_zero_returns_red() {
    // At t=0 the closure returns the first stop (Color::Prismatic of red).
    let f = interpolate(&[red(), blue()], "prismatic");
    let Color::Prismatic(Prismatic { l, r, g, b, .. }) = f(0.0) else {
        panic!("expected Prismatic")
    };
    assert_close(l, 1.0, "l");
    assert_close(r, 1.0, "r");
    assert_close(g, 0.0, "g");
    assert_close(b, 0.0, "b");
}

// Issue culori#140 — an easing function that returns values outside
// `[0, 1]` (e.g. `back-in-out`, which overshoots both ends) must not
// produce NaN channels. The eased `t` selects a piecewise segment that
// extrapolates past the endpoints linearly, exactly as the linear
// interpolator already handles for raw `t > 1` / `t < 0`.
#[test]
fn easing_returning_outside_unit_range_does_not_nan() {
    // `back-in-out` cubic from <https://github.com/mattdesl/eases/blob/master/back-in-out.js>:
    fn back_in_out(mut t: f64) -> f64 {
        let s = 1.701_58 * 1.525;
        t *= 2.0;
        if t < 1.0 {
            0.5 * (t * t * ((s + 1.0) * t - s))
        } else {
            t -= 2.0;
            0.5 * (t * t * ((s + 1.0) * t + s) + 2.0)
        }
    }

    let stops = [
        // #ff0000
        Color::Rgb(Rgb {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            alpha: None,
        }),
        // #cc8833
        Color::Rgb(Rgb {
            r: 0xcc as f64 / 255.0,
            g: 0x88 as f64 / 255.0,
            b: 0x33 as f64 / 255.0,
            alpha: None,
        }),
        // #3344cc
        Color::Rgb(Rgb {
            r: 0x33 as f64 / 255.0,
            g: 0x44 as f64 / 255.0,
            b: 0xcc as f64 / 255.0,
            alpha: None,
        }),
    ];
    let opts = InterpolateOptions::new().easing(back_in_out);
    let f = interpolate_with(&stops, "rgb", opts);
    // Sweep across the range — back-in-out overshoots near both ends.
    for &t in &[0.05, 0.1, 0.5, 0.9, 0.95] {
        let out = f(t);
        let Color::Rgb(c) = out else {
            panic!("expected Rgb");
        };
        assert!(!c.r.is_nan(), "r is NaN at t={t}");
        assert!(!c.g.is_nan(), "g is NaN at t={t}");
        assert!(!c.b.is_nan(), "b is NaN at t={t}");
    }
}
