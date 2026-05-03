//! Ported tests for the `Hwb` color space.
//!
//! Reference values produced by culori 4.0.2 and pasted verbatim. HWB is
//! `{h, w, b}` where `b` is "blackness", not blue.

use culor::spaces::{Hsv, Hwb, Rgb};
use culor::ColorSpace;

#[path = "common/mod.rs"]
mod common;

const EPS: f64 = 1e-12;
const CHAIN_EPS: f64 = 1e-10;

#[test]
fn hwb_channels_excludes_alpha() {
    assert_eq!(Hwb::CHANNELS, &["h", "w", "b"]);
    assert_eq!(Hwb::CHANNELS.len(), 3);
    assert_eq!(Hwb::MODE, "hwb");
}

#[test]
fn rgb_red_to_hwb() {
    // c.hwb({mode:'rgb', r:1, g:0, b:0})
    // -> {"w":0,"b":0,"h":0}
    let rgb = Rgb {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        alpha: None,
    };
    let hwb: Hwb = rgb.into();
    common::assert_close(hwb.h, 0.0, EPS);
    common::assert_close(hwb.w, 0.0, EPS);
    common::assert_close(hwb.b, 0.0, EPS);
}

#[test]
fn rgb_green_to_hwb() {
    let rgb = Rgb {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        alpha: None,
    };
    let hwb: Hwb = rgb.into();
    common::assert_close(hwb.h, 120.0, EPS);
    common::assert_close(hwb.w, 0.0, EPS);
    common::assert_close(hwb.b, 0.0, EPS);
}

#[test]
fn rgb_blue_to_hwb() {
    let rgb = Rgb {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        alpha: None,
    };
    let hwb: Hwb = rgb.into();
    common::assert_close(hwb.h, 240.0, EPS);
    common::assert_close(hwb.w, 0.0, EPS);
    common::assert_close(hwb.b, 0.0, EPS);
}

#[test]
fn rgb_white_to_hwb() {
    // c.hwb({mode:'rgb', r:1, g:1, b:1})
    // -> {"w":1,"b":0} with h omitted (NaN sentinel here).
    let rgb = Rgb {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        alpha: None,
    };
    let hwb: Hwb = rgb.into();
    assert!(hwb.h.is_nan(), "expected NaN hue for white, got {}", hwb.h);
    common::assert_close(hwb.w, 1.0, EPS);
    common::assert_close(hwb.b, 0.0, EPS);
}

#[test]
fn rgb_black_to_hwb() {
    let rgb = Rgb {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        alpha: None,
    };
    let hwb: Hwb = rgb.into();
    assert!(hwb.h.is_nan());
    common::assert_close(hwb.w, 0.0, EPS);
    common::assert_close(hwb.b, 1.0, EPS);
}

#[test]
fn rgb_grey_to_hwb() {
    let rgb = Rgb {
        r: 0.5,
        g: 0.5,
        b: 0.5,
        alpha: None,
    };
    let hwb: Hwb = rgb.into();
    assert!(hwb.h.is_nan());
    common::assert_close(hwb.w, 0.5, EPS);
    common::assert_close(hwb.b, 0.5, EPS);
}

#[test]
fn rgb_arbitrary_to_hwb() {
    // c.hwb({mode:'rgb', r:0.3, g:0.6, b:0.9})
    // -> {"w":0.29999999999999993, "b":0.09999999999999998, "h":210}
    let rgb = Rgb {
        r: 0.3,
        g: 0.6,
        b: 0.9,
        alpha: None,
    };
    let hwb: Hwb = rgb.into();
    common::assert_close(hwb.h, 210.0, EPS);
    common::assert_close(hwb.w, 0.29999999999999993, EPS);
    common::assert_close(hwb.b, 0.09999999999999998, EPS);
}

#[test]
fn hwb_red_to_rgb() {
    // c.rgb({mode:'hwb', h:0, w:0, b:0})
    let hwb = Hwb {
        h: 0.0,
        w: 0.0,
        b: 0.0,
        alpha: None,
    };
    let rgb: Rgb = hwb.into();
    common::assert_close(rgb.r, 1.0, EPS);
    common::assert_close(rgb.g, 0.0, EPS);
    common::assert_close(rgb.b, 0.0, EPS);
}

#[test]
fn hwb_white_to_rgb() {
    let hwb = Hwb {
        h: 0.0,
        w: 1.0,
        b: 0.0,
        alpha: None,
    };
    let rgb: Rgb = hwb.into();
    common::assert_close(rgb.r, 1.0, EPS);
    common::assert_close(rgb.g, 1.0, EPS);
    common::assert_close(rgb.b, 1.0, EPS);
}

#[test]
fn hwb_black_to_rgb() {
    let hwb = Hwb {
        h: 0.0,
        w: 0.0,
        b: 1.0,
        alpha: None,
    };
    let rgb: Rgb = hwb.into();
    common::assert_close(rgb.r, 0.0, EPS);
    common::assert_close(rgb.g, 0.0, EPS);
    common::assert_close(rgb.b, 0.0, EPS);
}

#[test]
fn hwb_grey_to_rgb() {
    // c.rgb({mode:'hwb', h:0, w:0.5, b:0.5}) -> {r:0.5,g:0.5,b:0.5}
    let hwb = Hwb {
        h: 0.0,
        w: 0.5,
        b: 0.5,
        alpha: None,
    };
    let rgb: Rgb = hwb.into();
    common::assert_close(rgb.r, 0.5, EPS);
    common::assert_close(rgb.g, 0.5, EPS);
    common::assert_close(rgb.b, 0.5, EPS);
}

