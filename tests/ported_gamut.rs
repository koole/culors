//! Tests for gamut mapping (`in_gamut`, `clamp_gamut`, `clamp_chroma`,
//! `to_gamut`), ported from culori 4.0.2.
//!
//! Each expected output was produced with `node -e "import('culori').then(c =>
//! { console.log(JSON.stringify(c.clampGamut('rgb')(c.parse('oklch(70% 1
//! 30deg)')))); })"` against the version of culori vendored in
//! `node_modules/`.

use culor::spaces::{Lab, Lch, Oklab, Oklch, Rgb};
use culor::{clamp_chroma, clamp_gamut, in_gamut, parse, to_gamut, Color};

const TOL_RGB: f64 = 1e-12;
// `clamp_chroma` uses a binary search whose final precision is the loop
// epsilon (`(range_max - range_min) / 2^13`). The reported chroma agrees
// with culori to 1e-9 because both implementations share the same epsilon.
const TOL_CHROMA: f64 = 1e-10;

fn rgb(r: f64, g: f64, b: f64) -> Color {
    Color::Rgb(Rgb {
        r,
        g,
        b,
        alpha: None,
    })
}

fn oklch(l: f64, c: f64, h: f64) -> Color {
    Color::Oklch(Oklch {
        l,
        c,
        h,
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
fn unwrap_lch(c: Color) -> Lch {
    match c {
        Color::Lch(v) => v,
        other => panic!("expected Lch, got {other:?}"),
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

// ----- clamp_chroma -----

#[test]
fn clamp_chroma_oklch_high_chroma() {
    // culori output:
    // {"mode":"oklch","l":0.7,"c":0.191497802734375,"h":30}
    let c = parse("oklch(70% 1 30deg)").unwrap();
    let out = unwrap_oklch(clamp_chroma(c, "oklch"));
    close(out.l, 0.7, TOL_RGB, "l");
    close(out.c, 0.191497802734375, TOL_CHROMA, "c");
    close(out.h, 30.0, TOL_RGB, "h");
}

#[test]
fn clamp_chroma_oklch_blue_high_chroma() {
    // culori output:
    // {"mode":"oklch","l":0.5,"c":0.0849609375,"h":200}
    let c = parse("oklch(50% 0.4 200deg)").unwrap();
    let out = unwrap_oklch(clamp_chroma(c, "oklch"));
    close(out.l, 0.5, TOL_RGB, "l");
    close(out.c, 0.0849609375, TOL_CHROMA, "c");
    close(out.h, 200.0, TOL_RGB, "h");
}

#[test]
fn clamp_chroma_oklch_already_in_gamut_returns_unchanged() {
    // culori returns the input directly when already displayable.
    let c = parse("oklch(70% 0.05 30deg)").unwrap();
    let out = unwrap_oklch(clamp_chroma(c, "oklch"));
    close(out.l, 0.7, TOL_RGB, "l");
    close(out.c, 0.05, TOL_RGB, "c");
    close(out.h, 30.0, TOL_RGB, "h");
}

#[test]
fn clamp_chroma_lch_high_chroma() {
    // culori output:
    // {"mode":"lch","l":50,"c":87.9150390625,"h":30}
    let c = parse("lch(50% 100 30deg)").unwrap();
    let out = unwrap_lch(clamp_chroma(c, "lch"));
    close(out.l, 50.0, TOL_RGB, "l");
    close(out.c, 87.9150390625, 1e-9, "c");
    close(out.h, 30.0, TOL_RGB, "h");
}

#[test]
fn clamp_chroma_lch_yellow_high_chroma() {
    // culori output:
    // {"mode":"lch","l":80,"c":80.0537109375,"h":90}
    let c = parse("lch(80% 200 90deg)").unwrap();
    let out = unwrap_lch(clamp_chroma(c, "lch"));
    close(out.l, 80.0, TOL_RGB, "l");
    close(out.c, 80.0537109375, 1e-9, "c");
    close(out.h, 90.0, TOL_RGB, "h");
}

// ----- to_gamut -----

#[test]
fn to_gamut_high_chroma_red() {
    // culori output:
    // {"mode":"rgb","r":0.9999999999999992,"g":0.3449982717769464,
    //  "b":0.2644352837465229}
    let c = parse("oklch(70% 1 30deg)").unwrap();
    let out = unwrap_rgb(to_gamut(c, "rgb"));
    close(out.r, 0.9999999999999992, 1e-9, "r");
    close(out.g, 0.3449982717769464, 1e-9, "g");
    close(out.b, 0.2644352837465229, 1e-9, "b");
}

#[test]
fn to_gamut_high_chroma_teal() {
    // culori output:
    // {"mode":"rgb","r":-4.3032244434471065e-15,"g":0.45526244812891287,
    //  "b":0.4805155106881282}
    let c = parse("oklch(50% 0.4 200deg)").unwrap();
    let out = unwrap_rgb(to_gamut(c, "rgb"));
    close(out.r, -4.3032244434471065e-15, 1e-9, "r");
    close(out.g, 0.45526244812891287, 1e-9, "g");
    close(out.b, 0.4805155106881282, 1e-9, "b");
}

#[test]
fn to_gamut_high_chroma_yellow() {
    // culori output:
    // {"mode":"rgb","r":0.9218146142528202,"g":0.7133978445979436,
    //  "b":2.1516122217235533e-15}
    let c = parse("oklch(80% 0.5 90deg)").unwrap();
    let out = unwrap_rgb(to_gamut(c, "rgb"));
    close(out.r, 0.9218146142528202, 1e-9, "r");
    close(out.g, 0.7133978445979436, 1e-9, "g");
    close(out.b, 2.1516122217235533e-15, 1e-9, "b");
}

#[test]
fn to_gamut_lch_high_chroma() {
    // culori output (toGamut('rgb','oklch') of lch(50% 100 30deg)):
    // {"mode":"rgb","r":0.9487331494776389,"g":3.944622406493181e-15,
    //  "b":0.18887421263906104}
    let c = parse("lch(50% 100 30deg)").unwrap();
    let out = unwrap_rgb(to_gamut(c, "rgb"));
    close(out.r, 0.9487331494776389, 1e-9, "r");
    close(out.g, 3.944622406493181e-15, 1e-9, "g");
    close(out.b, 0.18887421263906104, 1e-9, "b");
}

#[test]
fn to_gamut_lab_high_chroma() {
    // culori output: {"mode":"rgb","r":0.905369558050305,
    // "g":6.9927397206015484e-15,"b":0.48259968601310493}
    let c = Color::Lab(Lab {
        l: 50.0,
        a: 80.0,
        b: 0.0,
        alpha: None,
    });
    let out = unwrap_rgb(to_gamut(c, "rgb"));
    close(out.r, 0.905369558050305, 1e-9, "r");
    close(out.g, 6.9927397206015484e-15, 1e-9, "g");
    close(out.b, 0.48259968601310493, 1e-9, "b");
}

#[test]
fn to_gamut_above_white_short_circuits() {
    // l >= 1 -> returns the destination's white verbatim.
    let c = oklch(1.2, 0.3, 30.0);
    let out = unwrap_rgb(to_gamut(c, "rgb"));
    close(out.r, 1.0, TOL_RGB, "r");
    close(out.g, 1.0, TOL_RGB, "g");
    close(out.b, 1.0, TOL_RGB, "b");
}

#[test]
fn to_gamut_below_black_short_circuits() {
    let c = oklch(0.0, 0.0, 0.0);
    let out = unwrap_rgb(to_gamut(c, "rgb"));
    close(out.r, 0.0, TOL_RGB, "r");
    close(out.g, 0.0, TOL_RGB, "g");
    close(out.b, 0.0, TOL_RGB, "b");
}

#[test]
fn to_gamut_in_gamut_round_trips_through_oklch() {
    // culori converts the input to oklch and back, so even an in-gamut
    // input is not bit-identical. Output for #ff0000:
    // {"mode":"rgb","r":0.999906128490129,"g":0.0009349404303559801,
    //  "b":0.0005127046739268403}
    let c = parse("#ff0000").unwrap();
    let out = unwrap_rgb(to_gamut(c, "rgb"));
    close(out.r, 0.999906128490129, 1e-9, "r");
    close(out.g, 0.0009349404303559801, 1e-9, "g");
    close(out.b, 0.0005127046739268403, 1e-9, "b");
}

#[test]
fn to_gamut_alpha_passthrough() {
    // Alpha should survive both the short-circuit white path and the
    // binary-search path.
    let c = Color::Oklch(Oklch {
        l: 1.2,
        c: 0.3,
        h: 30.0,
        alpha: Some(0.5),
    });
    let out = unwrap_rgb(to_gamut(c, "rgb"));
    assert_eq!(out.alpha, Some(0.5));

    let c2 = Color::Oklch(Oklch {
        l: 0.7,
        c: 1.0,
        h: 30.0,
        alpha: Some(0.25),
    });
    let out2 = unwrap_rgb(to_gamut(c2, "rgb"));
    assert_eq!(out2.alpha, Some(0.25));
}
