//! Ported tests for the four hue-fixup strategies.
//!
//! Mirrors culori 4.0.2's `test/fixupHue.test.js`. Each expected vector
//! comes from running the corresponding `fixupHue*` function in culori
//! 4.0.2 against the same input array (see `node -e` snippets in the
//! v1.6 release notes). `NaN` here stands in for culori's `undefined`,
//! and the accumulator's NaN-propagation rule is verified explicitly.
//!
//! Up to v1.5 these strategies were exercised only indirectly via
//! `interpolate(..., HueFixup::X)`. v1.6 promotes the four functions to
//! the public surface so callers building custom interpolation pipelines
//! can reach for them directly, and so the inputs/outputs are tested
//! against literal arrays instead of round-tripping through a colour.

use culors::{fixup_hue_decreasing, fixup_hue_increasing, fixup_hue_longer, fixup_hue_shorter};

// Helper: assert two slices are bit-for-bit equal, treating NaN as equal
// to NaN (the standard PartialEq says they are not). Mirrors the
// `assert.deepEqual` semantics culori's tests rely on.
fn assert_eq_with_nan(actual: &[f64], expected: &[f64]) {
    assert_eq!(
        actual.len(),
        expected.len(),
        "length mismatch: {actual:?} vs {expected:?}"
    );
    for (i, (&a, &e)) in actual.iter().zip(expected.iter()).enumerate() {
        if a.is_nan() && e.is_nan() {
            continue;
        }
        assert_eq!(a, e, "mismatch at index {i}: {actual:?} vs {expected:?}");
    }
}

// --- fixup_hue_shorter ---------------------------------------------------

#[test]
fn shorter_culori_fixture() {
    // culori: fixupHueShorter([0, 340, 30, 0, 170]) === [0, -20, 30, 0, 170]
    assert_eq!(
        fixup_hue_shorter(&[0.0, 340.0, 30.0, 0.0, 170.0]),
        vec![0.0, -20.0, 30.0, 0.0, 170.0]
    );
}

#[test]
fn shorter_negative_inputs_normalize_then_fix() {
    // culori: fixupHueShorter([-250, -8]) === [110, -8]
    // (-250 mod 360 -> 110; -8 mod 360 -> 352, delta = 242, > 180 so -118 -> 110 - 118 = -8)
    assert_eq!(fixup_hue_shorter(&[-250.0, -8.0]), vec![110.0, -8.0]);
}

#[test]
fn shorter_empty_returns_empty() {
    assert!(fixup_hue_shorter(&[]).is_empty());
}

#[test]
fn shorter_single_value_normalized() {
    // No previous anchor, so the first value is normalized to [0, 360).
    assert_eq!(fixup_hue_shorter(&[45.0]), vec![45.0]);
    assert_eq!(fixup_hue_shorter(&[450.0]), vec![90.0]);
}

#[test]
fn shorter_repeated_value_zero_delta() {
    assert_eq!(
        fixup_hue_shorter(&[90.0, 90.0, 90.0]),
        vec![90.0, 90.0, 90.0]
    );
}

#[test]
fn shorter_exact_180_passes_through() {
    // |delta| == 180 is on the boundary; culori keeps the positive direction.
    assert_eq!(fixup_hue_shorter(&[0.0, 180.0]), vec![0.0, 180.0]);
}

#[test]
fn shorter_large_negatives_collapse_to_anchor() {
    // culori: fixupHueShorter([-720, -360, 0, 360, 720]) === [0, 0, 0, 0, 0]
    // Each entry normalizes to 0; deltas are 0, accumulator stays at 0.
    assert_eq!(
        fixup_hue_shorter(&[-720.0, -360.0, 0.0, 360.0, 720.0]),
        vec![0.0, 0.0, 0.0, 0.0, 0.0]
    );
}

#[test]
fn shorter_nan_resets_chain() {
    // culors uses NaN where culori uses `undefined`. A leading NaN passes
    // through, and the next defined value re-anchors the chain just as
    // culori's reducer does for `[undefined, 30, 60]` (which yields the
    // same `[undefined, 30, 60]` literally — no NaN poisoning).
    assert_eq_with_nan(
        &fixup_hue_shorter(&[f64::NAN, 30.0, 60.0]),
        &[f64::NAN, 30.0, 60.0],
    );
}

#[test]
fn shorter_nan_in_middle_breaks_chain() {
    // culori reference (with `undefined` instead of NaN):
    //   fixupHueShorter([10, undefined, 350]) === [10, undefined, 350]
    assert_eq_with_nan(
        &fixup_hue_shorter(&[10.0, f64::NAN, 350.0]),
        &[10.0, f64::NAN, 350.0],
    );
}

#[test]
fn shorter_all_nan_unchanged() {
    assert_eq_with_nan(
        &fixup_hue_shorter(&[f64::NAN, f64::NAN]),
        &[f64::NAN, f64::NAN],
    );
}

// --- fixup_hue_longer ----------------------------------------------------

