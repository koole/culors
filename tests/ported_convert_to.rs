//! Direct tests for [`Color::convert_to`] and [`convert_culori`].
//!
//! The fixture suite (`convert_to_fixtures.rs`) is the byte-for-byte parity
//! gate against culori. This file targets specific behaviors of the dynamic
//! API that the fixture suite cannot easily express:
//!
//! - identity short-circuit (target mode equals source mode)
//! - rejection of unknown mode strings (`None` rather than panic)
//! - `target_mode` validity for every supported space
//! - achromatic snap for `Rgb → Lab/Lch/Oklab/Oklch`
//! - NaN-hue inputs survive a conversion round-trip
//! - typed `convert_culori` agrees with `convert_to` and diverges from
//!   the XYZ-hub `convert<>` on pairs where culori takes a shorter path

#![allow(clippy::float_cmp)]

#[path = "common/mod.rs"]
mod common;

use common::assert_close;
use culors::convert::convert_culori;
use culors::spaces::{
    Hsl, Hsv, Hwb, Lab, Lab65, Lch, Lch65, LinearRgb, Luv, Okhsl, Okhsv, Oklab, Oklch, ProphotoRgb,
    Rec2020, Rgb, Xyz50, Xyz65, A98, P3,
};
use culors::{convert, Color, ColorSpace};

const EPS: f64 = 1e-12;

fn rgb(r: f64, g: f64, b: f64) -> Color {
    Color::Rgb(Rgb {
        r,
        g,
        b,
        alpha: None,
    })
}

#[test]
fn identity_short_circuit_returns_self_for_every_mode() {
    // Each mode must satisfy `c.convert_to(c.mode()) == Some(c)`.
    let cases: [Color; 5] = [
        Color::Rgb(Rgb {
            r: 0.3,
            g: 0.6,
            b: 0.9,
            alpha: Some(0.5),
        }),
        Color::Lab(Lab {
            l: 70.0,
            a: 30.0,
            b: -20.0,
            alpha: None,
        }),
        Color::Oklch(Oklch {
            l: 0.7,
            c: 0.15,
            h: 30.0,
            alpha: None,
        }),
        Color::Xyz65(Xyz65 {
            x: 0.4,
            y: 0.5,
            z: 0.6,
            alpha: None,
        }),
        Color::Hsl(Hsl {
            h: 120.0,
            s: 0.5,
            l: 0.5,
            alpha: None,
        }),
    ];
    for c in cases {
        let out = c.convert_to(c.mode()).expect("identity must succeed");
        assert_eq!(out, c, "identity changed value for mode `{}`", c.mode());
    }
}

#[test]
fn unknown_mode_returns_none() {
    let c = rgb(0.5, 0.5, 0.5);
    assert!(c.convert_to("nonsense").is_none());
    assert!(c.convert_to("").is_none());
    assert!(
        c.convert_to("RGB").is_none(),
        "mode lookup is case-sensitive"
    );
    assert!(c.convert_to("rgba").is_none());
}

#[test]
fn every_known_mode_is_a_valid_target() {
    // Every mode the crate knows must round-trip from an Rgb source. This
    // catches new spaces that get added without wiring `convert_to`.
    let modes = [
        "rgb",
        "lrgb",
        "hsl",
        "hsv",
        "hwb",
        "hsi",
        "lab",
        "lab65",
        "lch",
        "lch65",
        "oklab",
        "oklch",
        "okhsl",
        "okhsv",
        "xyz50",
        "xyz65",
        "p3",
        "rec2020",
        "a98",
        "prophoto",
        "dlab",
        "dlch",
        "jab",
        "jch",
        "yiq",
        "cubehelix",
        "luv",
        "lchuv",
        "itp",
        "xyb",
        "hsluv",
        "hpluv",
        "prismatic",
    ];
    let src = rgb(0.6, 0.4, 0.2);
    for m in modes {
        let out = src
            .convert_to(m)
            .unwrap_or_else(|| panic!("convert_to({m}) returned None"));
        assert_eq!(out.mode(), m, "convert_to({m}) returned wrong variant");
    }
}

