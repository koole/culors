//! Ported tests for [`culors::map`]: `mapper`, `map_alpha_multiply`,
//! `map_alpha_divide`, `map_transfer_linear`, `map_transfer_gamma`.
//!
//! Expected values from culori 4.0.2 via `node -e`:
//!
//! ```js
//! mapper(mapAlphaMultiply, 'rgb')({mode:'rgb', r:0.5, g:0.5, b:0.5, alpha:0.5})
//!   === { mode: 'rgb', r: 0.25, g: 0.25, b: 0.25, alpha: 0.5 }
//! mapper(mapAlphaDivide, 'rgb')({mode:'rgb', r:0.5, g:0.5, b:0.5, alpha:0.5})
//!   === { mode: 'rgb', r: 1, g: 1, b: 1, alpha: 0.5 }
//! mapper(mapTransferLinear(2, 0.1), 'rgb')({mode:'rgb', r:0.1, g:0.2, b:0.3})
//!   === { r: 0.30000000000000004, g: 0.5, b: 0.7 }
//! mapper(mapTransferGamma(1, 2, 0), 'rgb')({mode:'rgb', r:0.1, g:0.2, b:0.3})
//!   === { r: 0.010000000000000002, g: 0.04…, b: 0.09 }
//! ```

use culors::spaces::Rgb;
use culors::{
    map_alpha_divide, map_alpha_multiply, map_transfer_gamma, map_transfer_linear, mapper, Color,
};

const EPS: f64 = 1e-12;

#[track_caller]
fn close(label: &str, got: f64, expected: f64) {
    let d = (got - expected).abs();
    assert!(d < EPS, "{label}: got {got}, expected {expected}, |Δ|={d}");
}

fn rgb(r: f64, g: f64, b: f64, a: Option<f64>) -> Color {
    Color::Rgb(Rgb {
        r,
        g,
        b,
        alpha: a,
    })
}

#[test]
fn alpha_multiply_premultiplies_rgb() {
    let f = mapper(map_alpha_multiply(), "rgb", false);
    let out = f(&rgb(0.5, 0.5, 0.5, Some(0.5)));
    let Color::Rgb(c) = out else {
        panic!("expected rgb output, got {out:?}")
    };
    close("r", c.r, 0.25);
    close("g", c.g, 0.25);
    close("b", c.b, 0.25);
    assert_eq!(c.alpha, Some(0.5));
}

#[test]
fn alpha_divide_un_premultiplies_rgb() {
    let f = mapper(map_alpha_divide(), "rgb", false);
    let out = f(&rgb(0.5, 0.5, 0.5, Some(0.5)));
    let Color::Rgb(c) = out else {
        panic!("expected rgb output, got {out:?}")
    };
    close("r", c.r, 1.0);
    close("g", c.g, 1.0);
    close("b", c.b, 1.0);
    assert_eq!(c.alpha, Some(0.5));
}

#[test]
fn alpha_divide_zero_alpha_passes_through() {
    let f = mapper(map_alpha_divide(), "rgb", false);
    let out = f(&rgb(0.5, 0.5, 0.5, Some(0.0)));
    let Color::Rgb(c) = out else { unreachable!() };
    // culori: with alpha == 0 the channel is returned unchanged.
    close("r", c.r, 0.5);
    assert_eq!(c.alpha, Some(0.0));
}

#[test]
fn transfer_linear_applies_per_channel() {
    let f = mapper(map_transfer_linear(2.0, 0.1), "rgb", false);
    let out = f(&rgb(0.1, 0.2, 0.3, None));
    let Color::Rgb(c) = out else { unreachable!() };
    close("r", c.r, 0.30000000000000004);
    close("g", c.g, 0.5);
    close("b", c.b, 0.7);
    assert_eq!(c.alpha, None);
}

#[test]
fn transfer_gamma_applies_per_channel() {
    let f = mapper(map_transfer_gamma(1.0, 2.0, 0.0), "rgb", false);
    let out = f(&rgb(0.1, 0.2, 0.3, None));
    let Color::Rgb(c) = out else { unreachable!() };
    close("r", c.r, 0.010_000_000_000_000_002);
    close("g", c.g, 0.04);
    close("b", c.b, 0.09);
}

#[test]
fn preserve_mode_round_trips_back_to_source_mode() {
    // culori:
    //   mapper(mapAlphaMultiply, 'rgb', true)({mode:'lab', l:50, a:10, b:-20, alpha:0.5})
    //   → { mode:'lab', l:25.063…, a:5.602…, b:-11.278…, alpha:0.5 }
    let lab = Color::Lab(culors::spaces::Lab {
        l: 50.0,
        a: 10.0,
        b: -20.0,
        alpha: Some(0.5),
    });
    let f = mapper(map_alpha_multiply(), "rgb", true);
    let out = f(&lab);
    let Color::Lab(c) = out else {
        panic!("preserve_mode should yield lab, got {out:?}")
    };
    close("l", c.l, 25.063_188_126_933_326);
    close("a", c.a, 5.602_304_203_002_534);
    close("b", c.b, -11.278_103_814_120_144);
    assert_eq!(c.alpha, Some(0.5));
}

#[test]
fn alpha_multiply_default_alpha_is_one() {
    // culori: alpha undefined → multiplier of 1, channels unchanged.
    let f = mapper(map_alpha_multiply(), "rgb", false);
    let out = f(&rgb(0.5, 0.5, 0.5, None));
    let Color::Rgb(c) = out else { unreachable!() };
    close("r", c.r, 0.5);
    close("g", c.g, 0.5);
    close("b", c.b, 0.5);
    assert_eq!(c.alpha, None);
}

#[test]
fn transfer_linear_leaves_alpha_alone() {
    // culori: ch === 'alpha' → transfer is identity.
    let f = mapper(map_transfer_linear(2.0, 1.0), "rgb", false);
    let out = f(&rgb(0.1, 0.2, 0.3, Some(0.7)));
    let Color::Rgb(c) = out else { unreachable!() };
    assert_eq!(c.alpha, Some(0.7));
    close("r", c.r, 1.2);
}
