//! Ported tests for [`culors::samples`] / [`culors::samples_with_easing`],
//! matching culori 4.0.2's `samples(n, γ)`.
//!
//! Spot checks taken from `node -e`:
//!
//! ```js
//! samples(11, 2)   === [0, 0.01, 0.04, 0.09, 0.16, 0.25, 0.36, 0.49, 0.64, 0.81, 1]
//! samples(5, 2.2)  === [0, 0.04736…, 0.21763…, 0.53104…, 1]
//! samples(1, 2)    === [0.25]
//! ```

use culors::{easing_gamma, easing_smoothstep, samples, samples_with_easing};

#[track_caller]
fn assert_close_slice(label: &str, got: &[f64], expected: &[f64]) {
    assert_eq!(got.len(), expected.len(), "{label}: length mismatch");
    for (i, (g, e)) in got.iter().zip(expected.iter()).enumerate() {
        let diff = (g - e).abs();
        assert!(
            diff < 1e-15,
            "{label}: element {i} differs by {diff}: got {g}, expected {e}"
        );
    }
}

#[test]
fn samples_zero() {
    let got = samples(0);
    assert!(got.is_empty(), "n=0 should be empty");
}

#[test]
fn samples_one() {
    // culori returns [ease(0.5)] for n=1 with γ=1 → [0.5].
    let got = samples(1);
    assert_close_slice("n=1", &got, &[0.5]);
}

#[test]
fn samples_two_endpoints() {
    let got = samples(2);
    assert_close_slice("n=2", &got, &[0.0, 1.0]);
}

#[test]
fn samples_eleven_matches_culori() {
    // node -e "import('culori').then(c=>console.log(c.samples(11)))"
    let got = samples(11);
    assert_close_slice(
        "n=11",
        &got,
        &[0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0],
    );
}

#[test]
fn samples_hundred_endpoints_exact() {
    let got = samples(100);
    assert_eq!(got.len(), 100);
    assert_eq!(got[0], 0.0);
    assert_eq!(got[99], 1.0);
}

#[test]
fn samples_with_gamma_two() {
    // culori: samples(11, 2) → squares of [0, 0.1, 0.2, …, 1.0].
    let got = samples_with_easing(11, easing_gamma(2.0));
    let expected = [
        0.0,
        0.010_000_000_000_000_002,
        0.040_000_000_000_000_01,
        0.09,
        0.160_000_000_000_000_03,
        0.25,
        0.36,
        0.489_999_999_999_999_94,
        0.640_000_000_000_000_1,
        0.81,
        1.0,
    ];
    assert_close_slice("γ=2", &got, &expected);
}

#[test]
fn samples_with_gamma_2_2() {
    let got = samples_with_easing(5, easing_gamma(2.2));
    let expected = [
        0.0,
        0.047_366_142_703_449_93,
        0.217_637_640_824_031,
        0.531_049_225_103_382_4,
        1.0,
    ];
    assert_close_slice("γ=2.2", &got, &expected);
}

#[test]
fn samples_with_gamma_n_one() {
    // culori: n=1 returns [ease(0.5)]; with γ=2 that's [0.25].
    let got = samples_with_easing(1, easing_gamma(2.0));
    assert_close_slice("γ=2 n=1", &got, &[0.25]);
}

#[test]
fn samples_with_smoothstep_yields_s_curve() {
    // smoothstep(t) = t² * (3 - 2t); endpoints stay pinned at 0 and 1, the
    // midpoint stays at 0.5, and t=0.25 lands at 0.15625.
    let got = samples_with_easing(5, easing_smoothstep());
    assert_close_slice("smoothstep n=5", &got, &[0.0, 0.15625, 0.5, 0.84375, 1.0]);
}
