//! A Rust port of [culori](https://github.com/evercoder/culori), the
//! JavaScript color library by Dan Burzo.
//!
//! culors implements 30+ color spaces, conversion between any pair of
//! them, a CSS Color Module 4 string parser and matching formatter,
//! interpolation, gamut mapping, ΔE, separable blend modes, mode-aware
//! averaging, WCAG contrast, and the CSS filter set. Output values
//! agree with culori 4.0.2 within 1e-10 across an exhaustive fixture
//! set (110 conversion pairs, 365 parse cases, 303 format
//! round-trips).
//!
//! # Quick start
//!
//! Parse two CSS strings, blend them, and format the result back:
//!
//! ```rust
//! use culors::{blend, format_css, parse, BlendMode};
//!
//! let red = parse("#ff0000").unwrap();
//! let blue = parse("rgb(0 0 255 / 0.5)").unwrap();
//! let mixed = blend(&[red, blue], BlendMode::Multiply);
//! let css = format_css(&mixed);
//! assert!(css.starts_with("color(srgb"));
//! ```
//!
//! Parse, convert, and format:
//!
//! ```rust
//! use culors::{convert, format_css, parse, Color};
//! use culors::spaces::Lab;
//!
//! let parsed = parse("oklch(70% 0.15 30deg)").expect("valid CSS");
//! let lab: Lab = match parsed {
//!     Color::Oklch(c) => convert(c),
//!     _ => unreachable!("oklch(...) parses as Color::Oklch"),
//! };
//! let css = format_css(&Color::Lab(lab));
//! assert!(css.starts_with("lab("));
//! ```
//!
//! # Supported color spaces
//!
//! Spaces are exposed as plain structs in [`spaces`] and as variants of
//! [`Color`].
//!
//! | Family | Spaces |
//! |---|---|
//! | sRGB and linear | [`Color::Rgb`], [`Color::LinearRgb`] |
//! | Cylindrical sRGB | [`Color::Hsl`], [`Color::Hsv`], [`Color::Hwb`] |
//! | CIE | [`Color::Lab`], [`Color::Lch`], [`Color::Luv`], [`Color::Lchuv`], [`Color::Xyz50`], [`Color::Xyz65`] |
//! | Oklab | [`Color::Oklab`], [`Color::Oklch`], [`Color::Okhsl`], [`Color::Okhsv`] |
//! | Wide-gamut RGB | [`Color::P3`], [`Color::Rec2020`], [`Color::A98`], [`Color::ProphotoRgb`] |
//! | DIN99o | [`Color::Dlab`], [`Color::Dlch`] |
//! | JzAzBz / ICtCp | [`Color::Jab`], [`Color::Jch`], [`Color::Itp`] |
//! | HSLuv | [`Color::Hsluv`], [`Color::Hpluv`] |
//! | Other | [`Color::Cubehelix`], [`Color::Hsi`], [`Color::Yiq`], [`Color::Xyb`], [`Color::Prismatic`] |
//!
//! [`Color::Prismatic`] is a culors extension; culori 4.0.2 has no
//! Prismatic mode.
//!
//! # Public API tour
//!
//! - [`Color`] is the tagged union over every supported space. Each
//!   variant wraps the matching struct from [`spaces`].
//! - The [`ColorSpace`] trait defines `to_xyz65` / `from_xyz65` for
//!   every space, plus alpha access.
//! - [`convert()`] is the generic conversion function. It routes
//!   through XYZ D65, so any pair of [`ColorSpace`] implementors
//!   works without enumerating conversion paths. For bit-exact culori
//!   parity on precision-critical pairs, use the direct `From` impls
//!   listed in [`mod@convert`], or the dynamic-dispatch
//!   [`Color::convert_to`] / typed [`convert::convert_culori`] APIs
//!   added in v1.2 — both follow culori's per-pair routing.
//! - [`parse()`] consumes CSS Color Module 4 syntax (named colors,
//!   hex, functional notation, `color()` profiles including the four
//!   wide-gamut spaces, and `color-mix()`). Malformed or unsupported
//!   input yields `None`.
//! - [`format_css`] serializes a [`Color`] to canonical CSS.
//! - [`interpolate()`] / [`interpolate_with()`] return a closure
//!   `Fn(f64) -> Color` that samples a multi-stop ramp at `t ∈ [0, 1]`
//!   in the requested space. [`HueFixup`] selects the cylindrical
//!   fixup strategy.
//! - [`blend()`] / [`blend_str()`] fold a stack of colors with one of the
//!   12 separable [`BlendMode`] modes, using Porter-Duff source-over
//!   with premultiplied alpha. Output is always `Color::Rgb`.
//! - [`average()`], [`average_number()`], [`average_angle()`] reduce a slice
//!   of colors / numbers / hue angles in a chosen mode.
//! - [`in_gamut`], [`clamp_gamut`], [`clamp_chroma`], [`to_gamut`]
//!   provide the gamut-mapping ladder, with [`to_gamut`] implementing
//!   the CSS Color Module 4 algorithm using ΔE OK.
//! - The `difference_*` family (Ciede76, Ciede94, Ciede2000, CMC, OK,
//!   JzAzBz, ICtCp, Euclidean, hue-chroma, hue-saturation) returns
//!   curried closures `Fn(&Color, &Color) -> f64`.
//! - [`wcag_luminance`] and [`wcag_contrast`] implement the WCAG 2.x
//!   contrast formula on sRGB.
//! - The `filter_*` family (`brightness`, `contrast`, `grayscale`,
//!   `hue_rotate`, `invert`, `saturate`, `sepia`, plus CVD `prot` /
//!   `deuter` / `trit`) returns a closure `Fn(&Color) -> Color`.
//!
//! # Interpolation
//!
//! ```rust
//! use culors::{interpolate, parse};
//!
//! let a = parse("oklch(70% 0.15 30deg)").unwrap();
//! let b = parse("oklch(70% 0.15 200deg)").unwrap();
//! let ramp = interpolate(&[a, b], "oklab");
//! let mid = ramp(0.5);
//! let _ = mid;
//! ```
//!
//! # WCAG contrast
//!
//! ```rust
//! use culors::{parse, wcag_contrast};
//!
//! let bg = parse("white").unwrap();
//! let fg = parse("black").unwrap();
//! let ratio = wcag_contrast(&bg, &fg);
//! assert!(ratio > 20.0);
//! ```
//!
//! # ΔE
//!
//! ```rust
//! use culors::{difference_ciede2000, parse};
//!
//! let de = difference_ciede2000(1.0, 1.0, 1.0);
//! let red = parse("red").unwrap();
//! let crimson = parse("crimson").unwrap();
//! assert!(de(&red, &crimson) > 0.0);
//! ```
//!
//! # Feature flags
//!
//! - `serde` (off by default): derives `Serialize` and `Deserialize`
//!   for every space struct and for [`Color`].
//!
//! # Status
//!
//! As of v1.4 culors is in maintenance/upstream-tracking mode. The
//! library tracks culori 4.0.2 with feature parity for everything
//! Rust-idiomatic. New features land in
//! [culori](https://github.com/Evercoder/culori) first; once accepted
//! upstream they're mirrored here. Bug fixes and culori version bumps
//! remain in scope.
//!
//! # Further reading
//!
//! See the project [README](https://github.com/koole/culors#readme) for
//! the features matrix, comparison to culori, and v1.0 known
//! divergences. Release history is in
//! [CHANGELOG.md](https://github.com/koole/culors/blob/main/CHANGELOG.md).
//!
//! # License
//!
//! MIT-licensed.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod average;
pub mod blend;
pub mod color;
pub mod contrast;
pub mod convert;
pub mod difference;
pub mod easing;
pub mod filter;
pub mod format;
pub mod gamut;
pub mod interpolate;
pub mod lerp;
pub mod map;
pub mod nearest;
pub mod parse;
pub mod random;
pub mod round;
pub mod samples;
pub mod spaces;
pub mod traits;
pub(crate) mod util;

