//! Parser and evaluator for the CSS Color Module 5 `color-mix()` function.
//!
//! Spec: <https://www.w3.org/TR/css-color-5/#color-mix>. The grammar is
//! `color-mix( <color-interpolation-method>, <color> <percentage>?, <color>
//! <percentage>? )`, where `<color-interpolation-method>` is `in <space>
//! [<hue-method> hue]?`.
//!
//! culori 4.0.2 does not implement `color-mix()`; the algorithm here
//! follows the W3C spec directly. The two colors are recursively parsed
//! through [`crate::parse`], converted to the chosen interpolation space
//! via the existing `interpolate_with` decomposition, and combined with
//! premultiplied-alpha linear interpolation. Hue channels are fixed up
//! (`shorter` / `longer` / `increasing` / `decreasing`) using the same
//! strategies the interpolator applies. Sub-100% percentage sums scale
//! the final alpha by `sum / 100`, also per spec.
//!
//! `color-mix()` is dispatched from [`super::functional::parse_functional`];
//! it does not modify any per-space parser.
//!
//! Supported `<space>` values, per the v0.2 space matrix:
//! `srgb`, `srgb-linear`, `hsl`, `hwb`, `lab`, `lch`, `oklab`, `oklch`,
//! `xyz`, `xyz-d50`, `xyz-d65`. (`color-mix` does not accept `hsv` —
//! that is a culori extension, not a CSS Color 5 space.) Any other
//! `<space>` returns `None`.

use crate::color::Color;
use crate::interpolate::{interpolate_with, HueFixup, InterpolateOptions};
use crate::spaces::{Hsl, Hwb, Lab, Lch, LinearRgb, Oklab, Oklch, Rgb, Xyz50, Xyz65};

/// Try to parse and evaluate a `color-mix(...)` string. Returns `None`
/// for inputs that do not look like `color-mix(`, for malformed input,
/// for unsupported spaces, or for the both-percentages-zero case which
/// the spec leaves as transparent black.
pub(crate) fn parse_color_mix(input: &str) -> Option<Color> {
    let trimmed = input.trim();
    let lower_head = trimmed
        .get(..10)
        .map(str::to_ascii_lowercase)
        .unwrap_or_default();
    if lower_head != "color-mix(" {
        return None;
    }
    if !trimmed.ends_with(')') {
        return None;
    }
    let inner = &trimmed[10..trimmed.len() - 1];
    let parts = split_top_level_commas(inner)?;
    if parts.len() != 3 {
        return None;
    }
    let method = parse_method(parts[0].trim())?;
    let (c1_str, p1) = split_color_and_percentage(parts[1].trim())?;
    let (c2_str, p2) = split_color_and_percentage(parts[2].trim())?;
    let c1 = crate::parse::parse(c1_str)?;
    let c2 = crate::parse::parse(c2_str)?;
    evaluate(c1, c2, p1, p2, method)
}

#[derive(Debug, Clone, Copy)]
struct Method {
    mode: &'static str,
    hue: HueFixup,
}

fn parse_method(s: &str) -> Option<Method> {
    let mut iter = s.split_ascii_whitespace();
    let in_kw = iter.next()?;
    if !in_kw.eq_ignore_ascii_case("in") {
        return None;
    }
    let space = iter.next()?.to_ascii_lowercase();
    let mode = space_to_mode(&space)?;
    let hue = parse_hue_method(&mut iter)?;
    if iter.next().is_some() {
        return None;
    }
    Some(Method { mode, hue })
}

fn parse_hue_method<'a>(iter: &mut impl Iterator<Item = &'a str>) -> Option<HueFixup> {
    let Some(word) = iter.next() else {
        return Some(HueFixup::Shorter);
    };
    let strategy = match word.to_ascii_lowercase().as_str() {
        "shorter" => HueFixup::Shorter,
        "longer" => HueFixup::Longer,
        "increasing" => HueFixup::Increasing,
        "decreasing" => HueFixup::Decreasing,
        _ => return None,
    };
    // CSS spec requires the literal `hue` keyword after the strategy.
    let hue_kw = iter.next()?;
    if !hue_kw.eq_ignore_ascii_case("hue") {
        return None;
    }
    Some(strategy)
}

fn space_to_mode(space: &str) -> Option<&'static str> {
    match space {
        "srgb" => Some("rgb"),
        "srgb-linear" => Some("lrgb"),
        "hsl" => Some("hsl"),
        "hwb" => Some("hwb"),
        "lab" => Some("lab"),
        "lch" => Some("lch"),
        "oklab" => Some("oklab"),
        "oklch" => Some("oklch"),
        "xyz" | "xyz-d65" => Some("xyz65"),
        "xyz-d50" => Some("xyz50"),
        _ => None,
    }
}

