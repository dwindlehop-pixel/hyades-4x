// The web client (docs/Hyades_interface.md §8): loads hyades_viewer.wasm and
// shows one screen at a time — the menu, the replay list, the palette sheet,
// and the replay viewer. In the viewer it hands the module a replay, forwards
// input, and puts what the module draws and writes on the page. Everything
// shown as text is formatted by the module; the client lays it out. Juicy
// mode is drawn on the GPU from the module's light list when the browser runs
// WebGL2 (gpu.js), and by the module otherwise.
import { createGpu } from "./gpu.js";

const $ = (id) => document.getElementById(id);
const stage = $("stage");
let canvas = $("view");
const decoder = new TextDecoder();
const encoder = new TextEncoder();

let hv = null;          // the module's exports
let gpu = null;         // the WebGL2 renderer, or null
let ctx2d = null;       // the 2D fallback
let index = [];         // the recorded replays: [{name, label, file}]
let loaded = null;      // the name of the replay in the module, or null
let dirty = true;       // something on screen is out of date
let baseRate = 1;       // the replay's opening rate, years per second
let direction = 1;      // +1 forward, −1 backward
let last = performance.now();
let lastLog = 0;
let lastTime = NaN;

const phone = window.matchMedia("(max-width: 760px)");
const bytes = (ptr, len) => new Uint8Array(hv.memory.buffer, ptr, len);
const text = (which, first = 0, rows = 0) => {
  const p = hv.hv_text(which, first, rows);
  return decoder.decode(bytes(p, hv.hv_text_len()).slice());
};
const put = (data) => {
  const p = hv.hv_input(data.length);
  bytes(p, data.length).set(data);
};

function showMessage(msg, retry) {
  $("message").hidden = !msg;
  $("message").replaceChildren(msg ?? "");
  if (retry) {
    const b = Object.assign(document.createElement("button"), { textContent: "Try again" });
    b.addEventListener("click", retry);
    $("message").append(b);
  }
}

// --- screens and the link ----------------------------------------------------
//
// The link names the screen: ?view=replays, ?view=palette, ?replay=<name> for
// the viewer, nothing for the menu. The palette rides in the hash on every
// screen, so a tuned palette survives moving between them.

function route() {
  const params = new URLSearchParams(location.search);
  const replay = params.get("replay");
  if (replay) return { screen: "viewer", replay };
  // A room link (docs/Hyades_sessions_discovery_and_security.md §5.1) is
  // #j=<payload>: it opens the New game screen on that room.
  const link = new URLSearchParams(location.hash.slice(1)).get("j");
  if (link) return { screen: "new", link };
  const view = params.get("view");
  return { screen: ["replays", "palette", "new"].includes(view) ? view : "menu" };
}

function go(search) {
  const url = new URL(location);
  url.search = search;
  history.pushState(null, "", url);
  show(route());
}

function show({ screen, replay, link }) {
  document.body.dataset.screen = screen;
  // The networking module loads only when this screen opens, so a replay
  // viewer never fetches it.
  if (screen === "new") import("./newgame.js").then((m) => m.enterNew(link));
  if (screen === "palette") buildSheet();
  if (screen === "replays") buildReplayList();
  if (screen === "viewer") {
    if (!phone.matches) document.body.classList.add("side-open", "log-open");
    if (replay && replay !== loaded) openRecorded(replay);
    dirty = true;
  }
}

window.addEventListener("popstate", () => show(route()));
document.querySelectorAll(".to-menu").forEach((b) => b.addEventListener("click", () => go("")));
$("menu-new").addEventListener("click", () => go("?view=new"));
$("menu-replays").addEventListener("click", () => go("?view=replays"));
$("menu-palette").addEventListener("click", () => go("?view=palette"));

