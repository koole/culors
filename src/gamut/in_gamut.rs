//! `in_gamut` — predicate matching culori's `inGamut(mode)`.

use crate::convert::convert;
use crate::spaces::{Hsv, ProphotoRgb, Rec2020, Rgb, A98, P3};
use crate::Color;

/// Returns `true` if `color` is inside the gamut of `mode`.
///
/// Mirrors culori 4.0.2's `inGamut(mode)` (`node_modules/culori/src/clamp.js`):
///
/// - `"rgb"`, `"hsl"`, `"hsv"`, `"hwb"` — convert `color` to sRGB and
///   require every channel in `[0, 1]`. The cylindrical modes have
///   `gamut: 'rgb'` in their definitions, so they share the sRGB box.
/// - `"p3"`, `"rec2020"`, `"a98"`, `"prophoto"` — convert `color` to that
///   wide-gamut RGB space and require every channel in `[0, 1]`.
/// - any other mode (`"lab"`, `"lch"`, `"oklab"`, `"oklch"`, `"lrgb"`,
///   `"xyz50"`, `"xyz65"`) — culori's mode definition has no `gamut`
///   field, and `inGamut` returns the constant `true`.
///
/// Panics on an unknown mode string. Callers should validate `mode` up
/// front; the gamut-bearing modes are sRGB / its cylindricals plus the
/// four wide-gamut RGB profiles.
pub fn in_gamut(color: &Color, mode: &str) -> bool {
    match mode {
        "rgb" | "hsl" | "hsv" | "hwb" => {
            let rgb = color_to_rgb(*color);
            inrange_rgb_channels(rgb.r, rgb.g, rgb.b)
        }
        "p3" => {
            let v: P3 = color_to_p3(*color);
            inrange_rgb_channels(v.r, v.g, v.b)
        }
        "rec2020" => {
            let v: Rec2020 = color_to_rec2020(*color);
            inrange_rgb_channels(v.r, v.g, v.b)
        }
        "a98" => {
            let v: A98 = color_to_a98(*color);
            inrange_rgb_channels(v.r, v.g, v.b)
        }
        "prophoto" => {
            let v: ProphotoRgb = color_to_prophoto(*color);
            inrange_rgb_channels(v.r, v.g, v.b)
        }
        "lrgb" | "lab" | "lch" | "oklab" | "oklch" | "xyz50" | "xyz65" => true,
        other => panic!("in_gamut: unknown mode '{other}'"),
    }
}

pub(crate) fn color_to_p3(c: Color) -> P3 {
    match c {
        Color::P3(x) => x,
        other => convert::<crate::spaces::Xyz65, P3>(super::clamp::to_xyz65(other)),
    }
}

pub(crate) fn color_to_rec2020(c: Color) -> Rec2020 {
    match c {
        Color::Rec2020(x) => x,
        other => convert::<crate::spaces::Xyz65, Rec2020>(super::clamp::to_xyz65(other)),
    }
}

pub(crate) fn color_to_a98(c: Color) -> A98 {
    match c {
        Color::A98(x) => x,
        other => convert::<crate::spaces::Xyz65, A98>(super::clamp::to_xyz65(other)),
    }
}

pub(crate) fn color_to_prophoto(c: Color) -> ProphotoRgb {
    match c {
        Color::ProphotoRgb(x) => x,
        other => convert::<crate::spaces::Xyz65, ProphotoRgb>(super::clamp::to_xyz65(other)),
    }
}

fn inrange_rgb_channels(r: f64, g: f64, b: f64) -> bool {
    // culori's `inrange_rgb` accepts `c.r === undefined` (channel absent).
    // Our typed structs always have channels, but NaN can stand in for an
    // absent channel after operations like interpolation. Match culori by
    // treating a NaN channel as in-range.
    in_range(r) && in_range(g) && in_range(b)
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
        Color::Lab65(x) => crate::convert::<crate::spaces::Lab65, Rgb>(x),
        Color::Lch(x) => crate::convert::<crate::spaces::Lch, Rgb>(x),
        Color::Lch65(x) => crate::convert::<crate::spaces::Lch65, Rgb>(x),
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
        Color::Okhsl(x) => crate::convert::<crate::spaces::Okhsl, Rgb>(x),
        Color::Okhsv(x) => crate::convert::<crate::spaces::Okhsv, Rgb>(x),
        Color::Itp(x) => crate::convert::<crate::spaces::Itp, Rgb>(x),
        Color::Xyb(x) => Rgb::from(x),
        Color::Luv(x) => crate::convert::<crate::spaces::Luv, Rgb>(x),
        Color::Lchuv(x) => crate::convert::<crate::spaces::Lchuv, Rgb>(x),
        Color::Prismatic(x) => Rgb::from(x),
    }
}
