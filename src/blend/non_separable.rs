//! Non-separable blend modes per CSS Compositing 1 § 5.8.
//!
//! These four modes (`hue`, `saturation`, `color`, `luminosity`) operate on
//! whole RGB triples rather than per-channel because they rely on luminance
//! and saturation, which are functions of all three channels at once.
//!
//! The helpers below — `lum`, `sat`, `clip_color`, `set_lum`, `set_sat` —
//! follow the spec verbatim. Of note: the spec uses the luminance weights
//! `(0.3, 0.59, 0.11)`, which differ from Rec. 709 `(0.2126, 0.7152, 0.0722)`.
//! culori 4.0.2 does not implement these modes; this module is a
//! spec-direct port, not a culori-compatible one.
//!
//! Reference: <https://www.w3.org/TR/compositing-1/#blendingnonseparable>

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Triple {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

/// Spec luminance: `Lum(c) = 0.3*r + 0.59*g + 0.11*b`.
fn lum(c: Triple) -> f64 {
    0.3 * c.r + 0.59 * c.g + 0.11 * c.b
}

/// Spec saturation: `Sat(c) = max(r,g,b) - min(r,g,b)`.
fn sat(c: Triple) -> f64 {
    let max = c.r.max(c.g).max(c.b);
    let min = c.r.min(c.g).min(c.b);
    max - min
}

/// `ClipColor(c)` from the spec — pulls out-of-gamut values back into
/// `[0, 1]` while preserving luminance.
fn clip_color(c: Triple) -> Triple {
    let l = lum(c);
    let n = c.r.min(c.g).min(c.b);
    let x = c.r.max(c.g).max(c.b);
    let mut r = c.r;
    let mut g = c.g;
    let mut b = c.b;
    if n < 0.0 {
        r = l + ((r - l) * l) / (l - n);
        g = l + ((g - l) * l) / (l - n);
        b = l + ((b - l) * l) / (l - n);
    }
    if x > 1.0 {
        r = l + ((r - l) * (1.0 - l)) / (x - l);
        g = l + ((g - l) * (1.0 - l)) / (x - l);
        b = l + ((b - l) * (1.0 - l)) / (x - l);
    }
    Triple { r, g, b }
}

/// `SetLum(c, l)` from the spec — translate `c` along the achromatic
/// diagonal so its luminance equals `l`, then clip back into gamut.
fn set_lum(c: Triple, l: f64) -> Triple {
    let d = l - lum(c);
    clip_color(Triple {
        r: c.r + d,
        g: c.g + d,
        b: c.b + d,
    })
}

/// `SetSat(c, s)` from the spec — rescale the chromatic component of `c`
/// so its saturation equals `s`, holding mid-channel position constant.
///
/// The spec orders channels by value (min, mid, max) and operates on those
/// labels; this implementation uses a permutation index to map back to
/// `(r, g, b)`.
fn set_sat(c: Triple, s: f64) -> Triple {
    // Sort indices by channel value to identify (min, mid, max).
    let mut idx = [(0, c.r), (1, c.g), (2, c.b)];
    idx.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    let (min_i, _) = idx[0];
    let (mid_i, mid_v) = idx[1];
    let (max_i, max_v) = idx[2];

    let mut out = [0.0f64; 3];
    if max_v > idx[0].1 {
        out[mid_i] = (mid_v - idx[0].1) * s / (max_v - idx[0].1);
        out[max_i] = s;
    } else {
        // c_max == c_min: the spec pins c_min = c_mid = c_max = 0.
        out[mid_i] = 0.0;
        out[max_i] = 0.0;
    }
    out[min_i] = 0.0;
    Triple {
        r: out[0],
        g: out[1],
        b: out[2],
    }
}

/// `B(b, s) = SetLum(SetSat(s, Sat(b)), Lum(b))` — source's hue, backdrop's
/// saturation and luminance.
pub(crate) fn hue(b: Triple, s: Triple) -> Triple {
    set_lum(set_sat(s, sat(b)), lum(b))
}

/// `B(b, s) = SetLum(SetSat(b, Sat(s)), Lum(b))` — backdrop's hue and
/// luminance, source's saturation.
pub(crate) fn saturation(b: Triple, s: Triple) -> Triple {
    set_lum(set_sat(b, sat(s)), lum(b))
}

/// `B(b, s) = SetLum(s, Lum(b))` — source's hue and saturation, backdrop's
/// luminance.
pub(crate) fn color(b: Triple, s: Triple) -> Triple {
    set_lum(s, lum(b))
}

/// `B(b, s) = SetLum(b, Lum(s))` — backdrop's hue and saturation, source's
/// luminance.
pub(crate) fn luminosity(b: Triple, s: Triple) -> Triple {
    set_lum(b, lum(s))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(r: f64, g: f64, b: f64) -> Triple {
        Triple { r, g, b }
    }

    #[test]
    fn lum_pure_green_uses_spec_weights() {
        // Spec weight on green is 0.59.
        assert!((lum(t(0.0, 1.0, 0.0)) - 0.59).abs() < 1e-15);
    }

    #[test]
    fn sat_full_red_is_one() {
        assert!((sat(t(1.0, 0.0, 0.0)) - 1.0).abs() < 1e-15);
    }

    #[test]
    fn set_sat_grey_stays_grey() {
        // SetSat on an achromatic input pins all channels to 0.
        let out = set_sat(t(0.5, 0.5, 0.5), 0.7);
        assert_eq!(out, t(0.0, 0.0, 0.0));
    }
}
