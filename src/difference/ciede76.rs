//! CIE76 color difference — Euclidean distance in D65 Lab.
//!
//! Equivalent to culori's `differenceCie76 = () => differenceEuclidean('lab65')`.

use crate::difference::extract::to_lab65;
use crate::Color;

/// CIE76 ΔE — Euclidean distance in D65 Lab. Fast, perceptually rough.
pub fn difference_ciede76() -> impl Fn(&Color, &Color) -> f64 {
    |std, smp| {
        let (l1, a1, b1) = to_lab65(*std);
        let (l2, a2, b2) = to_lab65(*smp);
        let dl = l1 - l2;
        let da = a1 - a2;
        let db = b1 - b2;
        (dl * dl + da * da + db * db).sqrt()
    }
}
