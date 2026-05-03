//! `clamp_gamut` — naïve per-channel clip, mirroring culori 4.0.2's
//! `clampGamut(mode)`.

use crate::gamut::in_gamut::{color_to_rgb, in_gamut};
use crate::spaces::{Hsl, Hsv, Hwb, LinearRgb, Rgb};
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

pub(crate) fn clamp_rgb_channels(c: Rgb) -> Rgb {
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
pub(crate) fn convert_rgb_back_to_source_mode(rgb: Rgb, template: Color) -> Color {
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
    }
}