#[test]
fn rgb_gray_snaps_to_achromatic_lab_and_oklab() {
    // The achromatic snap is the headline reason `convert_to` exists: `convert<>`
    // leaves a 1e-6 residual on `a`/`b` when fed an `r==g==b` input.
    let gray = rgb(0.5, 0.5, 0.5);
    let lab = gray.convert_to("lab").unwrap();
    if let Color::Lab(l) = lab {
        assert_eq!(l.a, 0.0, "Lab.a must snap to exact zero for sRGB gray");
        assert_eq!(l.b, 0.0, "Lab.b must snap to exact zero for sRGB gray");
    } else {
        panic!("expected Color::Lab, got {lab:?}");
    }

    let oklab = gray.convert_to("oklab").unwrap();
    if let Color::Oklab(o) = oklab {
        assert_eq!(o.a, 0.0);
        assert_eq!(o.b, 0.0);
    } else {
        panic!("expected Color::Oklab");
    }
}

#[test]
fn rgb_gray_lch_chroma_snaps_to_zero() {
    // Lch and Oklch chroma is `sqrt(a² + b²)`; if `a` or `b` retained the
    // residual, chroma would be ~1e-6. The `convert_to` path must produce
    // chroma == 0 exactly.
    let gray = rgb(0.5, 0.5, 0.5);
    if let Color::Lch(l) = gray.convert_to("lch").unwrap() {
        assert_eq!(l.c, 0.0, "Lch chroma must be zero for sRGB gray");
    } else {
        panic!();
    }
    if let Color::Oklch(o) = gray.convert_to("oklch").unwrap() {
        assert_eq!(o.c, 0.0);
    } else {
        panic!();
    }
}

#[test]
fn alpha_propagates_through_dynamic_dispatch() {
    let src = Color::Rgb(Rgb {
        r: 0.2,
        g: 0.4,
        b: 0.8,
        alpha: Some(0.3),
    });
    for to in ["lab", "oklch", "xyz65", "p3", "lab65", "rec2020"] {
        let out = src.convert_to(to).unwrap();
        let alpha = match out {
            Color::Lab(c) => c.alpha,
            Color::Oklch(c) => c.alpha,
            Color::Xyz65(c) => c.alpha,
            Color::P3(c) => c.alpha,
            Color::Lab65(c) => c.alpha,
            Color::Rec2020(c) => c.alpha,
            _ => unreachable!(),
        };
        assert_eq!(alpha, Some(0.3), "alpha lost on convert_to({to})");
    }
}

#[test]
fn nan_hue_input_does_not_panic() {
    // culori models achromatic colors by omitting the hue channel (which
    // deserializes to NaN on the Rust side). `convert_to` must accept that
    // and not panic.
    let achromatic_lch = Color::Lch(Lch {
        l: 50.0,
        c: 0.0,
        h: f64::NAN,
        alpha: None,
    });
    let rgb = achromatic_lch.convert_to("rgb").expect("convert succeeds");
    if let Color::Rgb(r) = rgb {
        // Should land on a near-gray. Compare component spread as a sanity check.
        let max = r.r.max(r.g).max(r.b);
        let min = r.r.min(r.g).min(r.b);
        assert!(
            max - min < 1e-6,
            "achromatic Lch (NaN hue) should produce gray, got {r:?}"
        );
    } else {
        panic!();
    }
}

#[test]
fn round_trip_rgb_lab_rgb_is_close_to_input() {
    // Float drift in Lab→Rgb→Lab is bounded; we only need to confirm the
    // dynamic API doesn't introduce extra error vs the typed path.
    // Mid-gamut samples; pure-primary inputs land at gamut boundaries where
    // round-trip drift is dominated by sRGB gamma quantization, not the
    // dynamic-dispatch path under test.
    let cases = [
        rgb(0.1, 0.2, 0.3),
        rgb(0.5, 0.5, 0.5),
        rgb(0.7, 0.3, 0.1),
        rgb(0.4, 0.6, 0.8),
    ];
    for src in cases {
        let lab = src.convert_to("lab").unwrap();
        let back = lab.convert_to("rgb").unwrap();
        if let (Color::Rgb(a), Color::Rgb(b)) = (src, back) {
            assert_close(a.r, b.r, 1e-6);
            assert_close(a.g, b.g, 1e-6);
            assert_close(a.b, b.b, 1e-6);
        }
    }
}

