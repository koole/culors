//! Ported tests for [`culor::samples`], matching culori 4.0.2's
//! `samples(n)` with default linear gamma.

use culor::samples;

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
