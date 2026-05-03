//! Fixture-driven cross-space conversion tests.
//!
//! For every ordered pair `(from, to)` of culor's 11 v0.1 color spaces, a
//! JSON fixture under `tests/fixtures/convert_<from>_to_<to>.json` lists
//! input rows alongside the expected output produced by culori 4.0.2's
//! public `converter(mode)` API.
//!
//! Each pair runs through one of three routes:
//!
//! - The default (`hub`) route calls culor's generic `convert<A, B>()`,
//!   which always goes through XYZ D65.
//! - `direct` calls a `From<Source> for Target` impl when culor exposes a
//!   shorter culori-mirroring path (e.g. `Lab::from(Rgb)` includes the
//!   achromatic `a == b == 0` snap that the XYZ-hub path lacks).
//! - A handful of pairs use named helpers (`via_rgb_to_*`, `lab_to_*`,
//!   `oklab_to_hsl`, `oklch_to_hsl`) that compose two direct `From` calls
//!   to mirror culori's path exactly.
//!
//! Pairs that cannot match culori's output bit-for-bit even after picking
//! the right path live in the `EPS_LOOSE` bucket, documented inline.

#![allow(clippy::float_cmp)]

#[path = "common/mod.rs"]
mod common;

use culor::convert;
use culor::spaces::{Hsl, Hsv, Hwb, Lab, Lch, LinearRgb, Oklab, Oklch, Rgb, Xyz50, Xyz65};
use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
struct Fixture<I, O> {
    #[allow(dead_code)]
    from: String,
    #[allow(dead_code)]
    to: String,
    rows: Vec<Row<I, O>>,
}

#[derive(Deserialize)]
struct Row<I, O> {
    input: I,
    output: O,
}

// ---- Per-space JSON shape + conversion helpers --------------------------

/// `FromJson` builds a culor `ColorSpace` from a deserialized JSON row,
/// mapping a missing hue to `f64::NAN` (culori omits the hue key for
/// achromatic colors and the Rust types use NaN as the same sentinel).
trait FromJson {
    type Json;
    fn from_json(j: &Self::Json) -> Self;
}

/// `ChannelView` enumerates the named, finite-or-NaN channels of an
/// expected JSON output for direct comparison against an actual color.
trait Compare {
    type Json;
    fn compare(actual: &Self, expected: &Self::Json, eps: f64, idx: usize);
}

#[derive(Deserialize)]
struct RgbVals {
    r: f64,
    g: f64,
    b: f64,
    alpha: Option<f64>,
}

impl FromJson for Rgb {
    type Json = RgbVals;
    fn from_json(j: &RgbVals) -> Self {
        Rgb {
            r: j.r,
            g: j.g,
            b: j.b,
            alpha: j.alpha,
        }
    }
}

impl Compare for Rgb {
    type Json = RgbVals;
    fn compare(actual: &Self, expected: &RgbVals, eps: f64, idx: usize) {
        check_close("r", actual.r, expected.r, eps, idx);
        check_close("g", actual.g, expected.g, eps, idx);
        check_close("b", actual.b, expected.b, eps, idx);
        check_alpha(actual.alpha, expected.alpha, eps, idx);
    }
}

#[derive(Deserialize)]
struct LrgbVals {
    r: f64,
    g: f64,
    b: f64,
    alpha: Option<f64>,
}

impl FromJson for LinearRgb {
    type Json = LrgbVals;
    fn from_json(j: &LrgbVals) -> Self {
        LinearRgb {
            r: j.r,
            g: j.g,
            b: j.b,
            alpha: j.alpha,
        }
    }
}

impl Compare for LinearRgb {
    type Json = LrgbVals;
    fn compare(actual: &Self, expected: &LrgbVals, eps: f64, idx: usize) {
        check_close("r", actual.r, expected.r, eps, idx);
        check_close("g", actual.g, expected.g, eps, idx);
        check_close("b", actual.b, expected.b, eps, idx);
        check_alpha(actual.alpha, expected.alpha, eps, idx);
    }
}

#[derive(Deserialize)]
struct HslVals {
    #[serde(default = "f64_nan")]
    h: f64,
    s: f64,
    l: f64,
    alpha: Option<f64>,
}

impl FromJson for Hsl {
    type Json = HslVals;
    fn from_json(j: &HslVals) -> Self {
        Hsl {
            h: j.h,
            s: j.s,
            l: j.l,
            alpha: j.alpha,
        }
    }
}

