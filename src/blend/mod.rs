//! CSS Compositing & Blending.
//!
//! The public entry point [`blend`] takes a slice of [`Color`]s and a
//! [`BlendMode`], converts each to sRGB, and folds them left-to-right with
//! Porter-Duff source-over compositing. Separable modes apply a per-channel
//! function from CSS Compositing 1 § 5.7; non-separable modes (`Hue`,
//! `Saturation`, `Color`, `Luminosity`) operate on whole RGB triples per
//! § 5.8. The output is a [`Color::Rgb`] with channels clipped to `[0, 1]`.
//!
//! The separable modes mirror culori 4.0.2's `src/blend.js`. The four
//! non-separable modes are spec-direct ports — culori 4.0.2 does not
//! implement them, so output for those modes is not culori-compatible.
//!
//! # Example
//!
//! ```rust
//! use culors::{blend, parse, BlendMode, Color};
//!
//! let red = parse("red").unwrap();
//! let white = parse("white").unwrap();
//! let result = blend(&[red, white], BlendMode::Multiply);
//! match result {
//!     Color::Rgb(c) => {
//!         assert!((c.r - 1.0).abs() < 1e-12);
//!         assert!((c.g - 0.0).abs() < 1e-12);
//!         assert!((c.b - 0.0).abs() < 1e-12);
//!     }
//!     _ => unreachable!("blend always returns Color::Rgb"),
//! }
//! ```

mod modes;
mod non_separable;

use crate::convert::convert;
use crate::spaces::{Hsv, Rgb, Xyz65};
use crate::traits::ColorSpace;
use crate::Color;
use non_separable::Triple;

/// CSS Compositing 1 blend modes — twelve separable from § 5.7 plus the
/// four non-separable ones from § 5.8.
///
/// The non-separable modes (`Hue`, `Saturation`, `Color`, `Luminosity`)
/// are spec-direct, not culori-compatible, because culori 4.0.2 does not
/// implement them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlendMode {
    /// `B(b, s) = s` — source replaces backdrop.
    Normal,
    /// `B(b, s) = b * s`.
    Multiply,
    /// `B(b, s) = b + s - b*s`.
    Screen,
    /// Hard-light blend mode.
    ///
    /// Culori's formula: `if s < 0.5 { 2*s*b } else { 2*s*(1-b) - 1 }`.
    ///
    /// (CSS Compositing 1 § 5.7.4 expresses the formula equivalently using a
    /// screen branch; culori's algebraic form is identical for valid inputs.)
    HardLight,
    /// Overlay blend mode.
    ///
    /// Culori's formula: `if b < 0.5 { s*2*b } else { 2*b*(1-s) - 1 }`.
    ///
    /// CSS Compositing 1 § 5.7.3 defines overlay as `hard-light(s, b)` —
    /// the same formula but with the branch driven by `s`, not `b`. Culori 4.0.2
    /// deviates from the spec; this implementation mirrors culori for parity.
    Overlay,
    /// `B(b, s) = min(b, s)`.
    Darken,
    /// `B(b, s) = max(b, s)`.
    Lighten,
    /// Color-dodge with culori's edge-case handling for `b==0` and `s==1`.
    ColorDodge,
    /// Color-burn with culori's edge-case handling for `b==1` and `s==0`.
    ColorBurn,
    /// Soft-light per culori's piecewise formula.
    SoftLight,
    /// `B(b, s) = |b - s|`.
    Difference,
    /// `B(b, s) = b + s - 2*b*s`.
    Exclusion,
    /// Non-separable. `B(b, s) = SetLum(SetSat(s, Sat(b)), Lum(b))` — the
    /// source's hue with the backdrop's saturation and luminance.
    /// CSS Compositing 1 § 5.8.1.
    Hue,
    /// Non-separable. `B(b, s) = SetLum(SetSat(b, Sat(s)), Lum(b))` — the
    /// backdrop's hue and luminance with the source's saturation.
    /// CSS Compositing 1 § 5.8.2.
    Saturation,
    /// Non-separable. `B(b, s) = SetLum(s, Lum(b))` — the source's hue and
    /// saturation with the backdrop's luminance. Useful for tinting.
    /// CSS Compositing 1 § 5.8.3.
    Color,
    /// Non-separable. `B(b, s) = SetLum(b, Lum(s))` — the backdrop's hue
    /// and saturation with the source's luminance.
    /// CSS Compositing 1 § 5.8.4.
    Luminosity,
}

impl BlendMode {
    fn is_non_separable(self) -> bool {
        matches!(
            self,
            BlendMode::Hue | BlendMode::Saturation | BlendMode::Color | BlendMode::Luminosity
        )
    }

    fn apply(self, b: f64, s: f64) -> f64 {
        match self {
            BlendMode::Normal => modes::normal(b, s),
            BlendMode::Multiply => modes::multiply(b, s),
            BlendMode::Screen => modes::screen(b, s),
            BlendMode::HardLight => modes::hard_light(b, s),
            BlendMode::Overlay => modes::overlay(b, s),
            BlendMode::Darken => modes::darken(b, s),
            BlendMode::Lighten => modes::lighten(b, s),
            BlendMode::ColorDodge => modes::color_dodge(b, s),
            BlendMode::ColorBurn => modes::color_burn(b, s),
            BlendMode::SoftLight => modes::soft_light(b, s),
            BlendMode::Difference => modes::difference(b, s),
            BlendMode::Exclusion => modes::exclusion(b, s),
            // Non-separable modes use `apply_triple` instead. Reaching here
            // is a programming error.
            BlendMode::Hue | BlendMode::Saturation | BlendMode::Color | BlendMode::Luminosity => {
                unreachable!("non-separable mode dispatched to scalar apply")
            }
        }
    }

