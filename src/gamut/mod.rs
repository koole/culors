//! Gamut mapping: deciding when a color sits inside a target gamut and
//! producing the closest in-gamut color when it does not.
//!
//! Mirrors culori 4.0.2's `clamp.js`. Three entry points so far:
//!
//! - [`in_gamut`] — boolean predicate, mirrors culori's `inGamut(mode)`.
//! - [`clamp_gamut`] — naïve per-channel clip, mirrors culori's
//!   `clampGamut(mode)`.
//! - [`clamp_chroma`] — chroma-bisection clip in an LCh-like space, mirrors
//!   culori's `clampChroma(color, mode)`.
//!
//! The `mode` argument names the destination gamut. Only `"rgb"`, `"hsl"`,
//! `"hsv"`, and `"hwb"` carry a gamut definition in culori; every other mode
//! returns `true` from `inGamut` and is a no-op for `clamp_gamut`. The
//! gamut for the cylindrical sRGB modes (`hsl` / `hsv` / `hwb`) is
//! `"rgb"`, so the four mode strings collapse to the same boundary check.

mod clamp;
mod in_gamut;

pub use clamp::{clamp_chroma, clamp_gamut};
pub use in_gamut::in_gamut;
