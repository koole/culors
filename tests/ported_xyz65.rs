//! Ported tests for the `Xyz65` hub color space.

use culor::ColorSpace;
use culor::spaces::Xyz65;

#[path = "common/mod.rs"]
mod common;

#[test]
fn xyz65_round_trip_through_self() {
    let c = Xyz65 {
        x: 0.5,
        y: 0.4,
        z: 0.3,
        alpha: None,
    };
    let back = Xyz65::from_xyz65(c.to_xyz65());
    common::assert_close(back.x, 0.5, 1e-15);
    common::assert_close(back.y, 0.4, 1e-15);
    common::assert_close(back.z, 0.3, 1e-15);
}

#[test]
fn xyz65_alpha_with_alpha() {
    let c = Xyz65 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        alpha: Some(0.5),
    };
    let with = c.with_alpha(Some(1.0));
    assert_eq!(with.alpha(), Some(1.0));
    let cleared = with.with_alpha(None);
    assert_eq!(cleared.alpha(), None);
}
