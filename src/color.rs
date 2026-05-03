//! Dynamic color enum.

use crate::spaces::Xyz65;

/// Tagged union over every supported color space. Variants are added as each
/// space lands.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Color {
    /// CIE XYZ D65.
    Xyz65(Xyz65),
}

impl From<Xyz65> for Color {
    fn from(c: Xyz65) -> Self {
        Color::Xyz65(c)
    }
}
