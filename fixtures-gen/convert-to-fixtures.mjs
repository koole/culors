// Generator for the `convert_to` byte-for-byte parity fixtures.
//
// For every ordered (from, to) pair across the culori-known spaces culors
// implements, this script emits the culori output of `converter(to)(input)`
// for a deterministic 6-row sample drawn from each source space. Output lands
// in `tests/fixtures/convert_to/<from>_to_<to>.json`.
//
// Spaces excluded from this run: hsluv, hpluv, prismatic. They are culors
// extensions; culori knows nothing about them, so there is no truth to
// compare against. Each has its own ported_*.rs suite already.
//
// Idempotent: input matrix is deterministic, key order is fixed.

import { converter } from "culori";
import { existsSync, mkdirSync, readdirSync, unlinkSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const OUT_DIR = join(__dirname, "..", "tests", "fixtures", "convert_to");

// All culori modes we mirror. Channel ranges chosen to land in-gamut for the
// space's natural domain. Keys here drive both input generation and the
// deserializer field set on the Rust side.
const SPACES = {
	rgb: { ch: ["r", "g", "b"], r: [0, 1], g: [0, 1], b: [0, 1] },
	lrgb: { ch: ["r", "g", "b"], r: [0, 1], g: [0, 1], b: [0, 1] },
	hsl: { ch: ["h", "s", "l"], h: [0, 360], s: [0, 1], l: [0, 1] },
	hsv: { ch: ["h", "s", "v"], h: [0, 360], s: [0, 1], v: [0, 1] },
	hwb: { ch: ["h", "w", "b"], h: [0, 360], w: [0, 1], b: [0, 1] },
	hsi: { ch: ["h", "s", "i"], h: [0, 360], s: [0, 1], i: [0, 1] },
	lab: { ch: ["l", "a", "b"], l: [0, 100], a: [-100, 100], b: [-100, 100] },
	lab65: { ch: ["l", "a", "b"], l: [0, 100], a: [-100, 100], b: [-100, 100] },
	lch: { ch: ["l", "c", "h"], l: [0, 100], c: [0, 130], h: [0, 360] },
	lch65: { ch: ["l", "c", "h"], l: [0, 100], c: [0, 130], h: [0, 360] },
	oklab: { ch: ["l", "a", "b"], l: [0, 1], a: [-0.4, 0.4], b: [-0.4, 0.4] },
	oklch: { ch: ["l", "c", "h"], l: [0, 1], c: [0, 0.4], h: [0, 360] },
	okhsl: { ch: ["h", "s", "l"], h: [0, 360], s: [0, 1], l: [0, 1] },
	okhsv: { ch: ["h", "s", "v"], h: [0, 360], s: [0, 1], v: [0, 1] },
	xyz50: { ch: ["x", "y", "z"], x: [0, 1], y: [0, 1], z: [0, 1] },
	xyz65: { ch: ["x", "y", "z"], x: [0, 1], y: [0, 1], z: [0, 1] },
	p3: { ch: ["r", "g", "b"], r: [0, 1], g: [0, 1], b: [0, 1] },
	rec2020: { ch: ["r", "g", "b"], r: [0, 1], g: [0, 1], b: [0, 1] },
	a98: { ch: ["r", "g", "b"], r: [0, 1], g: [0, 1], b: [0, 1] },
	prophoto: { ch: ["r", "g", "b"], r: [0, 1], g: [0, 1], b: [0, 1] },
	dlab: { ch: ["l", "a", "b"], l: [0, 100], a: [-50, 50], b: [-50, 50] },
	dlch: { ch: ["l", "c", "h"], l: [0, 100], c: [0, 60], h: [0, 360] },
	jab: { ch: ["j", "a", "b"], j: [0, 0.2], a: [-0.1, 0.1], b: [-0.1, 0.1] },
	jch: { ch: ["j", "c", "h"], j: [0, 0.2], c: [0, 0.1], h: [0, 360] },
	itp: { ch: ["i", "t", "p"], i: [0, 0.5], t: [-0.5, 0.5], p: [-0.5, 0.5] },
	xyb: { ch: ["x", "y", "b"], x: [-0.015, 0.028], y: [0, 0.85], b: [0, 0.85] },
	yiq: { ch: ["y", "i", "q"], y: [0, 1], i: [-0.6, 0.6], q: [-0.5, 0.5] },
	cubehelix: { ch: ["h", "s", "l"], h: [0, 360], s: [0, 4.6], l: [0, 1] },
	luv: { ch: ["l", "u", "v"], l: [0, 100], u: [-100, 100], v: [-100, 100] },
	lchuv: { ch: ["l", "c", "h"], l: [0, 100], c: [0, 100], h: [0, 360] },
};

const SPACE_NAMES = Object.keys(SPACES);

// Park-Miller LCG seeded from a string so re-runs are stable.
function makeLcg(seedStr) {
	let h = 0;
	for (let i = 0; i < seedStr.length; i++) {
		h = (h * 31 + seedStr.charCodeAt(i)) | 0;
	}
	let state = (h & 0x7fffffff) || 1;
	return () => {
		state = (state * 48271) % 0x7fffffff;
		return (state - 1) / (0x7fffffff - 1);
	};
}

function buildInputs(space) {
	const meta = SPACES[space];
	const lcg = makeLcg(`culors-convert-to-${space}`);
	const lerp = (a, b, t) => a + (b - a) * t;
	const rows = [];
	// One achromatic, one mid, four randoms.
	const mid = {};
	const ach = {};
	for (const ch of meta.ch) {
		const [lo, hi] = meta[ch];
		mid[ch] = lerp(lo, hi, 0.5);
		ach[ch] = ch === "h" ? 0 : lerp(lo, hi, 0.5);
	}
	// Achromatic-ish for spaces with hue: chroma 0 / saturation 0 / a=b=0.
	if (meta.ch.includes("c")) ach.c = 0;
	if (meta.ch.includes("s")) ach.s = 0;
	if (meta.ch.includes("a") && meta.ch.includes("b")) {
		ach.a = 0;
		ach.b = 0;
	}
	rows.push({ mode: space, ...ach });
	rows.push({ mode: space, ...mid });
	for (let i = 0; i < 4; i++) {
		const row = { mode: space };
		for (const ch of meta.ch) {
			const [lo, hi] = meta[ch];
			row[ch] = lerp(lo, hi, lcg());
		}
		rows.push(row);
	}
	return rows;
}

function projectRow(obj, channels) {
	const out = {};
	for (const ch of channels) {
		if (obj[ch] !== undefined) out[ch] = obj[ch];
	}
	if (obj.alpha !== undefined) out.alpha = obj.alpha;
	return out;
}

function clearOld() {
	if (!existsSync(OUT_DIR)) {
		mkdirSync(OUT_DIR, { recursive: true });
		return;
	}
	for (const name of readdirSync(OUT_DIR)) {
		if (name.endsWith(".json")) unlinkSync(join(OUT_DIR, name));
	}
}

clearOld();

const CONVERTERS = Object.fromEntries(SPACE_NAMES.map((s) => [s, converter(s)]));

let pairCount = 0;
let rowCount = 0;
for (const from of SPACE_NAMES) {
	const inputs = buildInputs(from);
	for (const to of SPACE_NAMES) {
		if (from === to) continue;
		const conv = CONVERTERS[to];
		const targetCh = SPACES[to].ch;
		const rows = inputs.map((input) => {
			const output = conv(input);
			return {
				input: projectRow(input, SPACES[from].ch),
				output: projectRow(output, targetCh),
			};
		});
		const fixture = { from, to, rows };
		const path = join(OUT_DIR, `${from}_to_${to}.json`);
		writeFileSync(path, JSON.stringify(fixture, null, 2) + "\n");
		pairCount++;
		rowCount += rows.length;
	}
}

console.log(`Wrote ${pairCount} fixture files (${rowCount} rows) to ${OUT_DIR}`);
