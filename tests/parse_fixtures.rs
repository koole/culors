//! Fixture-driven CSS parse tests.
//!
//! `tests/fixtures/parse_css.json` lists CSS input strings paired with
//! culori 4.0.2's `parse()` output. Each row reads as either:
//!
//! - `output: null` — culori rejects the input. culors must too.
//! - `output: { mode, ... }` — culori produces a parsed value tagged
//!   with `mode`. culors must produce a `Color` of the matching variant
//!   whose channels agree to within `EPS` (and whose alpha presence
//!   matches culori's).
//!
//! culori omits a channel's key when it is `none`; the comparison
//! treats absent keys as `f64::NAN` and matches them against culors's
//! NaN sentinel for the same channel. Alpha is the only `Option`-shaped
//! field: when culori omits `alpha` from its JSON the actual color must
//! also have `alpha: None`.

use culors::parse;
use culors::spaces::{Hsl, Hwb, Lab, Lch, LinearRgb, Oklab, Oklch, Rgb, Xyz50, Xyz65};
use culors::Color;
use serde::Deserialize;
use std::fs;

#[path = "common/mod.rs"]
mod common;

const EPS: f64 = 1e-12;

#[derive(Deserialize)]
struct ParseFixture {
    rows: Vec<ParseRow>,
}

#[derive(Deserialize)]
struct ParseRow {
    input: String,
    output: Option<serde_json::Value>,
}

#[test]
fn parse_fixtures_match_culori() {
    let json = fs::read_to_string("tests/fixtures/parse_css.json")
        .expect("missing tests/fixtures/parse_css.json");
    let fixture: ParseFixture = serde_json::from_str(&json).expect("invalid parse_css.json");
    let mut failures = Vec::new();
    for (i, row) in fixture.rows.iter().enumerate() {
        let actual = parse(&row.input);
        match (&row.output, actual) {
            (None, None) => {}
            (Some(expected), Some(actual_color)) => {
                if let Err(msg) = compare_color_to_json(&actual_color, expected) {
                    failures.push(format!("row {i}: input={:?} — {msg}", row.input));
                }
            }
            (None, Some(actual_color)) => failures.push(format!(
                "row {i}: input={:?} expected None, got {actual_color:?}",
                row.input
            )),
            (Some(expected), None) => failures.push(format!(
                "row {i}: input={:?} expected {expected}, got None",
                row.input
            )),
        }
    }
    if !failures.is_empty() {
        let n = failures.len();
        for f in failures.iter().take(20) {
            eprintln!("{f}");
        }
        if n > 20 {
            eprintln!("... and {} more", n - 20);
        }
        panic!("{n} parse fixture failures");
    }
}

fn compare_color_to_json(actual: &Color, expected: &serde_json::Value) -> Result<(), String> {
    let mode = expected
        .get("mode")
        .and_then(|m| m.as_str())
        .ok_or_else(|| format!("expected JSON missing `mode`: {expected}"))?;
    match (mode, actual) {
        ("rgb", Color::Rgb(c)) => check_rgb(c, expected),
        ("lrgb", Color::LinearRgb(c)) => check_lrgb(c, expected),
        ("hsl", Color::Hsl(c)) => check_hsl(c, expected),
        ("hwb", Color::Hwb(c)) => check_hwb(c, expected),
        ("lab", Color::Lab(c)) => check_lab(c, expected),
        ("lch", Color::Lch(c)) => check_lch(c, expected),
        ("oklab", Color::Oklab(c)) => check_oklab(c, expected),
        ("oklch", Color::Oklch(c)) => check_oklch(c, expected),
        ("xyz50", Color::Xyz50(c)) => check_xyz50(c, expected),
        ("xyz65", Color::Xyz65(c)) => check_xyz65(c, expected),
        (m, c) => Err(format!("mode mismatch: expected {m}, got {c:?}")),
    }
}

fn channel(json: &serde_json::Value, key: &str) -> f64 {
    json.get(key).and_then(|v| v.as_f64()).unwrap_or(f64::NAN)
}

fn alpha(json: &serde_json::Value) -> Option<f64> {
    json.get("alpha").and_then(|v| v.as_f64())
}