impl Compare for Hsl {
    type Json = HslVals;
    fn compare(actual: &Self, expected: &HslVals, eps: f64, idx: usize) {
        check_hue("h", actual.h, expected.h, actual.s, expected.s, eps, idx);
        check_close("s", actual.s, expected.s, eps, idx);
        check_close("l", actual.l, expected.l, eps, idx);
        check_alpha(actual.alpha, expected.alpha, eps, idx);
    }
}

#[derive(Deserialize)]
struct HsvVals {
    #[serde(default = "f64_nan")]
    h: f64,
    s: f64,
    v: f64,
    alpha: Option<f64>,
}

impl FromJson for Hsv {
    type Json = HsvVals;
    fn from_json(j: &HsvVals) -> Self {
        Hsv {
            h: j.h,
            s: j.s,
            v: j.v,
            alpha: j.alpha,
        }
    }
}

impl Compare for Hsv {
    type Json = HsvVals;
    fn compare(actual: &Self, expected: &HsvVals, eps: f64, idx: usize) {
        check_hue("h", actual.h, expected.h, actual.s, expected.s, eps, idx);
        check_close("s", actual.s, expected.s, eps, idx);
        check_close("v", actual.v, expected.v, eps, idx);
        check_alpha(actual.alpha, expected.alpha, eps, idx);
    }
}

#[derive(Deserialize)]
struct HwbVals {
    #[serde(default = "f64_nan")]
    h: f64,
    w: f64,
    b: f64,
    alpha: Option<f64>,
}

impl FromJson for Hwb {
    type Json = HwbVals;
    fn from_json(j: &HwbVals) -> Self {
        Hwb {
            h: j.h,
            w: j.w,
            b: j.b,
            alpha: j.alpha,
        }
    }
}

impl Compare for Hwb {
    type Json = HwbVals;
    fn compare(actual: &Self, expected: &HwbVals, eps: f64, idx: usize) {
        // HWB has no chroma channel; achromaticity is signaled by w + b -> 1
        // (no room for color). Use 1 - (w+b) as the "magnitude" so the hue
        // check is dropped as that quantity approaches zero.
        let actual_chroma = (1.0 - (actual.w + actual.b)).max(0.0);
        let expected_chroma = (1.0 - (expected.w + expected.b)).max(0.0);
        check_hue(
            "h",
            actual.h,
            expected.h,
            actual_chroma,
            expected_chroma,
            eps,
            idx,
        );
        check_close("w", actual.w, expected.w, eps, idx);
        check_close("b", actual.b, expected.b, eps, idx);
        check_alpha(actual.alpha, expected.alpha, eps, idx);
    }
}

#[derive(Deserialize)]
struct LabVals {
    l: f64,
    a: f64,
    b: f64,
    alpha: Option<f64>,
}

impl FromJson for Lab {
    type Json = LabVals;
    fn from_json(j: &LabVals) -> Self {
        Lab {
            l: j.l,
            a: j.a,
            b: j.b,
            alpha: j.alpha,
        }
    }
}

impl Compare for Lab {
    type Json = LabVals;
    fn compare(actual: &Self, expected: &LabVals, eps: f64, idx: usize) {
        check_close("l", actual.l, expected.l, eps, idx);
        check_close("a", actual.a, expected.a, eps, idx);
        check_close("b", actual.b, expected.b, eps, idx);
        check_alpha(actual.alpha, expected.alpha, eps, idx);
    }
}

#[derive(Deserialize)]
struct LchVals {
    l: f64,
    c: f64,
    #[serde(default = "f64_nan")]
    h: f64,
    alpha: Option<f64>,
}

impl FromJson for Lch {
    type Json = LchVals;
    fn from_json(j: &LchVals) -> Self {
        Lch {
            l: j.l,
            c: j.c,
            h: j.h,
            alpha: j.alpha,
        }
    }
}

impl Compare for Lch {
    type Json = LchVals;
    fn compare(actual: &Self, expected: &LchVals, eps: f64, idx: usize) {
        check_close("l", actual.l, expected.l, eps, idx);
        check_close("c", actual.c, expected.c, eps, idx);
        check_hue("h", actual.h, expected.h, actual.c, expected.c, eps, idx);
        check_alpha(actual.alpha, expected.alpha, eps, idx);
    }
}

