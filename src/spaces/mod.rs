//! Color-space implementations. One module per space.

mod lrgb;
mod rgb;
mod xyz50;
mod xyz65;

pub use lrgb::LinearRgb;
pub use rgb::Rgb;
pub use xyz50::Xyz50;
pub use xyz65::Xyz65;
