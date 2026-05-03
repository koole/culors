// Fixture generator: emits one JSON file per ordered (from, to) pair across
// the 11 culor color spaces. Outputs land in tests/fixtures/.
//
// Idempotent: the input matrix is deterministic (see inputs.mjs) and we
// stringify with stable key ordering. Running twice produces zero diff.

import { converter } from "culori";
import { mkdirSync, writeFileSync, readdirSync, unlinkSync, existsSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

import { ALL_INPUTS, SPACE_CHANNELS, SPACES } from "./inputs.mjs";

const __dirname = dirname(fileURLToPath(import.meta.url));
const OUT_DIR = join(__dirname, "..", "tests", "fixtures");

// Number formatter: round-trip safe doubles via the shortest decimal that
// reproduces the same value. JS `JSON.stringify` already does this, so we
// just rely on it. We do NOT special-case finite numbers further; the goal
// is a stable, lossless representation.

// Build a row's "input"/"output" object containing only the space's channels
// plus optional alpha. We exclude `mode` so the JSON stays small and the
// Rust deserializer can use plain field structs.
function projectRow(obj, channels) {
	const out = {};
	for (const ch of channels) {
		const v = obj[ch];
		// culori may omit a hue channel for achromatic colors. Treat absence
		// as "not present" — Rust deserializes that to NaN.
		if (v === undefined) continue;
		out[ch] = v;
	}
	if (obj.alpha !== undefined) {
		out.alpha = obj.alpha;
	}
	return out;
}

// Stable key ordering: we use the canonical channel order for each space
// (e.g., r,g,b,alpha) so the output JSON is byte-stable.
function stableStringify(value, indent) {
	return JSON.stringify(value, null, indent);
}

function fixturePath(from, to) {
	return join(OUT_DIR, `convert_${from}_to_${to}.json`);
}

function clearOldFixtures() {
	if (!existsSync(OUT_DIR)) {
		mkdirSync(OUT_DIR, { recursive: true });
		return;
	}
	for (const name of readdirSync(OUT_DIR)) {
		if (name.startsWith("convert_") && name.endsWith(".json")) {
			unlinkSync(join(OUT_DIR, name));
		}
	}
}

function generatePair(from, to) {
	const inputs = ALL_INPUTS[from];
	const fromChannels = SPACE_CHANNELS[from];
	const toChannels = SPACE_CHANNELS[to];
	const conv = converter(to);

	const rows = inputs.map((row) => {
		const output = conv(row);
		return {
			input: projectRow(row, fromChannels),
			output: projectRow(output, toChannels),
		};
	});

	const fixture = { from, to, rows };
	const json = stableStringify(fixture, "\t");
	writeFileSync(fixturePath(from, to), json + "\n");
}

function main() {
	clearOldFixtures();
	let count = 0;
	for (const from of SPACES) {
		for (const to of SPACES) {
			if (from === to) continue;
			generatePair(from, to);
			count++;
		}
	}
	const totalRows = SPACES.reduce(
		(acc, s) => acc + ALL_INPUTS[s].length,
		0,
	);
	process.stdout.write(
		`generated ${count} fixture files in ${OUT_DIR}\n` +
			`rows per source space: ${SPACES.map(
				(s) => `${s}=${ALL_INPUTS[s].length}`,
			).join(", ")}\n` +
			`total rows across files: ${count * 0} + ${totalRows} per source = ${
				totalRows * 10
			} (10 targets per source)\n`,
	);
}

main();
