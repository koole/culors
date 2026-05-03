//! Ported tests for `interpolate_with_premultiplied_alpha`.
//!
//! Reference values produced from culori 4.0.2 via `node -e "const c =
//! require('culori'); const f = c.interpolateWithPremultipliedAlpha([…],
//! 'rgb'); console.log(JSON.stringify(f(t)));"`. Each block also pins the
//! corresponding naive interpolation, so a divergence in either direction
//! is caught.

use culors::spaces::Rgb;
use culors::{interpolate, interpolate_with_premultiplied_alpha, Color, InterpolateOptions};

const TOL: f64 = 1e-12;

fn rgb(r: f64, g: f64, b: f64, alpha: Option<f64>) -> Color {
    Color::Rgb(Rgb { r, g, b, alpha })
}

fn unwrap_rgb(c: Color) -> Rgb {
    match c {
        Color::Rgb(r) => r,
        other => panic!("expected Rgb, got {other:?}"),
    }
}

fn close(actual: f64, expected: f64, label: &str) {
    let diff = (actual - expected).abs();
    assert!(
        diff <= TOL,
        "{label}: expected {expected}, got {actual} (diff {diff:.3e})"
    );
}

#[test]
fn fully_transparent_to_opaque_drops_the_transparent_color() {
    // Transparent red (alpha = 0) blended with opaque blue should be pure
    // blue at t=0.5 because the red contributes no premultiplied energy.
    // Naive RGB lerp would produce purple — that's the reason premultiplied
    // alpha exists.
    let red_a0 = rgb(1.0, 0.0, 0.0, Some(0.0));
    let blue = rgb(0.0, 0.0, 1.0, Some(1.0));
    let stops = [red_a0, blue];

    let pre = interpolate_with_premultiplied_alpha(&stops, "rgb", InterpolateOptions::new());
    let mid = unwrap_rgb(pre(0.5));
    close(mid.r, 0.0, "premul r");
    close(mid.g, 0.0, "premul g");
    close(mid.b, 1.0, "premul b");
    close(mid.alpha.unwrap(), 0.5, "premul alpha");

    // Sanity: naive interpolation gives the polluted purple.
    let naive = interpolate(&stops, "rgb");
    let naive_mid = unwrap_rgb(naive(0.5));
    close(naive_mid.r, 0.5, "naive r");
    close(naive_mid.b, 0.5, "naive b");
}

#[test]
fn half_transparent_to_opaque_quarter_steps() {
    let red_a05 = rgb(1.0, 0.0, 0.0, Some(0.5));
    let blue = rgb(0.0, 0.0, 1.0, Some(1.0));
    let stops = [red_a05, blue];

    let pre = interpolate_with_premultiplied_alpha(&stops, "rgb", InterpolateOptions::new());

    // Reference values from culori.
    // t=0.25: r=0.6, g=0, b=0.4, alpha=0.625
    let q = unwrap_rgb(pre(0.25));
    close(q.r, 0.6, "q r");
    close(q.b, 0.4, "q b");
    close(q.alpha.unwrap(), 0.625, "q alpha");

    // t=0.5: r=0.333..., b=0.666..., alpha=0.75
    let h = unwrap_rgb(pre(0.5));
    close(h.r, 0.3333333333333333, "h r");
    close(h.b, 0.6666666666666666, "h b");
    close(h.alpha.unwrap(), 0.75, "h alpha");

    // t=0.75: r=0.142857..., b=0.857142..., alpha=0.875
    let tq = unwrap_rgb(pre(0.75));
    close(tq.r, 0.14285714285714285, "tq r");
    close(tq.b, 0.8571428571428571, "tq b");
    close(tq.alpha.unwrap(), 0.875, "tq alpha");
}

#[test]
fn opaque_to_opaque_matches_naive() {
    let red = rgb(1.0, 0.0, 0.0, Some(1.0));
    let blue = rgb(0.0, 0.0, 1.0, Some(1.0));
    let stops = [red, blue];

    let pre = interpolate_with_premultiplied_alpha(&stops, "rgb", InterpolateOptions::new());
    let naive = interpolate(&stops, "rgb");

    for &t in &[0.0, 0.25, 0.5, 0.75, 1.0] {
        let p = unwrap_rgb(pre(t));
        let n = unwrap_rgb(naive(t));
        close(p.r, n.r, &format!("opaque r t={t}"));
        close(p.g, n.g, &format!("opaque g t={t}"));
        close(p.b, n.b, &format!("opaque b t={t}"));
    }
}

#[test]
fn transparent_white_to_opaque_red() {
    // Premultiplied white-to-red should keep red components dominant
    // through the ramp because the white side contributes zero
    // premultiplied energy.
    let white_a0 = rgb(1.0, 1.0, 1.0, Some(0.0));
    let red = rgb(1.0, 0.0, 0.0, Some(1.0));
    let stops = [white_a0, red];

    let pre = interpolate_with_premultiplied_alpha(&stops, "rgb", InterpolateOptions::new());

    // Reference: r=1, g=0, b=0 at every interior t.
    for &t in &[0.25_f64, 0.5, 0.75] {
        let v = unwrap_rgb(pre(t));
        close(v.r, 1.0, &format!("t={t} r"));
        close(v.g, 0.0, &format!("t={t} g"));
        close(v.b, 0.0, &format!("t={t} b"));
        close(v.alpha.unwrap(), t, &format!("t={t} alpha"));
    }
}

#[test]
fn no_alpha_specified_treated_as_opaque() {
    let red = rgb(1.0, 0.0, 0.0, None);
    let blue = rgb(0.0, 0.0, 1.0, None);
    let stops = [red, blue];

    let pre = interpolate_with_premultiplied_alpha(&stops, "rgb", InterpolateOptions::new());
    let mid = unwrap_rgb(pre(0.5));
    close(mid.r, 0.5, "no-alpha r");
    close(mid.g, 0.0, "no-alpha g");
    close(mid.b, 0.5, "no-alpha b");
    // Both inputs had alpha = None; culori preserves None on output.
    assert!(mid.alpha.is_none(), "alpha should be None");
}

#[test]
fn boundary_t_zero_returns_original_divided_by_alpha() {
    // culori's quirk: at t == 0 the result is the *original* first stop
    // divided by its alpha. For (r=1, alpha=0.5) that yields r=2.
    let red_a05 = rgb(1.0, 0.0, 0.0, Some(0.5));
    let blue = rgb(0.0, 0.0, 1.0, Some(1.0));
    let stops = [red_a05, blue];

    let pre = interpolate_with_premultiplied_alpha(&stops, "rgb", InterpolateOptions::new());
    let v = unwrap_rgb(pre(0.0));
    close(v.r, 2.0, "boundary r");
    close(v.alpha.unwrap(), 0.5, "boundary alpha");
}
