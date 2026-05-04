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
use crate::spaces::{
    Cubehelix, Dlab, Dlch, Hpluv, Hsi, Hsl, Hsluv, Hsv, Hwb, Itp, Jab, Jch, Lab, Lab65, Lch, Lch65,
    Lchuv, LinearRgb, Luv, Okhsl, Okhsv, Oklab, Oklch, Prismatic, ProphotoRgb, Rec2020, Rgb, Xyb,
    Xyz50, Xyz65, Yiq, A98, P3,
};
use crate::traits::ColorSpace;
use crate::Color;

mod hue_fixup;
mod lerp;
mod normalize_positions;
mod piecewise;
mod spline;

pub use hue_fixup::{
    fixup_alpha, fixup_hue_decreasing, fixup_hue_increasing, fixup_hue_longer, fixup_hue_shorter,
    HueFixup,
};
pub use normalize_positions::normalize_positions;
pub use piecewise::interpolator_piecewise;
pub use spline::{
    interpolator_spline_basis, interpolator_spline_basis_closed, interpolator_spline_monotone,
    interpolator_spline_monotone_2, interpolator_spline_monotone_closed,
    interpolator_spline_natural, interpolator_spline_natural_closed, ChannelInterp,
    ChannelInterpFactory,
};

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
    /// Per-channel interpolator factories. When set for a channel, replaces
    /// the default linear-piecewise interpolator with a custom one (e.g.
    /// one of the `interpolator_spline_*` factories). Alpha is keyed as
    /// `"alpha"`. Channels without an entry fall back to the linear
    /// interpolator that mirrors culori's default.
    pub channel_interpolators: HashMap<&'static str, ChannelInterpFactory>,
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
            .field(
                "channel_interpolators",
                &self
                    .channel_interpolators
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
            channel_interpolators: HashMap::new(),
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

    /// Set a per-channel interpolator factory. The factory builds a sampler
    /// from the channel's stop slice. Use one of the
    /// `interpolator_spline_*` constructors to swap the default linear
    /// sampler for a spline.
    pub fn channel_interpolator(
        mut self,
        channel: &'static str,
        factory: ChannelInterpFactory,
    ) -> Self {
        self.channel_interpolators.insert(channel, factory);
        self
    }
}

/// Build an interpolator for `colors` in `mode`. Returns a closure that
/// maps `t` (clamped to `[0, 1]`) to a [`Color`] of the requested mode.
///
/// An unrecognized mode panics — call sites should validate mode strings
/// up front.
///
/// Supported modes: `rgb`, `lrgb`, `hsl`, `hsv`, `hwb`, `lab`, `lab65`,
/// `lch`, `lch65`, `oklab`, `oklch`, `xyz50`, `xyz65`, `p3`, `rec2020`,
/// `a98`, `prophoto`, `cubehelix`, `dlab`, `dlch`, `jab`, `jch`, `yiq`,
/// `hsi`, `hsluv`, `hpluv`, `okhsl`, `okhsv`, `itp`, `xyb`, `luv`,
/// `lchuv`, `prismatic`. `prismatic` is the only four-channel mode;
/// every other mode is three channels. `hsluv`, `hpluv`, and `prismatic`
/// are culors extensions and not present in culori 4.0.2.
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

