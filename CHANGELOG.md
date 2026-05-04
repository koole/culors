# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.5.0]

### Added

- `interpolator_piecewise(f)` — higher-order channel-interpolator factory
  matching culori 4.0.2's `interpolatorPiecewise`. Given a per-segment
  function `f(a, b, t)`, returns a `ChannelInterpFactory` that builds a
  per-channel sampler over a stop slice. Slots into
  `InterpolateOptions::channel_interpolator` so callers can mix custom
  interpolators with the default linear-piecewise one.
- `converter(mode)` — reusable-closure factory matching culori 4.0.2's
  `converter`. Returns `Some(impl Fn(&Color) -> Color)` for known modes
  and `None` otherwise. The closure repeats `Color::convert_to` per call
  but is cheap to keep around and reuse, removing the per-call mode-string
  validation from hot loops.
- `interpolate::fixup_alpha` — promoted from `pub(crate)` to `pub`. Mirrors
  culori's `fixupAlpha` (`fixup/alpha.js`) so callers building custom
  interpolation pipelines can apply the same alpha-handling rule the
  built-in `interpolate` family uses.

### Fixed

- `in_gamut` and `clamp_gamut` no longer panic on modes outside the sRGB
  family or the four wide-gamut RGB profiles. The full culori 4.0.2 mode
  table is now respected: `lab`/`lch`/`oklab`/`oklch`/`xyz*`/`jab`/`jch`/
  `dlab`/`dlch`/`itp`/`xyb`/`luv`/`lchuv`/`cubehelix`/`yiq`/`lab65`/`lch65`
  return `true` / pass through (no `gamut` field), while `hsi`/`okhsl`/
  `okhsv`/`hsluv`/`hpluv`/`prismatic` route through the rgb gamut box.
  `lrgb` now checks its own linear-RGB unit cube. Truly unknown mode
  strings degrade through rgb instead of panicking.

## [1.4.0]

### Added

- Seven spline channel-interpolator factories matching culori's
  `interpolate/splineBasis.js`, `splineNatural.js`, and `splineMonotone.js`:
  `interpolator_spline_basis` (and `_closed`), `interpolator_spline_natural`
  (and `_closed`), `interpolator_spline_monotone`,
  `interpolator_spline_monotone_2`, `interpolator_spline_monotone_closed`.
  Each factory builds a per-channel sampler `Fn(f64) -> f64` over the stops
  and returns a `ChannelInterpFactory` so it slots into
  `InterpolateOptions::channel_interpolator`.
- `InterpolateOptions::channel_interpolators` — per-channel
  interpolator-factory map. When set for a named channel (or `"alpha"`),
  it replaces the default linear sampler with the supplied factory.
- `interpolate_with_premultiplied_alpha(colors, mode, options)` — mirrors
  culori's `interpolateWithPremultipliedAlpha`. Pre-multiplies each input
  color's non-alpha channels by its alpha before lerping, divides at the
  output. Boundary semantics match culori: at `t = 0` and `t = 1` the
  *original* boundary color is divided by its alpha (so a stop with
  `(r=1, alpha=0.5)` returns `r=2` at the boundary).
- `in_gamut`, `clamp_gamut`, and `to_gamut` accept the four wide-gamut RGB
  profiles `"p3"`, `"rec2020"`, `"a98"`, `"prophoto"`. Each profile's
  gamut boundary is the `[0, 1]` linear-RGB box in that space, matching
  culori's `clampGamut('p3' | 'rec2020' | 'a98' | 'prophoto')`.

### Stability

