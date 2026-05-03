//! Per-channel transfer pipeline.
//!
//! Mirrors culori 4.0.2's `map.js`. [`mapper`] returns a closure that
//! converts each input [`Color`] to a target mode, applies a per-channel
//! function (excluding `NaN` results), and optionally converts back to the
//! input's original mode.
//!
//! Three transfer factories are provided for the common cases:
//!
//! - [`map_alpha_multiply`] / [`map_alpha_divide`] for alpha pre-/de-multiply
//! - [`map_transfer_linear`] for `slope * v + intercept`
//! - [`map_transfer_gamma`] for `amp * v^exp + offset`
//!
//! Each transfer factory returns an `Fn(f64, &str, &Color) -> f64`, ready to
//! plug into [`mapper`]. The `&str` is the channel name; the `&Color` is the
//! mode-converted color so transfers can read sibling channels (e.g. alpha
//! pre-multiply needs the color's alpha).

use crate::color::Color;
use crate::spaces::{
    Cubehelix, Dlab, Dlch, Hpluv, Hsi, Hsl, Hsluv, Hsv, Hwb, Itp, Jab, Jch, Lab, Lab65, Lch, Lch65,
    Lchuv, LinearRgb, Luv, Okhsl, Okhsv, Oklab, Oklch, Prismatic, ProphotoRgb, Rec2020, Rgb, Xyb,
    Xyz50, Xyz65, Yiq, A98, P3,
};
use crate::traits::ColorSpace;

