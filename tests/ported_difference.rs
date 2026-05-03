//! Tests for the `difference_*` ΔE family.
//!
//! Reference values come from culori 4.0.2 via Node:
//!
//! ```bash
//! node -e "import('culori').then(c=>console.log(
//!   c.differenceCiede2000()(c.parse('red'), c.parse('blue'))))"
//! ```
//!
//! The HSV functional notation parser in culori 4.0.2 returns objects
//! that lack `mode`, which throws inside `differenceEuclidean`; we avoid
//! that input and use `oklch(...)` / `lch(...)` / `lab(...)` literals
//! instead when we want a hue-bearing source.

// Reference doubles are pasted verbatim from culori's Node output —
// changing the literals would silently weaken the parity check.
#![allow(clippy::approx_constant, clippy::excessive_precision)]

use culor::{
    difference_ciede2000, difference_ciede76, difference_ciede94, difference_ciede94_with,
    difference_cmc, difference_euclidean, difference_euclidean_with, difference_euclidean_xyz,
    difference_hue_chroma, difference_hue_saturation, difference_hyab, difference_itp,
    difference_jz, difference_ok, parse,
};

const EPS: f64 = 1e-10;

fn approx(a: f64, b: f64) -> bool {
    if a.is_nan() && b.is_nan() {
        return true;
    }
    (a - b).abs() < EPS
}

fn assert_close(label: &str, got: f64, expected: f64) {
    assert!(
        approx(got, expected),
        "{label}: got {got}, expected {expected} (diff {})",
        (got - expected).abs()
    );
}

fn p(s: &str) -> culor::Color {
    parse(s).unwrap_or_else(|| panic!("parse failed for {s}"))
}

// ----- difference_ciede76 -----

#[test]
fn cie76_identity_red() {
    let de = difference_ciede76();
    assert_close("red,red", de(&p("red"), &p("red")), 0.0);
}

#[test]
fn cie76_red_blue() {
    let de = difference_ciede76();
    assert_close("red,blue", de(&p("red"), &p("blue")), 176.30849559658239);
}

#[test]
fn cie76_red_green() {
    let de = difference_ciede76();
    assert_close("red,green", de(&p("red"), &p("green")), 133.10353104157542);
}

#[test]
fn cie76_white_black() {
    let de = difference_ciede76();
    assert_close("white,black", de(&p("white"), &p("black")), 100.0);
}

#[test]
fn cie76_one_lsb_in_red() {
    let de = difference_ciede76();
    assert_close(
        "#ff0000,#fe0000",
        de(&p("#ff0000"), &p("#fe0000")),
        0.3730329364679128,
    );
}

#[test]
fn cie76_lab_inputs_are_d65() {
    // Lab in culor is D50; the difference uses lab65. Two D50 Lab inputs
    // with identical numbers therefore have a non-zero distance only if
    // we routed through the wrong illuminant. Same-color identity must
    // still hold though, so check identity here.
    let de = difference_ciede76();
    let c = p("lab(50 30 -20)");
    assert_close("lab,lab identity", de(&c, &c), 0.0);
}

// ----- difference_ciede94 -----

#[test]
fn cie94_identity() {
    let de = difference_ciede94(false);
    assert_close("red,red", de(&p("red"), &p("red")), 0.0);
}

#[test]
fn cie94_default_red_blue() {
    let de = difference_ciede94(false);
    assert_close("red,blue", de(&p("red"), &p("blue")), 70.57699903580162);
}

#[test]
fn cie94_default_red_green() {
    let de = difference_ciede94(false);
    assert_close("red,green", de(&p("red"), &p("green")), 50.97483909078679);
}

#[test]
fn cie94_default_white_black() {
    let de = difference_ciede94(false);
    assert_close("white,black", de(&p("white"), &p("black")), 100.0);
}

#[test]
fn cie94_textiles_red_blue() {
    let de = difference_ciede94(true);
    assert_close("red,blue T", de(&p("red"), &p("blue")), 71.00109643642175);
}

#[test]
fn cie94_textiles_white_black() {
    let de = difference_ciede94(true);
    // Textile case halves k_l so d_l contribution drops by 4×.
    assert_close("white,black T", de(&p("white"), &p("black")), 50.0);
}

