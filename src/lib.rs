//! culor — Rust port of culori.
//!
//! See README for a high-level overview and the design document
//! at `docs/plans/2026-05-03-culor-rust-port-design.md`.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod color;
pub mod convert;
pub mod format;
pub mod parse;
pub mod spaces;
pub mod traits;
pub(crate) mod util;

pub use color::Color;
pub use convert::convert;
pub use format::format_css;
pub use parse::parse;
pub use traits::ColorSpace;
