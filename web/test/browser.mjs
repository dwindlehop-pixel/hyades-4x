// Browser test of the deployed client (docs/Hyades_interface.md §8): serves
// the site as Pages will, opens it in headless Chromium, and checks that
//   - the menu leads to the replay list and a replay opens in the viewer, on
//     WebGL2, with no console error, and draws;
//   - a click on a drawn hull selects it;
//   - the GPU juicy frame matches the module's CPU juicy frame (the
//     reference) within a stated tolerance;
//   - play, the log text filter and a log row's seek work;
//   - the palette editor, the palette screen and the old palette.html link;
//   - a replay answered 503 once is fetched again;
//   - a lost WebGL context falls back to the CPU renderer, which draws and picks;
//   - on a phone (Pixel 7 emulation) the theater fills most of the screen with
//     no sideways scroll, and a tap on a drawn hull selects it.
// With --shots <dir>, saves screenshots.
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
// Name the resource behind a "Failed to load resource" console error.
const failed = (r) => { if (r.status() >= 400) errors.push(`${r.status()} ${r.url()}`); };
page.on("response", failed);

/// Clicks (or taps) the first hull the module reports drawn inside the
/// canvas, and returns the inspector's text.
async function clickDrawn(p, press) {
  const box = await p.locator("#view").boundingBox();
  const drawn = await p.evaluate(() => window.hyades.text(10).split("\n").filter(Boolean).map((l) => l.split("\t").map(Number)));
  const hit = drawn.find(([, x, y]) => x > 20 && y > 20 && x < box.width - 20 && y < box.height - 20);
  if (!hit) return "nothing drawn";
  await press(box, hit[1], hit[2]);
  await p.waitForTimeout(200);
  return p.textContent("#inspector");
}

let failures = 0;
const check = (ok, what) => {
  console.log(`${ok ? "ok  " : "FAIL"} ${what}`);
  if (!ok) failures++;
};

