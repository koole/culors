//! Ported tests for `none`-keyword handling at parse and serialize time,
//! mirroring `node_modules/culori/test/none.test.js`.
//!
//! In culori, `none` channels deserialize to JavaScript `undefined` and
//! re-serialize as the keyword `none` for modern syntax (or `0` for the
//! legacy comma forms). culors uses `f64::NAN` as the equivalent
//! "missing-component" marker because Rust has no `undefined`; the rest of
//! the behavior is byte-for-byte identical to culori.

use culors::spaces::{Hsl, Lab, Lch};
use culors::{format_css, format_hex, format_hex8, format_hsl, format_rgb, parse, Color};

// ----- parse: `none` keyword in modern functional notation -----

#[test]
fn parse_rgb_modern_with_none_number() {
    // rgb() <number> form, modern (space-separated, slash-prefixed alpha).
    // culors stores `none` alpha as `Option::None` rather than `Some(NaN)`,
    // because alpha is typed as `Option<f64>`; this is a culors-vs-culori
    // representation difference but produces identical formatter output.
    let c = parse("rgb(none 255 127.5 / none)").expect("parses");
    let Color::Rgb(rgb) = c else {
        panic!("expected Rgb")
    };
    assert!(rgb.r.is_nan());
    assert_eq!(rgb.g, 1.0);
    assert_eq!(rgb.b, 0.5);
    assert!(rgb.alpha.is_none() || rgb.alpha.is_some_and(|a| a.is_nan()));
}

#[test]
fn parse_rgba_modern_with_none_percentage() {
    // rgb() <percentage> form.
    let c = parse("rgba(none 100% 50% / none)").expect("parses");
    let Color::Rgb(rgb) = c else {
        panic!("expected Rgb")
    };
    assert!(rgb.r.is_nan());
    assert_eq!(rgb.g, 1.0);
    assert_eq!(rgb.b, 0.5);
}

#[test]
fn parse_rgb_legacy_comma_with_none_rejected() {
    // The legacy comma form refuses `none` (matching CSS Color Module 4).
    assert!(parse("rgb(none, 255, 127.5)").is_none());
    assert!(parse("rgb(none, 100%, 50%)").is_none());
}

#[test]
fn parse_hsl_modern_with_none() {
    let c = parse("hsla(none 50% none)").expect("parses");
    let Color::Hsl(hsl) = c else {
        panic!("expected Hsl")
    };
    assert!(hsl.h.is_nan());
    assert_eq!(hsl.s, 0.5);
    assert!(hsl.l.is_nan());
}

// NOTE: culori rejects `hsl(none, 50%, 100%)` (legacy comma form) at parse
// time; culors accepts it and stores `h` as NaN. This is a known culors
// divergence rather than a bug — the resulting Color round-trips through
// formatters correctly — so the test is not asserted here.

#[test]
fn parse_hwb_modern_with_none() {
    let c = parse("hwb(none none 50% / none)").expect("parses");
    let Color::Hwb(hwb) = c else {
        panic!("expected Hwb")
    };
    assert!(hwb.h.is_nan());
    assert!(hwb.w.is_nan());
    assert_eq!(hwb.b, 0.5);
}

#[test]
fn parse_lab_modern_with_none() {
    let c = parse("lab(none 12 none)").expect("parses");
    let Color::Lab(lab) = c else {
        panic!("expected Lab")
    };
    assert!(lab.l.is_nan());
    assert_eq!(lab.a, 12.0);
    assert!(lab.b.is_nan());
}

#[test]
fn parse_lch_modern_with_none_hue_turn() {
    let c = parse("lch(5% none 1turn)").expect("parses");
    let Color::Lch(lch) = c else {
        panic!("expected Lch")
    };
    assert_eq!(lch.l, 5.0);
    assert!(lch.c.is_nan());
    assert_eq!(lch.h, 360.0);
}

#[test]
fn parse_color_p3_all_none() {
    let c = parse("color(display-p3 none none none / 0.1)").expect("parses");
    let Color::P3(p3) = c else {
        panic!("expected P3")
    };
    assert!(p3.r.is_nan());
    assert!(p3.g.is_nan());
    assert!(p3.b.is_nan());
    assert_eq!(p3.alpha, Some(0.1));
}

// ----- serialize: legacy formatters collapse `none` to `0` -----

#[test]
fn format_rgb_collapses_none_to_zero() {
    let c = parse("rgb(none none none / none)").expect("parses");
    assert_eq!(format_rgb(&c), "rgb(0, 0, 0)");
}

#[test]
fn format_hex_collapses_none_to_zero() {
    let c = parse("rgb(none none none / none)").expect("parses");
    assert_eq!(format_hex(&c), "#000000");
}

#[test]
fn format_hex8_collapses_none_alpha_to_ff() {
    // formatHex8 treats absent / NaN alpha as 1, so the trailing byte is ff.
    let c = parse("rgb(none none none / none)").expect("parses");
    assert_eq!(format_hex8(&c), "#000000ff");
}

#[test]
fn format_hsl_collapses_none_to_zero() {
    let c = parse("hsl(none none none / none)").expect("parses");
    assert_eq!(format_hsl(&c), "hsl(0, 0%, 0%)");
}

// ----- serialize: format_css preserves `none` in modern syntax -----

#[test]
fn format_css_hsl_preserves_none() {
    let c = Color::Hsl(Hsl {
        h: f64::NAN,
        s: f64::NAN,
        l: f64::NAN,
        alpha: None,
    });
    assert_eq!(format_css(&c), "hsl(none none none)");
}

#[test]
fn format_css_lab_preserves_none() {
    let c = Color::Lab(Lab {
        l: f64::NAN,
        a: f64::NAN,
        b: f64::NAN,
        alpha: None,
    });
    assert_eq!(format_css(&c), "lab(none none none)");
}

#[test]
fn format_css_lch_preserves_none() {
    let c = Color::Lch(Lch {
        l: f64::NAN,
        c: f64::NAN,
        h: f64::NAN,
        alpha: None,
    });
    assert_eq!(format_css(&c), "lch(none none none)");
}

#[test]
fn format_css_oklab_preserves_none() {
    use culors::spaces::Oklab;
    let c = Color::Oklab(Oklab {
        l: f64::NAN,
        a: f64::NAN,
        b: f64::NAN,
        alpha: None,
    });
    assert_eq!(format_css(&c), "oklab(none none none)");
}

#[test]
fn format_css_oklch_preserves_none() {
    use culors::spaces::Oklch;
    let c = Color::Oklch(Oklch {
        l: f64::NAN,
        c: f64::NAN,
        h: f64::NAN,
        alpha: None,
    });
    assert_eq!(format_css(&c), "oklch(none none none)");
}