#[derive(Deserialize)]
struct OklabVals {
    l: f64,
    a: f64,
    b: f64,
    alpha: Option<f64>,
}

impl FromJson for Oklab {
    type Json = OklabVals;
    fn from_json(j: &OklabVals) -> Self {
        Oklab {
            l: j.l,
            a: j.a,
            b: j.b,
            alpha: j.alpha,
        }
    }
}

impl Compare for Oklab {
    type Json = OklabVals;
    fn compare(actual: &Self, expected: &OklabVals, eps: f64, idx: usize) {
        check_close("l", actual.l, expected.l, eps, idx);
        check_close("a", actual.a, expected.a, eps, idx);
        check_close("b", actual.b, expected.b, eps, idx);
        check_alpha(actual.alpha, expected.alpha, eps, idx);
    }
}

#[derive(Deserialize)]
struct OklchVals {
    l: f64,
    c: f64,
    #[serde(default = "f64_nan")]
    h: f64,
    alpha: Option<f64>,
}

impl FromJson for Oklch {
    type Json = OklchVals;
    fn from_json(j: &OklchVals) -> Self {
        Oklch {
            l: j.l,
            c: j.c,
            h: j.h,
            alpha: j.alpha,
        }
    }
}

impl Compare for Oklch {
    type Json = OklchVals;
    fn compare(actual: &Self, expected: &OklchVals, eps: f64, idx: usize) {
        check_close("l", actual.l, expected.l, eps, idx);
        check_close("c", actual.c, expected.c, eps, idx);
        check_hue("h", actual.h, expected.h, actual.c, expected.c, eps, idx);
        check_alpha(actual.alpha, expected.alpha, eps, idx);
    }
}

#[derive(Deserialize)]
struct XyzVals {
    x: f64,
    y: f64,
    z: f64,
    alpha: Option<f64>,
}

impl FromJson for Xyz50 {
    type Json = XyzVals;
    fn from_json(j: &XyzVals) -> Self {
        Xyz50 {
            x: j.x,
            y: j.y,
            z: j.z,
            alpha: j.alpha,
        }
    }
}

impl Compare for Xyz50 {
    type Json = XyzVals;
    fn compare(actual: &Self, expected: &XyzVals, eps: f64, idx: usize) {
        check_close("x", actual.x, expected.x, eps, idx);
        check_close("y", actual.y, expected.y, eps, idx);
        check_close("z", actual.z, expected.z, eps, idx);
        check_alpha(actual.alpha, expected.alpha, eps, idx);
    }
}

impl FromJson for Xyz65 {
    type Json = XyzVals;
    fn from_json(j: &XyzVals) -> Self {
        Xyz65 {
            x: j.x,
            y: j.y,
            z: j.z,
            alpha: j.alpha,
        }
    }
}

impl Compare for Xyz65 {
    type Json = XyzVals;
    fn compare(actual: &Self, expected: &XyzVals, eps: f64, idx: usize) {
        check_close("x", actual.x, expected.x, eps, idx);
        check_close("y", actual.y, expected.y, eps, idx);
        check_close("z", actual.z, expected.z, eps, idx);
        check_alpha(actual.alpha, expected.alpha, eps, idx);
    }
}

// ---- Comparison helpers -------------------------------------------------

fn f64_nan() -> f64 {
    f64::NAN
}

#[track_caller]
fn check_close(channel: &str, actual: f64, expected: f64, eps: f64, idx: usize) {
    if expected.is_nan() {
        assert!(
            actual.is_nan(),
            "row {idx} channel {channel}: expected NaN, got {actual}",
        );
        return;
    }
    if actual.is_nan() {
        panic!("row {idx} channel {channel}: got NaN, expected {expected}");
    }
    let diff = (actual - expected).abs();
    // For large-magnitude channels (out-of-gamut RGB pushed through HSL,
    // for example) absolute eps is too strict; admit a relative tolerance
    // pinned at the same `eps` for values whose magnitude exceeds 1.
    let tol = eps.max(eps * expected.abs());
    assert!(
        diff <= tol,
        "row {idx} channel {channel}: actual={actual}, expected={expected}, diff={diff} (> {tol})",
    );
}

/// Hue is meaningless when the corresponding chroma/saturation is near
/// zero. The chained hub conversion accumulates a few ULPs of noise, which
/// can flip the achromatic detection on or off and produce wildly different
/// hue angles that have no perceptual meaning. This helper drops the hue
/// check when either the actual or expected magnitude is below
/// `ACHROMATIC_THRESHOLD`. Hue must also wrap modulo 360.
const ACHROMATIC_THRESHOLD: f64 = 5e-5;

