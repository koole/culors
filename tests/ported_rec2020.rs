//! Ported tests for the Rec. 2020 color space.

use culor::spaces::Rec2020;
use culor::ColorSpace;

#[path = "common/mod.rs"]
mod common;

const EPS: f64 = 1e-12;

#[test]
fn rec2020_metadata() {
    assert_eq!(Rec2020::CHANNELS, &["r", "g", "b"]);
    assert_eq!(Rec2020::MODE, "rec2020");
}

#[test]
fn rec2020_white() {
    let c = Rec2020 {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        alpha: None,
    };
    let xyz = c.to_xyz65();
    common::assert_close(xyz.x, 0.950455927051672, EPS);
    common::assert_close(xyz.y, 1.0000000000000002, EPS);
    common::assert_close(xyz.z, 1.0890577507598787, EPS);
}

#[test]
fn rec2020_black() {
    let c = Rec2020 {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        alpha: None,
    };
    let xyz = c.to_xyz65();
    common::assert_close(xyz.x, 0.0, 1e-15);
    common::assert_close(xyz.y, 0.0, 1e-15);
    common::assert_close(xyz.z, 0.0, 1e-15);
}

#[test]
fn rec2020_red_primary() {
    let c = Rec2020 {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        alpha: None,
    };
    let xyz = c.to_xyz65();
    common::assert_close(xyz.x, 0.6369580483012914, EPS);
    common::assert_close(xyz.y, 0.2627002120112671, EPS);
    common::assert_close(xyz.z, 0.0, 1e-15);
}

#[test]
fn rec2020_green_primary() {
    let c = Rec2020 {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        alpha: None,
    };
    let xyz = c.to_xyz65();
    common::assert_close(xyz.x, 0.14461690358620835, EPS);
    common::assert_close(xyz.y, 0.6779980715188711, EPS);
    common::assert_close(xyz.z, 0.028072693049087414, EPS);
}

#[test]
fn rec2020_blue_primary() {
    let c = Rec2020 {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        alpha: None,
    };
    let xyz = c.to_xyz65();
    common::assert_close(xyz.x, 0.1688809751641722, EPS);
    common::assert_close(xyz.y, 0.05930171646986203, EPS);
    common::assert_close(xyz.z, 1.0609850577107913, EPS);
}

#[test]
fn rec2020_arbitrary_above_threshold() {
    let c = Rec2020 {
        r: 0.25,
        g: 0.4,
        b: 0.75,
        alpha: None,
    };
    let xyz = c.to_xyz65();
    common::assert_close(xyz.x, 0.1700644515202445, EPS);
    common::assert_close(xyz.y, 0.17134883007352456, EPS);
    common::assert_close(xyz.z, 0.6028540198785831, EPS);
}

#[test]
fn rec2020_below_threshold_uses_linear_segment() {
    // 0.01 < β * 4.5 (0.0812...): linear segment.
    let c = Rec2020 {
        r: 0.01,
        g: 0.01,
        b: 0.01,
        alpha: None,
    };
    let xyz = c.to_xyz65();
    common::assert_close(xyz.x, 0.002112124282337048, EPS);
    common::assert_close(xyz.y, 0.002222222222222222, EPS);
    common::assert_close(xyz.z, 0.0024201283350219517, EPS);
}

#[test]
fn rec2020_round_trip() {
    let c = Rec2020 {
        r: 0.25,
        g: 0.4,
        b: 0.75,
        alpha: None,
    };
    let back = Rec2020::from_xyz65(c.to_xyz65());
    common::assert_close(back.r, 0.25, 1e-10);
    common::assert_close(back.g, 0.4, 1e-10);
    common::assert_close(back.b, 0.75, 1e-10);
}

#[test]
fn rec2020_alpha_preserved() {
    let c = Rec2020 {
        r: 0.5,
        g: 0.5,
        b: 0.5,
        alpha: Some(0.7),
    };
    let xyz = c.to_xyz65();
    assert_eq!(xyz.alpha, Some(0.7));
    let back = Rec2020::from_xyz65(xyz);
    assert_eq!(back.alpha, Some(0.7));
}
