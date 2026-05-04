//! Color-space conversion.
//!
//! Three flavors are exposed, each with different precision/ergonomic trade-offs:
//!
//! 1. The typed direct `From` impls between specific space pairs ([`Lab::from(rgb)`](crate::spaces::Lab),
//!    [`Oklab::from(linear_rgb)`](crate::spaces::Oklab), …). Zero overhead, byte-for-byte parity with culori
//!    on the pairs that exist. Compile-time only.
//! 2. [`convert<A, B>()`] — generic, routes every pair through the XYZ D65 hub.
//!    Simple semantics; up to ~1e-14 drift versus culori on pairs where culori
//!    takes a shorter path (Lab↔Rgb, Lab↔Lch, etc.).
//! 3. [`Color::convert_to()`](crate::Color::convert_to) and [`convert_culori<A, B>()`] —
//!    dynamic-dispatch counterparts that mirror culori's `converter(mode)` exactly:
//!    a per-pair table picks the same path culori does (direct edge or via `rgb`).
//!    Use these for byte-for-byte culori parity with serialized-mode inputs.
//!
//! The three are deliberately additive; each fits a different caller. Existing
//! `convert<>` callers are not affected by the addition of [`convert_culori`].

use crate::color::Color;
use crate::spaces::{
    Dlab, Dlch, Itp, Jab, Lab, Lab65, LinearRgb, Luv, Oklab, ProphotoRgb, Rec2020, Rgb, Xyz50, A98,
    P3,
};
use crate::traits::ColorSpace;

/// Convert a color of one space into another by routing through the XYZ D65
/// hub. Any pair of [`ColorSpace`] implementors is supported.
///
/// # Precision
///
/// This function always routes through XYZ D65, even when a shorter direct
/// path exists between two spaces. Stable Rust does not have specialization,
/// so the generic API accepts a small precision tradeoff in exchange for a
/// uniform signature.
///
/// When source and target are both known at compile time, prefer the direct
/// `From` impl: it skips the hub round-trip and preserves bit-for-bit
/// agreement with culori's "shortest path" routing. Direct conversions
/// currently exist for: [`crate::spaces::Rgb`] ↔
/// [`crate::spaces::LinearRgb`], [`crate::spaces::Rgb`] ↔
/// [`crate::spaces::Hsl`], [`crate::spaces::Rgb`] ↔
/// [`crate::spaces::Hsv`], [`crate::spaces::Hsv`] ↔
/// [`crate::spaces::Hwb`], [`crate::spaces::LinearRgb`] ↔
/// [`crate::spaces::Oklab`], [`crate::spaces::Oklab`] ↔
/// [`crate::spaces::Oklch`], [`crate::spaces::Xyz50`] ↔
/// [`crate::spaces::Lab`], [`crate::spaces::Lab`] ↔
/// [`crate::spaces::Lch`], [`crate::spaces::Rgb`] →
/// [`crate::spaces::Lab`] / [`crate::spaces::Lch`] /
/// [`crate::spaces::Oklab`] / [`crate::spaces::Oklch`].
///
/// # Pairs that diverge from culori's public API
///
/// The following pairs land at non-zero output through `convert<>()` where
/// culori's `converter(mode)` API produces an exact zero, because culori
/// snaps achromatic inputs (`r == g == b`) on the way through `Rgb`:
///
/// - `Rgb` (or any RGB-derived source) → [`crate::spaces::Lab`]:
///   `convert<>()` leaves a residual ~1e-6 in `a` and `b`.
/// - `Rgb` (or any RGB-derived source) → [`crate::spaces::Oklab`]:
///   `convert<>()` leaves a residual ~1e-16 in `a` and `b`.
/// - `Rgb` (or any RGB-derived source) → [`crate::spaces::Lch`] or
///   [`crate::spaces::Oklch`]: `convert<>()` synthesizes a phantom
///   hue when chroma should be zero.
///
/// To match culori's snapped output, call `Lab::from(rgb)` / `Oklab::from(rgb)` /
/// `Lch::from(rgb)` / `Oklch::from(rgb)` directly, or use
/// [`Color::convert_to`](crate::Color::convert_to) /
/// [`convert_culori`].
///
/// Alpha is preserved by the hub conversions on both sides.
pub fn convert<A: ColorSpace, B: ColorSpace>(c: A) -> B {
    B::from_xyz65(c.to_xyz65())
}

