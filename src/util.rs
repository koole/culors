//! Math helpers shared across color-space implementations.

#[allow(dead_code)]
#[inline]
pub(crate) fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

#[allow(dead_code)]
#[inline]
pub(crate) fn clamp(x: f64, lo: f64, hi: f64) -> f64 {
    x.max(lo).min(hi)
}
