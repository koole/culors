# culor

[![crates.io](https://img.shields.io/crates/v/culor.svg)](https://crates.io/crates/culor)
[![docs.rs](https://docs.rs/culor/badge.svg)](https://docs.rs/culor)
[![CI](https://github.com/koole/culor/actions/workflows/ci.yml/badge.svg)](https://github.com/koole/culor/actions/workflows/ci.yml)

A Rust port of [culori](https://github.com/evercoder/culori), the JavaScript color library by Dan Burzo. Color spaces, conversion, CSS Color Module 4 parsing and formatting, interpolation, gamut mapping, ΔE, blending, averaging, WCAG contrast, and CSS filters. Output values match culori 4.0.2 within 1e-10 across an exhaustive fixture set.

## Features (v1.0)

| Feature | Coverage |
|---|---|
| Color spaces (30) | rgb, lrgb, hsl, hsv, hwb, lab (D50), lch, oklab, oklch, xyz50, xyz65, p3, rec2020, a98, prophoto-rgb, cubehelix, dlab, dlch, jab, jch, yiq, hsi, hsluv, hpluv, okhsl, okhsv, itp, xyb, luv, lchuv |
| Conversion | generic `convert<A, B>` plus direct `From` impls between adjacent spaces |
| CSS parser | named colors, hex, functional `rgb`/`hsl`/`hwb`/`lab`/`lch`/`oklab`/`oklch`, `color()` with `srgb`/`srgb-linear`/`xyz`/`xyz-d50`/`xyz-d65`/`display-p3`/`rec2020`/`a98-rgb`/`prophoto-rgb`, plus `color-mix()` |
| CSS formatter | round-trip stable for canonical CSS Color Module 4 forms, including wide-gamut `color()` profiles |
| Interpolation | `interpolate` / `interpolate_with`, hue-fixup (shorter / longer / increasing / decreasing / raw), per-channel easing |
| Gamut mapping | `in_gamut`, `clamp_gamut`, `clamp_chroma`, `to_gamut` (CSS Color Module 4 with ΔE OK) |
| ΔE | `ciede76`, `ciede94`, `ciede2000`, `cmc`, `euclidean`, `hue_chroma`, `hue_saturation`, `ok`, `jz`, `itp`, `euclidean_xyz` |
| Blending | 12 separable modes (normal, multiply, screen, hard-light, overlay, darken, lighten, color-dodge, color-burn, soft-light, difference, exclusion) |
| Averaging | `average`, `average_number`, `average_angle` (mode-aware, hue-circular) |
| WCAG | `wcag_luminance`, `wcag_contrast` |
| CSS filters | `brightness`, `contrast`, `grayscale`, `hue-rotate`, `invert`, `saturate`, `sepia`, plus CVD `prot` / `deuter` / `trit` |
| Fixture coverage | 110 conversion pairs, 365 parse cases, 303 format round-trips, all verified against culori 4.0.2 |

## Installation

```toml
[dependencies]
culor = "1"
```

With serde support:

```toml
[dependencies]
culor = { version = "1", features = ["serde"] }
```

## Quick start

```rust
use culor::{blend, format_css, parse, BlendMode, Color};

let red = parse("#ff0000").unwrap();
let blue = parse("rgb(0 0 255 / 0.5)").unwrap();
let mixed = blend(&[red, blue], BlendMode::Multiply);
let css = format_css(&mixed);
assert!(css.starts_with("color(srgb"));
```

Convert through the generic `convert` function, or use a direct `From`
impl when bit-for-bit culori parity matters:

```rust
use culor::{convert, Color};
use culor::spaces::{Lab, Oklch, Rgb};

let red = Rgb { r: 1.0, g: 0.0, b: 0.0, alpha: None };

// Generic — routes through XYZ D65, ~1e-14 drift from culori.
let lab_via_hub: Lab = convert(red);

// Direct — matches culori's per-pair routing exactly, including
// the achromatic snap on grayscale inputs.
let lab_direct: Lab = Lab::from(red);

// Cylindrical: oklch with hue fixup for grayscale.
let oklch: Oklch = Oklch::from(red);
assert!(!oklch.l.is_nan());
```

Interpolate between two colors in Oklab and sample at `t = 0.5`:

```rust
use culor::{interpolate, parse};

let a = parse("oklch(70% 0.15 30deg)").unwrap();
let b = parse("oklch(70% 0.15 200deg)").unwrap();
let ramp = interpolate(&[a, b], "oklab");
let mid = ramp(0.5);
let _ = mid;
```

## Comparison to culori

Every public function in culori 4.0.2 has a culor equivalent, with the
exceptions listed under "v1.0 known divergences" below. The mapping is
direct enough that culori code translates almost mechanically: `culori
.parse(s)` becomes `culor::parse(s)`, `culori.convert(c, mode)` becomes
either the generic `convert::<_, T>()` or a direct `T::from(c)`, and
the curried difference / interpolate / blend factories return Rust
closures with the same shape.

| culori function | culor equivalent |
|---|---|
| `parse(str)` | `parse(&str)` |
| `formatCss(c)` | `format_css(&c)` |
| `converter(mode)` | `convert::<_, T>()` or `T::from(c)` |
| `interpolate(colors, mode)` | `interpolate(&colors, mode)` |
| `inGamut(mode)` / `clampRgb` / `clampChroma` / `toGamut` | `in_gamut`, `clamp_gamut`, `clamp_chroma`, `to_gamut` |
| `differenceCiede76` … `differenceItp` | `difference_ciede76` … `difference_itp` |
| `blend` | `blend`, `blend_str` |
| `average` | `average`, `average_number`, `average_angle` |
| `wcagLuminance` / `wcagContrast` | `wcag_luminance`, `wcag_contrast` |
| `filterBrightness` … `filterDeficiencyTrit` | `filter_brightness` … `filter_deficiency_trit` |
| `colorsNamed` table | `parse(name)` (built-in) |

## v1.0 known divergences from culori

- The generic `convert::<A, B>()` always routes through XYZ D65, even
  when culori's public `converter(mode)` API takes a shorter path.
  Output drifts from culori by ~1e-14, well below any practical color
  tolerance, but not bit-for-bit. For bit-exact parity, use the direct
  `From` impls (`Rgb` ↔ `LinearRgb`, `Rgb` ↔ `Hsl`, `Rgb` ↔ `Hsv`,
  `Hsv` ↔ `Hwb`, `LinearRgb` ↔ `Oklab`, `Oklab` ↔ `Oklch`,
  `Xyz50` ↔ `Lab`, `Lab` ↔ `Lch`, plus the four achromatic-snap paths
  `Rgb` → `Lab` / `Lch` / `Oklab` / `Oklch`).
- culori's `convertRgbToLab` and `convertRgbToOklab` snap `a` and `b`
  to exactly zero when `r == g == b`. The XYZ-hub path in
  `convert::<>()` leaves a residual on the order of 1e-6 (Lab) or
  1e-16 (Oklab) and feeds a phantom hue into `Lch` / `Oklch`. The
  direct `From` impls perform the snap; the generic does not.
- `Prismatic` color space is not implemented. culori has it but no
  canonical numerical reference exists outside culori itself; it can
  be added in a 1.x point release without breaking changes.
- Non-separable blend modes (`hue`, `saturation`, `color`,
  `luminosity`) are absent. culori 4.0.2 does not implement them.
- `interpolate` and `average` accept the eleven 0.1 spaces only
  (`Rgb`, `LinearRgb`, `Hsl`, `Hsv`, `Hwb`, `Lab`, `Lch`, `Oklab`,
  `Oklch`, `Xyz50`, `Xyz65`). Calls in the long-tail spaces panic
  with an unsupported-mode message. culori 4.0.2 has the same gap
  for several of these spaces.

## Documentation

API reference on [docs.rs](https://docs.rs/culor). Release history is
in [CHANGELOG.md](CHANGELOG.md).

## Contributing

Bug reports, fixture additions, and color-space implementations are
welcome. The fixture generators under `fixtures-gen/` consume culori
4.0.2 directly, so any drift between Rust and JavaScript output
surfaces immediately when you regenerate. Run `npm run gen-fixtures &&
npm run gen-parse-fixtures && npm run gen-format-fixtures` before
opening a PR; CI fails if regeneration produces a diff.

## License

Dual-licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