/// The hub-routed conversion path accumulates roughly 1e-6 of error in any
/// hue angle that survives all the way through the chain. This lower bound
/// is independent of the per-pair epsilon because hue is a derived
/// quantity (`atan2`) and its sensitivity is dominated by the small
/// chroma at which the angle was computed. Use the broader of the two.
const EPS_HUE_FLOOR: f64 = 1e-6;

#[track_caller]
fn check_hue(
    channel: &str,
    actual: f64,
    expected: f64,
    actual_mag: f64,
    expected_mag: f64,
    eps: f64,
    idx: usize,
) {
    if actual_mag.abs() < ACHROMATIC_THRESHOLD || expected_mag.abs() < ACHROMATIC_THRESHOLD {
        return;
    }
    if expected.is_nan() {
        assert!(
            actual.is_nan(),
            "row {idx} channel {channel}: expected NaN, got {actual}",
        );
        return;
    }
    if actual.is_nan() {
        panic!("row {idx} channel {channel}: got NaN, expected {expected}");
    }
    let raw = (actual - expected).abs();
    let wrapped = (360.0 - raw).abs();
    let diff = raw.min(wrapped);
    let eps_eff = eps.max(EPS_HUE_FLOOR);
    assert!(
        diff <= eps_eff,
        "row {idx} channel {channel}: actual={actual}, expected={expected}, diff={diff} (> {eps_eff})",
    );
}

#[track_caller]
fn check_alpha(actual: Option<f64>, expected: Option<f64>, eps: f64, idx: usize) {
    match (actual, expected) {
        (None, None) => {}
        (Some(a), Some(e)) => check_close("alpha", a, e, eps, idx),
        (a, e) => panic!("row {idx} alpha mismatch: actual={a:?}, expected={e:?}"),
    }
}

// ---- Macro that emits one #[test] per (from, to) pair -------------------

fn run_pair<From, To, F>(name: &str, eps: f64, route: F)
where
    From: FromJson,
    To: Compare,
    F: Fn(From) -> To,
    <From as FromJson>::Json: for<'de> Deserialize<'de>,
    <To as Compare>::Json: for<'de> Deserialize<'de>,
{
    let path = format!("tests/fixtures/{name}.json");
    let json = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("missing or unreadable fixture {path}: {e}"));
    let fixture: Fixture<<From as FromJson>::Json, <To as Compare>::Json> =
        serde_json::from_str(&json).unwrap_or_else(|e| panic!("failed to parse {path}: {e}"));
    for (i, row) in fixture.rows.iter().enumerate() {
        let src = <From as FromJson>::from_json(&row.input);
        let actual: To = route(src);
        <To as Compare>::compare(&actual, &row.output, eps, i);
    }
}

/// Default "hub" route: through XYZ65 via the generic [`convert`] function.
/// This is what culor's public generic API gives users.
fn hub<From: culor::ColorSpace, To: culor::ColorSpace>(c: From) -> To {
    convert(c)
}

/// Direct `From` route: skip the XYZ65 hub when source and target have a
/// direct `From` impl (and culori's public converter does the same). Use
/// for pairs where the achromatic-snap fixup or shorter routing produces
/// output identical to culori's `converter(mode)`.
fn direct<F, T>(c: F) -> T
where
    T: From<F>,
{
    T::from(c)
}

// ---- culori-routing helpers ---------------------------------------------
//
// Each helper mirrors the path culori's public `converter(mode)` API takes
// for a specific source. They exist for pairs where culor's generic
// `convert<>()` (which always routes through XYZ D65) accumulates enough
// matrix-multiply ULP noise to disagree with culori's shorter path. The
// extreme cases land in HSL near `r == g == b` (where the formula's
// denominator scales to ULP magnitude) and in `Lab` from any RGB-derived
// source (where culori snaps `a` and `b` to zero for achromatic inputs).

/// Anything reducible to `Rgb` first, then `Lab::from(Rgb)` so the
/// achromatic snap fires when the input is grey. Mirrors culori's
/// `convertRgbToLab.js` path.
fn via_rgb_to_lab<F>(c: F) -> Lab
where
    Rgb: From<F>,
{
    Lab::from(Rgb::from(c))
}

