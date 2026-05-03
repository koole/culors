//! Ported tests for the CSS Color Module 4 formatter.
//!
//! Each expected string was produced by running culori 4.0.2's
//! `formatCss()` on the equivalent JS color object (see Phase F.1 report
//! for the `node -e` script). Channel-mode mapping mirrors the parser
//! suite: culori's modes `rgb`, `hsl`, `hwb`, `hsv`, `lab`, `lch`,
//! `oklab`, `oklch`, `lrgb`, `xyz65`, `xyz50` map onto the matching
//! `Color` variants.

use culors::format_css;
use culors::parse;
use culors::spaces::{
    Hsl, Hsv, Hwb, Lab, Lch, LinearRgb, Oklab, Oklch, ProphotoRgb, Rec2020, Rgb, Xyz50, Xyz65, A98,
    P3,
};
use culors::Color;

// ---------- sRGB ----------

#[test]
fn rgb_red() {
    let c = Color::Rgb(Rgb {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        alpha: None,
    });
    assert_eq!(format_css(&c), "color(srgb 1 0 0)");
}

#[test]
fn rgb_red_alpha_half() {
    let c = Color::Rgb(Rgb {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        alpha: Some(0.5),
    });
    assert_eq!(format_css(&c), "color(srgb 1 0 0 / 0.5)");
}

#[test]
fn rgb_red_alpha_one_omitted() {
    let c = Color::Rgb(Rgb {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        alpha: Some(1.0),
    });
    assert_eq!(format_css(&c), "color(srgb 1 0 0)");
}

#[test]
fn rgb_gray() {
    let c = Color::Rgb(Rgb {
        r: 0.5,
        g: 0.5,
        b: 0.5,
        alpha: None,
    });
    assert_eq!(format_css(&c), "color(srgb 0.5 0.5 0.5)");
}

#[test]
fn rgb_out_of_gamut_preserved() {
    let c = Color::Rgb(Rgb {
        r: -0.5,
        g: 1.5,
        b: 0.0,
        alpha: None,
    });
    assert_eq!(format_css(&c), "color(srgb -0.5 1.5 0)");
}

#[test]
fn rgb_alpha_zero_included() {
    let c = Color::Rgb(Rgb {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        alpha: Some(0.0),
    });
    assert_eq!(format_css(&c), "color(srgb 1 1 1 / 0)");
}

#[test]
fn rgb_one_third() {
    let c = Color::Rgb(Rgb {
        r: 1.0 / 3.0,
        g: 0.0,
        b: 0.0,
        alpha: None,
    });
    assert_eq!(format_css(&c), "color(srgb 0.3333333333333333 0 0)");
}

// ---------- linear sRGB ----------

#[test]
fn lrgb_basic() {
    let c = Color::LinearRgb(LinearRgb {
        r: 0.5,
        g: 0.5,
        b: 0.5,
        alpha: None,
    });
    assert_eq!(format_css(&c), "color(srgb-linear 0.5 0.5 0.5)");
}

#[test]
fn lrgb_alpha() {
    let c = Color::LinearRgb(LinearRgb {
        r: 0.5,
        g: 0.5,
        b: 0.5,
        alpha: Some(0.7),
    });
    assert_eq!(format_css(&c), "color(srgb-linear 0.5 0.5 0.5 / 0.7)");
}

// ---------- HSV ----------

#[test]
fn hsv_basic() {
    let c = Color::Hsv(Hsv {
        h: 120.0,
        s: 0.5,
        v: 0.5,
        alpha: None,
    });
    assert_eq!(format_css(&c), "color(--hsv 120 0.5 0.5)");
}

#[test]
fn hsv_alpha() {
    let c = Color::Hsv(Hsv {
        h: 120.0,
        s: 0.5,
        v: 0.5,
        alpha: Some(0.25),
    });
    assert_eq!(format_css(&c), "color(--hsv 120 0.5 0.5 / 0.25)");
}

// ---------- HSL ----------

#[test]
fn hsl_blue() {
    let c = Color::Hsl(Hsl {
        h: 240.0,
        s: 1.0,
        l: 0.5,
        alpha: None,
    });
    assert_eq!(format_css(&c), "hsl(240 100% 50%)");
}

#[test]
fn hsl_alpha_half() {
    let c = Color::Hsl(Hsl {
        h: 120.0,
        s: 0.5,
        l: 0.5,
        alpha: Some(0.5),
    });
    assert_eq!(format_css(&c), "hsl(120 50% 50% / 0.5)");
}

#[test]
fn hsl_alpha_one_omitted() {
    let c = Color::Hsl(Hsl {
        h: 120.0,
        s: 0.5,
        l: 0.5,
        alpha: Some(1.0),
    });
    assert_eq!(format_css(&c), "hsl(120 50% 50%)");
}

