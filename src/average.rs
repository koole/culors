//! Color averaging across CSS Color Module 4 spaces.
//!
//! Mirrors culori 4.0.2's `src/average.js`. The public entry point
//! [`average`] takes a slice of colors and a mode string and returns a
//! [`Color`] in the requested mode whose channels are the per-channel
//! average of the inputs (after conversion to `mode`).
//!
//! Helpers [`average_number`] and [`average_angle`] are exported because
//! culori exports them and they are useful in their own right. Both treat
//! `NaN` as "missing" — culori uses `undefined`, which we model as `NaN`
//! for non-alpha channels and `Option<f64>` for alpha.
//!
//! # Algorithm
//!
//! For each channel of the target mode:
//!
//! - hue channels (the `h` of `hsl`, `hsv`, `hwb`, `lch`, `oklch`) use
//!   [`average_angle`] — the circular mean from
//!   <https://en.wikipedia.org/wiki/Mean_of_circular_quantities>;
//! - every other channel, including alpha, uses [`average_number`].
//!
//! When every input value for a channel is missing, the channel is left
//! at its default (`NaN` for non-alpha, `None` for alpha). culori does
//! that by skipping the assignment in its `reduce`.

use crate::convert::convert;
use crate::spaces::{
    Cubehelix, Dlab, Dlch, Hpluv, Hsi, Hsl, Hsluv, Hsv, Hwb, Itp, Jab, Jch, Lab, Lch, Lchuv,
    LinearRgb, Luv, Okhsl, Okhsv, Oklab, Oklch, ProphotoRgb, Rec2020, Rgb, Xyb, Xyz50, Xyz65, Yiq,
    A98, P3,
};
use crate::traits::ColorSpace;
use crate::Color;

/// Arithmetic mean of `values`, ignoring `NaN`. Returns `NaN` if every
/// value is `NaN` or if `values` is empty.
pub fn average_number(values: &[f64]) -> f64 {
    let mut sum = 0.0;
    let mut count = 0usize;
    for &v in values {
        if !v.is_nan() {
            sum += v;
            count += 1;
        }
    }
    if count == 0 {
        f64::NAN
    } else {
        sum / count as f64
    }
}

/// Circular mean of `angles` in degrees, ignoring `NaN`.
///
/// Returns a value in `[0, 360]`. The upper bound shows up when atan2
/// underflow lands the raw result just below zero, which the wrap-around
/// branch maps to `360 + angle`. culori behaves identically.
///
/// Empty input or all-NaN input returns `0` because `atan2(0, 0) = 0`,
/// and `0 < 0` is false.
pub fn average_angle(angles: &[f64]) -> f64 {
    let mut sum_sin = 0.0;
    let mut sum_cos = 0.0;
    for &a in angles {
        if !a.is_nan() {
            let rad = a.to_radians();
            sum_sin += rad.sin();
            sum_cos += rad.cos();
        }
    }
    let angle = sum_sin.atan2(sum_cos).to_degrees();
    if angle < 0.0 {
        360.0 + angle
    } else {
        angle
    }
}