As of v1.4 culors is in maintenance/upstream-tracking mode. The library
tracks culori 4.0.2 with feature parity for everything Rust-idiomatic.
New features land in [culori](https://github.com/Evercoder/culori) first;
once accepted upstream they're mirrored here. Bug fixes and culori
version bumps remain in scope.

## [1.3.0]

### Added

- Six easing factories matching culori's `easing/` family: `easing_midpoint`,
  `easing_smoothstep`, `easing_smoothstep_inverse`, `easing_smootherstep`,
  `easing_in_out_sine`, `easing_gamma`. Each returns a closure
  `Fn(f64) -> f64`, so they compose with [`samples_with_easing`] and the
  per-channel `easing` field of `InterpolateOptions`.
- `samples_with_easing(n, easing)` — culori's `samples(n, γ)` generalised
  to any easing curve. The existing `samples(n)` keeps its linear shape.
- Scalar interpolation utilities `lerp`, `unlerp`, `blerp`, `trilerp`,
  matching culori's `interpolate/lerp.js` argument order so per-bit results
  agree across platforms.
- `mapper`, `map_alpha_multiply`, `map_alpha_divide`, `map_transfer_linear`,
  `map_transfer_gamma` — the per-channel transfer pipeline from culori's
  `map.js`. `mapper` accepts a `preserve_mode` flag that round-trips the
  result back to the source color's mode, matching culori's third
  positional argument.
- `format_hex(&Color) -> String` and `format_hex8(&Color) -> String` —
  legacy `#rrggbb` / `#rrggbbaa` serializers mirroring culori's `formatHex`
  / `formatHex8`. The input is converted to sRGB; channels are clamped to
  0..=255 with `Math.round`-style rounding (NaN → 0).
- `format_rgb(&Color) -> String` — legacy comma-form `rgb(R, G, B)` /
  `rgba(R, G, B, A)` serializer mirroring culori's `formatRgb`. Alpha is
  rendered with up to two decimal places; opaque colors omit the alpha
  component.
- `format_hsl(&Color) -> String` — legacy comma-form `hsl(H, S%, L%)` /
  `hsla(H, S%, L%, A)` serializer mirroring culori's `formatHsl`. Hue
  passes through unchanged when finite (negatives and values >360 are
  preserved); saturation and lightness are clamped to [0, 100] %.
- `displayable(color)` and `clamp_rgb(color)` — convenience aliases for
  `in_gamut(color, "rgb")` and `clamp_gamut(color, "rgb")`, mirroring
  culori 4.0.2's same-named entry points in `clamp.js`.
- `random(mode)` and `random_with_constraints(mode, constraints)` —
  uniform random color generators mirroring culori's `random.js`.
  Channel ranges follow each space's `definition.js` with culori's
  `[0, 1]` fallback for unranged channels. Alpha is opt-in via an
  `alpha` constraint. The PRNG is a thread-local xorshift64 seeded
  from `SystemTime` and the thread id; reproducibility is intentionally
  not exposed because culori's `Math.random()` has no public seed
  either.

### Fixed

- `Lab65` and `Luv` reference white points are now expressed as runtime
  divisions (`0.3127 / 0.329` and friends) instead of precomputed
  literals, matching JS bit-for-bit and harmonising with the existing
  convention in `util.rs`. A regression test in `ported_lab65.rs`
  feeds the runtime D65 white through `Lab65::from(Xyz65)` and pins
  the result at `(L=100, a=0, b=0)`.
- `ported_jch.rs` now covers the PQ_inv negative-value guard on
  `convertJabToXyz65`. culori clamps `v < 0` to `0` because the inverse
  PQ formula raises a fractional power of a sign-dependent quantity;
  without the clamp, a Jch input with negative `j` produces NaN. The
  regression tests pin the all-zero XYZ65 output that culori produces
  for negative-`j` inputs both with and without chroma.
- `ported_xyz65.rs` extended with sRGB primary / white round-trip and
  identity checks (the file was previously skeletal).

## [1.2.0] - 2026-05-03

Adds dynamic-mode conversion that mirrors culori's `converter(mode)`
dispatch exactly, closing the precision gap between the existing
`convert<A, B>()` (always routes through XYZ D65) and culori's per-pair
shortest-path routing.

### Added

- `Color::convert_to(target_mode: &str) -> Option<Color>` — runtime
  dispatch keyed on a culori mode string. Returns `None` for unrecognized
  modes; otherwise picks the same conversion path culori does (a direct
  edge if culori's `converters` table has one, else `source → rgb →
  target`). Handy for CSS tooling, design-tool UIs, and any caller that
  carries the target space as a `&str`.
- `convert_culori<A, B>(c: A) -> B` — typed wrapper around
  `Color::convert_to` for callers who prefer compile-time typing while
  still wanting culori's per-pair routing. Mirrors
  `culori.converter(B::MODE)(c)`.
- `Color::mode() -> &'static str` — returns the culori mode string for
  the current variant, matching the corresponding `ColorSpace::MODE`.
- `TryFrom<Color>` for every space struct (`Rgb`, `Lab`, `Oklch`, …),
  unwrapping the matching variant or returning `ColorVariantMismatch`.

### Removed limitations

- `convert<A, B>()` continues to route through XYZ D65 unchanged. The
  previously documented ~1e-14 drift on pairs where culori takes a
  shorter path is now closeable: callers who need byte-for-byte culori
  parity should use `Color::convert_to` or `convert_culori`. The two
  APIs coexist; existing `convert<>` callers are unaffected.

### Internal

- New fixture suite `tests/fixtures/convert_to/` (870 ordered pairs ×
  6 inputs) pins `Color::convert_to` against culori's `converter(mode)`
  output across every culori-known space pair.

## [1.1.0] - 2026-05-03

Closes the public-API gap with culori 4.0.2 by adding the D65 Lab/Lch
pair and three small utilities that round out the difference and palette
toolset. No breaking changes from 1.0.0; everything below is additive.

### Added

- `Lab65` and `Lch65` color spaces (CIE Lab and CIE Lch with the D65
  illuminant). Both implement `ColorSpace`, are reachable through the
  generic `convert()` hub, and round-trip through CSS using the
  `color(--lab-d65 …)` / `color(--lch-d65 …)` syntax. Direct
  `Rgb` → `Lab65` / `Lch65` impls carry the same achromatic snap as
  `Lab::from(Rgb)`.
- `difference_hyab()` — HyAB color difference (`|ΔL| + √(Δa² + Δb²)`)
  computed in `lab65`. Matches culori's `differenceHyab`.
- `difference_hue_naive(mode)` — signed angular hue distance, exposing
  the previously private `HueDiffKind::Naive` reducer. Mirrors culori's
  `differenceHueNaive`.
- `difference_kotsarenko_ramos()` — convenience wrapper for
  `differenceEuclidean('yiq', [0.5053, 0.299, 0.1957])`.
- `samples(n)` — `n` evenly spaced values in `[0, 1]`. Matches
  culori's `samples(n)` for `n ≥ 0`.
- `round(places)` — factory returning `Fn(f64) -> f64` that rounds to
  the requested decimal places using `Math.round`-style half-away-from-zero
  rounding.
- `nearest(palette, metric)` — palette-search factory returning
  `Fn(&Color, usize) -> Vec<Color>`, the closest `n` colors under the
  chosen metric (Euclidean by default).
- `Prismatic` color space (Hauke 2009 definition: a four-channel
  `(L, r, g, b)` decomposition where `L = max(r, g, b)` and the
  remaining channels are normalized chromatic components). Multiple
  competing definitions appear in the literature; this implementation
  follows Hauke's original 2009 paper and is documented as a culors
  extension. culori 4.0.2 does not ship a `prismatic` definition, so
  the round-trip is verified against an internal reference rather
  than fixture parity.
- Non-separable blend modes `BlendMode::Hue`, `BlendMode::Saturation`,
  `BlendMode::Color`, `BlendMode::Luminosity`, implementing the
  luminance-preserving formulas in CSS Compositing Level 1 § 5.8.
  `blend_str` accepts the `"hue"`, `"saturation"`, `"color"`, and
  `"luminosity"` keys. Not in culori 4.0.2; cross-checked against the
  CSS spec and `colorjs.io`.
- `interpolate`, `interpolate_with`, and `average` extended to cover
  every long-tail color space added in v0.4: `cubehelix`, `dlab`,
  `dlch`, `jab`, `jch`, `yiq`, `hsi`, `hsluv`, `hpluv`, `okhsl`,
  `okhsv`, `itp`, `xyb`, `luv`, `lchuv`, `p3`, `rec2020`, `a98`,
  `prophoto`, plus `lab65`, `lch65`, and `prismatic`. Mode-specific
  channel layouts (rectangular vs. cylindrical, hue position,
  alpha-as-NaN missing marker) match culori where culori implements
  them. `hsluv`, `hpluv`, and `prismatic` remain culors extensions
  because culori 4.0.2 omits them from `interpolate`.

### Changed

- Generalized `interpolate` and `average` internal channel arrays
  from fixed 3-channel to variable-size, enabling `lab65`, `lch65`,
  and `prismatic` as interpolation modes. Public API unchanged.

### Removed limitations

The following items were called out as deferred in the 1.0.0
"Limitations" section and are now closed:

- `Prismatic` color space is implemented (see Added).
- Non-separable blend modes are implemented (see Added).
- `interpolate` and `average` no longer panic on the v0.4 long-tail
  modes, and `lab65` / `lch65` / `prismatic` are now accepted as
  interpolation and averaging modes alongside the rest.

The remaining 1.0.0 limitation still stands:

- `convert::<A, B>()` routes through XYZ D65 for any pair without a
  direct `From` impl, which differs from culori's per-pair routing
  by approximately 1e-14. The direct `From` impls listed in the
  `convert` module docs provide bit-for-bit parity for the
  precision-critical pairs.

## [1.0.0] - 2026-05-03

First stable release. The public API matches culori 4.0.2 across color
spaces, parsing, formatting, conversion, interpolation, gamut mapping,
ΔE, blending, averaging, contrast, and CSS filters, with the documented
exceptions in the Limitations section below.

### Added (since 0.1.0)

#### Color spaces

Beyond the eleven shipped in 0.1.0 (`Rgb`, `LinearRgb`, `Hsl`, `Hsv`,
`Hwb`, `Lab`, `Lch`, `Oklab`, `Oklch`, `Xyz50`, `Xyz65`), 1.0.0 adds 19
spaces. Each implements the `ColorSpace` trait and round-trips through
the XYZ D65 hub.

- Wide-gamut RGB families: `P3` (Display P3), `Rec2020`, `A98`
  (Adobe RGB 1998), `ProphotoRgb`. The CSS `color()` parser and
  formatter accept `display-p3`, `rec2020`, `a98-rgb`, and
  `prophoto-rgb` profiles.
- DIN99o family: `Dlab` (DIN99o Lab) and `Dlch` (DIN99o LCh polar).
- JzAzBz family: `Jab` (JzAzBz) and `Jch` (JzCzHz polar), feeding
  `difference_jz`.
- ICtCp: `Itp` (Rec. BT.2100), feeding `difference_itp`.
- JPEG XL: `Xyb`.
- Cubehelix (Green 2011 sequential ramp).
- HSLuv family: `Hsluv` and `Hpluv` (perceptually uniform HSL/HPL).
- Oklab-derived cylindrical: `Okhsl` and `Okhsv`.
- CIELUV: `Luv` and `Lchuv`.
- NTSC: `Yiq`.
- HSI: hue/saturation/intensity.

#### Interpolation

- `interpolate(colors, mode)` returns a closure `Fn(f64) -> Color`
  that produces the interpolated color at `t ∈ [0, 1]` in the chosen
  space.
- `interpolate_with(colors, mode, options)` exposes per-channel
  easing and hue-fixup configuration.
- Hue-fixup strategies for cylindrical spaces: `Shorter` (CSS Color
  Module 4 default), `Longer`, `Increasing`, `Decreasing`, `Raw`.
- Multi-stop interpolation with even spacing of intermediate stops.

#### Gamut mapping

- `in_gamut(color)`: predicate.
- `clamp_gamut(color)`: per-channel clamp.
- `clamp_chroma(color, mode)`: chroma binary search in LCh-like
  spaces.
- `to_gamut(color, mode, jnd)`: the CSS Color Module 4 gamut-mapping
  algorithm using ΔE OK as the perceptual delta.

#### CSS color-mix()

- `parse()` accepts `color-mix(in <space> [<hue-method> hue]?, <c1>
  <p1>?, <c2> <p2>?)` for `srgb`, `srgb-linear`, `hsl`, `hwb`, `lab`,
  `lch`, `oklab`, `oklch`, `xyz`, `xyz-d50`, `xyz-d65`.
- Hue methods: `shorter` (default), `longer`, `increasing`,
  `decreasing`. Rejected on rectangular spaces.
- Implementation follows the W3C CSS Color Module 5 § 11 algorithm:
  percentage normalization, premultiplied interpolation, post-mix
  alpha scaling when the percentage sum is below 100. culori 4.0.2
  does not ship `color-mix()`, so reference values come from a
  hand-rolled spec port cross-checked against `colorjs.io`.

#### ΔE color difference

Each function returns a closure `Fn(&Color, &Color) -> f64`, mirroring
culori's curried API.

- `difference_ciede76()` — Euclidean distance in D65 Lab.
- `difference_ciede94(textiles)` and
  `difference_ciede94_with(kL, K1, K2)` — graphic-arts and textile
  parametric variants.
- `difference_ciede2000(kL, kC, kH)` — Sharma/Wu/Dalal 2005.
- `difference_cmc(l, c)` — CMC l:c with the `T` hue-region branch.
- `difference_euclidean(mode)`,
  `difference_euclidean_with(mode, weights)`, and
  `difference_euclidean_xyz()`.
- `difference_hue_chroma(mode)` and
  `difference_hue_saturation(mode)` — signed polar-hue distance for
  LCh-likes and HSx-likes respectively.
- `difference_ok()` — Oklab Euclidean.
- `difference_jz()` — JzAzBz Euclidean.
- `difference_itp()` — ICtCp scaled distance per Rec. BT.2124.

#### Blending

- `blend(colors, mode)` and string-keyed `blend_str(colors, mode)`.
- `BlendMode` covers every separable mode from CSS Compositing Level
  1 § 5.7: `Normal`, `Multiply`, `Screen`, `HardLight`, `Overlay`,
  `Darken`, `Lighten`, `ColorDodge`, `ColorBurn`, `SoftLight`,
  `Difference`, `Exclusion`.
- Inputs convert to sRGB; missing alphas default to 1; the stack
  folds left-to-right with Porter-Duff source-over. Output is always
  `Color::Rgb`. Per-mode formulae match culori 4.0.2's `src/blend.js`
  byte-for-byte, including its overlay branch (which differs from
  the CSS spec's swap-of-hard-light definition).

#### Averaging

- `average(colors, mode)` reduces in `mode`: hue channels (`h`) use
  the circular mean, every other channel including alpha uses the
  arithmetic mean.
- `average_number(values)` and `average_angle(angles)` helpers,
  treating `NaN` as the missing marker (culori uses `undefined`).
  `average_angle` returns `[0, 360]` per the wrap-around branch in
  culori.

#### WCAG contrast

- `wcag_luminance(color)` — relative luminance from sRGB.
- `wcag_contrast(a, b)` — `(L1 + 0.05) / (L2 + 0.05)`.

#### CSS filters

Ten filters mirroring culori's `src/filter.js`. Each takes an amount
and returns a closure `Fn(&Color) -> Color`.

- `filter_brightness`, `filter_contrast`, `filter_grayscale`,
  `filter_hue_rotate`, `filter_invert`, `filter_saturate`,
  `filter_sepia` (CSS Filter Effects 1).
- Color-vision-deficiency filters: `filter_deficiency_prot`,
  `filter_deficiency_deuter`, `filter_deficiency_trit` (Brettel /
  Machado, matching culori's coefficients).

### Changed

- `Rgb → Lab` / `Lch` / `Oklab` / `Oklch` `From` impls perform
  culori's achromatic snap (zero `a`/`b` and `NaN` hue when
  `r == g == b`). The generic `convert::<Rgb, _>()` does not. This
  was added in 0.1.0 but is worth reiterating: callers that need
  byte-for-byte parity with culori's public `converter()` API must
  use the direct `From` impls listed in the `convert` module docs.
- `parse()` now dispatches `color-mix()` syntax through a separate
  expression parser. Earlier dev versions of 0.2 rejected
  hue-interpolation-method on rectangular spaces silently; 1.0
  surfaces `None` consistently.

### Limitations

- `Prismatic` color space deferred. culori implements it but no
  canonical numerical reference is available outside culori itself,
  and the implementation is straightforward enough that a future
  release can add it without breaking changes.
- Non-separable blend modes (`hue`, `saturation`, `color`,
  `luminosity`) are absent because culori 4.0.2 does not implement
  them. They are listed in CSS Compositing Level 1 § 5.8.
- `interpolate` and `average` accept only the eleven 0.1 spaces
  (`Rgb`, `LinearRgb`, `Hsl`, `Hsv`, `Hwb`, `Lab`, `Lch`, `Oklab`,
  `Oklch`, `Xyz50`, `Xyz65`). Calls in the long-tail spaces panic
  with an unsupported-mode message. culori 4.0.2 has the same gap
  for several of these spaces.
- `convert::<A, B>()` always routes through XYZ D65, which differs
  from culori's per-pair routing by approximately 1e-14. The direct
  `From` impls (`Rgb` ↔ `LinearRgb`, `Rgb` ↔ `Hsl`, `Rgb` ↔ `Hsv`,
  `Hsv` ↔ `Hwb`, `LinearRgb` ↔ `Oklab`, `Oklab` ↔ `Oklch`,
  `Xyz50` ↔ `Lab`, `Lab` ↔ `Lch`, plus the four achromatic-snap
  paths above) provide byte-for-byte parity for the precision-
  critical pairs.

## [0.1.0] - 2026-05-03

### Added

- Eleven color spaces: `Rgb`, `LinearRgb`, `Hsl`, `Hsv`, `Hwb`, `Lab` (D50), `Lch` (D50), `Oklab`, `Oklch`, `Xyz50`, `Xyz65`. Each space is a plain struct re-exported from `culors::spaces`.
- `ColorSpace` trait with `to_xyz65` / `from_xyz65` plus alpha access. Every space implements it.
- `Color` enum: a tagged union over every space, with `From<Space>` impls for ergonomic construction.
- Generic `convert<A, B>(c: A) -> B` function that routes through XYZ D65 for any pair of `ColorSpace` implementors.
- Direct `From` impls for the precision-critical pairs: `Rgb` ↔ `LinearRgb`, `Rgb` ↔ `Hsl`, `Rgb` ↔ `Hsv`, `Hsv` ↔ `Hwb`, `LinearRgb` ↔ `Oklab`, `Oklab` ↔ `Oklch`, `Xyz50` ↔ `Lab`, `Lab` ↔ `Lch`, and `Rgb` → `Lab` / `Lch` / `Oklab` / `Oklch` with culori's achromatic snap.
- CSS Color Module 4 parser (`parse(&str) -> Option<Color>`) covering named colors, `transparent`, hex (`#rgb`, `#rgba`, `#rrggbb`, `#rrggbbaa`), functional `rgb()`, `rgba()`, `hsl()`, `hsla()`, `hwb()`, `lab()`, `lch()`, `oklab()`, `oklch()`, and `color()` with `srgb`, `srgb-linear`, `xyz`, `xyz-d50`, `xyz-d65` profiles.
- CSS Color Module 4 formatter (`format_css(&Color) -> String`) emitting modern functional notation with slash-prefixed alpha and `none` for NaN channels.
- Fixture-based test infrastructure: 110 conversion pairs, 365 parse cases, 303 format round-trips, all generated from culori 4.0.2 and re-verified on every CI run.
- Optional `serde` feature deriving `Serialize` and `Deserialize` for every space struct and `Color`.

### Known limitations

- `convert<A, B>` always routes through XYZ D65. For byte-for-byte parity with culori's per-pair routing, use the direct `Type::from(value)` impls listed above for compile-time-known pairs. Pairs without a direct impl drift from culori by approximately 1e-14, acceptable for color use cases but not bit-exact.
- `color()` function with `display-p3`, `rec2020`, `prophoto-rgb`, or `a98-rgb` profiles returns `None`. Those spaces are deferred to v0.4.

[Unreleased]: https://github.com/koole/culors/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/koole/culors/compare/v0.1.0...v1.0.0
[0.1.0]: https://github.com/koole/culors/releases/tag/v0.1.0