#[test]
fn hsl_nan_hue_emits_none() {
    let c = Color::Hsl(Hsl {
        h: f64::NAN,
        s: 0.5,
        l: 0.5,
        alpha: None,
    });
    assert_eq!(format_css(&c), "hsl(none 50% 50%)");
}

// ---------- HWB ----------

#[test]
fn hwb_basic() {
    let c = Color::Hwb(Hwb {
        h: 120.0,
        w: 0.3,
        b: 0.4,
        alpha: None,
    });
    assert_eq!(format_css(&c), "hwb(120 30% 40%)");
}

#[test]
fn hwb_alpha() {
    let c = Color::Hwb(Hwb {
        h: 60.0,
        w: 0.1,
        b: 0.2,
        alpha: Some(0.5),
    });
    assert_eq!(format_css(&c), "hwb(60 10% 20% / 0.5)");
}

// ---------- Lab ----------

#[test]
fn lab_basic() {
    let c = Color::Lab(Lab {
        l: 50.0,
        a: 40.0,
        b: -30.0,
        alpha: None,
    });
    assert_eq!(format_css(&c), "lab(50 40 -30)");
}

#[test]
fn lab_alpha_half() {
    let c = Color::Lab(Lab {
        l: 50.0,
        a: 40.0,
        b: -30.0,
        alpha: Some(0.5),
    });
    assert_eq!(format_css(&c), "lab(50 40 -30 / 0.5)");
}

#[test]
fn lab_alpha_one_omitted() {
    let c = Color::Lab(Lab {
        l: 50.0,
        a: 40.0,
        b: -30.0,
        alpha: Some(1.0),
    });
    assert_eq!(format_css(&c), "lab(50 40 -30)");
}

// ---------- Lch ----------

#[test]
fn lch_basic() {
    let c = Color::Lch(Lch {
        l: 50.0,
        c: 30.0,
        h: 120.0,
        alpha: None,
    });
    assert_eq!(format_css(&c), "lch(50 30 120)");
}

#[test]
fn lch_zero_chroma_nan_hue() {
    let c = Color::Lch(Lch {
        l: 50.0,
        c: 0.0,
        h: f64::NAN,
        alpha: None,
    });
    assert_eq!(format_css(&c), "lch(50 0 none)");
}

// ---------- Oklab ----------

#[test]
fn oklab_basic() {
    let c = Color::Oklab(Oklab {
        l: 0.5,
        a: 0.1,
        b: -0.1,
        alpha: None,
    });
    assert_eq!(format_css(&c), "oklab(0.5 0.1 -0.1)");
}

#[test]
fn oklab_alpha() {
    let c = Color::Oklab(Oklab {
        l: 0.6,
        a: 0.05,
        b: 0.05,
        alpha: Some(0.4),
    });
    assert_eq!(format_css(&c), "oklab(0.6 0.05 0.05 / 0.4)");
}

// ---------- Oklch ----------

#[test]
fn oklch_basic() {
    let c = Color::Oklch(Oklch {
        l: 0.5,
        c: 0.1,
        h: 120.0,
        alpha: None,
    });
    assert_eq!(format_css(&c), "oklch(0.5 0.1 120)");
}

#[test]
fn oklch_zero_chroma_nan_hue() {
    let c = Color::Oklch(Oklch {
        l: 0.5,
        c: 0.0,
        h: f64::NAN,
        alpha: None,
    });
    assert_eq!(format_css(&c), "oklch(0.5 0 none)");
}

// ---------- XYZ ----------

#[test]
fn xyz65_basic() {
    let c = Color::Xyz65(Xyz65 {
        x: 0.5,
        y: 0.5,
        z: 0.5,
        alpha: None,
    });
    assert_eq!(format_css(&c), "color(xyz-d65 0.5 0.5 0.5)");
}

#[test]
fn xyz65_alpha() {
    let c = Color::Xyz65(Xyz65 {
        x: 0.4,
        y: 0.5,
        z: 0.6,
        alpha: Some(0.5),
    });
    assert_eq!(format_css(&c), "color(xyz-d65 0.4 0.5 0.6 / 0.5)");
}

#[test]
fn xyz50_basic() {
    let c = Color::Xyz50(Xyz50 {
        x: 0.5,
        y: 0.5,
        z: 0.5,
        alpha: None,
    });
    assert_eq!(format_css(&c), "color(xyz-d50 0.5 0.5 0.5)");
}

#[test]
fn xyz50_alpha() {
    let c = Color::Xyz50(Xyz50 {
        x: 0.4,
        y: 0.5,
        z: 0.6,
        alpha: Some(0.5),
    });
    assert_eq!(format_css(&c), "color(xyz-d50 0.4 0.5 0.6 / 0.5)");
}