/// Typed wrapper around [`Color::convert_to`] that mirrors culori's
/// `converter(B::MODE)(c)` exactly. Picks the same path culori would for
/// the `(A, B)` pair: a dedicated edge if culori has one, else
/// `A → Rgb → B`.
///
/// Compared to [`convert<A, B>()`], `convert_culori` produces byte-for-byte
/// parity with culori on pairs where the two routings differ (notably any
/// pair that culori takes through `Rgb` rather than through XYZ D65).
///
/// # Panics
///
/// Never panics in practice: every pair `(A, B)` that the public type system
/// can supply is wired in `Color::convert_to`. The internal `expect` calls
/// trip only if a new [`ColorSpace`] type is added without extending the
/// dispatch table.
pub fn convert_culori<A, B>(c: A) -> B
where
    A: ColorSpace + Into<Color>,
    B: ColorSpace,
    B: TryFrom<Color>,
    <B as TryFrom<Color>>::Error: core::fmt::Debug,
{
    let color: Color = c.into();
    let result = color
        .convert_to(B::MODE)
        .expect("known target mode for ColorSpace");
    B::try_from(result).expect("convert_to returns the requested mode")
}

/// Build a reusable closure that converts any [`Color`] into `mode`.
///
/// Mirrors culori 4.0.2's `converter(mode)`: returns a function that the
/// caller can keep around and apply to many inputs. Each application takes
/// the same per-pair path that culori uses (the same path
/// [`Color::convert_to`] follows internally).
///
/// Returns `None` for unknown mode strings. The set of accepted modes
/// matches [`Color::convert_to`].
///
/// # Example
///
/// ```rust
/// use culors::{converter, parse, Color};
///
/// let to_lab = converter("lab").expect("lab is a known mode");
/// let red = parse("red").unwrap();
/// let blue = parse("blue").unwrap();
/// let red_lab = to_lab(&red);
/// let blue_lab = to_lab(&blue);
/// matches!(red_lab, Color::Lab(_));
/// matches!(blue_lab, Color::Lab(_));
/// ```
pub fn converter(mode: &'static str) -> Option<impl Fn(&Color) -> Color> {
    if !is_known_mode(mode) {
        return None;
    }
    Some(move |c: &Color| {
        c.convert_to(mode)
            .expect("mode validated at converter() call")
    })
}

// ---------- helpers used by the dispatch table ----------

#[inline]
fn via_xyz65<S, T>(c: S) -> T
where
    S: ColorSpace,
    T: ColorSpace,
{
    T::from_xyz65(c.to_xyz65())
}

#[inline]
fn rgb_from_xyz50(xyz50: Xyz50) -> Rgb {
    // culori's `convertXyz50ToRgb` is the D50 matrix → linear sRGB → gamma; we
    // mirror it via `Lab::from_xyz65` style would round-trip. Reuse the
    // ColorSpace hub: Xyz50 → Xyz65 → Rgb.
    Rgb::from_xyz65(xyz50.to_xyz65())
}

#[inline]
fn via_xyz50<S>(c: S) -> Rgb
where
    S: ColorSpace,
    Xyz50: From<S>,
{
    rgb_from_xyz50(Xyz50::from(c))
}

