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
//! `color()` profiles supported in v0.1: `srgb`, `srgb-linear`, `xyz`,
//! `xyz-d50`, `xyz-d65`. Other profiles (`display-p3`, `rec2020`,
//! `prophoto-rgb`, `a98-rgb`) return `None` until those spaces land.

pub(crate) mod functional;
pub(crate) mod hex;
pub(crate) mod named;
