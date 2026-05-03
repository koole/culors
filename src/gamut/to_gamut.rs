//! `to_gamut` — CSS Color Module 4 gamut-mapping algorithm.
//!
//! Mirrors culori 4.0.2's `toGamut('rgb', 'oklch')` with its default
//! difference function `differenceEuclidean('oklch')` and just-noticeable
//! difference `jnd = 0.02`. The chroma bisection runs in OkLCh; the
//! candidate is judged in gamut either by the literal sRGB box test or by
//! a small ΔE comparison against its naïvely-clipped projection.

use crate::difference::difference_euclidean;
use crate::gamut::clamp::clamp_gamut;
use crate::gamut::in_gamut::{color_to_rgb, in_gamut};
use crate::spaces::{Oklab, Oklch, Rgb};
use crate::Color;

/// CSS Color Module 4 gamut-mapping algorithm.
///
/// Returns a color in the destination gamut named by `mode`. The chroma
/// bisection runs in OkLCh; the just-noticeable-difference threshold is
/// `0.02` units of `differenceEuclidean('oklch')`. These constants come
/// from culori 4.0.2's defaults.
///
/// The result is returned in `mode` (typically `"rgb"`). For `mode` values
/// without a gamut definition the input is returned converted to that mode
/// without further mapping.
///
/// # Panics
///
/// Panics if `mode` is unknown.
pub fn to_gamut(color: Color, mode: &str) -> Color {
    // For lab/lch/oklab/oklch/xyz/lrgb the mode has no gamut: there is
    // nothing to map. culori still calls the destination converter; we do
    // the same so the output mode matches `mode`.
    let dest_has_gamut = matches!(mode, "rgb" | "hsl" | "hsv" | "hwb");
    if !dest_has_gamut {
        return convert_to_mode(color, mode);
    }

    // Convert the input to OkLCh — the UCS used for chroma bisection.
    let mut candidate = to_oklch(color);
    if candidate.l.is_nan() {
        candidate.l = 0.0;
    }
    if candidate.c.is_nan() {
        candidate.c = 0.0;
    }

    // Short-circuit at the OkLCh lightness range boundaries: `l >= 1`
    // returns destination white, `l <= 0` returns destination black. This
    // matches culori's `ranges.l` early-exit for the OkLCh ucs.
    if candidate.l >= 1.0 {
        return dest_white(mode, candidate.alpha);
    }
    if candidate.l <= 0.0 {
        return dest_black(mode, candidate.alpha);
    }

    // If the OkLCh candidate already maps to an in-gamut color, return its
    // conversion to `mode` (no further bisection needed).
    let candidate_color = Color::Oklch(candidate);
    if in_gamut(&candidate_color, mode) {
        return convert_to_mode(candidate_color, mode);
    }

    // Bisect chroma. culori's epsilon is `(ranges.c[1] - ranges.c[0]) /
    // 4000`. For OkLCh `ranges.c = [0, 0.4]`, giving `epsilon = 0.0001`.
    const EPSILON: f64 = 0.4 / 4000.0;
    // Just-noticeable difference threshold for the deltaE Euclidean-OkLCh
    // metric. Verbatim from culori's `toGamut` default.
    const JND: f64 = 0.02;

    let mut start = 0.0;
    let mut end = candidate.c;
    let mut last_clipped: Oklch = unwrap_oklch(clamp_gamut(Color::Oklch(candidate), mode));
    let de = difference_euclidean("oklch");
    while end - start > EPSILON {
        candidate.c = (start + end) * 0.5;
        let working = Color::Oklch(candidate);
        let clipped = unwrap_oklch(clamp_gamut(working, mode));
        let in_g = in_gamut(&working, mode);
        let delta = de(&working, &Color::Oklch(clipped));
        if in_g || delta <= JND {
            start = candidate.c;
        } else {
            end = candidate.c;
        }
        last_clipped = clipped;
    }

    let final_candidate = Color::Oklch(candidate);
    if in_gamut(&final_candidate, mode) {
        convert_to_mode(final_candidate, mode)
    } else {
        // `last_clipped` is in OkLCh (clamp_gamut preserves source mode).
        // Convert it to `mode` for the final output.
        convert_to_mode(Color::Oklch(last_clipped), mode)
    }
}

fn unwrap_oklch(c: Color) -> Oklch {
    match c {
        Color::Oklch(x) => x,
        other => panic!("internal: expected Oklch, got {other:?}"),
    }
}

fn dest_white(mode: &str, alpha: Option<f64>) -> Color {
    // `destMode.white` for rgb/hsl/hsv/hwb is sRGB white (1, 1, 1).
    // culori spreads `destMode.white` then sets `mode: dest`, which means
    // the returned color is rgb-shaped. For modes whose `destMode.white`
    // doesn't have keys for the cylindrical channels (`hsl`, `hsv`,
    // `hwb`), culori carries the rgb keys and stamps `mode: 'hsl'`, which
    // is technically a malformed object. We avoid that by converting the
    // sRGB white into the destination mode.
    let white = Color::Rgb(Rgb {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        alpha,
    });
    convert_to_mode(white, mode)
}

fn dest_black(mode: &str, alpha: Option<f64>) -> Color {
    let black = Color::Rgb(Rgb {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        alpha,
    });
    convert_to_mode(black, mode)
}

fn to_oklch(color: Color) -> Oklch {
    use crate::traits::ColorSpace;
    match color {
        Color::Oklch(x) => x,
        Color::Oklab(x) => x.into(),
        Color::Rgb(x) => x.into(),
        Color::LinearRgb(x) => Oklab::from(x).into(),
        other => Oklch::from(Oklab::from_xyz65(crate::gamut::clamp::to_xyz65(other))),
    }
}

fn convert_to_mode(color: Color, mode: &str) -> Color {
    use crate::spaces::{Hsl, Hsv, Hwb, Lab, Lch, LinearRgb, Xyz50, Xyz65};
    let rgb = color_to_rgb(color);
    match mode {
        "rgb" => Color::Rgb(rgb),
        "lrgb" => Color::LinearRgb(LinearRgb::from(rgb)),
        "hsl" => Color::Hsl(Hsl::from(rgb)),
        "hsv" => Color::Hsv(Hsv::from(rgb)),
        "hwb" => Color::Hwb(Hwb::from(Hsv::from(rgb))),
        "lab" => Color::Lab(Lab::from(rgb)),
        "lch" => Color::Lch(Lch::from(rgb)),
        "oklab" => Color::Oklab(Oklab::from(rgb)),
        "oklch" => Color::Oklch(Oklch::from(rgb)),
        "xyz50" => Color::Xyz50(crate::convert::<Rgb, Xyz50>(rgb)),
        "xyz65" => Color::Xyz65(crate::convert::<Rgb, Xyz65>(rgb)),
        other => panic!("to_gamut: unknown mode '{other}'"),
    }
}
