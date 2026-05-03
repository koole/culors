//! Random color generation, mirroring culori 4.0.2's `random.js`.
//!
//! culori's API is `random(mode = 'rgb', constraints = {})`: it picks a
//! uniform random value in each channel's natural range, optionally
//! overridden per channel by `constraints`. culors mirrors the surface in
//! two functions:
//!
//! - [`random`] — random color in the given mode using each channel's
//!   default range.
//! - [`random_with_constraints`] — random color with per-channel
//!   overrides. Each constraint is `(channel_name, (min, max))`. A
//!   constraint with `min == max` produces a fixed value.
//!
//! By design these are *not* reproducible across calls: culori uses
//! `Math.random()`, and culors uses a thread-local xorshift64 seeded
//! from `SystemTime` plus a per-thread counter. There is no public
//! seeding hook because culori has none — callers that need
//! reproducibility should sample channel values themselves and build
//! the `Color` directly.
//!
//! Channel ranges are taken from each space's `definition.js`. For
//! channels that culori's mode definitions leave unspecified (e.g.
//! `s`/`l` of `hsl`), culori's `useMode` registration falls back to
//! `[0, 1]`. culors mirrors the same fallback.

use crate::spaces::{
    Cubehelix, Dlab, Dlch, Hpluv, Hsi, Hsl, Hsluv, Hsv, Hwb, Itp, Jab, Jch, Lab, Lab65, Lch, Lch65,
    Lchuv, LinearRgb, Luv, Okhsl, Okhsv, Oklab, Oklch, Prismatic, ProphotoRgb, Rec2020, Rgb, Xyb,
    Xyz50, Xyz65, Yiq, A98, P3,
};
use crate::Color;
use std::cell::Cell;
use std::time::{SystemTime, UNIX_EPOCH};

thread_local! {
    static RNG_STATE: Cell<u64> = Cell::new(seed_from_clock());
}

fn seed_from_clock() -> u64 {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0x9E3779B97F4A7C15);
    // Mix the thread's identity in to differentiate concurrent threads
    // that observe the same clock tick.
    let tid_mix = std::thread::current().id();
    let tid_hash = {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut h = DefaultHasher::new();
        tid_mix.hash(&mut h);
        h.finish()
    };
    let mut s = nanos ^ tid_hash;
    if s == 0 {
        s = 0x9E3779B97F4A7C15;
    }
    s
}

/// Single xorshift64 step, returning a uniform u64.
fn next_u64() -> u64 {
    RNG_STATE.with(|c| {
        let mut x = c.get();
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        c.set(x);
        x
    })
}

/// Uniform `f64` in `[0, 1)` — mirrors `Math.random()`.
fn next_f64() -> f64 {
    // Take the top 53 bits and divide by 2^53.
    ((next_u64() >> 11) as f64) / (1u64 << 53) as f64
}

#[inline]
fn rand_in(min: f64, max: f64) -> f64 {
    min + next_f64() * (max - min)
}

/// Channel descriptor for a mode: list of channel names (excluding
/// `alpha`) and their default ranges.
struct ModeRanges {
    channels: &'static [&'static str],
    ranges: &'static [(f64, f64)],
}

