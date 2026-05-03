//! Internal helpers shared by every `difference_*` factory: convert a
//! [`Color`] into the channel triple of a target mode, and convert into
//! D65 Lab (which culori's CIE76 / CIE94 / CIEDE2000 / CMC all sit on top
//! of, via `lab65`).

use crate::convert::convert;
use crate::spaces::{
    Hsl, Hsv, Hwb, Lab, Lab65, Lch, Lch65, LinearRgb, Oklab, Oklch, Rgb, Xyz50, Xyz65, Yiq,
};
use crate::traits::ColorSpace;
use crate::Color;

/// Channel descriptor for a mode. `is_hue` flags the hue channel so the
/// generic euclidean reducer can apply the polar-difference operator
/// configured by the mode (matching culori's `def.difference[k]`).
#[derive(Clone, Copy)]
pub(crate) struct ChannelInfo {
    pub(crate) is_hue: bool,
}

/// Mode metadata: ordered channels and the polar-distance operator to
/// apply on the hue channel (or `None` for non-cylindrical modes).
#[derive(Clone, Copy)]
pub(crate) struct ModeShape {
    pub(crate) channels: [ChannelInfo; 3],
    /// Polar-distance operator: produces the per-pair `delta` for the
    /// hue channel. Mirrors culori's `def.difference.h`. `None` falls back
    /// to plain numeric subtraction (useful for HWB which uses
    /// `differenceHueNaive`, but we currently treat HWB hue as plain).
    pub(crate) hue_diff: Option<HueDiffKind>,
}

/// The three flavours of polar hue distance culori defines: LCh-style
/// (`differenceHueChroma`), HSx-style (`differenceHueSaturation`), and
/// the naive signed wrap (`differenceHueNaive`).
#[derive(Clone, Copy)]
pub(crate) enum HueDiffKind {
    /// `2 * sqrt(c1 * c2) * sin((((h2 - h1 + 360) / 2) * π) / 180)`
    Chroma,
    /// `2 * sqrt(s1 * s2) * sin((((h2 - h1 + 360) / 2) * π) / 180)`
    Saturation,
    /// Signed wrap distance: at most 180 degrees, signed.
    Naive,
}

const fn linear() -> [ChannelInfo; 3] {
    [
        ChannelInfo { is_hue: false },
        ChannelInfo { is_hue: false },
        ChannelInfo { is_hue: false },
    ]
}

/// Channels are ordered (h, s, l) / (h, s, v) / (h, w, b). The hue is the
/// first channel.
const fn hsl_like() -> [ChannelInfo; 3] {
    [
        ChannelInfo { is_hue: true },
        ChannelInfo { is_hue: false },
        ChannelInfo { is_hue: false },
    ]
}

/// LCh-like channels are ordered (l, c, h). The hue is the third channel.
const fn lch_like() -> [ChannelInfo; 3] {
    [
        ChannelInfo { is_hue: false },
        ChannelInfo { is_hue: false },
        ChannelInfo { is_hue: true },
    ]
}

pub(crate) fn mode_shape(mode: &str) -> ModeShape {
    match mode {
        "rgb" | "lrgb" | "lab" | "lab65" | "oklab" | "xyz50" | "xyz65" | "jab" | "itp" | "yiq" => {
            ModeShape {
                channels: linear(),
                hue_diff: None,
            }
        }
        "lch" | "lch65" | "oklch" => ModeShape {
            channels: lch_like(),
            hue_diff: Some(HueDiffKind::Chroma),
        },
        "hsl" | "hsv" => ModeShape {
            channels: hsl_like(),
            hue_diff: Some(HueDiffKind::Saturation),
        },
        "hwb" => ModeShape {
            channels: hsl_like(),
            hue_diff: Some(HueDiffKind::Naive),
        },
        other => panic!("difference: unknown mode '{other}'"),
    }
}

