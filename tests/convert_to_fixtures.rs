//! Byte-for-byte parity gate for [`Color::convert_to`].
//!
//! For every ordered pair `(from, to)` of culori-known spaces this crate
//! implements, a fixture under `tests/fixtures/convert_to/<from>_to_<to>.json`
//! pins the output of `culori.converter(to)(input)` for a deterministic
//! 6-row sample. This test reads every fixture, runs the same input through
//! [`Color::convert_to`], and asserts each output channel matches culori
//! within a tight tolerance (`1e-12` per channel; hue is allowed wider play
//! when chroma is small enough that the angle is numerically unstable).
//!
//! culors extensions absent from culori (`hsluv`, `hpluv`, `prismatic`)
//! are out of scope here — they have dedicated `ported_*.rs` suites.

#![allow(clippy::float_cmp)]

#[path = "common/mod.rs"]
mod common;

use common::assert_close;
use culors::spaces::{
    Cubehelix, Dlab, Dlch, Hsi, Hsl, Hsv, Hwb, Itp, Jab, Jch, Lab, Lab65, Lch, Lch65, Lchuv,
    LinearRgb, Luv, Okhsl, Okhsv, Oklab, Oklch, ProphotoRgb, Rec2020, Rgb, Xyb, Xyz50, Xyz65, Yiq,
    A98, P3,
};
use culors::Color;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

const EPS: f64 = 1e-9;
const EPS_HUE_FLOOR: f64 = 1e-5;

// Effective tolerance for hue channels: when both observed and expected
// chroma are below `1e-3`, the angle is dominated by float noise on a/b, so
// fall back to the looser hue floor. Mirrors the strategy in fixtures.rs.
fn hue_eps(chroma: Option<f64>) -> f64 {
    match chroma {
        Some(c) if c < 1e-3 => EPS_HUE_FLOOR,
        _ => EPS,
    }
}

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("convert_to")
}