#[test]
fn longer_culori_fixture() {
    // culori: fixupHueLonger([0, 340, 30, 0, 170]) === [0, 340, 30, 360, 170]
    assert_eq!(
        fixup_hue_longer(&[0.0, 340.0, 30.0, 0.0, 170.0]),
        vec![0.0, 340.0, 30.0, 360.0, 170.0]
    );
}

#[test]
fn longer_equal_consecutive_values() {
    // culori test comment: "equal consecutive values"
    // fixupHueLonger([0, 179, 179, 360]) === [0, -181, -181, 0]
    assert_eq!(
        fixup_hue_longer(&[0.0, 179.0, 179.0, 360.0]),
        vec![0.0, -181.0, -181.0, 0.0]
    );
}

#[test]
fn longer_repeated_value_zero_delta_passes_through() {
    // delta == 0 hits the special-case `d == 0.0` branch and returns 0 unchanged.
    assert_eq!(
        fixup_hue_longer(&[90.0, 90.0, 90.0]),
        vec![90.0, 90.0, 90.0]
    );
}

#[test]
fn longer_exact_180_takes_long_way() {
    // |delta| == 180 is >= 180 so it passes through (any direction is "long").
    assert_eq!(fixup_hue_longer(&[0.0, 180.0]), vec![0.0, 180.0]);
}

#[test]
fn longer_nan_breaks_chain() {
    // culori reference (with `undefined`):
    //   fixupHueLonger([10, undefined, 200]) === [10, undefined, 200]
    assert_eq_with_nan(
        &fixup_hue_longer(&[10.0, f64::NAN, 200.0]),
        &[10.0, f64::NAN, 200.0],
    );
}

// --- fixup_hue_increasing -----------------------------------------------

#[test]
fn increasing_culori_fixture() {
    // culori: fixupHueIncreasing([0, 340, 30, 0, 170]) === [0, 340, 390, 720, 890]
    assert_eq!(
        fixup_hue_increasing(&[0.0, 340.0, 30.0, 0.0, 170.0]),
        vec![0.0, 340.0, 390.0, 720.0, 890.0]
    );
}

#[test]
fn increasing_empty() {
    assert!(fixup_hue_increasing(&[]).is_empty());
}

#[test]
fn increasing_repeated_values_no_jump() {
    // delta == 0 is non-negative, so it stays 0 (no +360 shift).
    assert_eq!(fixup_hue_increasing(&[90.0, 90.0]), vec![90.0, 90.0]);
}

#[test]
fn increasing_monotone_property() {
    // After fixup every consecutive delta must be non-negative.
    let out = fixup_hue_increasing(&[10.0, 350.0, 5.0, 200.0, 100.0]);
    for w in out.windows(2) {
        assert!(w[1] >= w[0], "non-monotone increasing output: {out:?}");
    }
}

#[test]
fn increasing_nan_breaks_chain() {
    // culori reference (with `undefined`):
    //   fixupHueIncreasing([10, undefined, 350]) === [10, undefined, 350]
    assert_eq_with_nan(
        &fixup_hue_increasing(&[10.0, f64::NAN, 350.0]),
        &[10.0, f64::NAN, 350.0],
    );
}

// --- fixup_hue_decreasing -----------------------------------------------

#[test]
fn decreasing_culori_fixture() {
    // culori: fixupHueDecreasing([0, 340, 30, 0, 170]) === [0, -20, -330, -360, -550]
    assert_eq!(
        fixup_hue_decreasing(&[0.0, 340.0, 30.0, 0.0, 170.0]),
        vec![0.0, -20.0, -330.0, -360.0, -550.0]
    );
}

#[test]
fn decreasing_repeated_values_no_jump() {
    // delta == 0 is non-positive, so it stays 0 (no -360 shift).
    assert_eq!(fixup_hue_decreasing(&[90.0, 90.0]), vec![90.0, 90.0]);
}

#[test]
fn decreasing_monotone_property() {
    // After fixup every consecutive delta must be non-positive.
    let out = fixup_hue_decreasing(&[10.0, 350.0, 5.0, 200.0, 100.0]);
    for w in out.windows(2) {
        assert!(w[1] <= w[0], "non-monotone decreasing output: {out:?}");
    }
}

#[test]
fn decreasing_nan_breaks_chain() {
    // culori reference (with `undefined`):
    //   fixupHueDecreasing([10, undefined, 350]) === [10, undefined, 350]
    assert_eq_with_nan(
        &fixup_hue_decreasing(&[10.0, f64::NAN, 350.0]),
        &[10.0, f64::NAN, 350.0],
    );
}

// --- Strategies agree at length-0 and length-1 ---------------------------

#[test]
fn all_strategies_match_on_trivial_inputs() {
    for input in [vec![], vec![0.0], vec![123.456]] {
        let s = fixup_hue_shorter(&input);
        let l = fixup_hue_longer(&input);
        let i = fixup_hue_increasing(&input);
        let d = fixup_hue_decreasing(&input);
        assert_eq!(s, l);
        assert_eq!(l, i);
        assert_eq!(i, d);
    }
}