try {
  await page.goto(`${base}/`);
  await page.waitForFunction(() => window.hyades?.hv(), null, { timeout: 30000 });
  check(await page.evaluate(() => document.body.dataset.screen) === "menu", "the client opens on the menu");
  check(!(await page.isDisabled("#menu-new")), "New game is enabled (the relay test, web/test/relay-match.mjs)");
  await page.click("#menu-replays");
  await page.click('button[data-replay="sentries"]');
  await page.waitForFunction(() => window.hyades.loaded() === "sentries" && document.getElementById("message").hidden, null, { timeout: 30000 });
  check(new URL(page.url()).search === "?replay=sentries", `the viewer's link names the replay: ${new URL(page.url()).search}`);
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

  // The glyph legend shows every role's mark, from the module.
  const roles = await page.evaluate(() => [...document.querySelectorAll("#roles li")].map((li) => [li.dataset.role, li.querySelectorAll(".mark i[style]").length]));
  check(roles.length === 7 && roles.some(([r, n]) => r === "Picket" && n === 5), `the legend shows the role marks: ${JSON.stringify(roles)}`);
  // The material key uses the ratified in-game names (galaxy §4.1).
  const mats = await page.evaluate(() => [...document.querySelectorAll("#materials li")].map((li) => li.dataset.material));
  check(mats.length === 9 && mats[0] === "Cage Ice" && mats[2] === "Voltslate" && mats.includes("Strange Matter"), `the material key uses the in-game names: ${JSON.stringify(mats)}`);

  // A click on a drawn hull selects it.
  const picked = await clickDrawn(page, (box, x, y) => page.mouse.click(box.x + x + 2, box.y + y + 2));
  check(picked.startsWith("Hull "), `a click on a drawn hull selects it: ${picked.split("\n")[0]}`);

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

  // Backward, then ▶: the clock turns and runs forward again.
  const now = () => page.evaluate(() => window.hyades.hv().hv_time());
  await page.click("#rewind");
  await page.waitForTimeout(400);
  const back = await now();
  await page.click("#play");
  await page.waitForTimeout(400);
  const fwd = await now();
  await page.waitForTimeout(400);
  const fwd2 = await now();
  await page.click("#play");
  check(back < t1 && fwd2 > fwd, `▶ after ◀ plays forward: t ${t1.toFixed(4)} → ${back.toFixed(4)} ◀, then ${fwd.toFixed(4)} → ${fwd2.toFixed(4)} ▶`);
  check(!(await page.evaluate(() => window.hyades.hv().hv_playing())), "▶ while playing forward pauses");

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

  // The live palette editor: a palette in the link opens with the page, a
  // slider redraws the canvas and rewrites the link, a hand-set color reaches
  // the settings line, and reset returns to the proposal.
  const tuned = await browser.newPage({ viewport: { width: 1280, height: 800 } });
  tuned.on("pageerror", (e) => errors.push(String(e)));
  await tuned.goto(`${base}/?replay=sentries#palette=${encodeURIComponent("ink=0.1 Hit=#00ff00")}`);
  await tuned.waitForFunction(() => window.hyades?.hv() && document.getElementById("label").textContent !== "", null, { timeout: 30000 });
  const line = await tuned.evaluate(() => window.hyades.text(9));
  check(line.startsWith("ink=0.1 ") && line.endsWith("Hit=#00ff00"), `a palette in the link opens with the page: ${line}`);
  const groundAt = () => tuned.evaluate(() => {
    window.hyades.draw();
    const c = document.getElementById("view");
    const gl = c.getContext("webgl2");
    const px = new Uint8Array(4);
    gl.readPixels(2, 2, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, px);
    return Array.from(px.slice(0, 3));
  });
  const before = await groundAt();
  await tuned.click("#tuner summary");
  await tuned.$eval("#tune-ink", (el) => { el.value = "0.25"; el.dispatchEvent(new Event("input", { bubbles: true })); });
  const after = await groundAt();
  const hash = decodeURIComponent(await tuned.evaluate(() => location.hash));
  check(after.join() !== before.join() && hash.includes("ink=0.25"), `the ink slider redraws the ground (${before} → ${after}) and rewrites the link`);
  await tuned.$eval('#tune-colors label[data-name="hy_red"] input', (el) => { el.value = "#123456"; el.dispatchEvent(new Event("input", { bubbles: true })); });
  check((await tuned.inputValue("#tune-text")).includes("hy_red=#123456"), "a hand-set color reaches the settings line");
  await tuned.click("#tune-reset");
  check(await tuned.evaluate(() => window.hyades.text(9)) === "ink=0.02 paper=0.95 warm=0.92 cool=0.6 pull=0.3 anchors=38,78,118,228 fill=0.7", "reset returns to the proposal");
  if (shots) await tuned.screenshot({ path: path.join(shots, "tuner.png") });

  // Back to the menu, then the palette screen; the old palette.html link
  // lands there too, with the palette it carried.
  await page.click("#screen-viewer .to-menu");
  check(await page.evaluate(() => document.body.dataset.screen) === "menu", "the viewer's back button returns to the menu");
  await page.click("#menu-palette");
  check(await page.evaluate(() => document.querySelectorAll("#chips-source .chip").length) === 40, "the palette screen lists 40 source colors");
  if (shots) await page.screenshot({ path: path.join(shots, "palette.png"), fullPage: true });
  const old = await browser.newPage();
  await old.goto(`${base}/palette.html#palette=${encodeURIComponent("ink=0.1")}`);
  await old.waitForFunction(() => window.hyades?.hv() && document.querySelectorAll("#chips-source .chip").length === 40, null, { timeout: 15000 });
  check((await old.evaluate(() => window.hyades.text(9))).startsWith("ink=0.1 "), "the old palette.html link opens the palette screen with its palette");

  // A replay the server answers 503 once is fetched again.
  const flaky = await browser.newPage();
  let refused = 0;
  await flaky.route("**/replays/sentries.json", (route) => {
    if (refused++ === 0) route.fulfill({ status: 503, body: "busy" });
    else route.continue();
  });
  await flaky.goto(`${base}/?replay=sentries`);
  await flaky.waitForFunction(() => window.hyades?.loaded() === "sentries" && document.getElementById("message").hidden, null, { timeout: 30000 });
  check(refused === 2, `a replay answered 503 is fetched again (${refused} requests)`);

  // A lost WebGL context moves drawing to the module's CPU renderer, on a
  // fresh canvas that still draws and still takes a click.
  await flaky.evaluate(() => window.hyades.gpu().gl.getExtension("WEBGL_lose_context").loseContext());
  await flaky.waitForFunction(() => document.body.dataset.renderer === "cpu", null, { timeout: 5000 });
  await flaky.click("#mode-juicy");
  await flaky.waitForTimeout(300);
  const cpuLit = await flaky.evaluate(() => {
    const c = document.getElementById("view");
    const d = c.getContext("2d").getImageData(0, 0, c.width, c.height).data;
    let lit = 0;
    for (let i = 0; i < d.length; i += 4) if (d[i] + d[i + 1] + d[i + 2] > 30) lit++;
    return lit;
  });
  check(cpuLit > 100, `after a lost context the CPU renderer draws: ${cpuLit} lit pixels`);
  const afterLoss = await clickDrawn(flaky, (box, x, y) => flaky.mouse.click(box.x + x + 2, box.y + y + 2));
  check(afterLoss.startsWith("Hull "), `after a lost context a click still selects: ${afterLoss.split("\n")[0]}`);

  // A phone: the theater fills most of the screen, nothing scrolls sideways,
  // and a tap on a drawn hull selects it.
  const phone = await browser.newContext({ ...playwright.devices["Pixel 7"] });
  const mobile = await phone.newPage();
  mobile.on("pageerror", (e) => errors.push(String(e)));
  await mobile.goto(`${base}/?replay=expansion`);
  await mobile.waitForFunction(() => window.hyades?.loaded() === "expansion" && document.getElementById("message").hidden, null, { timeout: 30000 });
  const geo = await mobile.evaluate(() => ({
    scroll: document.documentElement.scrollWidth, width: innerWidth, height: innerHeight,
    canvas: document.getElementById("view").clientHeight,
  }));
  check(geo.scroll <= geo.width, `no sideways scroll on a phone: ${geo.scroll} ≤ ${geo.width}`);
  check(geo.canvas >= 0.6 * geo.height, `the theater is at least 60% of a phone's height: ${geo.canvas} of ${geo.height}`);
  const tapped = await clickDrawn(mobile, (box, x, y) => mobile.touchscreen.tap(box.x + x + 6, box.y + y + 6));
  check(tapped.startsWith("Hull "), `a tap beside a drawn hull selects it: ${tapped.split("\n")[0]}`);
  check(await mobile.evaluate(() => !document.getElementById("badge").hidden), "a phone shows the selection over the theater");
  if (shots) await mobile.screenshot({ path: path.join(shots, "phone.png") });
  await phone.close();

  check(errors.length === 0, `no console errors${errors.length ? ": " + errors.join(" | ") : ""}`);
} catch (e) {
  check(false, `the test ran: ${e.message}`);
} finally {
  await browser.close();
  server.close();
}
process.exit(failures ? 1 : 0);
