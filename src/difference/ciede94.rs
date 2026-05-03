//! CIE94 color difference. Direct port of culori's `differenceCie94` in
//! `node_modules/culori/src/difference.js`.

use crate::difference::extract::to_lab65;
use crate::Color;

/// CIE94 ΔE with explicit graphic-arts / textile parameters. Mirrors
/// culori's three-argument `differenceCie94(kL, K1, K2)`.
///
/// Graphic-arts defaults: `kL=1, K1=0.045, K2=0.015`. Textile case:
/// `kL=2, K1=0.048, K2=0.014`.
pub fn difference_ciede94_with(k_l: f64, k1: f64, k2: f64) -> impl Fn(&Color, &Color) -> f64 {
    move |std, smp| {
        let (l_std, a_std, b_std) = to_lab65(*std);
        let (l_smp, a_smp, b_smp) = to_lab65(*smp);
        let c_std = (a_std * a_std + b_std * b_std).sqrt();
        let c_smp = (a_smp * a_smp + b_smp * b_smp).sqrt();

        let dl2 = (l_std - l_smp).powi(2);
        let dc2 = (c_std - c_smp).powi(2);
        let dh2 = (a_std - a_smp).powi(2) + (b_std - b_smp).powi(2) - dc2;

        (dl2 / k_l.powi(2) + dc2 / (1.0 + k1 * c_std).powi(2) + dh2 / (1.0 + k2 * c_std).powi(2))
            .sqrt()
    }
}

/// CIE94 ΔE with the graphic-arts defaults (`kL=1, K1=0.045, K2=0.015`).
/// Pass `textiles=true` to swap to the textile parameters
/// `(kL=2, K1=0.048, K2=0.014)`.
pub fn difference_ciede94(textiles: bool) -> impl Fn(&Color, &Color) -> f64 {
    let (k_l, k1, k2) = if textiles {
        (2.0, 0.048, 0.014)
    } else {
        (1.0, 0.045, 0.015)
    };
    difference_ciede94_with(k_l, k1, k2)
}
