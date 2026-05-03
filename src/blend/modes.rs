//! Per-mode separable blend functions.
//!
//! Each function takes `(b, s)` — backdrop and source channel values in
//! `[0, 1]` — and returns the blended channel value. Constants and edge
//! cases match culori 4.0.2's `src/blend.js` byte-for-byte.

pub(crate) fn normal(_b: f64, s: f64) -> f64 {
    s
}

pub(crate) fn multiply(b: f64, s: f64) -> f64 {
    b * s
}

pub(crate) fn screen(b: f64, s: f64) -> f64 {
    b + s - b * s
}

pub(crate) fn hard_light(b: f64, s: f64) -> f64 {
    if s < 0.5 {
        b * 2.0 * s
    } else {
        2.0 * s * (1.0 - b) - 1.0
    }
}

pub(crate) fn overlay(b: f64, s: f64) -> f64 {
    if b < 0.5 {
        s * 2.0 * b
    } else {
        2.0 * b * (1.0 - s) - 1.0
    }
}

pub(crate) fn darken(b: f64, s: f64) -> f64 {
    b.min(s)
}

pub(crate) fn lighten(b: f64, s: f64) -> f64 {
    b.max(s)
}

pub(crate) fn color_dodge(b: f64, s: f64) -> f64 {
    if b == 0.0 {
        0.0
    } else if s == 1.0 {
        1.0
    } else {
        1.0_f64.min(b / (1.0 - s))
    }
}

pub(crate) fn color_burn(b: f64, s: f64) -> f64 {
    if b == 1.0 {
        1.0
    } else if s == 0.0 {
        0.0
    } else {
        1.0 - 1.0_f64.min((1.0 - b) / s)
    }
}

pub(crate) fn soft_light(b: f64, s: f64) -> f64 {
    if s < 0.5 {
        b - (1.0 - 2.0 * s) * b * (1.0 - b)
    } else {
        let d = if b < 0.25 {
            ((16.0 * b - 12.0) * b + 4.0) * b
        } else {
            b.sqrt()
        };
        b + (2.0 * s - 1.0) * (d - b)
    }
}

pub(crate) fn difference(b: f64, s: f64) -> f64 {
    (b - s).abs()
}

pub(crate) fn exclusion(b: f64, s: f64) -> f64 {
    b + s - 2.0 * b * s
}
