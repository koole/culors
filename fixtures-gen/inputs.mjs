// Deterministic input matrix for fixture generation.
//
// Each space exports an array of culori-shaped color objects (with `mode`)
// covering boundary values, primary colors, and 50 random samples drawn from
// a Park-Miller LCG (Lehmer RNG) seeded from the string "culor-v0.1".
//
// LCG: x_{n+1} = (a * x_n) mod m, with a = 48271, m = 2^31 - 1 (a "minimal
// standard" Park-Miller). This is reproducible across Node versions and has
// adequate quality for spreading inputs across each space's domain.

const SEED_STRING = "culor-v0.1";
const LCG_A = 48271;
const LCG_M = 0x7fffffff; // 2^31 - 1

function seedFromString(s) {
	let h = 0;
	for (let i = 0; i < s.length; i++) {
		h = (h * 31 + s.charCodeAt(i)) | 0;
	}
	let seed = h & LCG_M;
	if (seed === 0) seed = 1;
	return seed;
}

function makeLcg(seedStr) {
	let state = seedFromString(seedStr);
	return function next() {
		state = (state * LCG_A) % LCG_M;
		return (state - 1) / (LCG_M - 1); // 0..1 inclusive of 0, exclusive of 1-ish
	};
}

const RANDOM_COUNT = 50;
const ALPHA_RATIO = 0.2; // 20% of random rows include alpha

// Channel ranges per space. Hue channels span 0..360 (we'll add achromatic
// edge cases by hand). Other channels are space-appropriate.
const SPACE_RANGES = {
	rgb: { r: [0, 1], g: [0, 1], b: [0, 1] },
	lrgb: { r: [0, 1], g: [0, 1], b: [0, 1] },
	hsl: { h: [0, 360], s: [0, 1], l: [0, 1] },
	hsv: { h: [0, 360], s: [0, 1], v: [0, 1] },
	hwb: { h: [0, 360], w: [0, 1], b: [0, 1] },
	lab: { l: [0, 100], a: [-100, 100], b: [-100, 100] },
	lch: { l: [0, 100], c: [0, 130], h: [0, 360] },
	oklab: { l: [0, 1], a: [-0.4, 0.4], b: [-0.4, 0.4] },
	oklch: { l: [0, 1], c: [0, 0.4], h: [0, 360] },
	xyz50: { x: [0, 1], y: [0, 1], z: [0, 1] },
	xyz65: { x: [0, 1], y: [0, 1], z: [0, 1] },
};

function lerp(rng, [lo, hi]) {
	return lo + rng() * (hi - lo);
}

function randomRow(mode, rng) {
	const ranges = SPACE_RANGES[mode];
	const row = { mode };
	for (const [ch, range] of Object.entries(ranges)) {
		row[ch] = lerp(rng, range);
	}
	if (rng() < ALPHA_RATIO) {
		row.alpha = lerp(rng, [0, 1]);
	}
	return row;
}

function randomRows(mode, rng, count = RANDOM_COUNT) {
	const out = [];
	for (let i = 0; i < count; i++) {
		out.push(randomRow(mode, rng));
	}
	return out;
}

// ---- Boundary / handpicked rows per space ------------------------------

const RGB_BOUNDARY = [
	{ mode: "rgb", r: 0, g: 0, b: 0 },
	{ mode: "rgb", r: 1, g: 1, b: 1 },
	{ mode: "rgb", r: 1, g: 0, b: 0 },
	{ mode: "rgb", r: 0, g: 1, b: 0 },
	{ mode: "rgb", r: 0, g: 0, b: 1 },
	{ mode: "rgb", r: 0.5, g: 0.5, b: 0.5 },
	{ mode: "rgb", r: 1, g: 0.5, b: 0.25 },
	{ mode: "rgb", r: 0.25, g: 0.6, b: 0.9 },
	{ mode: "rgb", r: 0, g: 0, b: 0, alpha: 0 },
	{ mode: "rgb", r: 1, g: 1, b: 1, alpha: 1 },
	{ mode: "rgb", r: 0.7, g: 0.3, b: 0.1, alpha: 0.5 },
];

