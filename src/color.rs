//! Dynamic color enum.

use crate::spaces::{Rgb, Xyz65};

/// Tagged union over every supported color space. Variants are added as each
/// space lands.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Color {
    /// sRGB.
    Rgb(Rgb),
    /// CIE XYZ D65.
    Xyz65(Xyz65),
}

impl From<Rgb> for Color {
    fn from(c: Rgb) -> Self {
        Color::Rgb(c)
    }
}

impl From<Xyz65> for Color {
    fn from(c: Xyz65) -> Self {
        Color::Xyz65(c)
    }
}
