//! WCAG 2.1 luminance and contrast helpers. Mirrors culori 4.0.2's
//! `wcag.js` (`luminance`, `contrast`).

mod wcag;

pub use wcag::{wcag_contrast, wcag_luminance};