const LRGB_BOUNDARY = [
	{ mode: "lrgb", r: 0, g: 0, b: 0 },
	{ mode: "lrgb", r: 1, g: 1, b: 1 },
	{ mode: "lrgb", r: 1, g: 0, b: 0 },
	{ mode: "lrgb", r: 0, g: 1, b: 0 },
	{ mode: "lrgb", r: 0, g: 0, b: 1 },
	{ mode: "lrgb", r: 0.5, g: 0.5, b: 0.5 },
	{ mode: "lrgb", r: 0.18, g: 0.5, b: 0.82 },
	{ mode: "lrgb", r: 0.5, g: 0.5, b: 0.5, alpha: 0.4 },
];

// HSL: include achromatic (h omitted) to mirror culori's output for s=0.
const HSL_BOUNDARY = [
	{ mode: "hsl", h: 0, s: 0, l: 0 },
	{ mode: "hsl", h: 0, s: 0, l: 1 },
	{ mode: "hsl", h: 0, s: 1, l: 0.5 },
	{ mode: "hsl", h: 120, s: 1, l: 0.5 },
	{ mode: "hsl", h: 240, s: 1, l: 0.5 },
	{ mode: "hsl", h: 60, s: 0.5, l: 0.5 },
	{ mode: "hsl", h: 180, s: 0.25, l: 0.75 },
	{ mode: "hsl", h: 300, s: 0.8, l: 0.4 },
	{ mode: "hsl", s: 0, l: 0.5 }, // achromatic, h omitted
	{ mode: "hsl", s: 0, l: 0.25, alpha: 0.5 },
	{ mode: "hsl", h: 30, s: 0.6, l: 0.4, alpha: 0.7 },
];

const HSV_BOUNDARY = [
	{ mode: "hsv", h: 0, s: 0, v: 0 },
	{ mode: "hsv", h: 0, s: 0, v: 1 },
	{ mode: "hsv", h: 0, s: 1, v: 1 },
	{ mode: "hsv", h: 120, s: 1, v: 1 },
	{ mode: "hsv", h: 240, s: 1, v: 1 },
	{ mode: "hsv", h: 60, s: 0.5, v: 0.5 },
	{ mode: "hsv", h: 200, s: 0.4, v: 0.8 },
	{ mode: "hsv", s: 0, v: 0.5 }, // achromatic
	{ mode: "hsv", h: 90, s: 0.7, v: 0.6, alpha: 0.3 },
];

const HWB_BOUNDARY = [
	{ mode: "hwb", h: 0, w: 0, b: 0 },
	{ mode: "hwb", h: 0, w: 1, b: 0 },
	{ mode: "hwb", h: 0, w: 0, b: 1 },
	{ mode: "hwb", h: 120, w: 0, b: 0 },
	{ mode: "hwb", h: 240, w: 0.2, b: 0.2 },
	{ mode: "hwb", h: 60, w: 0.7, b: 0.7 }, // w+b>1 normalization
	{ mode: "hwb", h: 200, w: 0.6, b: 0.5 }, // w+b>1
	{ mode: "hwb", w: 0.3, b: 0.3 }, // achromatic
	{ mode: "hwb", h: 30, w: 0.1, b: 0.2, alpha: 0.5 },
];

const LAB_BOUNDARY = [
	{ mode: "lab", l: 0, a: 0, b: 0 },
	{ mode: "lab", l: 100, a: 0, b: 0 },
	{ mode: "lab", l: 50, a: 0, b: 0 },
	{ mode: "lab", l: 54.29054294696968, a: 80.80492033462417, b: 69.89098825896278 },
	{ mode: "lab", l: 87.81853633115202, a: -79.27108223854806, b: 80.99459785152247 },
	{ mode: "lab", l: 29.568297153444703, a: 68.2874066521555, b: -112.02971798617645 },
	{ mode: "lab", l: 50, a: 25, b: -25, alpha: 0.6 },
];

const LCH_BOUNDARY = [
	{ mode: "lch", l: 0, c: 0, h: 0 },
	{ mode: "lch", l: 100, c: 0, h: 0 },
	{ mode: "lch", l: 50, c: 0, h: 0 }, // achromatic, but h: 0 still given
	{ mode: "lch", l: 50, c: 0 }, // achromatic, h omitted
	{ mode: "lch", l: 54.29, c: 106.84, h: 40.86 },
	{ mode: "lch", l: 87.82, c: 113.34, h: 134.39 },
	{ mode: "lch", l: 29.57, c: 131.21, h: 301.36 },
	{ mode: "lch", l: 60, c: 50, h: 270, alpha: 0.5 },
];

