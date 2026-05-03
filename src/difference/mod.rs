//! Color-difference (ΔE) factories. Mirrors culori 4.0.2's
//! `difference.js` family, plus forward-looking placeholders for v0.4
//! spaces (`difference_jz`, `difference_itp`). Each public function
//! returns a closure that computes the configured metric on a pair of
//! colors.
//!
//! Every factory takes its parameters once and returns a `Fn(&Color,
//! &Color) -> f64`, matching culori's curried API:
//!
//! ```rust
//! use culor::{difference_ciede2000, parse};
//! let de = difference_ciede2000(1.0, 1.0, 1.0);
//! let red = parse("red").unwrap();
//! let blue = parse("blue").unwrap();
//! let _delta = de(&red, &blue);
//! ```
//!
//! Variants whose underlying color space is scheduled for v0.4
//! ([`difference_jz`], [`difference_itp`]) compile but return `f64::NAN`.

mod ciede2000;
mod ciede76;
mod ciede94;
mod cmc;
mod euclidean;
pub(crate) mod extract;
mod hue;
mod stub;

pub use ciede2000::difference_ciede2000;
pub use ciede76::difference_ciede76;
pub use ciede94::{difference_ciede94, difference_ciede94_with};
pub use cmc::difference_cmc;
pub use euclidean::{
    difference_euclidean, difference_euclidean_with, difference_euclidean_xyz, difference_ok,
};
pub use hue::{difference_hue_chroma, difference_hue_saturation};
pub use stub::{difference_itp, difference_jz};