fn check_close(channel: &str, actual: f64, expected: f64) -> Result<(), String> {
    if expected.is_nan() {
        if actual.is_nan() {
            return Ok(());
        }
        return Err(format!("channel {channel}: expected NaN, got {actual}"));
    }
    if actual.is_nan() {
        return Err(format!("channel {channel}: got NaN, expected {expected}"));
    }
    let diff = (actual - expected).abs();
    let tol = EPS.max(EPS * expected.abs());
    if diff > tol {
        return Err(format!(
            "channel {channel}: actual={actual}, expected={expected}, diff={diff} (> {tol})"
        ));
    }
    Ok(())
}

fn check_alpha(actual: Option<f64>, expected: Option<f64>) -> Result<(), String> {
    match (actual, expected) {
        (None, None) => Ok(()),
        (Some(a), Some(e)) => check_close("alpha", a, e),
        (a, e) => Err(format!("alpha mismatch: actual={a:?}, expected={e:?}")),
    }
}

fn check_rgb(c: &Rgb, j: &serde_json::Value) -> Result<(), String> {
    check_close("r", c.r, channel(j, "r"))?;
    check_close("g", c.g, channel(j, "g"))?;
    check_close("b", c.b, channel(j, "b"))?;
    check_alpha(c.alpha, alpha(j))
}

fn check_lrgb(c: &LinearRgb, j: &serde_json::Value) -> Result<(), String> {
    check_close("r", c.r, channel(j, "r"))?;
    check_close("g", c.g, channel(j, "g"))?;
    check_close("b", c.b, channel(j, "b"))?;
    check_alpha(c.alpha, alpha(j))
}

fn check_hsl(c: &Hsl, j: &serde_json::Value) -> Result<(), String> {
    check_close("h", c.h, channel(j, "h"))?;
    check_close("s", c.s, channel(j, "s"))?;
    check_close("l", c.l, channel(j, "l"))?;
    check_alpha(c.alpha, alpha(j))
}

fn check_hwb(c: &Hwb, j: &serde_json::Value) -> Result<(), String> {
    check_close("h", c.h, channel(j, "h"))?;
    check_close("w", c.w, channel(j, "w"))?;
    check_close("b", c.b, channel(j, "b"))?;
    check_alpha(c.alpha, alpha(j))
}

fn check_lab(c: &Lab, j: &serde_json::Value) -> Result<(), String> {
    check_close("l", c.l, channel(j, "l"))?;
    check_close("a", c.a, channel(j, "a"))?;
    check_close("b", c.b, channel(j, "b"))?;
    check_alpha(c.alpha, alpha(j))
}

fn check_lch(c: &Lch, j: &serde_json::Value) -> Result<(), String> {
    check_close("l", c.l, channel(j, "l"))?;
    check_close("c", c.c, channel(j, "c"))?;
    check_close("h", c.h, channel(j, "h"))?;
    check_alpha(c.alpha, alpha(j))
}

fn check_oklab(c: &Oklab, j: &serde_json::Value) -> Result<(), String> {
    check_close("l", c.l, channel(j, "l"))?;
    check_close("a", c.a, channel(j, "a"))?;
    check_close("b", c.b, channel(j, "b"))?;
    check_alpha(c.alpha, alpha(j))
}

fn check_oklch(c: &Oklch, j: &serde_json::Value) -> Result<(), String> {
    check_close("l", c.l, channel(j, "l"))?;
    check_close("c", c.c, channel(j, "c"))?;
    check_close("h", c.h, channel(j, "h"))?;
    check_alpha(c.alpha, alpha(j))
}

fn check_xyz50(c: &Xyz50, j: &serde_json::Value) -> Result<(), String> {
    check_close("x", c.x, channel(j, "x"))?;
    check_close("y", c.y, channel(j, "y"))?;
    check_close("z", c.z, channel(j, "z"))?;
    check_alpha(c.alpha, alpha(j))
}

fn check_xyz65(c: &Xyz65, j: &serde_json::Value) -> Result<(), String> {
    check_close("x", c.x, channel(j, "x"))?;
    check_close("y", c.y, channel(j, "y"))?;
    check_close("z", c.z, channel(j, "z"))?;
    check_alpha(c.alpha, alpha(j))
}
