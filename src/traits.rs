//! The `ColorSpace` trait — every color space implements it.

use crate::spaces::Xyz65;

/// A color space: a fixed set of channels with conversions to and from the
/// XYZ D65 hub.
pub trait ColorSpace: Sized + Copy + Clone + PartialEq {
    /// Stable identifier for this space (matches culori's `mode` string).
    const MODE: &'static str;
    /// Channel names in declaration order.
    const CHANNELS: &'static [&'static str];

    /// Returns the alpha channel, if set.
    fn alpha(&self) -> Option<f64>;
    /// Returns a copy of `self` with the given alpha.
    fn with_alpha(self, alpha: Option<f64>) -> Self;

    /// Convert this color into the XYZ D65 hub space.
    fn to_xyz65(&self) -> Xyz65;
    /// Construct a color of this space from the XYZ D65 hub space.
    fn from_xyz65(xyz: Xyz65) -> Self;
}
