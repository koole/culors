//! `clamp_gamut` (naïve per-channel clip) and `clamp_chroma` (chroma
//! bisection in an LCh-like space). Both mirror culori 4.0.2's `clamp.js`.

use crate::gamut::in_gamut::{color_to_rgb, in_gamut};
use crate::spaces::{Hsl, Hsv, Hwb, Lch, LinearRgb, Oklch, Rgb};
use crate::Color;

/// Naïve per-channel clamp into the gamut named by `mode`.
///
/// Mirrors culori's `clampGamut(mode)`: convert the color to the gamut's
/// underlying RGB space, clamp each channel to `[0, 1]`, then convert back
/// to the source mode. For `mode` values without a gamut definition
/// (`lab`/`lch`/`oklab`/`oklch`/`lrgb`/`xyz*`) the input is returned as-is.
///
/// The output stays in the source mode (the variant of the input
/// `Color`). Hue is not preserved — per-channel clipping shifts both
/// chroma and hue.
pub fn clamp_gamut(color: Color, mode: &str) -> Color {
    match mode {
        "rgb" | "hsl" | "hsv" | "hwb" => {
            if in_gamut(&color, mode) {
                return color;
            }
            let clamped_rgb = clamp_rgb_channels(color_to_rgb(color));
            convert_rgb_back_to_source_mode(clamped_rgb, color)
        }
        "lrgb" | "lab" | "lch" | "oklab" | "oklch" | "xyz50" | "xyz65" => color,
        other => panic!("clamp_gamut: unknown mode '{other}'"),
    }
}

fn clamp_rgb_channels(c: Rgb) -> Rgb {
    Rgb {
        // culori's `fixup_rgb`: `Math.max(0, Math.min(c.r !== undefined ? c.r
        // : 0, 1))`. A missing channel becomes `0`. Our typed struct can
        // carry NaN through the conversion stack; treat NaN as absent and
        // map it to `0` to match.
        r: clamp01(c.r),
        g: clamp01(c.g),
        b: clamp01(c.b),
        alpha: c.alpha,
    }
}

fn clamp01(v: f64) -> f64 {
    if v.is_nan() {
        0.0
    } else {
        v.clamp(0.0, 1.0)
    }
}

/// Convert an `Rgb` back to whatever mode `template` is in. We need the
/// source mode (not the source value), so the caller passes a representative
/// `Color` of that mode.
fn convert_rgb_back_to_source_mode(rgb: Rgb, template: Color) -> Color {
    match template {
        Color::Rgb(_) => Color::Rgb(rgb),
        Color::LinearRgb(_) => Color::LinearRgb(LinearRgb::from(rgb)),
        Color::Hsl(_) => Color::Hsl(Hsl::from(rgb)),
        Color::Hsv(_) => Color::Hsv(Hsv::from(rgb)),
        Color::Hwb(_) => Color::Hwb(Hwb::from(Hsv::from(rgb))),
        Color::Lab(_) => Color::Lab(rgb.into()),
        Color::Lch(_) => Color::Lch(rgb.into()),
        Color::Oklab(_) => Color::Oklab(rgb.into()),
        Color::Oklch(_) => Color::Oklch(rgb.into()),
        Color::Xyz50(_) => Color::Xyz50(crate::convert(rgb)),
        Color::Xyz65(_) => Color::Xyz65(crate::convert(rgb)),
        Color::P3(_) => Color::P3(crate::convert(rgb)),
        Color::Rec2020(_) => Color::Rec2020(crate::convert(rgb)),
        Color::A98(_) => Color::A98(crate::convert(rgb)),
        Color::ProphotoRgb(_) => Color::ProphotoRgb(crate::convert(rgb)),
    }
}