pub use average::{average, average_angle, average_number};
pub use blend::{blend, blend_str, BlendMode};
pub use color::{Color, ColorVariantMismatch};
pub use contrast::{wcag_contrast, wcag_luminance};
pub use convert::{convert, convert_culori, converter};
pub use difference::{
    difference_ciede2000, difference_ciede76, difference_ciede94, difference_ciede94_with,
    difference_cmc, difference_euclidean, difference_euclidean_with, difference_euclidean_xyz,
    difference_hue_chroma, difference_hue_naive, difference_hue_saturation, difference_hyab,
    difference_itp, difference_jz, difference_kotsarenko_ramos, difference_ok,
};
pub use easing::{
    easing_gamma, easing_in_out_sine, easing_midpoint, easing_smootherstep, easing_smoothstep,
    easing_smoothstep_inverse,
};
pub use filter::{
    filter_brightness, filter_contrast, filter_deficiency_deuter, filter_deficiency_prot,
    filter_deficiency_trit, filter_grayscale, filter_hue_rotate, filter_invert, filter_saturate,
    filter_sepia,
};
pub use format::{format_css, format_hex, format_hex8, format_hsl, format_rgb};
pub use gamut::{clamp_chroma, clamp_gamut, clamp_rgb, displayable, in_gamut, to_gamut};
pub use interpolate::{
    interpolate, interpolate_with, interpolate_with_premultiplied_alpha, interpolator_piecewise,
    interpolator_spline_basis, interpolator_spline_basis_closed, interpolator_spline_monotone,
    interpolator_spline_monotone_2, interpolator_spline_monotone_closed,
    interpolator_spline_natural, interpolator_spline_natural_closed, ChannelInterp,
    ChannelInterpFactory, HueFixup, InterpolateOptions,
};
pub use lerp::{blerp, lerp, trilerp, unlerp};
pub use map::{
    map_alpha_divide, map_alpha_multiply, map_transfer_gamma, map_transfer_linear, mapper,
};
pub use nearest::nearest;
pub use parse::parse;
pub use random::{random, random_with_constraints};
pub use round::round;
pub use samples::{samples, samples_with_easing};
pub use traits::ColorSpace;
