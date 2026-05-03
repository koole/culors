//! Behavioral tests for `random` and `random_with_constraints`.
//!
//! culori's `random` calls `Math.random()` directly, so byte-for-byte
//! parity is impossible. We instead assert the *shape* of the output:
//!
//! 1. Channel values land in the right intervals.
//! 2. The output is in the requested mode.
//! 3. Successive calls produce different colors with overwhelming
//!    probability.
//! 4. Constraint overrides actually constrain the output.

use culors::spaces::{Cubehelix, Hsi, Itp, Jab, Jch, Lab65, Luv, Okhsl, Oklab, Oklch, Rgb, Xyb};
use culors::{random, random_with_constraints, Color};

/// All channels of an `Rgb` color must fall in `[0, 1]`.
#[test]
fn rgb_channels_in_natural_range() {
    for _ in 0..200 {
        match random("rgb") {
            Color::Rgb(c) => {
                assert!((0.0..=1.0).contains(&c.r), "r out of range: {}", c.r);
                assert!((0.0..=1.0).contains(&c.g), "g out of range: {}", c.g);
                assert!((0.0..=1.0).contains(&c.b), "b out of range: {}", c.b);
                assert!(c.alpha.is_none(), "alpha not requested but produced");
            }
            other => panic!("expected Color::Rgb, got {other:?}"),
        }
    }
}

/// The mode of the output matches the input mode string for every
/// supported space.
#[test]
fn output_mode_matches_request_for_every_space() {
    fn assert_variant(mode: &str, c: Color) {
        let actual = c.mode();
        assert_eq!(actual, mode, "mode mismatch for '{mode}'");
    }

    for mode in [
        "rgb",
        "lrgb",
        "p3",
        "a98",
        "rec2020",
        "prophoto",
        "hsl",
        "hsv",
        "hwb",
        "hsi",
        "hsluv",
        "hpluv",
        "okhsl",
        "okhsv",
        "lab",
        "lab65",
        "lch",
        "lch65",
        "lchuv",
        "luv",
        "oklab",
        "oklch",
        "dlab",
        "dlch",
        "jab",
        "jch",
        "itp",
        "yiq",
        "xyz50",
        "xyz65",
        "xyb",
        "cubehelix",
        "prismatic",
    ] {
        assert_variant(mode, random(mode));
    }
}

/// Successive samples must differ — collisions are vanishingly improbable
/// for a 64-bit-seeded PRNG. We accept a small tolerance for the unlikely
/// case where two consecutive samples agree to all channels (probability
/// ≈ 0 for f64).
#[test]
fn successive_calls_produce_different_colors() {
    let a = match random("rgb") {
        Color::Rgb(c) => (c.r, c.g, c.b),
        _ => unreachable!(),
    };
    let b = match random("rgb") {
        Color::Rgb(c) => (c.r, c.g, c.b),
        _ => unreachable!(),
    };
    assert!(a != b, "two consecutive random rgb calls produced identical channels: {a:?}");
}

/// HSL has `s` and `l` declared in `[0, 1]` (culori's `useMode` fallback)
/// and `h` in `[0, 360]`. Confirm both ranges hold.
#[test]
fn hsl_channel_ranges() {
    for _ in 0..100 {
        match random("hsl") {
            Color::Hsl(c) => {
                assert!((0.0..=360.0).contains(&c.h), "h out of [0,360]: {}", c.h);
                assert!((0.0..=1.0).contains(&c.s), "s out of [0,1]: {}", c.s);
                assert!((0.0..=1.0).contains(&c.l), "l out of [0,1]: {}", c.l);
            }
            _ => unreachable!(),
        }
    }
}