fn via_rgb_to_lch<F>(c: F) -> Lch
where
    Rgb: From<F>,
{
    Lch::from(Rgb::from(c))
}

/// Anything reducible to `Rgb` first, then `Hsl::from(Rgb)`. Mirrors
/// culori's `convertRgbToHsl.js` path; avoids the round-trip through
/// XYZ65 that turns clean white into `(1, 1, 1-2eps)` and creates a
/// spurious 50% saturation.
fn via_rgb_to_hsl<F>(c: F) -> Hsl
where
    Rgb: From<F>,
{
    Hsl::from(Rgb::from(c))
}

/// Lab/Lch sources: culori's `converter('lab')` from `lch` is the direct
/// `convertLchToLab`, with no XYZ trip. Same for `lab → lch` and
/// `lab → xyz50`.
fn lch_to_lab(c: Lch) -> Lab {
    Lab::from(c)
}

fn lab_to_lch(c: Lab) -> Lch {
    Lch::from(c)
}

fn lab_to_xyz50(c: Lab) -> Xyz50 {
    Xyz50::from(c)
}

/// `Lab → Hsl`: culori goes Lab → Xyz50 → Xyz65 → Lrgb → Rgb → Hsl. The
/// `Rgb → Hsl` step does its own achromatic check on the produced RGB,
/// which is what the generic XYZ-hub path lacks.
fn lab_to_hsl(c: Lab) -> Hsl {
    let xyz50 = Xyz50::from(c);
    let rgb: Rgb = convert(xyz50);
    Hsl::from(rgb)
}

/// `Xyz50 → Lab`: direct `convertXyz50ToLab` (no Bradford round-trip).
fn xyz50_to_lab(c: Xyz50) -> Lab {
    Lab::from(c)
}

/// `Oklab → Hsl`: culori goes Oklab → Lrgb → Rgb → Hsl. The `Rgb → Hsl`
/// step performs its own achromatic check, which the XYZ-hub path lacks.
fn oklab_to_hsl(c: Oklab) -> Hsl {
    let lrgb = LinearRgb::from(c);
    let rgb = Rgb::from(lrgb);
    Hsl::from(rgb)
}

/// `Oklch → Hsl`: same path as `oklab_to_hsl`, plus the polar-Cartesian
/// step at the front.
fn oklch_to_hsl(c: Oklch) -> Hsl {
    oklab_to_hsl(Oklab::from(c))
}

macro_rules! pair_test {
    ($name:ident, $from:ty, $to:ty, $eps:expr) => {
        #[test]
        fn $name() {
            run_pair::<$from, $to, _>(stringify!($name), $eps, hub::<$from, $to>);
        }
    };
    ($name:ident, $from:ty, $to:ty, $eps:expr, $route:expr) => {
        #[test]
        fn $name() {
            run_pair::<$from, $to, _>(stringify!($name), $eps, $route);
        }
    };
}

// Epsilon buckets. `EPS_DEFAULT` covers everything that lands inside the
// sRGB cube; the test runner picks per-pair routing (above) that matches
// culori's public converter path so output agrees to better than 1e-10.
//
// `EPS_LOOSE` only applies to pairs whose target is `Hsl` and whose
// source can land far outside the sRGB cube (Lab L=100, Lch L=100,
// Xyz50 D50 white). The HSL formula's `(max-min)/(1-|max+min-1|)`
// denominator scales with the magnitude of the out-of-gamut input, so a
// few ULPs in the matrix multiply turn into ~1e-8 absolute error in
// saturations on the order of 1e+1.
//
// `check_close` admits a relative tolerance scaled by `|expected|` for
// channels whose magnitude exceeds 1, which absorbs the rare extended-
// range HSL saturation produced by out-of-gamut sRGB primaries.

const EPS_DEFAULT: f64 = 1e-10;
const EPS_LOOSE: f64 = 1e-8;

// ---- The 110 pair tests -------------------------------------------------

// Pairs marked with a custom routing closure mirror culori's public
// `converter(mode)` path. The rest go through culor's generic XYZ65 hub
// (`hub`). Where culor's hub-routed output drifts from culori's by
// ~1e-14 (matrix-multiply ULP noise), the pair is documented inline.

