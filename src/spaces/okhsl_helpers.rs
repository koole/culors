//! Shared helpers for Okhsl and Okhsv.
//!
//! Adapted verbatim from Björn Ottosson's reference (MIT-licensed),
//! ported to Rust from culori 4.0.2 (`node_modules/culori/src/okhsl/helpers.js`).
//! Constants are kept verbatim for bit-for-bit parity with culori's output.

#![allow(clippy::excessive_precision)]

use crate::spaces::{LinearRgb, Oklab};

/// Cube-root toe function (Ottosson). Used to compress L into perceptual
/// 0..1 lightness.
#[inline]
pub(crate) fn toe(x: f64) -> f64 {
    let k1 = 0.206;
    let k2 = 0.03;
    let k3 = (1.0 + k1) / (1.0 + k2);
    0.5 * (k3 * x - k1 + ((k3 * x - k1).powi(2) + 4.0 * k2 * k3 * x).sqrt())
}

/// Inverse of [`toe`].
#[inline]
pub(crate) fn toe_inv(x: f64) -> f64 {
    let k1 = 0.206;
    let k2 = 0.03;
    let k3 = (1.0 + k1) / (1.0 + k2);
    (x * x + k1 * x) / (k3 * (x + k2))
}

/// Find the maximum saturation `S = C/L` along a given Oklab hue
/// direction (`a`, `b`) such that the resulting linear-sRGB triple has
/// at least one channel at zero. Polynomial-and-Halley as in the
/// reference.
fn compute_max_saturation(a: f64, b: f64) -> f64 {
    let (k0, k1, k2, k3, k4, wl, wm, ws) = if -1.88170328 * a - 0.80936493 * b > 1.0 {
        // Red component
        (
            1.19086277,
            1.76576728,
            0.59662641,
            0.75515197,
            0.56771245,
            4.0767416621,
            -3.3077115913,
            0.2309699292,
        )
    } else if 1.81444104 * a - 1.19445276 * b > 1.0 {
        // Green component
        (
            0.73956515,
            -0.45954404,
            0.08285427,
            0.1254107,
            0.14503204,
            -1.2684380046,
            2.6097574011,
            -0.3413193965,
        )
    } else {
        // Blue component
        (
            1.35733652,
            -0.00915799,
            -1.1513021,
            -0.50559606,
            0.00692167,
            -0.0041960863,
            -0.7034186147,
            1.707614701,
        )
    };

    let mut s = k0 + k1 * a + k2 * b + k3 * a * a + k4 * a * b;

    let k_l = 0.3963377774 * a + 0.2158037573 * b;
    let k_m = -0.1055613458 * a - 0.0638541728 * b;
    let k_s = -0.0894841775 * a - 1.291485548 * b;

    {
        let l_ = 1.0 + s * k_l;
        let m_ = 1.0 + s * k_m;
        let s_ = 1.0 + s * k_s;

        let l = l_ * l_ * l_;
        let m = m_ * m_ * m_;
        let s_cube = s_ * s_ * s_;

        let l_ds = 3.0 * k_l * l_ * l_;
        let m_ds = 3.0 * k_m * m_ * m_;
        let s_ds = 3.0 * k_s * s_ * s_;

        let l_ds2 = 6.0 * k_l * k_l * l_;
        let m_ds2 = 6.0 * k_m * k_m * m_;
        let s_ds2 = 6.0 * k_s * k_s * s_;

        let f = wl * l + wm * m + ws * s_cube;
        let f1 = wl * l_ds + wm * m_ds + ws * s_ds;
        let f2 = wl * l_ds2 + wm * m_ds2 + ws * s_ds2;

        s -= (f * f1) / (f1 * f1 - 0.5 * f * f2);
    }

    s
}

pub(crate) fn find_cusp(a: f64, b: f64) -> (f64, f64) {
    let s_cusp = compute_max_saturation(a, b);
    let rgb = LinearRgb::from(Oklab {
        l: 1.0,
        a: s_cusp * a,
        b: s_cusp * b,
        alpha: None,
    });
    let l_cusp = (1.0 / rgb.r.max(rgb.g).max(rgb.b)).cbrt();
    let c_cusp = l_cusp * s_cusp;
    (l_cusp, c_cusp)
}