/// Convert `c` into the three channels of `mode`, in the channel order
/// of [`mode_shape`].
pub(crate) fn extract(c: Color, mode: &str) -> [f64; 3] {
    match mode {
        "rgb" => {
            let v: Rgb = match c {
                Color::Rgb(x) => x,
                Color::LinearRgb(x) => x.into(),
                Color::Hsl(x) => x.into(),
                Color::Hsv(x) => x.into(),
                Color::Hwb(x) => Hsv::from(x).into(),
                other => convert::<Xyz65, Rgb>(to_xyz65(other)),
            };
            [v.r, v.g, v.b]
        }
        "lrgb" => {
            let v: LinearRgb = match c {
                Color::Rgb(x) => x.into(),
                Color::LinearRgb(x) => x,
                other => convert::<Xyz65, LinearRgb>(to_xyz65(other)),
            };
            [v.r, v.g, v.b]
        }
        "hsl" => {
            let v: Hsl = match c {
                Color::Hsl(x) => x,
                Color::Rgb(x) => x.into(),
                Color::LinearRgb(x) => Rgb::from(x).into(),
                Color::Hsv(x) => Rgb::from(x).into(),
                Color::Hwb(x) => Rgb::from(Hsv::from(x)).into(),
                other => convert::<Xyz65, Rgb>(to_xyz65(other)).into(),
            };
            [v.h, v.s, v.l]
        }
        "hsv" => {
            let v: Hsv = match c {
                Color::Hsv(x) => x,
                Color::Hwb(x) => x.into(),
                Color::Rgb(x) => x.into(),
                Color::LinearRgb(x) => Rgb::from(x).into(),
                Color::Hsl(x) => Rgb::from(x).into(),
                other => convert::<Xyz65, Rgb>(to_xyz65(other)).into(),
            };
            [v.h, v.s, v.v]
        }
        "hwb" => {
            let v: Hwb = match c {
                Color::Hwb(x) => x,
                Color::Hsv(x) => x.into(),
                Color::Rgb(x) => Hsv::from(x).into(),
                Color::LinearRgb(x) => Hsv::from(Rgb::from(x)).into(),
                Color::Hsl(x) => Hsv::from(Rgb::from(x)).into(),
                other => Hsv::from(convert::<Xyz65, Rgb>(to_xyz65(other))).into(),
            };
            [v.h, v.w, v.b]
        }
        "lab" => {
            let v: Lab = match c {
                Color::Lab(x) => x,
                Color::Lch(x) => x.into(),
                Color::Xyz50(x) => x.into(),
                Color::Rgb(x) => x.into(),
                other => convert::<Xyz65, Lab>(to_xyz65(other)),
            };
            [v.l, v.a, v.b]
        }
        "lab65" => {
            // D65 Lab — culori's `lab65` mode. Direct path for Lab65 and
            // Rgb (with the achromatic snap); other modes route through
            // xyz65 to match the generic hub.
            let v: Lab65 = match c {
                Color::Lab65(x) => x,
                Color::Lch65(x) => x.into(),
                Color::Rgb(x) => x.into(),
                other => Lab65::from(to_xyz65(other)),
            };
            [v.l, v.a, v.b]
        }
        "lch" => {
            let v: Lch = match c {
                Color::Lch(x) => x,
                Color::Lab(x) => x.into(),
                Color::Rgb(x) => x.into(),
                other => convert::<Xyz65, Lab>(to_xyz65(other)).into(),
            };
            [v.l, v.c, v.h]
        }
        "lch65" => {
            let v: Lch65 = match c {
                Color::Lch65(x) => x,
                Color::Lab65(x) => x.into(),
                Color::Rgb(x) => x.into(),
                other => Lab65::from(to_xyz65(other)).into(),
            };
            [v.l, v.c, v.h]
        }
        "oklab" => {
            let v: Oklab = match c {
                Color::Oklab(x) => x,
                Color::Oklch(x) => x.into(),
                Color::Rgb(x) => x.into(),
                Color::LinearRgb(x) => x.into(),
                other => convert::<Xyz65, Oklab>(to_xyz65(other)),
            };
            [v.l, v.a, v.b]
        }
        "oklch" => {
            let v: Oklch = match c {
                Color::Oklch(x) => x,
                Color::Oklab(x) => x.into(),
                Color::Rgb(x) => x.into(),
                Color::LinearRgb(x) => Oklab::from(x).into(),
                other => convert::<Xyz65, Oklab>(to_xyz65(other)).into(),
            };
            [v.l, v.c, v.h]
        }
        "xyz50" => {
            let v: Xyz50 = match c {
                Color::Xyz50(x) => x,
                Color::Lab(x) => x.into(),
                other => convert::<Xyz65, Xyz50>(to_xyz65(other)),
            };
            [v.x, v.y, v.z]
        }
        "xyz65" => {
            let v: Xyz65 = to_xyz65(c);
            [v.x, v.y, v.z]
        }
        "jab" => {
            let v: crate::spaces::Jab = match c {
                Color::Jab(x) => x,
                Color::Rgb(x) => x.into(),
                other => convert::<Xyz65, crate::spaces::Jab>(to_xyz65(other)),
            };
            [v.j, v.a, v.b]
        }
        "itp" => {
            let v: crate::spaces::Itp = convert::<Xyz65, crate::spaces::Itp>(to_xyz65(c));
            [v.i, v.t, v.p]
        }
        "yiq" => {
            let v: Yiq = match c {
                Color::Yiq(x) => x,
                Color::Rgb(x) => x.into(),
                Color::LinearRgb(x) => Rgb::from(x).into(),
                other => convert::<Xyz65, Rgb>(to_xyz65(other)).into(),
            };
            [v.y, v.i, v.q]
        }
        other => panic!("difference: unknown mode '{other}'"),
    }
}