#[test]
fn cie94_with_explicit_params_matches_textile() {
    let de = difference_ciede94_with(2.0, 0.048, 0.014);
    assert_close(
        "explicit textile",
        de(&p("red"), &p("blue")),
        71.00109643642175,
    );
}

// ----- difference_ciede2000 -----

#[test]
fn ciede2000_identity() {
    let de = difference_ciede2000(1.0, 1.0, 1.0);
    assert_close("red,red", de(&p("red"), &p("red")), 0.0);
}

#[test]
fn ciede2000_red_blue() {
    let de = difference_ciede2000(1.0, 1.0, 1.0);
    assert_close("red,blue", de(&p("red"), &p("blue")), 52.878195285926445);
}

#[test]
fn ciede2000_red_green() {
    let de = difference_ciede2000(1.0, 1.0, 1.0);
    assert_close("red,green", de(&p("red"), &p("green")), 72.18053591242);
}

#[test]
fn ciede2000_red_lime() {
    let de = difference_ciede2000(1.0, 1.0, 1.0);
    assert_close("red,lime", de(&p("red"), &p("lime")), 86.60781444907963);
}

#[test]
fn ciede2000_white_black() {
    let de = difference_ciede2000(1.0, 1.0, 1.0);
    assert_close("white,black", de(&p("white"), &p("black")), 100.0);
}

#[test]
fn ciede2000_one_lsb() {
    let de = difference_ciede2000(1.0, 1.0, 1.0);
    assert_close(
        "#ff0000,#fe0000",
        de(&p("#ff0000"), &p("#fe0000")),
        0.20785170919580034,
    );
}

#[test]
fn ciede2000_kl_2() {
    // Doubling k_l halves the lightness contribution.
    let de = difference_ciede2000(2.0, 1.0, 1.0);
    assert_close("white,black k_l=2", de(&p("white"), &p("black")), 50.0);
}

#[test]
fn ciede2000_arbitrary_lab() {
    let de = difference_ciede2000(1.0, 1.0, 1.0);
    assert_close(
        "lab a vs lab b",
        de(&p("lab(50 30 -20)"), &p("lab(60 -10 25)")),
        48.01924204708141,
    );
}

#[test]
fn ciede2000_oklch_inputs() {
    let de = difference_ciede2000(1.0, 1.0, 1.0);
    assert_close(
        "oklch vs oklch",
        de(&p("oklch(0.7 0.15 30)"), &p("oklch(0.7 0.15 60)")),
        19.640635223081706,
    );
}

// ----- difference_cmc -----

#[test]
fn cmc_identity() {
    let de = difference_cmc(1.0, 1.0);
    assert_close("red,red", de(&p("red"), &p("red")), 0.0);
}

#[test]
fn cmc_default_red_blue() {
    let de = difference_cmc(1.0, 1.0);
    assert_close("red,blue", de(&p("red"), &p("blue")), 109.75972786343982);
}

#[test]
fn cmc_default_red_green() {
    let de = difference_cmc(1.0, 1.0);
    assert_close("red,green", de(&p("red"), &p("green")), 81.28819468966392);
}

#[test]
fn cmc_default_white_black() {
    let de = difference_cmc(1.0, 1.0);
    assert_close(
        "white,black",
        de(&p("white"), &p("black")),
        67.48017083587553,
    );
}

#[test]
fn cmc_textile_2_1() {
    let de = difference_cmc(2.0, 1.0);
    assert_close("red,blue 2:1", de(&p("red"), &p("blue")), 108.5692523388881);
}

#[test]
fn cmc_arbitrary() {
    let de = difference_cmc(1.0, 1.0);
    assert_close(
        "lab vs lab",
        de(&p("lab(50 30 -20)"), &p("lab(60 -10 25)")),
        40.68199889476971,
    );
}

#[test]
fn cmc_low_l_branch() {
    // l < 16 takes the 0.511 branch for S_L.
    let de = difference_cmc(1.0, 1.0);
    assert_close(
        "near black",
        de(&p("#000000"), &p("#010101")),
        0.5365455969973609,
    );
}

// ----- difference_euclidean -----

