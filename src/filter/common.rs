//! Internal helpers shared by every filter.

use crate::convert::convert;
use crate::spaces::{Hsv, Rgb, Xyz65};
use crate::traits::ColorSpace;
use crate::Color;

/// Converts any [`Color`] to sRGB. Mirrors culori's `converter('rgb')(color)`
/// semantics — direct conversions where a shorter routing exists, otherwise
/// via XYZ D65.
pub(crate) fn to_rgb(color: Color) -> Rgb {
    match color {
        Color::Rgb(x) => x,
        Color::LinearRgb(x) => x.into(),
        Color::Hsl(x) => x.into(),
        Color::Hsv(x) => x.into(),
        Color::Hwb(x) => Hsv::from(x).into(),
        other => convert::<Xyz65, Rgb>(color_to_xyz65(other)),
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
