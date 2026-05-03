//! Ported tests for `wcag_luminance` and `wcag_contrast` mirroring
//! `node_modules/culori/test/wcag.test.js`. The broader WCAG suite lives in
//! `tests/ported_contrast.rs`; this file pins the three specific assertions
//! culori's upstream test exercises that the broader suite does not state
//! verbatim.

use culors::{parse, wcag_contrast, wcag_luminance};

#[test]
fn luminance_hex_999() {
    // culori: wcagLuminance('#999') === 0.31854677812509186.
    let c = parse("#999").unwrap();
    assert_eq!(wcag_luminance(&c), 0.31854677812509186);
}

#[test]
fn contrast_white_black_symmetric() {
    let white = parse("white").unwrap();
    let black = parse("black").unwrap();
    assert_eq!(wcag_contrast(&black, &white), 21.0);
    assert_eq!(wcag_contrast(&white, &black), 21.0);
}

#[test]
fn contrast_self_pair_is_one() {
    // culori: wcagContrast('red', 'red') === 1.
    let red = parse("red").unwrap();
    assert_eq!(wcag_contrast(&red, &red), 1.0);
}
