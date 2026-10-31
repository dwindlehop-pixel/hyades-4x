// The wasm32 half of the parity check (T-128). Usage:
//
//   node tools/parity/compare.mjs <hyades_parity.wasm> <native output file>
//
// Runs every case in the wasm32 module, prints its line in the native
// binary's format (`kind seats seed horizon digest`), and exits 1 at the first line
// that differs from the native output. Each line is printed as it finishes, so
// a slow run shows progress.
import { readFileSync } from "node:fs";

const [wasmPath, nativePath] = process.argv.slice(2);
if (!wasmPath || !nativePath) {
  console.error("usage: node compare.mjs <hyades_parity.wasm> <native output file>");
  process.exit(2);
}
const native = readFileSync(nativePath, "utf8").trim().split("\n");
const { instance } = await WebAssembly.instantiate(readFileSync(wasmPath), {});
const m = instance.exports;
const hex = (x) => (x >>> 0).toString(16).padStart(8, "0");

const n = m.parity_cases();
if (n !== native.length) {
  console.error(`case count: wasm32 ${n}, native ${native.length}`);
  process.exit(1);
}
let failed = false;
for (let i = 0; i < n; i++) {
  const t0 = Date.now();
  m.parity_run(i);
  const line = `${m.parity_case(i, 0)} ${m.parity_case(i, 1)} ${m.parity_case(i, 2)} ${m.parity_case(i, 3)} ${hex(m.parity_hi())}${hex(m.parity_lo())}`;
  const same = line === native[i];
  console.log(`${same ? "same" : "DIFF"}  wasm32 ${line}  native ${native[i]}  (${((Date.now() - t0) / 1000).toFixed(1)} s)`);
  failed ||= !same;
}
if (failed) {
  console.error("native and wasm32 runs disagree: the engine is not bit-identical across targets");
  process.exit(1);
}
console.log(`all ${n} cases bit-identical across native and wasm32`);