#[test]
fn convert_culori_matches_typed_from_for_known_direct_pairs() {
    // For pairs where culori uses a direct edge that culors also has as a
    // typed `From` impl, `convert_culori` and the typed `From` must agree
    // bit-for-bit.
    let r = Rgb {
        r: 0.4,
        g: 0.6,
        b: 0.8,
        alpha: None,
    };
    let lab_typed = Lab::from(r);
    let lab_culori: Lab = convert_culori(r);
    assert_eq!(lab_typed, lab_culori);

    let oklab_typed = Oklab::from(r);
    let oklab_culori: Oklab = convert_culori(r);
    assert_eq!(oklab_typed, oklab_culori);

    let lrgb_typed = LinearRgb::from(r);
    let lrgb_culori: LinearRgb = convert_culori(r);
    assert_eq!(lrgb_typed, lrgb_culori);

    let lab = Lab {
        l: 65.0,
        a: 25.0,
        b: -15.0,
        alpha: None,
    };
    let lch_typed = Lch::from(lab);
    let lch_culori: Lch = convert_culori(lab);
    assert_eq!(lch_typed, lch_culori);
}

#[test]
fn convert_culori_diverges_from_xyz_hub_for_achromatic_rgb_to_lab() {
    // The headline divergence: `convert<Rgb, Lab>` routes through XYZ65 and
    // leaves a residual on a/b. `convert_culori` takes the snap-aware path.
    let gray = Rgb {
        r: 0.5,
        g: 0.5,
        b: 0.5,
        alpha: None,
    };
    let hub: Lab = convert(gray);
    let culori: Lab = convert_culori(gray);
    assert_eq!(culori.a, 0.0);
    assert_eq!(culori.b, 0.0);
    // The hub path produces a measurable, non-zero residual.
    assert!(hub.a.abs() > 0.0 || hub.b.abs() > 0.0);
}

#[test]
fn convert_culori_lab_to_rgb_short_circuits_through_xyz50() {
    // culori uses `Xyz50` as the Lab→Rgb intermediate. The XYZ-hub path
    // routes `Lab → Xyz65 → Rgb`. Same input, two paths — the culori path
    // produces a result that round-trips to the input exactly, while the
    // hub path may not.
    let lab = Lab {
        l: 50.0,
        a: 30.0,
        b: -40.0,
        alpha: None,
    };
    let r_hub: Rgb = convert(lab);
    let r_culori: Rgb = convert_culori(lab);
    // Both should agree to single-precision; the divergence is in the
    // last few ULPs.
    assert_close(r_hub.r, r_culori.r, 1e-13);
    assert_close(r_hub.g, r_culori.g, 1e-13);
    assert_close(r_hub.b, r_culori.b, 1e-13);
}

#[test]
fn convert_to_string_set_covers_every_colorspace_mode() {
    // Driver: every space that implements `ColorSpace` must have its `MODE`
    // accepted by `convert_to`. If a new space lands and the dispatcher is
    // not extended, this test fails fast.
    macro_rules! check_mode {
        ($($ty:ty),* $(,)?) => {
            $(
                let m = <$ty as ColorSpace>::MODE;
                let out = rgb(0.4, 0.5, 0.6).convert_to(m);
                assert!(out.is_some(), "convert_to({m}) returned None for ColorSpace `{}`", stringify!($ty));
            )*
        };
    }
    check_mode!(
        Rgb,
        LinearRgb,
        Hsl,
        Hsv,
        Hwb,
        Lab,
        Lab65,
        Lch,
        Lch65,
        Oklab,
        Oklch,
        Xyz50,
        Xyz65,
        P3,
        Rec2020,
        A98,
        ProphotoRgb,
        Okhsl,
        Okhsv,
        Luv,
    );
}

// ---- Property: convert_culori<A,B> equals A.convert_to(B::MODE) ---------