/// Build an interpolator that premultiplies each input by its alpha,
/// interpolates, then divides the result back out.
///
/// Mirrors culori 4.0.2's `interpolateWithPremultipliedAlpha` (defined in
/// `interpolate/interpolate.js` as `interpolateWith(mapAlphaMultiply,
/// mapAlphaDivide)`). Premultiplication is the right default for blending
/// semi-transparent colors: a 50/50 mix of opaque red and fully transparent
/// blue stays red, since the transparent blue contributes no color energy.
///
/// # Panics
///
/// Panics if `colors` is empty or `mode` is unknown.
pub fn interpolate_with_premultiplied_alpha(
    colors: &[Color],
    mode: &str,
    options: InterpolateOptions,
) -> Box<dyn Fn(f64) -> Color + Send + Sync> {
    use crate::map::{map_alpha_divide, map_alpha_multiply, mapper};

    assert!(
        !colors.is_empty(),
        "interpolate_with_premultiplied_alpha: at least one color is required"
    );
    let mode_static = mode_static_str(mode);

    // Pre-multiply each input color's non-alpha channels by its alpha.
    let pre = mapper(map_alpha_multiply(), mode_static, false);
    let premapped: Vec<Color> = colors.iter().map(&pre).collect();

    // Build the inner interpolator on the premultiplied stops.
    let inner = interpolate_with(&premapped, mode, options);

    // culori's `interpolate_fn` short-circuits `t <= positions[0]` and
    // `t > positions[n]` to return the *unmapped* boundary colors; the
    // post-divide then divides those originals by their alpha. To mirror
    // that, keep the original first / last colors and route the boundary
    // values through `post` directly.
    let first = pre_converted(colors[0], mode_static);
    let last = pre_converted(colors[colors.len() - 1], mode_static);
    let post = mapper(map_alpha_divide(), mode_static, false);
    Box::new(move |t: f64| {
        let t_clamped = t.clamp(0.0, 1.0);
        if t_clamped <= 0.0 {
            return post(&first);
        }
        if t_clamped >= 1.0 {
            return post(&last);
        }
        post(&inner(t_clamped))
    })
}

