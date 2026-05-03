//! Color-space implementations. One module per space.

mod hsl;
mod hsv;
mod hwb;
mod lab;
mod lch;
mod lrgb;
mod oklab;
mod oklch;
mod rgb;
mod xyz50;
mod xyz65;

pub use hsl::Hsl;
pub use hsv::Hsv;
pub use hwb::Hwb;
pub use lab::Lab;
pub use lch::Lch;
pub use lrgb::LinearRgb;
pub use oklab::Oklab;
pub use oklch::Oklch;
pub use rgb::Rgb;
pub use xyz50::Xyz50;
pub use xyz65::Xyz65;
