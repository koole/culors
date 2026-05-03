//! Ported tests for the legacy hex / `rgb()` / `hsl()` serializers.
//!
//! Mirrors `node_modules/culori/test/formatter.test.js` (fetched from
//! https://github.com/Evercoder/culori). Each expected output was confirmed
//! against `culori.formatHex / formatHex8 / formatRgb / formatHsl` via
//! `node -e`.

use culors::spaces::{Hsl, Rgb};
use culors::{format_hex, format_hex8, format_hsl, format_rgb, parse, Color};

// ---------- formatHex ----------

#[test]
fn format_hex_tomato() {
    let tomato = parse("tomato").expect("tomato is a CSS named color");
    assert_eq!(format_hex(&tomato), "#ff6347");
}

// ---------- formatHex8 ----------

#[test]
fn format_hex8_white_alpha_zero() {
    let c = Color::Rgb(Rgb {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        alpha: Some(0.0),
    });
    assert_eq!(format_hex8(&c), "#ffffff00");
}

// ---------- formatRgb ----------

#[test]
fn format_rgb_hex_with_alpha() {
    // culori parses #f0f0f0f0 as rgb with alpha = 240/255 ≈ 0.941...,
    // serialize rounds to 0.94.
    let c = parse("#f0f0f0f0").expect("8-char hex parses");
    assert_eq!(format_rgb(&c), "rgba(240, 240, 240, 0.94)");
}

// ---------- formatHsl ----------

#[test]
fn format_hsl_named_red() {
    let red = parse("red").expect("red is a CSS named color");
    assert_eq!(format_hsl(&red), "hsl(0, 100%, 50%)");
}

#[test]
fn format_hsl_decimals_negative_alpha() {
    let c = Color::Hsl(Hsl {
        h: 30.21,
        s: 0.2361,
        l: 0.48321,
        alpha: Some(-0.2),
    });
    assert_eq!(format_hsl(&c), "hsla(30.21, 23.61%, 48.32%, 0)");
}

#[test]
fn format_hsl_out_of_range_clamped() {
    // Hue passes through unchanged (no wrap); S clamps to 100%, L clamps to
    // 0%. Alpha is absent, so the `hsl(...)` form is used.
    let c = Color::Hsl(Hsl {
        h: 405.0,
        s: 1.2,
        l: -1.0,
        alpha: None,
    });
    assert_eq!(format_hsl(&c), "hsl(405, 100%, 0%)");
}