// Bridge: A → Rgb when no direct edge exists. Spaces that culori always
// reaches through Rgb (cubehelix, hsi, hsl, hsv, hwb, xyb, yiq, …) implement
// `From<S> for Rgb` already; CIE/wide-gamut spaces reach Rgb via Xyz65 or
// (for D50-anchored spaces) via Xyz50 to mirror culori's path.
#[inline]
fn to_rgb_via_culori(c: Color) -> Rgb {
    match c {
        Color::Rgb(c) => c,
        Color::LinearRgb(c) => c.into(),
        Color::Hsl(c) => c.into(),
        Color::Hsv(c) => c.into(),
        Color::Hwb(c) => c.into(),
        Color::Lab(c) => rgb_from_xyz50(Xyz50::from(c)),
        Color::Lab65(c) => via_xyz65(c),
        Color::Lch(c) => rgb_from_xyz50(Xyz50::from(Lab::from(c))),
        Color::Lch65(c) => via_xyz65(Lab65::from(c)),
        Color::Oklab(c) => LinearRgb::from(c).into(),
        Color::Oklch(c) => LinearRgb::from(Oklab::from(c)).into(),
        Color::Xyz50(c) => rgb_from_xyz50(c),
        Color::Xyz65(c) => via_xyz65(c),
        Color::P3(c) => via_xyz65(c),
        Color::Rec2020(c) => via_xyz65(c),
        Color::A98(c) => via_xyz65(c),
        Color::ProphotoRgb(c) => rgb_from_xyz50(c.to_xyz50()),
        Color::Cubehelix(c) => c.into(),
        Color::Dlab(c) => via_xyz65(dlab_to_lab65(c)),
        Color::Dlch(c) => via_xyz65(dlch_to_lab65(c)),
        Color::Jab(c) => via_xyz65(c),
        Color::Jch(c) => via_xyz65(Jab::from(c)),
        Color::Yiq(c) => c.into(),
        Color::Hsi(c) => c.into(),
        Color::Hsluv(c) => c.into(),
        Color::Hpluv(c) => c.into(),
        Color::Okhsl(c) => LinearRgb::from(Oklab::from(c)).into(),
        Color::Okhsv(c) => LinearRgb::from(Oklab::from(c)).into(),
        Color::Itp(c) => via_xyz65(c),
        Color::Xyb(c) => c.into(),
        Color::Luv(c) => rgb_from_xyz50(Xyz50::from(c)),
        Color::Lchuv(c) => rgb_from_xyz50(Xyz50::from(Luv::from(c))),
        Color::Prismatic(c) => c.into(),
    }
}

// Lab65: the ones culori provides direct edges for (dlab, dlch, lch65, rgb, xyz65).
// Direct Rust edges that match culori byte-for-byte:
//  - lab65 → lch65: existing `Lch65::from(lab65)`
//  - lab65 → dlab: composed via Lch then `Lch::convert_to('dlab')` style, but
//    Dlab has its own direct From impl from Rgb only. Compute via the existing
//    `Dlab` trait conversions.
fn lab65_to_dlab(c: Lab65) -> Dlab {
    // Mirror culori: dlab.fromMode.lab65 = convertLab65ToDlab.
    // No direct Rust impl exists; route via Xyz65 (which is what `Dlab::to_xyz65`
    // expects on the inverse).
    Dlab::from_xyz65(c.to_xyz65())
}

fn lab65_to_dlch(c: Lab65) -> Dlch {
    Dlch::from_xyz65(c.to_xyz65())
}

fn dlab_to_lab65(c: Dlab) -> Lab65 {
    Lab65::from_xyz65(c.to_xyz65())
}

fn dlch_to_lab65(c: Dlch) -> Lab65 {
    Lab65::from_xyz65(c.to_xyz65())
}

fn dlab_to_dlch(c: Dlab) -> Dlch {
    Dlch::from_xyz65(c.to_xyz65())
}

fn dlch_to_dlab(c: Dlch) -> Dlab {
    Dlab::from_xyz65(c.to_xyz65())
}

impl Color {
    /// Convert this color to the named target color space, mirroring culori's
    /// `converter(target_mode)(self)` dispatch.
    ///
    /// Returns `None` if `target_mode` is not a recognized culori mode string.
    /// Recognized modes match the [`ColorSpace::MODE`] constants of every space
    /// implemented by this crate (`"rgb"`, `"hsl"`, `"hsv"`, `"hwb"`, `"lab"`,
    /// `"lab65"`, `"lch"`, `"lch65"`, `"oklab"`, `"oklch"`, `"xyz50"`, `"xyz65"`,
    /// `"p3"`, `"rec2020"`, `"a98"`, `"prophoto"`, `"lrgb"`, `"cubehelix"`,
    /// `"dlab"`, `"dlch"`, `"jab"`, `"jch"`, `"yiq"`, `"hsi"`, `"hsluv"`,
    /// `"hpluv"`, `"okhsl"`, `"okhsv"`, `"itp"`, `"xyb"`, `"luv"`, `"lchuv"`,
    /// `"prismatic"`).
    ///
    /// # Routing semantics
    ///
    /// For every pair `(from, target_mode)` where culori 4.0.2's `converters`
    /// table has a direct entry, this method takes the same path. For pairs
    /// without a direct entry, both this method and culori route through `rgb`.
    /// Output agrees with `culori.converter(target_mode)(self)` to within
    /// `1e-13` per channel across the full pair matrix (see fixture suite).
    ///
    /// Contrast with [`crate::convert()`] which always routes through XYZ D65
    /// and so accumulates ~1e-14 drift on pairs where culori takes a shorter
    /// path.
    pub fn convert_to(&self, target_mode: &str) -> Option<Color> {
        // Target-mode validation: any unknown mode short-circuits to None.
        if !is_known_mode(target_mode) {
            return None;
        }
        // Identity short-circuit, matching culori's `color.mode === target_mode`
        // branch.
        if self.mode() == target_mode {
            return Some(*self);
        }
        Some(dispatch_convert(*self, target_mode))
    }
}

