//! Dynamic color enum.

use crate::spaces::{
    Cubehelix, Dlab, Dlch, Hpluv, Hsi, Hsl, Hsluv, Hsv, Hwb, Itp, Jab, Jch, Lab, Lab65, Lch, Lch65,
    Lchuv, LinearRgb, Luv, Okhsl, Okhsv, Oklab, Oklch, Prismatic, ProphotoRgb, Rec2020, Rgb, Xyb,
    Xyz50, Xyz65, Yiq, A98, P3,
};
use crate::traits::ColorSpace;

/// Tagged union over every supported color space. Variants are added as each
/// space lands.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Color {
    /// sRGB.
    Rgb(Rgb),
    /// Linear-sRGB.
    LinearRgb(LinearRgb),
    /// HSL (cylindrical sRGB).
    Hsl(Hsl),
    /// HSV (cylindrical sRGB).
    Hsv(Hsv),
    /// HWB (hue/whiteness/blackness).
    Hwb(Hwb),
    /// CIE Lab D50.
    Lab(Lab),
    /// CIE Lab D65.
    Lab65(Lab65),
    /// CIE Lch D50 (polar Lab).
    Lch(Lch),
    /// CIE Lch D65 (polar Lab65).
    Lch65(Lch65),
    /// Oklab (perceptually uniform).
    Oklab(Oklab),
    /// Oklch (polar Oklab).
    Oklch(Oklch),
    /// CIE XYZ D50.
    Xyz50(Xyz50),
    /// CIE XYZ D65.
    Xyz65(Xyz65),
    /// Display P3.
    P3(P3),
    /// Rec. 2020.
    Rec2020(Rec2020),
    /// Adobe RGB (1998).
    A98(A98),
    /// ProPhoto RGB.
    ProphotoRgb(ProphotoRgb),
    /// Cubehelix (Dave Green's astronomical color scheme as a space).
    Cubehelix(Cubehelix),
    /// DIN99o Lab (rectangular form of DIN99o LCh).
    Dlab(Dlab),
    /// DIN99o LCh (polar form).
    Dlch(Dlch),
    /// JzAzBz (HDR perceptual Lab).
    Jab(Jab),
    /// JzCzHz (polar form of JzAzBz).
    Jch(Jch),
    /// NTSC Y'IQ.
    Yiq(Yiq),
    /// HSI (Hue/Saturation/Intensity).
    Hsi(Hsi),
    /// HSLuv (perceptually uniform HSL).
    Hsluv(Hsluv),
    /// HPLuv (perceptually uniform HSL, pastel).
    Hpluv(Hpluv),
    /// OkHSL (Oklab-derived HSL).
    Okhsl(Okhsl),
    /// OkHSV (Oklab-derived HSV).
    Okhsv(Okhsv),
    /// ICtCp (HDR perceptual, Rec. BT.2100).
    Itp(Itp),
    /// XYB (JPEG XL).
    Xyb(Xyb),
    /// CIELUV (D50).
    Luv(Luv),
    /// CIELChuv (polar form of CIELUV).
    Lchuv(Lchuv),
    /// Prismatic (intensity + barycentric chromaticity, Hauke 2009).
    /// culors extension; not in culori 4.0.2.
    Prismatic(Prismatic),
}

impl From<Rgb> for Color {
    fn from(c: Rgb) -> Self {
        Color::Rgb(c)
    }
}

impl From<LinearRgb> for Color {
    fn from(c: LinearRgb) -> Self {
        Color::LinearRgb(c)
    }
}

impl From<Hsl> for Color {
    fn from(c: Hsl) -> Self {
        Color::Hsl(c)
    }
}

impl From<Hsv> for Color {
    fn from(c: Hsv) -> Self {
        Color::Hsv(c)
    }
}

impl From<Hwb> for Color {
    fn from(c: Hwb) -> Self {
        Color::Hwb(c)
    }
}

