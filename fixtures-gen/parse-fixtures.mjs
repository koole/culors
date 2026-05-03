// Parse-fixture generator: emits one JSON file with a list of CSS input
// strings paired with culori's parsed output for each. The Rust test in
// `tests/parse_fixtures.rs` asserts that `culor::parse(input)` matches
// culori's output channel-by-channel.
//
// Idempotent: the input list is deterministic (defined inline below) and
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

const __dirname = dirname(fileURLToPath(import.meta.url));
const OUT_PATH = join(
	__dirname,
	"..",
	"tests",
	"fixtures",
	"parse_css.json",
);

// Subset of CSS named colors covering each spelling category (single
// word, hyphen-free compound, `dark*` / `light*` / `medium*` prefixes,
// the `*gray` / `*grey` aliases, and the misc ones like `rebeccapurple`
// and `transparent`). Picked to be representative of the 148 entries.
const NAMED_SAMPLE = [
	"aliceblue",
	"aqua",
	"aquamarine",
	"black",
	"blue",
	"blueviolet",
	"brown",
	"burlywood",
	"chartreuse",
	"cornflowerblue",
	"crimson",
	"cyan",
	"darkblue",
	"darkgray",
	"darkgrey",
	"darkkhaki",
	"darkolivegreen",
	"darkslategray",
	"darkslategrey",
	"deeppink",
	"dimgray",
	"dimgrey",
	"dodgerblue",
	"firebrick",
	"floralwhite",
	"forestgreen",
	"fuchsia",
	"gainsboro",
	"gold",
	"goldenrod",
	"gray",
	"green",
	"greenyellow",
	"grey",
	"hotpink",
	"indianred",
	"indigo",
	"khaki",
	"lavender",
	"lavenderblush",
	"lemonchiffon",
	"lightblue",
	"lightgoldenrodyellow",
	"lightgray",
	"lightgrey",
	"lightseagreen",
	"lightslategray",
	"lightslategrey",
	"lime",
	"limegreen",
	"magenta",
	"maroon",
	"mediumaquamarine",
	"mediumorchid",
	"mediumpurple",
	"mediumvioletred",
	"midnightblue",
	"navy",
	"olive",
	"olivedrab",
	"orange",
	"orangered",
	"orchid",
	"papayawhip",
	"peachpuff",
	"pink",
	"plum",
	"purple",
	"rebeccapurple",
	"red",
	"royalblue",
	"saddlebrown",
	"salmon",
	"seagreen",
	"silver",
	"skyblue",
	"slategray",
	"slategrey",
	"steelblue",
	"tan",
	"teal",
	"tomato",
	"turquoise",
	"violet",
	"white",
	"whitesmoke",
	"yellow",
	"yellowgreen",
];

const HEX_INPUTS = [
	"#000",
	"#fff",
	"#f00",
	"#0f0",
	"#00f",
	"#abc",
	"#ABC",
	"#aBc",
	"#abcd",
	"#ABCD",
	"#aBcD",
	"#000000",
	"#ffffff",
	"#ff0000",
	"#00ff00",
	"#0000ff",
	"#FF0000",
	"#fF00aA",
	"#deadbe",
	"#00000000",
	"#ffffffff",
	"#ff000080",
	"#FF000080",
	"#abcdef12",
	// Invalid hex shapes
	"#",
	"#a",
	"#ab",
	"#abcde",
	"#abcdefg",
	"#abcdefghi",
	"#xyz",
	"#xyzxyz",
	"#GG0000",
	"#-100000",
	"100000",
];