/// Channel ranges per mode, mirroring culori's `definition.js` plus the
/// `[0, 1]` fallback applied by `useMode` for channels with no explicit
/// range. Order in `channels` matches `ranges` index-wise.
fn mode_ranges(mode: &str) -> Option<ModeRanges> {
    Some(match mode {
        // RGB-likes — every channel falls back to [0, 1].
        "rgb" | "lrgb" | "p3" | "a98" | "rec2020" | "prophoto" => ModeRanges {
            channels: &["r", "g", "b"],
            ranges: &[(0.0, 1.0), (0.0, 1.0), (0.0, 1.0)],
        },
        // HSL family.
        "hsl" => ModeRanges {
            channels: &["h", "s", "l"],
            ranges: &[(0.0, 360.0), (0.0, 1.0), (0.0, 1.0)],
        },
        "hsv" => ModeRanges {
            channels: &["h", "s", "v"],
            ranges: &[(0.0, 360.0), (0.0, 1.0), (0.0, 1.0)],
        },
        "hwb" => ModeRanges {
            channels: &["h", "w", "b"],
            ranges: &[(0.0, 360.0), (0.0, 1.0), (0.0, 1.0)],
        },
        "hsi" => ModeRanges {
            channels: &["h", "s", "i"],
            ranges: &[(0.0, 360.0), (0.0, 1.0), (0.0, 1.0)],
        },
        "hsluv" | "hpluv" => ModeRanges {
            channels: &["h", "s", "l"],
            ranges: &[(0.0, 360.0), (0.0, 100.0), (0.0, 100.0)],
        },
        "okhsl" | "okhsv" => ModeRanges {
            channels: &["h", "s", "l"],
            ranges: &[(0.0, 360.0), (0.0, 1.0), (0.0, 1.0)],
        },
        // CIE Lab / Lch.
        "lab" | "lab65" => ModeRanges {
            channels: &["l", "a", "b"],
            ranges: &[(0.0, 100.0), (-125.0, 125.0), (-125.0, 125.0)],
        },
        "lch" | "lch65" => ModeRanges {
            channels: &["l", "c", "h"],
            ranges: &[(0.0, 100.0), (0.0, 150.0), (0.0, 360.0)],
        },
        "luv" => ModeRanges {
            channels: &["l", "u", "v"],
            ranges: &[(0.0, 100.0), (-84.936, 175.042), (-125.882, 87.243)],
        },
        "lchuv" => ModeRanges {
            channels: &["l", "c", "h"],
            ranges: &[(0.0, 100.0), (0.0, 176.956), (0.0, 360.0)],
        },
        // Oklab / Oklch.
        "oklab" => ModeRanges {
            channels: &["l", "a", "b"],
            ranges: &[(0.0, 1.0), (-0.4, 0.4), (-0.4, 0.4)],
        },
        "oklch" => ModeRanges {
            channels: &["l", "c", "h"],
            ranges: &[(0.0, 1.0), (0.0, 0.4), (0.0, 360.0)],
        },
        // DIN99o.
        "dlab" => ModeRanges {
            channels: &["l", "a", "b"],
            ranges: &[(0.0, 100.0), (-40.09, 45.501), (-40.469, 44.344)],
        },
        "dlch" => ModeRanges {
            channels: &["l", "c", "h"],
            ranges: &[(0.0, 100.0), (0.0, 51.484), (0.0, 360.0)],
        },
        // JzAzBz / JzCzHz.
        "jab" => ModeRanges {
            channels: &["j", "a", "b"],
            ranges: &[(0.0, 0.222), (-0.109, 0.129), (-0.185, 0.134)],
        },
        "jch" => ModeRanges {
            channels: &["j", "c", "h"],
            ranges: &[(0.0, 0.221), (0.0, 0.19), (0.0, 360.0)],
        },
        // ICtCp.
        "itp" => ModeRanges {
            channels: &["i", "t", "p"],
            ranges: &[(0.0, 0.581), (-0.369, 0.272), (-0.164, 0.331)],
        },
        // YIQ. culori only declares ranges for `i` and `q`; `y` falls back
        // to [0, 1].
        "yiq" => ModeRanges {
            channels: &["y", "i", "q"],
            ranges: &[(0.0, 1.0), (-0.595, 0.595), (-0.522, 0.522)],
        },
        // XYZ.
        "xyz50" => ModeRanges {
            channels: &["x", "y", "z"],
            ranges: &[(0.0, 0.964), (0.0, 0.999), (0.0, 0.825)],
        },
        "xyz65" => ModeRanges {
            channels: &["x", "y", "z"],
            ranges: &[(0.0, 0.95), (0.0, 1.0), (0.0, 1.088)],
        },
        // XYB.
        "xyb" => ModeRanges {
            channels: &["x", "y", "b"],
            ranges: &[(-0.0154, 0.0281), (0.0, 0.8453), (-0.2778, 0.388)],
        },
        // Cubehelix.
        "cubehelix" => ModeRanges {
            channels: &["h", "s", "l"],
            ranges: &[(0.0, 360.0), (0.0, 4.614), (0.0, 1.0)],
        },
        // Prismatic — culors extension; no culori counterpart. Channels
        // are L plus three prismatic intensities, all in [0, 1].
        "prismatic" => ModeRanges {
            channels: &["l", "r", "g", "b"],
            ranges: &[(0.0, 1.0), (0.0, 1.0), (0.0, 1.0), (0.0, 1.0)],
        },
        _ => return None,
    })
}