impl From<Lab> for Color {
    fn from(c: Lab) -> Self {
        Color::Lab(c)
    }
}

impl From<Lab65> for Color {
    fn from(c: Lab65) -> Self {
        Color::Lab65(c)
    }
}

impl From<Lch> for Color {
    fn from(c: Lch) -> Self {
        Color::Lch(c)
    }
}

impl From<Lch65> for Color {
    fn from(c: Lch65) -> Self {
        Color::Lch65(c)
    }
}

impl From<Oklab> for Color {
    fn from(c: Oklab) -> Self {
        Color::Oklab(c)
    }
}

impl From<Oklch> for Color {
    fn from(c: Oklch) -> Self {
        Color::Oklch(c)
    }
}

impl From<Xyz50> for Color {
    fn from(c: Xyz50) -> Self {
        Color::Xyz50(c)
    }
}

impl From<Xyz65> for Color {
    fn from(c: Xyz65) -> Self {
        Color::Xyz65(c)
    }
}

impl From<P3> for Color {
    fn from(c: P3) -> Self {
        Color::P3(c)
    }
}

impl From<Rec2020> for Color {
    fn from(c: Rec2020) -> Self {
        Color::Rec2020(c)
    }
}

impl From<A98> for Color {
    fn from(c: A98) -> Self {
        Color::A98(c)
    }
}

impl From<ProphotoRgb> for Color {
    fn from(c: ProphotoRgb) -> Self {
        Color::ProphotoRgb(c)
    }
}

impl From<Cubehelix> for Color {
    fn from(c: Cubehelix) -> Self {
        Color::Cubehelix(c)
    }
}

impl From<Dlab> for Color {
    fn from(c: Dlab) -> Self {
        Color::Dlab(c)
    }
}

impl From<Dlch> for Color {
    fn from(c: Dlch) -> Self {
        Color::Dlch(c)
    }
}

impl From<Jab> for Color {
    fn from(c: Jab) -> Self {
        Color::Jab(c)
    }
}

impl From<Jch> for Color {
    fn from(c: Jch) -> Self {
        Color::Jch(c)
    }
}

impl From<Yiq> for Color {
    fn from(c: Yiq) -> Self {
        Color::Yiq(c)
    }
}

impl From<Hsi> for Color {
    fn from(c: Hsi) -> Self {
        Color::Hsi(c)
    }
}

impl From<Hsluv> for Color {
    fn from(c: Hsluv) -> Self {
        Color::Hsluv(c)
    }
}

impl From<Hpluv> for Color {
    fn from(c: Hpluv) -> Self {
        Color::Hpluv(c)
    }
}

impl From<Okhsl> for Color {
    fn from(c: Okhsl) -> Self {
        Color::Okhsl(c)
    }
}

impl From<Okhsv> for Color {
    fn from(c: Okhsv) -> Self {
        Color::Okhsv(c)
    }
}

impl From<Itp> for Color {
    fn from(c: Itp) -> Self {
        Color::Itp(c)
    }
}

impl From<Xyb> for Color {
    fn from(c: Xyb) -> Self {
        Color::Xyb(c)
    }
}

impl From<Luv> for Color {
    fn from(c: Luv) -> Self {
        Color::Luv(c)
    }
}

impl From<Lchuv> for Color {
    fn from(c: Lchuv) -> Self {
        Color::Lchuv(c)
    }
}

impl From<Prismatic> for Color {
    fn from(c: Prismatic) -> Self {
        Color::Prismatic(c)
    }
}

