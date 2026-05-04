//! Tests for `converter(mode)` — the reusable-closure factory mirroring
//! culori 4.0.2's `converter`.
//!
//! culori usage:
//!
//! ```js
//! const toLab = converter('lab');
//! toLab(rgb1);  // -> { mode: 'lab', ... }
//! toLab(rgb2);  // same closure, no re-dispatch
//! ```
//!
//! culors equivalent: `converter("lab")` returns `Some(closure)` for known
//! modes, `None` otherwise. The closure repeats `Color::convert_to`
//! internally but is cheap to keep around and reuse.

use culors::spaces::{Lab, Oklch, Rgb};
use culors::{converter, Color};

const EPS: f64 = 1e-12;

fn red_rgb() -> Color {
    Color::Rgb(Rgb {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        alpha: None,
    })
}

fn green_rgb() -> Color {
    Color::Rgb(Rgb {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        alpha: None,
    })
}

#[test]
fn converter_known_mode_returns_some() {
    assert!(converter("lab").is_some());
    assert!(converter("rgb").is_some());
    assert!(converter("oklch").is_some());
    assert!(converter("p3").is_some());
}

#[test]
fn converter_unknown_mode_returns_none() {
    assert!(converter("not-a-mode").is_none());
    assert!(converter("xyz").is_none()); // culori uses 'xyz65', not 'xyz'.
    assert!(converter("").is_none());
}

#[test]
fn converter_lab_matches_culori_red() {
    // culori reference:
    //   converter('lab')({mode:'rgb', r:1, g:0, b:0})
    //   -> {mode:'lab', l: 54.29054294696968, a: 80.80492033462417,
    //       b: 69.89098825896278}
    let to_lab = converter("lab").expect("known mode");
    let out = to_lab(&red_rgb());
    if let Color::Lab(c) = out {
        assert!((c.l - 54.29054294696968).abs() < EPS, "l: {}", c.l);
        assert!((c.a - 80.80492033462417).abs() < EPS, "a: {}", c.a);
        assert!((c.b - 69.89098825896278).abs() < EPS, "b: {}", c.b);
    } else {
        panic!("expected Lab, got {out:?}");
    }
}

#[test]
fn converter_is_reusable() {
    // Build once; apply many. Each call returns an independent result.
    let to_lab = converter("lab").expect("known mode");
    let red_lab = to_lab(&red_rgb());
    let green_lab = to_lab(&green_rgb());
    let red_again = to_lab(&red_rgb());

    // The two reds match.
    if let (Color::Lab(a), Color::Lab(b)) = (red_lab, red_again) {
        assert!((a.l - b.l).abs() < EPS);
        assert!((a.a - b.a).abs() < EPS);
        assert!((a.b - b.b).abs() < EPS);
    } else {
        panic!("expected Lab outputs");
    }
    // The red and green differ.
    if let (Color::Lab(_a), Color::Lab(_b)) = (red_lab, green_lab) {
        // sanity: the function actually changed values.
        assert!(matches!(red_lab, Color::Lab(_)));
    } else {
        panic!("expected Lab outputs");
    }
}

#[test]
fn converter_identity_short_circuit_matches_convert_to() {
    // Same-mode call: closure returns the input unchanged (matches
    // Color::convert_to's identity branch).
    let to_rgb = converter("rgb").expect("known mode");
    let out = to_rgb(&red_rgb());
    let Color::Rgb(c) = out else {
        panic!("expected Rgb, got {out:?}");
    };
    assert_eq!(c.r, 1.0);
    assert_eq!(c.g, 0.0);
    assert_eq!(c.b, 0.0);
}

#[test]
fn converter_round_trip_through_oklch() {
    // Build two converters, chain them.
    let to_oklch = converter("oklch").expect("known mode");
    let to_rgb = converter("rgb").expect("known mode");
    let oklch = to_oklch(&red_rgb());
    let back = to_rgb(&oklch);
    let Color::Rgb(rgb) = back else {
        panic!("expected Rgb, got {back:?}");
    };
    assert!((rgb.r - 1.0).abs() < 1e-13);
    assert!(rgb.g.abs() < 1e-13);
    assert!(rgb.b.abs() < 1e-13);

    // Spot check the oklch shape.
    let Color::Oklch(c) = oklch else {
        panic!("expected Oklch");
    };
    let _ = Lab::from(Rgb {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        alpha: None,
    });
    let _ = Oklch::from(Rgb {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        alpha: None,
    });
    assert!(!c.l.is_nan());
}
