//! Per-space CSS formatter helpers. Pure stdlib, no allocations beyond the
//! returned `String`.

use std::fmt::Write as _;

/// Render a single channel value. NaN becomes `none` (mirroring culori's
/// `value !== undefined ? value : 'none'` after the JS engine has already
/// dropped the property for NaN-sentinel channels). Finite values use
/// Rust's default `f64` formatter, which produces the shortest
/// round-tripping decimal — same algorithm as JS `Number.prototype.toString`
/// for every value with magnitude in `[1e-6, 1e21)`.
fn channel(value: f64) -> String {
    if value.is_nan() {
        "none".to_string()
    } else {
        format!("{value}")
    }
}

/// Render a percentage channel for `hsl()` / `hwb()`. culori multiplies the
/// 0..1 channel by 100 before serializing (`c.s * 100 + '%'`); we match
/// that, again falling back to `none` for NaN.
fn channel_pct(value: f64) -> String {
    if value.is_nan() {
        "none".to_string()
    } else {
        format!("{}%", value * 100.0)
    }
}

/// Render the trailing alpha component, if any. culori's serializers all
/// share `c.alpha < 1 ? ` / ${c.alpha}` : ''`, which omits the suffix when
/// alpha is `undefined`, exactly `1`, or `NaN` (since each comparison is
/// `false` in JS). We translate that to: append iff the option is `Some(a)`
/// with `a < 1.0` and `a` is finite.
fn alpha_suffix(alpha: Option<f64>) -> String {
    match alpha {
        Some(a) if a.is_finite() && a < 1.0 => format!(" / {a}"),
        _ => String::new(),
    }
}

/// `color(<id> ch1 ch2 ch3[ / alpha])` — used by every space whose
/// `definition.js` exports `serialize` as a string identifier.
pub fn format_color_fn(id: &str, channels: &[f64; 3], alpha: Option<f64>) -> String {
    let mut out = String::with_capacity(32);
    out.push_str("color(");
    out.push_str(id);
    for &c in channels {
        out.push(' ');
        out.push_str(&channel(c));
    }
    out.push_str(&alpha_suffix(alpha));
    out.push(')');
    out
}

/// `color(<id> ch1 ch2 ch3 ch4[ / alpha])` — for spaces with more than
/// three numeric channels. Used by Prismatic (`l r g b`).
pub fn format_color_fn_4(id: &str, channels: &[f64; 4], alpha: Option<f64>) -> String {
    let mut out = String::with_capacity(36);
    out.push_str("color(");
    out.push_str(id);
    for &c in channels {
        out.push(' ');
        out.push_str(&channel(c));
    }
    out.push_str(&alpha_suffix(alpha));
    out.push(')');
    out
}

pub fn format_hsl(h: f64, s: f64, l: f64, alpha: Option<f64>) -> String {
    let mut out = String::with_capacity(24);
    write!(
        out,
        "hsl({} {} {}{})",
        channel(h),
        channel_pct(s),
        channel_pct(l),
        alpha_suffix(alpha),
    )
    .unwrap();
    out
}

pub fn format_hwb(h: f64, w: f64, b: f64, alpha: Option<f64>) -> String {
    let mut out = String::with_capacity(24);
    write!(
        out,
        "hwb({} {} {}{})",
        channel(h),
        channel_pct(w),
        channel_pct(b),
        alpha_suffix(alpha),
    )
    .unwrap();
    out
}

/// `lab()` and `oklab()` share an identical layout; only the function name
/// and the numeric ranges of `L`/`a`/`b` differ, so we reuse one helper.
pub fn format_lab_like(name: &str, l: f64, a: f64, b: f64, alpha: Option<f64>) -> String {
    let mut out = String::with_capacity(24);
    write!(
        out,
        "{name}({} {} {}{})",
        channel(l),
        channel(a),
        channel(b),
        alpha_suffix(alpha),
    )
    .unwrap();
    out
}

/// `lch()` and `oklch()` share an identical layout; reuse one helper.
pub fn format_lch_like(name: &str, l: f64, c: f64, h: f64, alpha: Option<f64>) -> String {
    let mut out = String::with_capacity(24);
    write!(
        out,
        "{name}({} {} {}{})",
        channel(l),
        channel(c),
        channel(h),
        alpha_suffix(alpha),
    )
    .unwrap();
    out
}
