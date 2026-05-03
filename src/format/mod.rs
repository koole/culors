//! CSS Color Module 4 formatter.
//!
//! Mirrors culori 4.0.2's `formatCss` (`node_modules/culori/src/formatter.js`
//! plus the per-space `serialize` field in each `definition.js`). Output is
//! the modern functional notation: space-separated channels, slash-prefixed
//! alpha, with `none` for missing components.
//!
//! Two output families:
//!
//! 1. Functional-notation spaces, `hsl()`, `hwb()`, `lab()`, `lch()`,
//!    `oklab()`, `oklch()`. Their `definition.js` uses an inline `c => …`
//!    serializer.
//! 2. `color()`-syntax spaces, `rgb` (id `srgb`), `lrgb` (`srgb-linear`),
//!    `hsv` (`--hsv`), `xyz65` (`xyz-d65`), `xyz50` (`xyz-d50`). Their
//!    `definition.js` uses a string `serialize` field which culori's
//!    dispatcher feeds into the `color()` template.
//!
//! NaN channels render as the CSS keyword `none` for both families. CSS
//! Color Module 4 specifies `none` as the missing-component keyword for
//! `color()` as well as for the functional spaces, so culor uses it
//! uniformly. culori, by contrast, emits the literal string `"NaN"` for a
//! `color()` channel that is `Number.NaN`; that path is unreachable from
//! culori's own pipeline (its converters never emit NaN into a `color()`
//! space) and only surfaces if a caller hand-builds such a value, so the
//! divergence is artificial and the Rust output is the spec-compliant one.
//!
//! Alpha mirrors culori's `c.alpha < 1` test: the alpha component only
//! appears when alpha is present and strictly less than 1. Alpha equal to
//! 1 or absent (`Option::None`) is omitted, matching `formatCss({…,
//! alpha:1})` → `…` and `formatCss({…})` → `…`.
//!
//! Numeric formatting uses Rust's default f64 `{}`, which agrees with JS's
//! `String(x)` for every value in the [1e-6, 1e21) magnitude range. Outside
//! that range the two diverge (JS switches to scientific notation; Rust
//! does not), but channel values in CSS color syntax never reach those
//! magnitudes.

mod css;

use crate::color::Color;

/// Format a [`Color`] as a CSS Color Module 4 string.
///
/// Output matches `culori.formatCss(c)` for any `Color` whose channels
/// originate from culori-equivalent inputs. Round-tripping through
/// [`crate::parse()`] is stable for canonical inputs (anything the
/// formatter itself produces).
pub fn format_css(color: &Color) -> String {
    match color {
        Color::Rgb(c) => css::format_color_fn("srgb", &[c.r, c.g, c.b], c.alpha),
        Color::LinearRgb(c) => css::format_color_fn("srgb-linear", &[c.r, c.g, c.b], c.alpha),
        Color::Hsv(c) => css::format_color_fn("--hsv", &[c.h, c.s, c.v], c.alpha),
        Color::Xyz65(c) => css::format_color_fn("xyz-d65", &[c.x, c.y, c.z], c.alpha),
        Color::Xyz50(c) => css::format_color_fn("xyz-d50", &[c.x, c.y, c.z], c.alpha),
        Color::Hsl(c) => css::format_hsl(c.h, c.s, c.l, c.alpha),
        Color::Hwb(c) => css::format_hwb(c.h, c.w, c.b, c.alpha),
        Color::Lab(c) => css::format_lab_like("lab", c.l, c.a, c.b, c.alpha),
        Color::Lch(c) => css::format_lch_like("lch", c.l, c.c, c.h, c.alpha),
        Color::Oklab(c) => css::format_lab_like("oklab", c.l, c.a, c.b, c.alpha),
        Color::Oklch(c) => css::format_lch_like("oklch", c.l, c.c, c.h, c.alpha),
        Color::P3(c) => css::format_color_fn("display-p3", &[c.r, c.g, c.b], c.alpha),
        Color::Rec2020(c) => css::format_color_fn("rec2020", &[c.r, c.g, c.b], c.alpha),
        Color::A98(c) => css::format_color_fn("a98-rgb", &[c.r, c.g, c.b], c.alpha),
        Color::ProphotoRgb(c) => css::format_color_fn("prophoto-rgb", &[c.r, c.g, c.b], c.alpha),
        Color::Cubehelix(c) => css::format_color_fn("--cubehelix", &[c.h, c.s, c.l], c.alpha),
        Color::Dlab(c) => css::format_color_fn("--din99o-lab", &[c.l, c.a, c.b], c.alpha),
        Color::Dlch(c) => css::format_color_fn("--din99o-lch", &[c.l, c.c, c.h], c.alpha),
        Color::Jab(c) => css::format_color_fn("--jzazbz", &[c.j, c.a, c.b], c.alpha),
    }
}
