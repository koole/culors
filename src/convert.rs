//! Generic conversion between color spaces, via the XYZ D65 hub.

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
/// currently exist for: [`Rgb`](crate::spaces::Rgb) ↔
/// [`LinearRgb`](crate::spaces::LinearRgb), [`Rgb`](crate::spaces::Rgb) ↔
/// [`Hsl`](crate::spaces::Hsl), [`Rgb`](crate::spaces::Rgb) ↔
/// [`Hsv`](crate::spaces::Hsv), [`Hsv`](crate::spaces::Hsv) ↔
/// [`Hwb`](crate::spaces::Hwb), [`LinearRgb`](crate::spaces::LinearRgb) ↔
/// [`Oklab`](crate::spaces::Oklab), [`Oklab`](crate::spaces::Oklab) ↔
/// [`Oklch`](crate::spaces::Oklch), [`Xyz50`](crate::spaces::Xyz50) ↔
/// [`Lab`](crate::spaces::Lab), [`Lab`](crate::spaces::Lab) ↔
/// [`Lch`](crate::spaces::Lch), [`Rgb`](crate::spaces::Rgb) →
/// [`Lab`](crate::spaces::Lab) / [`Lch`](crate::spaces::Lch) /
/// [`Oklab`](crate::spaces::Oklab) / [`Oklch`](crate::spaces::Oklch).
///
/// # Pairs that diverge from culori's public API
///
/// The following pairs land at non-zero output through `convert<>()` where
/// culori's `converter(mode)` API produces an exact zero, because culori
/// snaps achromatic inputs (`r == g == b`) on the way through `Rgb`:
///
/// - `Rgb` (or any RGB-derived source) → [`Lab`](crate::spaces::Lab):
///   `convert<>()` leaves a residual ~1e-6 in `a` and `b`.
/// - `Rgb` (or any RGB-derived source) → [`Oklab`](crate::spaces::Oklab):
///   `convert<>()` leaves a residual ~1e-16 in `a` and `b`.
/// - `Rgb` (or any RGB-derived source) → [`Lch`](crate::spaces::Lch) or
///   [`Oklch`](crate::spaces::Oklch): `convert<>()` synthesizes a phantom
///   hue when chroma should be zero.
///
/// To match culori's snapped output, call `Lab::from(rgb)` / `Oklab::from(rgb)` /
/// `Lch::from(rgb)` / `Oklch::from(rgb)` directly.
///
/// Alpha is preserved by the hub conversions on both sides.
pub fn convert<A: ColorSpace, B: ColorSpace>(c: A) -> B {
    B::from_xyz65(c.to_xyz65())
}
