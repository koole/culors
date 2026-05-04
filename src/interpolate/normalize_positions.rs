//! Color-stop position normalization.
//!
//! Mirrors culori 4.0.2's `util/normalizePositions.js`, which implements
//! the CSS Images Module 4 stop-fixup algorithm
//! (<https://drafts.csswg.org/css-images-4/#color-stop-fixup>):
//!
//! 1. Set the first position to `0` if missing, the last to `1` if missing.
//! 2. Spread runs of missing positions evenly between their defined
//!    neighbours.
//! 3. Clamp every position to be at least as large as the previous one.
//!
//! culors uses `NaN` as the "missing" marker (where culori uses
//! `undefined`). Pass [`f64::NAN`] for stops without an explicit position.
//!
//! culors's `interpolate` family always assigns evenly spaced positions
//! internally, so this helper is exposed for callers building gradient
//! pipelines who need the explicit-position semantics culori's
//! `interpolate(stops, mode)` accepts via `[color, position]` pairs.

/// Normalize a slice of stop positions in place using the CSS Images
/// Module 4 stop-fixup rules. Returns the same slice for chaining.
///
/// `f64::NAN` stands in for "missing" (`undefined` in culori). The slice
/// is mutated directly: defined positions are kept, missing positions are
/// filled, and out-of-order positions are pulled forward to the previous
/// one's value.
///
/// # Examples
///
/// All-missing positions are spread evenly across `[0, 1]`:
///
/// ```rust
/// use culors::normalize_positions;
/// let mut stops = [f64::NAN; 5];
/// normalize_positions(&mut stops);
/// assert_eq!(stops, [0.0, 0.25, 0.5, 0.75, 1.0]);
/// ```
///
/// Defined endpoints anchor the gap fill:
///
/// ```rust
/// use culors::normalize_positions;
/// let mut stops = [0.2, f64::NAN, f64::NAN, 0.8];
/// normalize_positions(&mut stops);
/// assert_eq!(stops, [0.2, 0.4, 0.6000000000000001, 0.8]);
/// ```
///
/// Out-of-order stops clamp forward (rule 3):
///
/// ```rust
/// use culors::normalize_positions;
/// let mut stops = [0.5, 0.3, 0.8];
/// normalize_positions(&mut stops);
/// assert_eq!(stops, [0.5, 0.5, 0.8]);
/// ```
pub fn normalize_positions(arr: &mut [f64]) -> &mut [f64] {
    let n = arr.len();
    if n == 0 {
        return arr;
    }

    // Rule 1: anchor first/last to 0 / 1 when missing.
    if arr[0].is_nan() {
        arr[0] = 0.0;
    }
    if arr[n - 1].is_nan() {
        arr[n - 1] = 1.0;
    }

    // The single-element list is fully defined after rule 1.
    if n == 1 {
        return arr;
    }

    let mut i = 1usize;
    while i < n {
        if arr[i].is_nan() {
            // Rule 2: spread the run of missing positions evenly between
            // arr[i - 1] (always defined here) and the next defined value.
            let from_idx = i;
            let from_pos = arr[i - 1];
            let mut j = i;
            while j < n && arr[j].is_nan() {
                j += 1;
            }
            // arr[n - 1] was anchored to 1.0 above, so j is guaranteed < n.
            debug_assert!(j < n, "tail must be defined after rule 1");
            let inc = (arr[j] - from_pos) / (j - i + 1) as f64;
            while i < j {
                arr[i] = from_pos + (i + 1 - from_idx) as f64 * inc;
                i += 1;
            }
        } else if arr[i] < arr[i - 1] {
            // Rule 3: clamp non-monotone positions forward.
            arr[i] = arr[i - 1];
        }
        i += 1;
    }
    arr
}