/// Average a list of colors in the requested `mode`.
///
/// Each color is converted to `mode`, then each channel is reduced
/// independently: hue channels use [`average_angle`], everything else
/// uses [`average_number`]. The resulting [`Color`] is of the requested
/// mode. When every input lacks a value for a given channel (all `NaN`
/// or, for alpha, all missing), the result keeps that channel as `NaN`
/// or `None`.
///
/// Supported modes: `rgb`, `lrgb`, `hsl`, `hsv`, `hwb`, `lab`, `lch`,
/// `oklab`, `oklch`, `xyz50`, `xyz65`, `p3`, `rec2020`, `a98`, `prophoto`,
/// `cubehelix`, `dlab`, `dlch`, `jab`, `jch`, `yiq`, `hsi`, `hsluv`,
/// `hpluv`, `okhsl`, `okhsv`, `itp`, `xyb`, `luv`, `lchuv`. (`hsluv` and
/// `hpluv` are culor extensions and not present in culori 4.0.2.)
///
/// # Panics
///
/// Panics if `mode` is unknown. Empty `colors` does not panic — it
/// returns the mode's default color (every channel `NaN`, alpha `None`).
pub fn average(colors: &[Color], mode: &str) -> Color {
    let info = mode_info(mode);

    let mut channels: Vec<Vec<f64>> = vec![Vec::with_capacity(colors.len()); info.channels.len()];
    let mut alphas: Vec<f64> = Vec::with_capacity(colors.len());
    for color in colors {
        let (chs, a) = decompose(*color, mode);
        for (i, v) in chs.iter().enumerate() {
            channels[i].push(*v);
        }
        alphas.push(a);
    }

    let mut out_channels = Vec::with_capacity(info.channels.len());
    for (i, ch) in info.channels.iter().enumerate() {
        let any_defined = channels[i].iter().any(|v| !v.is_nan());
        if !any_defined {
            out_channels.push(f64::NAN);
            continue;
        }
        let v = if ch.is_hue {
            average_angle(&channels[i])
        } else {
            average_number(&channels[i])
        };
        out_channels.push(v);
    }

    let alpha = if alphas.iter().any(|v| !v.is_nan()) {
        Some(average_number(&alphas))
    } else {
        None
    };

    compose(info.mode_str, &out_channels, alpha)
}

#[derive(Debug, Clone, Copy)]
struct ChannelInfo {
    is_hue: bool,
}

#[derive(Debug, Clone, Copy)]
struct ModeInfo {
    mode_str: &'static str,
    channels: &'static [ChannelInfo],
}

const RGB_CHANNELS: &[ChannelInfo] = &[
    ChannelInfo { is_hue: false },
    ChannelInfo { is_hue: false },
    ChannelInfo { is_hue: false },
];

const HSL_CHANNELS: &[ChannelInfo] = &[
    ChannelInfo { is_hue: true },
    ChannelInfo { is_hue: false },
    ChannelInfo { is_hue: false },
];

const HSV_CHANNELS: &[ChannelInfo] = HSL_CHANNELS;
const HWB_CHANNELS: &[ChannelInfo] = HSL_CHANNELS;

const LAB_CHANNELS: &[ChannelInfo] = RGB_CHANNELS;

const LCH_CHANNELS: &[ChannelInfo] = &[
    ChannelInfo { is_hue: false },
    ChannelInfo { is_hue: false },
    ChannelInfo { is_hue: true },
];

const XYZ_CHANNELS: &[ChannelInfo] = RGB_CHANNELS;

