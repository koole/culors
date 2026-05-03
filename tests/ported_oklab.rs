//! Ported tests for the `Oklab` color space.
//!
//! Reference values produced by culori 4.0.2 and pasted verbatim. Oklab is
//! defined relative to linear sRGB, not gamma-encoded sRGB.

use culors::spaces::{LinearRgb, Oklab, Oklch, Rgb, Xyz65};
use culors::ColorSpace;

#[path = "common/mod.rs"]
mod common;

const EPS: f64 = 1e-12;
const CHAIN_EPS: f64 = 1e-10;

#[test]
fn oklab_channels_excludes_alpha() {
    assert_eq!(Oklab::CHANNELS, &["l", "a", "b"]);
    assert_eq!(Oklab::CHANNELS.len(), 3);
    assert_eq!(Oklab::MODE, "oklab");
}

#[test]
fn lrgb_white_to_oklab() {
    // c.oklab({mode:'lrgb', r:1, g:1, b:1})
    // -> {"l":1.0000000000000002,"a":-4.996003610813204e-16,"b":0}
    let lrgb = LinearRgb {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        alpha: None,
    };
    let oklab = Oklab::from(lrgb);
    common::assert_close(oklab.l, 1.0000000000000002, EPS);
    common::assert_close(oklab.a, -4.996003610813204e-16, EPS);
    common::assert_close(oklab.b, 0.0, EPS);
}

#[test]
fn lrgb_red_to_oklab() {
    // c.oklab({mode:'lrgb', r:1, g:0, b:0})
    // matches sRGB red since lrgb=rgb=1 for red.
    let lrgb = LinearRgb {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        alpha: None,
    };
    let oklab = Oklab::from(lrgb);
    common::assert_close(oklab.l, 0.6279553639214311, EPS);
    common::assert_close(oklab.a, 0.22486306842627443, EPS);
    common::assert_close(oklab.b, 0.12584627733058495, EPS);
}

#[test]
fn lrgb_black_to_oklab() {
    let lrgb = LinearRgb {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        alpha: None,
    };
    let oklab = Oklab::from(lrgb);
    common::assert_close(oklab.l, 0.0, EPS);
    common::assert_close(oklab.a, 0.0, EPS);
    common::assert_close(oklab.b, 0.0, EPS);
}

#[test]
fn srgb_red_to_oklab_via_xyz65() {
    // c.oklab({mode:'rgb', r:1, g:0, b:0})
    let rgb = Rgb {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        alpha: None,
    };
    let oklab = Oklab::from_xyz65(rgb.to_xyz65());
    common::assert_close(oklab.l, 0.6279553639214311, CHAIN_EPS);
    common::assert_close(oklab.a, 0.22486306842627443, CHAIN_EPS);
    common::assert_close(oklab.b, 0.12584627733058495, CHAIN_EPS);
}

#[test]
fn srgb_green_to_oklab_via_xyz65() {
    // c.oklab({mode:'rgb', r:0, g:1, b:0})
    let rgb = Rgb {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        alpha: None,
    };
    let oklab = Oklab::from_xyz65(rgb.to_xyz65());
    common::assert_close(oklab.l, 0.8664396175234368, CHAIN_EPS);
    common::assert_close(oklab.a, -0.2338875809365577, CHAIN_EPS);
    common::assert_close(oklab.b, 0.1794984451609376, CHAIN_EPS);
}

#[test]
fn srgb_blue_to_oklab_via_xyz65() {
    // c.oklab({mode:'rgb', r:0, g:0, b:1})
    let rgb = Rgb {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        alpha: None,
    };
    let oklab = Oklab::from_xyz65(rgb.to_xyz65());
    common::assert_close(oklab.l, 0.45201371817442365, CHAIN_EPS);
    common::assert_close(oklab.a, -0.032456975170797764, CHAIN_EPS);
    common::assert_close(oklab.b, -0.31152816567757763, CHAIN_EPS);
}

#[test]
fn srgb_grey_is_achromatic() {
    // c.oklab({mode:'rgb', r:0.5, g:0.5, b:0.5})
    // -> {"l":0.5981807305268477,"a":0,"b":0}
    let rgb = Rgb {
        r: 0.5,
        g: 0.5,
        b: 0.5,
        alpha: None,
    };
    let oklab = Oklab::from_xyz65(rgb.to_xyz65());
    common::assert_close(oklab.l, 0.5981807305268477, CHAIN_EPS);
    common::assert_close(oklab.a, 0.0, CHAIN_EPS);
    common::assert_close(oklab.b, 0.0, CHAIN_EPS);
}

#[test]
fn oklab_to_lrgb_arbitrary() {
    // c.lrgb({mode:'oklab', l:0.5, a:0.1, b:-0.1})
    // -> {"r":0.21870561272089278, "g":0.060342338545462296, "b":0.32103792558352606}
    let oklab = Oklab {
        l: 0.5,
        a: 0.1,
        b: -0.1,
        alpha: None,
    };
    let lrgb = LinearRgb::from(oklab);
    common::assert_close(lrgb.r, 0.21870561272089278, EPS);
    common::assert_close(lrgb.g, 0.060342338545462296, EPS);
    common::assert_close(lrgb.b, 0.32103792558352606, EPS);
}

#[test]
fn oklab_round_trip_through_lrgb() {
    let lrgb = LinearRgb {
        r: 0.2,
        g: 0.5,
        b: 0.8,
        alpha: None,
    };
    let oklab = Oklab::from(lrgb);
    let back = LinearRgb::from(oklab);
    common::assert_close(back.r, lrgb.r, CHAIN_EPS);
    common::assert_close(back.g, lrgb.g, CHAIN_EPS);
    common::assert_close(back.b, lrgb.b, CHAIN_EPS);
}

