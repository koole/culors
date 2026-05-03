//! CMC l:c color difference. Direct port of culori's `differenceCmc`
//! (`node_modules/culori/src/difference.js`), itself derived from
//! Bruce Lindbloom's reference equations.

use crate::difference::extract::to_lab65;
use crate::Color;
use std::f64::consts::PI;

/// CMC l:c ΔE. Defaults `(l=1, c=1)` mirror culori's
/// `differenceCmc(l=1, c=1)`. The `l:c` ratio for textile applications is
/// `2:1`.
pub fn difference_cmc(l_param: f64, c_param: f64) -> impl Fn(&Color, &Color) -> f64 {
    move |std, smp| cmc(*std, *smp, l_param, c_param)
}

fn cmc(std: Color, smp: Color, l_param: f64, c_param: f64) -> f64 {
    let (l_std, a_std, b_std) = to_lab65(std);
    let (l_smp, a_smp, b_smp) = to_lab65(smp);

    let c_std = (a_std * a_std + b_std * b_std).sqrt();
    // hStd in [0, 2π), as in culori.
    let mut h_std = b_std.atan2(a_std);
    if h_std < 0.0 {
        h_std += 2.0 * PI;
    }

    let c_smp = (a_smp * a_smp + b_smp * b_smp).sqrt();

    let dl2 = (l_std - l_smp).powi(2);
    let dc2 = (c_std - c_smp).powi(2);
    let dh2 = (a_std - a_smp).powi(2) + (b_std - b_smp).powi(2) - dc2;

    let f = (c_std.powi(4) / (c_std.powi(4) + 1900.0)).sqrt();
    let lower = 164.0 / 180.0 * PI;
    let upper = 345.0 / 180.0 * PI;
    let t = if h_std >= lower && h_std <= upper {
        0.56 + (0.2 * (h_std + 168.0 / 180.0 * PI).cos()).abs()
    } else {
        0.36 + (0.4 * (h_std + 35.0 / 180.0 * PI).cos()).abs()
    };

    let s_l = if l_std < 16.0 {
        0.511
    } else {
        (0.040975 * l_std) / (1.0 + 0.01765 * l_std)
    };
    let s_c = (0.0638 * c_std) / (1.0 + 0.0131 * c_std) + 0.638;
    let s_h = s_c * (f * t + 1.0 - f);

    (dl2 / (l_param * s_l).powi(2) + dc2 / (c_param * s_c).powi(2) + dh2 / s_h.powi(2)).sqrt()
}