#[test]
fn euclidean_rgb_red_blue() {
    let de = difference_euclidean("rgb");
    assert_close(
        "rgb red,blue",
        de(&p("red"), &p("blue")),
        1.4142135623730951,
    );
}

#[test]
fn euclidean_oklab_red_blue() {
    let de = difference_euclidean("oklab");
    assert_close(
        "oklab red,blue",
        de(&p("red"), &p("blue")),
        0.5370898164568614,
    );
}

#[test]
fn euclidean_oklch_red_blue() {
    let de = difference_euclidean("oklch");
    // Polar-OkLCh distance with the chroma-aware hue term.
    assert_close(
        "oklch red,blue",
        de(&p("red"), &p("blue")),
        0.5370898164568614,
    );
}

#[test]
fn euclidean_lab65_red_blue() {
    let de = difference_euclidean("lab65");
    assert_close(
        "lab65 red,blue",
        de(&p("red"), &p("blue")),
        176.30849559658239,
    );
}

#[test]
fn euclidean_xyz_red_blue() {
    let de = difference_euclidean("xyz65");
    assert_close(
        "xyz red,blue",
        de(&p("red"), &p("blue")),
        0.9698677485049477,
    );
}

#[test]
fn euclidean_xyz_helper_matches() {
    let by_name = difference_euclidean("xyz65");
    let helper = difference_euclidean_xyz();
    let a = p("red");
    let b = p("blue");
    assert_close("helper agreement", helper(&a, &b), by_name(&a, &b));
}

#[test]
fn euclidean_ok_helper_matches() {
    let helper = difference_ok();
    let by_name = difference_euclidean("oklab");
    let a = p("red");
    let b = p("green");
    assert_close("ok helper", helper(&a, &b), by_name(&a, &b));
}

#[test]
fn euclidean_with_custom_weights() {
    // Setting the hue weight to 0 in oklch zeroes out the `dH` term, so
    // the result is sqrt(dL^2 + dC^2). For red→blue (which differs in
    // hue and chroma) this is strictly less than the full distance.
    let full = difference_euclidean("oklch")(&p("red"), &p("blue"));
    let no_hue = difference_euclidean_with("oklch", [1.0, 1.0, 0.0, 0.0])(&p("red"), &p("blue"));
    assert!(no_hue < full, "no_hue={no_hue} full={full}");
}

#[test]
fn euclidean_with_custom_weights_hsl_hue_only() {
    // weights = [1,0,0,0] in hsl → only the hue-saturation polar term.
    let de = difference_euclidean_with("hsl", [1.0, 0.0, 0.0, 0.0]);
    assert_close(
        "hsl 0->60 hue only",
        de(&p("hsl(0 100% 50%)"), &p("hsl(60 100% 50%)")),
        1.0000000000000002,
    );
}

#[test]
fn euclidean_chroma_zero_zeroes_oklch_hue_term() {
    let de = difference_euclidean("oklch");
    let a = p("oklch(0.7 0 30)");
    let b = p("oklch(0.7 0.2 60)");
    let got = de(&a, &b);
    // dL=0, dC=0.2, dH=0. Distance = 0.2.
    assert_close("oklch chroma=0", got, 0.2);
}

#[test]
fn euclidean_oklch_hue_wrap() {
    // oklch(... 350) vs oklch(... 10) — culori reference.
    let de = difference_euclidean("oklch");
    let got = de(&p("oklch(0.7 0.15 350)"), &p("oklch(0.7 0.15 10)"));
    let expected = 0.0520944533000791; // hue term only; l and c match.
    assert_close("oklch wrap", got, expected);
}

#[test]
fn euclidean_lrgb() {
    // Pure-red sRGB → linear. Identity must be 0.
    let de = difference_euclidean("lrgb");
    assert_close("lrgb identity", de(&p("red"), &p("red")), 0.0);
}

#[test]
fn euclidean_xyz50() {
    let de = difference_euclidean("xyz50");
    assert_close("xyz50 identity", de(&p("red"), &p("red")), 0.0);
}

// ----- difference_hue_chroma -----