/// Returns a closure that applies `fn_` to every non-alpha channel of every
/// input color.
///
/// `mode` selects the working space — the input is converted into it (via
/// [`crate::convert_culori`]'s culori-mirroring routing), `fn_` is called on
/// each named channel of that space (alpha included; transfers like
/// [`map_alpha_multiply`] decide whether to act on alpha or not), and a new
/// [`Color`] is composed.
///
/// `preserve_mode = true` converts the result back to the input's original
/// mode before returning. This matches culori's third `mapper` argument.
///
/// `NaN` results from `fn_` are dropped (matching culori's `isNaN` skip).
/// Resulting channels default to `f64::NAN` when omitted.
pub fn mapper<F>(fn_: F, mode: &'static str, preserve_mode: bool) -> impl Fn(&Color) -> Color
where
    F: Fn(f64, &str, &Color) -> f64,
{
    move |color: &Color| {
        let conv = convert_to_mode(color, mode);
        let channels = mode_channel_names(mode);
        let mut buf = [f64::NAN; 4];
        for (i, ch_name) in channels.iter().enumerate() {
            let v = read_channel(&conv, ch_name);
            let mapped = fn_(v, ch_name, &conv);
            if !mapped.is_nan() {
                buf[i] = mapped;
            }
        }
        // Alpha is also passed through fn_ at culori's "alpha" channel name.
        let in_alpha = color_alpha(&conv);
        let alpha_in = in_alpha.unwrap_or(f64::NAN);
        let alpha_mapped = fn_(alpha_in, "alpha", &conv);
        let alpha_out = if alpha_mapped.is_nan() {
            None
        } else {
            Some(alpha_mapped)
        };

        let res = compose(mode, &buf, alpha_out);
        if !preserve_mode {
            return res;
        }
        let in_mode = color.mode();
        if in_mode == mode {
            res
        } else {
            convert_to_mode(&res, in_mode)
        }
    }
}

/// Pre-multiply non-alpha channels by alpha. Identical to culori's
/// `mapAlphaMultiply`.
pub fn map_alpha_multiply() -> impl Fn(f64, &str, &Color) -> f64 {
    |v: f64, ch: &str, c: &Color| {
        if ch == "alpha" {
            v
        } else {
            let a = color_alpha(c).unwrap_or(1.0);
            let value = if v.is_nan() { 0.0 } else { v };
            value * a
        }
    }
}

/// Inverse of [`map_alpha_multiply`]. When alpha is `0` the channel is
/// returned unchanged (avoiding division by zero), matching culori.
pub fn map_alpha_divide() -> impl Fn(f64, &str, &Color) -> f64 {
    |v: f64, ch: &str, c: &Color| {
        if ch == "alpha" {
            return v;
        }
        let a = color_alpha(c).unwrap_or(1.0);
        if a == 0.0 {
            return v;
        }
        let value = if v.is_nan() { 0.0 } else { v };
        value / a
    }
}

/// Linear transfer `slope * v + intercept` on every non-alpha channel.
pub fn map_transfer_linear(slope: f64, intercept: f64) -> impl Fn(f64, &str, &Color) -> f64 {
    move |v: f64, ch: &str, _c: &Color| {
        if ch == "alpha" {
            v
        } else {
            v * slope + intercept
        }
    }
}

/// Gamma transfer `amp * v^exp + offset` on every non-alpha channel.
pub fn map_transfer_gamma(amp: f64, exp: f64, offset: f64) -> impl Fn(f64, &str, &Color) -> f64 {
    move |v: f64, ch: &str, _c: &Color| {
        if ch == "alpha" {
            v
        } else {
            amp * v.powf(exp) + offset
        }
    }
}

fn convert_to_mode(c: &Color, mode: &str) -> Color {
    c.convert_to(mode)
        .unwrap_or_else(|| panic!("map: unsupported mode `{mode}`"))
}

fn mode_channel_names(mode: &str) -> &'static [&'static str] {
    match mode {
        "rgb" => Rgb::CHANNELS,
        "lrgb" => LinearRgb::CHANNELS,
        "hsl" => Hsl::CHANNELS,
        "hsv" => Hsv::CHANNELS,
        "hwb" => Hwb::CHANNELS,
        "lab" => Lab::CHANNELS,
        "lab65" => Lab65::CHANNELS,
        "lch" => Lch::CHANNELS,
        "lch65" => Lch65::CHANNELS,
        "oklab" => Oklab::CHANNELS,
        "oklch" => Oklch::CHANNELS,
        "xyz50" => Xyz50::CHANNELS,
        "xyz65" => Xyz65::CHANNELS,
        "p3" => P3::CHANNELS,
        "rec2020" => Rec2020::CHANNELS,
        "a98" => A98::CHANNELS,
        "prophoto" => ProphotoRgb::CHANNELS,
        "cubehelix" => Cubehelix::CHANNELS,
        "dlab" => Dlab::CHANNELS,
        "dlch" => Dlch::CHANNELS,
        "jab" => Jab::CHANNELS,
        "jch" => Jch::CHANNELS,
        "yiq" => Yiq::CHANNELS,
        "hsi" => Hsi::CHANNELS,
        "hsluv" => Hsluv::CHANNELS,
        "hpluv" => Hpluv::CHANNELS,
        "okhsl" => Okhsl::CHANNELS,
        "okhsv" => Okhsv::CHANNELS,
        "itp" => Itp::CHANNELS,
        "xyb" => Xyb::CHANNELS,
        "luv" => Luv::CHANNELS,
        "lchuv" => Lchuv::CHANNELS,
        "prismatic" => Prismatic::CHANNELS,
        _ => panic!("map: unsupported mode `{mode}`"),
    }
}

