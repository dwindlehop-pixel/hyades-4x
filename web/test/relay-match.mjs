// End-to-end test of the New game screen (docs/Hyades_sessions_discovery_and_security.md
// §5.1, §4.3.2): serves the site as Pages will, runs a mock Nostr relay on
// loopback that rate-limits writes per connection and sends nips#2498's
// backoff hint, and plays a match in three headless Chromium tabs — a host
// and two guests who open the host's link — through the real client.
// Checks that every tab resolves every round to the same orders, that the
// relay's rate limit was hit and waited out, that a diagnostics file can be
// saved and records the rate-limited replies, and that no tab logged a
// console error (a CSP violation is one).
//
// Usage: node web/test/relay-match.mjs <site dir>
// Needs Playwright and ws (npm) and a Chromium Playwright can launch.
import fs from "node:fs";
import http from "node:http";
import path from "node:path";
import { createRequire } from "node:module";
import { execSync } from "node:child_process";

const require = createRequire(import.meta.url);
const load = (name) => {
  for (const base of [process.cwd(), path.dirname(new URL(import.meta.url).pathname), execSync("npm root -g").toString().trim()]) {
    try {
      return require(require.resolve(name, { paths: [base, path.join(base, "node_modules")] }));
    } catch {}
  }
  throw new Error(`cannot find ${name}; npm install --no-save ${name}`);
};
const playwright = load("playwright");
const { WebSocketServer } = load("ws");

const site = path.resolve(process.argv[2]);
const failures = [];
const check = (ok, what) => {
  console.log(`${ok ? "ok  " : "FAIL"} ${what}`);
  if (!ok) failures.push(what);
};

// --- the site -----------------------------------------------------------------
const types = { ".html": "text/html", ".js": "text/javascript", ".css": "text/css", ".json": "application/json", ".wasm": "application/wasm" };
const server = http.createServer((req, res) => {
  const url = new URL(req.url, "http://x");
  const file = path.join(site, decodeURIComponent(url.pathname === "/" ? "/index.html" : url.pathname));
  if (!file.startsWith(site) || !fs.existsSync(file) || fs.statSync(file).isDirectory()) {
    res.writeHead(404).end();
    return;
  }
  res.writeHead(200, { "content-type": types[path.extname(file)] ?? "application/octet-stream" });
  fs.createReadStream(file).pipe(res);
});
await new Promise((r) => server.listen(0, "127.0.0.1", r));
const base = `http://127.0.0.1:${server.address().port}`;

// --- the mock relay ---------------------------------------------------------------
// NIP-01 over one websocket server: EVENT (stored, broadcast to matching
// subscriptions), REQ (stored matches, then EOSE). Writes are limited per
// connection by a token bucket; a rejection carries a hint in ms. A write
// that arrives before the hint it was given has run out is a violation.
const RATE = 1 / 2; // writes per second
const BURST = 4;
const HINT_MS = 2500;
const events = [];
let rateLimited = 0;
let violations = 0;
// NIP-11 over HTTP on the same port, with the CORS header NIP-11 requires.
const relayHttp = http.createServer((req, res) => {
  res.writeHead(200, { "content-type": "application/nostr+json", "access-control-allow-origin": "*", "access-control-allow-headers": "*" });
  res.end(req.method === "OPTIONS" ? "" : JSON.stringify({ name: "mock", software: "relay-match.mjs", limitation: { max_content_length: 65536 } }));
});
const wss = new WebSocketServer({ server: relayHttp });
await new Promise((r) => relayHttp.listen(0, "127.0.0.1", r));
const relayUrl = `ws://127.0.0.1:${relayHttp.address().port}`;
const topics = (e) => e.tags.filter((t) => t[0] === "t").map((t) => t[1]);
const conns = new Set();
wss.on("connection", (ws) => {
  const c = { ws, subs: new Map(), tokens: BURST, last: Date.now(), until: 0 };
  conns.add(c);
  ws.on("close", () => conns.delete(c));
  ws.on("message", (data) => {
    let m;
    try {
      m = JSON.parse(String(data));
    } catch {
      return;
    }
    if (m[0] === "REQ") {
      const want = new Set(m[2]["#t"] ?? []);
      c.subs.set(m[1], want);
      for (const e of events) if (topics(e).some((t) => want.has(t))) ws.send(JSON.stringify(["EVENT", m[1], e]));
      ws.send(JSON.stringify(["EOSE", m[1]]));
    } else if (m[0] === "EVENT") {
      const e = m[1];
      const now = Date.now();
      if (now < c.until) violations++;
      c.tokens = Math.min(BURST, c.tokens + ((now - c.last) / 1000) * RATE);
      c.last = now;
      if (c.tokens < 1) {
        rateLimited++;
        c.until = now + HINT_MS;
        ws.send(JSON.stringify(["OK", e.id, false, "rate-limited: slow down", HINT_MS]));
        return;
      }
      c.tokens -= 1;
      if (events.some((x) => x.id === e.id)) {
        ws.send(JSON.stringify(["OK", e.id, true, "duplicate: already have it"]));
        return;
      }
      events.push(e);
      ws.send(JSON.stringify(["OK", e.id, true, ""]));
      for (const other of conns) {
        for (const [sub, want] of other.subs) if (topics(e).some((t) => want.has(t))) other.ws.send(JSON.stringify(["EVENT", sub, e]));
      }
    }
  });
});

// --- three tabs ----------------------------------------------------------------------
const browser = await playwright.chromium.launch({ channel: process.env.HYADES_CHROME_CHANNEL || undefined });
const errors = [];
const pages = [];
async function tab(name) {
  const ctx = await browser.newContext({ acceptDownloads: true });
  const page = await ctx.newPage();
  pages.push([name, page]);
  page.on("console", (m) => { if (m.type() === "error") errors.push(`${name}: ${m.text()}`); });
  page.on("pageerror", (e) => errors.push(`${name}: ${e}`));
  return page;
}
const view = (page) => page.evaluate(() => document.getElementById("net-log").textContent);
const text = (page, id) => page.evaluate((id) => document.getElementById(id).textContent, id);

