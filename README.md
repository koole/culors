# culor

A Rust port of [culori](https://github.com/evercoder/culori), the JavaScript color library by Dan Burzo.

Color spaces, conversion, CSS Color Module 4 parsing and formatting, with output values matching culori within 1e-12.

## Status

v0.1.0 — Foundation. RGB / linear RGB / HSL / HSV / HWB / Lab / LCH / Oklab / Oklch / XYZ D50 / XYZ D65 with CSS Color Module 4 parsing and formatting.

See the [design document](docs/plans/2026-05-03-culor-rust-port-design.md) for the full roadmap.

## v0.1 known divergences from culori

Culor's generic `convert::<A, B>()` always routes through XYZ D65, even when culori's public `converter(mode)` API takes a shorter path. For pairs whose routing differs, culor's output drifts from culori by ~1e-14 — well below any practical color tolerance, but not bit-for-bit. The two cases worth knowing about:

1. **Achromatic-snap**. culori's `convertRgbToLab.js` and `convertRgbToOklab.js` snap `a` and `b` to exactly zero when the input is `r == g == b`. The XYZ-hub path leaves a residual on the order of 1e-6 (Lab) or 1e-16 (Oklab) and feeds a phantom hue into Lch / Oklch. To match culori's public-API output, call `Lab::from(rgb)` / `Oklab::from(rgb)` / `Lch::from(rgb)` / `Oklch::from(rgb)` directly. These From impls perform the snap; the generic `convert<>()` does not.

2. **Routing through XYZ for non-XYZ pairs**. `Lab → Xyz50`, `Lab ↔ Lch`, `Xyz50 → Lab`, and similar pairs have direct `From` impls that mirror culori's path. The generic `convert<>()` instead routes through XYZ D65, picking up Bradford-adaptation ULP noise. Use the direct `From` when bit-for-bit culori parity matters.

v0.2 will add a culori-mirroring `Color::convert_to(target_mode)` API that selects the right path automatically.

## License

Dual-licensed under MIT or Apache-2.0.