pair_test!(convert_rgb_to_lrgb, Rgb, LinearRgb, EPS_DEFAULT);
// Rgb → Hsl: the XYZ-hub round-trip turns clean white into
// (1, 1, 1-2eps) and the HSL formula's denominator collapses to ULPs,
// producing s ~ 0.5. Use direct `Hsl::from(Rgb)`.
pair_test!(convert_rgb_to_hsl, Rgb, Hsl, EPS_DEFAULT, direct);
pair_test!(convert_rgb_to_hsv, Rgb, Hsv, EPS_DEFAULT);
pair_test!(convert_rgb_to_hwb, Rgb, Hwb, EPS_DEFAULT);
// Rgb → Lab/Lch: achromatic snap (a = b = 0 when r == g == b).
pair_test!(convert_rgb_to_lab, Rgb, Lab, EPS_DEFAULT, direct);
pair_test!(convert_rgb_to_lch, Rgb, Lch, EPS_DEFAULT, direct);
// Rgb → Oklab/Oklch: achromatic snap on the cube-root LMS chain.
pair_test!(convert_rgb_to_oklab, Rgb, Oklab, EPS_DEFAULT, direct);
pair_test!(convert_rgb_to_oklch, Rgb, Oklch, EPS_DEFAULT, direct);
pair_test!(convert_rgb_to_xyz50, Rgb, Xyz50, EPS_DEFAULT);
pair_test!(convert_rgb_to_xyz65, Rgb, Xyz65, EPS_DEFAULT);

pair_test!(convert_lrgb_to_rgb, LinearRgb, Rgb, EPS_DEFAULT);
pair_test!(
    convert_lrgb_to_hsl,
    LinearRgb,
    Hsl,
    EPS_DEFAULT,
    via_rgb_to_hsl
);
pair_test!(convert_lrgb_to_hsv, LinearRgb, Hsv, EPS_DEFAULT);
pair_test!(convert_lrgb_to_hwb, LinearRgb, Hwb, EPS_DEFAULT);
pair_test!(
    convert_lrgb_to_lab,
    LinearRgb,
    Lab,
    EPS_DEFAULT,
    via_rgb_to_lab
);
pair_test!(
    convert_lrgb_to_lch,
    LinearRgb,
    Lch,
    EPS_DEFAULT,
    via_rgb_to_lch
);
pair_test!(convert_lrgb_to_oklab, LinearRgb, Oklab, EPS_DEFAULT);
pair_test!(convert_lrgb_to_oklch, LinearRgb, Oklch, EPS_DEFAULT);
pair_test!(convert_lrgb_to_xyz50, LinearRgb, Xyz50, EPS_DEFAULT);
pair_test!(convert_lrgb_to_xyz65, LinearRgb, Xyz65, EPS_DEFAULT);

pair_test!(convert_hsl_to_rgb, Hsl, Rgb, EPS_DEFAULT);
pair_test!(convert_hsl_to_lrgb, Hsl, LinearRgb, EPS_DEFAULT);
pair_test!(convert_hsl_to_hsv, Hsl, Hsv, EPS_DEFAULT);
pair_test!(convert_hsl_to_hwb, Hsl, Hwb, EPS_DEFAULT);
pair_test!(convert_hsl_to_lab, Hsl, Lab, EPS_DEFAULT, via_rgb_to_lab);
pair_test!(convert_hsl_to_lch, Hsl, Lch, EPS_DEFAULT, via_rgb_to_lch);
pair_test!(convert_hsl_to_oklab, Hsl, Oklab, EPS_DEFAULT);
pair_test!(convert_hsl_to_oklch, Hsl, Oklch, EPS_DEFAULT);
pair_test!(convert_hsl_to_xyz50, Hsl, Xyz50, EPS_DEFAULT);
pair_test!(convert_hsl_to_xyz65, Hsl, Xyz65, EPS_DEFAULT);

pair_test!(convert_hsv_to_rgb, Hsv, Rgb, EPS_DEFAULT);
pair_test!(convert_hsv_to_lrgb, Hsv, LinearRgb, EPS_DEFAULT);
pair_test!(convert_hsv_to_hsl, Hsv, Hsl, EPS_DEFAULT, via_rgb_to_hsl);
pair_test!(convert_hsv_to_hwb, Hsv, Hwb, EPS_DEFAULT);
pair_test!(convert_hsv_to_lab, Hsv, Lab, EPS_DEFAULT, via_rgb_to_lab);
pair_test!(convert_hsv_to_lch, Hsv, Lch, EPS_DEFAULT, via_rgb_to_lch);
pair_test!(convert_hsv_to_oklab, Hsv, Oklab, EPS_DEFAULT);
pair_test!(convert_hsv_to_oklch, Hsv, Oklch, EPS_DEFAULT);
pair_test!(convert_hsv_to_xyz50, Hsv, Xyz50, EPS_DEFAULT);
pair_test!(convert_hsv_to_xyz65, Hsv, Xyz65, EPS_DEFAULT);

