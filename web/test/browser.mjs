// Browser test of the deployed page (docs/Hyades_interface.md §8): serves
// web/ as Pages will, opens it in headless Chromium, and checks that
//   - the page starts on WebGL2 with no console error, and draws;
//   - the GPU juicy frame matches the module's CPU juicy frame (the
//     reference) within a stated tolerance;
//   - play, the log text filter and a log row's seek work.
// With --shots <dir>, saves screenshots of both modes.
//
// Usage: node web/test/browser.mjs <site dir> [--shots <dir>]
// Needs Playwright (npm) and a Chromium it can launch.
import fs from "node:fs";
import http from "node:http";
import path from "node:path";
import { createRequire } from "node:module";
import { execSync } from "node:child_process";

const require = createRequire(import.meta.url);
let playwright;
try {
  playwright = require("playwright");
} catch {
  playwright = require(path.join(execSync("npm root -g").toString().trim(), "playwright"));
}

const site = process.argv[2];
const shotsAt = process.argv.indexOf("--shots");
const shots = shotsAt > 0 ? process.argv[shotsAt + 1] : null;
if (shots) fs.mkdirSync(shots, { recursive: true });

const types = { ".html": "text/html", ".js": "text/javascript", ".css": "text/css", ".json": "application/json", ".wasm": "application/wasm" };
const server = http.createServer((req, res) => {
  const url = new URL(req.url, "http://x");
  const file = path.join(path.resolve(site), decodeURIComponent(url.pathname === "/" ? "/index.html" : url.pathname));
  if (!file.startsWith(path.resolve(site)) || !fs.existsSync(file)) {
    res.writeHead(404).end();
    return;
  }
  res.writeHead(200, { "content-type": types[path.extname(file)] ?? "application/octet-stream" });
  fs.createReadStream(file).pipe(res);
});
await new Promise((r) => server.listen(0, "127.0.0.1", r));
const base = `http://127.0.0.1:${server.address().port}`;

// CI launches the runner's installed Chrome (HYADES_CHROME_CHANNEL=chrome)
// so no browser is downloaded; locally, Playwright's own Chromium.
const browser = await playwright.chromium.launch({
  channel: process.env.HYADES_CHROME_CHANNEL || undefined,
  args: ["--use-angle=swiftshader", "--enable-unsafe-swiftshader"],
});
const page = await browser.newPage({ viewport: { width: 1280, height: 800 } });
const errors = [];
page.on("console", (m) => { if (m.type() === "error") errors.push(m.text()); });
page.on("pageerror", (e) => errors.push(String(e)));

let failures = 0;
const check = (ok, what) => {
  console.log(`${ok ? "ok  " : "FAIL"} ${what}`);
  if (!ok) failures++;
};