const RGB_INPUTS = [
	// Modern, integers
	"rgb(0 0 0)",
	"rgb(255 0 0)",
	"rgb(0 255 0)",
	"rgb(0 0 255)",
	"rgb(255 255 255)",
	"rgb(128 64 32)",
	// Modern, percentages
	"rgb(0% 0% 0%)",
	"rgb(100% 0% 0%)",
	"rgb(50% 50% 50%)",
	"rgb(50% 25% 0%)",
	// Modern with alpha
	"rgb(255 0 0 / 0.5)",
	"rgb(255 0 0 / 50%)",
	"rgb(255 0 0 / 0)",
	"rgb(255 0 0 / 1)",
	"rgb(255 0 0 / 100%)",
	// Out-of-range
	"rgb(300 0 0)",
	"rgb(-50 0 0)",
	"rgb(255 -10 510)",
	"rgb(150% 0% 0%)",
	"rgb(-50% 0% 0%)",
	// `none` channels
	"rgb(none 0 0)",
	"rgb(255 none 0)",
	"rgb(255 0 none)",
	"rgb(none none none)",
	"rgb(255 0 0 / none)",
	"rgb(none none none / none)",
	// rgba()
	"rgba(255 0 0)",
	"rgba(255 0 0 / 0.5)",
	// Legacy with commas (integers)
	"rgb(255, 0, 0)",
	"rgb(0, 255, 0)",
	"rgb(0, 0, 0)",
	"rgb(255, 255, 255)",
	"rgba(255, 0, 0, 0.5)",
	"rgba(255, 0, 0, 0)",
	"rgba(255, 0, 0, 1)",
	"rgba(255, 0, 0, 50%)",
	// Legacy with commas (percentages)
	"rgb(50%, 25%, 0%)",
	"rgba(50%, 25%, 0%, 0.5)",
	// Whitespace
	"rgb( 255 0 0 )",
	"  rgb(255 0 0)  ",
	"rgb(  255  ,  0  ,  0  )",
	"RGB(255 0 0)",
	"Rgb(255 0 0)",
	// Invalid
	"rgb(255 0)",
	"rgb(255 0 0 0)",
	"rgb()",
	"rgb(255, 0)",
	"rgb(255, 0, 0,)",
	"rgb(50%, 50, 0%)",
	"rgb(255 0 0 0.5)",
	"rgb(255 0 0,)",
];

const HSL_INPUTS = [
	// Modern
	"hsl(0 100% 50%)",
	"hsl(120 100% 50%)",
	"hsl(240 100% 50%)",
	"hsl(360 100% 50%)",
	"hsl(180 50% 50%)",
	"hsl(0 0% 0%)",
	"hsl(0 0% 100%)",
	"hsl(0 0% 50%)",
	// Hue units
	"hsl(180deg 50% 50%)",
	"hsl(0.5turn 50% 50%)",
	"hsl(200grad 50% 50%)",
	"hsl(3.14159rad 50% 50%)",
	// Number-as-percent (culori extension)
	"hsl(180 50 50)",
	// Out-of-range hue
	"hsl(720 50% 50%)",
	"hsl(-180 50% 50%)",
	// Modern with alpha
	"hsl(180 50% 50% / 0.5)",
	"hsl(180 50% 50% / 50%)",
	// `none`
	"hsl(none 50% 50%)",
	"hsl(180 none 50%)",
	"hsl(180 50% none)",
	"hsl(180 50% 50% / none)",
	// hsla
	"hsla(180 50% 50%)",
	"hsla(180 50% 50% / 0.5)",
	// Legacy comma-form
	"hsl(180, 50%, 50%)",
	"hsl(0, 100%, 50%)",
	"hsl(120, 100%, 50%)",
	"hsla(180, 50%, 50%, 0.5)",
	"hsla(180, 50%, 50%, 50%)",
	"hsl(180deg, 50%, 50%)",
	"hsl(0.5turn, 50%, 50%)",
	// Whitespace / case
	"  hsl(180 50% 50%)  ",
	"HSL(180 50% 50%)",
	// Invalid
	"hsl(180 50%, 50%)",
	"hsl(180, 50% 50%)",
	"hsl(180 50% 50% 0.5)",
	"hsl(180 50%)",
	"hsl()",
	"hsl(50% 50% 50%)",
];