fn mode_info(mode: &str) -> ModeInfo {
    match mode {
        "rgb" => ModeInfo {
            mode_str: "rgb",
            channels: RGB_CHANNELS,
        },
        "lrgb" => ModeInfo {
            mode_str: "lrgb",
            channels: RGB_CHANNELS,
        },
        "hsl" => ModeInfo {
            mode_str: "hsl",
            channels: HSL_CHANNELS,
        },
        "hsv" => ModeInfo {
            mode_str: "hsv",
            channels: HSV_CHANNELS,
        },
        "hwb" => ModeInfo {
            mode_str: "hwb",
            channels: HWB_CHANNELS,
        },
        "lab" => ModeInfo {
            mode_str: "lab",
            channels: LAB_CHANNELS,
        },
        "lch" => ModeInfo {
            mode_str: "lch",
            channels: LCH_CHANNELS,
        },
        "oklab" => ModeInfo {
            mode_str: "oklab",
            channels: LAB_CHANNELS,
        },
        "oklch" => ModeInfo {
            mode_str: "oklch",
            channels: LCH_CHANNELS,
        },
        "xyz50" => ModeInfo {
            mode_str: "xyz50",
            channels: XYZ_CHANNELS,
        },
        "xyz65" => ModeInfo {
            mode_str: "xyz65",
            channels: XYZ_CHANNELS,
        },
        "p3" => ModeInfo {
            mode_str: "p3",
            channels: RGB_CHANNELS,
        },
        "rec2020" => ModeInfo {
            mode_str: "rec2020",
            channels: RGB_CHANNELS,
        },
        "a98" => ModeInfo {
            mode_str: "a98",
            channels: RGB_CHANNELS,
        },
        "prophoto" => ModeInfo {
            mode_str: "prophoto",
            channels: RGB_CHANNELS,
        },
        "cubehelix" => ModeInfo {
            mode_str: "cubehelix",
            channels: HSL_CHANNELS,
        },
        "dlab" => ModeInfo {
            mode_str: "dlab",
            channels: LAB_CHANNELS,
        },
        "dlch" => ModeInfo {
            mode_str: "dlch",
            channels: LCH_CHANNELS,
        },
        "jab" => ModeInfo {
            mode_str: "jab",
            channels: LAB_CHANNELS,
        },
        "jch" => ModeInfo {
            mode_str: "jch",
            channels: LCH_CHANNELS,
        },
        "yiq" => ModeInfo {
            mode_str: "yiq",
            channels: RGB_CHANNELS,
        },
        "hsi" => ModeInfo {
            mode_str: "hsi",
            channels: HSL_CHANNELS,
        },
        "hsluv" => ModeInfo {
            mode_str: "hsluv",
            channels: HSL_CHANNELS,
        },
        "hpluv" => ModeInfo {
            mode_str: "hpluv",
            channels: HSL_CHANNELS,
        },
        "okhsl" => ModeInfo {
            mode_str: "okhsl",
            channels: HSL_CHANNELS,
        },
        "okhsv" => ModeInfo {
            mode_str: "okhsv",
            channels: HSV_CHANNELS,
        },
        "itp" => ModeInfo {
            mode_str: "itp",
            channels: RGB_CHANNELS,
        },
        "xyb" => ModeInfo {
            mode_str: "xyb",
            channels: RGB_CHANNELS,
        },
        "luv" => ModeInfo {
            mode_str: "luv",
            channels: LAB_CHANNELS,
        },
        "lchuv" => ModeInfo {
            mode_str: "lchuv",
            channels: LCH_CHANNELS,
        },
        other => panic!("average: unknown mode '{other}'"),
    }
}

fn alpha_to_f64(a: Option<f64>) -> f64 {
    a.unwrap_or(f64::NAN)
}

