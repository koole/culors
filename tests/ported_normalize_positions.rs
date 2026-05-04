//! Ported tests for color-stop position normalization.
//!
//! Mirrors culori 4.0.2's `test/normalizePositions.test.js`. culori
//! exposes the helper as `util/normalizePositions.js`; culors promotes
//! it to the public surface as [`culors::normalize_positions`] for
//! callers building gradient pipelines.
//!
//! Each expected vector was produced by running the equivalent JS port
//! of culori's algorithm (see `node -e` snippets in the v1.6 release
//! notes). `f64::NAN` stands in for culori's `undefined`. Note that
//! culori's reference output for the partial fill `[0.2, _, _, 0.8]`
//! contains the literal floating-point value `0.6000000000000001`; we
//! match it bit-for-bit, not the algebraic ideal `0.6`.

use culors::normalize_positions;

// --- Direct culori fixtures ---------------------------------------------

#[test]
fn all_missing_spread_evenly() {
    // culori: normalize([undefined; 5]) === [0, 0.25, 0.5, 0.75, 1]
    let mut stops = [f64::NAN; 5];
    normalize_positions(&mut stops);
    assert_eq!(stops, [0.0, 0.25, 0.5, 0.75, 1.0]);
}

#[test]
fn partial_endpoints_anchor_fill() {
    // culori: normalize([0.2, undefined, undefined, 0.8])
    //   === [0.2, 0.4, 0.6000000000000001, 0.8]
    // The trailing 1ULP comes from the iterative `from + k * inc` build-
    // up; culori does the same and we must match its exact output.
    let mut stops = [0.2, f64::NAN, f64::NAN, 0.8];
    normalize_positions(&mut stops);
    assert_eq!(stops, [0.2, 0.4, 0.600_000_000_000_000_1, 0.8]);
}

// --- Rule 1: anchor first/last ------------------------------------------

#[test]
fn missing_first_anchors_to_zero() {
    let mut stops = [f64::NAN, f64::NAN, 0.8];
    normalize_positions(&mut stops);
    assert_eq!(stops, [0.0, 0.4, 0.8]);
}

#[test]
fn missing_last_anchors_to_one() {
    let mut stops = [0.5, f64::NAN, f64::NAN];
    normalize_positions(&mut stops);
    assert_eq!(stops, [0.5, 0.75, 1.0]);
}

#[test]
fn three_missing_anchored() {
    // [_, _, _] -> [0, 0.5, 1].
    let mut stops = [f64::NAN, f64::NAN, f64::NAN];
    normalize_positions(&mut stops);
    assert_eq!(stops, [0.0, 0.5, 1.0]);
}

#[test]
fn two_missing_become_zero_one() {
    let mut stops = [f64::NAN, f64::NAN];
    normalize_positions(&mut stops);
    assert_eq!(stops, [0.0, 1.0]);
}

// --- Rule 2: gap-fill semantics -----------------------------------------

#[test]
fn nested_gaps_fill_against_local_neighbours() {
    // [0, _, 0.4, _, _, 1] -> [0, 0.2, 0.4, 0.6, 0.8, 1]
    let mut stops = [0.0, f64::NAN, 0.4, f64::NAN, f64::NAN, 1.0];
    normalize_positions(&mut stops);
    assert_eq!(stops, [0.0, 0.2, 0.4, 0.6, 0.8, 1.0]);
}

#[test]
fn gap_fill_outside_unit_range_is_preserved() {
    // culori does not clamp into [0, 1]. Anchors outside the unit range
    // pull the gap fill outside too.
    // [-0.5, _, _, 1.5] -> [-0.5, 0.16666..., 0.83333..., 1.5]
    let mut stops = [-0.5, f64::NAN, f64::NAN, 1.5];
    normalize_positions(&mut stops);
    assert_eq!(stops[0], -0.5);
    assert_eq!(stops[3], 1.5);
    let expected_mid_lo = -0.5 + (1.5 - -0.5) / 3.0; // 0.16666...
    let expected_mid_hi = -0.5 + 2.0 * (1.5 - -0.5) / 3.0; // 0.83333...
    assert!((stops[1] - expected_mid_lo).abs() < 1e-12);
    assert!((stops[2] - expected_mid_hi).abs() < 1e-12);
}

// --- Rule 3: monotone clamping ------------------------------------------

#[test]
fn out_of_order_position_clamps_forward() {
    // culori: normalize([0.5, 0.3, 0.8]) === [0.5, 0.5, 0.8]
    let mut stops = [0.5, 0.3, 0.8];
    normalize_positions(&mut stops);
    assert_eq!(stops, [0.5, 0.5, 0.8]);
}

#[test]
fn long_decreasing_run_clamps_to_max_so_far() {
    // culori: normalize([0.5, 0.3, 0.4, 0.2, 0.6]) === [0.5, 0.5, 0.5, 0.5, 0.6]
    let mut stops = [0.5, 0.3, 0.4, 0.2, 0.6];
    normalize_positions(&mut stops);
    assert_eq!(stops, [0.5, 0.5, 0.5, 0.5, 0.6]);
}

#[test]
fn fully_defined_monotone_input_unchanged() {
    let mut stops = [0.0, 0.5, 1.0];
    normalize_positions(&mut stops);
    assert_eq!(stops, [0.0, 0.5, 1.0]);
}

// --- Edge cases ---------------------------------------------------------

#[test]
fn empty_slice_is_noop() {
    let mut stops: [f64; 0] = [];
    normalize_positions(&mut stops);
    assert_eq!(stops.len(), 0);
}

#[test]
fn single_missing_anchors_to_zero() {
    // First-element anchor wins for the single-element case.
    let mut stops = [f64::NAN];
    normalize_positions(&mut stops);
    assert_eq!(stops, [0.0]);
}

#[test]
fn single_defined_passes_through() {
    let mut stops = [0.5];
    normalize_positions(&mut stops);
    assert_eq!(stops, [0.5]);
}

#[test]
fn returns_same_slice_for_chaining() {
    let mut stops = [f64::NAN, f64::NAN];
    let out = normalize_positions(&mut stops);
    assert_eq!(out.as_ptr(), stops.as_ptr());
}