function buildReplayList() {
  $("replay-list").replaceChildren(...index.map((r) => {
    const b = document.createElement("button");
    const sub = document.createElement("span");
    sub.textContent = r.label;
    b.append(r.name, sub);
    b.dataset.replay = r.name;
    b.addEventListener("click", () => go(`?replay=${encodeURIComponent(r.name)}`));
    const li = document.createElement("li");
    li.append(b);
    return li;
  }));
  $("replays-message").textContent = index.length ? "" : "No recorded replays here. Open a replay file.";
}

// --- the palette sheet -------------------------------------------------------

function buildSheet() {
  if (!hv) return;
  const [head, ...lines] = text(7).trim().split("\n");
  const [status, settings] = head.split("\t");
  const code = document.createElement("code");
  code.textContent = settings;
  $("palette-status").replaceChildren(
    `Status: ${status.toUpperCase()} — to be ratified on a replay (R-UI1), where it can be tuned.`,
    document.createElement("br"), "Settings: ", code);
  for (const id of ["source", "role", "seat", "status"]) $(`chips-${id}`).replaceChildren();
  for (const line of lines) {
    const [section, name, src, shown] = line.split("\t");
    const chip = document.createElement("div");
    chip.className = "chip";
    const sw = document.createElement("div");
    sw.className = "sw";
    for (const c of src ? [src, shown] : [shown]) {
      const i = document.createElement("i");
      i.style.background = c;
      sw.append(i);
    }
    const label = document.createElement("p");
    const sub = document.createElement("span");
    sub.textContent = `${src ? src + " → " : ""}${shown}`;
    label.append(name, document.createElement("br"), sub);
    chip.append(sw, label);
    $(`chips-${section}`)?.append(chip);
  }
}

// --- the palette drives the chrome ---------------------------------------

function applyPalette() {
  const vars = {
    ground: "--ground", panel: "--panel", grid: "--grid",
    text_dim: "--text-dim", text: "--text", text_bright: "--text-bright",
    Selected: "--selected", Combat: "--combat", hy_cyan: "--mat-cyan", hy_yellow: "--mat-yellow",
  };
  for (const line of text(7).split("\n").slice(1)) {
    const [, name, , shown] = line.split("\t");
    if (vars[name] && shown) document.documentElement.style.setProperty(vars[name], shown);
  }
  refreshLegend();
  buildRoleLegend();
  buildMaterialLegend();
  if (document.body.dataset.screen === "palette") buildSheet();
}

/// Each role's mark as the module stamps it (`hv_text(11)`), in its accent.
function buildRoleLegend() {
  $("roles").replaceChildren(...text(11).split("\n").filter(Boolean).map((line) => {
    const [role, bits, color] = line.split("\t");
    const mark = document.createElement("span");
    mark.className = "mark";
    for (const b of bits) {
      const px = document.createElement("i");
      if (b === "1") px.style.background = color;
      mark.append(px);
    }
    const li = document.createElement("li");
    li.dataset.role = role;
    li.append(mark, role.toLowerCase());
    return li;
  }));
}

/// Each cargo stripe's color under the material's in-game name (`hv_text(12)`).
function buildMaterialLegend() {
  $("materials").replaceChildren(...text(12).split("\n").filter(Boolean).map((line) => {
    const [name, color] = line.split("\t");
    const swatch = document.createElement("span");
    swatch.className = "swatch";
    swatch.style.background = color;
    const li = document.createElement("li");
    li.dataset.material = name;
    li.append(swatch, name);
    return li;
  }));
}

// --- the live palette editor (docs/Hyades_interface.md §6.2) ---------------
//
// The module owns the palette: the editor sends it the settings line
// (`ink=… paper=… … hy_red=#c83a2c`), and reads back the canonical line and
// the sheet of resulting colors. The line also rides in the page's link, so a
// tuned palette can be sent, reopened and ratified as it was seen.

