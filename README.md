# culor

[![crates.io](https://img.shields.io/crates/v/culor.svg)](https://crates.io/crates/culor)
[![docs.rs](https://docs.rs/culor/badge.svg)](https://docs.rs/culor)
[![CI](https://github.com/koole/culor/actions/workflows/ci.yml/badge.svg)](https://github.com/koole/culor/actions/workflows/ci.yml)

A Rust port of [culori](https://github.com/evercoder/culori), the JavaScript color library by Dan Burzo. Color spaces, conversion, CSS Color Module 4 parsing and formatting, with output values matching culori within 1e-10 across an exhaustive fixture set.

## Features (v0.1)

| Feature | Coverage |
|---|---|
| Color spaces | rgb, lrgb, hsl, hsv, hwb, lab (D50), lch, oklab, oklch, xyz50, xyz65 |
| Conversion | generic `convert<A, B>` plus direct `From` impls between adjacent spaces |
| CSS parser | named colors, hex, functional `rgb`/`hsl`/`hwb`/`lab`/`lch`/`oklab`/`oklch`, `color()` with `srgb`/`srgb-linear`/`xyz`/`xyz-d50`/`xyz-d65` |
| CSS formatter | round-trip stable for canonical CSS Color Module 4 forms |
| Fixture coverage | 110 conversion pairs, 365 parse cases, 303 format round-trips, all verified against culori 4.0.2 |

## Installation

```toml
[dependencies]
culor = "0.1"
```

With serde support:

```toml
[dependencies]
culor = { version = "0.1", features = ["serde"] }
```

## Quick start

```rust
use culor::{convert, format_css, parse, Color};
use culor::spaces::Lab;

let parsed = parse("oklch(70% 0.15 30deg)").expect("valid CSS");
let lab: Lab = match parsed {
    Color::Oklch(c) => convert(c),
    _ => unreachable!(),
};
let css = format_css(&Color::Lab(lab));
assert!(css.starts_with("lab("));
```

For pairs where culori takes a shorter routing than XYZ D65 (typically
`Rgb` to `Lab` / `Lch` / `Oklab` / `Oklch`), call the matching `From`
impl directly to get bit-for-bit culori parity:

```rust
use culor::spaces::{Lab, Rgb};

let red = Rgb { r: 1.0, g: 0.0, b: 0.0, alpha: None };
let lab: Lab = Lab::from(red); // achromatic snap applies
```

## v0.1 known divergences from culori

The generic `convert::<A, B>()` always routes through XYZ D65, even when culori's public `converter(mode)` API takes a shorter path. For pairs whose routing differs, output drifts from culori by ~1e-14, well below any practical color tolerance, but not bit-for-bit. Two cases are worth knowing about:

1. **Achromatic snap.** culori's `convertRgbToLab.js` and `convertRgbToOklab.js` snap `a` and `b` to exactly zero when the input is `r == g == b`. The XYZ-hub path leaves a residual on the order of 1e-6 (Lab) or 1e-16 (Oklab) and feeds a phantom hue into `Lch` / `Oklch`. To match culori's public-API output, call `Lab::from(rgb)` / `Oklab::from(rgb)` / `Lch::from(rgb)` / `Oklch::from(rgb)` directly. These `From` impls perform the snap; the generic `convert<>()` does not.

2. **Routing through XYZ for non-XYZ pairs.** `Lab → Xyz50`, `Lab ↔ Lch`, `Xyz50 → Lab`, and similar pairs have direct `From` impls that mirror culori's path. The generic `convert<>()` instead routes through XYZ D65, picking up Bradford-adaptation ULP noise. Use the direct `From` when bit-for-bit culori parity matters.

`color()` profiles for spaces culor has not yet implemented (`display-p3`, `rec2020`, `prophoto-rgb`, `a98-rgb`) return `None` from `parse()`. Those spaces are deferred to v0.4.

v0.2 will add a culori-mirroring `Color::convert_to(target_mode)` API that selects the right path automatically.

## Documentation

API reference on [docs.rs](https://docs.rs/culor). The full design document lives in [`docs/plans/2026-05-03-culor-rust-port-design.md`](docs/plans/2026-05-03-culor-rust-port-design.md). Release history is in [CHANGELOG.md](CHANGELOG.md).

## Contributing

Bug reports, fixture additions, and color-space implementations are welcome. The fixture generators under `fixtures-gen/` consume culori 4.0.2 directly, so any drift between Rust and JavaScript output surfaces immediately when you regenerate. Run `npm run gen-fixtures && npm run gen-parse-fixtures && npm run gen-format-fixtures` before opening a PR; CI fails if regeneration produces a diff.

## License

Dual-licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