const OKLAB_BOUNDARY = [
	{ mode: "oklab", l: 0, a: 0, b: 0 },
	{ mode: "oklab", l: 1, a: 0, b: 0 },
	{ mode: "oklab", l: 0.5, a: 0, b: 0 },
	{ mode: "oklab", l: 0.6279553639214311, a: 0.22486306106597398, b: 0.1258462985307351 },
	{ mode: "oklab", l: 0.5, a: -0.1, b: 0.1, alpha: 0.5 },
];

const OKLCH_BOUNDARY = [
	{ mode: "oklch", l: 0, c: 0, h: 0 },
	{ mode: "oklch", l: 1, c: 0, h: 0 },
	{ mode: "oklch", l: 0.5, c: 0 }, // achromatic, h omitted
	{ mode: "oklch", l: 0.628, c: 0.258, h: 29.23 },
	{ mode: "oklch", l: 0.866, c: 0.295, h: 142.5 },
	{ mode: "oklch", l: 0.452, c: 0.313, h: 264.05 },
	{ mode: "oklch", l: 0.6, c: 0.15, h: 200, alpha: 0.4 },
];

const XYZ50_BOUNDARY = [
	{ mode: "xyz50", x: 0, y: 0, z: 0 },
	{ mode: "xyz50", x: 0.9642956764295677, y: 1, z: 0.8251046025104602 }, // D50 white
	{ mode: "xyz50", x: 0.5, y: 0.5, z: 0.5 },
	{ mode: "xyz50", x: 0.18, y: 0.2, z: 0.15, alpha: 0.5 },
];

const XYZ65_BOUNDARY = [
	{ mode: "xyz65", x: 0, y: 0, z: 0 },
	{ mode: "xyz65", x: 0.95047, y: 1, z: 1.08883 }, // D65 white (approx)
	{ mode: "xyz65", x: 0.5, y: 0.5, z: 0.5 },
	{ mode: "xyz65", x: 0.4124, y: 0.2126, z: 0.0193 }, // approx red
	{ mode: "xyz65", x: 0.3, y: 0.4, z: 0.6, alpha: 0.7 },
];

// Build per-space arrays. Each space gets its own LCG branch derived from the
// same seed string, suffixed with the mode name to decorrelate the streams.
function buildInputs(mode, boundary) {
	const rng = makeLcg(`${SEED_STRING}::${mode}`);
	return [...boundary, ...randomRows(mode, rng)];
}

export const RGB_INPUTS = buildInputs("rgb", RGB_BOUNDARY);
export const LRGB_INPUTS = buildInputs("lrgb", LRGB_BOUNDARY);
export const HSL_INPUTS = buildInputs("hsl", HSL_BOUNDARY);
export const HSV_INPUTS = buildInputs("hsv", HSV_BOUNDARY);
export const HWB_INPUTS = buildInputs("hwb", HWB_BOUNDARY);
export const LAB_INPUTS = buildInputs("lab", LAB_BOUNDARY);
export const LCH_INPUTS = buildInputs("lch", LCH_BOUNDARY);
export const OKLAB_INPUTS = buildInputs("oklab", OKLAB_BOUNDARY);
export const OKLCH_INPUTS = buildInputs("oklch", OKLCH_BOUNDARY);
export const XYZ50_INPUTS = buildInputs("xyz50", XYZ50_BOUNDARY);
export const XYZ65_INPUTS = buildInputs("xyz65", XYZ65_BOUNDARY);

export const ALL_INPUTS = {
	rgb: RGB_INPUTS,
	lrgb: LRGB_INPUTS,
	hsl: HSL_INPUTS,
	hsv: HSV_INPUTS,
	hwb: HWB_INPUTS,
	lab: LAB_INPUTS,
	lch: LCH_INPUTS,
	oklab: OKLAB_INPUTS,
	oklch: OKLCH_INPUTS,
	xyz50: XYZ50_INPUTS,
	xyz65: XYZ65_INPUTS,
};

export const SPACE_CHANNELS = {
	rgb: ["r", "g", "b"],
	lrgb: ["r", "g", "b"],
	hsl: ["h", "s", "l"],
	hsv: ["h", "s", "v"],
	hwb: ["h", "w", "b"],
	lab: ["l", "a", "b"],
	lch: ["l", "c", "h"],
	oklab: ["l", "a", "b"],
	oklch: ["l", "c", "h"],
	xyz50: ["x", "y", "z"],
	xyz65: ["x", "y", "z"],
};

export const SPACES = Object.keys(ALL_INPUTS);
