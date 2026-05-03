//! A Rust port of [culori](https://github.com/evercoder/culori), the
//! JavaScript color library by Dan Burzo.
//!
//! culor implements a fixed set of color spaces, conversion between any pair
//! of them, a CSS Color Module 4 string parser, and a matching formatter.
//! Output values agree with culori 4.0.2 within 1e-10 across an exhaustive
//! fixture set (110 conversion pairs, 365 parse cases, 303 format
//! round-trips).
//!
//! # Quick start
//!
//! Parse a CSS string, convert it to another space, and format it back:
//!
//! ```rust
//! use culor::{convert, format_css, parse, Color};
//! use culor::spaces::Lab;
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
//! Round-trip a hex string through the formatter:
//!
//! ```rust
//! use culor::{format_css, parse, Color};
//!
//! let red = parse("#ff0000").unwrap();
//! assert!(matches!(red, Color::Rgb(_)));
//! assert_eq!(format_css(&red), "color(srgb 1 0 0)");
//! ```
//!
//! # Supported color spaces
//!
//! | Space       | Variant                             | CSS notation                    |
//! |-------------|-------------------------------------|---------------------------------|
//! | sRGB        | [`Color::Rgb`]                      | `rgb(...)`, `#rrggbb`, named    |
//! | Linear sRGB | [`Color::LinearRgb`]                | `color(srgb-linear ...)`        |
//! | HSL         | [`Color::Hsl`]                      | `hsl(...)`                      |
//! | HSV         | [`Color::Hsv`]                      | `color(--hsv ...)` (formatter)  |
//! | HWB         | [`Color::Hwb`]                      | `hwb(...)`                      |
//! | CIE Lab D50 | [`Color::Lab`]                      | `lab(...)`                      |
//! | CIE LCh D50 | [`Color::Lch`]                      | `lch(...)`                      |
//! | Oklab       | [`Color::Oklab`]                    | `oklab(...)`                    |
//! | Oklch       | [`Color::Oklch`]                    | `oklch(...)`                    |
//! | XYZ D50     | [`Color::Xyz50`]                    | `color(xyz-d50 ...)`            |
//! | XYZ D65     | [`Color::Xyz65`]                    | `color(xyz ...)` / `xyz-d65`    |
//!
//! # Public API tour
//!
//! - [`Color`] is the tagged union over every supported space. Each variant
//!   wraps the matching struct from [`spaces`].
//! - The [`ColorSpace`] trait defines `to_xyz65` / `from_xyz65` for every
//!   space, plus alpha access. It is the extension point for adding spaces.
//! - [`convert()`] is the generic conversion function. It routes through XYZ
//!   D65, so any pair of [`ColorSpace`] implementors works without enumerating
//!   conversion paths.
//! - [`parse()`] consumes a CSS Color Module 4 string and returns
//!   `Option<Color>`. Malformed input yields `None`; unsupported `color()`
//!   profiles also yield `None` (see the [`mod@parse`] module docs for the
//!   profile list).
//! - [`format_css`] serializes a [`Color`] to the CSS Color Module 4
//!   functional notation that [`parse()`] accepts.
//!
//! For pairs where culori takes a shorter routing than XYZ D65 (notably
//! anything `Rgb`-derived going to `Lab` / `Lch` / `Oklab` / `Oklch`), call
//! the matching `From` impl directly to get bit-for-bit culori parity. See
//! [`convert()`] for the full list.
//!
//! # Feature flags
//!
//! - `serde` (off by default): derives `Serialize` and `Deserialize` for
//!   every space struct and for [`Color`].
//!
//! # Further reading
//!
//! See the project [README](https://github.com/koole/culor#readme) for a
//! features matrix and the list of v0.1 known divergences from culori, and
//! [CHANGELOG.md](https://github.com/koole/culor/blob/main/CHANGELOG.md) for
//! release history.
//!
//! # License
//!
//! Dual-licensed under MIT or Apache-2.0.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod color;
pub mod convert;
pub mod format;
pub mod interpolate;
pub mod parse;
pub mod spaces;
pub mod traits;
pub(crate) mod util;

pub use color::Color;
pub use convert::convert;
pub use format::format_css;
pub use interpolate::{interpolate, interpolate_with, HueFixup, InterpolateOptions};
pub use parse::parse;
pub use traits::ColorSpace;
