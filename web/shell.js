// The web shell (docs/Hyades_interface.md §8): loads hyades_viewer.wasm,
// hands it a replay, forwards input, and puts what it draws and writes on the
// page. Everything shown as text is formatted by the module; the shell lays
// it out. Juicy mode is drawn on the GPU from the module's light list when
// the browser runs WebGL2 (gpu.js), and by the module otherwise.
import { createGpu } from "./gpu.js";

const $ = (id) => document.getElementById(id);
const canvas = $("view");
const decoder = new TextDecoder();

let hv = null;          // the module's exports
let gpu = null;         // the WebGL2 renderer, or null
let ctx2d = null;       // the 2D fallback
let dirty = true;       // something on screen is out of date
let baseRate = 1;       // the replay's opening rate, years per second
let direction = 1;      // +1 forward, −1 backward
let last = performance.now();
let lastLog = 0;
let lastTime = NaN;

const bytes = (ptr, len) => new Uint8Array(hv.memory.buffer, ptr, len);
const text = (which, first = 0, rows = 0) => {
  const p = hv.hv_text(which, first, rows);
  return decoder.decode(bytes(p, hv.hv_text_len()).slice());
};
const put = (data) => {
  const p = hv.hv_input(data.length);
  bytes(p, data.length).set(data);
};

function showMessage(msg) {
  $("message").hidden = !msg;
  $("message").textContent = msg ?? "";
}

// --- the palette drives the chrome ---------------------------------------

function applyPalette() {
  const vars = {
    ground: "--ground", panel: "--panel", grid: "--grid",
    text_dim: "--text-dim", text: "--text", text_bright: "--text-bright",
    Selected: "--selected", Combat: "--combat",
  };
  for (const line of text(7).split("\n").slice(1)) {
    const [, name, , shown] = line.split("\t");
    if (vars[name] && shown) document.documentElement.style.setProperty(vars[name], shown);
  }
}

// --- loading --------------------------------------------------------------

function cssSize() {
  return [Math.max(1, canvas.clientWidth), Math.max(1, canvas.clientHeight)];
}

function load(data, label) {
  put(data);
  const [w, h] = cssSize();
  if (hv.hv_load(w, h) !== 0) {
    showMessage(`Could not read ${label}: ${text(3)}`);
    return;
  }
  showMessage(null);
  baseRate = hv.hv_rate();
  direction = 1;
  setRate();
  buildFilters();
  $("label").textContent = text(8);
  dirty = true;
  updateLog(true);
}

async function loadUrl(url, label) {
  showMessage(`Loading ${label}…`);
  try {
    const res = await fetch(url);
    if (!res.ok) throw new Error(`${res.status} ${res.statusText}`);
    load(new Uint8Array(await res.arrayBuffer()), label);
  } catch (e) {
    showMessage(`Could not load ${label}: ${e.message}`);
  }
}

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
    li.append(sw, `${name} · ${arch}`);
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
  if (!hv) return;
  hv.hv_filter_categories(mask("cat"));
  hv.hv_filter_seats(mask("seat"), $("unseated").checked ? 1 : 0);
  put(new TextEncoder().encode($("text").value));
  hv.hv_filter_text();
  put(new TextEncoder().encode($("kind").value));
  hv.hv_filter_kind();
  hv.hv_filter_window(Number($("window").value), hv.hv_frame_years());
  updateLog(true);
}

// --- the log, a virtual list ------------------------------------------------

const ROW = 18;
const logBox = $("log");
const spacer = document.createElement("div");
spacer.style.position = "relative";
logBox.append(spacer);

function updateLog(force) {
  if (!hv) return;
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
  void force;
}

logBox.addEventListener("click", (e) => {
  const row = e.target.closest(".row");
  if (!row || !hv) return;
  hv.hv_log_seek(Number(row.dataset.row));
  dirty = true;
});
logBox.addEventListener("wheel", () => { $("followlog").checked = false; }, { passive: true });
logBox.addEventListener("scroll", () => { if (!$("followlog").checked) updateLog(true); });

// --- drawing ---------------------------------------------------------------

function resize() {
  const dpr = window.devicePixelRatio || 1;
  const [w, h] = cssSize();
  const bw = Math.round(w * dpr), bh = Math.round(h * dpr);
  if (canvas.width !== bw || canvas.height !== bh) {
    canvas.width = bw;
    canvas.height = bh;
    if (hv) hv.hv_resize(w, h);
    dirty = true;
  }
}

const offscreen = document.createElement("canvas");

function draw() {
  const juicy = hv.hv_mode() === 1;
  const [w, h] = cssSize();
  if (juicy && gpu) {
    hv.hv_prepare_lights();
    const scene = new Float32Array(hv.memory.buffer, hv.hv_lights_ptr(0), hv.hv_lights_len(0) * 6);
    const terr = new Float32Array(hv.memory.buffer, hv.hv_lights_ptr(1), hv.hv_lights_len(1) * 6);
    gpu.juicy(scene, hv.hv_lights_len(0), terr, hv.hv_lights_len(1), w, h);
    return;
  }
  const ptr = hv.hv_render();
  const fw = hv.hv_frame_w(), fh = hv.hv_frame_h();
  const px = bytes(ptr, fw * fh * 4);
  if (gpu) {
    gpu.frame(px, fw, fh, juicy ? gpu.LINEAR : gpu.NEAREST);
  } else {
    offscreen.width = fw;
    offscreen.height = fh;
    offscreen.getContext("2d").putImageData(new ImageData(new Uint8ClampedArray(px), fw, fh), 0, 0);
    ctx2d.imageSmoothingEnabled = juicy;
    ctx2d.drawImage(offscreen, 0, 0, canvas.width, canvas.height);
  }
}

