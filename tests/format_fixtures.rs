//! Fixture-driven CSS format round-trip tests.
//!
//! `tests/fixtures/format_round_trip.json` lists CSS input strings paired
//! with culori 4.0.2's `formatCss(parse(input))` output. This test
//! asserts that `culor::format_css(culor::parse(input).unwrap())`
//! produces the byte-identical string for every supported input.
//!
//! The comparison is a plain string equality. Any divergence is a real
//! bug — either in the formatter, or (less likely; flagged but not fixed
//! here) in the parser feeding it.

use culor::{format_css, parse};
use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
struct FormatFixture {
    rows: Vec<FormatRow>,
}

#[derive(Deserialize)]
struct FormatRow {
    input: String,
    formatted: String,
}

#[test]
fn format_round_trip_matches_culori() {
    let json = fs::read_to_string("tests/fixtures/format_round_trip.json")
        .expect("missing tests/fixtures/format_round_trip.json");
    let fixture: FormatFixture =
        serde_json::from_str(&json).expect("invalid format_round_trip.json");
    let mut failures = Vec::new();
    for (i, row) in fixture.rows.iter().enumerate() {
        let parsed = parse(&row.input);
        match parsed {
            Some(c) => {
                let actual = format_css(&c);
                if actual != row.formatted {
                    failures.push(format!(
                        "row {}: input={:?}\n  expected: {:?}\n  actual:   {:?}",
                        i, row.input, row.formatted, actual
                    ));
                }
            }
            None => failures.push(format!(
                "row {}: input={:?} could not be parsed",
                i, row.input
            )),
        }
    }
    if !failures.is_empty() {
        let n = failures.len();
        for f in failures.iter().take(20) {
            eprintln!("{f}");
        }
        if n > 20 {
            eprintln!("... and {} more", n - 20);
        }
        panic!("{n} format round-trip failures");
    }
}