fn decompose(c: Color, mode: &str) -> ([f64; 3], f64) {
    match mode {
        "rgb" => {
            let v: Rgb = match c {
                Color::Rgb(x) => x,
                Color::LinearRgb(x) => x.into(),
                Color::Hsl(x) => x.into(),
                Color::Hsv(x) => x.into(),
                Color::Hwb(x) => Hsv::from(x).into(),
                other => convert::<Xyz65, Rgb>(color_to_xyz65(other)),
            };
            ([v.r, v.g, v.b], alpha_to_f64(v.alpha))
        }
        "lrgb" => {
            let v: LinearRgb = match c {
                Color::Rgb(x) => x.into(),
                Color::LinearRgb(x) => x,
                other => convert::<Xyz65, LinearRgb>(color_to_xyz65(other)),
            };
            ([v.r, v.g, v.b], alpha_to_f64(v.alpha))
        }
        "hsl" => {
            let v: Hsl = match c {
                Color::Hsl(x) => x,
                Color::Rgb(x) => x.into(),
                Color::LinearRgb(x) => Rgb::from(x).into(),
                Color::Hsv(x) => Rgb::from(x).into(),
                Color::Hwb(x) => Rgb::from(Hsv::from(x)).into(),
                other => convert::<Xyz65, Rgb>(color_to_xyz65(other)).into(),
            };
            ([v.h, v.s, v.l], alpha_to_f64(v.alpha))
        }
        "hsv" => {
            let v: Hsv = match c {
                Color::Hsv(x) => x,
                Color::Hwb(x) => x.into(),
                Color::Rgb(x) => x.into(),
                Color::LinearRgb(x) => Rgb::from(x).into(),
                Color::Hsl(x) => Rgb::from(x).into(),
                other => convert::<Xyz65, Rgb>(color_to_xyz65(other)).into(),
            };
            ([v.h, v.s, v.v], alpha_to_f64(v.alpha))
        }
        "hwb" => {
            let v: Hwb = match c {
                Color::Hwb(x) => x,
                Color::Hsv(x) => x.into(),
                Color::Rgb(x) => Hsv::from(x).into(),
                Color::LinearRgb(x) => Hsv::from(Rgb::from(x)).into(),
                Color::Hsl(x) => Hsv::from(Rgb::from(x)).into(),
                other => Hsv::from(convert::<Xyz65, Rgb>(color_to_xyz65(other))).into(),
            };
            ([v.h, v.w, v.b], alpha_to_f64(v.alpha))
        }
        "lab" => {
            let v: Lab = match c {
                Color::Lab(x) => x,
                Color::Lch(x) => x.into(),
                Color::Xyz50(x) => x.into(),
                Color::Rgb(x) => x.into(),
                other => convert::<Xyz65, Lab>(color_to_xyz65(other)),
            };
            ([v.l, v.a, v.b], alpha_to_f64(v.alpha))
        }
        "lch" => {
            let v: Lch = match c {
                Color::Lch(x) => x,
                Color::Lab(x) => x.into(),
                Color::Rgb(x) => x.into(),
                other => convert::<Xyz65, Lab>(color_to_xyz65(other)).into(),
            };
            ([v.l, v.c, v.h], alpha_to_f64(v.alpha))
        }
        "oklab" => {
            let v: Oklab = match c {
                Color::Oklab(x) => x,
                Color::Oklch(x) => x.into(),
                Color::Rgb(x) => x.into(),
                Color::LinearRgb(x) => x.into(),
                other => convert::<Xyz65, Oklab>(color_to_xyz65(other)),
            };
            ([v.l, v.a, v.b], alpha_to_f64(v.alpha))
        }
        "oklch" => {
            let v: Oklch = match c {
                Color::Oklch(x) => x,
                Color::Oklab(x) => x.into(),
                Color::Rgb(x) => x.into(),
                Color::LinearRgb(x) => Oklab::from(x).into(),
                other => convert::<Xyz65, Oklab>(color_to_xyz65(other)).into(),
            };
            ([v.l, v.c, v.h], alpha_to_f64(v.alpha))
        }
        "xyz50" => {
            let v: Xyz50 = match c {
                Color::Xyz50(x) => x,
                Color::Lab(x) => x.into(),
                other => convert::<Xyz65, Xyz50>(color_to_xyz65(other)),
            };
            ([v.x, v.y, v.z], alpha_to_f64(v.alpha))
        }
        "xyz65" => {
            let v: Xyz65 = color_to_xyz65(c);
            ([v.x, v.y, v.z], alpha_to_f64(v.alpha))
        }
        "p3" => {
            let v: P3 = match c {
                Color::P3(x) => x,
                other => convert::<Xyz65, P3>(color_to_xyz65(other)),
            };
            ([v.r, v.g, v.b], alpha_to_f64(v.alpha))
        }
        "rec2020" => {
            let v: Rec2020 = match c {
                Color::Rec2020(x) => x,
                other => convert::<Xyz65, Rec2020>(color_to_xyz65(other)),
            };
            ([v.r, v.g, v.b], alpha_to_f64(v.alpha))
        }
        "a98" => {
            let v: A98 = match c {
                Color::A98(x) => x,
                other => convert::<Xyz65, A98>(color_to_xyz65(other)),
            };
            ([v.r, v.g, v.b], alpha_to_f64(v.alpha))
        }
        "prophoto" => {
            let v: ProphotoRgb = match c {
                Color::ProphotoRgb(x) => x,
                other => convert::<Xyz65, ProphotoRgb>(color_to_xyz65(other)),
            };
            ([v.r, v.g, v.b], alpha_to_f64(v.alpha))
        }
        "cubehelix" => {
            let v: Cubehelix = match c {
                Color::Cubehelix(x) => x,
                Color::Rgb(x) => x.into(),
                other => convert::<Xyz65, Rgb>(color_to_xyz65(other)).into(),
            };
            ([v.h, v.s, v.l], alpha_to_f64(v.alpha))
        }
        "dlab" => {
            let v: Dlab = match c {
                Color::Dlab(x) => x,
                Color::Rgb(x) => x.into(),
                other => convert::<Xyz65, Rgb>(color_to_xyz65(other)).into(),
            };
            ([v.l, v.a, v.b], alpha_to_f64(v.alpha))
        }
        "dlch" => {
            let v: Dlch = match c {
                Color::Dlch(x) => x,
                Color::Rgb(x) => x.into(),
                other => convert::<Xyz65, Rgb>(color_to_xyz65(other)).into(),
            };
            ([v.l, v.c, v.h], alpha_to_f64(v.alpha))
        }
        "jab" => {
            let v: Jab = match c {
                Color::Jab(x) => x,
                Color::Jch(x) => x.into(),
                Color::Rgb(x) => x.into(),
                other => convert::<Xyz65, Rgb>(color_to_xyz65(other)).into(),
            };
            ([v.j, v.a, v.b], alpha_to_f64(v.alpha))
        }
        "jch" => {
            let v: Jch = match c {
                Color::Jch(x) => x,
                Color::Jab(x) => x.into(),
                Color::Rgb(x) => x.into(),
                other => convert::<Xyz65, Rgb>(color_to_xyz65(other)).into(),
            };
            ([v.j, v.c, v.h], alpha_to_f64(v.alpha))
        }
        "yiq" => {
            let v: Yiq = match c {
                Color::Yiq(x) => x,
                Color::Rgb(x) => x.into(),
                other => convert::<Xyz65, Rgb>(color_to_xyz65(other)).into(),
            };
            ([v.y, v.i, v.q], alpha_to_f64(v.alpha))
        }
        "hsi" => {
            let v: Hsi = match c {
                Color::Hsi(x) => x,
                Color::Rgb(x) => x.into(),
                other => convert::<Xyz65, Rgb>(color_to_xyz65(other)).into(),
            };
            ([v.h, v.s, v.i], alpha_to_f64(v.alpha))
        }
        "hsluv" => {
            let v: Hsluv = match c {
                Color::Hsluv(x) => x,
                Color::Rgb(x) => x.into(),
                other => convert::<Xyz65, Rgb>(color_to_xyz65(other)).into(),
            };
            ([v.h, v.s, v.l], alpha_to_f64(v.alpha))
        }
        "hpluv" => {
            let v: Hpluv = match c {
                Color::Hpluv(x) => x,
                Color::Rgb(x) => x.into(),
                other => convert::<Xyz65, Rgb>(color_to_xyz65(other)).into(),
            };
            ([v.h, v.s, v.l], alpha_to_f64(v.alpha))
        }
        "okhsl" => {
            let v: Okhsl = match c {
                Color::Okhsl(x) => x,
                Color::Rgb(x) => x.into(),
                Color::Oklab(x) => x.into(),
                other => convert::<Xyz65, Rgb>(color_to_xyz65(other)).into(),
            };
            ([v.h, v.s, v.l], alpha_to_f64(v.alpha))
        }
        "okhsv" => {
            let v: Okhsv = match c {
                Color::Okhsv(x) => x,
                Color::Rgb(x) => x.into(),
                Color::Oklab(x) => x.into(),
                other => convert::<Xyz65, Rgb>(color_to_xyz65(other)).into(),
            };
            ([v.h, v.s, v.v], alpha_to_f64(v.alpha))
        }
        "itp" => {
            let v: Itp = match c {
                Color::Itp(x) => x,
                other => convert::<Xyz65, Itp>(color_to_xyz65(other)),
            };
            ([v.i, v.t, v.p], alpha_to_f64(v.alpha))
        }
        "xyb" => {
            let v: Xyb = match c {
                Color::Xyb(x) => x,
                Color::Rgb(x) => x.into(),
                other => convert::<Xyz65, Rgb>(color_to_xyz65(other)).into(),
            };
            ([v.x, v.y, v.b], alpha_to_f64(v.alpha))
        }
        "luv" => {
            let v: Luv = match c {
                Color::Luv(x) => x,
                Color::Lchuv(x) => x.into(),
                Color::Rgb(x) => x.into(),
                other => convert::<Xyz65, Luv>(color_to_xyz65(other)),
            };
            ([v.l, v.u, v.v], alpha_to_f64(v.alpha))
        }
        "lchuv" => {
            let v: Lchuv = match c {
                Color::Lchuv(x) => x,
                Color::Luv(x) => x.into(),
                Color::Rgb(x) => x.into(),
                other => convert::<Xyz65, Luv>(color_to_xyz65(other)).into(),
            };
            ([v.l, v.c, v.h], alpha_to_f64(v.alpha))
        }
        _ => unreachable!("mode_info already validated"),
    }
}

