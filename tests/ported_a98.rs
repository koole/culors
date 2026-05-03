//! Ported tests for the Adobe RGB (1998) color space.

use culors::spaces::A98;
use culors::ColorSpace;

#[path = "common/mod.rs"]
mod common;

const EPS: f64 = 1e-12;

#[test]
fn a98_metadata() {
    assert_eq!(A98::CHANNELS, &["r", "g", "b"]);
    assert_eq!(A98::MODE, "a98");
}

#[test]
fn a98_white() {
    let c = A98 {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        alpha: None,
    };
    let xyz = c.to_xyz65();
    common::assert_close(xyz.x, 0.9504559270516715, EPS);
    common::assert_close(xyz.y, 1.0, EPS);
    common::assert_close(xyz.z, 1.089057750759878, EPS);
}

#[test]
fn a98_red_primary() {
    let c = A98 {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        alpha: None,
    };
    let xyz = c.to_xyz65();
    common::assert_close(xyz.x, 0.5766690429101305, EPS);
    common::assert_close(xyz.y, 0.297344975250536, EPS);
    common::assert_close(xyz.z, 0.0270313613864123, EPS);
}

#[test]
fn a98_green_primary() {
    let c = A98 {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        alpha: None,
    };
    let xyz = c.to_xyz65();
    common::assert_close(xyz.x, 0.1855582379065463, EPS);
    common::assert_close(xyz.y, 0.6273635662554661, EPS);
    common::assert_close(xyz.z, 0.0706888525358272, EPS);
}

#[test]
fn a98_blue_primary() {
    let c = A98 {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        alpha: None,
    };
    let xyz = c.to_xyz65();
    common::assert_close(xyz.x, 0.1882286462349947, EPS);
    common::assert_close(xyz.y, 0.0752914584939979, EPS);
    common::assert_close(xyz.z, 0.9913375368376386, EPS);
}

#[test]
fn a98_arbitrary_matches_culori() {
    let c = A98 {
        r: 0.25,
        g: 0.4,
        b: 0.75,
        alpha: None,
    };
    let xyz = c.to_xyz65();
    common::assert_close(xyz.x, 0.15206096989167903, EPS);
    common::assert_close(xyz.y, 0.13772181769213862, EPS);
    common::assert_close(xyz.z, 0.5372722231689777, EPS);
}

#[test]
fn a98_mid_gray() {
    let c = A98 {
        r: 0.5,
        g: 0.5,
        b: 0.5,
        alpha: None,
    };
    let xyz = c.to_xyz65();
    common::assert_close(xyz.x, 0.20696703237310687, EPS);
    common::assert_close(xyz.y, 0.21775552814439453, EPS);
    common::assert_close(xyz.z, 0.23714834569646365, EPS);
}

#[test]
fn a98_round_trip() {
    let c = A98 {
        r: 0.25,
        g: 0.4,
        b: 0.75,
        alpha: None,
    };
    let back = A98::from_xyz65(c.to_xyz65());
    common::assert_close(back.r, 0.25, 1e-10);
    common::assert_close(back.g, 0.4, 1e-10);
    common::assert_close(back.b, 0.75, 1e-10);
}

#[test]
fn a98_alpha_preserved() {
    let c = A98 {
        r: 0.5,
        g: 0.5,
        b: 0.5,
        alpha: Some(0.7),
    };
    let xyz = c.to_xyz65();
    assert_eq!(xyz.alpha, Some(0.7));
    let back = A98::from_xyz65(xyz);
    assert_eq!(back.alpha, Some(0.7));
}
