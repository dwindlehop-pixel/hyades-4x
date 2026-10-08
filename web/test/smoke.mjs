// Smoke test of the shipped viewer module (docs/Hyades_interface.md §8):
// loads hyades_viewer.wasm the way the page does, opens every replay in the
// index, renders frames across the span in both modes, and fails if a frame
// is blank or a call throws. With --png <dir>, writes each frame as a PNG.
//
// Usage: node web/test/smoke.mjs <wasm> <replay dir> [--png <dir>]
import fs from "node:fs";
import path from "node:path";
import zlib from "node:zlib";

const [wasmPath, replayDir] = process.argv.slice(2);
const pngAt = process.argv.indexOf("--png");
const pngDir = pngAt > 0 ? process.argv[pngAt + 1] : null;
if (pngDir) fs.mkdirSync(pngDir, { recursive: true });

const { instance } = await WebAssembly.instantiate(fs.readFileSync(wasmPath), {});
const hv = instance.exports;
const bytes = (ptr, len) => new Uint8Array(hv.memory.buffer, ptr, len);
const text = (which, first = 0, rows = 0) => {
  const p = hv.hv_text(which, first, rows);
  return new TextDecoder().decode(bytes(p, hv.hv_text_len()).slice());
};
const put = (data) => bytes(hv.hv_input(data.length), data.length).set(data);

function crc32(buf) {
  let c, crc = 0xffffffff;
  for (const b of buf) {
    c = (crc ^ b) & 0xff;
    for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    crc = (crc >>> 8) ^ c;
  }
  return (crc ^ 0xffffffff) >>> 0;
}
function png(rgba, w, h) {
  const raw = Buffer.alloc((w * 4 + 1) * h);
  for (let y = 0; y < h; y++) {
    raw[y * (w * 4 + 1)] = 0;
    Buffer.from(rgba.buffer, rgba.byteOffset + y * w * 4, w * 4).copy(raw, y * (w * 4 + 1) + 1);
  }
  const chunk = (type, data) => {
    const len = Buffer.alloc(4); len.writeUInt32BE(data.length);
    const td = Buffer.concat([Buffer.from(type), data]);
    const crc = Buffer.alloc(4); crc.writeUInt32BE(crc32(td));
    return Buffer.concat([len, td, crc]);
  };
  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(w, 0); ihdr.writeUInt32BE(h, 4); ihdr[8] = 8; ihdr[9] = 6;
  return Buffer.concat([Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]), chunk("IHDR", ihdr),
    chunk("IDAT", zlib.deflateSync(raw)), chunk("IEND", Buffer.alloc(0))]);
}

const index = JSON.parse(fs.readFileSync(path.join(replayDir, "index.json"), "utf8"));
let failures = 0;
for (const entry of index) {
  const data = fs.readFileSync(path.join(replayDir, entry.file));
  put(data);
  let t = performance.now();
  if (hv.hv_load(960, 600) !== 0) {
    console.log(`FAIL ${entry.name}: ${text(3)}`);
    failures++;
    continue;
  }
  const loadMs = performance.now() - t;
  const times = { 0: [], 1: [] };
  for (const f of [0, 0.25, 0.5, 0.75, 1]) {
    hv.hv_seek_fraction(f);
    for (const mode of [0, 1]) {
      hv.hv_set_mode(mode);
      t = performance.now();
      const ptr = hv.hv_render();
      times[mode].push(performance.now() - t);
      const w = hv.hv_frame_w(), h = hv.hv_frame_h();
      const px = bytes(ptr, w * h * 4).slice();
      const distinct = new Set();
      for (let i = 0; i < px.length; i += 4 * 7) distinct.add((px[i] << 16) | (px[i + 1] << 8) | px[i + 2]);
      if (distinct.size < 3) {
        console.log(`FAIL ${entry.name} f=${f} mode=${mode}: ${distinct.size} colors`);
        failures++;
      }
      if (pngDir) {
        const name = `${entry.name}-${mode ? "juicy" : "tactical"}-${String(Math.round(f * 100)).padStart(3, "0")}.png`;
        fs.writeFileSync(path.join(pngDir, name), png(px, w, h));
      }
    }
  }
  const med = (a) => a.sort((x, y) => x - y)[a.length >> 1].toFixed(1);
  console.log(`${entry.name.padEnd(10)} load ${loadMs.toFixed(0)} ms · render tactical ${med(times[0])} ms, juicy ${med(times[1])} ms · ${text(0)}`);
}
process.exit(failures ? 1 : 0);