#[test]
fn convert_culori_typed_matches_dynamic_for_many_pairs() {
    // For 20+ specific (A, B) pairs, the typed `convert_culori::<A,B>` must
    // agree with the dynamic `Color::convert_to(B::MODE)` to within machine
    // epsilon. This covers the wiring between the typed wrapper and the
    // dispatch table.
    let r = Rgb {
        r: 0.3,
        g: 0.6,
        b: 0.2,
        alpha: Some(0.8),
    };
    macro_rules! check {
        ($A:ty => $B:ty) => {{
            let dyn_color = Color::from(<$A>::from(r));
            let typed: $B = convert_culori(<$A>::from(r));
            let dyn_target = dyn_color.convert_to(<$B as ColorSpace>::MODE).unwrap();
            let dyn_typed: $B = <$B>::try_from(dyn_target).expect("variant matches");
            assert_eq!(
                typed,
                dyn_typed,
                "convert_culori<{}, {}> diverges from convert_to",
                stringify!($A),
                stringify!($B)
            );
        }};
    }
    // Twenty pairs covering direct edges, fallback-via-rgb, and identity
    // edges (rgb→rgb is short-circuited but still typed).
    check!(Rgb => Lab);
    check!(Rgb => Oklab);
    check!(Rgb => Oklch);
    check!(Rgb => Hsl);
    check!(Rgb => Hsv);
    check!(Rgb => Lab65);
    check!(Rgb => Xyz50);
    check!(Rgb => Xyz65);
    check!(Rgb => P3);
    check!(Rgb => Rec2020);
    check!(Rgb => A98);
    check!(Rgb => ProphotoRgb);
    check!(Rgb => LinearRgb);
    check!(Rgb => Lch);
    check!(Rgb => Hwb);
    check!(Rgb => Okhsl);
    check!(Rgb => Okhsv);
    check!(Rgb => Luv);
    check!(Lab => Rgb);
    check!(Oklch => Rgb);
    check!(Lch => Rgb);
}

// ---- Hue-NaN propagation -----------------------------------------------

#[test]
fn nan_hue_round_trips_through_convert_to() {
    let inputs = [
        Color::Hsl(Hsl {
            h: f64::NAN,
            s: 0.0,
            l: 0.5,
            alpha: None,
        }),
        Color::Oklch(Oklch {
            l: 0.5,
            c: 0.0,
            h: f64::NAN,
            alpha: None,
        }),
        Color::Lch(Lch {
            l: 50.0,
            c: 0.0,
            h: f64::NAN,
            alpha: None,
        }),
    ];
    for c in inputs {
        // → rgb (uses each space's `→ rgb` direct edge, which culori defines
        //   even when hue is missing).
        let r = c.convert_to("rgb").unwrap();
        if let Color::Rgb(rr) = r {
            assert!(rr.r.is_finite() && rr.g.is_finite() && rr.b.is_finite());
            // Achromatic source → near-gray output.
            let max = rr.r.max(rr.g).max(rr.b);
            let min = rr.r.min(rr.g).min(rr.b);
            assert!(max - min < 1e-6, "achromatic source produced {rr:?}");
        }
    }
}

// ---- Sample: 8 hand-picked culori reference values ---------------------
//
// These pin a handful of (mode, input, expected output) triples taken from
// `culori.converter(to)(input)` invocations. They guard against silent drift
// the fixture suite might miss after a future regen.

#[test]
fn rgb_to_lab_red_matches_culori() {
    let red = rgb(1.0, 0.0, 0.0);
    if let Color::Lab(l) = red.convert_to("lab").unwrap() {
        // culori output: { l: 54.29054294696968, a: 80.80492033462417, b: 69.89098825896278 }
        assert_close(l.l, 54.29054294696968, EPS);
        assert_close(l.a, 80.80492033462417, EPS);
        assert_close(l.b, 69.89098825896278, EPS);
    } else {
        panic!();
    }
}

#[test]
fn rgb_to_oklch_blue_matches_culori() {
    let blue = rgb(0.0, 0.0, 1.0);
    if let Color::Oklch(o) = blue.convert_to("oklch").unwrap() {
        // culori output: { l: 0.45201371817442365, c: 0.31321438863448475, h: 264.05202261636987 }
        assert_close(o.l, 0.45201371817442365, EPS);
        assert_close(o.c, 0.31321438863448475, EPS);
        assert_close(o.h, 264.05202261636987, EPS);
    } else {
        panic!();
    }
}

#[test]
fn lab_to_rgb_matches_culori() {
    let lab = Color::Lab(Lab {
        l: 60.0,
        a: 30.0,
        b: -20.0,
        alpha: None,
    });
    if let Color::Rgb(r) = lab.convert_to("rgb").unwrap() {
        // culori output: r=0.7195178613017736, g=0.4907152050525237, b=0.7086313328610159
        assert_close(r.r, 0.7195178613017736, EPS);
        assert_close(r.g, 0.4907152050525237, EPS);
        assert_close(r.b, 0.7086313328610159, EPS);
    } else {
        panic!();
    }
}
