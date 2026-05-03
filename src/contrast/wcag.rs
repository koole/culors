//! WCAG 2.1 relative luminance and contrast ratio.
//!
//! Ports culori 4.0.2's `node_modules/culori/src/wcag.js`:
//!
//! ```js
//! export function luminance(color) {
//!     let c = converter('lrgb')(color);
//!     return 0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b;
//! }
//! export function contrast(a, b) {
//!     let L1 = luminance(a);
//!     let L2 = luminance(b);
//!     return (Math.max(L1, L2) + 0.05) / (Math.min(L1, L2) + 0.05);
//! }
//! ```
//!
//! The luminance coefficients come from Rec. 709 / sRGB, matching
//! culori verbatim. Alpha is not part of the calculation: culori routes
//! through `converter('lrgb')`, which does not premultiply.

use crate::convert::convert;
use crate::spaces::{Hsv, LinearRgb, Rgb, Xyz65};
use crate::traits::ColorSpace;
use crate::Color;

/// WCAG 2.1 relative luminance. Returns a value in `[0, 1]` for in-gamut
/// sRGB inputs; out-of-gamut inputs may exceed those bounds because the
/// linearization is unclamped, matching culori.
pub fn wcag_luminance(c: &Color) -> f64 {
    let lrgb = to_lrgb(c);
    0.2126 * lrgb.r + 0.7152 * lrgb.g + 0.0722 * lrgb.b
}

/// WCAG 2.1 contrast ratio between two colors. Returns a value in
/// `[1, 21]` for in-gamut sRGB inputs. The result is symmetric in its
/// arguments.
pub fn wcag_contrast(a: &Color, b: &Color) -> f64 {
    let l1 = wcag_luminance(a);
    let l2 = wcag_luminance(b);
    (l1.max(l2) + 0.05) / (l1.min(l2) + 0.05)
}

fn to_lrgb(c: &Color) -> LinearRgb {
    match *c {
        Color::LinearRgb(x) => x,
        Color::Rgb(x) => x.into(),
        Color::Hsl(x) => Rgb::from(x).into(),
        Color::Hsv(x) => Rgb::from(x).into(),
        Color::Hwb(x) => Rgb::from(Hsv::from(x)).into(),
        other => convert::<Xyz65, LinearRgb>(to_xyz65(other)),
    }
}

fn to_xyz65(c: Color) -> Xyz65 {
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
