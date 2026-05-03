// Format-round-trip fixture generator: emits a JSON file pairing every
// CSS input string from the parse-fixture set with culori 4.0.2's
// `formatCss(parse(input))` output. The Rust test in
// `tests/format_fixtures.rs` asserts that
// `culors::format_css(culors::parse(input).unwrap())` produces the same
// string as culori for every supported input.
//
// Idempotent: input list is deterministic, JSON is written with stable
// key ordering. Re-running produces zero diff.
//
// Inputs whose culori-parsed mode is not in `SUPPORTED_MODES` (e.g.
// `p3`, `rec2020`) are skipped, since culors cannot represent them and
// they're verified separately in the Rust unit tests.

import { formatCss, parse } from "culori";
import { writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

import { ALL_INPUTS, SUPPORTED_MODES } from "./parse-inputs.mjs";

const __dirname = dirname(fileURLToPath(import.meta.url));
const OUT_PATH = join(
	__dirname,
	"..",
	"tests",
	"fixtures",
	"format_round_trip.json",
);

function buildRow(input) {
	const parsed = parse(input);
	if (parsed === undefined) return null;
	if (!SUPPORTED_MODES.has(parsed.mode)) return null;
	const formatted = formatCss(parsed);
	if (formatted === undefined) return null;
	return { input, formatted };
}

function main() {
	const seen = new Set();
	const rows = [];
	for (const input of ALL_INPUTS) {
		if (seen.has(input)) continue;
		seen.add(input);
		const row = buildRow(input);
		if (row !== null) rows.push(row);
	}

	const fixture = { rows };
	const json = JSON.stringify(fixture, null, "\t");
	writeFileSync(OUT_PATH, json + "\n");

	process.stdout.write(
		`generated ${rows.length} format-round-trip rows in ${OUT_PATH}\n`,
	);
}

main();
