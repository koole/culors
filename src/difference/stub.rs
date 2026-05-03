//! Placeholders for ΔE variants whose underlying color space is scheduled
//! for v0.4 (`JzAzBz`, `ICtCp`). Both factories compile and accept the
//! same argument shape culori exports, but every call returns `f64::NAN`
//! until the spaces land.

use crate::Color;

/// Euclidean distance in JzAzBz. Returns NaN until JzAzBz lands in v0.4.
///
/// Note: culori 4.0.2 does not export an equivalent function; this is a
/// forward-looking placeholder added for symmetry with `difference_itp`
/// (which culori does export).
// TODO(v0.4): wire up once `Jab` (and friends) ship.
pub fn difference_jz() -> impl Fn(&Color, &Color) -> f64 {
    |_std, _smp| f64::NAN
}

/// `differenceItp()` — ICtCp ΔE_ITP per Rec. ITU-R BT.2124. Mirrors
/// culori's `differenceEuclidean('itp', [518400, 129600, 518400])`.
/// Returns `NaN` until ICtCp lands in v0.4.
// TODO(v0.4): port `differenceEuclidean('itp', [518400, 129600, 518400])`.
pub fn difference_itp() -> impl Fn(&Color, &Color) -> f64 {
    |_std, _smp| f64::NAN
}