const SLIDERS = [
  ["ink", "ink", 0, 0.4, 0.005],
  ["paper", "paper", 0.6, 1, 0.005],
  ["warm", "warm chroma", 0, 1.5, 0.01],
  ["cool", "cool chroma", 0, 1.5, 0.01],
  ["pull", "hue pull", 0, 1, 0.01],
  ["anchor0", "anchor 1 °", 0, 360, 1],
  ["anchor1", "anchor 2 °", 0, 360, 1],
  ["anchor2", "anchor 3 °", 0, 360, 1],
  ["anchor3", "anchor 4 °", 0, 360, 1],
  ["fill", "glyph fill dim", 0, 0.9, 0.01],
];

let tune = null;  // {ink, paper, warm, cool, pull, anchor0..3, fill, overrides: {name: hex}}

function readSettings(line) {
  const t = { overrides: {} };
  for (const pair of line.split(/[\s&]+/).filter(Boolean)) {
    const [k, v] = pair.split("=");
    if (k === "anchors") v.split(",").forEach((a, i) => (t[`anchor${i}`] = Number(a)));
    else if (v.startsWith("#")) t.overrides[k] = v;
    else t[k] = Number(v);
  }
  return t;
}

function settingsLine(t) {
  const base = `ink=${t.ink} paper=${t.paper} warm=${t.warm} cool=${t.cool} pull=${t.pull} ` +
    `anchors=${t.anchor0},${t.anchor1},${t.anchor2},${t.anchor3} fill=${t.fill}`;
  return [base, ...Object.entries(t.overrides).map(([k, v]) => `${k}=${v}`)].join(" ");
}

/// Sends `line` to the module. On success the editor, the link and the
/// chrome follow the module's canonical line; on failure nothing changes.
function setPalette(line) {
  put(encoder.encode(line));
  const bad = hv.hv_palette_set() !== 0;
  $("tune-error").hidden = !bad;
  if (bad) {
    $("tune-error").textContent = text(3);
    return false;
  }
  const canonical = text(9);
  tune = readSettings(canonical);
  $("tune-text").value = canonical;
  // Keep the hash's other keys: a room link rides there too (#j=, newgame.js).
  const url = new URL(location);
  const others = [...new URLSearchParams(url.hash.slice(1))].filter(([k]) => k !== "palette");
  url.hash = [`palette=${encodeURIComponent(canonical)}`, ...others.map(([k, v]) => `${k}=${encodeURIComponent(v)}`)].join("&");
  history.replaceState(null, "", url);
  applyPalette();
  buildTuner();
  dirty = true;
  return true;
}

function buildTuner() {
  for (const [key, label, min, max, step] of SLIDERS) {
    let input = document.getElementById(`tune-${key}`);
    if (!input) {
      const row = document.createElement("label");
      row.className = "slider";
      input = Object.assign(document.createElement("input"), { type: "range", id: `tune-${key}`, min, max, step });
      input.addEventListener("input", () => {
        tune[key] = Number(input.value);
        setPalette(settingsLine(tune));
      });
      row.append(label, input, document.createElement("output"));
      $("tune-sliders").append(row);
    }
    if (document.activeElement !== input) input.value = tune[key];
    input.parentElement.querySelector("output").textContent = tune[key];
  }
  const lines = text(7).split("\n").slice(1).map((l) => l.split("\t"));
  const box = $("tune-colors");
  for (const [sec, name, , shown] of lines) {
    if (sec !== "source" && sec !== "status") continue;
    let row = box.querySelector(`label[data-name="${name}"]`);
    if (!row) {
      row = document.createElement("label");
      row.dataset.name = name;
      const input = Object.assign(document.createElement("input"), { type: "color" });
      input.addEventListener("input", () => {
        tune.overrides[name] = input.value;
        setPalette(settingsLine(tune));
      });
      const clear = Object.assign(document.createElement("button"), { textContent: "×", title: "Back to the tone map" });
      clear.addEventListener("click", (e) => {
        e.preventDefault();
        delete tune.overrides[name];
        setPalette(settingsLine(tune));
      });
      row.append(input, name.replace(/^hy_/, ""), clear);
      box.append(row);
    }
    const set = name in tune.overrides;
    row.className = "swatch" + (set ? " set" : "");
    row.title = set ? `${name}, set by hand` : `${name}, from the tone map`;
    row.querySelector("button").hidden = !set;
    const input = row.querySelector("input");
    if (document.activeElement !== input) input.value = shown;
  }
}

