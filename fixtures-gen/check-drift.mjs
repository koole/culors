// Compare regenerated fixtures against committed fixtures with floating-point
// tolerance. Across platforms (Linux x86_64 / macOS arm64) the libm functions
// (atan2, sqrt, cbrt, pow) can differ by up to 1 ULP, producing identical f64
// values that round-trip differently through Number.toString. The Rust test
// suite handles this via epsilon tolerance; this script does the same for the
// fixture-drift CI gate.
//
// Usage: node fixtures-gen/check-drift.mjs <dir1> <dir2>
// Exits 0 if all numbers within tolerance, 1 otherwise.

import { readdirSync, readFileSync, statSync } from 'node:fs';
import { join } from 'node:path';

const ABS_TOL = 1e-13;
const REL_TOL = 1e-13;

function listJsonFiles(dir) {
    const out = [];
    for (const entry of readdirSync(dir)) {
        const p = join(dir, entry);
        if (statSync(p).isDirectory()) out.push(...listJsonFiles(p));
        else if (entry.endsWith('.json')) out.push(p);
    }
    return out.sort();
}

function relPath(root, abs) {
    return abs.slice(root.length + 1);
}

function close(a, b) {
    if (Number.isNaN(a) && Number.isNaN(b)) return true;
    if (a === b) return true;
    if (typeof a !== 'number' || typeof b !== 'number') return false;
    if (!Number.isFinite(a) || !Number.isFinite(b)) return a === b;
    const diff = Math.abs(a - b);
    if (diff <= ABS_TOL) return true;
    return diff <= REL_TOL * Math.max(Math.abs(a), Math.abs(b));
}

function compare(path, a, b, errs) {
    if (typeof a === 'number' || typeof b === 'number') {
        if (!close(a, b)) errs.push(`${path}: ${a} vs ${b} (Δ=${Math.abs(a - b)})`);
        return;
    }
    if (a === null || b === null || typeof a !== 'object' || typeof b !== 'object') {
        if (a !== b) errs.push(`${path}: ${JSON.stringify(a)} vs ${JSON.stringify(b)}`);
        return;
    }
    if (Array.isArray(a) !== Array.isArray(b)) {
        errs.push(`${path}: array vs object`);
        return;
    }
    const aKeys = Array.isArray(a) ? a.map((_, i) => i) : Object.keys(a).sort();
    const bKeys = Array.isArray(b) ? b.map((_, i) => i) : Object.keys(b).sort();
    if (aKeys.length !== bKeys.length || aKeys.some((k, i) => k !== bKeys[i])) {
        errs.push(`${path}: key set differs (${aKeys.join(',')} vs ${bKeys.join(',')})`);
        return;
    }
    for (const k of aKeys) compare(`${path}/${k}`, a[k], b[k], errs);
}

const [dirA, dirB] = process.argv.slice(2);
if (!dirA || !dirB) {
    console.error('usage: check-drift.mjs <committed-dir> <regenerated-dir>');
    process.exit(2);
}

const filesA = listJsonFiles(dirA);
const filesB = listJsonFiles(dirB);
const namesA = new Set(filesA.map(p => relPath(dirA, p)));
const namesB = new Set(filesB.map(p => relPath(dirB, p)));

const onlyA = [...namesA].filter(n => !namesB.has(n));
const onlyB = [...namesB].filter(n => !namesA.has(n));
if (onlyA.length || onlyB.length) {
    if (onlyA.length) console.error(`Missing from regen: ${onlyA.join(', ')}`);
    if (onlyB.length) console.error(`New in regen: ${onlyB.join(', ')}`);
    process.exit(1);
}

let totalErrs = 0;
for (const name of [...namesA].sort()) {
    const a = JSON.parse(readFileSync(join(dirA, name), 'utf8'));
    const b = JSON.parse(readFileSync(join(dirB, name), 'utf8'));
    const errs = [];
    compare(name, a, b, errs);
    if (errs.length) {
        console.error(`\n${name}: ${errs.length} mismatch(es) beyond ${ABS_TOL} tolerance`);
        for (const e of errs.slice(0, 10)) console.error(`  ${e}`);
        if (errs.length > 10) console.error(`  ... and ${errs.length - 10} more`);
        totalErrs += errs.length;
    }
}

if (totalErrs > 0) {
    console.error(`\nTotal: ${totalErrs} fixture-drift mismatches beyond floating-point tolerance.`);
    process.exit(1);
}
console.log('Fixture drift within floating-point tolerance.');