#[test]
fn hwb_w_b_sum_gt_1_normalizes() {
    // c.rgb({mode:'hwb', h:0, w:0.6, b:0.6}) -> {r:0.5,g:0.5,b:0.5}
    let hwb = Hwb {
        h: 0.0,
        w: 0.6,
        b: 0.6,
        alpha: None,
    };
    let rgb: Rgb = hwb.into();
    common::assert_close(rgb.r, 0.5, EPS);
    common::assert_close(rgb.g, 0.5, EPS);
    common::assert_close(rgb.b, 0.5, EPS);
}

#[test]
fn hwb_arbitrary_to_rgb() {
    // c.rgb({mode:'hwb', h:120, w:0.2, b:0.3})
    // -> {r:0.20000000000000007, g:0.7, b:0.20000000000000007}
    let hwb = Hwb {
        h: 120.0,
        w: 0.2,
        b: 0.3,
        alpha: None,
    };
    let rgb: Rgb = hwb.into();
    common::assert_close(rgb.r, 0.20000000000000007, EPS);
    common::assert_close(rgb.g, 0.7, EPS);
    common::assert_close(rgb.b, 0.20000000000000007, EPS);
}

#[test]
fn hwb_nan_hue_treated_as_zero() {
    // c.rgb({mode:'hwb', w:0.3, b:0.2}) -> r=0.8, g=0.3, b=0.3
    let hwb = Hwb {
        h: f64::NAN,
        w: 0.3,
        b: 0.2,
        alpha: None,
    };
    let rgb: Rgb = hwb.into();
    common::assert_close(rgb.r, 0.8, EPS);
    common::assert_close(rgb.g, 0.30000000000000004, EPS);
    common::assert_close(rgb.b, 0.30000000000000004, EPS);
}

#[test]
fn hsv_to_hwb_arbitrary() {
    // c.hwb({mode:'hsv', h:30, s:0.4, v:0.7})
    // -> {"w":0.42,"b":0.30000000000000004,"h":29.999999999999993}
    let hsv = Hsv {
        h: 30.0,
        s: 0.4,
        v: 0.7,
        alpha: None,
    };
    let hwb: Hwb = hsv.into();
    common::assert_close(hwb.h, 30.0, EPS);
    common::assert_close(hwb.w, 0.42, EPS);
    common::assert_close(hwb.b, 0.30000000000000004, EPS);
}

#[test]
fn hwb_to_hsv_arbitrary() {
    // c.hsv({mode:'hwb', h:30, w:0.42, b:0.3})
    // -> {"s":0.4,"v":0.7,"h":29.999999999999993}
    let hwb = Hwb {
        h: 30.0,
        w: 0.42,
        b: 0.3,
        alpha: None,
    };
    let hsv: Hsv = hwb.into();
    common::assert_close(hsv.h, 30.0, EPS);
    common::assert_close(hsv.s, 0.4, EPS);
    common::assert_close(hsv.v, 0.7, EPS);
}

#[test]
fn hsv_grey_to_hwb_propagates_nan() {
    let hsv = Hsv {
        h: f64::NAN,
        s: 0.0,
        v: 0.5,
        alpha: None,
    };
    let hwb: Hwb = hsv.into();
    assert!(hwb.h.is_nan());
    common::assert_close(hwb.w, 0.5, EPS);
    common::assert_close(hwb.b, 0.5, EPS);
}

#[test]
fn hwb_alpha_preserved() {
    let hwb = Hwb {
        h: 30.0,
        w: 0.2,
        b: 0.3,
        alpha: Some(0.5),
    };
    let hsv: Hsv = hwb.into();
    assert_eq!(hsv.alpha, Some(0.5));
    let back: Hwb = hsv.into();
    assert_eq!(back.alpha, Some(0.5));
}

#[test]
fn hwb_to_xyz65_matches_hsv_path() {
    let hwb = Hwb {
        h: 210.0,
        w: 0.3,
        b: 0.1,
        alpha: None,
    };
    let xyz = hwb.to_xyz65();
    let expected = Hsv::from(hwb).to_xyz65();
    common::assert_close(xyz.x, expected.x, EPS);
    common::assert_close(xyz.y, expected.y, EPS);
    common::assert_close(xyz.z, expected.z, EPS);
}

#[test]
fn rgb_round_trip_through_hwb() {
    let rgb = Rgb {
        r: 0.3,
        g: 0.6,
        b: 0.9,
        alpha: None,
    };
    let hwb: Hwb = rgb.into();
    let back: Rgb = hwb.into();
    common::assert_close(back.r, 0.29999999999999993, CHAIN_EPS);
    common::assert_close(back.g, 0.6, CHAIN_EPS);
    common::assert_close(back.b, 0.9, CHAIN_EPS);
}

#[test]
fn xyz65_round_trip_through_hwb() {
    let rgb = Rgb {
        r: 0.4,
        g: 0.6,
        b: 0.7,
        alpha: None,
    };
    let xyz = rgb.to_xyz65();
    let hwb = Hwb::from_xyz65(xyz);
    let back = hwb.to_xyz65();
    common::assert_close(back.x, xyz.x, CHAIN_EPS);
    common::assert_close(back.y, xyz.y, CHAIN_EPS);
    common::assert_close(back.z, xyz.z, CHAIN_EPS);
}
