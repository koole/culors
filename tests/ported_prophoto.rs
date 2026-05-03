//! Ported tests for the ProPhoto RGB color space.
//!
//! Native conversion is to XYZ D50; the ColorSpace impl routes through
//! XYZ D65 via Bradford adaptation.

use culor::spaces::{ProphotoRgb, Xyz50, Xyz65};
use culor::ColorSpace;

#[path = "common/mod.rs"]
mod common;

const EPS: f64 = 1e-12;

#[test]
fn prophoto_metadata() {
    assert_eq!(ProphotoRgb::CHANNELS, &["r", "g", "b"]);
    assert_eq!(ProphotoRgb::MODE, "prophoto");
}

#[test]
fn prophoto_white_to_xyz50() {
    let c = ProphotoRgb {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        alpha: None,
    };
    let xyz = c.to_xyz50();
    // c.xyz50({mode:'prophoto', r:1, g:1, b:1})
    common::assert_close(xyz.x, 0.9642956764295676, EPS);
    common::assert_close(xyz.y, 1.0, EPS);
    common::assert_close(xyz.z, 0.8251046025104602, EPS);
}

#[test]
fn prophoto_red_primary_to_xyz50() {
    let c = ProphotoRgb {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        alpha: None,
    };
    let xyz = c.to_xyz50();
    common::assert_close(xyz.x, 0.7977666449006423, EPS);
    common::assert_close(xyz.y, 0.2880748288194013, EPS);
    common::assert_close(xyz.z, 0.0, 1e-15);
}

#[test]
fn prophoto_green_primary_to_xyz50() {
    let c = ProphotoRgb {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        alpha: None,
    };
    let xyz = c.to_xyz50();
    common::assert_close(xyz.x, 0.1351812974005331, EPS);
    common::assert_close(xyz.y, 0.7118352342418731, EPS);
    common::assert_close(xyz.z, 0.0, 1e-15);
}

#[test]
fn prophoto_blue_primary_to_xyz50() {
    let c = ProphotoRgb {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        alpha: None,
    };
    let xyz = c.to_xyz50();
    common::assert_close(xyz.x, 0.0313477341283922, EPS);
    common::assert_close(xyz.y, 0.0000899369387256, EPS);
    common::assert_close(xyz.z, 0.8251046025104602, EPS);
}

#[test]
fn prophoto_arbitrary_above_threshold_to_xyz50() {
    let c = ProphotoRgb {
        r: 0.25,
        g: 0.4,
        b: 0.75,
        alpha: None,
    };
    let xyz = c.to_xyz50();
    common::assert_close(xyz.x, 0.11044774230839083, EPS);
    common::assert_close(xyz.y, 0.16061132995499883, EPS);
    common::assert_close(xyz.z, 0.491608387315228, EPS);
}

#[test]
fn prophoto_below_threshold_uses_linear_segment() {
    // 0.01 < 16/512 (0.03125): linear segment with slope 1/16.
    let c = ProphotoRgb {
        r: 0.01,
        g: 0.01,
        b: 0.01,
        alpha: None,
    };
    let xyz = c.to_xyz50();
    common::assert_close(xyz.x, 0.0006026847977684799, EPS);
    common::assert_close(xyz.y, 0.0006249999999999999, EPS);
    common::assert_close(xyz.z, 0.0005156903765690376, EPS);
}

#[test]
fn prophoto_routes_through_xyz65_via_bradford() {
    // c.xyz65({mode:'prophoto', r:0.25, g:0.4, b:0.75})
    let c = ProphotoRgb {
        r: 0.25,
        g: 0.4,
        b: 0.75,
        alpha: None,
    };
    let xyz = c.to_xyz65();
    common::assert_close(xyz.x, 0.1329188056729154, 1e-10);
    common::assert_close(xyz.y, 0.1694274718877816, 1e-10);
    common::assert_close(xyz.z, 0.6520853379217711, 1e-10);
}

#[test]
fn prophoto_round_trip_through_xyz50() {
    let c = ProphotoRgb {
        r: 0.25,
        g: 0.4,
        b: 0.75,
        alpha: None,
    };
    let back = ProphotoRgb::from_xyz50(c.to_xyz50());
    common::assert_close(back.r, 0.25, 1e-10);
    common::assert_close(back.g, 0.4, 1e-10);
    common::assert_close(back.b, 0.75, 1e-10);
}

#[test]
fn prophoto_round_trip_through_xyz65() {
    let c = ProphotoRgb {
        r: 0.25,
        g: 0.4,
        b: 0.75,
        alpha: None,
    };
    let back = ProphotoRgb::from_xyz65(c.to_xyz65());
    common::assert_close(back.r, 0.25, 1e-6);
    common::assert_close(back.g, 0.4, 1e-6);
    common::assert_close(back.b, 0.75, 1e-6);
}

#[test]
fn prophoto_alpha_preserved() {
    let c = ProphotoRgb {
        r: 0.5,
        g: 0.5,
        b: 0.5,
        alpha: Some(0.7),
    };
    let xyz = c.to_xyz65();
    assert_eq!(xyz.alpha, Some(0.7));
    let back = ProphotoRgb::from_xyz65(xyz);
    assert_eq!(back.alpha, Some(0.7));
    // Also verify D50 path preserves alpha.
    let xyz50 = c.to_xyz50();
    assert_eq!(xyz50.alpha, Some(0.7));
    let _: Xyz65 = xyz; // silence unused
    let _: Xyz50 = xyz50;
}
