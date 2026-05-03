//! `in_gamut` — predicate matching culori's `inGamut(mode)`.

use crate::spaces::{Hsv, Rgb};
use crate::Color;

/// Returns `true` if `color` is inside the gamut of `mode`.
///
/// Mirrors culori 4.0.2's `inGamut(mode)` (`node_modules/culori/src/clamp.js`):
///
/// - `"rgb"`, `"hsl"`, `"hsv"`, `"hwb"` — convert `color` to sRGB and
///   require every channel in `[0, 1]`. The cylindrical modes have
///   `gamut: 'rgb'` in their definitions, so they share the sRGB box.
/// - any other mode (`"lab"`, `"lch"`, `"oklab"`, `"oklch"`, `"lrgb"`,
///   `"xyz50"`, `"xyz65"`) — culori's mode definition has no `gamut`
///   field, and `inGamut` returns the constant `true`.
///
/// Panics on an unknown mode string. Callers should validate `mode` up
/// front; in practice the four "real" gamut modes are `rgb` / `hsl` /
/// `hsv` / `hwb`.
pub fn in_gamut(color: &Color, mode: &str) -> bool {
    match mode {
        "rgb" | "hsl" | "hsv" | "hwb" => {
            let rgb = color_to_rgb(*color);
            inrange_rgb(&rgb)
        }
        "lrgb" | "lab" | "lch" | "oklab" | "oklch" | "xyz50" | "xyz65" => true,
        other => panic!("in_gamut: unknown mode '{other}'"),
    }
}

fn inrange_rgb(c: &Rgb) -> bool {
    // culori's `inrange_rgb` accepts `c.r === undefined` (channel absent).
    // Our typed structs always have channels, but NaN can stand in for an
    // absent channel after operations like interpolation. Match culori by
    // treating a NaN channel as in-range.
    in_range(c.r) && in_range(c.g) && in_range(c.b)
}

fn in_range(v: f64) -> bool {
    v.is_nan() || (0.0..=1.0).contains(&v)
}

/// Convert any `Color` to `Rgb` along the same path culori uses inside
/// `converter('rgb')`. We exploit each space's most direct route:
///
/// - cylindrical sRGB (`hsl` / `hsv` / `hwb`) — direct `From` implementations.
/// - everything else — through `XYZ65` via the [`crate::convert`] hub. For
///   the lab/lch/oklab/oklch family this matches culori's
///   `convertOklabToRgb` / `convertLabToRgb` to within ~1e-15 because the
///   linear-sRGB ↔ XYZ matrix and its inverse cancel.
pub(crate) fn color_to_rgb(c: Color) -> Rgb {
    match c {
        Color::Rgb(x) => x,
        Color::LinearRgb(x) => x.into(),
        Color::Hsl(x) => x.into(),
        Color::Hsv(x) => x.into(),
        Color::Hwb(x) => Rgb::from(Hsv::from(x)),
        Color::Lab(x) => crate::convert::<crate::spaces::Lab, Rgb>(x),
        Color::Lch(x) => crate::convert::<crate::spaces::Lch, Rgb>(x),
        Color::Oklab(x) => crate::convert::<crate::spaces::Oklab, Rgb>(x),
        Color::Oklch(x) => crate::convert::<crate::spaces::Oklch, Rgb>(x),
        Color::Xyz50(x) => crate::convert::<crate::spaces::Xyz50, Rgb>(x),
        Color::Xyz65(x) => crate::convert::<crate::spaces::Xyz65, Rgb>(x),
        Color::P3(x) => crate::convert::<crate::spaces::P3, Rgb>(x),
        Color::Rec2020(x) => crate::convert::<crate::spaces::Rec2020, Rgb>(x),
        Color::A98(x) => crate::convert::<crate::spaces::A98, Rgb>(x),
        Color::ProphotoRgb(x) => crate::convert::<crate::spaces::ProphotoRgb, Rgb>(x),
        Color::Cubehelix(x) => Rgb::from(x),
        Color::Dlab(x) => crate::convert::<crate::spaces::Dlab, Rgb>(x),
        Color::Dlch(x) => crate::convert::<crate::spaces::Dlch, Rgb>(x),
        Color::Jab(x) => crate::convert::<crate::spaces::Jab, Rgb>(x),
        Color::Jch(x) => crate::convert::<crate::spaces::Jch, Rgb>(x),
        Color::Yiq(x) => Rgb::from(x),
        Color::Hsi(x) => Rgb::from(x),
        Color::Hsluv(x) => Rgb::from(x),
        Color::Hpluv(x) => Rgb::from(x),
    }
}