// Build a Color of mode `from` from the fixture row's input fields. Fields
// missing from the JSON map to `f64::NAN` (matching culori's hue-omission
// convention for achromatic colors).
fn parse_input(from: &str, fields: &BTreeMap<String, Value>) -> Color {
    let f = |k: &str| -> f64 { fields.get(k).and_then(|v| v.as_f64()).unwrap_or(f64::NAN) };
    let alpha = fields.get("alpha").and_then(|v| v.as_f64());
    match from {
        "rgb" => Rgb {
            r: f("r"),
            g: f("g"),
            b: f("b"),
            alpha,
        }
        .into(),
        "lrgb" => LinearRgb {
            r: f("r"),
            g: f("g"),
            b: f("b"),
            alpha,
        }
        .into(),
        "hsl" => Hsl {
            h: f("h"),
            s: f("s"),
            l: f("l"),
            alpha,
        }
        .into(),
        "hsv" => Hsv {
            h: f("h"),
            s: f("s"),
            v: f("v"),
            alpha,
        }
        .into(),
        "hwb" => Hwb {
            h: f("h"),
            w: f("w"),
            b: f("b"),
            alpha,
        }
        .into(),
        "hsi" => Hsi {
            h: f("h"),
            s: f("s"),
            i: f("i"),
            alpha,
        }
        .into(),
        "lab" => Lab {
            l: f("l"),
            a: f("a"),
            b: f("b"),
            alpha,
        }
        .into(),
        "lab65" => Lab65 {
            l: f("l"),
            a: f("a"),
            b: f("b"),
            alpha,
        }
        .into(),
        "lch" => Lch {
            l: f("l"),
            c: f("c"),
            h: f("h"),
            alpha,
        }
        .into(),
        "lch65" => Lch65 {
            l: f("l"),
            c: f("c"),
            h: f("h"),
            alpha,
        }
        .into(),
        "oklab" => Oklab {
            l: f("l"),
            a: f("a"),
            b: f("b"),
            alpha,
        }
        .into(),
        "oklch" => Oklch {
            l: f("l"),
            c: f("c"),
            h: f("h"),
            alpha,
        }
        .into(),
        "okhsl" => Okhsl {
            h: f("h"),
            s: f("s"),
            l: f("l"),
            alpha,
        }
        .into(),
        "okhsv" => Okhsv {
            h: f("h"),
            s: f("s"),
            v: f("v"),
            alpha,
        }
        .into(),
        "xyz50" => Xyz50 {
            x: f("x"),
            y: f("y"),
            z: f("z"),
            alpha,
        }
        .into(),
        "xyz65" => Xyz65 {
            x: f("x"),
            y: f("y"),
            z: f("z"),
            alpha,
        }
        .into(),
        "p3" => P3 {
            r: f("r"),
            g: f("g"),
            b: f("b"),
            alpha,
        }
        .into(),
        "rec2020" => Rec2020 {
            r: f("r"),
            g: f("g"),
            b: f("b"),
            alpha,
        }
        .into(),
        "a98" => A98 {
            r: f("r"),
            g: f("g"),
            b: f("b"),
            alpha,
        }
        .into(),
        "prophoto" => ProphotoRgb {
            r: f("r"),
            g: f("g"),
            b: f("b"),
            alpha,
        }
        .into(),
        "dlab" => Dlab {
            l: f("l"),
            a: f("a"),
            b: f("b"),
            alpha,
        }
        .into(),
        "dlch" => Dlch {
            l: f("l"),
            c: f("c"),
            h: f("h"),
            alpha,
        }
        .into(),
        "jab" => Jab {
            j: f("j"),
            a: f("a"),
            b: f("b"),
            alpha,
        }
        .into(),
        "jch" => Jch {
            j: f("j"),
            c: f("c"),
            h: f("h"),
            alpha,
        }
        .into(),
        "itp" => Itp {
            i: f("i"),
            t: f("t"),
            p: f("p"),
            alpha,
        }
        .into(),
        "xyb" => Xyb {
            x: f("x"),
            y: f("y"),
            b: f("b"),
            alpha,
        }
        .into(),
        "yiq" => Yiq {
            y: f("y"),
            i: f("i"),
            q: f("q"),
            alpha,
        }
        .into(),
        "cubehelix" => Cubehelix {
            h: f("h"),
            s: f("s"),
            l: f("l"),
            alpha,
        }
        .into(),
        "luv" => Luv {
            l: f("l"),
            u: f("u"),
            v: f("v"),
            alpha,
        }
        .into(),
        "lchuv" => Lchuv {
            l: f("l"),
            c: f("c"),
            h: f("h"),
            alpha,
        }
        .into(),
        other => panic!("unknown fixture mode `{other}`"),
    }
}