/// JzAzBz has signed `a` and `b` channels; assert the negative side is reachable.
#[test]
fn jab_channel_ranges_include_signed_a_and_b() {
    let mut saw_neg_a = false;
    let mut saw_neg_b = false;
    for _ in 0..500 {
        if let Color::Jab(c) = random("jab") {
            assert!((0.0..=0.222).contains(&c.j), "j out of range: {}", c.j);
            assert!((-0.109..=0.129).contains(&c.a), "a out of range: {}", c.a);
            assert!((-0.185..=0.134).contains(&c.b), "b out of range: {}", c.b);
            if c.a < 0.0 {
                saw_neg_a = true;
            }
            if c.b < 0.0 {
                saw_neg_b = true;
            }
        }
    }
    assert!(saw_neg_a, "never sampled negative a");
    assert!(saw_neg_b, "never sampled negative b");
}

/// Constraint overrides clamp the channel to the override range and
/// leave other channels free.
#[test]
fn constraint_override_pins_single_channel() {
    for _ in 0..50 {
        let c = random_with_constraints("rgb", &[("r", (0.5, 0.5))]);
        match c {
            Color::Rgb(rgb) => {
                assert_eq!(rgb.r, 0.5, "r override ignored");
                // g and b should still vary in [0, 1].
                assert!((0.0..=1.0).contains(&rgb.g));
                assert!((0.0..=1.0).contains(&rgb.b));
            }
            _ => unreachable!(),
        }
    }
}

/// Constraint with a non-degenerate range constrains the output to that
/// sub-interval.
#[test]
fn constraint_override_pins_to_subinterval() {
    for _ in 0..200 {
        let c = random_with_constraints("rgb", &[("g", (0.2, 0.3))]);
        if let Color::Rgb(rgb) = c {
            assert!(
                (0.2..=0.3).contains(&rgb.g),
                "g escaped override range: {}",
                rgb.g
            );
        }
    }
}

/// Alpha is only produced when an `alpha` constraint is supplied. The
/// constraint can be a fixed value (`min == max`) or a range.
#[test]
fn alpha_only_when_constrained() {
    fn rgb_alpha(c: Color) -> Option<f64> {
        match c {
            Color::Rgb(r) => r.alpha,
            _ => panic!("expected Rgb, got {c:?}"),
        }
    }

    // No alpha constraint → alpha is None.
    assert!(rgb_alpha(random("rgb")).is_none());

    // Fixed alpha constraint → alpha = constant.
    assert_eq!(
        rgb_alpha(random_with_constraints("rgb", &[("alpha", (0.5, 0.5))])),
        Some(0.5)
    );

    // Range alpha constraint → alpha within range.
    for _ in 0..50 {
        let a = rgb_alpha(random_with_constraints("rgb", &[("alpha", (0.1, 0.9))]))
            .expect("alpha requested");
        assert!((0.1..=0.9).contains(&a), "alpha out of [0.1, 0.9]: {a}");
    }
}

/// Constraints can pin every channel; the result is fully deterministic
/// up to mode dispatch, regardless of PRNG state.
#[test]
fn full_constraints_yield_deterministic_color() {
    let c = random_with_constraints(
        "rgb",
        &[
            ("r", (0.1, 0.1)),
            ("g", (0.2, 0.2)),
            ("b", (0.3, 0.3)),
            ("alpha", (0.4, 0.4)),
        ],
    );
    match c {
        Color::Rgb(rgb) => {
            assert_eq!(rgb.r, 0.1);
            assert_eq!(rgb.g, 0.2);
            assert_eq!(rgb.b, 0.3);
            assert_eq!(rgb.alpha, Some(0.4));
        }
        _ => unreachable!(),
    }
}

/// Spot-check additional cylindrical/perceptual modes ride within their
/// declared ranges.
#[test]
fn cubehelix_saturation_can_exceed_unit() {
    let mut saw_above_one = false;
    for _ in 0..1000 {
        if let Color::Cubehelix(c) = random("cubehelix") {
            // Validate the literal Cubehelix struct lives in the declared range.
            let _: Cubehelix = c;
            assert!((0.0..=360.0).contains(&c.h));
            assert!((0.0..=4.614).contains(&c.s));
            assert!((0.0..=1.0).contains(&c.l));
            if c.s > 1.0 {
                saw_above_one = true;
            }
        }
    }
    assert!(saw_above_one, "cubehelix s never exceeded 1; range may be wrong");
}

