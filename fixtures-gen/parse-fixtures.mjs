// Parse-fixture generator: emits one JSON file with a list of CSS input
// strings paired with culori's parsed output for each. The Rust test in
// `tests/parse_fixtures.rs` asserts that `culor::parse(input)` matches
// culori's output channel-by-channel.
//
// Idempotent: the input list is deterministic (`./parse-inputs.mjs`) and
// we stringify with stable key ordering. Re-running produces zero diff.
//
// Outputs come from culori 4.0.2's public `parse()` function. For
// unsupported `color()` profiles culori returns a parsed value tagged
// with a mode culor does not implement (`p3`, `rec2020`, `prophoto`,
// `a98`); we exclude those inputs from the fixture and exercise them
// only in the Rust unit tests, where culor returns `None`.

import { parse } from "culori";
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
	"parse_css.json",
);

function buildRow(input) {
	const out = parse(input);
	if (out === undefined) {
		return { input, output: null };
	}
	if (!SUPPORTED_MODES.has(out.mode)) {
		return null; // skip; verified separately in Rust unit tests
	}
	return { input, output: out };
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

	const valid = rows.filter((r) => r.output !== null).length;
	const invalid = rows.length - valid;
	process.stdout.write(
		`generated ${rows.length} parse rows in ${OUT_PATH}\n` +
			`  ${valid} valid, ${invalid} reject (None)\n`,
	);
}

main();
