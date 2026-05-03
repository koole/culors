//! Chromatic adaptation cross-validation, mirroring
//! `node_modules/culori/test/cat.test.js`.
//!
//! The two paths `rgb → xyz50` and `rgb → xyz65 → xyz50` must produce
//! identical XYZ-D50 values; likewise `rgb → xyz50 → rgb` must agree with
//! `rgb → xyz50 → xyz65 → rgb`. culori asserts agreement to 1e-14; culors
//! routes both paths through the same Bradford matrix so the comparison is
//! exact.

use culors::convert;
use culors::spaces::{Rgb, Xyz50, Xyz65};
use culors::{parse, Color};

const E: f64 = 1e-14;

fn rgb_of(name: &str) -> Rgb {
    match parse(name).unwrap_or_else(|| panic!("{name} parses")) {
        Color::Rgb(r) => r,
        other => match other.convert_to("rgb").expect("rgb is a known mode") {
            Color::Rgb(r) => r,
            _ => unreachable!(),
        },
    }
}

fn close(a: f64, b: f64, label: &str) {
    let diff = (a - b).abs();
    assert!(diff <= E, "{label}: diff {diff:.3e}");
}

const NAMED: &[&str] = &["red", "green", "blue", "white", "black", "magenta", "tomato"];

#[test]
fn rgb_to_xyz50_matches_via_xyz65() {
    for name in NAMED {
        let rgb = rgb_of(name);
        let direct: Xyz50 = convert(rgb);
        let via_65: Xyz65 = convert(rgb);
        let composed: Xyz50 = convert(via_65);
        close(direct.x, composed.x, &format!("{name}.x"));
        close(direct.y, composed.y, &format!("{name}.y"));
        close(direct.z, composed.z, &format!("{name}.z"));
    }
}

#[test]
fn rgb_xyz50_rgb_matches_via_xyz65() {
    for name in NAMED {
        let rgb = rgb_of(name);
        let xyz50: Xyz50 = convert(rgb);
        let direct_back: Rgb = convert(xyz50);
        let via_65: Xyz65 = convert(xyz50);
        let composed_back: Rgb = convert(via_65);
        close(direct_back.r, composed_back.r, &format!("{name}.r"));
        close(direct_back.g, composed_back.g, &format!("{name}.g"));
        close(direct_back.b, composed_back.b, &format!("{name}.b"));
    }
}
