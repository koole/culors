//! Dynamic color enum.

use crate::spaces::{Hsl, Hsv, Hwb, Lab, LinearRgb, Oklab, Rgb, Xyz50, Xyz65};

/// Tagged union over every supported color space. Variants are added as each
/// space lands.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Color {
    /// sRGB.
    Rgb(Rgb),
    /// Linear-sRGB.
    LinearRgb(LinearRgb),
    /// HSL (cylindrical sRGB).
    Hsl(Hsl),
    /// HSV (cylindrical sRGB).
    Hsv(Hsv),
    /// HWB (hue/whiteness/blackness).
    Hwb(Hwb),
    /// CIE Lab D50.
    Lab(Lab),
    /// Oklab (perceptually uniform).
    Oklab(Oklab),
    /// CIE XYZ D50.
    Xyz50(Xyz50),
    /// CIE XYZ D65.
    Xyz65(Xyz65),
}

impl From<Rgb> for Color {
    fn from(c: Rgb) -> Self {
        Color::Rgb(c)
    }
}

impl From<LinearRgb> for Color {
    fn from(c: LinearRgb) -> Self {
        Color::LinearRgb(c)
    }
}

impl From<Hsl> for Color {
    fn from(c: Hsl) -> Self {
        Color::Hsl(c)
    }
}

impl From<Hsv> for Color {
    fn from(c: Hsv) -> Self {
        Color::Hsv(c)
    }
}

impl From<Hwb> for Color {
    fn from(c: Hwb) -> Self {
        Color::Hwb(c)
    }
}

impl From<Lab> for Color {
    fn from(c: Lab) -> Self {
        Color::Lab(c)
    }
}

impl From<Oklab> for Color {
    fn from(c: Oklab) -> Self {
        Color::Oklab(c)
    }
}

impl From<Xyz50> for Color {
    fn from(c: Xyz50) -> Self {
        Color::Xyz50(c)
    }
}

impl From<Xyz65> for Color {
    fn from(c: Xyz65) -> Self {
        Color::Xyz65(c)
    }
}
