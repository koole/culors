//! CSS-style image filters.
//!
//! Each filter is a curried factory matching culori 4.0.2's `filter*`
//! exports: `filter_x(amount)` returns a closure that maps a
//! [`crate::Color`] to a new [`crate::Color`]. The returned color is
//! always a [`crate::Color::Rgb`] in nominal `[0, 1]` channel range,
//! without clipping. Alpha is preserved.
//!
//! culori's matrices and constants are reproduced verbatim. See the
//! per-module docs for the exact references.

mod adjust;
mod common;
mod deficiency;

pub use adjust::{
    filter_brightness, filter_contrast, filter_grayscale, filter_hue_rotate, filter_invert,
    filter_saturate, filter_sepia,
};
pub use deficiency::{filter_deficiency_deuter, filter_deficiency_prot, filter_deficiency_trit};