fn find_gamut_intersection(a: f64, b: f64, l1: f64, c1: f64, l0: f64, cusp: (f64, f64)) -> f64 {
    let mut t;
    if (l1 - l0) * cusp.1 - (cusp.0 - l0) * c1 <= 0.0 {
        t = (cusp.1 * l0) / (c1 * cusp.0 + cusp.1 * (l0 - l1));
    } else {
        t = (cusp.1 * (l0 - 1.0)) / (c1 * (cusp.0 - 1.0) + cusp.1 * (l0 - l1));

        let dl = l1 - l0;
        let dc = c1;

        let k_l = 0.3963377774 * a + 0.2158037573 * b;
        let k_m = -0.1055613458 * a - 0.0638541728 * b;
        let k_s = -0.0894841775 * a - 1.291485548 * b;

        let l_dt = dl + dc * k_l;
        let m_dt = dl + dc * k_m;
        let s_dt = dl + dc * k_s;

        let l_val = l0 * (1.0 - t) + t * l1;
        let c_val = t * c1;

        let l_ = l_val + c_val * k_l;
        let m_ = l_val + c_val * k_m;
        let s_ = l_val + c_val * k_s;

        let l = l_ * l_ * l_;
        let m = m_ * m_ * m_;
        let s_cube = s_ * s_ * s_;

        let ldt = 3.0 * l_dt * l_ * l_;
        let mdt = 3.0 * m_dt * m_ * m_;
        let sdt = 3.0 * s_dt * s_ * s_;

        let ldt2 = 6.0 * l_dt * l_dt * l_;
        let mdt2 = 6.0 * m_dt * m_dt * m_;
        let sdt2 = 6.0 * s_dt * s_dt * s_;

        let r = 4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s_cube - 1.0;
        let r1 = 4.0767416621 * ldt - 3.3077115913 * mdt + 0.2309699292 * sdt;
        let r2 = 4.0767416621 * ldt2 - 3.3077115913 * mdt2 + 0.2309699292 * sdt2;
        let u_r = r1 / (r1 * r1 - 0.5 * r * r2);
        let mut t_r = -r * u_r;

        let g = -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s_cube - 1.0;
        let g1 = -1.2684380046 * ldt + 2.6097574011 * mdt - 0.3413193965 * sdt;
        let g2 = -1.2684380046 * ldt2 + 2.6097574011 * mdt2 - 0.3413193965 * sdt2;
        let u_g = g1 / (g1 * g1 - 0.5 * g * g2);
        let mut t_g = -g * u_g;

        let b_val = -0.0041960863 * l - 0.7034186147 * m + 1.707614701 * s_cube - 1.0;
        let b1 = -0.0041960863 * ldt - 0.7034186147 * mdt + 1.707614701 * sdt;
        let b2 = -0.0041960863 * ldt2 - 0.7034186147 * mdt2 + 1.707614701 * sdt2;
        let u_b = b1 / (b1 * b1 - 0.5 * b_val * b2);
        let mut t_b = -b_val * u_b;

        if u_r < 0.0 {
            t_r = 1.0e5;
        }
        if u_g < 0.0 {
            t_g = 1.0e5;
        }
        if u_b < 0.0 {
            t_b = 1.0e5;
        }

        t += t_r.min(t_g).min(t_b);
    }
    t
}

pub(crate) fn get_st_max(a: f64, b: f64, cusp: Option<(f64, f64)>) -> (f64, f64) {
    let cusp = cusp.unwrap_or_else(|| find_cusp(a, b));
    (cusp.1 / cusp.0, cusp.1 / (1.0 - cusp.0))
}

pub(crate) fn get_cs(l: f64, a: f64, b: f64) -> (f64, f64, f64) {
    let cusp = find_cusp(a, b);
    let c_max = find_gamut_intersection(a, b, l, 1.0, l, cusp);
    let st_max = get_st_max(a, b, Some(cusp));

    let s_mid = 0.11516993
        + 1.0
            / (7.4477897
                + 4.1590124 * b
                + a * (-2.19557347
                    + 1.75198401 * b
                    + a * (-2.13704948 - 10.02301043 * b
                        + a * (-4.24894561 + 5.38770819 * b + 4.69891013 * a))));
    let t_mid = 0.11239642
        + 1.0
            / (1.6132032 - 0.68124379 * b
                + a * (0.40370612
                    + 0.90148123 * b
                    + a * (-0.27087943
                        + 0.6122399 * b
                        + a * (0.00299215 - 0.45399568 * b - 0.14661872 * a))));

    let k = c_max / (l * st_max.0).min((1.0 - l) * st_max.1);

    let mut c_a = l * s_mid;
    let mut c_b = (1.0 - l) * t_mid;
    let c_mid = 0.9
        * k
        * ((1.0 / (1.0 / (c_a * c_a * c_a * c_a) + 1.0 / (c_b * c_b * c_b * c_b))).sqrt()).sqrt();

    c_a = l * 0.4;
    c_b = (1.0 - l) * 0.8;
    let c_0 = (1.0 / (1.0 / (c_a * c_a) + 1.0 / (c_b * c_b))).sqrt();
    (c_0, c_mid, c_max)
}