try {
  await page.goto(`${base}/?replay=sentries`);
  await page.waitForFunction(() => window.hyades?.hv() && document.getElementById("label").textContent !== "", null, { timeout: 30000 });
  const renderer = await page.evaluate(() => document.body.dataset.renderer);
  check(renderer === "webgl2", `the page renders with ${renderer}`);

  // The canvas has drawn something other than the ground.
  await page.waitForTimeout(300);
  const colors = await page.evaluate(() => {
    const c = document.getElementById("view");
    const gl = c.getContext("webgl2");
    const px = new Uint8Array(c.width * c.height * 4);
    gl.readPixels(0, 0, c.width, c.height, gl.RGBA, gl.UNSIGNED_BYTE, px);
    const set = new Set();
    for (let i = 0; i < px.length; i += 4 * 13) set.add((px[i] << 16) | (px[i + 1] << 8) | px[i + 2]);
    return set.size;
  });
  check(colors >= 4, `tactical draws: ${colors} distinct colors sampled`);
  if (shots) await page.screenshot({ path: path.join(shots, "tactical.png") });

  // GPU juicy against the module's CPU juicy, frame by frame.
  const diff = await page.evaluate(() => {
    const hv = window.hyades.hv(), gpu = window.hyades.gpu();
    hv.hv_seek_fraction(0.3);
    hv.hv_set_mode(1);
    const c = document.getElementById("view");
    const w = Math.max(1, c.clientWidth), h = Math.max(1, c.clientHeight);
    hv.hv_prepare_lights();
    const n0 = hv.hv_lights_len(0), n1 = hv.hv_lights_len(1);
    const scene = new Float32Array(hv.memory.buffer, hv.hv_lights_ptr(0), n0 * 6).slice();
    const terr = new Float32Array(hv.memory.buffer, hv.hv_lights_ptr(1), n1 * 6).slice();
    gpu.juicy(scene, n0, terr, n1, w, h, "check");
    const g = gpu.read("check");
    const p = hv.hv_render();
    const cpu = new Uint8Array(hv.memory.buffer, p, hv.hv_frame_w() * hv.hv_frame_h() * 4).slice();
    if (cpu.length !== g.length) return { error: `sizes ${cpu.length} ${g.length}` };
    const d = [];
    let sum = 0, lit = 0;
    for (let i = 0; i < g.length; i += 4) {
      const m = Math.max(Math.abs(g[i] - cpu[i]), Math.abs(g[i + 1] - cpu[i + 1]), Math.abs(g[i + 2] - cpu[i + 2]));
      d.push(m);
      sum += m;
      if (cpu[i] + cpu[i + 1] + cpu[i + 2] > 30) lit++;
    }
    d.sort((a, b) => a - b);
    return { lights: n0 + n1, lit, mean: sum / d.length, p99: d[Math.floor(d.length * 0.99)], max: d[d.length - 1] };
  });
  console.log("     GPU against CPU juicy:", JSON.stringify(diff));
  check(!diff.error && diff.lit > 100, "the juicy frame is lit");
  check(!diff.error && diff.mean < 1.0 && diff.p99 <= 4 && diff.max <= 24, "GPU juicy matches the CPU reference (mean < 1 code, 99th percentile ≤ 4, max ≤ 24)");
  await page.evaluate(() => window.hyades.draw());
  if (shots) await page.screenshot({ path: path.join(shots, "juicy.png") });

  // Play moves the clock.
  const t0 = await page.evaluate(() => window.hyades.hv().hv_time());
  await page.keyboard.press("Space");
  await page.waitForTimeout(600);
  await page.keyboard.press("k");
  const t1 = await page.evaluate(() => window.hyades.hv().hv_time());
  check(t1 > t0, `space plays: t ${t0.toFixed(4)} → ${t1.toFixed(4)}`);

  // The text filter narrows the log; a row seeks the clock to its event.
  await page.selectOption("#window", "0");
  const all = Number((await page.textContent("#logcount")).split(" ")[0]);
  await page.fill("#text", "wrecked");
  await page.waitForTimeout(200);
  const some = Number((await page.textContent("#logcount")).split(" ")[0]);
  check(all > some && some > 0, `the text filter narrows the log: ${all} → ${some}`);
  await page.uncheck("#followlog");
  await page.click("#log .row");
  const seek = await page.evaluate(() => {
    const row = document.querySelector("#log .row");
    return [Number(row.children[0].textContent), window.hyades.hv().hv_time()];
  });
  check(Math.abs(seek[0] - seek[1]) < 1e-3, `a log row seeks to its event: ${seek[0]} vs ${seek[1].toFixed(4)}`);

  const palette = await browser.newPage();
  await palette.goto(`${base}/palette.html`);
  await palette.waitForFunction(() => document.querySelectorAll("#source .chip").length === 40, null, { timeout: 15000 });
  check(true, "the palette sheet lists 40 source colors");
  if (shots) await palette.screenshot({ path: path.join(shots, "palette.png"), fullPage: true });

  check(errors.length === 0, `no console errors${errors.length ? ": " + errors.join(" | ") : ""}`);
} catch (e) {
  check(false, `the test ran: ${e.message}`);
} finally {
  await browser.close();
  server.close();
}
process.exit(failures ? 1 : 0);