function refreshLegend() {
  if (!loaded) return;
  const seats = text(4).split("\n").filter(Boolean).map((l) => l.split("\t"));
  $("legend").querySelectorAll("i").forEach((sw, i) => { if (seats[i]) sw.style.background = seats[i][1]; });
  document.querySelectorAll('#seats label').forEach((l, i) => { if (seats[i]) l.style.color = seats[i][1]; });
}

async function copy(textValue, button) {
  try {
    await navigator.clipboard.writeText(textValue);
    button.textContent = "Copied";
  } catch {
    $("tune-text").select();
    button.textContent = "Selected — copy it";
  }
  setTimeout(() => { button.textContent = button.dataset.label; }, 1500);
}

for (const id of ["tune-copy", "tune-link"]) $(id).dataset.label = $(id).textContent;
$("tune-copy").addEventListener("click", () => copy($("tune-text").value, $("tune-copy")));
$("tune-link").addEventListener("click", () => copy(location.href, $("tune-link")));
$("tune-reset").addEventListener("click", () => { if (hv) setPalette(""); });
$("tune-text").addEventListener("change", () => { if (hv && !setPalette($("tune-text").value)) $("tune-text").focus(); });

// --- loading --------------------------------------------------------------

function cssSize() {
  return [Math.max(1, canvas.clientWidth), Math.max(1, canvas.clientHeight)];
}

/// Hands the module a replay. The viewer screen is shown first, so the
/// camera fits the canvas's laid-out size and not a hidden one's.
function load(data, label, name) {
  document.body.dataset.screen = "viewer";
  put(data);
  const [w, h] = cssSize();
  if (hv.hv_load(w, h) !== 0) {
    loaded = null;
    showMessage(`Could not read ${label}: ${text(3)}`);
    return;
  }
  loaded = name;
  showMessage(null);
  baseRate = hv.hv_rate();
  direction = 1;
  setRate();
  buildFilters();
  $("label").textContent = text(8);
  dirty = true;
  updateLog();
}

/// Fetches `url`, trying again on a server error or a dropped connection:
/// Pages answers 503 for a moment while a deployment replaces the site.
async function fetchBytes(url, tries = 4) {
  let why = "";
  for (let k = 0; k < tries; k++) {
    if (k > 0) await new Promise((r) => setTimeout(r, 500 * 2 ** (k - 1)));
    try {
      const res = await fetch(url, { cache: k > 0 ? "reload" : "default" });
      if (res.ok) return new Uint8Array(await res.arrayBuffer());
      why = `the server answered ${res.status}${res.statusText ? " " + res.statusText : ""}`;
      if (res.status < 500) break;
    } catch (e) {
      why = e.message;
    }
  }
  throw new Error(`${why} (${tries} tries)`);
}

async function openRecorded(name) {
  const entry = index.find((r) => r.name === name);
  document.body.dataset.screen = "viewer";
  if (!entry) {
    showMessage(name === "file"
      ? "A replay opened from a file is not kept across a reload; open it again from Replays."
      : `No recorded replay is named ${name}.`);
    return;
  }
  loaded = name;
  showMessage(`Loading ${name}…`);
  try {
    load(await fetchBytes(`replays/${entry.file}`), name, name);
  } catch (e) {
    loaded = null;
    showMessage(`Could not load ${name}: ${e.message}.`, () => openRecorded(name));
  }
}

$("file").addEventListener("change", async (e) => {
  const f = e.target.files[0];
  if (!f) return;
  const url = new URL(location);
  url.search = "?replay=file";
  history.pushState(null, "", url);
  load(new Uint8Array(await f.arrayBuffer()), f.name, "file");
  e.target.value = "";
});

// --- filters --------------------------------------------------------------