fn split_top_level_commas(s: &str) -> Option<Vec<&str>> {
    let mut depth: i32 = 0;
    let mut start = 0;
    let mut parts: Vec<&str> = Vec::new();
    for (i, b) in s.bytes().enumerate() {
        match b {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth < 0 {
                    return None;
                }
            }
            b',' if depth == 0 => {
                parts.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    if depth != 0 {
        return None;
    }
    parts.push(&s[start..]);
    Some(parts)
}

/// Strip a percentage from either the leading or trailing edge of a
/// `<color> <percentage>?` chunk. Returns `(color_str, optional_pct)`.
fn split_color_and_percentage(s: &str) -> Option<(&str, Option<f64>)> {
    if s.is_empty() {
        return None;
    }
    if let Some((pct, rest)) = strip_percentage_prefix(s) {
        let rest = rest.trim_start();
        if rest.is_empty() {
            return None;
        }
        return Some((rest, Some(pct)));
    }
    if let Some((rest, pct)) = strip_percentage_suffix(s) {
        let rest = rest.trim_end();
        if rest.is_empty() {
            return None;
        }
        return Some((rest, Some(pct)));
    }
    Some((s, None))
}

fn strip_percentage_prefix(s: &str) -> Option<(f64, &str)> {
    let bytes = s.as_bytes();
    let (num, end) = read_number(bytes, 0)?;
    if bytes.get(end) != Some(&b'%') {
        return None;
    }
    let after = end + 1;
    if !matches!(bytes.get(after), Some(c) if c.is_ascii_whitespace()) {
        return None;
    }
    Some((num, &s[after..]))
}

fn strip_percentage_suffix(s: &str) -> Option<(&str, f64)> {
    let trimmed = s.trim_end();
    let bytes = trimmed.as_bytes();
    if bytes.last() != Some(&b'%') {
        return None;
    }
    // Walk backwards over the number characters.
    let mut i = bytes.len() - 1;
    while i > 0 {
        let c = bytes[i - 1];
        if c.is_ascii_digit() || c == b'.' || c == b'-' || c == b'+' {
            i -= 1;
        } else {
            break;
        }
    }
    let num_start = i;
    let num_end = bytes.len() - 1;
    if num_start >= num_end {
        return None;
    }
    let (num, parsed_end) = read_number(bytes, num_start)?;
    if parsed_end != num_end {
        return None;
    }
    let before_num = &trimmed[..num_start];
    if !before_num.ends_with(|c: char| c.is_ascii_whitespace()) {
        return None;
    }
    Some((before_num, num))
}

fn read_number(bytes: &[u8], start: usize) -> Option<(f64, usize)> {
    let mut i = start;
    if matches!(bytes.get(i), Some(b'+') | Some(b'-')) {
        i += 1;
    }
    let int_start = i;
    while matches!(bytes.get(i), Some(c) if c.is_ascii_digit()) {
        i += 1;
    }
    let mut had_int = i > int_start;
    if matches!(bytes.get(i), Some(b'.')) {
        i += 1;
        let frac_start = i;
        while matches!(bytes.get(i), Some(c) if c.is_ascii_digit()) {
            i += 1;
        }
        if i > frac_start {
            had_int = true;
        }
    }
    if !had_int {
        return None;
    }
    let s = std::str::from_utf8(&bytes[start..i]).ok()?;
    let v: f64 = s.parse().ok()?;
    Some((v, i))
}

fn evaluate(
    c1: Color,
    c2: Color,
    p1: Option<f64>,
    p2: Option<f64>,
    method: Method,
) -> Option<Color> {
    let (p1, p2) = match (p1, p2) {
        (None, None) => (50.0, 50.0),
        (Some(a), None) => (a, 100.0 - a),
        (None, Some(b)) => (100.0 - b, b),
        (Some(a), Some(b)) => (a, b),
    };
    if p1 < 0.0 || p2 < 0.0 || !p1.is_finite() || !p2.is_finite() {
        return None;
    }
    let sum = p1 + p2;
    if sum == 0.0 {
        return None;
    }
    let alpha_mult = if sum < 100.0 { sum / 100.0 } else { 1.0 };
    // After normalizing p1 + p2 to 100, the proportion of c2 (= t) is
    // p2 / sum. The per-color weights only matter through that ratio.
    let t = p2 / sum;
    Some(mix_premultiplied(c1, c2, t, alpha_mult, method))
}

fn mix_premultiplied(c1: Color, c2: Color, t: f64, alpha_mult: f64, method: Method) -> Color {
    // Build interpolation stops at t=0 and t=1 so we can reuse the same
    // hue-fixup logic as the rest of the crate. After the fixup we run
    // the premultiplied lerp by hand: `interpolate_with` does not
    // premultiply, but on the hot path of color-mix we only need a
    // single sample at `t`, so doing it inline is simpler than threading
    // a "premultiply" flag through the whole interpolator.
    let interp = interpolate_with(
        &[c1, c2],
        method.mode,
        InterpolateOptions::new().hue_fixup(method.hue),
    );
    let endpoint_a = interp(0.0);
    let endpoint_b = interp(1.0);
    let (chs_a, alpha_a) = decompose(endpoint_a, method.mode);
    let (chs_b, alpha_b) = decompose(endpoint_b, method.mode);
    let alpha_a = alpha_a.unwrap_or(1.0);
    let alpha_b = alpha_b.unwrap_or(1.0);

    let hue_index = hue_channel_index(method.mode);
    let hue_at_t: Option<f64> = if let Some(idx) = hue_index {
        // Reuse the interpolator's hue-fixup-aware sample at the actual t.
        let mid = interp(t);
        let (mid_chs, _) = decompose(mid, method.mode);
        Some(mid_chs[idx])
    } else {
        None
    };

    let mut out_chs = [0.0_f64; 3];
    for i in 0..3 {
        if hue_index == Some(i) {
            // CSS Color 5 normalizes the hue back to [0, 360) before
            // serializing the mixed colour.
            let h = hue_at_t.unwrap();
            out_chs[i] = if h.is_nan() {
                f64::NAN
            } else {
                let m = h % 360.0;
                if m < 0.0 {
                    m + 360.0
                } else {
                    m
                }
            };
            continue;
        }
        let va = chs_a[i];
        let vb = chs_b[i];
        // Treat NaN ("none" / powerless) as zero for premult, matching
        // how the existing alpha-fixup ("any defined fills missing")
        // already promotes to a real alpha here.
        let va_p = if va.is_nan() { 0.0 } else { va * alpha_a };
        let vb_p = if vb.is_nan() { 0.0 } else { vb * alpha_b };
        out_chs[i] = va_p * (1.0 - t) + vb_p * t;
    }
    let mut alpha_mixed = alpha_a * (1.0 - t) + alpha_b * t;
    if alpha_mixed > 0.0 {
        for (i, ch) in out_chs.iter_mut().enumerate() {
            if hue_index == Some(i) {
                continue;
            }
            *ch /= alpha_mixed;
        }
    }
    alpha_mixed *= alpha_mult;
    let final_alpha = if (alpha_mixed - 1.0).abs() < 1e-15 {
        None
    } else {
        Some(alpha_mixed)
    };
    compose(method.mode, out_chs, final_alpha)
}

fn hue_channel_index(mode: &str) -> Option<usize> {
    match mode {
        "hsl" | "hwb" => Some(0),
        "lch" | "oklch" => Some(2),
        _ => None,
    }
}

fn decompose(c: Color, mode: &str) -> ([f64; 3], Option<f64>) {
    match (c, mode) {
        (Color::Rgb(v), "rgb") => ([v.r, v.g, v.b], v.alpha),
        (Color::LinearRgb(v), "lrgb") => ([v.r, v.g, v.b], v.alpha),
        (Color::Hsl(v), "hsl") => ([v.h, v.s, v.l], v.alpha),
        (Color::Hwb(v), "hwb") => ([v.h, v.w, v.b], v.alpha),
        (Color::Lab(v), "lab") => ([v.l, v.a, v.b], v.alpha),
        (Color::Lch(v), "lch") => ([v.l, v.c, v.h], v.alpha),
        (Color::Oklab(v), "oklab") => ([v.l, v.a, v.b], v.alpha),
        (Color::Oklch(v), "oklch") => ([v.l, v.c, v.h], v.alpha),
        (Color::Xyz50(v), "xyz50") => ([v.x, v.y, v.z], v.alpha),
        (Color::Xyz65(v), "xyz65") => ([v.x, v.y, v.z], v.alpha),
        _ => unreachable!("interpolate_with returns the requested mode"),
    }
}

fn compose(mode: &str, chs: [f64; 3], alpha: Option<f64>) -> Color {
    match mode {
        "rgb" => Color::Rgb(Rgb {
            r: chs[0],
            g: chs[1],
            b: chs[2],
            alpha,
        }),
        "lrgb" => Color::LinearRgb(LinearRgb {
            r: chs[0],
            g: chs[1],
            b: chs[2],
            alpha,
        }),
        "hsl" => Color::Hsl(Hsl {
            h: chs[0],
            s: chs[1],
            l: chs[2],
            alpha,
        }),
        "hwb" => Color::Hwb(Hwb {
            h: chs[0],
            w: chs[1],
            b: chs[2],
            alpha,
        }),
        "lab" => Color::Lab(Lab {
            l: chs[0],
            a: chs[1],
            b: chs[2],
            alpha,
        }),
        "lch" => Color::Lch(Lch {
            l: chs[0],
            c: chs[1],
            h: chs[2],
            alpha,
        }),
        "oklab" => Color::Oklab(Oklab {
            l: chs[0],
            a: chs[1],
            b: chs[2],
            alpha,
        }),
        "oklch" => Color::Oklch(Oklch {
            l: chs[0],
            c: chs[1],
            h: chs[2],
            alpha,
        }),
        "xyz50" => Color::Xyz50(Xyz50 {
            x: chs[0],
            y: chs[1],
            z: chs[2],
            alpha,
        }),
        "xyz65" => Color::Xyz65(Xyz65 {
            x: chs[0],
            y: chs[1],
            z: chs[2],
            alpha,
        }),
        _ => unreachable!("space_to_mode validated"),
    }
}