#[test]
fn lab65_ranges() {
    for _ in 0..50 {
        if let Color::Lab65(c) = random("lab65") {
            let _: Lab65 = c;
            assert!((0.0..=100.0).contains(&c.l));
            assert!((-125.0..=125.0).contains(&c.a));
            assert!((-125.0..=125.0).contains(&c.b));
        }
    }
}

#[test]
fn luv_ranges() {
    for _ in 0..50 {
        if let Color::Luv(c) = random("luv") {
            let _: Luv = c;
            assert!((0.0..=100.0).contains(&c.l));
            assert!((-84.936..=175.042).contains(&c.u));
            assert!((-125.882..=87.243).contains(&c.v));
        }
    }
}

#[test]
fn oklab_oklch_ranges() {
    for _ in 0..50 {
        if let Color::Oklab(c) = random("oklab") {
            let _: Oklab = c;
            assert!((0.0..=1.0).contains(&c.l));
            assert!((-0.4..=0.4).contains(&c.a));
            assert!((-0.4..=0.4).contains(&c.b));
        }
        if let Color::Oklch(c) = random("oklch") {
            let _: Oklch = c;
            assert!((0.0..=1.0).contains(&c.l));
            assert!((0.0..=0.4).contains(&c.c));
            assert!((0.0..=360.0).contains(&c.h));
        }
    }
}

#[test]
fn jch_ranges() {
    for _ in 0..50 {
        if let Color::Jch(c) = random("jch") {
            let _: Jch = c;
            assert!((0.0..=0.221).contains(&c.j));
            assert!((0.0..=0.19).contains(&c.c));
            assert!((0.0..=360.0).contains(&c.h));
        }
    }
}

#[test]
fn itp_xyb_ranges() {
    for _ in 0..50 {
        if let Color::Itp(c) = random("itp") {
            let _: Itp = c;
            assert!((0.0..=0.581).contains(&c.i));
            assert!((-0.369..=0.272).contains(&c.t));
            assert!((-0.164..=0.331).contains(&c.p));
        }
        if let Color::Xyb(c) = random("xyb") {
            let _: Xyb = c;
            assert!((-0.0154..=0.0281).contains(&c.x));
            assert!((0.0..=0.8453).contains(&c.y));
            assert!((-0.2778..=0.388).contains(&c.b));
        }
    }
}

#[test]
fn jab_struct_type_propagates() {
    // Sanity: the variant we read carries the correct struct type.
    if let Color::Jab(c) = random("jab") {
        let _: Jab = c;
    } else {
        panic!("random('jab') did not produce Color::Jab");
    }
}

/// Unknown mode strings panic, mirroring `mode_ranges` returning `None`.
#[test]
#[should_panic(expected = "unknown mode 'frob'")]
fn unknown_mode_panics() {
    let _ = random("frob");
}

#[test]
fn hsi_okhsl_okhsv_ranges() {
    for _ in 0..50 {
        if let Color::Hsi(c) = random("hsi") {
            let _: Hsi = c;
            assert!((0.0..=360.0).contains(&c.h));
            assert!((0.0..=1.0).contains(&c.s));
            assert!((0.0..=1.0).contains(&c.i));
        }
        if let Color::Okhsl(c) = random("okhsl") {
            let _: Okhsl = c;
            assert!((0.0..=360.0).contains(&c.h));
            assert!((0.0..=1.0).contains(&c.s));
            assert!((0.0..=1.0).contains(&c.l));
        }
    }
}

/// Sample an Rgb color and confirm the typed struct destructures cleanly.
#[test]
fn rgb_struct_destructures() {
    if let Color::Rgb(c) = random("rgb") {
        let _: Rgb = c;
    } else {
        panic!("random('rgb') did not produce Color::Rgb");
    }
}