try {
  const host = await tab("host");
  await host.goto(`${base}/?relays=${encodeURIComponent(relayUrl)}`);
  check(!(await host.isDisabled("#menu-new")), "New game is enabled");
  // The menu's link drops the query, and with it the loopback relay; open the screen directly.
  await host.goto(`${base}/?view=new&relays=${encodeURIComponent(relayUrl)}`);
  await host.waitForFunction(() => document.getElementById("net-status").textContent.includes("Connected"), null, { timeout: 15000 });
  check(true, "the network module started and the relay connected");
  await host.fill("#f-name", "Host");
  await host.fill("#f-seats", "3");
  await host.fill("#f-round", "2");
  await host.fill("#f-rounds", "3");
  await host.fill("#f-patience", "8");
  await host.click("#create");
  await host.waitForFunction(() => document.querySelector("#room-head code"), null, { timeout: 10000 });
  const link = await host.evaluate(() => document.querySelector("#room-head code").textContent);
  check(/#j=[A-Za-z0-9_-]{119}$/.test(link), `the link is the site with a 119-character payload: ${link.length} chars`);
  check(!/127\.0\.0\.1:\d+.*relay|ws:/.test(link.split("#")[1]), "the link names no relay and no address");

  const guests = [];
  for (const name of ["Ann", "Bo"]) {
    const g = await tab(name);
    // The page is served from loopback, so it may name the loopback relay.
    await g.goto(`${base}/?relays=${encodeURIComponent(relayUrl)}${link.slice(link.indexOf("#"))}`);
    await g.fill("#f-name", name);
    await g.waitForSelector("#room-actions button", { timeout: 15000 });
    await g.click("#room-actions button");
    guests.push(g);
  }
  await host.waitForFunction(() => document.querySelectorAll("#room-seats li").length === 3, null, { timeout: 20000 });
  check(true, "both guests took a seat");
  await host.click("text=Start the match");
  for (const p of [host, ...guests]) {
    await p.waitForSelector("text=Accept the parameters and sign", { timeout: 20000 });
    await p.click("text=Accept the parameters and sign");
  }
  for (const p of [host, ...guests]) {
    await p.waitForFunction(() => document.getElementById("match-head").textContent.includes("done"), null, { timeout: 120000 });
  }
  check(true, "every tab finished the match");
  // Let the final checkpoints arrive.
  await host.waitForFunction(() => [...document.querySelectorAll("#match-history tr")].slice(1).every((r) => r.lastChild.textContent === "yes"), null, { timeout: 60000 }).catch(() => {});
  const histories = await Promise.all([host, ...guests].map((p) => p.evaluate(() =>
    [...document.querySelectorAll("#match-history tr")].slice(1).map((r) => r.children[1].textContent))));
  check(histories[0].length === 3, `three rounds resolved: ${JSON.stringify(histories[0])}`);
  check(histories.every((h) => JSON.stringify(h) === JSON.stringify(histories[0])), "every tab resolved the same orders");
  const timedOut = await host.evaluate(() => [...document.querySelectorAll("#match-history tr")].slice(1).map((r) => r.children[2].textContent));
  check(timedOut.every((t) => t === ""), `no seat was timed out: the limits were waited out within patience (${JSON.stringify(timedOut)})`);
  const agree = await host.evaluate(() => [...document.querySelectorAll("#match-history tr")].slice(1).map((r) => r.lastChild.textContent));
  check(agree.every((a) => a === "yes"), `every seat signed the same checkpoints: ${agree}`);
  check(rateLimited > 0, `the relay rate-limited ${rateLimited} writes`);
  check(violations === 0, `no tab wrote before its hint ran out (${violations} violations)`);

  const [download] = await Promise.all([host.waitForEvent("download"), host.click("#export-diag")]);
  const tsv = fs.readFileSync(await download.path(), "utf8");
  check(tsv.startsWith("t_s\trelay\tevent\tdetail"), "the diagnostics file saves as TSV");
  const limitedRows = tsv.split("\n").filter((l) => l.split("\t")[2] === "rate-limited").length;
  check(limitedRows > 0, `the diagnostics record rate-limited replies (${limitedRows} for the host)`);
  check(/hint 2500/.test(tsv), "a hint is recorded as the relay sent it");
  const [rec] = await Promise.all([guests[0].waitForEvent("download"), guests[0].click("#export-record")]);
  const record = JSON.parse(fs.readFileSync(await rec.path(), "utf8"));
  check(record.format === "hyades-transcript" && record.frames.length >= 3 * 3 * 3, `the match record holds the frames (${record.frames.length})`);
  check(errors.length === 0, `no console errors${errors.length ? ": " + errors.join(" | ") : ""}`);
  void view;
  void text;
} catch (e) {
  failures.push(String(e));
  console.log(`FAIL ${e}`);
  for (const [name, p] of pages) {
    const state = await p.evaluate(() => [document.getElementById("net-status").textContent, document.getElementById("net-log").textContent]).catch(() => ["?", "?"]);
    console.log(`--- ${name}: ${state[0]}\n${state[1].split("\n").slice(0, 40).join("\n")}`);
  }
  console.log(`errors: ${errors.join(" | ")}`);
} finally {
  await browser.close();
  server.close();
  wss.close();
  relayHttp.close();
}
if (failures.length) {
  console.log(`${failures.length} failure(s)`);
  process.exit(1);
}
console.log("relay match: all checks passed");
