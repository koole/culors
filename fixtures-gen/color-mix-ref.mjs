// Reference implementation of CSS color-mix() built on top of culori's
// converter() pipeline. Used to generate expected values for the Rust
// parser tests. Mirrors the W3C CSS Color Module 5 § 11 algorithm:
// percentage normalization, premultiplied interpolation, post-mix alpha
// scaling when the percentage sum is below 100.

import * as culori from 'culori';

const HUE = { hsl: 'h', hwb: 'h', lch: 'h', oklch: 'h' };
const CHANS = {
  rgb: ['r', 'g', 'b'],
  lrgb: ['r', 'g', 'b'],
  hsl: ['h', 's', 'l'],
  hwb: ['h', 'w', 'b'],
  lab: ['l', 'a', 'b'],
  lch: ['l', 'c', 'h'],
  oklab: ['l', 'a', 'b'],
  oklch: ['l', 'c', 'h'],
  xyz50: ['x', 'y', 'z'],
  xyz65: ['x', 'y', 'z'],
};

const SPACE_TO_MODE = {
  srgb: 'rgb',
  'srgb-linear': 'lrgb',
  hsl: 'hsl',
  hwb: 'hwb',
  lab: 'lab',
  lch: 'lch',
  oklab: 'oklab',
  oklch: 'oklch',
  xyz: 'xyz65',
  'xyz-d50': 'xyz50',
  'xyz-d65': 'xyz65',
};

function alphaOf(c) {
  return c.alpha === undefined ? 1 : c.alpha;
}

function fixupHue(ha, hb, strategy) {
  if (Number.isNaN(ha) || Number.isNaN(hb)) return [ha, hb];
  const norm = (v) => ((v % 360) + 360) % 360;
  let a = norm(ha);
  let b = norm(hb);
  let d = b - a;
  if (strategy === 'shorter') {
    if (d > 180) b -= 360;
    else if (d < -180) b += 360;
  } else if (strategy === 'longer') {
    if (-180 < d && d < 180) {
      b += d > 0 ? -360 : 360;
    }
  } else if (strategy === 'increasing') {
    if (b < a) b += 360;
  } else if (strategy === 'decreasing') {
    if (b > a) b -= 360;
  }
  return [a, b];
}

export function mixCss(input) {
  // Parse string of form "color-mix(in <space> [<hue> hue]?, <c1> [p1%]?, <c2> [p2%]?)"
  const m = input.match(/^\s*color-mix\(\s*(.+)\s*\)\s*$/i);
  if (!m) throw new Error('not color-mix');
  const inner = m[1];
  // Split on top-level commas (no nested parens here for simple cases used in tests; tests don't nest).
  const parts = splitTopLevel(inner, ',');
  if (parts.length !== 3) throw new Error('expected 3 parts');
  const methodPart = parts[0].trim();
  const c1Part = parts[1].trim();
  const c2Part = parts[2].trim();
  const methodMatch = methodPart.match(/^in\s+([a-zA-Z0-9-]+)(?:\s+(shorter|longer|increasing|decreasing)(?:\s+hue)?)?$/i);
  if (!methodMatch) throw new Error('bad method');
  const space = methodMatch[1].toLowerCase();
  const hueStrategy = (methodMatch[2] || 'shorter').toLowerCase();
  const mode = SPACE_TO_MODE[space];
  if (!mode) throw new Error('unsupported space ' + space);

  const [c1Str, p1] = splitColorAndPct(c1Part);
  const [c2Str, p2] = splitColorAndPct(c2Part);
  let p1n = p1, p2n = p2;
  if (p1n === null && p2n === null) { p1n = 50; p2n = 50; }
  else if (p1n === null) p1n = 100 - p2n;
  else if (p2n === null) p2n = 100 - p1n;
  const sum = p1n + p2n;
  if (sum === 0) return null;
  let alphaMult = 1;
  if (sum !== 100) {
    if (sum < 100) {
      alphaMult = sum / 100;
    }
    p1n = (p1n / sum) * 100;
    p2n = (p2n / sum) * 100;
  }
  const t = p2n / 100;

  const c1raw = culori.parse(c1Str);
  const c2raw = culori.parse(c2Str);
  if (!c1raw || !c2raw) throw new Error('bad color');
  const conv = culori.converter(mode);
  const ca = conv(c1raw);
  const cb = conv(c2raw);
  const aa = alphaOf(ca);
  const ab = alphaOf(cb);

  const chs = CHANS[mode];
  const hueCh = HUE[mode];
  const out = { mode };
  for (const ch of chs) {
    let va = ca[ch];
    let vb = cb[ch];
    if (ch === hueCh) {
      [va, vb] = fixupHue(va, vb, hueStrategy);
      const v = va * (1 - t) + vb * t;
      out[ch] = ((v % 360) + 360) % 360;
    } else {
      const pa = (Number.isNaN(va) ? 0 : va) * aa;
      const pb = (Number.isNaN(vb) ? 0 : vb) * ab;
      out[ch] = pa * (1 - t) + pb * t;
    }
  }
  let alpha = aa * (1 - t) + ab * t;
  if (alpha > 0) {
    for (const ch of chs) {
      if (ch !== hueCh) out[ch] = out[ch] / alpha;
    }
  }
  alpha *= alphaMult;
  out.alpha = alpha;
  return out;
}

