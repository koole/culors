//! Ported tests for [`culor::nearest`], matching culori 4.0.2's
//! `nearest(colors, metric, accessor)` factory.

#![allow(clippy::type_complexity)]

use culor::{difference_euclidean, nearest, parse, Color};

fn p(s: &str) -> Color {
    parse(s).unwrap()
}

fn rgb(c: &Color) -> (f64, f64, f64) {
    match c {
        Color::Rgb(r) => (r.r, r.g, r.b),
        _ => panic!("expected Rgb, got {c:?}"),
    }
}

#[test]
fn nearest_single_match_red() {
    let palette = vec![p("red"), p("green"), p("blue")];
    let find = nearest(palette, None);
    let hits = find(&p("red"), 1);
    assert_eq!(hits.len(), 1);
    assert_eq!(rgb(&hits[0]), (1.0, 0.0, 0.0));
}

#[test]
fn nearest_purple_picks_magenta_first() {
    // Mirrors `c.nearest(palette)('#800080', 1)` in culori.
    let palette = vec![
        p("red"),
        p("green"),
        p("blue"),
        p("yellow"),
        p("cyan"),
        p("magenta"),
        p("black"),
        p("white"),
    ];
    let find = nearest(palette, None);
    let hits = find(&p("#800080"), 1);
    assert_eq!(hits.len(), 1);
    // Magenta is rgb(1,0,1) — closest to purple in RGB Euclidean.
    assert_eq!(rgb(&hits[0]), (1.0, 0.0, 1.0));
}

#[test]
fn nearest_purple_top_three() {
    let palette = vec![
        p("red"),
        p("green"),
        p("blue"),
        p("yellow"),
        p("cyan"),
        p("magenta"),
        p("black"),
        p("white"),
    ];
    let find = nearest(palette, None);
    let hits = find(&p("#800080"), 3);
    // culori order: magenta, red, blue.
    let colors: Vec<_> = hits.iter().map(rgb).collect();
    assert_eq!(
        colors,
        vec![(1.0, 0.0, 1.0), (1.0, 0.0, 0.0), (0.0, 0.0, 1.0)]
    );
}

#[test]
fn nearest_n_clamped_to_len_minus_one() {
    // n = palette.len() (finite) clamps to len-1, matching culori.
    let palette = vec![p("red"), p("green"), p("blue"), p("yellow")];
    let find = nearest(palette, None);
    let hits = find(&p("red"), 4);
    assert_eq!(hits.len(), 3);
}

#[test]
fn nearest_usize_max_returns_all_sorted() {
    // Equivalent to culori's `Infinity`: bypass the clamp and return the
    // whole palette sorted by distance.
    let palette = vec![p("red"), p("green"), p("blue"), p("yellow")];
    let find = nearest(palette, None);
    let hits = find(&p("red"), usize::MAX);
    assert_eq!(hits.len(), 4);
    // First entry is the query itself.
    assert_eq!(rgb(&hits[0]), (1.0, 0.0, 0.0));
}

#[test]
fn nearest_custom_metric() {
    // Same palette as `nearest_purple_picks_magenta_first` but with the
    // metric passed explicitly. Result should match.
    let palette = vec![
        p("red"),
        p("green"),
        p("blue"),
        p("yellow"),
        p("cyan"),
        p("magenta"),
        p("black"),
        p("white"),
    ];
    let metric: Box<dyn Fn(&Color, &Color) -> f64> = Box::new(difference_euclidean("rgb"));
    let find = nearest(palette, Some(metric));
    let hits = find(&p("#800080"), 1);
    assert_eq!(hits.len(), 1);
    assert_eq!(rgb(&hits[0]), (1.0, 0.0, 1.0));
}

#[test]
fn nearest_empty_palette_returns_empty() {
    let find = nearest(Vec::<Color>::new(), None);
    let hits = find(&p("red"), 5);
    assert!(hits.is_empty());
}