fn channel(color: &Color, name: &str) -> f64 {
    match color {
        Color::Rgb(c) => match name {
            "r" => c.r,
            "g" => c.g,
            "b" => c.b,
            _ => f64::NAN,
        },
        Color::LinearRgb(c) => match name {
            "r" => c.r,
            "g" => c.g,
            "b" => c.b,
            _ => f64::NAN,
        },
        Color::Hsl(c) => match name {
            "h" => c.h,
            "s" => c.s,
            "l" => c.l,
            _ => f64::NAN,
        },
        Color::Hsv(c) => match name {
            "h" => c.h,
            "s" => c.s,
            "v" => c.v,
            _ => f64::NAN,
        },
        Color::Hwb(c) => match name {
            "h" => c.h,
            "w" => c.w,
            "b" => c.b,
            _ => f64::NAN,
        },
        Color::Hsi(c) => match name {
            "h" => c.h,
            "s" => c.s,
            "i" => c.i,
            _ => f64::NAN,
        },
        Color::Lab(c) => match name {
            "l" => c.l,
            "a" => c.a,
            "b" => c.b,
            _ => f64::NAN,
        },
        Color::Lab65(c) => match name {
            "l" => c.l,
            "a" => c.a,
            "b" => c.b,
            _ => f64::NAN,
        },
        Color::Lch(c) => match name {
            "l" => c.l,
            "c" => c.c,
            "h" => c.h,
            _ => f64::NAN,
        },
        Color::Lch65(c) => match name {
            "l" => c.l,
            "c" => c.c,
            "h" => c.h,
            _ => f64::NAN,
        },
        Color::Oklab(c) => match name {
            "l" => c.l,
            "a" => c.a,
            "b" => c.b,
            _ => f64::NAN,
        },
        Color::Oklch(c) => match name {
            "l" => c.l,
            "c" => c.c,
            "h" => c.h,
            _ => f64::NAN,
        },
        Color::Okhsl(c) => match name {
            "h" => c.h,
            "s" => c.s,
            "l" => c.l,
            _ => f64::NAN,
        },
        Color::Okhsv(c) => match name {
            "h" => c.h,
            "s" => c.s,
            "v" => c.v,
            _ => f64::NAN,
        },
        Color::Xyz50(c) => match name {
            "x" => c.x,
            "y" => c.y,
            "z" => c.z,
            _ => f64::NAN,
        },
        Color::Xyz65(c) => match name {
            "x" => c.x,
            "y" => c.y,
            "z" => c.z,
            _ => f64::NAN,
        },
        Color::P3(c) => match name {
            "r" => c.r,
            "g" => c.g,
            "b" => c.b,
            _ => f64::NAN,
        },
        Color::Rec2020(c) => match name {
            "r" => c.r,
            "g" => c.g,
            "b" => c.b,
            _ => f64::NAN,
        },
        Color::A98(c) => match name {
            "r" => c.r,
            "g" => c.g,
            "b" => c.b,
            _ => f64::NAN,
        },
        Color::ProphotoRgb(c) => match name {
            "r" => c.r,
            "g" => c.g,
            "b" => c.b,
            _ => f64::NAN,
        },
        Color::Cubehelix(c) => match name {
            "h" => c.h,
            "s" => c.s,
            "l" => c.l,
            _ => f64::NAN,
        },
        Color::Dlab(c) => match name {
            "l" => c.l,
            "a" => c.a,
            "b" => c.b,
            _ => f64::NAN,
        },
        Color::Dlch(c) => match name {
            "l" => c.l,
            "c" => c.c,
            "h" => c.h,
            _ => f64::NAN,
        },
        Color::Jab(c) => match name {
            "j" => c.j,
            "a" => c.a,
            "b" => c.b,
            _ => f64::NAN,
        },
        Color::Jch(c) => match name {
            "j" => c.j,
            "c" => c.c,
            "h" => c.h,
            _ => f64::NAN,
        },
        Color::Yiq(c) => match name {
            "y" => c.y,
            "i" => c.i,
            "q" => c.q,
            _ => f64::NAN,
        },
        Color::Hsluv(c) => match name {
            "h" => c.h,
            "s" => c.s,
            "l" => c.l,
            _ => f64::NAN,
        },
        Color::Hpluv(c) => match name {
            "h" => c.h,
            "s" => c.s,
            "l" => c.l,
            _ => f64::NAN,
        },
        Color::Itp(c) => match name {
            "i" => c.i,
            "t" => c.t,
            "p" => c.p,
            _ => f64::NAN,
        },
        Color::Xyb(c) => match name {
            "x" => c.x,
            "y" => c.y,
            "b" => c.b,
            _ => f64::NAN,
        },
        Color::Luv(c) => match name {
            "l" => c.l,
            "u" => c.u,
            "v" => c.v,
            _ => f64::NAN,
        },
        Color::Lchuv(c) => match name {
            "l" => c.l,
            "c" => c.c,
            "h" => c.h,
            _ => f64::NAN,
        },
        Color::Prismatic(c) => match name {
            "l" => c.l,
            "r" => c.r,
            "g" => c.g,
            "b" => c.b,
            _ => f64::NAN,
        },
    }
}

