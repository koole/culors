//! Ported tests for the `Hsl` color space.
//!
//! Reference values produced by culori 4.0.2 and pasted verbatim. culori
//! omits the `h` property for achromatic colors; we mirror that with NaN
//! since our struct stores `h` as `f64`.

use culors::spaces::{Hsl, Rgb, Xyz65};
use culors::ColorSpace;

#[path = "common/mod.rs"]
mod common;

const EPS: f64 = 1e-12;
const CHAIN_EPS: f64 = 1e-10;

#[test]
fn hsl_channels_excludes_alpha() {
    assert_eq!(Hsl::CHANNELS, &["h", "s", "l"]);
    assert_eq!(Hsl::CHANNELS.len(), 3);
    assert_eq!(Hsl::MODE, "hsl");
}

#[test]
fn rgb_red_to_hsl() {
    let rgb = Rgb {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        alpha: None,
    };
    let hsl: Hsl = rgb.into();
    common::assert_close(hsl.h, 0.0, EPS);
    common::assert_close(hsl.s, 1.0, EPS);
    common::assert_close(hsl.l, 0.5, EPS);
}

#[test]
fn rgb_green_to_hsl() {
    let rgb = Rgb {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        alpha: None,
    };
    let hsl: Hsl = rgb.into();
    common::assert_close(hsl.h, 120.0, EPS);
    common::assert_close(hsl.s, 1.0, EPS);
    common::assert_close(hsl.l, 0.5, EPS);
}

#[test]
fn rgb_blue_to_hsl() {
    let rgb = Rgb {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        alpha: None,
    };
    let hsl: Hsl = rgb.into();
    common::assert_close(hsl.h, 240.0, EPS);
    common::assert_close(hsl.s, 1.0, EPS);
    common::assert_close(hsl.l, 0.5, EPS);
}

#[test]
fn rgb_orange_to_hsl() {
    let rgb = Rgb {
        r: 1.0,
        g: 0.5,
        b: 0.25,
        alpha: None,
    };
    let hsl: Hsl = rgb.into();
    common::assert_close(hsl.h, 20.0, EPS);
    common::assert_close(hsl.s, 1.0, EPS);
    common::assert_close(hsl.l, 0.625, EPS);
}

#[test]
fn rgb_g_lt_b_wraps_hue() {
    // r=max, g<b — culori adds 360 via the `g < b` boolean.
    let rgb = Rgb {
        r: 1.0,
        g: 0.0,
        b: 0.5,
        alpha: None,
    };
    let hsl: Hsl = rgb.into();
    common::assert_close(hsl.h, 330.0, EPS);
    common::assert_close(hsl.s, 1.0, EPS);
    common::assert_close(hsl.l, 0.5, EPS);
}

#[test]
fn rgb_grey_produces_nan_hue() {
    // culori omits `h` when M === m. We set NaN.
    let rgb = Rgb {
        r: 0.5,
        g: 0.5,
        b: 0.5,
        alpha: None,
    };
    let hsl: Hsl = rgb.into();
    assert!(hsl.h.is_nan(), "expected NaN hue for grey, got {}", hsl.h);
    common::assert_close(hsl.s, 0.0, EPS);
    common::assert_close(hsl.l, 0.5, EPS);
}

#[test]
fn rgb_white_produces_nan_hue() {
    let rgb = Rgb {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        alpha: None,
    };
    let hsl: Hsl = rgb.into();
    assert!(hsl.h.is_nan());
    common::assert_close(hsl.s, 0.0, EPS);
    common::assert_close(hsl.l, 1.0, EPS);
}

#[test]
fn hsl_to_rgb_red() {
    let hsl = Hsl {
        h: 0.0,
        s: 1.0,
        l: 0.5,
        alpha: None,
    };
    let rgb: Rgb = hsl.into();
    common::assert_close(rgb.r, 1.0, EPS);
    common::assert_close(rgb.g, 0.0, EPS);
    common::assert_close(rgb.b, 0.0, EPS);
}

#[test]
fn hsl_to_rgb_yellow() {
    let hsl = Hsl {
        h: 60.0,
        s: 1.0,
        l: 0.5,
        alpha: None,
    };
    let rgb: Rgb = hsl.into();
    common::assert_close(rgb.r, 1.0, EPS);
    common::assert_close(rgb.g, 1.0, EPS);
    common::assert_close(rgb.b, 0.0, EPS);
}

#[test]
fn hsl_to_rgb_arbitrary() {
    let hsl = Hsl {
        h: 30.0,
        s: 0.4,
        l: 0.7,
        alpha: None,
    };
    let rgb: Rgb = hsl.into();
    common::assert_close(rgb.r, 0.82, EPS);
    common::assert_close(rgb.g, 0.7, EPS);
    common::assert_close(rgb.b, 0.58, EPS);
}