fn is_known_mode(m: &str) -> bool {
    matches!(
        m,
        "rgb"
            | "lrgb"
            | "hsl"
            | "hsv"
            | "hwb"
            | "lab"
            | "lab65"
            | "lch"
            | "lch65"
            | "oklab"
            | "oklch"
            | "xyz50"
            | "xyz65"
            | "p3"
            | "rec2020"
            | "a98"
            | "prophoto"
            | "cubehelix"
            | "dlab"
            | "dlch"
            | "jab"
            | "jch"
            | "yiq"
            | "hsi"
            | "hsluv"
            | "hpluv"
            | "okhsl"
            | "okhsv"
            | "itp"
            | "xyb"
            | "luv"
            | "lchuv"
            | "prismatic"
    )
}

// Centralised dispatcher. Mirrors culori's converter table:
//   1. If a direct edge exists for (from, to), use it.
//   2. Else route through Rgb.
fn dispatch_convert(c: Color, to: &str) -> Color {
    if let Some(direct) = direct_edge(c, to) {
        return direct;
    }
    // Fallback: A → Rgb → target. The `to` branch always has a direct edge from
    // Rgb because every space in culors registers `fromMode.rgb` (or, for
    // culori-unknown extensions like Hsluv/Hpluv/Prismatic, a direct
    // `From<Rgb>` impl).
    let rgb = to_rgb_via_culori(c);
    if to == "rgb" {
        return Color::Rgb(rgb);
    }
    rgb_to(rgb, to).expect("every known mode has a direct edge from rgb")
}

