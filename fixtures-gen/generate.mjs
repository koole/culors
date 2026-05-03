// Fixture generator: emits one JSON file per ordered (from, to) pair across
// the 11 culor color spaces. Outputs land in tests/fixtures/.
//
// Idempotent: the input matrix is deterministic (see inputs.mjs) and we
// stringify with stable key ordering. Running twice produces zero diff.

import {
	convertRgbToLrgb,
	convertLrgbToRgb,
	convertRgbToHsl,
	convertHslToRgb,
	convertRgbToHsv,
	convertHsvToRgb,
	convertRgbToHwb,
	convertHwbToRgb,
	convertXyz65ToXyz50,
	convertXyz50ToXyz65,
	convertXyz50ToLab,
	convertLabToXyz50,
	convertLabToLch,
	convertLchToLab,
	convertLrgbToOklab,
	convertOklabToLrgb,
} from "culori";
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

// culor's generic `convert<A, B>()` routes every conversion through XYZ65
// using each space's own `to_xyz65` / `from_xyz65`. culori's high-level
// `converter(mode)` takes shortest paths and applies fixups (e.g.
// `convertRgbToLab` snaps achromatic inputs to a=b=0). To match the Rust
// pipeline bit-for-bit on numerically sensitive inputs, the generator
// composes the same primitive culori conversions Rust uses, manually.

// Direct linear-sRGB <-> XYZ65 matrix multiplication. Rust's
// `LinearRgb::to_xyz65` calls `util::lrgb_to_xyz65` directly with the same
// constants culori uses inside `convertRgbToXyz65`. To keep the fixture
// numerically identical, we inline that matrix here rather than detouring
// through gamma-encoded sRGB (`convertLrgbToRgb` then `convertRgbToXyz65`).
function lrgbToXyz65({ r, g, b, alpha }) {
	const out = {
		mode: "xyz65",
		x: 0.4123907992659593 * r + 0.357584339383878 * g + 0.1804807884018343 * b,
		y: 0.2126390058715102 * r + 0.715168678767756 * g + 0.0721923153607337 * b,
		z: 0.0193308187155918 * r + 0.119194779794626 * g + 0.9505321522496607 * b,
	};
	if (alpha !== undefined) out.alpha = alpha;
	return out;
}

function xyz65ToLrgb({ x, y, z, alpha }) {
	const out = {
		mode: "lrgb",
		r: x * 3.2409699419045226 - y * 1.5373831775700939 - 0.4986107602930034 * z,
		g: x * -0.9692436362808796 + y * 1.8759675015077204 + 0.0415550574071756 * z,
		b: x * 0.0556300796969936 - y * 0.2039769588889765 + 1.0569715142428784 * z,
	};
	if (alpha !== undefined) out.alpha = alpha;
	return out;
}

// Each `toHub(input)` mirrors `<Space>::to_xyz65(self)` in Rust. We pin the
// exact intermediate sequence Rust uses, including which space directly
// touches the LRGB <-> XYZ65 matrix, so floating-point output agrees to the
// last bit.
const toHub = {
	rgb: (c) => lrgbToXyz65(convertRgbToLrgb(c)),
	lrgb: (c) => lrgbToXyz65(c),
	hsl: (c) => lrgbToXyz65(convertRgbToLrgb(convertHslToRgb(c))),
	hsv: (c) => lrgbToXyz65(convertRgbToLrgb(convertHsvToRgb(c))),
	hwb: (c) => lrgbToXyz65(convertRgbToLrgb(convertHwbToRgb(c))),
	lab: (c) => convertXyz50ToXyz65(convertLabToXyz50(c)),
	lch: (c) => convertXyz50ToXyz65(convertLabToXyz50(convertLchToLab(c))),
	oklab: (c) => lrgbToXyz65(convertOklabToLrgb(c)),
	oklch: (c) =>
		lrgbToXyz65(convertOklabToLrgb(convertLchToLab(c, "oklab"))),
	xyz50: (c) => convertXyz50ToXyz65(c),
	xyz65: (c) => ({ ...c }),
};

// Each `fromHub(xyz65)` mirrors `<Space>::from_xyz65(xyz)` in Rust.
const fromHub = {
	rgb: (c) => convertLrgbToRgb(xyz65ToLrgb(c)),
	lrgb: (c) => xyz65ToLrgb(c),
	hsl: (c) => convertRgbToHsl(convertLrgbToRgb(xyz65ToLrgb(c))),
	hsv: (c) => convertRgbToHsv(convertLrgbToRgb(xyz65ToLrgb(c))),
	hwb: (c) => convertRgbToHwb(convertLrgbToRgb(xyz65ToLrgb(c))),
	lab: (c) => convertXyz50ToLab(convertXyz65ToXyz50(c)),
	lch: (c) => convertLabToLch(convertXyz50ToLab(convertXyz65ToXyz50(c))),
	oklab: (c) => convertLrgbToOklab(xyz65ToLrgb(c)),
	oklch: (c) =>
		convertLabToLch(convertLrgbToOklab(xyz65ToLrgb(c)), "oklch"),
	xyz50: (c) => convertXyz65ToXyz50(c),
	xyz65: (c) => ({ ...c }),
};

// culori's primitive conversion functions read raw channels off the input
// without consulting `mode`. They also return objects carrying their own
// `mode`. Alpha is preserved by every primitive.

function ensureMode(obj, mode) {
	return { mode, ...obj };
}

function generatePair(from, to) {
	const inputs = ALL_INPUTS[from];
	const fromChannels = SPACE_CHANNELS[from];
	const toChannels = SPACE_CHANNELS[to];

	const rows = inputs.map((row) => {
		const hub = toHub[from](row);
		const out = fromHub[to](hub);
		// Some primitives (e.g. convertXyz65ToXyz50) drop alpha; reattach
		// from the source row to mirror Rust's hub conversions, which carry
		// alpha through unconditionally.
		if (row.alpha !== undefined && out.alpha === undefined) {
			out.alpha = row.alpha;
		}
		return {
			input: projectRow(row, fromChannels),
			output: projectRow(ensureMode(out, to), toChannels),
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
