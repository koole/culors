//! Tests for `fixup_alpha`, ported from culori 4.0.2's
//! `test/fixupAlpha.test.js`.
//!
//! culori reference (each case verified with `node --experimental-vm-modules
//! node_modules/.bin/mocha test/fixupAlpha.test.js`):
//!
//! ```text
//! fixupAlpha([undefined, 0, undefined])    -> [1, 0, 1]
//! fixupAlpha([undefined, undefined, undefined]) -> [undefined, undefined, undefined]
//! ```
//!
//! culors uses `f64::NAN` as the "missing" marker (the typed equivalent of
//! culori's `undefined`).

use culors::fixup_alpha;

fn nan_eq(a: f64, b: f64) -> bool {
    if a.is_nan() && b.is_nan() {
        return true;
    }
    a == b
}

fn assert_alpha_slice(actual: &[f64], expected: &[f64]) {
    assert_eq!(
        actual.len(),
        expected.len(),
        "lengths differ: {actual:?} vs {expected:?}"
    );
    for (i, (a, e)) in actual.iter().zip(expected.iter()).enumerate() {
        assert!(
            nan_eq(*a, *e),
            "index {i}: got {a}, expected {e} (full: {actual:?} vs {expected:?})"
        );
    }
}

#[test]
fn fixup_alpha_some_defined() {
    // culori: `fixupAlpha([undefined, 0, undefined]) -> [1, 0, 1]`
    let out = fixup_alpha(&[f64::NAN, 0.0, f64::NAN]);
    assert_alpha_slice(&out, &[1.0, 0.0, 1.0]);
}

#[test]
fn fixup_alpha_all_undefined() {
    // culori: `fixupAlpha([undefined, undefined, undefined])` returns the
    // input unchanged. We mirror that with NaN-pass-through.
    let out = fixup_alpha(&[f64::NAN, f64::NAN, f64::NAN]);
    assert_alpha_slice(&out, &[f64::NAN, f64::NAN, f64::NAN]);
}

// Additional cases that lock down the same behavior at the boundaries.
// These mirror culori's contract even though `test/fixupAlpha.test.js`
// covers only the two cases above.

#[test]
fn fixup_alpha_all_defined_passes_through() {
    let out = fixup_alpha(&[0.25, 0.5, 1.0]);
    assert_alpha_slice(&out, &[0.25, 0.5, 1.0]);
}

#[test]
fn fixup_alpha_single_defined_first() {
    let out = fixup_alpha(&[0.5, f64::NAN, f64::NAN]);
    assert_alpha_slice(&out, &[0.5, 1.0, 1.0]);
}

#[test]
fn fixup_alpha_single_defined_last() {
    let out = fixup_alpha(&[f64::NAN, f64::NAN, 0.7]);
    assert_alpha_slice(&out, &[1.0, 1.0, 0.7]);
}

#[test]
fn fixup_alpha_empty() {
    let out = fixup_alpha(&[]);
    assert!(out.is_empty());
}
