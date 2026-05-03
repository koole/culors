//! CIEDE2000 color difference. Direct port of culori's
//! `differenceCiede2000` (`node_modules/culori/src/difference.js`), which
//! itself ports Gaurav Sharma's reference Matlab implementation
//! (Sharma, Wu, Dalal 2005,
//! <http://www2.ece.rochester.edu/~gsharma/ciede2000/>). Constants are
//! verbatim from the JS source: every magic number, every angular offset,
//! every threshold.

use crate::difference::extract::to_lab65;
use crate::Color;
use std::f64::consts::PI;

/// CIEDE2000 ΔE — the modern industry standard. `Kl`, `Kc`, `Kh` are the
/// parametric weighting factors (default: all 1.0). Mirrors culori's
/// `differenceCiede2000(Kl=1, Kc=1, Kh=1)`.
pub fn difference_ciede2000(k_l: f64, k_c: f64, k_h: f64) -> impl Fn(&Color, &Color) -> f64 {
    move |std, smp| ciede2000(*std, *smp, k_l, k_c, k_h)
}

fn ciede2000(std: Color, smp: Color, k_l: f64, k_c: f64, k_h: f64) -> f64 {
    let (l_std, a_std, b_std) = to_lab65(std);
    let (l_smp, a_smp, b_smp) = to_lab65(smp);

    let c_std = (a_std * a_std + b_std * b_std).sqrt();
    let c_smp = (a_smp * a_smp + b_smp * b_smp).sqrt();

    let c_avg = (c_std + c_smp) / 2.0;

    // G = 0.5 * (1 - sqrt(c_avg^7 / (c_avg^7 + 25^7)))
    let g = 0.5 * (1.0 - (c_avg.powi(7) / (c_avg.powi(7) + 25f64.powi(7))).sqrt());

    let ap_std = a_std * (1.0 + g);
    let ap_smp = a_smp * (1.0 + g);

    let cp_std = (ap_std * ap_std + b_std * b_std).sqrt();
    let cp_smp = (ap_smp * ap_smp + b_smp * b_smp).sqrt();

    // hpStd: atan2 result wrapped into [0, 2π). When both Lab components
    // are zero culori returns 0 explicitly to dodge the atan2(0, 0) edge.
    let mut hp_std = if ap_std.abs() + b_std.abs() == 0.0 {
        0.0
    } else {
        b_std.atan2(ap_std)
    };
    if hp_std < 0.0 {
        hp_std += 2.0 * PI;
    }
    let mut hp_smp = if ap_smp.abs() + b_smp.abs() == 0.0 {
        0.0
    } else {
        b_smp.atan2(ap_smp)
    };
    if hp_smp < 0.0 {
        hp_smp += 2.0 * PI;
    }

    let d_l = l_smp - l_std;
    let d_c = cp_smp - cp_std;

    // dhp: shortest signed angular distance, zeroed when either chroma is
    // zero (matches culori's `cpStd * cpSmp === 0 ? 0 : ...`).
    let mut dhp = if cp_std * cp_smp == 0.0 {
        0.0
    } else {
        hp_smp - hp_std
    };
    if dhp > PI {
        dhp -= 2.0 * PI;
    }
    if dhp < -PI {
        dhp += 2.0 * PI;
    }

    let d_h = 2.0 * (cp_std * cp_smp).sqrt() * (dhp / 2.0).sin();

    let lp = (l_std + l_smp) / 2.0;
    let cp = (cp_std + cp_smp) / 2.0;

    // hp: arithmetic mean hue, with the wrap-aware adjustment culori
    // implements via `(Math.abs(...) > π) * π` and a follow-up
    // non-negative wrap. When either chroma is zero culori uses the sum
    // instead of the mean.
    let hp = if cp_std * cp_smp == 0.0 {
        hp_std + hp_smp
    } else {
        let mut hp = (hp_std + hp_smp) / 2.0;
        if (hp_std - hp_smp).abs() > PI {
            hp -= PI;
        }
        if hp < 0.0 {
            hp += 2.0 * PI;
        }
        hp
    };

    let lpm50 = (lp - 50.0).powi(2);

    // T term — the four cosine offsets are 30°, 0°, 6° (= π/30),
    // and 63° (= 63π/180).
    let t = 1.0 - 0.17 * (hp - PI / 6.0).cos()
        + 0.24 * (2.0 * hp).cos()
        + 0.32 * (3.0 * hp + PI / 30.0).cos()
        - 0.2 * (4.0 * hp - 63.0 * PI / 180.0).cos();

    let s_l = 1.0 + (0.015 * lpm50) / (20.0 + lpm50).sqrt();
    let s_c = 1.0 + 0.045 * cp;
    let s_h = 1.0 + 0.015 * cp * t;

    // Rotation term Rt — Gaussian-weighted around hue 275°.
    let delta_theta = (30.0 * PI / 180.0) * (-(((180.0 / PI) * hp - 275.0) / 25.0).powi(2)).exp();
    let r_c = 2.0 * (cp.powi(7) / (cp.powi(7) + 25f64.powi(7))).sqrt();
    let r_t = -(2.0 * delta_theta).sin() * r_c;

    let term_l = d_l / (k_l * s_l);
    let term_c = d_c / (k_c * s_c);
    let term_h = d_h / (k_h * s_h);

    (term_l * term_l + term_c * term_c + term_h * term_h + r_t * term_c * term_h).sqrt()
}
