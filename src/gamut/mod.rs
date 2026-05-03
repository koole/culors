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
//! gamut definition in culori: `"rgb"`, `"hsl"`, `"hsv"`, `"hwb"`, plus
//! the four wide-gamut RGB profiles (`"p3"`, `"rec2020"`, `"a98"`,
//! `"prophoto"`). Every other mode returns `true` from `in_gamut` and is a
//! no-op for `clamp_gamut`. The gamut for the cylindrical sRGB modes
//! (`hsl` / `hsv` / `hwb`) is `"rgb"`, so those four mode strings collapse
//! to the same boundary check; each wide-gamut profile defines its own
//! `[0, 1]` linear-RGB box.

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