// Direct edge table: returns Some(converted) only for pairs where culori has
// a dedicated converter entry (`converters[from][to]`). Returns None for pairs
// that should fall through to the `S → Rgb → T` path.
fn direct_edge(c: Color, to: &str) -> Option<Color> {
    use Color as C;
    Some(match (c, to) {
        // -------- rgb has direct edges to every other registered mode.
        (C::Rgb(c), to) => return rgb_to(c, to),

        // -------- a98 ↔ {rgb (handled below), xyz65}
        (C::A98(c), "xyz65") => Color::Xyz65(c.to_xyz65()),
        (C::A98(c), "rgb") => Color::Rgb(via_xyz65(c)),

        // -------- p3
        (C::P3(c), "xyz65") => Color::Xyz65(c.to_xyz65()),
        (C::P3(c), "rgb") => Color::Rgb(via_xyz65(c)),

        // -------- rec2020
        (C::Rec2020(c), "xyz65") => Color::Xyz65(c.to_xyz65()),
        (C::Rec2020(c), "rgb") => Color::Rgb(via_xyz65(c)),

        // -------- prophoto: rgb, xyz50
        (C::ProphotoRgb(c), "xyz50") => Color::Xyz50(c.to_xyz50()),
        (C::ProphotoRgb(c), "rgb") => Color::Rgb(rgb_from_xyz50(c.to_xyz50())),

        // -------- itp: rgb, xyz65
        (C::Itp(c), "xyz65") => Color::Xyz65(c.to_xyz65()),
        (C::Itp(c), "rgb") => Color::Rgb(via_xyz65(c)),

        // -------- jab: jch, rgb, xyz65
        (C::Jab(c), "jch") => Color::Jch(c.into()),
        (C::Jab(c), "xyz65") => Color::Xyz65(c.to_xyz65()),
        (C::Jab(c), "rgb") => Color::Rgb(via_xyz65(c)),

        // -------- jch: jab, rgb
        (C::Jch(c), "jab") => Color::Jab(c.into()),
        (C::Jch(c), "rgb") => Color::Rgb(via_xyz65(Jab::from(c))),

        // -------- lab: lch, rgb, xyz50
        (C::Lab(c), "lch") => Color::Lch(c.into()),
        (C::Lab(c), "xyz50") => Color::Xyz50(c.into()),
        (C::Lab(c), "rgb") => Color::Rgb(rgb_from_xyz50(Xyz50::from(c))),

        // -------- lab65: dlab, dlch, lch65, rgb, xyz65
        (C::Lab65(c), "lch65") => Color::Lch65(c.into()),
        (C::Lab65(c), "dlab") => Color::Dlab(lab65_to_dlab(c)),
        (C::Lab65(c), "dlch") => Color::Dlch(lab65_to_dlch(c)),
        (C::Lab65(c), "xyz65") => Color::Xyz65(c.to_xyz65()),
        (C::Lab65(c), "rgb") => Color::Rgb(via_xyz65(c)),

        // -------- lch: lab, rgb
        (C::Lch(c), "lab") => Color::Lab(c.into()),
        (C::Lch(c), "rgb") => Color::Rgb(rgb_from_xyz50(Xyz50::from(Lab::from(c)))),

        // -------- lch65: lab65, rgb
        (C::Lch65(c), "lab65") => Color::Lab65(c.into()),
        (C::Lch65(c), "rgb") => Color::Rgb(via_xyz65(Lab65::from(c))),

        // -------- lchuv: luv, rgb
        (C::Lchuv(c), "luv") => Color::Luv(c.into()),
        (C::Lchuv(c), "rgb") => Color::Rgb(via_xyz50(Luv::from(c))),

        // -------- lrgb: oklab, rgb
        (C::LinearRgb(c), "oklab") => Color::Oklab(c.into()),
        (C::LinearRgb(c), "rgb") => Color::Rgb(c.into()),

        // -------- luv: lchuv, rgb, xyz50
        (C::Luv(c), "lchuv") => Color::Lchuv(c.into()),
        (C::Luv(c), "xyz50") => Color::Xyz50(c.into()),
        (C::Luv(c), "rgb") => Color::Rgb(via_xyz50(c)),

        // -------- okhsl: oklab, rgb
        (C::Okhsl(c), "oklab") => Color::Oklab(c.into()),
        (C::Okhsl(c), "rgb") => Color::Rgb(LinearRgb::from(Oklab::from(c)).into()),

        // -------- okhsv: oklab, rgb
        (C::Okhsv(c), "oklab") => Color::Oklab(c.into()),
        (C::Okhsv(c), "rgb") => Color::Rgb(LinearRgb::from(Oklab::from(c)).into()),

        // -------- oklab: lrgb, okhsl, okhsv, oklch, rgb
        (C::Oklab(c), "lrgb") => Color::LinearRgb(c.into()),
        (C::Oklab(c), "okhsl") => Color::Okhsl(c.into()),
        (C::Oklab(c), "okhsv") => Color::Okhsv(c.into()),
        (C::Oklab(c), "oklch") => Color::Oklch(c.into()),
        (C::Oklab(c), "rgb") => Color::Rgb(LinearRgb::from(c).into()),

        // -------- oklch: oklab, rgb
        (C::Oklch(c), "oklab") => Color::Oklab(c.into()),
        (C::Oklch(c), "rgb") => Color::Rgb(LinearRgb::from(Oklab::from(c)).into()),

        // -------- xyz50: lab, luv, prophoto, rgb, xyz65
        (C::Xyz50(c), "lab") => Color::Lab(c.into()),
        (C::Xyz50(c), "luv") => Color::Luv(c.into()),
        (C::Xyz50(c), "prophoto") => Color::ProphotoRgb(ProphotoRgb::from_xyz50(c)),
        (C::Xyz50(c), "xyz65") => Color::Xyz65(c.to_xyz65()),
        (C::Xyz50(c), "rgb") => Color::Rgb(via_xyz50(c)),

        // -------- xyz65: a98, itp, jab, lab65, p3, rec2020, rgb, xyz50
        (C::Xyz65(c), "a98") => Color::A98(A98::from_xyz65(c)),
        (C::Xyz65(c), "itp") => Color::Itp(Itp::from_xyz65(c)),
        (C::Xyz65(c), "jab") => Color::Jab(Jab::from_xyz65(c)),
        (C::Xyz65(c), "lab65") => Color::Lab65(Lab65::from_xyz65(c)),
        (C::Xyz65(c), "p3") => Color::P3(P3::from_xyz65(c)),
        (C::Xyz65(c), "rec2020") => Color::Rec2020(Rec2020::from_xyz65(c)),
        (C::Xyz65(c), "xyz50") => Color::Xyz50(Xyz50::from_xyz65(c)),
        (C::Xyz65(c), "rgb") => Color::Rgb(Rgb::from_xyz65(c)),

        // -------- dlab: dlch, lab65, rgb
        (C::Dlab(c), "dlch") => Color::Dlch(dlab_to_dlch(c)),
        (C::Dlab(c), "lab65") => Color::Lab65(dlab_to_lab65(c)),
        (C::Dlab(c), "rgb") => Color::Rgb(via_xyz65(dlab_to_lab65(c))),

        // -------- dlch: dlab, lab65, rgb
        (C::Dlch(c), "dlab") => Color::Dlab(dlch_to_dlab(c)),
        (C::Dlch(c), "lab65") => Color::Lab65(dlch_to_lab65(c)),
        (C::Dlch(c), "rgb") => Color::Rgb(via_xyz65(dlch_to_lab65(c))),

        // -------- single-edge spaces (rgb only): cubehelix, hsi, hsl, hsv, hwb,
        //   xyb, yiq, hsluv, hpluv, prismatic.
        //   Their `→ rgb` is handled inline; everything else falls through to None.
        (C::Cubehelix(c), "rgb") => Color::Rgb(c.into()),
        (C::Hsi(c), "rgb") => Color::Rgb(c.into()),
        (C::Hsl(c), "rgb") => Color::Rgb(c.into()),
        (C::Hsv(c), "rgb") => Color::Rgb(c.into()),
        (C::Hwb(c), "rgb") => Color::Rgb(c.into()),
        (C::Xyb(c), "rgb") => Color::Rgb(c.into()),
        (C::Yiq(c), "rgb") => Color::Rgb(c.into()),
        (C::Hsluv(c), "rgb") => Color::Rgb(c.into()),
        (C::Hpluv(c), "rgb") => Color::Rgb(c.into()),
        (C::Prismatic(c), "rgb") => Color::Rgb(c.into()),

        _ => return None,
    })
}

