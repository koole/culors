//! CIELUV (CIE 1976 L*u*v*).
//!
//! Constants and formulas verbatim from culori 4.0.2
//! (`node_modules/culori/src/luv/`). Defined relative to the D50
//! reference white that culori uses for CIELab; the conversion routes
//! through [`Xyz50`].

#![allow(clippy::excessive_precision)]

use crate::spaces::{Rgb, Xyz50, Xyz65};
use crate::traits::ColorSpace;

const D50_X: f64 = 0.9642956764295677;
const D50_Y: f64 = 1.0;
const D50_Z: f64 = 0.8251046025104602;
const K: f64 = 24389.0 / 27.0;
const E: f64 = 216.0 / 24389.0;

#[inline]
fn u_fn(x: f64, y: f64, z: f64) -> f64 {
    (4.0 * x) / (x + 15.0 * y + 3.0 * z)
}

#[inline]
fn v_fn(x: f64, y: f64, z: f64) -> f64 {
    (9.0 * y) / (x + 15.0 * y + 3.0 * z)
}

fn un() -> f64 {
    u_fn(D50_X, D50_Y, D50_Z)
}

fn vn() -> f64 {
    v_fn(D50_X, D50_Y, D50_Z)
}

#[inline]
fn l_fn(value: f64) -> f64 {
    if value <= E {
        K * value
    } else {
        116.0 * value.cbrt() - 16.0
    }
}

/// CIELUV (D50). `l` is in 0..100, `u` and `v` are signed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Luv {
    /// Lightness in 0..100.
    pub l: f64,
    /// u opponent.
    pub u: f64,
    /// v opponent.
    pub v: f64,
    /// Optional alpha in 0..1.
    pub alpha: Option<f64>,
}

impl ColorSpace for Luv {
    const MODE: &'static str = "luv";
    const CHANNELS: &'static [&'static str] = &["l", "u", "v"];

    fn alpha(&self) -> Option<f64> {
        self.alpha
    }

    fn with_alpha(self, alpha: Option<f64>) -> Self {
        Self { alpha, ..self }
    }

    fn to_xyz65(&self) -> Xyz65 {
        Xyz50::from(*self).to_xyz65()
    }

    fn from_xyz65(xyz: Xyz65) -> Self {
        Xyz50::from_xyz65(xyz).into()
    }
}

impl From<Xyz50> for Luv {
    fn from(xyz: Xyz50) -> Self {
        let mut l = l_fn(xyz.y / D50_Y);
        let mut u = u_fn(xyz.x, xyz.y, xyz.z);
        let mut v = v_fn(xyz.x, xyz.y, xyz.z);
        if !u.is_finite() || !v.is_finite() {
            l = 0.0;
            u = 0.0;
            v = 0.0;
        } else {
            u = 13.0 * l * (u - un());
            v = 13.0 * l * (v - vn());
        }
        Self {
            l,
            u,
            v,
            alpha: xyz.alpha,
        }
    }
}

impl From<Luv> for Xyz50 {
    fn from(luv: Luv) -> Self {
        if luv.l == 0.0 {
            return Self {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                alpha: luv.alpha,
            };
        }
        let up = luv.u / (13.0 * luv.l) + un();
        let vp = luv.v / (13.0 * luv.l) + vn();
        let y = D50_Y
            * if luv.l <= 8.0 {
                luv.l / K
            } else {
                ((luv.l + 16.0) / 116.0).powi(3)
            };
        let x = (y * (9.0 * up)) / (4.0 * vp);
        let z = (y * (12.0 - 3.0 * up - 20.0 * vp)) / (4.0 * vp);
        Self {
            x,
            y,
            z,
            alpha: luv.alpha,
        }
    }
}

/// Direct `Rgb` → `Luv` mirroring culori's path with the achromatic-
/// RGB snap applied at the xyz50 level (the same trick `Lab` uses).
impl From<Rgb> for Luv {
    fn from(c: Rgb) -> Self {
        let mut xyz50 = Xyz50::from_xyz65(c.to_xyz65());
        if c.r == c.g && c.g == c.b {
            // Force achromatic into the white-axis: u/v become exactly 0.
            xyz50.x = D50_X * xyz50.y;
            xyz50.z = D50_Z * xyz50.y;
        }
        Luv::from(xyz50)
    }
}