#[test]
fn oklab_alpha_preserved() {
    let oklab = Oklab {
        l: 0.5,
        a: 0.1,
        b: -0.1,
        alpha: Some(0.42),
    };
    let lrgb: LinearRgb = oklab.into();
    assert_eq!(lrgb.alpha, Some(0.42));
    let back: Oklab = lrgb.into();
    assert_eq!(back.alpha, Some(0.42));
}

#[test]
fn oklab_to_xyz65_matches_lrgb_path() {
    // Hub path should equal LinearRgb::from(self).to_xyz65() exactly.
    let oklab = Oklab {
        l: 0.6,
        a: -0.02,
        b: 0.05,
        alpha: None,
    };
    let xyz = oklab.to_xyz65();
    let expected = LinearRgb::from(oklab).to_xyz65();
    common::assert_close(xyz.x, expected.x, EPS);
    common::assert_close(xyz.y, expected.y, EPS);
    common::assert_close(xyz.z, expected.z, EPS);
}

#[test]
fn oklab_lrgb_out_of_gamut_round_trip() {
    // HDR-style input with negative channel; verify round-trip preserves
    // it. Oklab's cube-root nonlinearity is sign-preserving via f64::cbrt,
    // so negative LMS values survive the forward/inverse pair.
    let lrgb = LinearRgb {
        r: 1.5,
        g: -0.1,
        b: 0.0,
        alpha: None,
    };
    let oklab: Oklab = lrgb.into();
    let back: LinearRgb = oklab.into();
    common::assert_close(back.r, 1.5, 1e-12);
    common::assert_close(back.g, -0.1, 1e-12);
    common::assert_close(back.b, 0.0, 1e-12);
}

#[test]
fn oklab_hdr_lrgb_matches_culori() {
    // c.oklab({mode:'lrgb', r:1.5, g:-0.1, b:0.0})
    // -> {"l":0.6718402835881381,"a":0.31753813322383284,"b":0.1337485625307116}
    let lrgb = LinearRgb {
        r: 1.5,
        g: -0.1,
        b: 0.0,
        alpha: None,
    };
    let oklab: Oklab = lrgb.into();
    common::assert_close(oklab.l, 0.6718402835881381, 1e-12);
    common::assert_close(oklab.a, 0.31753813322383284, 1e-12);
    common::assert_close(oklab.b, 0.1337485625307116, 1e-12);
}

// ---- Achromatic fixup: Rgb -> Oklab/Oklch -------------------------------
//
// culori's `convertRgbToOklab.js` snaps `a` and `b` to exactly zero when
// the input is achromatic (r === g === b). The cube-root LMS chain leaves
// a residual on the order of 1e-16 in both opponent channels without the
// snap. Reference values come from culori 4.0.2's public `oklab()` and
// `oklch()` converters invoked on `{mode:'rgb', ...}` inputs.

#[test]
fn rgb_white_to_oklab_snaps_a_b_to_zero() {
    // c.oklab({mode:'rgb', r:1, g:1, b:1})
    // -> {"l":1.0000000000000002,"a":0,"b":0}
    let oklab: Oklab = Oklab::from(Rgb {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        alpha: None,
    });
    assert_eq!(oklab.a, 0.0);
    assert_eq!(oklab.b, 0.0);
    common::assert_close(oklab.l, 1.0000000000000002, EPS);
}

#[test]
fn rgb_grey_to_oklab_snaps_a_b_to_zero() {
    // c.oklab({mode:'rgb', r:0.5, g:0.5, b:0.5})
    // -> {"l":0.5981807305268477,"a":0,"b":0}
    let oklab: Oklab = Oklab::from(Rgb {
        r: 0.5,
        g: 0.5,
        b: 0.5,
        alpha: None,
    });
    assert_eq!(oklab.a, 0.0);
    assert_eq!(oklab.b, 0.0);
    common::assert_close(oklab.l, 0.5981807305268477, EPS);
}

#[test]
fn rgb_black_to_oklab_snaps_a_b_to_zero() {
    let oklab: Oklab = Oklab::from(Rgb {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        alpha: None,
    });
    assert_eq!(oklab.a, 0.0);
    assert_eq!(oklab.b, 0.0);
    common::assert_close(oklab.l, 0.0, EPS);
}

#[test]
fn rgb_white_to_oklch_has_nan_hue() {
    let oklch: Oklch = Oklch::from(Rgb {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        alpha: None,
    });
    assert_eq!(oklch.c, 0.0);
    assert!(oklch.h.is_nan());
    common::assert_close(oklch.l, 1.0000000000000002, EPS);
}

#[test]
fn rgb_chromatic_oklab_matches_culori() {
    // c.oklab({mode:'rgb', r:1, g:0, b:0})
    // -> {"l":0.6279553639214311,"a":0.22486306842627443,"b":0.12584627733058495}
    // Achromatic fixup must NOT trigger here (r != g).
    let oklab: Oklab = Oklab::from(Rgb {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        alpha: None,
    });
    common::assert_close(oklab.l, 0.6279553639214311, EPS);
    common::assert_close(oklab.a, 0.22486306842627443, EPS);
    common::assert_close(oklab.b, 0.12584627733058495, EPS);
}

#[test]
fn xyz65_round_trip_through_oklab() {
    let xyz = Xyz65 {
        x: 0.4,
        y: 0.5,
        z: 0.6,
        alpha: None,
    };
    let oklab = Oklab::from_xyz65(xyz);
    let back = oklab.to_xyz65();
    common::assert_close(back.x, xyz.x, CHAIN_EPS);
    common::assert_close(back.y, xyz.y, CHAIN_EPS);
    common::assert_close(back.z, xyz.z, CHAIN_EPS);
}
