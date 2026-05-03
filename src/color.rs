//! Dynamic color enum.

use crate::spaces::{
    Cubehelix, Dlab, Dlch, Hpluv, Hsi, Hsl, Hsluv, Hsv, Hwb, Itp, Jab, Jch, Lab, Lch, LinearRgb,
    Okhsl, Okhsv, Oklab, Oklch, ProphotoRgb, Rec2020, Rgb, Xyb, Xyz50, Xyz65, Yiq, A98, P3,
};

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
    /// CIE Lch D50 (polar Lab).
    Lch(Lch),
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

impl From<Lch> for Color {
    fn from(c: Lch) -> Self {
        Color::Lch(c)
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
