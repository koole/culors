//! ICtCp — HDR-friendly perceptual color space (Rec. BT.2100).
//!
//! Constants and matrices verbatim from culori 4.0.2
//! (`node_modules/culori/src/itp/`, `hdr/transfer.js`, `hdr/constants.js`).
//! ICtCp sits on absolute XYZ (Y=203 cd/m² as media white) with the PQ
//! transfer applied to LMS-shaped tristimulus values.

#![allow(clippy::excessive_precision)]

use crate::spaces::Xyz65;
use crate::traits::ColorSpace;

const YW: f64 = 203.0;

// PQ transfer constants (`hdr/transfer.js`).
const M1: f64 = 0.1593017578125;
const M2: f64 = 78.84375;
const C1: f64 = 0.8359375;
const C2: f64 = 18.8515625;
const C3: f64 = 18.6875;

#[inline]
fn pq_encode(v: f64) -> f64 {
    if v < 0.0 {
        0.0
    } else {
        let c = (v / 1e4).powf(M1);
        ((C1 + C2 * c) / (1.0 + C3 * c)).powf(M2)
    }
}

#[inline]
fn pq_decode(v: f64) -> f64 {
    if v < 0.0 {
        0.0
    } else {
        let c = v.powf(1.0 / M2);
        1e4 * ((c - C1).max(0.0) / (C2 - C3 * c)).powf(1.0 / M1)
    }
}

/// ICtCp color. `i` is the intensity (0..~1 for in-gamut sRGB), `t` and
/// `p` are signed chroma channels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Itp {
    /// Intensity (Iz).
    pub i: f64,
    /// Tritan chroma (Ct).
    pub t: f64,
    /// Protan chroma (Cp).
    pub p: f64,
    /// Optional alpha in 0..1.
    pub alpha: Option<f64>,
}

impl ColorSpace for Itp {
    const MODE: &'static str = "itp";
    const CHANNELS: &'static [&'static str] = &["i", "t", "p"];

    fn alpha(&self) -> Option<f64> {
        self.alpha
    }

    fn with_alpha(self, alpha: Option<f64>) -> Self {
        Self { alpha, ..self }
    }

    fn to_xyz65(&self) -> Xyz65 {
        let l = pq_decode(self.i + 0.008609037037932761 * self.t + 0.11102962500302593 * self.p);
        let m = pq_decode(self.i - 0.00860903703793275 * self.t - 0.11102962500302599 * self.p);
        let s = pq_decode(self.i + 0.5600313357106791 * self.t - 0.32062717498731885 * self.p);

        let to_rel = |c: f64| (c / YW).max(0.0);
        Xyz65 {
            x: to_rel(2.0701522183894219 * l - 1.3263473389671556 * m + 0.2066510476294051 * s),
            y: to_rel(0.3647385209748074 * l + 0.680566024947227 * m - 0.0453045459220346 * s),
            z: to_rel(-0.049747207535812 * l - 0.0492609666966138 * m + 1.1880659249923042 * s),
            alpha: self.alpha,
        }
    }

    fn from_xyz65(xyz: Xyz65) -> Self {
        let to_abs = |c: f64| (c * YW).max(0.0);
        let abs_x = to_abs(xyz.x);
        let abs_y = to_abs(xyz.y);
        let abs_z = to_abs(xyz.z);

        let l = pq_encode(
            0.3592832590121217 * abs_x + 0.6976051147779502 * abs_y - 0.0358915932320289 * abs_z,
        );
        let m = pq_encode(
            -0.1920808463704995 * abs_x + 1.1004767970374323 * abs_y + 0.0753748658519118 * abs_z,
        );
        let s = pq_encode(
            0.0070797844607477 * abs_x + 0.0748396662186366 * abs_y + 0.8433265453898765 * abs_z,
        );

        Self {
            i: 0.5 * l + 0.5 * m,
            t: 1.61376953125 * l - 3.323486328125 * m + 1.709716796875 * s,
            p: 4.378173828125 * l - 4.24560546875 * m - 0.132568359375 * s,
            alpha: xyz.alpha,
        }
    }
}