fn color_to_xyz65(c: Color) -> Xyz65 {
    match c {
        Color::Rgb(x) => x.to_xyz65(),
        Color::LinearRgb(x) => x.to_xyz65(),
        Color::Hsl(x) => x.to_xyz65(),
        Color::Hsv(x) => x.to_xyz65(),
        Color::Hwb(x) => x.to_xyz65(),
        Color::Lab(x) => x.to_xyz65(),
        Color::Lab65(x) => x.to_xyz65(),
        Color::Lch(x) => x.to_xyz65(),
        Color::Lch65(x) => x.to_xyz65(),
        Color::Oklab(x) => x.to_xyz65(),
        Color::Oklch(x) => x.to_xyz65(),
        Color::Xyz50(x) => x.to_xyz65(),
        Color::Xyz65(x) => x,
        Color::P3(x) => x.to_xyz65(),
        Color::Rec2020(x) => x.to_xyz65(),
        Color::A98(x) => x.to_xyz65(),
        Color::ProphotoRgb(x) => x.to_xyz65(),
        Color::Cubehelix(x) => x.to_xyz65(),
        Color::Dlab(x) => x.to_xyz65(),
        Color::Dlch(x) => x.to_xyz65(),
        Color::Jab(x) => x.to_xyz65(),
        Color::Jch(x) => x.to_xyz65(),
        Color::Yiq(x) => x.to_xyz65(),
        Color::Hsi(x) => x.to_xyz65(),
        Color::Hsluv(x) => x.to_xyz65(),
        Color::Hpluv(x) => x.to_xyz65(),
        Color::Okhsl(x) => x.to_xyz65(),
        Color::Okhsv(x) => x.to_xyz65(),
        Color::Itp(x) => x.to_xyz65(),
        Color::Xyb(x) => x.to_xyz65(),
        Color::Luv(x) => x.to_xyz65(),
        Color::Lchuv(x) => x.to_xyz65(),
        Color::Prismatic(x) => x.to_xyz65(),
    }
}

