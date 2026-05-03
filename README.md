# culors

[![crates.io](https://img.shields.io/crates/v/culors.svg)](https://crates.io/crates/culors)
[![docs.rs](https://docs.rs/culors/badge.svg)](https://docs.rs/culors)
[![CI](https://github.com/koole/culors/actions/workflows/ci.yml/badge.svg)](https://github.com/koole/culors/actions/workflows/ci.yml)

A Rust port of [culori](https://github.com/evercoder/culori), the JavaScript color library by Dan Burzo. Color spaces, conversion, CSS Color Module 4 parsing and formatting, interpolation, gamut mapping, ΔE, blending, averaging, WCAG contrast, and CSS filters. Output values match culori 4.0.2 within 1e-10 across an exhaustive fixture set.

Used in production by [Spectralite](https://spectralite.studio), a lighting-control application whose UI relies on culori. We needed the Rust render core to produce the same colors as the JS layer, end to end — culors is the result.

## Features (v1.2)

| Feature | Coverage |
|---|---|
| Color spaces (33) | rgb, lrgb, hsl, hsv, hwb, lab (D50), lch (D50), lab65, lch65, oklab, oklch, xyz50, xyz65, p3, rec2020, a98, prophoto-rgb, cubehelix, dlab, dlch, jab, jch, yiq, hsi, hsluv, hpluv, okhsl, okhsv, itp, xyb, luv, lchuv, prismatic |
| Conversion | three flavors: direct `From` (typed, zero-overhead, bit-exact culori parity), generic `convert<A, B>` (typed, always XYZ-D65 hub, ~1e-14 drift), and dynamic `Color::convert_to(mode)` / typed `convert_culori<A, B>` (culori's per-pair routing — closes the 1e-14 gap with byte-for-byte parity) |
| CSS parser | named colors, hex, functional `rgb`/`hsl`/`hwb`/`lab`/`lch`/`oklab`/`oklch`, `color()` with `srgb`/`srgb-linear`/`xyz`/`xyz-d50`/`xyz-d65`/`display-p3`/`rec2020`/`a98-rgb`/`prophoto-rgb`/`--lab-d65`/`--lch-d65`, plus `color-mix()` |
| CSS formatter | round-trip stable for canonical CSS Color Module 4 forms, including wide-gamut `color()` profiles and the `--lab-d65` / `--lch-d65` custom profiles |
| Interpolation | `interpolate` / `interpolate_with` over rgb, lrgb, hsl, hsv, hwb, lab, lab65, lch, lch65, oklab, oklch, xyz50, xyz65, p3, rec2020, a98, prophoto, cubehelix, dlab, dlch, jab, jch, yiq, hsi, hsluv, hpluv, okhsl, okhsv, itp, xyb, luv, lchuv, prismatic. Hue-fixup (shorter / longer / increasing / decreasing / raw), per-channel easing |
| Gamut mapping | `in_gamut`, `clamp_gamut`, `clamp_chroma`, `to_gamut` (CSS Color Module 4 with ΔE OK) |
| ΔE | `ciede76`, `ciede94`, `ciede2000`, `cmc`, `euclidean`, `hyab`, `hue_chroma`, `hue_saturation`, `hue_naive`, `ok`, `jz`, `itp`, `euclidean_xyz`, `kotsarenko_ramos` |
| Blending | 16 modes — 12 separable (normal, multiply, screen, hard-light, overlay, darken, lighten, color-dodge, color-burn, soft-light, difference, exclusion) plus 4 non-separable from CSS Compositing 1 § 5.8 (hue, saturation, color, luminosity) |
| Averaging | `average`, `average_number`, `average_angle` (mode-aware, hue-circular). Same mode list as `interpolate` |
| Palette utilities | `samples(n)`, `nearest(palette, metric)`, `round(places)` |
| WCAG | `wcag_luminance`, `wcag_contrast` |
| CSS filters | `brightness`, `contrast`, `grayscale`, `hue-rotate`, `invert`, `saturate`, `sepia`, plus CVD `prot` / `deuter` / `trit` |
| Fixture coverage | 110 conversion pairs, 365 parse cases, 303 format round-trips, all verified against culori 4.0.2 |

## Installation

```toml
[dependencies]
culors = "1"
```

With serde support:

```toml
[dependencies]
culors = { version = "1", features = ["serde"] }
```

## Quick start

```rust
use culors::{blend, format_css, parse, BlendMode, Color};

let red = parse("#ff0000").unwrap();
let blue = parse("rgb(0 0 255 / 0.5)").unwrap();
let mixed = blend(&[red, blue], BlendMode::Multiply);
let css = format_css(&mixed);
assert!(css.starts_with("color(srgb"));
```

Convert in three flavors, each with different precision/ergonomic
trade-offs:

```rust
use culors::convert::convert_culori;
use culors::{convert, Color};
use culors::spaces::{Lab, Oklch, Rgb};

let red = Rgb { r: 1.0, g: 0.0, b: 0.0, alpha: None };

// 1. Direct `From` — typed, zero overhead, bit-for-bit culori parity on
//    pairs where the impl exists. Best when both spaces are known at
//    compile time.
let lab_direct: Lab = Lab::from(red);

// 2. Generic `convert<A, B>` — typed, simple semantics, always routes
//    through XYZ D65. ~1e-14 drift versus culori on pairs where culori
//    takes a shorter path; back-compatible with v1.0 / v1.1 callers.
let lab_via_hub: Lab = convert(red);

// 3. `Color::convert_to` (dynamic) and `convert_culori<A, B>` (typed
//    wrapper) — match culori's `converter(mode)` dispatch exactly. Per-pair
//    routing closes the 1e-14 gap.
let lab_culori: Lab = convert_culori(red);
let lab_dyn = Color::Rgb(red).convert_to("lab").unwrap();

// Cylindrical: oklch with hue fixup for grayscale.
let oklch: Oklch = Oklch::from(red);
assert!(!oklch.l.is_nan());
```

`Color::convert_to` returns `None` when the target string is not a known
mode; otherwise it produces the same routing culori would. Use it for
CSS tooling, design-tool UIs, and any caller that carries the target
space as a `&str`. Use `convert_culori<A, B>` when the source and target
types are known at compile time but you still want culori's per-pair
routing.

Interpolate between two colors in Oklab and sample at `t = 0.5`:

```rust
use culors::{interpolate, parse};

let a = parse("oklch(70% 0.15 30deg)").unwrap();
let b = parse("oklch(70% 0.15 200deg)").unwrap();
let ramp = interpolate(&[a, b], "oklab");
let mid = ramp(0.5);
let _ = mid;
```

## Comparison to culori

Every public function in culori 4.0.2 has a culors equivalent, with the
exceptions listed under "Known divergences" below. The mapping is
direct enough that culori code translates almost mechanically: `culori
.parse(s)` becomes `culors::parse(s)`, `culori.convert(c, mode)` becomes
either the generic `convert::<_, T>()` or a direct `T::from(c)`, and
the curried difference / interpolate / blend factories return Rust
closures with the same shape.

| culori function | culors equivalent |
|---|---|
| `parse(str)` | `parse(&str)` |
| `formatCss(c)` | `format_css(&c)` |
| `converter(mode)` | `Color::convert_to(mode)`, `convert_culori::<_, T>()`, `convert::<_, T>()`, or `T::from(c)` |
| `interpolate(colors, mode)` | `interpolate(&colors, mode)` |
| `inGamut(mode)` / `clampRgb` / `clampChroma` / `toGamut` | `in_gamut`, `clamp_gamut`, `clamp_chroma`, `to_gamut` |
| `differenceCiede76` … `differenceItp` | `difference_ciede76` … `difference_itp` |
| `blend` | `blend`, `blend_str` |
| `average` | `average`, `average_number`, `average_angle` |
| `wcagLuminance` / `wcagContrast` | `wcag_luminance`, `wcag_contrast` |
| `filterBrightness` … `filterDeficiencyTrit` | `filter_brightness` … `filter_deficiency_trit` |
| `colorsNamed` table | `parse(name)` (built-in) |

## Known divergences from culori

- `Prismatic` follows the Hauke 2009 definition because culori 4.0.2
  ships no `prismatic` mode against which to fixture-test. The
  literature contains other definitions under the same name; this
  one is documented as a culors extension rather than a culori port.
  `interpolate` and `average` accept `"prismatic"` and operate on the
  four channels directly; the reference values for those tests are
  hand-computed rather than derived from culori.

## Documentation

API reference on [docs.rs](https://docs.rs/culors). Release history is
in [CHANGELOG.md](CHANGELOG.md).

## Contributing

Bug reports, fixture additions, and color-space implementations are
welcome. The fixture generators under `fixtures-gen/` consume culori
4.0.2 directly, so any drift between Rust and JavaScript output
surfaces immediately when you regenerate. Run `npm run gen-fixtures &&
npm run gen-parse-fixtures && npm run gen-format-fixtures` before
opening a PR; CI fails if regeneration produces a diff.

## License

[MIT](LICENSE).