    fn apply_triple(self, b: Triple, s: Triple) -> Triple {
        match self {
            BlendMode::Hue => non_separable::hue(b, s),
            BlendMode::Saturation => non_separable::saturation(b, s),
            BlendMode::Color => non_separable::color(b, s),
            BlendMode::Luminosity => non_separable::luminosity(b, s),
            _ => unreachable!("separable mode dispatched to triple apply"),
        }
    }

    /// Parse a CSS keyword (e.g. `"multiply"`, `"color-dodge"`) into a
    /// [`BlendMode`]. Returns `None` for unknown names.
    pub fn from_css_name(name: &str) -> Option<Self> {
        Some(match name {
            "normal" => BlendMode::Normal,
            "multiply" => BlendMode::Multiply,
            "screen" => BlendMode::Screen,
            "hard-light" => BlendMode::HardLight,
            "overlay" => BlendMode::Overlay,
            "darken" => BlendMode::Darken,
            "lighten" => BlendMode::Lighten,
            "color-dodge" => BlendMode::ColorDodge,
            "color-burn" => BlendMode::ColorBurn,
            "soft-light" => BlendMode::SoftLight,
            "difference" => BlendMode::Difference,
            "exclusion" => BlendMode::Exclusion,
            "hue" => BlendMode::Hue,
            "saturation" => BlendMode::Saturation,
            "color" => BlendMode::Color,
            "luminosity" => BlendMode::Luminosity,
            _ => return None,
        })
    }
}

/// Blend a stack of colors using the given separable blend mode.
///
/// Each color is converted to sRGB; missing alphas default to `1`. The
/// stack is folded left-to-right: the first color is the initial backdrop,
/// each subsequent color is the source applied on top. The output is a
/// [`Color::Rgb`] with alpha set, channels clipped to `[0, 1]`.
///
/// # Panics
///
/// Panics if `colors` is empty, matching culori's `reduce`-with-no-initial
/// `TypeError` behaviour. Single-element input is a valid no-op that
/// returns the input converted to sRGB.
pub fn blend(colors: &[Color], mode: BlendMode) -> Color {
    assert!(!colors.is_empty(), "blend: at least one color is required");

    let mut iter = colors.iter().map(|c| to_rgb_with_alpha(*c));
    let mut acc = iter.next().expect("non-empty checked above");
    for src in iter {
        acc = porter_duff(acc, src, mode);
    }

    Color::Rgb(Rgb {
        r: acc.r,
        g: acc.g,
        b: acc.b,
        alpha: Some(acc.a),
    })
}

/// String-keyed convenience wrapper around [`blend`]. Accepts the same CSS
/// keywords as [`BlendMode::from_css_name`] (e.g. `"multiply"`,
/// `"color-dodge"`). Returns `None` for unknown modes.
///
/// # Panics
///
/// Panics if `colors` is empty.
pub fn blend_str(colors: &[Color], mode: &str) -> Option<Color> {
    BlendMode::from_css_name(mode).map(|m| blend(colors, m))
}

#[derive(Clone, Copy)]
struct Rgba {
    r: f64,
    g: f64,
    b: f64,
    a: f64,
}

fn to_rgb_with_alpha(c: Color) -> Rgba {
    let rgb: Rgb = match c {
        Color::Rgb(x) => x,
        Color::LinearRgb(x) => x.into(),
        Color::Hsl(x) => x.into(),
        Color::Hsv(x) => x.into(),
        Color::Hwb(x) => Hsv::from(x).into(),
        other => convert::<Xyz65, Rgb>(color_to_xyz65(other)),
    };
    Rgba {
        r: rgb.r,
        g: rgb.g,
        b: rgb.b,
        a: rgb.alpha.unwrap_or(1.0),
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

fn porter_duff(b: Rgba, s: Rgba, mode: BlendMode) -> Rgba {
    let alpha = s.a + b.a * (1.0 - s.a);
    if alpha == 0.0 {
        return Rgba {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        };
    }
    // Compute the per-channel blend output `f`. Separable modes apply the
    // mode function to each channel independently; non-separable modes
    // operate on the whole triple at once.
    let (fr, fg, fb) = if mode.is_non_separable() {
        let triple = mode.apply_triple(
            Triple {
                r: b.r,
                g: b.g,
                b: b.b,
            },
            Triple {
                r: s.r,
                g: s.g,
                b: s.b,
            },
        );
        (triple.r, triple.g, triple.b)
    } else {
        (
            mode.apply(b.r, s.r),
            mode.apply(b.g, s.g),
            mode.apply(b.b, s.b),
        )
    };
    let combine = |bc: f64, sc: f64, f: f64| -> f64 {
        let v = s.a * (1.0 - b.a) * sc + s.a * b.a * f + (1.0 - s.a) * b.a * bc;
        (v / alpha).clamp(0.0, 1.0)
    };
    Rgba {
        r: combine(b.r, s.r, fr),
        g: combine(b.g, s.g, fg),
        b: combine(b.b, s.b, fb),
        a: alpha,
    }
}