fn compose(mode: &str, channels: &[f64], alpha: Option<f64>) -> Color {
    match mode {
        "rgb" => Color::Rgb(Rgb {
            r: channels[0],
            g: channels[1],
            b: channels[2],
            alpha,
        }),
        "lrgb" => Color::LinearRgb(LinearRgb {
            r: channels[0],
            g: channels[1],
            b: channels[2],
            alpha,
        }),
        "hsl" => Color::Hsl(Hsl {
            h: channels[0],
            s: channels[1],
            l: channels[2],
            alpha,
        }),
        "hsv" => Color::Hsv(Hsv {
            h: channels[0],
            s: channels[1],
            v: channels[2],
            alpha,
        }),
        "hwb" => Color::Hwb(Hwb {
            h: channels[0],
            w: channels[1],
            b: channels[2],
            alpha,
        }),
        "lab" => Color::Lab(Lab {
            l: channels[0],
            a: channels[1],
            b: channels[2],
            alpha,
        }),
        "lch" => Color::Lch(Lch {
            l: channels[0],
            c: channels[1],
            h: channels[2],
            alpha,
        }),
        "oklab" => Color::Oklab(Oklab {
            l: channels[0],
            a: channels[1],
            b: channels[2],
            alpha,
        }),
        "oklch" => Color::Oklch(Oklch {
            l: channels[0],
            c: channels[1],
            h: channels[2],
            alpha,
        }),
        "xyz50" => Color::Xyz50(Xyz50 {
            x: channels[0],
            y: channels[1],
            z: channels[2],
            alpha,
        }),
        "xyz65" => Color::Xyz65(Xyz65 {
            x: channels[0],
            y: channels[1],
            z: channels[2],
            alpha,
        }),
        "p3" => Color::P3(P3 {
            r: channels[0],
            g: channels[1],
            b: channels[2],
            alpha,
        }),
        "rec2020" => Color::Rec2020(Rec2020 {
            r: channels[0],
            g: channels[1],
            b: channels[2],
            alpha,
        }),
        "a98" => Color::A98(A98 {
            r: channels[0],
            g: channels[1],
            b: channels[2],
            alpha,
        }),
        "prophoto" => Color::ProphotoRgb(ProphotoRgb {
            r: channels[0],
            g: channels[1],
            b: channels[2],
            alpha,
        }),
        "cubehelix" => Color::Cubehelix(Cubehelix {
            h: channels[0],
            s: channels[1],
            l: channels[2],
            alpha,
        }),
        "dlab" => Color::Dlab(Dlab {
            l: channels[0],
            a: channels[1],
            b: channels[2],
            alpha,
        }),
        "dlch" => Color::Dlch(Dlch {
            l: channels[0],
            c: channels[1],
            h: channels[2],
            alpha,
        }),
        "jab" => Color::Jab(Jab {
            j: channels[0],
            a: channels[1],
            b: channels[2],
            alpha,
        }),
        "jch" => Color::Jch(Jch {
            j: channels[0],
            c: channels[1],
            h: channels[2],
            alpha,
        }),
        "yiq" => Color::Yiq(Yiq {
            y: channels[0],
            i: channels[1],
            q: channels[2],
            alpha,
        }),
        "hsi" => Color::Hsi(Hsi {
            h: channels[0],
            s: channels[1],
            i: channels[2],
            alpha,
        }),
        "hsluv" => Color::Hsluv(Hsluv {
            h: channels[0],
            s: channels[1],
            l: channels[2],
            alpha,
        }),
        "hpluv" => Color::Hpluv(Hpluv {
            h: channels[0],
            s: channels[1],
            l: channels[2],
            alpha,
        }),
        "okhsl" => Color::Okhsl(Okhsl {
            h: channels[0],
            s: channels[1],
            l: channels[2],
            alpha,
        }),
        "okhsv" => Color::Okhsv(Okhsv {
            h: channels[0],
            s: channels[1],
            v: channels[2],
            alpha,
        }),
        "itp" => Color::Itp(Itp {
            i: channels[0],
            t: channels[1],
            p: channels[2],
            alpha,
        }),
        "xyb" => Color::Xyb(Xyb {
            x: channels[0],
            y: channels[1],
            b: channels[2],
            alpha,
        }),
        "luv" => Color::Luv(Luv {
            l: channels[0],
            u: channels[1],
            v: channels[2],
            alpha,
        }),
        "lchuv" => Color::Lchuv(Lchuv {
            l: channels[0],
            c: channels[1],
            h: channels[2],
            alpha,
        }),
        _ => unreachable!("mode_info already validated"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn average_number_arithmetic() {
        let v = average_number(&[1.0, 2.0, 3.0]);
        assert!((v - 2.0).abs() < 1e-12);
    }

    #[test]
    fn average_angle_circular() {
        let v = average_angle(&[10.0, 350.0]);
        assert!((v - 360.0).abs() < 1e-9 || v.abs() < 1e-9);
    }
}
