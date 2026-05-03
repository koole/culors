//! Color interpolation across CSS Color Module 4 spaces.
//!
//! Mirrors culori 4.0.2's `interpolate/interpolate.js`. The public entry
//! point [`interpolate`] takes a slice of colors and a mode string, returns
//! a closure `Fn(f64) -> Color`. The closure clamps `t` to `[0, 1]` and
//! produces a [`Color`] of the requested mode.
//!
//! Multi-stop input is interpreted as evenly-spaced positions in `[0, 1]`,
//! exactly as culori does when no explicit positions are given. Powerless
//! channels (e.g. an achromatic hue) carry `NaN` through fixup; the lerp
//! propagates the defined endpoint to a missing one, so interpolating from
//! grey to red in HSL produces red's hue at every `t > 0`.
//!
//! Hue fixup is selected via [`HueFixup`] and applied to every hue channel
//! present in the target space. Per-channel and global easing functions are
//! configured through [`InterpolateOptions`]; the fallback closure
//! ([`interpolate`]) uses [`HueFixup::Shorter`] (CSS Color Module 4 default)
//! and no easing.

use std::collections::HashMap;

use crate::convert::convert;
use crate::spaces::{Hsl, Hsv, Hwb, Lab, Lch, LinearRgb, Oklab, Oklch, Rgb, Xyz50, Xyz65};
use crate::traits::ColorSpace;
use crate::Color;

mod hue_fixup;
mod lerp;

pub use hue_fixup::HueFixup;

use hue_fixup::fixup_alpha;
use lerp::linear_interpolator;

/// Configuration for [`interpolate_with`]. Defaults mirror culori: hue
/// fixup is `Shorter`, no easing. Use the type's builder-style methods to
/// override individual fields.
pub struct InterpolateOptions {
    /// Strategy for hue cycling on cylindrical spaces.
    pub hue_fixup: HueFixup,
    /// Global easing function applied to every channel before sampling.
    pub easing: Option<Box<dyn Fn(f64) -> f64 + Send + Sync>>,
    /// Per-channel easing functions, keyed by channel name (e.g. `"l"`,
    /// `"h"`). When set for a channel, overrides the global easing for that
    /// channel. Alpha is keyed as `"alpha"`.
    pub channel_easings: HashMap<&'static str, Box<dyn Fn(f64) -> f64 + Send + Sync>>,
}

impl std::fmt::Debug for InterpolateOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InterpolateOptions")
            .field("hue_fixup", &self.hue_fixup)
            .field("easing", &self.easing.as_ref().map(|_| "<fn>"))
            .field(
                "channel_easings",
                &self
                    .channel_easings
                    .keys()
                    .copied()
                    .collect::<Vec<&'static str>>(),
            )
            .finish()
    }
}

impl Default for InterpolateOptions {
    fn default() -> Self {
        Self {
            hue_fixup: HueFixup::Shorter,
            easing: None,
            channel_easings: HashMap::new(),
        }
    }
}

impl InterpolateOptions {
    /// New options with defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the hue fixup strategy.
    pub fn hue_fixup(mut self, strategy: HueFixup) -> Self {
        self.hue_fixup = strategy;
        self
    }

    /// Set a global easing function.
    pub fn easing<F>(mut self, easing: F) -> Self
    where
        F: Fn(f64) -> f64 + Send + Sync + 'static,
    {
        self.easing = Some(Box::new(easing));
        self
    }