function frame(now) {
  const dt = Math.min(0.25, (now - last) / 1000);
  last = now;
  if (hv) {
    resize();
    if (hv.hv_playing()) {
      hv.hv_tick(dt);
      dirty = true;
    }
    if (dirty) {
      draw();
      dirty = false;
      $("status").textContent = text(0);
      $("inspector").textContent = text(1);
      $("scrub").value = String(Math.round(hv.hv_fraction() * 10000));
      $("play").textContent = hv.hv_playing() ? "❚❚" : "▶";
      const t = hv.hv_time();
      if (t !== lastTime && now - lastLog > 100) {
        lastTime = t;
        lastLog = now;
        updateLog(false);
      }
    }
  }
  requestAnimationFrame(frame);
}

// --- input -------------------------------------------------------------------

function setRate() {
  hv.hv_set_rate(direction * baseRate * Number($("rate").value));
}

function setMode(m) {
  hv.hv_set_mode(m);
  $("mode-tactical").classList.toggle("on", m === 0);
  $("mode-juicy").classList.toggle("on", m === 1);
  dirty = true;
}

function act(fn) {
  return () => { if (hv) { fn(); dirty = true; } };
}

$("play").addEventListener("click", act(() => hv.hv_play(2)));
$("rewind").addEventListener("click", act(() => { direction = -1; setRate(); hv.hv_play(1); }));
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
$("followlog").addEventListener("change", () => updateLog(true));

let drag = null;
canvas.addEventListener("pointerdown", (e) => {
  canvas.setPointerCapture(e.pointerId);
  drag = { x: e.offsetX, y: e.offsetY, moved: 0 };
});
canvas.addEventListener("pointermove", (e) => {
  if (!drag || !hv) return;
  const dx = e.offsetX - drag.x, dy = e.offsetY - drag.y;
  drag.moved += Math.abs(dx) + Math.abs(dy);
  if (drag.moved > 3) {
    hv.hv_pan(dx, dy);
    dirty = true;
  }
  drag.x = e.offsetX;
  drag.y = e.offsetY;
});
canvas.addEventListener("pointerup", (e) => {
  if (drag && drag.moved <= 3 && hv) {
    hv.hv_pick(e.offsetX, e.offsetY);
    dirty = true;
  }
  drag = null;
});
canvas.addEventListener("wheel", (e) => {
  e.preventDefault();
  if (!hv) return;
  hv.hv_zoom(e.offsetX, e.offsetY, Math.exp(-e.deltaY * 0.0015));
  dirty = true;
}, { passive: false });

window.addEventListener("keydown", (e) => {
  if (!hv || e.target.matches("input[type=search], select")) return;
  const keys = {
    " ": () => hv.hv_play(2),
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

$("file").addEventListener("change", async (e) => {
  const f = e.target.files[0];
  if (f) load(new Uint8Array(await f.arrayBuffer()), f.name);
});
$("replay").addEventListener("change", (e) => {
  const url = new URL(location);
  url.searchParams.set("replay", e.target.value);
  history.replaceState(null, "", url);
  loadUrl(`replays/${e.target.value}.json`, e.target.value);
});

// --- start -------------------------------------------------------------------

async function start() {
  const { instance } = await WebAssembly.instantiateStreaming(fetch("hyades_viewer.wasm"), {});
  hv = instance.exports;
  applyPalette();
  const constants = {
    downsample: hv.hv_juicy_constant(0), weight: hv.hv_juicy_constant(1), exposure: hv.hv_juicy_constant(2),
    radius: hv.hv_juicy_constant(3), passes: hv.hv_juicy_constant(4),
  };
  const params = new URL(location).searchParams;
  gpu = params.get("gpu") === "0" ? null : createGpu(canvas, constants);
  if (!gpu) ctx2d = canvas.getContext("2d");
  document.body.dataset.renderer = gpu ? "webgl2" : "cpu";
  window.hyades = { hv: () => hv, gpu: () => gpu, draw: () => draw() };
  requestAnimationFrame(frame);
  try {
    const index = await (await fetch("replays/index.json")).json();
    $("replay").replaceChildren(...index.map((r) => new Option(`${r.name} — ${r.label}`, r.name)));
    const want = params.get("replay");
    const pick = index.find((r) => r.name === want) ?? index[0];
    if (pick) {
      $("replay").value = pick.name;
      await loadUrl(`replays/${pick.file}`, pick.name);
    }
  } catch {
    showMessage("No recorded replays here. Open a replay file.");
  }
}

start().catch((e) => showMessage(`The viewer could not start: ${e.message}`));