// ---------- Round-trip stability ----------
//
// A canonical CSS string is one the formatter itself produces. For every
// such string, parse → format must reproduce the input verbatim. The
// inputs below were all confirmed stable through culori's own pipeline.

#[test]
fn round_trip_canonical_strings() {
    let canonicals = [
        "color(srgb 1 0 0)",
        "color(srgb 0.5 0.5 0.5)",
        "color(srgb 1 0 0 / 0.5)",
        "color(srgb-linear 0.5 0.5 0.5)",
        "color(xyz-d65 0.5 0.5 0.5)",
        "color(xyz-d50 0.5 0.5 0.5)",
        "hsl(240 100% 50%)",
        "hsl(120 50% 50% / 0.5)",
        "hwb(120 30% 40%)",
        "lab(50 40 -30)",
        "lab(50 40 -30 / 0.5)",
        "lch(50 30 120)",
        "oklab(0.5 0.1 -0.1)",
        "oklch(0.5 0.1 120)",
    ];
    for s in canonicals {
        let parsed = parse(s).unwrap_or_else(|| panic!("failed to parse {s}"));
        let formatted = format_css(&parsed);
        assert_eq!(formatted, s, "round-trip failed for {s}");
    }
}

// ---------- NaN / non-finite hardening ----------
//
// `format_css` must never panic. NaN channels render as `none`; alpha
// outside `[finite, < 1)` is omitted.

#[test]
fn nan_channels_render_as_none() {
    let c = Color::Rgb(Rgb {
        r: f64::NAN,
        g: 0.0,
        b: 0.0,
        alpha: None,
    });
    assert_eq!(format_css(&c), "color(srgb none 0 0)");
}

#[test]
fn nan_alpha_omitted() {
    // culori's `c.alpha < 1` is `false` for NaN, so the suffix is dropped.
    let c = Color::Rgb(Rgb {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        alpha: Some(f64::NAN),
    });
    assert_eq!(format_css(&c), "color(srgb 1 0 0)");
}

#[test]
fn infinite_alpha_omitted() {
    let c = Color::Rgb(Rgb {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        alpha: Some(f64::INFINITY),
    });
    assert_eq!(format_css(&c), "color(srgb 1 0 0)");
}

#[test]
fn does_not_panic_on_extremes() {
    // Pathological values across every space — only the "no panic" axis
    // is asserted, since these inputs aren't meaningful colors.
    let _ = format_css(&Color::Rgb(Rgb {
        r: f64::INFINITY,
        g: f64::NEG_INFINITY,
        b: f64::NAN,
        alpha: Some(f64::NAN),
    }));
    let _ = format_css(&Color::Hsl(Hsl {
        h: f64::INFINITY,
        s: f64::NAN,
        l: -1.0,
        alpha: Some(-0.5),
    }));
    let _ = format_css(&Color::Lab(Lab {
        l: f64::NAN,
        a: f64::NAN,
        b: f64::NAN,
        alpha: None,
    }));
}

#[test]
fn format_p3_roundtrip() {
    let c = Color::P3(P3 {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        alpha: None,
    });
    assert_eq!(format_css(&c), "color(display-p3 1 0 0)");
    let parsed = parse("color(display-p3 1 0 0)").unwrap();
    assert_eq!(format_css(&parsed), "color(display-p3 1 0 0)");
}

#[test]
fn format_rec2020_with_alpha() {
    let c = Color::Rec2020(Rec2020 {
        r: 0.25,
        g: 0.5,
        b: 0.75,
        alpha: Some(0.4),
    });
    assert_eq!(format_css(&c), "color(rec2020 0.25 0.5 0.75 / 0.4)");
}

#[test]
fn format_a98_roundtrip() {
    let c = Color::A98(A98 {
        r: 0.5,
        g: 0.5,
        b: 0.5,
        alpha: None,
    });
    assert_eq!(format_css(&c), "color(a98-rgb 0.5 0.5 0.5)");
    let parsed = parse("color(a98-rgb 0.5 0.5 0.5)").unwrap();
    assert_eq!(format_css(&parsed), "color(a98-rgb 0.5 0.5 0.5)");
}

#[test]
fn format_prophoto_roundtrip() {
    let c = Color::ProphotoRgb(ProphotoRgb {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        alpha: None,
    });
    assert_eq!(format_css(&c), "color(prophoto-rgb 1 1 1)");
    let parsed = parse("color(prophoto-rgb 1 1 1)").unwrap();
    assert_eq!(format_css(&parsed), "color(prophoto-rgb 1 1 1)");
}