impl Color {
    /// Returns the culori `mode` string for this color's underlying space
    /// (`"rgb"`, `"lab"`, `"oklch"`, etc.). Identical to the corresponding
    /// space struct's [`ColorSpace::MODE`].
    pub fn mode(&self) -> &'static str {
        match self {
            Color::Rgb(_) => Rgb::MODE,
            Color::LinearRgb(_) => LinearRgb::MODE,
            Color::Hsl(_) => Hsl::MODE,
            Color::Hsv(_) => Hsv::MODE,
            Color::Hwb(_) => Hwb::MODE,
            Color::Lab(_) => Lab::MODE,
            Color::Lab65(_) => Lab65::MODE,
            Color::Lch(_) => Lch::MODE,
            Color::Lch65(_) => Lch65::MODE,
            Color::Oklab(_) => Oklab::MODE,
            Color::Oklch(_) => Oklch::MODE,
            Color::Xyz50(_) => Xyz50::MODE,
            Color::Xyz65(_) => Xyz65::MODE,
            Color::P3(_) => P3::MODE,
            Color::Rec2020(_) => Rec2020::MODE,
            Color::A98(_) => A98::MODE,
            Color::ProphotoRgb(_) => ProphotoRgb::MODE,
            Color::Cubehelix(_) => Cubehelix::MODE,
            Color::Dlab(_) => Dlab::MODE,
            Color::Dlch(_) => Dlch::MODE,
            Color::Jab(_) => Jab::MODE,
            Color::Jch(_) => Jch::MODE,
            Color::Yiq(_) => Yiq::MODE,
            Color::Hsi(_) => Hsi::MODE,
            Color::Hsluv(_) => Hsluv::MODE,
            Color::Hpluv(_) => Hpluv::MODE,
            Color::Okhsl(_) => Okhsl::MODE,
            Color::Okhsv(_) => Okhsv::MODE,
            Color::Itp(_) => Itp::MODE,
            Color::Xyb(_) => Xyb::MODE,
            Color::Luv(_) => Luv::MODE,
            Color::Lchuv(_) => Lchuv::MODE,
            Color::Prismatic(_) => Prismatic::MODE,
        }
    }
}

/// Error returned when [`TryFrom<Color>`] is asked for a space that does
/// not match the color's underlying variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorVariantMismatch {
    /// The mode that was requested (target space).
    pub expected: &'static str,
    /// The mode of the [`Color`] that was supplied.
    pub actual: &'static str,
}

impl core::fmt::Display for ColorVariantMismatch {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Color variant mismatch: expected `{}`, found `{}`",
            self.expected, self.actual
        )
    }
}

impl std::error::Error for ColorVariantMismatch {}

macro_rules! impl_try_from_color {
    ($($variant:ident => $ty:ty),* $(,)?) => {
        $(
            impl TryFrom<Color> for $ty {
                type Error = ColorVariantMismatch;
                fn try_from(c: Color) -> Result<Self, Self::Error> {
                    match c {
                        Color::$variant(inner) => Ok(inner),
                        other => Err(ColorVariantMismatch {
                            expected: <$ty>::MODE,
                            actual: other.mode(),
                        }),
                    }
                }
            }
        )*
    };
}

impl_try_from_color! {
    Rgb => Rgb,
    LinearRgb => LinearRgb,
    Hsl => Hsl,
    Hsv => Hsv,
    Hwb => Hwb,
    Lab => Lab,
    Lab65 => Lab65,
    Lch => Lch,
    Lch65 => Lch65,
    Oklab => Oklab,
    Oklch => Oklch,
    Xyz50 => Xyz50,
    Xyz65 => Xyz65,
    P3 => P3,
    Rec2020 => Rec2020,
    A98 => A98,
    ProphotoRgb => ProphotoRgb,
    Cubehelix => Cubehelix,
    Dlab => Dlab,
    Dlch => Dlch,
    Jab => Jab,
    Jch => Jch,
    Yiq => Yiq,
    Hsi => Hsi,
    Hsluv => Hsluv,
    Hpluv => Hpluv,
    Okhsl => Okhsl,
    Okhsv => Okhsv,
    Itp => Itp,
    Xyb => Xyb,
    Luv => Luv,
    Lchuv => Lchuv,
    Prismatic => Prismatic,
}