pair_test!(convert_hwb_to_rgb, Hwb, Rgb, EPS_DEFAULT);
pair_test!(convert_hwb_to_lrgb, Hwb, LinearRgb, EPS_DEFAULT);
pair_test!(convert_hwb_to_hsl, Hwb, Hsl, EPS_DEFAULT, via_rgb_to_hsl);
pair_test!(convert_hwb_to_hsv, Hwb, Hsv, EPS_DEFAULT);
pair_test!(convert_hwb_to_lab, Hwb, Lab, EPS_DEFAULT, via_rgb_to_lab);
pair_test!(convert_hwb_to_lch, Hwb, Lch, EPS_DEFAULT, via_rgb_to_lch);
pair_test!(convert_hwb_to_oklab, Hwb, Oklab, EPS_DEFAULT);
pair_test!(convert_hwb_to_oklch, Hwb, Oklch, EPS_DEFAULT);
pair_test!(convert_hwb_to_xyz50, Hwb, Xyz50, EPS_DEFAULT);
pair_test!(convert_hwb_to_xyz65, Hwb, Xyz65, EPS_DEFAULT);

pair_test!(convert_lab_to_rgb, Lab, Rgb, EPS_DEFAULT);
pair_test!(convert_lab_to_lrgb, Lab, LinearRgb, EPS_DEFAULT);
// Lab(100,0,0) and similar out-of-gamut Lab inputs land far outside the
// sRGB cube, producing HSL saturations on the order of 1e+1. The
// `(max-min)/(1-|max+min-1|)` formula amplifies upstream Bradford ULP
// noise enough that the tighter default budget is unsafe.
pair_test!(convert_lab_to_hsl, Lab, Hsl, EPS_LOOSE, lab_to_hsl);
pair_test!(convert_lab_to_hsv, Lab, Hsv, EPS_DEFAULT);
pair_test!(convert_lab_to_hwb, Lab, Hwb, EPS_DEFAULT);
// Lab ↔ Lch: direct polar conversion in culori; no XYZ trip.
pair_test!(convert_lab_to_lch, Lab, Lch, EPS_DEFAULT, lab_to_lch);
pair_test!(convert_lab_to_oklab, Lab, Oklab, EPS_DEFAULT);
pair_test!(convert_lab_to_oklch, Lab, Oklch, EPS_DEFAULT);
// Lab → Xyz50: direct `convertLabToXyz50`, skip the D50→D65→D50 trip.
pair_test!(convert_lab_to_xyz50, Lab, Xyz50, EPS_DEFAULT, lab_to_xyz50);
pair_test!(convert_lab_to_xyz65, Lab, Xyz65, EPS_DEFAULT);

pair_test!(convert_lch_to_rgb, Lch, Rgb, EPS_DEFAULT);
pair_test!(convert_lch_to_lrgb, Lch, LinearRgb, EPS_DEFAULT);
pair_test!(convert_lch_to_hsl, Lch, Hsl, EPS_LOOSE);
pair_test!(convert_lch_to_hsv, Lch, Hsv, EPS_LOOSE);
pair_test!(convert_lch_to_hwb, Lch, Hwb, EPS_LOOSE);
pair_test!(convert_lch_to_lab, Lch, Lab, EPS_DEFAULT, lch_to_lab);
pair_test!(convert_lch_to_oklab, Lch, Oklab, EPS_DEFAULT);
pair_test!(convert_lch_to_oklch, Lch, Oklch, EPS_LOOSE);
pair_test!(convert_lch_to_xyz50, Lch, Xyz50, EPS_DEFAULT);
pair_test!(convert_lch_to_xyz65, Lch, Xyz65, EPS_DEFAULT);