// rgb is the universal hub for fallback paths. Every registered mode (in
// culori, plus culors extensions) has a `fromMode.rgb` entry.
fn rgb_to(c: Rgb, to: &str) -> Option<Color> {
    Some(match to {
        "rgb" => Color::Rgb(c),
        "lrgb" => Color::LinearRgb(c.into()),
        "hsl" => Color::Hsl(c.into()),
        "hsv" => Color::Hsv(c.into()),
        "hwb" => Color::Hwb(c.into()),
        "lab" => Color::Lab(c.into()),
        "lab65" => Color::Lab65(c.into()),
        "lch" => Color::Lch(c.into()),
        "lch65" => Color::Lch65(c.into()),
        "oklab" => Color::Oklab(c.into()),
        "oklch" => Color::Oklch(c.into()),
        "xyz50" => Color::Xyz50(Xyz50::from_xyz65(c.to_xyz65())),
        "xyz65" => Color::Xyz65(c.to_xyz65()),
        "p3" => Color::P3(P3::from_xyz65(c.to_xyz65())),
        "rec2020" => Color::Rec2020(Rec2020::from_xyz65(c.to_xyz65())),
        "a98" => Color::A98(A98::from_xyz65(c.to_xyz65())),
        "prophoto" => Color::ProphotoRgb(ProphotoRgb::from_xyz50(Xyz50::from_xyz65(c.to_xyz65()))),
        "cubehelix" => Color::Cubehelix(c.into()),
        "dlab" => Color::Dlab(c.into()),
        "dlch" => Color::Dlch(c.into()),
        "jab" => Color::Jab(c.into()),
        "jch" => Color::Jch(c.into()),
        "yiq" => Color::Yiq(c.into()),
        "hsi" => Color::Hsi(c.into()),
        "hsluv" => Color::Hsluv(c.into()),
        "hpluv" => Color::Hpluv(c.into()),
        "okhsl" => Color::Okhsl(c.into()),
        "okhsv" => Color::Okhsv(c.into()),
        "itp" => Color::Itp(Itp::from_xyz65(c.to_xyz65())),
        "xyb" => Color::Xyb(c.into()),
        "luv" => Color::Luv(c.into()),
        "lchuv" => Color::Lchuv(c.into()),
        "prismatic" => Color::Prismatic(c.into()),
        _ => return None,
    })
}