    /// Set a per-channel easing function.
    pub fn channel_easing<F>(mut self, channel: &'static str, easing: F) -> Self
    where
        F: Fn(f64) -> f64 + Send + Sync + 'static,
    {
        self.channel_easings.insert(channel, Box::new(easing));
        self
    }
}

/// Build an interpolator for `colors` in `mode`. Returns a closure that
/// maps `t` (clamped to `[0, 1]`) to a [`Color`] of the requested mode.
///
/// `mode` is one of culori's mode strings: `"rgb"`, `"hsl"`, `"hsv"`,
/// `"hwb"`, `"lab"`, `"lch"`, `"oklab"`, `"oklch"`, `"lrgb"`, `"xyz50"`,
/// `"xyz65"`. An unrecognized mode panics — call sites should validate
/// mode strings up front.
///
/// Currently supports the v0.1 modes: rgb, lrgb, hsl, hsv, hwb, lab, lch,
/// oklab, oklch, xyz50, xyz65. Other modes (cubehelix, dlab/dlch, jab/jch,
/// luv/lchuv, hsluv/hpluv, okhsl/okhsv, itp, xyb, yiq, hsi, p3, rec2020,
/// a98, prophoto) are not yet supported by `interpolate` or `average` —
/// passing them will panic. Wider support is planned for a future release.
///
/// Uses [`HueFixup::Shorter`] and no easing. For other strategies, see
/// [`interpolate_with`].
///
/// # Panics
///
/// Panics if `colors` is empty or `mode` is unknown.
pub fn interpolate(colors: &[Color], mode: &str) -> Box<dyn Fn(f64) -> Color + Send + Sync> {
    interpolate_with(colors, mode, InterpolateOptions::default())
}

/// Build an interpolator with explicit options.
///
/// # Panics
///
/// Panics if `colors` is empty or `mode` is unknown.
pub fn interpolate_with(
    colors: &[Color],
    mode: &str,
    options: InterpolateOptions,
) -> Box<dyn Fn(f64) -> Color + Send + Sync> {
    assert!(
        !colors.is_empty(),
        "interpolate: at least one color is required"
    );
    let info = mode_info(mode);

    // Convert each color to the target mode and extract per-channel values.
    let mut channels: Vec<Vec<f64>> = vec![Vec::with_capacity(colors.len()); info.channels.len()];
    let mut alphas: Vec<f64> = Vec::with_capacity(colors.len());
    for color in colors {
        let (chs, a) = decompose(*color, mode);
        for (i, v) in chs.iter().enumerate() {
            channels[i].push(*v);
        }
        alphas.push(a);
    }

    // Per-channel fixup: hue channels go through HueFixup; alpha gets
    // culori's "any defined → fill missing with 1" rule; everything else
    // passes through.
    let fixed: Vec<Vec<f64>> = info
        .channels
        .iter()
        .enumerate()
        .map(|(i, ch)| {
            if ch.is_hue {
                hue_fixup::apply(&channels[i], options.hue_fixup)
            } else {
                channels[i].clone()
            }
        })
        .collect();
    let fixed_alpha = fixup_alpha(&alphas);

    // Snapshot the first and last stops (pre-fixup). culori short-circuits
    // `t <= positions[0]` / `t > positions[n]` to return these literal
    // colors, preserving NaN hues at the boundaries.
    let first_channels: Vec<f64> = channels.iter().map(|c| c[0]).collect();
    let first_alpha = alphas[0];
    let last_idx = colors.len() - 1;
    let last_channels: Vec<f64> = channels.iter().map(|c| c[last_idx]).collect();
    let last_alpha = alphas[last_idx];

    // Build per-channel piecewise linear interpolators.
    let interps: Vec<Box<dyn Fn(f64) -> f64 + Send + Sync>> = fixed
        .into_iter()
        .map(|stops| {
            let f = linear_interpolator(stops);
            Box::new(f) as Box<dyn Fn(f64) -> f64 + Send + Sync>
        })
        .collect();
    let alpha_interp = linear_interpolator(fixed_alpha);

    let channel_names: Vec<&'static str> = info.channels.iter().map(|c| c.name).collect();
    let mode_owned = info.mode_str;
    let easing = options.easing;
    let channel_easings = options.channel_easings;

    Box::new(move |t: f64| {
        let t = t.clamp(0.0, 1.0);
        if t <= 0.0 {
            let alpha = nan_to_option(first_alpha);
            return compose(mode_owned, &first_channels, alpha);
        }
        if t >= 1.0 {
            let alpha = nan_to_option(last_alpha);
            return compose(mode_owned, &last_channels, alpha);
        }
        let mut out_channels = Vec::with_capacity(channel_names.len());
        for (i, name) in channel_names.iter().enumerate() {
            let local_t = ease(t, name, &easing, &channel_easings);
            out_channels.push(interps[i](local_t));
        }
        let alpha_t = ease(t, "alpha", &easing, &channel_easings);
        let alpha_val = alpha_interp(alpha_t);
        compose(mode_owned, &out_channels, nan_to_option(alpha_val))
    })
}

fn ease(
    t: f64,
    channel: &str,
    global: &Option<Box<dyn Fn(f64) -> f64 + Send + Sync>>,
    per_channel: &HashMap<&'static str, Box<dyn Fn(f64) -> f64 + Send + Sync>>,
) -> f64 {
    if let Some(f) = per_channel.get(channel) {
        return f(t);
    }
    if let Some(f) = global {
        return f(t);
    }
    t
}

#[derive(Debug, Clone, Copy)]
struct ChannelInfo {
    name: &'static str,
    is_hue: bool,
}

#[derive(Debug, Clone, Copy)]
struct ModeInfo {
    mode_str: &'static str,
    channels: &'static [ChannelInfo],
}

const RGB_CHANNELS: &[ChannelInfo] = &[
    ChannelInfo {
        name: "r",
        is_hue: false,
    },
    ChannelInfo {
        name: "g",
        is_hue: false,
    },
    ChannelInfo {
        name: "b",
        is_hue: false,
    },
];

const LRGB_CHANNELS: &[ChannelInfo] = RGB_CHANNELS;

const HSL_CHANNELS: &[ChannelInfo] = &[
    ChannelInfo {
        name: "h",
        is_hue: true,
    },
    ChannelInfo {
        name: "s",
        is_hue: false,
    },
    ChannelInfo {
        name: "l",
        is_hue: false,
    },
];

const HSV_CHANNELS: &[ChannelInfo] = &[
    ChannelInfo {
        name: "h",
        is_hue: true,
    },
    ChannelInfo {
        name: "s",
        is_hue: false,
    },
    ChannelInfo {
        name: "v",
        is_hue: false,
    },
];

const HWB_CHANNELS: &[ChannelInfo] = &[
    ChannelInfo {
        name: "h",
        is_hue: true,
    },
    ChannelInfo {
        name: "w",
        is_hue: false,
    },
    ChannelInfo {
        name: "b",
        is_hue: false,
    },
];

const LAB_CHANNELS: &[ChannelInfo] = &[
    ChannelInfo {
        name: "l",
        is_hue: false,
    },
    ChannelInfo {
        name: "a",
        is_hue: false,
    },
    ChannelInfo {
        name: "b",
        is_hue: false,
    },
];

const LCH_CHANNELS: &[ChannelInfo] = &[
    ChannelInfo {
        name: "l",
        is_hue: false,
    },
    ChannelInfo {
        name: "c",
        is_hue: false,
    },
    ChannelInfo {
        name: "h",
        is_hue: true,
    },
];

const XYZ_CHANNELS: &[ChannelInfo] = &[
    ChannelInfo {
        name: "x",
        is_hue: false,
    },
    ChannelInfo {
        name: "y",
        is_hue: false,
    },
    ChannelInfo {
        name: "z",
        is_hue: false,
    },
];

fn mode_info(mode: &str) -> ModeInfo {
    match mode {
        "rgb" => ModeInfo {
            mode_str: "rgb",
            channels: RGB_CHANNELS,
        },
        "lrgb" => ModeInfo {
            mode_str: "lrgb",
            channels: LRGB_CHANNELS,
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
        other => panic!("interpolate: unknown mode '{other}'"),
    }
}

fn alpha_to_f64(a: Option<f64>) -> f64 {
    a.unwrap_or(f64::NAN)
}

fn nan_to_option(v: f64) -> Option<f64> {
    if v.is_nan() {
        None
    } else {
        Some(v)
    }
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