fn color_alpha(c: &Color) -> Option<f64> {
    match c {
        Color::Rgb(x) => x.alpha,
        Color::LinearRgb(x) => x.alpha,
        Color::Hsl(x) => x.alpha,
        Color::Hsv(x) => x.alpha,
        Color::Hwb(x) => x.alpha,
        Color::Lab(x) => x.alpha,
        Color::Lab65(x) => x.alpha,
        Color::Lch(x) => x.alpha,
        Color::Lch65(x) => x.alpha,
        Color::Oklab(x) => x.alpha,
        Color::Oklch(x) => x.alpha,
        Color::Xyz50(x) => x.alpha,
        Color::Xyz65(x) => x.alpha,
        Color::P3(x) => x.alpha,
        Color::Rec2020(x) => x.alpha,
        Color::A98(x) => x.alpha,
        Color::ProphotoRgb(x) => x.alpha,
        Color::Cubehelix(x) => x.alpha,
        Color::Dlab(x) => x.alpha,
        Color::Dlch(x) => x.alpha,
        Color::Jab(x) => x.alpha,
        Color::Jch(x) => x.alpha,
        Color::Yiq(x) => x.alpha,
        Color::Hsi(x) => x.alpha,
        Color::Hsluv(x) => x.alpha,
        Color::Hpluv(x) => x.alpha,
        Color::Okhsl(x) => x.alpha,
        Color::Okhsv(x) => x.alpha,
        Color::Itp(x) => x.alpha,
        Color::Xyb(x) => x.alpha,
        Color::Luv(x) => x.alpha,
        Color::Lchuv(x) => x.alpha,
        Color::Prismatic(x) => x.alpha,
    }
}

fn read_channel(c: &Color, ch: &str) -> f64 {
    match (c, ch) {
        (Color::Rgb(x), "r") => x.r,
        (Color::Rgb(x), "g") => x.g,
        (Color::Rgb(x), "b") => x.b,
        (Color::LinearRgb(x), "r") => x.r,
        (Color::LinearRgb(x), "g") => x.g,
        (Color::LinearRgb(x), "b") => x.b,
        (Color::Hsl(x), "h") => x.h,
        (Color::Hsl(x), "s") => x.s,
        (Color::Hsl(x), "l") => x.l,
        (Color::Hsv(x), "h") => x.h,
        (Color::Hsv(x), "s") => x.s,
        (Color::Hsv(x), "v") => x.v,
        (Color::Hwb(x), "h") => x.h,
        (Color::Hwb(x), "w") => x.w,
        (Color::Hwb(x), "b") => x.b,
        (Color::Lab(x), "l") => x.l,
        (Color::Lab(x), "a") => x.a,
        (Color::Lab(x), "b") => x.b,
        (Color::Lab65(x), "l") => x.l,
        (Color::Lab65(x), "a") => x.a,
        (Color::Lab65(x), "b") => x.b,
        (Color::Lch(x), "l") => x.l,
        (Color::Lch(x), "c") => x.c,
        (Color::Lch(x), "h") => x.h,
        (Color::Lch65(x), "l") => x.l,
        (Color::Lch65(x), "c") => x.c,
        (Color::Lch65(x), "h") => x.h,
        (Color::Oklab(x), "l") => x.l,
        (Color::Oklab(x), "a") => x.a,
        (Color::Oklab(x), "b") => x.b,
        (Color::Oklch(x), "l") => x.l,
        (Color::Oklch(x), "c") => x.c,
        (Color::Oklch(x), "h") => x.h,
        (Color::Xyz50(x), "x") => x.x,
        (Color::Xyz50(x), "y") => x.y,
        (Color::Xyz50(x), "z") => x.z,
        (Color::Xyz65(x), "x") => x.x,
        (Color::Xyz65(x), "y") => x.y,
        (Color::Xyz65(x), "z") => x.z,
        (Color::P3(x), "r") => x.r,
        (Color::P3(x), "g") => x.g,
        (Color::P3(x), "b") => x.b,
        (Color::Rec2020(x), "r") => x.r,
        (Color::Rec2020(x), "g") => x.g,
        (Color::Rec2020(x), "b") => x.b,
        (Color::A98(x), "r") => x.r,
        (Color::A98(x), "g") => x.g,
        (Color::A98(x), "b") => x.b,
        (Color::ProphotoRgb(x), "r") => x.r,
        (Color::ProphotoRgb(x), "g") => x.g,
        (Color::ProphotoRgb(x), "b") => x.b,
        (Color::Cubehelix(x), "h") => x.h,
        (Color::Cubehelix(x), "s") => x.s,
        (Color::Cubehelix(x), "l") => x.l,
        (Color::Dlab(x), "l") => x.l,
        (Color::Dlab(x), "a") => x.a,
        (Color::Dlab(x), "b") => x.b,
        (Color::Dlch(x), "l") => x.l,
        (Color::Dlch(x), "c") => x.c,
        (Color::Dlch(x), "h") => x.h,
        (Color::Jab(x), "j") => x.j,
        (Color::Jab(x), "a") => x.a,
        (Color::Jab(x), "b") => x.b,
        (Color::Jch(x), "j") => x.j,
        (Color::Jch(x), "c") => x.c,
        (Color::Jch(x), "h") => x.h,
        (Color::Yiq(x), "y") => x.y,
        (Color::Yiq(x), "i") => x.i,
        (Color::Yiq(x), "q") => x.q,
        (Color::Hsi(x), "h") => x.h,
        (Color::Hsi(x), "s") => x.s,
        (Color::Hsi(x), "i") => x.i,
        (Color::Hsluv(x), "h") => x.h,
        (Color::Hsluv(x), "s") => x.s,
        (Color::Hsluv(x), "l") => x.l,
        (Color::Hpluv(x), "h") => x.h,
        (Color::Hpluv(x), "s") => x.s,
        (Color::Hpluv(x), "l") => x.l,
        (Color::Okhsl(x), "h") => x.h,
        (Color::Okhsl(x), "s") => x.s,
        (Color::Okhsl(x), "l") => x.l,
        (Color::Okhsv(x), "h") => x.h,
        (Color::Okhsv(x), "s") => x.s,
        (Color::Okhsv(x), "v") => x.v,
        (Color::Itp(x), "i") => x.i,
        (Color::Itp(x), "t") => x.t,
        (Color::Itp(x), "p") => x.p,
        (Color::Xyb(x), "x") => x.x,
        (Color::Xyb(x), "y") => x.y,
        (Color::Xyb(x), "b") => x.b,
        (Color::Luv(x), "l") => x.l,
        (Color::Luv(x), "u") => x.u,
        (Color::Luv(x), "v") => x.v,
        (Color::Lchuv(x), "l") => x.l,
        (Color::Lchuv(x), "c") => x.c,
        (Color::Lchuv(x), "h") => x.h,
        (Color::Prismatic(x), "l") => x.l,
        (Color::Prismatic(x), "r") => x.r,
        (Color::Prismatic(x), "g") => x.g,
        (Color::Prismatic(x), "b") => x.b,
        _ => f64::NAN,
    }
}