#[test]
fn hsl_negative_hue_normalizes() {
    let hsl = Hsl {
        h: -10.0,
        s: 1.0,
        l: 0.5,
        alpha: None,
    };
    let rgb: Rgb = hsl.into();
    common::assert_close(rgb.r, 1.0, EPS);
    common::assert_close(rgb.g, 0.0, EPS);
    common::assert_close(rgb.b, 0.16666666666666696, EPS);
}

#[test]
fn hsl_360_normalizes_to_zero() {
    let hsl = Hsl {
        h: 360.0,
        s: 1.0,
        l: 0.5,
        alpha: None,
    };
    let rgb: Rgb = hsl.into();
    common::assert_close(rgb.r, 1.0, EPS);
    common::assert_close(rgb.g, 0.0, EPS);
    common::assert_close(rgb.b, 0.0, EPS);
}

#[test]
fn hsl_nan_hue_treated_as_zero() {
    // culori treats undefined h as 0; we use NaN, and our impl must coerce
    // NaN to 0 before normalization to mirror culori's behavior on grey
    // colors that round-trip through Rgb.
    let hsl = Hsl {
        h: f64::NAN,
        s: 0.0,
        l: 0.5,
        alpha: None,
    };
    let rgb: Rgb = hsl.into();
    common::assert_close(rgb.r, 0.5, EPS);
    common::assert_close(rgb.g, 0.5, EPS);
    common::assert_close(rgb.b, 0.5, EPS);
}

#[test]
fn hsl_nan_hue_with_nonzero_s_coerces_to_zero() {
    // NaN hue coerces to 0 regardless of s, matching culori's `h !== undefined ? h : 0`.
    let nan_hsl = Hsl {
        h: f64::NAN,
        s: 0.5,
        l: 0.5,
        alpha: None,
    };
    let zero_hsl = Hsl {
        h: 0.0,
        s: 0.5,
        l: 0.5,
        alpha: None,
    };
    let nan_rgb: Rgb = nan_hsl.into();
    let zero_rgb: Rgb = zero_hsl.into();
    common::assert_close(nan_rgb.r, zero_rgb.r, EPS);
    common::assert_close(nan_rgb.g, zero_rgb.g, EPS);
    common::assert_close(nan_rgb.b, zero_rgb.b, EPS);
}

#[test]
fn rgb_round_trip_through_hsl() {
    let rgb = Rgb {
        r: 0.3,
        g: 0.6,
        b: 0.9,
        alpha: None,
    };
    let hsl: Hsl = rgb.into();
    common::assert_close(hsl.h, 210.0, EPS);
    common::assert_close(hsl.s, 0.7500000000000001, EPS);
    common::assert_close(hsl.l, 0.6, EPS);
    let back: Rgb = hsl.into();
    common::assert_close(back.r, 0.29999999999999993, EPS);
    common::assert_close(back.g, 0.6, EPS);
    common::assert_close(back.b, 0.9, EPS);
}

#[test]
fn hsl_alpha_preserved() {
    let hsl = Hsl {
        h: 30.0,
        s: 0.4,
        l: 0.7,
        alpha: Some(0.7),
    };
    let rgb: Rgb = hsl.into();
    assert_eq!(rgb.alpha, Some(0.7));
    let back: Hsl = rgb.into();
    assert_eq!(back.alpha, Some(0.7));
}

#[test]
fn hsl_to_xyz65() {
    // Composed via Hsl -> Rgb -> Xyz65.
    let hsl = Hsl {
        h: 30.0,
        s: 0.4,
        l: 0.7,
        alpha: None,
    };
    let xyz = hsl.to_xyz65();
    common::assert_close(xyz.x, 0.4767838997209961, CHAIN_EPS);
    common::assert_close(xyz.y, 0.4774584429539756, CHAIN_EPS);
    common::assert_close(xyz.z, 0.34680858900739076, CHAIN_EPS);
}

#[test]
fn hsl_from_xyz65_white() {
    // Matrix imprecision pushes this away from a clean grey, so culori
    // produces a slight saturation. We pin culori's actual output.
    let xyz = Xyz65 {
        x: 0.9504559270516715,
        y: 0.9999999999999999,
        z: 1.0890577507598784,
        alpha: None,
    };
    let hsl = Hsl::from_xyz65(xyz);
    common::assert_close(hsl.h, 60.0, CHAIN_EPS);
    common::assert_close(hsl.s, 0.5, CHAIN_EPS);
    common::assert_close(hsl.l, 0.9999999999999998, CHAIN_EPS);
}