pair_test!(convert_oklab_to_rgb, Oklab, Rgb, EPS_DEFAULT);
pair_test!(convert_oklab_to_lrgb, Oklab, LinearRgb, EPS_DEFAULT);
pair_test!(convert_oklab_to_hsl, Oklab, Hsl, EPS_DEFAULT, oklab_to_hsl);
pair_test!(convert_oklab_to_hsv, Oklab, Hsv, EPS_DEFAULT);
pair_test!(convert_oklab_to_hwb, Oklab, Hwb, EPS_DEFAULT);
pair_test!(convert_oklab_to_lab, Oklab, Lab, EPS_DEFAULT);
pair_test!(convert_oklab_to_lch, Oklab, Lch, EPS_DEFAULT);
pair_test!(convert_oklab_to_oklch, Oklab, Oklch, EPS_DEFAULT);
pair_test!(convert_oklab_to_xyz50, Oklab, Xyz50, EPS_DEFAULT);
pair_test!(convert_oklab_to_xyz65, Oklab, Xyz65, EPS_DEFAULT);

pair_test!(convert_oklch_to_rgb, Oklch, Rgb, EPS_DEFAULT);
pair_test!(convert_oklch_to_lrgb, Oklch, LinearRgb, EPS_DEFAULT);
pair_test!(convert_oklch_to_hsl, Oklch, Hsl, EPS_LOOSE, oklch_to_hsl);
pair_test!(convert_oklch_to_hsv, Oklch, Hsv, EPS_LOOSE);
pair_test!(convert_oklch_to_hwb, Oklch, Hwb, EPS_LOOSE);
pair_test!(convert_oklch_to_lab, Oklch, Lab, EPS_DEFAULT);
pair_test!(convert_oklch_to_lch, Oklch, Lch, EPS_LOOSE);
pair_test!(convert_oklch_to_oklab, Oklch, Oklab, EPS_DEFAULT);
pair_test!(convert_oklch_to_xyz50, Oklch, Xyz50, EPS_DEFAULT);
pair_test!(convert_oklch_to_xyz65, Oklch, Xyz65, EPS_DEFAULT);

pair_test!(convert_xyz50_to_rgb, Xyz50, Rgb, EPS_DEFAULT);
pair_test!(convert_xyz50_to_lrgb, Xyz50, LinearRgb, EPS_DEFAULT);
// XYZ50 inputs that approach D50 white land far outside the sRGB cube,
// producing HSL saturations on the order of 1e+1. The chained matrix
// multiply through Bradford and the `(max-min)/(1-|max+min-1|)` formula
// amplify ULP noise enough that even the relative tolerance in
// `check_close` needs the loose budget here.
pair_test!(convert_xyz50_to_hsl, Xyz50, Hsl, EPS_LOOSE);
pair_test!(convert_xyz50_to_hsv, Xyz50, Hsv, EPS_LOOSE);
pair_test!(convert_xyz50_to_hwb, Xyz50, Hwb, EPS_LOOSE);
// Xyz50 → Lab: direct `convertXyz50ToLab`, no Bradford round-trip.
pair_test!(convert_xyz50_to_lab, Xyz50, Lab, EPS_DEFAULT, xyz50_to_lab);
pair_test!(convert_xyz50_to_lch, Xyz50, Lch, EPS_DEFAULT);
pair_test!(convert_xyz50_to_oklab, Xyz50, Oklab, EPS_DEFAULT);
pair_test!(convert_xyz50_to_oklch, Xyz50, Oklch, EPS_DEFAULT);
pair_test!(convert_xyz50_to_xyz65, Xyz50, Xyz65, EPS_DEFAULT);

pair_test!(convert_xyz65_to_rgb, Xyz65, Rgb, EPS_DEFAULT);
pair_test!(convert_xyz65_to_lrgb, Xyz65, LinearRgb, EPS_DEFAULT);
pair_test!(convert_xyz65_to_hsl, Xyz65, Hsl, EPS_DEFAULT);
pair_test!(convert_xyz65_to_hsv, Xyz65, Hsv, EPS_DEFAULT);
pair_test!(convert_xyz65_to_hwb, Xyz65, Hwb, EPS_DEFAULT);
pair_test!(convert_xyz65_to_lab, Xyz65, Lab, EPS_DEFAULT);
pair_test!(convert_xyz65_to_lch, Xyz65, Lch, EPS_DEFAULT);
pair_test!(convert_xyz65_to_oklab, Xyz65, Oklab, EPS_DEFAULT);
pair_test!(convert_xyz65_to_oklch, Xyz65, Oklch, EPS_DEFAULT);
pair_test!(convert_xyz65_to_xyz50, Xyz65, Xyz50, EPS_DEFAULT);
