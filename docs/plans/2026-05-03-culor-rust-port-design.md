# Culor — Rust Port of Culori (Design)

**Date**: 2026-05-03
**Status**: Validated, ready for implementation planning
**Crate name**: `culors` (confirmed available on crates.io)
**Source library**: [evercoder/culori](https://github.com/evercoder/culori)

## Goals

Port the JavaScript color library [culori](https://github.com/evercoder/culori) to Rust as a standalone open-source crate published to crates.io. Achieve full feature parity with culori's public API, with output values matching culori's JavaScript output to within a tight floating-point tolerance.

## Decisions

| Topic | Decision |
|---|---|
| Purpose | Standalone open-source crate, published to crates.io |
| Precision target | Tolerance-based equivalence (`(rust - js).abs() < epsilon`); 1e-12 default |
| Test data source | Hybrid: port culori's own test suite + generate randomized fixtures |
| Release strategy | Phased by capability (v0.1 → v0.2 → v0.3 → v0.4 → v1.0) |
| Type model | Hybrid: typed structs + `ColorSpace` trait + `Color` enum for dynamic dispatch |
| Numeric type | f64 only |
| Culori source access | npm devDependency, pinned via `package-lock.json` |

## Repository Layout

Standalone crate, not part of the spectralite-studio Cargo workspace. Worktree: `.worktrees/culors/`.

```
culors/
├── Cargo.toml
├── README.md
├── LICENSE-MIT, LICENSE-APACHE         # dual-license, standard for crates.io
├── package.json                        # culori as devDependency, pinned
├── package-lock.json
├── src/
│   ├── lib.rs                          # public API re-exports
│   ├── color.rs                        # Color enum
│   ├── traits.rs                       # ColorSpace trait
│   ├── spaces/
│   │   ├── mod.rs
│   │   ├── rgb.rs                      # one file per color space
│   │   ├── lab.rs
│   │   └── ...
│   ├── parse/                          # CSS Color parsing
│   ├── format/                         # CSS Color formatting
│   ├── convert.rs                      # XYZ-hub conversions
│   └── util.rs                         # math helpers
├── tests/
│   ├── ported/                         # culori's own tests, ported 1:1
│   └── fixtures/                       # generated JSON fixtures
├── fixtures-gen/
│   ├── generate.mjs                    # Node script: culori → JSON fixtures
│   └── inputs.mjs                      # input matrix definitions
└── benches/                            # criterion benchmarks (added later)
```

**Toolchain**: stable Rust, MSRV target 1.75+. No `unsafe`. No `no_std` for v0.x. Minimal dependencies; `serde` behind a feature flag for fixture deserialization.

## Core Types

### `ColorSpace` trait

Backbone of the type system. Every color space implements this trait. Conversions compose through XYZ D65 (matching culori's hub model).

```rust
pub trait ColorSpace: Sized + Copy + Clone + PartialEq {
    const MODE: &'static str;            // matches culori's mode strings
    const CHANNELS: &'static [&'static str];

    fn alpha(&self) -> Option<f64>;
    fn with_alpha(self, alpha: Option<f64>) -> Self;

    fn to_xyz65(&self) -> Xyz65;
    fn from_xyz65(xyz: Xyz65) -> Self;
}
```

A free function `convert<A, B>(c: A) -> B where A: ColorSpace, B: ColorSpace` provides the typed entry point. Spaces that have direct (non-XYZ) conversions for precision (e.g., Lab ↔ LCH) provide them via dedicated `From` impls; the convert function picks the shorter path when available.

### Per-space structs

One struct per color space, in `src/spaces/<name>.rs`. Channels stored in their natural unit (RGB in 0..1, Lab L in 0..100, etc.) — same as culori. No silent normalization.

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rgb {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub alpha: Option<f64>,
}
```

`alpha: Option<f64>` mirrors culori's distinction between "no alpha defined" and `alpha: 1.0`. Required for round-tripping CSS strings: `rgb(255 0 0)` and `rgb(255 0 0 / 1)` must format back to themselves.

### `Color` enum

Dynamic dispatch when the color space is determined at runtime (CSS parsing, mixed-space interpolation, generic culori-style APIs).

```rust
pub enum Color {
    Rgb(Rgb), Hsl(Hsl), Hsv(Hsv), Lab(Lab),
    Lch(Lch), Oklab(Oklab), Oklch(Oklch), /* ... */
}
```

Implements `From<Rgb>`, `From<Lab>`, etc. for ergonomic construction.

## Test Strategy

### Suite 1 — Ported culori tests (`tests/ported/`)

Each file in culori's `test/` directory has a Rust counterpart. Inputs and expected outputs hardcoded inline, lifted from the JS test:

```rust
// tests/ported/lab.rs — mirrors culori/test/lab.test.js
#[test]
fn lab_white() {
    let c = parse("white").unwrap();
    let lab: Lab = convert(c);
    assert_close(lab.l, 100.0, 1e-12);
    assert_close(lab.a, 0.0, 1e-12);
    assert_close(lab.b, 0.0, 1e-12);
}
```

Helper module: `assert_close(actual, expected, eps)` and `assert_color_close(actual: &Color, expected: &Color, eps)`.

### Suite 2 — Generated fixture tests (`tests/fixtures/`)

`fixtures-gen/generate.mjs` produces JSON files. Inputs: deterministic boundary values (0, 1, NaN, ±infinity) crossed with random samples from a fixed-seed PRNG. Outputs: every conversion path, every parser case, every difference function.

```
tests/fixtures/
├── convert_rgb_to_lab.json
├── convert_lab_to_oklch.json
└── parse_css.json
```

A single Rust integration test per fixture file deserializes and asserts every row.

### Epsilon defaults

| Path | Epsilon |
|---|---|
| Direct single conversion | `1e-12` |
| Chained conversion (≥2 hops) | `1e-10` |
| Transcendental-heavy (LCH ↔ Lab, atan2) | `1e-9` |

Epsilon is documented at each assertion site.

### TDD workflow per feature

1. Read culori's `src/<space>/` and `test/<space>.test.js`
2. Port the JS tests to `tests/ported/<space>.rs` — they fail (no impl)
3. Add the struct + `ColorSpace` impl until ported tests pass
4. Add the space's inputs to `fixtures-gen/inputs.mjs`, regenerate fixtures
5. Run fixture tests; if they fail, the algorithm has numerical drift — fix
6. Commit ported tests + impl + regenerated fixtures together

## Phased Release Plan

Every phase ends with: ported tests pass, fixture tests pass, README documents available surface, CHANGELOG entry written.

### v0.1 — Foundation (~3-4 weeks)

- `ColorSpace` trait + `Color` enum
- Color spaces: `Rgb`, `LinearRgb`, `Hsl`, `Hsv`, `Hwb`, `Lab` (D50), `Lch`, `Oklab`, `Oklch`, `Xyz50`, `Xyz65`
- Conversions through XYZ hub
- CSS Color parser (Color Module 4: named, hex, `rgb()`, `hsl()`, `lab()`, `lch()`, `oklab()`, `oklch()`, `color(display-p3 ...)`)
- CSS Color formatter (round-trip stable)
- Fixture generator infrastructure
- Covers ~80% of typical user needs

### v0.2 — Interpolation & gamut (~2-3 weeks)

- `interpolate(colors, mode, options)` with hue-fixup strategies (shorter, longer, increasing, decreasing)
- Per-channel easing functions
- Gamut mapping: `inGamut`, `clampChroma`, `clampGamut`, `toGamut`
- CSS `color-mix()` semantics

### v0.3 — Difference & blending (~2 weeks)

- deltaE variants: 76, CMC, 94, 2000, JZ, OK, ITP, EuclideanXyz
- `differenceHueChroma`, `differenceHueSaturation`, etc.
- Blend modes (multiply, screen, overlay, etc.) — Porter-Duff + separable blends
- `average*` functions

### v0.4 — Long tail (~3-4 weeks)

- Remaining color spaces: `P3`, `Rec2020`, `A98`, `ProphotoRgb`, `Cubehelix`, `Dlab`, `Dlch`, `Jab`, `Jch`, `Yiq`, `Hsi`, `Hsluv`, `Hpluv`, `Okhsl`, `Okhsv`, `Itp`, `Xyb`, `Lrgb-linear`, `Luv`, `Lchuv`, `Prismatic`
- Filters: `filterBrightness`, `filterContrast`, `filterDeficiencyDeuter`, `filterGrayscale`, `filterHueRotate`, `filterInvert`, `filterSaturate`, `filterSepia`
- `formatCss`, `formatHex`, `formatHsl`, `formatRgb` parity
- WCAG contrast functions

### v1.0

Released when feature parity is verified by full fixture sweep (every culori public function has ≥1 fixture test) with no known divergences from culori in tracked issues.

## Risks

1. **Numerical drift through XYZ hub**. Culori sometimes uses direct space-to-space conversions for precision. The trait permits override — spaces with optimized direct paths can implement `From<Other>` directly.

2. **CSS parser surface area**. CSS Color Module 4 + 5 is large (relative colors, `color-mix()`, `color()` with custom profiles). v0.1 ships Module 4 only; Module 5 deferred to v0.2 alongside the interpolation engine.

3. **Culori updates during the port**. Pin culori in `package-lock.json` to a specific version. Document that version in README. Bumping is a deliberate task: re-run fixtures, fix drift, release.

4. **NaN handling for powerless channels**. Culori uses `NaN` to mark powerless components (hue is undefined for achromatic colors). Tests must use `is_nan()` checks, not equality, where this applies.

## Resolved

- **Crate name**: `culors` confirmed available on crates.io.
- **`serde` integration**: shipped in v0.1 behind a `serde` feature flag (off by default). Implements `Serialize`/`Deserialize` on every color struct and the `Color` enum.

## Open Questions

- **MSRV policy**: lock to current stable at v0.1, bump in major versions only.

## Next Step

Create `.worktrees/culors/` worktree and produce v0.1 implementation plan via `superpowers:writing-plans`.
