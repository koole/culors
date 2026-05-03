# Fixture generator

Emits JSON fixtures under `tests/fixtures/` for `culors`'s integration test
suite. Reference values are produced by `culori` 4.0.2 (npm). The Rust
runner reads these files directly; running it does not require Node.

## Regenerate

```bash
npm install            # once
npm run gen-fixtures   # writes 110 JSON files into tests/fixtures/
```

The generator wipes any existing `tests/fixtures/convert_*.json` before
writing, so stale fixtures cannot survive a regeneration.

## When to regenerate

- After a `culori` version bump in `package.json`.
- After changing the input matrix (`fixtures-gen/inputs.mjs`).
- After adding a new color space (the cross-product grows).

After regenerating, run `cargo test --test fixtures` to confirm nothing
drifted, and commit `tests/fixtures/` together with the change that
caused the regeneration.

## Determinism

Inputs are deterministic:

- Boundary rows are hardcoded per space (black, white, primaries,
  achromatic, alpha cases, plus space-specific edges such as `w + b > 1`
  for HWB).
- 50 random rows per space are drawn from a Park-Miller LCG
  (multiplier `48271`, modulus `2^31 - 1`). The seed is derived from the
  string `"culors-v0.1"`, decorrelated per space by suffixing the mode
  name (e.g., `culors-v0.1::rgb`).
- 20% of random rows include an `alpha` channel.

JSON output uses `JSON.stringify` with tab indentation and the canonical
channel order from `inputs.mjs`. Re-running the generator produces a
byte-identical diff against the previous run; CI relies on this.

## File layout

For every ordered pair `(from, to)` with `from != to` across the 11 v0.1
spaces (`rgb, lrgb, hsl, hsv, hwb, lab, lch, oklab, oklch, xyz50, xyz65`),
one file is emitted:

```
tests/fixtures/convert_<from>_to_<to>.json
```

That's `11 * 10 = 110` files. Each file has the shape:

```json
{
  "from": "rgb",
  "to": "lab",
  "rows": [
    { "input": { "r": 1, "g": 0, "b": 0 }, "output": { "l": 54.29, "a": 80.80, "b": 69.89 } }
  ]
}
```

Hue channels for achromatic colors are omitted from `input` / `output`
to match culori's output shape; the Rust runner deserializes a missing
hue as `NaN`. Alpha is preserved when the input row carries one.