/// Sample value for a single channel; constraint takes precedence.
fn sample_channel(name: &str, default: (f64, f64), constraints: &[(&str, (f64, f64))]) -> f64 {
    let (min, max) = constraints
        .iter()
        .find(|(k, _)| *k == name)
        .map(|(_, r)| *r)
        .unwrap_or(default);
    rand_in(min, max)
}

/// Generate a random color in `mode`, using each channel's natural
/// range. Mirrors culori 4.0.2's `random(mode)`.
///
/// `mode` must be a recognized mode string; unknown modes panic. The
/// alpha channel is *not* randomized unless overridden via
/// [`random_with_constraints`] — culori's default behavior.
///
/// ```rust
/// use culors::random;
///
/// let c = random("rgb");
/// match c {
///     culors::Color::Rgb(rgb) => {
///         assert!((0.0..=1.0).contains(&rgb.r));
///         assert!((0.0..=1.0).contains(&rgb.g));
///         assert!((0.0..=1.0).contains(&rgb.b));
///         assert!(rgb.alpha.is_none());
///     }
///     _ => unreachable!(),
/// }
/// ```
pub fn random(mode: &str) -> Color {
    random_with_constraints(mode, &[])
}

/// Like [`random`], but each entry in `constraints` overrides the
/// natural range for the named channel. An entry `("alpha", (a, b))`
/// also turns alpha generation on (matching culori's behavior:
/// "ignore alpha if not present in constraints").
///
/// Constraint ranges are not validated against the channel's natural
/// range; they are passed through verbatim, just as culori does.
pub fn random_with_constraints(mode: &str, constraints: &[(&str, (f64, f64))]) -> Color {
    let ModeRanges { channels, ranges } =
        mode_ranges(mode).unwrap_or_else(|| panic!("random: unknown mode '{mode}'"));

    // Build values vector channel-by-channel.
    let mut values = [0.0_f64; 4];
    for (i, ch) in channels.iter().enumerate() {
        values[i] = sample_channel(ch, ranges[i], constraints);
    }

    // Alpha: only sampled if the caller passed an `alpha` constraint.
    let alpha = constraints
        .iter()
        .find(|(k, _)| *k == "alpha")
        .map(|(_, (lo, hi))| rand_in(*lo, *hi));

    construct_color(mode, &values, alpha)
}