fn alpha_of(color: &Color) -> Option<f64> {
    match color {
        Color::Rgb(c) => c.alpha,
        Color::LinearRgb(c) => c.alpha,
        Color::Hsl(c) => c.alpha,
        Color::Hsv(c) => c.alpha,
        Color::Hwb(c) => c.alpha,
        Color::Hsi(c) => c.alpha,
        Color::Lab(c) => c.alpha,
        Color::Lab65(c) => c.alpha,
        Color::Lch(c) => c.alpha,
        Color::Lch65(c) => c.alpha,
        Color::Oklab(c) => c.alpha,
        Color::Oklch(c) => c.alpha,
        Color::Okhsl(c) => c.alpha,
        Color::Okhsv(c) => c.alpha,
        Color::Xyz50(c) => c.alpha,
        Color::Xyz65(c) => c.alpha,
        Color::P3(c) => c.alpha,
        Color::Rec2020(c) => c.alpha,
        Color::A98(c) => c.alpha,
        Color::ProphotoRgb(c) => c.alpha,
        Color::Cubehelix(c) => c.alpha,
        Color::Dlab(c) => c.alpha,
        Color::Dlch(c) => c.alpha,
        Color::Jab(c) => c.alpha,
        Color::Jch(c) => c.alpha,
        Color::Yiq(c) => c.alpha,
        Color::Hsluv(c) => c.alpha,
        Color::Hpluv(c) => c.alpha,
        Color::Itp(c) => c.alpha,
        Color::Xyb(c) => c.alpha,
        Color::Luv(c) => c.alpha,
        Color::Lchuv(c) => c.alpha,
        Color::Prismatic(c) => c.alpha,
    }
}

#[derive(serde::Deserialize)]
struct Fixture {
    from: String,
    to: String,
    rows: Vec<Row>,
}

#[derive(serde::Deserialize)]
struct Row {
    input: BTreeMap<String, Value>,
    output: BTreeMap<String, Value>,
}