#[test]
fn hue_chroma_oklch_default() {
    // culori returns a signed polar distance: ascending hue → negative
    // (because sin((smp - std + 360)/2 deg) is negative when 0 < smp-std < 360).
    let de = difference_hue_chroma("oklch");
    assert_close(
        "oklch hc",
        de(&p("oklch(0.7 0.15 30)"), &p("oklch(0.7 0.15 60)")),
        -0.0776457135307561,
    );
}

#[test]
fn hue_chroma_oklch_chroma_zero() {
    let de = difference_hue_chroma("oklch");
    assert_close(
        "oklch hc chroma=0",
        de(&p("oklch(0.7 0 30)"), &p("oklch(0.7 0.2 60)")),
        0.0,
    );
}

#[test]
fn hue_chroma_oklch_wrap() {
    // 350 → 10: polar formula uses (10 - 350 + 360)/2 = 10°, sin(10°)
    // is positive, so the result is positive (chroma weight unchanged).
    let de = difference_hue_chroma("oklch");
    assert_close(
        "oklch hc wrap",
        de(&p("oklch(0.7 0.15 350)"), &p("oklch(0.7 0.15 10)")),
        0.0520944533000791,
    );
}

#[test]
fn hue_chroma_lch() {
    let de = difference_hue_chroma("lch");
    assert_close(
        "lch hc",
        de(&p("lch(50 40 0)"), &p("lch(50 40 90)")),
        -56.5685424949238,
    );
}

#[test]
fn hue_chroma_oklch_identity() {
    let de = difference_hue_chroma("oklch");
    assert_close(
        "oklch hc identity",
        de(&p("oklch(0.7 0.15 30)"), &p("oklch(0.7 0.15 30)")),
        0.0,
    );
}

// ----- difference_hue_saturation -----

#[test]
fn hue_saturation_hsl_default() {
    let de = difference_hue_saturation("hsl");
    assert_close(
        "hsl hs",
        de(&p("hsl(0 100% 50%)"), &p("hsl(60 100% 50%)")),
        -1.0000000000000002,
    );
}

#[test]
fn hue_saturation_hsl_zero_saturation() {
    let de = difference_hue_saturation("hsl");
    assert_close(
        "hsl hs s=0",
        de(&p("hsl(0 0% 50%)"), &p("hsl(60 100% 50%)")),
        0.0,
    );
}

#[test]
fn hue_saturation_hsl_wrap() {
    let de = difference_hue_saturation("hsl");
    assert_close(
        "hsl hs wrap",
        de(&p("hsl(350 50% 50%)"), &p("hsl(10 50% 50%)")),
        0.17364817766693033,
    );
}

#[test]
fn hue_saturation_hsl_identity() {
    let de = difference_hue_saturation("hsl");
    assert_close(
        "hsl hs identity",
        de(&p("hsl(60 100% 50%)"), &p("hsl(60 100% 50%)")),
        0.0,
    );
}

// ----- difference_jz / difference_itp -----

#[test]
fn jz_red_to_blue_matches_culori() {
    let de = difference_jz();
    assert_close(
        "difference_jz(red, blue)",
        de(&p("red"), &p("blue")),
        0.33960388420164006,
    );
}

#[test]
fn itp_red_to_blue_matches_culori() {
    let de = difference_itp();
    assert_close(
        "difference_itp(red, blue)",
        de(&p("red"), &p("blue")),
        349.7161209211369,
    );
}

// ----- cross-cuts -----

#[test]
fn ciede76_matches_euclidean_lab65() {
    let a = p("lab(50 30 -20)");
    let b = p("lab(60 -10 25)");
    let cie = difference_ciede76()(&a, &b);
    let eu = difference_euclidean("lab65")(&a, &b);
    assert_close("cie76 == eu lab65", cie, eu);
}

#[test]
fn euclidean_lab65_red_green() {
    let de = difference_euclidean("lab65");
    assert_close(
        "lab65 red,green",
        de(&p("red"), &p("green")),
        133.10353104157542,
    );
}

#[test]
fn euclidean_oklab_arbitrary() {
    let de = difference_euclidean("oklab");
    assert_close(
        "oklab arbitrary",
        de(&p("oklab(0.5 0.1 0.1)"), &p("oklab(0.6 -0.1 0.05)")),
        0.22912878474779202,
    );
}