const HWB_INPUTS = [
	// Modern only (no legacy form)
	"hwb(0 0% 0%)",
	"hwb(120 25% 25%)",
	"hwb(240 50% 50%)",
	"hwb(0 100% 0%)",
	"hwb(0 0% 100%)",
	"hwb(180 30% 30%)",
	// Hue units
	"hwb(180deg 30% 30%)",
	"hwb(0.5turn 30% 30%)",
	"hwb(200grad 30% 30%)",
	"hwb(3.14159rad 30% 30%)",
	// Alpha
	"hwb(180 30% 30% / 0.5)",
	"hwb(180 30% 30% / 50%)",
	// `none`
	"hwb(none 30% 30%)",
	"hwb(180 none 30%)",
	"hwb(180 30% none)",
	// Whitespace
	"  hwb(180 30% 30%)  ",
	"HWB(180 30% 30%)",
	// Invalid (legacy comma form not allowed for hwb)
	"hwb(180, 30%, 30%)",
	"hwb(180 30%)",
	"hwb()",
];

const LAB_INPUTS = [
	"lab(0 0 0)",
	"lab(50 25 -25)",
	"lab(100 0 0)",
	"lab(50% 25 -25)",
	"lab(50% 50% -50%)",
	"lab(0% 0 0)",
	"lab(100% 0 0)",
	"lab(50 25 -25 / 0.5)",
	"lab(50 25 -25 / 50%)",
	"lab(none 25 -25)",
	"lab(50 none -25)",
	"lab(50 25 none)",
	"lab(50 25 -25 / none)",
	// Out-of-range L gets clamped to 0..100
	"lab(150 25 -25)",
	"lab(-10 25 -25)",
	"lab(150% 25 -25)",
	// Whitespace / case
	"  lab(50 25 -25)  ",
	"LAB(50 25 -25)",
	// Invalid
	"lab(50, 25, -25)",
	"lab()",
	"lab(50 25)",
];

const LCH_INPUTS = [
	"lch(0 0 0)",
	"lch(50 30 180)",
	"lch(100 0 0)",
	"lch(50% 30 180)",
	"lch(50 50% 180)",
	"lch(50 30 180deg)",
	"lch(50 30 0.5turn)",
	"lch(50 30 200grad)",
	"lch(50 30 3.14159rad)",
	"lch(50 30 180 / 0.5)",
	"lch(50 30 180 / 50%)",
	"lch(none 30 180)",
	"lch(50 none 180)",
	"lch(50 30 none)",
	// Out-of-range
	"lch(150 30 180)",
	"lch(-10 30 180)",
	"lch(50 -30 180)",
	"lch(50 30 720)",
	// Whitespace / case
	"  lch(50 30 180)  ",
	"LCH(50 30 180)",
	// Invalid
	"lch(50, 30, 180)",
	"lch()",
	"lch(50 30)",
];

const OKLAB_INPUTS = [
	"oklab(0 0 0)",
	"oklab(0.5 0.1 -0.1)",
	"oklab(1 0 0)",
	"oklab(50% 0.1 -0.1)",
	"oklab(50% 50% -50%)",
	"oklab(0% 0 0)",
	"oklab(100% 0 0)",
	"oklab(0.5 0.1 -0.1 / 0.5)",
	"oklab(0.5 0.1 -0.1 / 50%)",
	"oklab(none 0.1 -0.1)",
	"oklab(0.5 none -0.1)",
	"oklab(0.5 0.1 none)",
	// Out-of-range L
	"oklab(1.5 0.1 -0.1)",
	"oklab(-0.1 0.1 -0.1)",
	"oklab(150% 0.1 -0.1)",
	// Whitespace / case
	"  oklab(0.5 0.1 -0.1)  ",
	"OKLAB(0.5 0.1 -0.1)",
	// Invalid
	"oklab(0.5, 0.1, -0.1)",
	"oklab()",
];

