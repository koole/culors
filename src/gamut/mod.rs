//! Gamut mapping: deciding when a color sits inside a target gamut and
//! producing the closest in-gamut color when it does not.
//!
//! Mirrors culori 4.0.2's `clamp.js`. Four entry points:
//!
//! - [`in_gamut`] — boolean predicate, mirrors culori's `inGamut(mode)`.
//! - [`clamp_gamut`] — naïve per-channel clip, mirrors culori's
//!   `clampGamut(mode)`.
//! - [`clamp_chroma`] — chroma-bisection clip in an LCh-like space, mirrors
//!   culori's `clampChroma(color, mode)`.
//! - [`to_gamut`] — CSS Color Module 4 gamut-mapping algorithm, mirrors
//!   culori's `toGamut(dest, mode)`.
//!
//! The `mode` argument names the destination gamut. Modes that carry a
//! gamut definition in culori 4.0.2:
//!
//! - sRGB family (`"rgb"`, `"hsl"`, `"hsv"`, `"hwb"`, `"hsi"`, `"okhsl"`,
//!   `"okhsv"`) — `gamut: 'rgb'`, evaluated on the sRGB unit cube.
//! - linear sRGB (`"lrgb"`) — `gamut: true`, evaluated on its own channels.
//! - wide-gamut RGB (`"p3"`, `"rec2020"`, `"a98"`, `"prophoto"`) — each
//!   defines its own `[0, 1]` linear-RGB box.
//!
//! Every other culori mode (`lab`, `lch`, `oklab`, `oklch`, `xyz*`, `jab`,
//! `dlab`, `itp`, `xyb`, `luv`, …) has no gamut field, so `in_gamut`
//! returns `true` and `clamp_gamut` returns the input unchanged. The three
//! culors-only modes (`hsluv`, `hpluv`, `prismatic`) share rgb's gamut box
//! since they have no culori entry to reference.
//!
//! Truly unknown mode strings degrade through the rgb gamut rather than
//! panicking; v1.5 dropped the panic from earlier versions.

mod clamp;
mod in_gamut;
mod to_gamut;

pub use clamp::{clamp_chroma, clamp_gamut};
pub use in_gamut::in_gamut;
pub use to_gamut::to_gamut;

use crate::Color;

/// Returns `true` if `color` is in the sRGB gamut.
///
/// Mirrors culori 4.0.2's `displayable(color)` (`node_modules/culori/src/clamp.js`):
/// equivalent to `in_gamut(color, "rgb")`. Convenience alias for callers
/// that don't want to spell out the mode string.
pub fn displayable(color: &Color) -> bool {
    in_gamut(color, "rgb")
}

/// Clamps `color` into the sRGB gamut by per-channel clipping.
///
/// Mirrors culori 4.0.2's `clampRgb(color)`: equivalent to
/// `clamp_gamut(color, "rgb")`. The result is returned in `color`'s
/// original mode.
pub fn clamp_rgb(color: Color) -> Color {
    clamp_gamut(color, "rgb")
}