/// Convert any [`Color`] into a triple of D65 Lab `(l, a, b)`. The
/// underlying transfer is culori's `xyz65 → lab65` (`lab65/convertXyz65ToLab65.js`).
pub(crate) fn to_lab65(c: Color) -> (f64, f64, f64) {
    let xyz = to_xyz65(c);
    xyz65_to_lab65(xyz.x, xyz.y, xyz.z)
}

/// Hub conversion to XYZ D65 used by every other extractor.
pub(crate) fn to_xyz65(c: Color) -> Xyz65 {
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

// D65 white, copied verbatim from culori's `constants.js`:
//   D65 = { X: 0.3127 / 0.329, Y: 1, Z: (1 - 0.3127 - 0.329) / 0.329 }
const D65_X: f64 = 0.3127 / 0.329;
const D65_Y: f64 = 1.0;
const D65_Z: f64 = (1.0 - 0.3127 - 0.329) / 0.329;
// `k = 29^3 / 3^3`, `e = 6^3 / 29^3` from `xyz65/constants.js`.
const K: f64 = 24389.0 / 27.0;
const E: f64 = 216.0 / 24389.0;

#[inline]
fn f_forward(value: f64) -> f64 {
    if value > E {
        value.cbrt()
    } else {
        (K * value + 16.0) / 116.0
    }
}

fn xyz65_to_lab65(x: f64, y: f64, z: f64) -> (f64, f64, f64) {
    let f0 = f_forward(x / D65_X);
    let f1 = f_forward(y / D65_Y);
    let f2 = f_forward(z / D65_Z);
    let l = 116.0 * f1 - 16.0;
    let a = 500.0 * (f0 - f1);
    let b = 200.0 * (f1 - f2);
    (l, a, b)
}

/// culori's `util/normalizeHue.js`: `(hue % 360 + 360) % 360`.
#[inline]
pub(crate) fn normalize_hue(h: f64) -> f64 {
    let h = h % 360.0;
    if h < 0.0 {
        h + 360.0
    } else {
        h
    }
}