function buildFilters() {
  const cats = text(5).split("\n").filter(Boolean);
  $("categories").replaceChildren(...cats.map((c, i) => checkbox(`cat${i}`, c, true)));
  const seats = text(4).split("\n").filter(Boolean).map((l) => l.split("\t"));
  $("seats").replaceChildren(...seats.map(([name, color], i) => checkbox(`seat${i}`, name, true, color)));
  const legend = seats.map(([name, color, arch]) => {
    const li = document.createElement("li");
    const sw = document.createElement("i");
    sw.style.background = color;
    li.append(sw, `${name} · ${arch} archetype`);
    return li;
  });
  $("legend").replaceChildren(...legend);
  const kinds = text(6).split("\n").filter(Boolean);
  $("kind").replaceChildren(new Option("every kind", ""), ...kinds.map((k) => new Option(k, k)));
  applyFilters();
}

function checkbox(id, label, checked, color) {
  const l = document.createElement("label");
  const c = document.createElement("input");
  c.type = "checkbox";
  c.id = id;
  c.checked = checked;
  c.addEventListener("change", applyFilters);
  l.append(c, label);
  if (color) l.style.color = color;
  return l;
}

function mask(prefix) {
  let m = 0;
  document.querySelectorAll(`input[id^="${prefix}"]`).forEach((c) => {
    if (c.checked) m |= 1 << Number(c.id.slice(prefix.length));
  });
  return m >>> 0;
}

function applyFilters() {
  if (!loaded) return;
  hv.hv_filter_categories(mask("cat"));
  hv.hv_filter_seats(mask("seat"), $("unseated").checked ? 1 : 0);
  put(encoder.encode($("text").value));
  hv.hv_filter_text();
  put(encoder.encode($("kind").value));
  hv.hv_filter_kind();
  hv.hv_filter_window(Number($("window").value), hv.hv_frame_years());
  updateLog();
}

// --- the log, a virtual list ------------------------------------------------

const ROW = 18;
const logBox = $("log");
const spacer = document.createElement("div");
spacer.style.position = "relative";
logBox.append(spacer);

function updateLog() {
  if (!loaded || logBox.clientHeight === 0) return;
  const rows = Math.ceil(logBox.clientHeight / ROW) + 2;
  const follow = $("followlog").checked;
  const first = follow ? -1 : Math.floor(logBox.scrollTop / ROW);
  const out = text(2, first, rows).split("\n");
  const [count, current, start] = out[0].split("\t");
  $("logcount").textContent = `${count} events`;
  spacer.style.height = `${Number(count) * ROW}px`;
  const lines = out.slice(1).filter(Boolean).map((line) => {
    const [row, t, cat, seat, kind, msg] = line.split("\t");
    const div = document.createElement("div");
    div.className = `row cat-${cat}` + (row === current ? " now" : "");
    div.style.position = "absolute";
    div.style.top = `${Number(row) * ROW}px`;
    div.style.left = "0";
    div.style.right = "0";
    div.style.height = `${ROW}px`;
    div.dataset.row = row;
    for (const v of [t, cat, seat, kind, msg]) {
      const s = document.createElement("span");
      s.textContent = v;
      div.append(s);
    }
    return div;
  });
  spacer.replaceChildren(...lines);
  if (follow) logBox.scrollTop = Math.max(0, Number(start) * ROW + rows * ROW - logBox.clientHeight - 2 * ROW);
}

logBox.addEventListener("click", (e) => {
  const row = e.target.closest(".row");
  if (!row || !loaded) return;
  hv.hv_log_seek(Number(row.dataset.row));
  dirty = true;
});
logBox.addEventListener("wheel", () => { $("followlog").checked = false; }, { passive: true });
logBox.addEventListener("touchstart", () => { $("followlog").checked = false; }, { passive: true });
logBox.addEventListener("scroll", () => { if (!$("followlog").checked) updateLog(); });

// --- panels ------------------------------------------------------------------
//
// On a wide screen the panel and the log stand beside and below the theater
// and the tabs hide them. On a phone they open as sheets over the bottom of
// the theater, one at a time.