const OKLCH_INPUTS = [
	"oklch(0 0 0)",
	"oklch(0.5 0.1 180)",
	"oklch(1 0 0)",
	"oklch(50% 0.1 180)",
	"oklch(0.5 50% 180)",
	"oklch(0.5 0.1 180deg)",
	"oklch(0.5 0.1 0.5turn)",
	"oklch(0.5 0.1 200grad)",
	"oklch(0.5 0.1 3.14159rad)",
	"oklch(0.5 0.1 180 / 0.5)",
	"oklch(0.5 0.1 180 / 50%)",
	"oklch(none 0.1 180)",
	"oklch(0.5 none 180)",
	"oklch(0.5 0.1 none)",
	// Out-of-range
	"oklch(1.5 0.1 180)",
	"oklch(0.5 -0.1 180)",
	// Whitespace / case
	"  oklch(0.5 0.1 180)  ",
	"OKLCH(0.5 0.1 180)",
	// Invalid
	"oklch(0.5, 0.1, 180)",
	"oklch()",
];

const COLOR_FN_INPUTS = [
	// srgb
	"color(srgb 1 0 0)",
	"color(srgb 0 1 0)",
	"color(srgb 0 0 1)",
	"color(srgb 0.5 0.5 0.5)",
	"color(srgb 1 0 0 / 0.5)",
	"color(srgb 1 0 0 / 50%)",
	"color(srgb none 0 0)",
	"color(srgb 1 none 0)",
	"color(srgb 1 0 none)",
	"color(srgb none none none)",
	// Out-of-range
	"color(srgb 1.5 0 0)",
	"color(srgb -0.5 0 0)",
	// Percentages
	"color(srgb 100% 0% 0%)",
	"color(srgb 50% 25% 0%)",
	// srgb-linear
	"color(srgb-linear 0.5 0.5 0.5)",
	"color(srgb-linear 1 0 0)",
	"color(srgb-linear 0 1 0 / 0.5)",
	"color(srgb-linear none 0 0)",
	// xyz / xyz-d65
	"color(xyz 0.5 0.5 0.5)",
	"color(xyz-d65 0.5 0.5 0.5)",
	"color(xyz 0 0 0)",
	"color(xyz-d65 1 1 1)",
	"color(xyz 0.4 0.5 0.6 / 0.5)",
	// xyz-d50
	"color(xyz-d50 0.5 0.5 0.5)",
	"color(xyz-d50 0 0 0)",
	"color(xyz-d50 1 1 1)",
	"color(xyz-d50 0.4 0.5 0.6 / 0.5)",
	"color(xyz-d50 none none none)",
	// Whitespace / case (note: profile name is case-sensitive in culori)
	"  color(srgb 1 0 0)  ",
	"COLOR(srgb 1 0 0)",
	// Invalid
	"color(unknown-profile 1 0 0)",
	"color(srgb)",
	"color(srgb 1 0)",
	"color()",
];

// Special / edge inputs.
const SPECIAL_INPUTS = [
	"transparent",
	"  transparent  ",
	"TRANSPARENT",
	"Transparent",
	// Case variants on named colors
	"RED",
	"Red",
	"BLUE",
	"  red  ",
	"\tred\n",
	// Invalids that should reject
	"",
	"   ",
	"not a color",
	"foobar",
	"red ",
	" red",
	"red red",
];

// `color()` calls for profiles culor doesn't support yet. Culori parses
// these into modes (`p3`, `rec2020`, `prophoto`, `a98`) we don't carry,
// so they are excluded from the fixture and verified separately in the
// Rust unit tests.

const ALL_INPUTS = [
	...NAMED_SAMPLE,
	...HEX_INPUTS,
	...RGB_INPUTS,
	...HSL_INPUTS,
	...HWB_INPUTS,
	...LAB_INPUTS,
	...LCH_INPUTS,
	...OKLAB_INPUTS,
	...OKLCH_INPUTS,
	...COLOR_FN_INPUTS,
	...SPECIAL_INPUTS,
];

// Modes culor supports. Anything else from culori (`p3`, `rec2020`,
// `prophoto`, `a98`, etc.) is dropped from the fixture.
const SUPPORTED_MODES = new Set([
	"rgb",
	"lrgb",
	"hsl",
	"hsv",
	"hwb",
	"lab",
	"lch",
	"oklab",
	"oklch",
	"xyz50",
	"xyz65",
]);

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
