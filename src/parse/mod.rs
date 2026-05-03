//! CSS Color Module 4 string parser.
//!
//! Mirrors culori 4.0.2's parser pipeline (`node_modules/culori/src/parse.js`
//! and the per-space `parse*.js` files). Returns `Option<Color>`; any
//! malformed input yields `None`. No channel-range validation happens here
//! beyond what culori clamps (alpha, lab/lch L, lch/oklch C). Out-of-range
//! values such as `rgb(300 0 0)` round-trip as their unclamped form, which
//! matches culori's behavior.
//!
//! Three syntactic groups feed into the dispatcher:
//!
//! 1. Named colors (case-insensitive) and `transparent`.
//! 2. Hex literals (`#rgb`, `#rgba`, `#rrggbb`, `#rrggbbaa`,
//!    case-insensitive).
//! 3. CSS function calls (`rgb()`, `rgba()`, `hsl()`, `hsla()`, `hwb()`,
//!    `lab()`, `lch()`, `oklab()`, `oklch()`, `color()`). Function names
//!    are case-sensitive in culori; we mirror that.
//!
//! `color()` profiles supported: `srgb`, `srgb-linear`, `xyz`, `xyz-d50`,
//! `xyz-d65`, `display-p3`, `rec2020`, `a98-rgb`, `prophoto-rgb`.

pub(crate) mod color_mix;
pub(crate) mod functional;
pub(crate) mod hex;
pub(crate) mod named;

use crate::color::Color;
use crate::spaces::Rgb;

/// Parse a CSS color string into a [`Color`].
///
/// Returns `None` if the input does not parse as any supported syntax.
/// The grammar matches CSS Color Module 4 with the same set of profiles
/// culori 4.0.2 ships.
///
/// Out-of-range channel values pass through unclamped, mirroring culori.
/// `none` channels become `f64::NAN` for that channel; `none` for alpha
/// becomes `alpha: None`.
pub fn parse(input: &str) -> Option<Color> {
    if let Some(c) = parse_named_or_transparent(input) {
        return Some(Color::Rgb(c));
    }
    if let Some(c) = hex::parse_hex(input) {
        return Some(Color::Rgb(c));
    }
    if let Some(c) = color_mix::parse_color_mix(input) {
        return Some(c);
    }
    functional::parse_functional(input)
}

fn parse_named_or_transparent(input: &str) -> Option<Rgb> {
    if input == "transparent" {
        return Some(Rgb {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            alpha: Some(0.0),
        });
    }
    let lower = input.to_ascii_lowercase();
    let packed = named::lookup(&lower)?;
    Some(unpack_rgb(packed))
}

fn unpack_rgb(packed: u32) -> Rgb {
    Rgb {
        r: ((packed >> 16) & 0xff) as f64 / 255.0,
        g: ((packed >> 8) & 0xff) as f64 / 255.0,
        b: (packed & 0xff) as f64 / 255.0,
        alpha: None,
    }
}
