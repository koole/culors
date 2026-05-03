//! ΔE variants for HDR-friendly perceptual spaces: JzAzBz and ICtCp.

use crate::difference::euclidean::difference_euclidean_with;
use crate::Color;

/// Euclidean distance in JzAzBz (`difference_jz`). Mirrors a plain
/// `differenceEuclidean('jab')` — the [`Jab`](crate::spaces::Jab) space
/// is already perceptually uniform, so the unweighted Euclidean
/// distance is the natural ΔE.
pub fn difference_jz() -> impl Fn(&Color, &Color) -> f64 {
    difference_euclidean_with("jab", [1.0, 1.0, 1.0, 0.0])
}

/// `differenceItp()` — ICtCp ΔE_ITP per Rec. ITU-R BT.2124. Mirrors
/// culori's `differenceEuclidean('itp', [518400, 129600, 518400])`.
pub fn difference_itp() -> impl Fn(&Color, &Color) -> f64 {
    difference_euclidean_with("itp", [518400.0, 129600.0, 518400.0, 0.0])
}