function toggle(which) {
  const cls = `${which}-open`;
  const open = !document.body.classList.contains(cls);
  if (phone.matches) document.body.classList.remove("side-open", "log-open");
  document.body.classList.toggle(cls, open);
  $("tab-side").classList.toggle("on", document.body.classList.contains("side-open"));
  $("tab-log").classList.toggle("on", document.body.classList.contains("log-open"));
  updateLog();
  dirty = true;
}
$("tab-side").addEventListener("click", () => toggle("side"));
$("tab-log").addEventListener("click", () => toggle("log"));
$("badge").addEventListener("click", () => { if (!document.body.classList.contains("side-open")) toggle("side"); });

// --- drawing ---------------------------------------------------------------

function resize() {
  const dpr = window.devicePixelRatio || 1;
  const [w, h] = cssSize();
  const bw = Math.round(w * dpr), bh = Math.round(h * dpr);
  if (canvas.width !== bw || canvas.height !== bh) {
    canvas.width = bw;
    canvas.height = bh;
    hv.hv_resize(w, h);
    dirty = true;
  }
}

/// Moves drawing to the module's CPU renderer for the rest of the session. A
/// canvas that has held a WebGL context cannot give a 2D one, so it is
/// replaced by a fresh one.
function fallBackToCpu(why) {
  if (!gpu) return;
  console.warn(`WebGL2 renderer stopped (${why}); drawing on the CPU.`);
  gpu = null;
  const fresh = canvas.cloneNode(false);
  canvas.replaceWith(fresh);
  canvas = fresh;
  ctx2d = canvas.getContext("2d");
  document.body.dataset.renderer = "cpu";
  dirty = true;
}

const offscreen = document.createElement("canvas");

function draw() {
  const juicy = hv.hv_mode() === 1;
  const [w, h] = cssSize();
  if (gpu) {
    try {
      if (juicy) {
        hv.hv_prepare_lights();
        const scene = new Float32Array(hv.memory.buffer, hv.hv_lights_ptr(0), hv.hv_lights_len(0) * 6);
        const terr = new Float32Array(hv.memory.buffer, hv.hv_lights_ptr(1), hv.hv_lights_len(1) * 6);
        gpu.juicy(scene, hv.hv_lights_len(0), terr, hv.hv_lights_len(1), w, h);
      } else {
        const ptr = hv.hv_render();
        gpu.frame(bytes(ptr, hv.hv_frame_w() * hv.hv_frame_h() * 4), hv.hv_frame_w(), hv.hv_frame_h(), gpu.NEAREST);
      }
      if (!gpu.gl.isContextLost()) return;
      fallBackToCpu("context lost");
    } catch (e) {
      fallBackToCpu(e.message);
    }
    resize();
  }
  const ptr = hv.hv_render();
  const fw = hv.hv_frame_w(), fh = hv.hv_frame_h();
  offscreen.width = fw;
  offscreen.height = fh;
  offscreen.getContext("2d").putImageData(new ImageData(new Uint8ClampedArray(bytes(ptr, fw * fh * 4)), fw, fh), 0, 0);
  ctx2d.imageSmoothingEnabled = juicy;
  ctx2d.drawImage(offscreen, 0, 0, canvas.width, canvas.height);
}

function frame(now) {
  const dt = Math.min(0.25, (now - last) / 1000);
  last = now;
  if (loaded && document.body.dataset.screen === "viewer") {
    resize();
    if (hv.hv_playing()) {
      hv.hv_tick(dt);
      dirty = true;
    }
    if (dirty) {
      draw();
      dirty = false;
      $("status").textContent = text(0);
      const inspector = text(1);
      $("inspector").textContent = inspector;
      const selected = !inspector.startsWith("Nothing selected");
      $("badge").hidden = !selected || document.body.classList.contains("side-open");
      if (selected) $("badge").textContent = inspector.split("\n").slice(0, 3).join("\n");
      $("scrub").value = String(Math.round(hv.hv_fraction() * 10000));
      const playing = hv.hv_playing();
      $("play").textContent = playing && direction > 0 ? "❚❚" : "▶";
      $("rewind").textContent = playing && direction < 0 ? "❚❚" : "◀";
      const t = hv.hv_time();
      if (t !== lastTime && now - lastLog > 100) {
        lastTime = t;
        lastLog = now;
        updateLog();
      }
    }
  }
  requestAnimationFrame(frame);
}

