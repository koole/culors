//! Color-space implementations. One module per space.

mod hsl;
mod hsv;
mod lab;
mod lrgb;
mod rgb;
mod xyz50;
mod xyz65;

pub use hsl::Hsl;
pub use hsv::Hsv;
pub use lab::Lab;
pub use lrgb::LinearRgb;
pub use rgb::Rgb;
pub use xyz50::Xyz50;
pub use xyz65::Xyz65;
