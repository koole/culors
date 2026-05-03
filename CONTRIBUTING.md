# Contributing to culors

## One-time setup

```bash
git clone https://github.com/koole/culors
cd culors
npm ci
git config core.hooksPath .githooks
```

The last line enables the pre-commit hook in `.githooks/pre-commit`,
which prevents the most common contributor mistake (described below).

## Working on the Rust crate

Standard Cargo workflow:

```bash
cargo build --all-features
cargo test --all-features
cargo clippy --all-features --all-targets -- -D warnings
cargo fmt
```

CI runs all of these on every push and PR.

## Working on the fixture generators

The fixtures under `tests/fixtures/` are committed to the repo so that
downstream consumers can run `cargo test --package culors` without
needing Node.js installed. They are produced by Node scripts under
`fixtures-gen/` that drive culori 4.0.2 directly.

If you modify any of:

- `fixtures-gen/inputs.mjs`
- `fixtures-gen/parse-inputs.mjs`
- `fixtures-gen/generate.mjs`
- `fixtures-gen/parse-fixtures.mjs`
- `fixtures-gen/format-fixtures.mjs`
- `package.json` / `package-lock.json` (which pin culori)

then **you must regenerate the fixtures and stage the result in the
same commit:**

```bash
npm run gen-fixtures
npm run gen-parse-fixtures
npm run gen-format-fixtures
git add tests/fixtures/
```

The pre-commit hook enforces this. If you commit a generator change
without regenerating, the hook re-runs the generators in a temporary
copy of the staged tree and refuses the commit if the output drifts
from the staged fixtures.

CI runs the same check, so a missed regen would be caught at PR time
even without the local hook — but the hook gives you faster feedback.

## Why fixtures aren't generated at test time

It would eliminate the staleness class of bug, but at the cost of
requiring every consumer of the crate to install Node.js to run their
own test suite (since fixture tests would invoke the generator from
`build.rs`). That's a hard sell for a published library.

The pre-commit hook + CI check is the practical compromise: contributor
machines and CI runners need Node, downstream consumers do not.

## Cross-platform fixture drift

Linux, macOS, and Windows libm functions (`atan2`, `sqrt`, `cbrt`,
`pow`) can differ by up to 1 ULP, producing `Number.toString` outputs
that diverge in the last digit. The drift check at
`fixtures-gen/check-drift.mjs` walks both fixture trees and compares
each numeric value within `1e-13` absolute or relative tolerance, so
1-ULP cross-platform noise passes while genuine algorithmic drift
fails. CI uses this check; the pre-commit hook uses it too.

If you regenerate fixtures on macOS arm64 and CI's Linux x86_64 run
produces "drift within tolerance — passing", that's expected. The
committed fixtures live in whichever shape the most recent regen
produced; the test suite tolerates either.

## Lints

`cargo clippy --all-features --all-targets -- -D warnings` must be
clean. The `--all-targets` flag is required (without it, dead-code
warnings in integration test files are missed). The CI workflow
enforces both `--all-features` and `--no-default-features`.

## License

All contributions are licensed under MIT (see `LICENSE`). By opening
a pull request you agree your contribution is licensed under those
terms.