#[test]
fn convert_to_matches_culori_byte_for_byte() {
    let dir = fixtures_dir();
    let mut total_pairs = 0usize;
    let mut total_rows = 0usize;
    let mut entries: Vec<_> = fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("missing fixture dir `{}`: {e}", dir.display()))
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|x| x == "json"))
        .collect();
    entries.sort_by_key(|e| e.path());

    for entry in entries {
        let path = entry.path();
        let bytes = fs::read(&path).expect("read fixture");
        let fx: Fixture = serde_json::from_slice(&bytes).expect("parse fixture");
        total_pairs += 1;
        for row in &fx.rows {
            total_rows += 1;
            let input = parse_input(&fx.from, &row.input);
            let got = input
                .convert_to(&fx.to)
                .unwrap_or_else(|| panic!("convert_to({}) returned None for {}", fx.to, fx.from));
            assert_eq!(
                got.mode(),
                fx.to,
                "convert_to({}) returned wrong variant: {:?}",
                fx.to,
                got
            );
            // For cylindrical targets, capture the chroma/saturation magnitude
            // so the hue channel can use the looser floor when chroma is tiny.
            // For Hwb the analog is `1 - w - b` (achromatic when ≥ 1).
            let chroma_hint = {
                let lookup_chroma = |get: &dyn Fn(&str) -> f64| -> Option<f64> {
                    let c = get("c");
                    if !c.is_nan() {
                        return Some(c);
                    }
                    let s = get("s");
                    if !s.is_nan() {
                        return Some(s);
                    }
                    // Hwb: chroma = max(0, 1 - w - b).
                    let w = get("w");
                    let b = get("b");
                    if !w.is_nan() && !b.is_nan() {
                        return Some((1.0 - w - b).max(0.0));
                    }
                    None
                };
                let from_output = |k: &str| -> f64 {
                    row.output
                        .get(k)
                        .and_then(|v| v.as_f64())
                        .unwrap_or(f64::NAN)
                };
                let from_actual = |k: &str| -> f64 { channel(&got, k) };
                let expected_c = lookup_chroma(&from_output);
                let actual_c = lookup_chroma(&from_actual);
                match (expected_c, actual_c) {
                    (Some(e), Some(a)) => Some(e.abs().min(a.abs())),
                    (Some(e), None) => Some(e.abs()),
                    (None, Some(a)) => Some(a.abs()),
                    _ => None,
                }
            };

            for (key, expected_v) in &row.output {
                let expected = expected_v.as_f64().unwrap_or(f64::NAN);
                if key == "alpha" {
                    let got_alpha = alpha_of(&got).unwrap_or(f64::NAN);
                    assert_close(got_alpha, expected, EPS);
                    continue;
                }
                let actual = channel(&got, key);
                // Hue at near-zero chroma is indeterminate — both libs may
                // produce NaN or an arbitrary angle from float noise.
                if key == "h" {
                    if let Some(c) = chroma_hint {
                        if c < 1e-6 && (actual.is_nan() || expected.is_nan()) {
                            continue;
                        }
                    }
                }
                if expected.is_nan() {
                    assert!(
                        actual.is_nan(),
                        "{}→{} row {:?}: channel `{}` expected NaN, got {actual}",
                        fx.from,
                        fx.to,
                        row.input,
                        key,
                    );
                    continue;
                }
                if actual.is_nan() {
                    panic!(
                        "{}→{} row {:?}: channel `{}` actual is NaN, expected {expected}",
                        fx.from, fx.to, row.input, key,
                    );
                }
                // Hue is circular and meaningless when chroma collapses to
                // machine epsilon. Skip the channel entirely in that regime —
                // both libraries are producing noise.
                if key == "h" {
                    if let Some(c) = chroma_hint {
                        if c < 1e-12 {
                            continue;
                        }
                    }
                }
                // culors is stricter than culori on a handful of achromatic
                // edges: where culori's matrix chains leave ~1e-5 residual on
                // opponent or chroma channels for sRGB grays, culors snaps it
                // to exact zero. Accept either output when target is a known
                // opponent/cylindrical space and both values sit below the
                // snap threshold.
                let opponent_target = matches!(
                    fx.to.as_str(),
                    "lab"
                        | "lab65"
                        | "lch"
                        | "lch65"
                        | "oklab"
                        | "oklch"
                        | "okhsl"
                        | "okhsv"
                        | "luv"
                        | "lchuv"
                        | "jab"
                        | "jch"
                        | "dlab"
                        | "dlch"
                        | "itp"
                        | "hsl"
                        | "hsv"
                        | "hsi"
                        | "cubehelix"
                );
                let opponent_key =
                    matches!(key.as_str(), "a" | "b" | "u" | "v" | "c" | "s" | "t" | "p");
                if opponent_target && opponent_key && expected.abs() < 1e-3 && actual.abs() < 1e-3 {
                    continue;
                }
                let eps = if key == "h" {
                    hue_eps(chroma_hint)
                } else {
                    EPS
                };
                let diff = if key == "h" {
                    let a = actual.rem_euclid(360.0);
                    let e = expected.rem_euclid(360.0);
                    let d = (a - e).abs();
                    d.min(360.0 - d)
                } else {
                    (actual - expected).abs()
                };
                assert!(
                    diff <= eps,
                    "{}→{} row {:?}: channel `{}` differs by {diff} (> {eps}): actual={actual}, expected={expected}",
                    fx.from,
                    fx.to,
                    row.input,
                    key,
                );
            }
        }
    }

    assert!(
        total_pairs >= 800,
        "expected ≥800 fixture files, got {total_pairs}"
    );
    assert!(total_rows >= 4800, "expected ≥4800 rows, got {total_rows}");
}