fn compose(mode: &str, ch: &[f64; 4], alpha: Option<f64>) -> Color {
    match mode {
        "rgb" => Color::Rgb(Rgb {
            r: ch[0],
            g: ch[1],
            b: ch[2],
            alpha,
        }),
        "lrgb" => Color::LinearRgb(LinearRgb {
            r: ch[0],
            g: ch[1],
            b: ch[2],
            alpha,
        }),
        "hsl" => Color::Hsl(Hsl {
            h: ch[0],
            s: ch[1],
            l: ch[2],
            alpha,
        }),
        "hsv" => Color::Hsv(Hsv {
            h: ch[0],
            s: ch[1],
            v: ch[2],
            alpha,
        }),
        "hwb" => Color::Hwb(Hwb {
            h: ch[0],
            w: ch[1],
            b: ch[2],
            alpha,
        }),
        "lab" => Color::Lab(Lab {
            l: ch[0],
            a: ch[1],
            b: ch[2],
            alpha,
        }),
        "lab65" => Color::Lab65(Lab65 {
            l: ch[0],
            a: ch[1],
            b: ch[2],
            alpha,
        }),
        "lch" => Color::Lch(Lch {
            l: ch[0],
            c: ch[1],
            h: ch[2],
            alpha,
        }),
        "lch65" => Color::Lch65(Lch65 {
            l: ch[0],
            c: ch[1],
            h: ch[2],
            alpha,
        }),
        "oklab" => Color::Oklab(Oklab {
            l: ch[0],
            a: ch[1],
            b: ch[2],
            alpha,
        }),
        "oklch" => Color::Oklch(Oklch {
            l: ch[0],
            c: ch[1],
            h: ch[2],
            alpha,
        }),
        "xyz50" => Color::Xyz50(Xyz50 {
            x: ch[0],
            y: ch[1],
            z: ch[2],
            alpha,
        }),
        "xyz65" => Color::Xyz65(Xyz65 {
            x: ch[0],
            y: ch[1],
            z: ch[2],
            alpha,
        }),
        "p3" => Color::P3(P3 {
            r: ch[0],
            g: ch[1],
            b: ch[2],
            alpha,
        }),
        "rec2020" => Color::Rec2020(Rec2020 {
            r: ch[0],
            g: ch[1],
            b: ch[2],
            alpha,
        }),
        "a98" => Color::A98(A98 {
            r: ch[0],
            g: ch[1],
            b: ch[2],
            alpha,
        }),
        "prophoto" => Color::ProphotoRgb(ProphotoRgb {
            r: ch[0],
            g: ch[1],
            b: ch[2],
            alpha,
        }),
        "cubehelix" => Color::Cubehelix(Cubehelix {
            h: ch[0],
            s: ch[1],
            l: ch[2],
            alpha,
        }),
        "dlab" => Color::Dlab(Dlab {
            l: ch[0],
            a: ch[1],
            b: ch[2],
            alpha,
        }),
        "dlch" => Color::Dlch(Dlch {
            l: ch[0],
            c: ch[1],
            h: ch[2],
            alpha,
        }),
        "jab" => Color::Jab(Jab {
            j: ch[0],
            a: ch[1],
            b: ch[2],
            alpha,
        }),
        "jch" => Color::Jch(Jch {
            j: ch[0],
            c: ch[1],
            h: ch[2],
            alpha,
        }),
        "yiq" => Color::Yiq(Yiq {
            y: ch[0],
            i: ch[1],
            q: ch[2],
            alpha,
        }),
        "hsi" => Color::Hsi(Hsi {
            h: ch[0],
            s: ch[1],
            i: ch[2],
            alpha,
        }),
        "hsluv" => Color::Hsluv(Hsluv {
            h: ch[0],
            s: ch[1],
            l: ch[2],
            alpha,
        }),
        "hpluv" => Color::Hpluv(Hpluv {
            h: ch[0],
            s: ch[1],
            l: ch[2],
            alpha,
        }),
        "okhsl" => Color::Okhsl(Okhsl {
            h: ch[0],
            s: ch[1],
            l: ch[2],
            alpha,
        }),
        "okhsv" => Color::Okhsv(Okhsv {
            h: ch[0],
            s: ch[1],
            v: ch[2],
            alpha,
        }),
        "itp" => Color::Itp(Itp {
            i: ch[0],
            t: ch[1],
            p: ch[2],
            alpha,
        }),
        "xyb" => Color::Xyb(Xyb {
            x: ch[0],
            y: ch[1],
            b: ch[2],
            alpha,
        }),
        "luv" => Color::Luv(Luv {
            l: ch[0],
            u: ch[1],
            v: ch[2],
            alpha,
        }),
        "lchuv" => Color::Lchuv(Lchuv {
            l: ch[0],
            c: ch[1],
            h: ch[2],
            alpha,
        }),
        "prismatic" => Color::Prismatic(Prismatic {
            l: ch[0],
            r: ch[1],
            g: ch[2],
            b: ch[3],
            alpha,
        }),
        _ => unreachable!(),
    }
}