#[test]
fn euclidean_oklch_arbitrary() {
    let de = difference_euclidean("oklch");
    assert_close(
        "oklch arbitrary",
        de(&p("oklch(0.7 0.15 30)"), &p("oklch(0.7 0.15 60)")),
        0.0776457135307561,
    );
}

#[test]
fn ciede2000_grays() {
    // Two pure grays — chroma is zero throughout, so the chroma and hue
    // terms drop out and the result is just the L difference.
    let de = difference_ciede2000(1.0, 1.0, 1.0);
    let dark = p("rgb(50 50 50)");
    let mid = p("rgb(150 150 150)");
    // L from culori's lab65 is (Y/D65_Y in [0,1]). Difference is
    // monotonic in the input gap; we just check it's positive and finite.
    let got = de(&dark, &mid);
    assert!(got > 0.0 && got.is_finite(), "got {got}");
}

#[test]
fn cmc_grays_low_l() {
    let de = difference_cmc(1.0, 1.0);
    let got = de(&p("#000000"), &p("#010101"));
    // Reference value from culori (low-L branch).
    assert_close("cmc near black", got, 0.5365455969973609);
}

#[test]
fn ciede94_one_lsb_default() {
    let de = difference_ciede94(false);
    assert_close(
        "cie94 #ff0000 vs #fe0000",
        de(&p("#ff0000"), &p("#fe0000")),
        0.21306047901728037,
    );
}

#[test]
fn ciede2000_arbitrary_lch_inputs() {
    let de = difference_ciede2000(1.0, 1.0, 1.0);
    assert_close(
        "lch vs lch",
        de(&p("lch(50 40 30)"), &p("lch(60 30 200)")),
        46.37013113626348,
    );
}

#[test]
fn euclidean_xyz_arbitrary() {
    let de = difference_euclidean_xyz();
    assert_close(
        "xyz arbitrary",
        de(&p("red"), &p("green")),
        0.34028831136242416,
    );
}

#[test]
fn euclidean_default_alpha_weight_zero() {
    // Two identical-RGB colors with different alphas are still distance 0
    // because the default alpha weight is 0.
    let de = difference_euclidean("rgb");
    let a = p("rgba(255 0 0 / 0.2)");
    let b = p("rgba(255 0 0 / 0.9)");
    assert_close("alpha ignored", de(&a, &b), 0.0);
}

#[test]
fn ciede76_hex_short() {
    let de = difference_ciede76();
    assert_close(
        "#abc vs #cba",
        de(&p("#abc"), &p("#cba")),
        22.086715431930024,
    );
}

#[test]
fn ciede2000_hex_short() {
    let de = difference_ciede2000(1.0, 1.0, 1.0);
    assert_close(
        "#abc vs #cba",
        de(&p("#abc"), &p("#cba")),
        18.39548249089912,
    );
}

#[test]
fn cmc_hex_short() {
    let de = difference_cmc(1.0, 1.0);
    assert_close(
        "#abc vs #cba",
        de(&p("#abc"), &p("#cba")),
        26.719757979421242,
    );
}

#[test]
fn ciede94_hex_short() {
    let de = difference_ciede94(false);
    assert_close(
        "#abc vs #cba",
        de(&p("#abc"), &p("#cba")),
        19.032710628226997,
    );
}

// ----- difference_hyab -----

#[test]
fn hyab_red_blue() {
    let de = difference_hyab();
    assert_close("red vs blue", de(&p("red"), &p("blue")), 195.9972588015325);
}

#[test]
fn hyab_red_green() {
    let de = difference_hyab();
    assert_close(
        "red vs green",
        de(&p("red"), &p("green")),
        139.92805737622862,
    );
}

#[test]
fn hyab_white_black() {
    let de = difference_hyab();
    assert_close("white vs black", de(&p("white"), &p("black")), 100.0);
}

#[test]
fn hyab_identity_red() {
    let de = difference_hyab();
    assert_close("red vs red", de(&p("red"), &p("red")), 0.0);
}

#[test]
fn hyab_hex_short() {
    let de = difference_hyab();
    assert_close(
        "#abc vs #cba",
        de(&p("#abc"), &p("#cba")),
        23.761286472410188,
    );
}