/// Clamp the chroma of `color` until the result fits inside the sRGB gamut.
///
/// Mirrors culori's `clampChroma(color, mode)` with `rgbGamut = 'rgb'` (the
/// only value culori uses internally). `mode` is the LCh-like space used
/// for the bisection; pass `"lch"` for CIELCh or `"oklch"` for OkLCh.
///
/// When the input is already displayable, it is returned unchanged. When
/// even chroma = 0 is not displayable (a degenerate L outside `[0, 1]` /
/// `[0, 100]` for OkLCh / Lch) the function falls back to per-channel sRGB
/// clipping, matching culori.
///
/// The result is returned in the source mode of `color`.
pub fn clamp_chroma(color: Color, mode: &str) -> Color {
    if in_gamut(&color, "rgb") {
        return color;
    }

    // Bisect in `mode`. culori limits this to LCh-like spaces; we do the
    // same via the explicit match. Other mode strings are an error.
    let (start_l, start_c, start_h, alpha, range_max) = match mode {
        "lch" => {
            let lch: Lch = match color {
                Color::Lch(x) => x,
                _ => crate::convert::<crate::spaces::Xyz65, Lch>(to_xyz65(color)),
            };
            (lch.l, lch.c, lch.h, lch.alpha, 150.0_f64)
        }
        "oklch" => {
            let oklch: Oklch = match color {
                Color::Oklch(x) => x,
                _ => {
                    use crate::traits::ColorSpace;
                    Oklch::from(crate::spaces::Oklab::from_xyz65(to_xyz65(color)))
                }
            };
            (oklch.l, oklch.c, oklch.h, oklch.alpha, 0.4_f64)
        }
        other => panic!("clamp_chroma: mode must be 'lch' or 'oklch', got '{other}'"),
    };

    // culori's `resolution = (range_max - range_min) / 2^13`. Both lch and
    // oklch have `range_min = 0`, so `resolution = range_max / 8192`.
    const ITER_DENOM: f64 = 8192.0;
    let resolution = range_max / ITER_DENOM;

    // Try chroma = 0 first. If even that is out of gamut, fall back to
    // per-channel rgb clipping of the chroma-zero color.
    let mut clamped = make_polar(mode, start_l, 0.0, start_h, alpha);
    if !in_gamut(&clamped, "rgb") {
        let rgb = clamp_rgb_channels(color_to_rgb(clamped));
        return convert_rgb_back_to_source_mode(rgb, color);
    }

    // Bisect: `start` is the largest chroma known to be in gamut, `end` is
    // the largest chroma known to be out.
    let mut start = 0.0;
    let mut end = if start_c.is_nan() { 0.0 } else { start_c };
    let mut last_good_c = 0.0;
    while end - start > resolution {
        let mid = start + (end - start) * 0.5;
        clamped = make_polar(mode, start_l, mid, start_h, alpha);
        if in_gamut(&clamped, "rgb") {
            last_good_c = mid;
            start = mid;
        } else {
            end = mid;
        }
    }

    // After the loop, `clamped` carries the last tested chroma. If that
    // chroma is in gamut, return as-is; otherwise back off to the last
    // known-good chroma. Mirrors culori's final ternary.
    let final_color = if in_gamut(&clamped, "rgb") {
        clamped
    } else {
        make_polar(mode, start_l, last_good_c, start_h, alpha)
    };

    // Convert the result back to the source mode.
    convert_polar_to_source_mode(final_color, color)
}

fn make_polar(mode: &str, l: f64, c: f64, h: f64, alpha: Option<f64>) -> Color {
    match mode {
        "lch" => Color::Lch(Lch { l, c, h, alpha }),
        "oklch" => Color::Oklch(Oklch { l, c, h, alpha }),
        _ => unreachable!(),
    }
}

fn convert_polar_to_source_mode(polar: Color, template: Color) -> Color {
    // Same source mode → return as-is.
    if std::mem::discriminant(&polar) == std::mem::discriminant(&template) {
        return polar;
    }
    // Otherwise fall through `XYZ65`. For the `Lab`/`Oklab` ↔ `Lch`/`Oklch`
    // sub-cases we could shave a step, but every other case routes through
    // the hub anyway.
    let xyz = to_xyz65(polar);
    from_xyz65_in_mode_of(xyz, template)
}

pub(crate) fn to_xyz65(c: Color) -> crate::spaces::Xyz65 {
    use crate::traits::ColorSpace;
    match c {
        Color::Rgb(x) => x.to_xyz65(),
        Color::LinearRgb(x) => x.to_xyz65(),
        Color::Hsl(x) => x.to_xyz65(),
        Color::Hsv(x) => x.to_xyz65(),
        Color::Hwb(x) => x.to_xyz65(),
        Color::Lab(x) => x.to_xyz65(),
        Color::Lch(x) => x.to_xyz65(),
        Color::Oklab(x) => x.to_xyz65(),
        Color::Oklch(x) => x.to_xyz65(),
        Color::Xyz50(x) => x.to_xyz65(),
        Color::Xyz65(x) => x,
        Color::P3(x) => x.to_xyz65(),
        Color::Rec2020(x) => x.to_xyz65(),
        Color::A98(x) => x.to_xyz65(),
        Color::ProphotoRgb(x) => x.to_xyz65(),
    }
}

fn from_xyz65_in_mode_of(xyz: crate::spaces::Xyz65, template: Color) -> Color {
    use crate::traits::ColorSpace;
    match template {
        Color::Rgb(_) => Color::Rgb(Rgb::from_xyz65(xyz)),
        Color::LinearRgb(_) => Color::LinearRgb(LinearRgb::from_xyz65(xyz)),
        Color::Hsl(_) => Color::Hsl(Hsl::from_xyz65(xyz)),
        Color::Hsv(_) => Color::Hsv(Hsv::from_xyz65(xyz)),
        Color::Hwb(_) => Color::Hwb(Hwb::from_xyz65(xyz)),
        Color::Lab(_) => Color::Lab(crate::spaces::Lab::from_xyz65(xyz)),
        Color::Lch(_) => Color::Lch(Lch::from_xyz65(xyz)),
        Color::Oklab(_) => Color::Oklab(crate::spaces::Oklab::from_xyz65(xyz)),
        Color::Oklch(_) => Color::Oklch(Oklch::from_xyz65(xyz)),
        Color::Xyz50(_) => Color::Xyz50(crate::spaces::Xyz50::from_xyz65(xyz)),
        Color::Xyz65(_) => Color::Xyz65(xyz),
        Color::P3(_) => Color::P3(crate::spaces::P3::from_xyz65(xyz)),
        Color::Rec2020(_) => Color::Rec2020(crate::spaces::Rec2020::from_xyz65(xyz)),
        Color::A98(_) => Color::A98(crate::spaces::A98::from_xyz65(xyz)),
        Color::ProphotoRgb(_) => Color::ProphotoRgb(crate::spaces::ProphotoRgb::from_xyz65(xyz)),
    }
}