fn construct_color(mode: &str, v: &[f64; 4], alpha: Option<f64>) -> Color {
    match mode {
        "rgb" => Color::Rgb(Rgb {
            r: v[0],
            g: v[1],
            b: v[2],
            alpha,
        }),
        "lrgb" => Color::LinearRgb(LinearRgb {
            r: v[0],
            g: v[1],
            b: v[2],
            alpha,
        }),
        "p3" => Color::P3(P3 {
            r: v[0],
            g: v[1],
            b: v[2],
            alpha,
        }),
        "a98" => Color::A98(A98 {
            r: v[0],
            g: v[1],
            b: v[2],
            alpha,
        }),
        "rec2020" => Color::Rec2020(Rec2020 {
            r: v[0],
            g: v[1],
            b: v[2],
            alpha,
        }),
        "prophoto" => Color::ProphotoRgb(ProphotoRgb {
            r: v[0],
            g: v[1],
            b: v[2],
            alpha,
        }),
        "hsl" => Color::Hsl(Hsl {
            h: v[0],
            s: v[1],
            l: v[2],
            alpha,
        }),
        "hsv" => Color::Hsv(Hsv {
            h: v[0],
            s: v[1],
            v: v[2],
            alpha,
        }),
        "hwb" => Color::Hwb(Hwb {
            h: v[0],
            w: v[1],
            b: v[2],
            alpha,
        }),
        "hsi" => Color::Hsi(Hsi {
            h: v[0],
            s: v[1],
            i: v[2],
            alpha,
        }),
        "hsluv" => Color::Hsluv(Hsluv {
            h: v[0],
            s: v[1],
            l: v[2],
            alpha,
        }),
        "hpluv" => Color::Hpluv(Hpluv {
            h: v[0],
            s: v[1],
            l: v[2],
            alpha,
        }),
        "okhsl" => Color::Okhsl(Okhsl {
            h: v[0],
            s: v[1],
            l: v[2],
            alpha,
        }),
        "okhsv" => Color::Okhsv(Okhsv {
            h: v[0],
            s: v[1],
            v: v[2],
            alpha,
        }),
        "lab" => Color::Lab(Lab {
            l: v[0],
            a: v[1],
            b: v[2],
            alpha,
        }),
        "lab65" => Color::Lab65(Lab65 {
            l: v[0],
            a: v[1],
            b: v[2],
            alpha,
        }),
        "lch" => Color::Lch(Lch {
            l: v[0],
            c: v[1],
            h: v[2],
            alpha,
        }),
        "lch65" => Color::Lch65(Lch65 {
            l: v[0],
            c: v[1],
            h: v[2],
            alpha,
        }),
        "lchuv" => Color::Lchuv(Lchuv {
            l: v[0],
            c: v[1],
            h: v[2],
            alpha,
        }),
        "luv" => Color::Luv(Luv {
            l: v[0],
            u: v[1],
            v: v[2],
            alpha,
        }),
        "oklab" => Color::Oklab(Oklab {
            l: v[0],
            a: v[1],
            b: v[2],
            alpha,
        }),
        "oklch" => Color::Oklch(Oklch {
            l: v[0],
            c: v[1],
            h: v[2],
            alpha,
        }),
        "dlab" => Color::Dlab(Dlab {
            l: v[0],
            a: v[1],
            b: v[2],
            alpha,
        }),
        "dlch" => Color::Dlch(Dlch {
            l: v[0],
            c: v[1],
            h: v[2],
            alpha,
        }),
        "jab" => Color::Jab(Jab {
            j: v[0],
            a: v[1],
            b: v[2],
            alpha,
        }),
        "jch" => Color::Jch(Jch {
            j: v[0],
            c: v[1],
            h: v[2],
            alpha,
        }),
        "itp" => Color::Itp(Itp {
            i: v[0],
            t: v[1],
            p: v[2],
            alpha,
        }),
        "yiq" => Color::Yiq(Yiq {
            y: v[0],
            i: v[1],
            q: v[2],
            alpha,
        }),
        "xyz50" => Color::Xyz50(Xyz50 {
            x: v[0],
            y: v[1],
            z: v[2],
            alpha,
        }),
        "xyz65" => Color::Xyz65(Xyz65 {
            x: v[0],
            y: v[1],
            z: v[2],
            alpha,
        }),
        "xyb" => Color::Xyb(Xyb {
            x: v[0],
            y: v[1],
            b: v[2],
            alpha,
        }),
        "cubehelix" => Color::Cubehelix(Cubehelix {
            h: v[0],
            s: v[1],
            l: v[2],
            alpha,
        }),
        "prismatic" => Color::Prismatic(Prismatic {
            l: v[0],
            r: v[1],
            g: v[2],
            b: v[3],
            alpha,
        }),
        _ => unreachable!("mode_ranges already validated `{mode}`"),
    }
}
