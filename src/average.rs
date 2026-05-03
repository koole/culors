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
use crate::spaces::{Hsl, Hsv, Hwb, Lab, Lch, LinearRgb, Oklab, Oklch, Rgb, Xyz50, Xyz65};
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
/// `mode` is one of culori's mode strings: `"rgb"`, `"lrgb"`, `"hsl"`,
/// `"hsv"`, `"hwb"`, `"lab"`, `"lch"`, `"oklab"`, `"oklch"`, `"xyz50"`,
/// `"xyz65"`.
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
        Color::Lch(x) => x.to_xyz65(),
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
