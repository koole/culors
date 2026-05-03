//! HyAB and Kotsarenko-Ramos color differences.
//!
//! Both factories live here because they are short formulae that don't
//! warrant their own modules.
//!
//! Ports culori 4.0.2's `differenceHyab` from `difference.js`:
//!
//! ```js
//! const differenceHyab = () => {
//!     let lab = converter('lab65');
//!     return (std, smp) => {
//!         let LabStd = lab(std);
//!         let LabSmp = lab(smp);
//!         let dL = LabStd.l - LabSmp.l;
//!         let dA = LabStd.a - LabSmp.a;
//!         let dB = LabStd.b - LabSmp.b;
//!         return Math.abs(dL) + Math.sqrt(dA * dA + dB * dB);
//!     };
//! };
//! ```
//!
//! Source: Abasi, Amani Tehran, Fairchild (2019), "Distance metrics for
//! very large color differences."

use crate::difference::euclidean::difference_euclidean_with;
use crate::difference::extract::extract;
use crate::Color;

/// HyAB color difference: `|ΔL| + √(Δa² + Δb²)` evaluated in `lab65`.
///
/// Designed by Abasi et al. (2019) for large color differences where
/// Euclidean distance in CIELab over-emphasizes lightness changes. Both
/// inputs are converted to D65 Lab before the formula is applied.
pub fn difference_hyab() -> impl Fn(&Color, &Color) -> f64 {
    move |std, smp| {
        let s = extract(*std, "lab65");
        let t = extract(*smp, "lab65");
        let dl = s[0] - t[0];
        let da = s[1] - t[1];
        let db = s[2] - t[2];
        dl.abs() + (da * da + db * db).sqrt()
    }
}

/// Kotsarenko-Ramos color difference, defined over Y'IQ.
///
/// Mirrors culori's `differenceKotsarenkoRamos`, which is a thin wrapper
/// around `differenceEuclidean('yiq', [0.5053, 0.299, 0.1957])`.
///
/// Source: Kotsarenko & Ramos (2010), "Measuring perceived color
/// difference using YIQ NTSC transmission color space in mobile
/// applications," Programación Matemática y Software.
pub fn difference_kotsarenko_ramos() -> impl Fn(&Color, &Color) -> f64 {
    difference_euclidean_with("yiq", [0.5053, 0.299, 0.1957, 0.0])
}