// --- input -------------------------------------------------------------------

function setRate() {
  hv.hv_set_rate(direction * baseRate * Number($("rate").value));
}

/// Plays in `dir` (+1 forward, −1 backward). Already playing that way, it
/// pauses; playing the other way or paused, it turns and plays — so ▶ always
/// leads forward and ◀ backward.
function playToward(dir) {
  if (direction === dir && hv.hv_playing()) {
    hv.hv_play(0);
    return;
  }
  direction = dir;
  setRate();
  hv.hv_play(1);
}

function setMode(m) {
  hv.hv_set_mode(m);
  $("mode-tactical").classList.toggle("on", m === 0);
  $("mode-juicy").classList.toggle("on", m === 1);
  dirty = true;
}

function act(fn) {
  return () => { if (loaded) { fn(); dirty = true; } };
}

$("play").addEventListener("click", act(() => playToward(1)));
$("rewind").addEventListener("click", act(() => playToward(-1)));
$("back").addEventListener("click", act(() => hv.hv_step(-1)));
$("step").addEventListener("click", act(() => hv.hv_step(1)));
$("rate").addEventListener("change", act(setRate));
$("scrub").addEventListener("input", act(() => hv.hv_seek_fraction(Number($("scrub").value) / 10000)));
$("mode-tactical").addEventListener("click", act(() => setMode(0)));
$("mode-juicy").addEventListener("click", act(() => setMode(1)));
$("fit").addEventListener("click", act(() => hv.hv_fit()));
$("focus").addEventListener("click", act(() => hv.hv_focus()));
$("follow").addEventListener("click", act(() => hv.hv_follow()));
for (const id of ["unseated", "kind", "window"]) $(id).addEventListener("change", applyFilters);
$("text").addEventListener("input", applyFilters);
$("followlog").addEventListener("change", () => updateLog());

// Pointers: one drags the view, two pinch it, and a press that does not move
// past TAP_SLOP picks. A finger covers more of the screen than a mouse
// cursor, so it picks over a wider radius (§8.3).
const TAP_SLOP = { mouse: 4, pen: 8, touch: 10 };
const PICK_RADIUS = { mouse: 10, pen: 14, touch: 24 };
const pointers = new Map();  // pointerId → {x, y, type, moved}

function at(e) {
  const r = canvas.getBoundingClientRect();
  return [e.clientX - r.left, e.clientY - r.top];
}

function pinch() {
  const [a, b] = [...pointers.values()];
  return { x: (a.x + b.x) / 2, y: (a.y + b.y) / 2, d: Math.hypot(a.x - b.x, a.y - b.y) };
}

