//! Wide-gamut profile coverage for `in_gamut` and `clamp_gamut`. Mirrors
//! the equivalent culori 4.0.2 paths through `clampGamut('p3' | 'rec2020'
//! | 'a98' | 'prophoto')`. Reference values produced from culori via
//! `node -e`.

use culors::spaces::{ProphotoRgb, Rec2020, Rgb, A98, P3};
use culors::{clamp_gamut, in_gamut, Color};

const TOL: f64 = 1e-10;

fn close(actual: f64, expected: f64, label: &str) {
    let diff = (actual - expected).abs();
    assert!(
        diff <= TOL,
        "{label}: expected {expected}, got {actual} (diff {diff:.3e})"
    );
}

fn rec2020(r: f64, g: f64, b: f64) -> Color {
    Color::Rec2020(Rec2020 {
        r,
        g,
        b,
        alpha: None,
    })
}

fn prophoto(r: f64, g: f64, b: f64) -> Color {
    Color::ProphotoRgb(ProphotoRgb {
        r,
        g,
        b,
        alpha: None,
    })
}

fn a98(r: f64, g: f64, b: f64) -> Color {
    Color::A98(A98 {
        r,
        g,
        b,
        alpha: None,
    })
}

fn p3(r: f64, g: f64, b: f64) -> Color {
    Color::P3(P3 {
        r,
        g,
        b,
        alpha: None,
    })
}

fn rgb(r: f64, g: f64, b: f64) -> Color {
    Color::Rgb(Rgb {
        r,
        g,
        b,
        alpha: None,
    })
}

// ----- p3 -----

#[test]
fn in_gamut_p3_accepts_p3_red() {
    assert!(in_gamut(&p3(1.0, 0.0, 0.0), "p3"));
}

#[test]
fn in_gamut_p3_rejects_rec2020_red() {
    // rec2020 red sits outside the P3 gamut.
    assert!(!in_gamut(&rec2020(1.0, 0.0, 0.0), "p3"));
}

#[test]
fn clamp_gamut_p3_clips_out_of_range_p3() {
    let out = match clamp_gamut(p3(1.2, -0.1, 0.5), "p3") {
        Color::P3(v) => v,
        other => panic!("expected P3, got {other:?}"),
    };
    close(out.r, 1.0, "p3 r");
    close(out.g, 0.0, "p3 g");
    close(out.b, 0.5, "p3 b");
}

#[test]
fn clamp_gamut_p3_from_rec2020() {
    let out = match clamp_gamut(rec2020(1.0, 0.0, 0.0), "p3") {
        Color::Rec2020(v) => v,
        other => panic!("expected Rec2020, got {other:?}"),
    };
    // culori's clampGamut('p3') returns the clipped color in the input mode.
    close(out.r, 0.8688106968252556, "rec2020-from-p3 r");
    close(out.g, 0.17512273983180493, "rec2020-from-p3 g");
    close(out.b, 0.007043372678308776, "rec2020-from-p3 b");
}

// ----- rec2020 -----

#[test]
fn in_gamut_rec2020_accepts_a98_red() {
    // a98 red maps inside the rec2020 gamut.
    assert!(in_gamut(&a98(1.0, 0.0, 0.0), "rec2020"));
}

#[test]
fn in_gamut_rec2020_rejects_prophoto_red() {
    assert!(!in_gamut(&prophoto(1.0, 0.0, 0.0), "rec2020"));
}

#[test]
fn clamp_gamut_rec2020_clips_prophoto_red() {
    let out = match clamp_gamut(prophoto(1.0, 0.0, 0.0), "rec2020") {
        Color::ProphotoRgb(v) => v,
        other => panic!("expected ProphotoRgb, got {other:?}"),
    };
    close(out.r, 0.9051741570592674, "prophoto-rec2020 r");
    close(out.g, 0.19785381118019055, "prophoto-rec2020 g");
    close(out.b, 0.03974532018725179, "prophoto-rec2020 b");
}

#[test]
fn clamp_gamut_rec2020_in_gamut_passes_through() {
    let inp = rec2020(0.5, 0.5, 0.5);
    let out = match clamp_gamut(inp, "rec2020") {
        Color::Rec2020(v) => v,
        other => panic!("expected Rec2020, got {other:?}"),
    };
    close(out.r, 0.5, "rec2020 r");
    close(out.g, 0.5, "rec2020 g");
    close(out.b, 0.5, "rec2020 b");
}

// ----- a98 -----

#[test]
fn in_gamut_a98_rejects_rgb_white_due_to_float_precision() {
    // sRGB → A98 round-trip leaves one channel at 1 + 2 ULP. culori's
    // inGamut('a98') reports false for the same input; we mirror.
    assert!(!in_gamut(&rgb(1.0, 1.0, 1.0), "a98"));
}

#[test]
fn in_gamut_a98_accepts_internal_a98() {
    assert!(in_gamut(&a98(0.5, 0.5, 0.5), "a98"));
}

#[test]
fn in_gamut_a98_rejects_rec2020_green() {
    assert!(!in_gamut(&rec2020(0.0, 1.0, 0.0), "a98"));
}

#[test]
fn clamp_gamut_a98_from_rec2020_green() {
    let out = match clamp_gamut(rec2020(0.0, 1.0, 0.0), "a98") {
        Color::Rec2020(v) => v,
        other => panic!("expected Rec2020, got {other:?}"),
    };
    close(out.r, 0.24846737098111393, "rec-a98 r");
    close(out.g, 0.944643248245651, "rec-a98 g");
    close(out.b, 0.16759940335199108, "rec-a98 b");
}

#[test]
fn clamp_gamut_a98_from_prophoto_blue() {
    let out = match clamp_gamut(prophoto(0.0, 0.0, 1.0), "a98") {
        Color::ProphotoRgb(v) => v,
        other => panic!("expected ProphotoRgb, got {other:?}"),
    };
    close(out.r, 0.3441486696201124, "prophoto-a98 r");
    close(out.g, 0.14088192714196324, "prophoto-a98 g");
    close(out.b, 0.9446701812278185, "prophoto-a98 b");
}

// ----- prophoto -----

#[test]
fn in_gamut_prophoto_accepts_rgb_red() {
    // sRGB red sits well inside ProPhoto.
    assert!(in_gamut(&rgb(1.0, 0.0, 0.0), "prophoto"));
}

#[test]
fn clamp_gamut_prophoto_clips_out_of_range_prophoto() {
    let out = match clamp_gamut(prophoto(1.2, -0.1, 0.5), "prophoto") {
        Color::ProphotoRgb(v) => v,
        other => panic!("expected ProphotoRgb, got {other:?}"),
    };
    close(out.r, 1.0, "prophoto r");
    close(out.g, 0.0, "prophoto g");
    close(out.b, 0.5, "prophoto b");
}

#[test]
fn clamp_gamut_prophoto_in_gamut_passes_through() {
    // rec2020 orange sits inside the ProPhoto gamut.
    let inp = rec2020(1.0, 0.5, 0.0);
    let out = match clamp_gamut(inp, "prophoto") {
        Color::Rec2020(v) => v,
        other => panic!("expected Rec2020, got {other:?}"),
    };
    close(out.r, 1.0, "rec2020 r");
    close(out.g, 0.5, "rec2020 g");
    close(out.b, 0.0, "rec2020 b");
}