// Convert a color into the working mode without any premapping. Used for
// the boundary short-circuit in `interpolate_with_premultiplied_alpha`.
fn pre_converted(color: Color, mode: &'static str) -> Color {
    use crate::map::mapper;
    // mapper with an identity function performs the mode conversion only.
    let identity = |v: f64, _ch: &str, _c: &Color| v;
    mapper(identity, mode, false)(&color)
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
    let n_channels = info.channels.len();
    let mut channels: Vec<Vec<f64>> = vec![Vec::with_capacity(colors.len()); n_channels];
    let mut alphas: Vec<f64> = Vec::with_capacity(colors.len());
    for color in colors {
        let (chs, a) = decompose(*color, mode);
        for (i, ch) in channels.iter_mut().enumerate().take(n_channels) {
            ch.push(chs[i]);
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

    let channel_names: Vec<&'static str> = info.channels.iter().map(|c| c.name).collect();

    // Build per-channel interpolators. Default to linear; allow per-channel
    // override via `options.channel_interpolators`.
    let interps: Vec<Box<dyn Fn(f64) -> f64 + Send + Sync>> = fixed
        .into_iter()
        .enumerate()
        .map(|(i, stops)| {
            let name = channel_names[i];
            if let Some(factory) = options.channel_interpolators.get(name) {
                factory(&stops)
            } else {
                Box::new(linear_interpolator(stops)) as Box<dyn Fn(f64) -> f64 + Send + Sync>
            }
        })
        .collect();
    let alpha_interp: Box<dyn Fn(f64) -> f64 + Send + Sync> =
        if let Some(factory) = options.channel_interpolators.get("alpha") {
            factory(&fixed_alpha)
        } else {
            Box::new(linear_interpolator(fixed_alpha))
        };

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

const HSI_CHANNELS: &[ChannelInfo] = &[
    ChannelInfo {
        name: "h",
        is_hue: true,
    },
    ChannelInfo {
        name: "s",
        is_hue: false,
    },
    ChannelInfo {
        name: "i",
        is_hue: false,
    },
];

const JAB_CHANNELS: &[ChannelInfo] = &[
    ChannelInfo {
        name: "j",
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

const JCH_CHANNELS: &[ChannelInfo] = &[
    ChannelInfo {
        name: "j",
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

const YIQ_CHANNELS: &[ChannelInfo] = &[
    ChannelInfo {
        name: "y",
        is_hue: false,
    },
    ChannelInfo {
        name: "i",
        is_hue: false,
    },
    ChannelInfo {
        name: "q",
        is_hue: false,
    },
];

const ITP_CHANNELS: &[ChannelInfo] = &[
    ChannelInfo {
        name: "i",
        is_hue: false,
    },
    ChannelInfo {
        name: "t",
        is_hue: false,
    },
    ChannelInfo {
        name: "p",
        is_hue: false,
    },
];

const XYB_CHANNELS: &[ChannelInfo] = &[
    ChannelInfo {
        name: "x",
        is_hue: false,
    },
    ChannelInfo {
        name: "y",
        is_hue: false,
    },
    ChannelInfo {
        name: "b",
        is_hue: false,
    },
];

const LUV_CHANNELS: &[ChannelInfo] = &[
    ChannelInfo {
        name: "l",
        is_hue: false,
    },
    ChannelInfo {
        name: "u",
        is_hue: false,
    },
    ChannelInfo {
        name: "v",
        is_hue: false,
    },
];

const PRISMATIC_CHANNELS: &[ChannelInfo] = &[
    ChannelInfo {
        name: "l",
        is_hue: false,
    },
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

fn mode_static_str(mode: &str) -> &'static str {
    mode_info(mode).mode_str
}

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
            channels: JAB_CHANNELS,
        },
        "jch" => ModeInfo {
            mode_str: "jch",
            channels: JCH_CHANNELS,
        },
        "yiq" => ModeInfo {
            mode_str: "yiq",
            channels: YIQ_CHANNELS,
        },
        "hsi" => ModeInfo {
            mode_str: "hsi",
            channels: HSI_CHANNELS,
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
            channels: ITP_CHANNELS,
        },
        "xyb" => ModeInfo {
            mode_str: "xyb",
            channels: XYB_CHANNELS,
        },
        "luv" => ModeInfo {
            mode_str: "luv",
            channels: LUV_CHANNELS,
        },
        "lchuv" => ModeInfo {
            mode_str: "lchuv",
            channels: LCH_CHANNELS,
        },
        "lab65" => ModeInfo {
            mode_str: "lab65",
            channels: LAB_CHANNELS,
        },
        "lch65" => ModeInfo {
            mode_str: "lch65",
            channels: LCH_CHANNELS,
        },
        "prismatic" => ModeInfo {
            mode_str: "prismatic",
            channels: PRISMATIC_CHANNELS,
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

fn decompose(c: Color, mode: &str) -> ([f64; 4], f64) {
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
            ([v.r, v.g, v.b, 0.0], alpha_to_f64(v.alpha))
        }
        "lrgb" => {
            let v: LinearRgb = match c {
                Color::Rgb(x) => x.into(),
                Color::LinearRgb(x) => x,
                other => convert::<Xyz65, LinearRgb>(color_to_xyz65(other)),
            };
            ([v.r, v.g, v.b, 0.0], alpha_to_f64(v.alpha))
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
            ([v.h, v.s, v.l, 0.0], alpha_to_f64(v.alpha))
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
            ([v.h, v.s, v.v, 0.0], alpha_to_f64(v.alpha))
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
            ([v.h, v.w, v.b, 0.0], alpha_to_f64(v.alpha))
        }
        "lab" => {
            let v: Lab = match c {
                Color::Lab(x) => x,
                Color::Lch(x) => x.into(),
                Color::Xyz50(x) => x.into(),
                Color::Rgb(x) => x.into(),
                other => convert::<Xyz65, Lab>(color_to_xyz65(other)),
            };
            ([v.l, v.a, v.b, 0.0], alpha_to_f64(v.alpha))
        }
        "lch" => {
            let v: Lch = match c {
                Color::Lch(x) => x,
                Color::Lab(x) => x.into(),
                Color::Rgb(x) => x.into(),
                other => convert::<Xyz65, Lab>(color_to_xyz65(other)).into(),
            };
            ([v.l, v.c, v.h, 0.0], alpha_to_f64(v.alpha))
        }
        "oklab" => {
            let v: Oklab = match c {
                Color::Oklab(x) => x,
                Color::Oklch(x) => x.into(),
                Color::Rgb(x) => x.into(),
                Color::LinearRgb(x) => x.into(),
                other => convert::<Xyz65, Oklab>(color_to_xyz65(other)),
            };
            ([v.l, v.a, v.b, 0.0], alpha_to_f64(v.alpha))
        }
        "oklch" => {
            let v: Oklch = match c {
                Color::Oklch(x) => x,
                Color::Oklab(x) => x.into(),
                Color::Rgb(x) => x.into(),
                Color::LinearRgb(x) => Oklab::from(x).into(),
                other => convert::<Xyz65, Oklab>(color_to_xyz65(other)).into(),
            };
            ([v.l, v.c, v.h, 0.0], alpha_to_f64(v.alpha))
        }
        "xyz50" => {
            let v: Xyz50 = match c {
                Color::Xyz50(x) => x,
                Color::Lab(x) => x.into(),
                other => convert::<Xyz65, Xyz50>(color_to_xyz65(other)),
            };
            ([v.x, v.y, v.z, 0.0], alpha_to_f64(v.alpha))
        }
        "xyz65" => {
            let v: Xyz65 = color_to_xyz65(c);
            ([v.x, v.y, v.z, 0.0], alpha_to_f64(v.alpha))
        }
        "p3" => {
            let v: P3 = match c {
                Color::P3(x) => x,
                other => convert::<Xyz65, P3>(color_to_xyz65(other)),
            };
            ([v.r, v.g, v.b, 0.0], alpha_to_f64(v.alpha))
        }
        "rec2020" => {
            let v: Rec2020 = match c {
                Color::Rec2020(x) => x,
                other => convert::<Xyz65, Rec2020>(color_to_xyz65(other)),
            };
            ([v.r, v.g, v.b, 0.0], alpha_to_f64(v.alpha))
        }
        "a98" => {
            let v: A98 = match c {
                Color::A98(x) => x,
                other => convert::<Xyz65, A98>(color_to_xyz65(other)),
            };
            ([v.r, v.g, v.b, 0.0], alpha_to_f64(v.alpha))
        }
        "prophoto" => {
            let v: ProphotoRgb = match c {
                Color::ProphotoRgb(x) => x,
                other => convert::<Xyz65, ProphotoRgb>(color_to_xyz65(other)),
            };
            ([v.r, v.g, v.b, 0.0], alpha_to_f64(v.alpha))
        }
        "cubehelix" => {
            let v: Cubehelix = match c {
                Color::Cubehelix(x) => x,
                Color::Rgb(x) => x.into(),
                other => convert::<Xyz65, Rgb>(color_to_xyz65(other)).into(),
            };
            ([v.h, v.s, v.l, 0.0], alpha_to_f64(v.alpha))
        }
        "dlab" => {
            let v: Dlab = match c {
                Color::Dlab(x) => x,
                Color::Rgb(x) => x.into(),
                other => convert::<Xyz65, Rgb>(color_to_xyz65(other)).into(),
            };
            ([v.l, v.a, v.b, 0.0], alpha_to_f64(v.alpha))
        }
        "dlch" => {
            let v: Dlch = match c {
                Color::Dlch(x) => x,
                Color::Rgb(x) => x.into(),
                other => convert::<Xyz65, Rgb>(color_to_xyz65(other)).into(),
            };
            ([v.l, v.c, v.h, 0.0], alpha_to_f64(v.alpha))
        }
        "jab" => {
            let v: Jab = match c {
                Color::Jab(x) => x,
                Color::Jch(x) => x.into(),
                Color::Rgb(x) => x.into(),
                other => convert::<Xyz65, Rgb>(color_to_xyz65(other)).into(),
            };
            ([v.j, v.a, v.b, 0.0], alpha_to_f64(v.alpha))
        }
        "jch" => {
            let v: Jch = match c {
                Color::Jch(x) => x,
                Color::Jab(x) => x.into(),
                Color::Rgb(x) => x.into(),
                other => convert::<Xyz65, Rgb>(color_to_xyz65(other)).into(),
            };
            ([v.j, v.c, v.h, 0.0], alpha_to_f64(v.alpha))
        }
        "yiq" => {
            let v: Yiq = match c {
                Color::Yiq(x) => x,
                Color::Rgb(x) => x.into(),
                other => convert::<Xyz65, Rgb>(color_to_xyz65(other)).into(),
            };
            ([v.y, v.i, v.q, 0.0], alpha_to_f64(v.alpha))
        }
        "hsi" => {
            let v: Hsi = match c {
                Color::Hsi(x) => x,
                Color::Rgb(x) => x.into(),
                other => convert::<Xyz65, Rgb>(color_to_xyz65(other)).into(),
            };
            ([v.h, v.s, v.i, 0.0], alpha_to_f64(v.alpha))
        }
        "hsluv" => {
            let v: Hsluv = match c {
                Color::Hsluv(x) => x,
                Color::Rgb(x) => x.into(),
                other => convert::<Xyz65, Rgb>(color_to_xyz65(other)).into(),
            };
            ([v.h, v.s, v.l, 0.0], alpha_to_f64(v.alpha))
        }
        "hpluv" => {
            let v: Hpluv = match c {
                Color::Hpluv(x) => x,
                Color::Rgb(x) => x.into(),
                other => convert::<Xyz65, Rgb>(color_to_xyz65(other)).into(),
            };
            ([v.h, v.s, v.l, 0.0], alpha_to_f64(v.alpha))
        }
        "okhsl" => {
            let v: Okhsl = match c {
                Color::Okhsl(x) => x,
                Color::Rgb(x) => x.into(),
                Color::Oklab(x) => x.into(),
                other => convert::<Xyz65, Rgb>(color_to_xyz65(other)).into(),
            };
            ([v.h, v.s, v.l, 0.0], alpha_to_f64(v.alpha))
        }
        "okhsv" => {
            let v: Okhsv = match c {
                Color::Okhsv(x) => x,
                Color::Rgb(x) => x.into(),
                Color::Oklab(x) => x.into(),
                other => convert::<Xyz65, Rgb>(color_to_xyz65(other)).into(),
            };
            ([v.h, v.s, v.v, 0.0], alpha_to_f64(v.alpha))
        }
        "itp" => {
            let v: Itp = match c {
                Color::Itp(x) => x,
                other => convert::<Xyz65, Itp>(color_to_xyz65(other)),
            };
            ([v.i, v.t, v.p, 0.0], alpha_to_f64(v.alpha))
        }
        "xyb" => {
            let v: Xyb = match c {
                Color::Xyb(x) => x,
                Color::Rgb(x) => x.into(),
                other => convert::<Xyz65, Rgb>(color_to_xyz65(other)).into(),
            };
            ([v.x, v.y, v.b, 0.0], alpha_to_f64(v.alpha))
        }
        "luv" => {
            let v: Luv = match c {
                Color::Luv(x) => x,
                Color::Lchuv(x) => x.into(),
                Color::Rgb(x) => x.into(),
                other => convert::<Xyz65, Luv>(color_to_xyz65(other)),
            };
            ([v.l, v.u, v.v, 0.0], alpha_to_f64(v.alpha))
        }
        "lchuv" => {
            let v: Lchuv = match c {
                Color::Lchuv(x) => x,
                Color::Luv(x) => x.into(),
                Color::Rgb(x) => x.into(),
                other => convert::<Xyz65, Luv>(color_to_xyz65(other)).into(),
            };
            ([v.l, v.c, v.h, 0.0], alpha_to_f64(v.alpha))
        }
        "lab65" => {
            let v: Lab65 = match c {
                Color::Lab65(x) => x,
                Color::Lch65(x) => x.into(),
                Color::Rgb(x) => x.into(),
                other => convert::<Xyz65, Lab65>(color_to_xyz65(other)),
            };
            ([v.l, v.a, v.b, 0.0], alpha_to_f64(v.alpha))
        }
        "lch65" => {
            let v: Lch65 = match c {
                Color::Lch65(x) => x,
                Color::Lab65(x) => x.into(),
                Color::Rgb(x) => x.into(),
                other => convert::<Xyz65, Lab65>(color_to_xyz65(other)).into(),
            };
            ([v.l, v.c, v.h, 0.0], alpha_to_f64(v.alpha))
        }
        "prismatic" => {
            let v: Prismatic = match c {
                Color::Prismatic(x) => x,
                Color::Rgb(x) => x.into(),
                other => convert::<Xyz65, Rgb>(color_to_xyz65(other)).into(),
            };
            ([v.l, v.r, v.g, v.b], alpha_to_f64(v.alpha))
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
        "lab65" => Color::Lab65(Lab65 {
            l: channels[0],
            a: channels[1],
            b: channels[2],
            alpha,
        }),
        "lch65" => Color::Lch65(Lch65 {
            l: channels[0],
            c: channels[1],
            h: channels[2],
            alpha,
        }),
        "prismatic" => Color::Prismatic(Prismatic {
            l: channels[0],
            r: channels[1],
            g: channels[2],
            b: channels[3],
            alpha,
        }),
        _ => unreachable!("mode_info already validated"),
    }
}