stage.addEventListener("pointerdown", (e) => {
  if (e.target !== canvas || !loaded) return;
  canvas.setPointerCapture(e.pointerId);
  const [x, y] = at(e);
  pointers.set(e.pointerId, { x, y, type: e.pointerType, moved: 0 });
  // A second finger makes it a pinch, and neither finger picks.
  if (pointers.size > 1) for (const p of pointers.values()) p.moved = Infinity;
});
stage.addEventListener("pointermove", (e) => {
  const p = pointers.get(e.pointerId);
  if (!p) return;
  const [x, y] = at(e);
  if (pointers.size === 2) {
    const before = pinch();
    Object.assign(p, { x, y });
    const after = pinch();
    hv.hv_pan(after.x - before.x, after.y - before.y);
    if (before.d > 0 && after.d > 0) hv.hv_zoom(after.x, after.y, after.d / before.d);
    dirty = true;
    return;
  }
  const dx = x - p.x, dy = y - p.y;
  p.moved += Math.abs(dx) + Math.abs(dy);
  if (p.moved > (TAP_SLOP[p.type] ?? 4)) {
    hv.hv_pan(dx, dy);
    dirty = true;
  }
  Object.assign(p, { x, y });
});
function release(e) {
  const p = pointers.get(e.pointerId);
  if (!p) return;
  pointers.delete(e.pointerId);
  if (e.type === "pointerup" && p.moved <= (TAP_SLOP[p.type] ?? 4)) {
    hv.hv_pick(p.x, p.y, PICK_RADIUS[p.type] ?? 10);
    dirty = true;
  }
}
stage.addEventListener("pointerup", release);
stage.addEventListener("pointercancel", release);
stage.addEventListener("wheel", (e) => {
  if (e.target !== canvas) return;
  e.preventDefault();
  if (!loaded) return;
  const [x, y] = at(e);
  hv.hv_zoom(x, y, Math.exp(-e.deltaY * 0.0015));
  dirty = true;
}, { passive: false });

window.addEventListener("keydown", (e) => {
  if (document.body.dataset.screen !== "viewer" || e.target.closest("input, select, textarea, button, summary")) return;
  if (e.key === "Escape") {
    go("");
    return;
  }
  if (!loaded) return;
  const keys = {
    " ": () => (hv.hv_playing() ? hv.hv_play(0) : playToward(1)),
    k: () => hv.hv_play(0),
    l: () => { direction = 1; setRate(); hv.hv_play(1); },
    j: () => { direction = -1; setRate(); hv.hv_play(1); },
    ArrowLeft: () => hv.hv_step(-1),
    ArrowRight: () => hv.hv_step(1),
    t: () => setMode(hv.hv_mode() === 1 ? 0 : 1),
    f: () => hv.hv_fit(),
    g: () => hv.hv_focus(),
    c: () => hv.hv_follow(),
    "+": () => hv.hv_zoom(canvas.clientWidth / 2, canvas.clientHeight / 2, 1.25),
    "-": () => hv.hv_zoom(canvas.clientWidth / 2, canvas.clientHeight / 2, 0.8),
  };
  const fn = keys[e.key];
  if (fn) {
    e.preventDefault();
    fn();
    dirty = true;
  }
});

// --- start -------------------------------------------------------------------

async function start() {
  const { instance } = await WebAssembly.instantiate(await fetchBytes("hyades_viewer.wasm"), {});
  hv = instance.exports;
  const fromLink = new URLSearchParams(location.hash.slice(1)).get("palette");
  if (!(fromLink && setPalette(fromLink))) setPalette(text(9));
  const constants = {
    downsample: hv.hv_juicy_constant(0), weight: hv.hv_juicy_constant(1), exposure: hv.hv_juicy_constant(2),
    radius: hv.hv_juicy_constant(3), passes: hv.hv_juicy_constant(4),
  };
  gpu = new URLSearchParams(location.search).get("gpu") === "0" ? null : createGpu(canvas, constants);
  if (!gpu) ctx2d = canvas.getContext("2d");
  else canvas.addEventListener("webglcontextlost", (e) => { e.preventDefault(); fallBackToCpu("context lost"); });
  document.body.dataset.renderer = gpu ? "webgl2" : "cpu";
  window.hyades = { hv: () => hv, gpu: () => gpu, draw: () => draw(), text, loaded: () => loaded };
  try {
    index = JSON.parse(decoder.decode(await fetchBytes("replays/index.json")));
  } catch {
    index = [];
  }
  phone.addEventListener("change", () => {
    document.body.classList.toggle("side-open", !phone.matches);
    document.body.classList.toggle("log-open", !phone.matches);
  });
  show(route());
  requestAnimationFrame(frame);
}

start().catch((e) => {
  document.body.dataset.screen = "viewer";
  showMessage(`The client could not start: ${e.message}`, () => location.reload());
});