function splitTopLevel(s, sep) {
  const out = [];
  let depth = 0;
  let cur = '';
  for (const ch of s) {
    if (ch === '(') depth++;
    else if (ch === ')') depth--;
    if (ch === sep && depth === 0) { out.push(cur); cur = ''; }
    else cur += ch;
  }
  out.push(cur);
  return out;
}

function splitColorAndPct(s) {
  // pct is a trailing number followed by %, optionally with leading whitespace
  const m = s.match(/^(.*?)(?:\s+(\d+(?:\.\d+)?|\.\d+)%)?\s*$/);
  if (!m) return [s, null];
  // Handle prefix percentage: "70% red" form
  const prefix = s.match(/^(\d+(?:\.\d+)?|\.\d+)%\s+(.+)$/);
  if (prefix) return [prefix[2].trim(), parseFloat(prefix[1])];
  if (m[2] === undefined) return [m[1].trim(), null];
  return [m[1].trim(), parseFloat(m[2])];
}

const inputs = [
  'color-mix(in srgb, red, blue)',
  'color-mix(in srgb, red 70%, blue)',
  'color-mix(in srgb, red, blue 70%)',
  'color-mix(in srgb, red 30%, blue 70%)',
  'color-mix(in lab, red, blue)',
  'color-mix(in lch, red, blue)',
  'color-mix(in oklab, red, blue)',
  'color-mix(in oklch, red, blue)',
  'color-mix(in oklch, red 70%, blue)',
  'color-mix(in hsl, red, blue)',
  'color-mix(in hwb, red, blue)',
  'color-mix(in hsl shorter hue, red, blue)',
  'color-mix(in hsl longer hue, red, blue)',
  'color-mix(in hsl increasing hue, red, blue)',
  'color-mix(in hsl decreasing hue, red, blue)',
  'color-mix(in oklch shorter hue, red, blue)',
  'color-mix(in oklch longer hue, red, blue)',
  'color-mix(in srgb, transparent, red)',
  'color-mix(in srgb, red 30%, blue 30%)',
  'color-mix(in srgb, red 0%, blue 0%)',
  'color-mix(in oklab, white, black)',
  'color-mix(in oklab, white 25%, black 75%)',
  'color-mix(in srgb, #ff0000, #0000ff)',
  'color-mix(in xyz, red, blue)',
  'color-mix(in xyz-d50, red, blue)',
  'color-mix(in srgb, rgba(255 0 0 / 0.5), rgba(0 0 255 / 0.5))',
  'color-mix(in lch longer hue, red 30%, blue 70%)',
  'color-mix(in srgb-linear, red, blue)',
];

for (const input of inputs) {
  let out;
  try { out = mixCss(input); }
  catch (e) { out = { error: e.message }; }
  console.log(input, '=>', JSON.stringify(out));
}
