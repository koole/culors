//! Shared test helpers.

#[track_caller]
pub fn assert_close(actual: f64, expected: f64, eps: f64) {
    if expected.is_nan() {
        assert!(actual.is_nan(), "expected NaN, got {actual}");
        return;
    }
    let diff = (actual - expected).abs();
    assert!(
        diff <= eps,
        "values differ by {diff} (> {eps}): actual={actual}, expected={expected}",
    );
}

#[track_caller]
pub fn assert_alpha_close(actual: Option<f64>, expected: Option<f64>, eps: f64) {
    match (actual, expected) {
        (None, None) => {}
        (Some(a), Some(e)) => assert_close(a, e, eps),
        (a, e) => panic!("alpha mismatch: actual={a:?}, expected={e:?}"),
    }
}
