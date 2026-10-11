// The New game screen (docs/Hyades_sessions_discovery_and_security.md §5):
// make a link, open one, list open games, and play the relay test (T-169).
// All protocol logic is in the hyades-net wasm module (net/); this file
// draws its view and passes the player's clicks to it. Text from other
// players is only ever set as textContent.
import init, { start_net } from "./net/hyades_net.js";

// The pinned relays (§3.1, §4.5 rule 2). The page's CSP allows these and
// nothing else; a link cannot add one.
const RELAYS = [
  "wss://relay.damus.io",
  "wss://nos.lol",
  "wss://relay.primal.net",
  "wss://nostr.mom",
  "wss://offchain.pub",
];

const $ = (id) => document.getElementById(id);
let net = null;
let starting = null;
let pendingLink = null;
let drawn = 0;

/** A page served from this machine may name its own relay (`?relays=`), for tests and a local relay. */
function relayList() {
  const local = ["127.0.0.1", "localhost"].includes(location.hostname);
  const asked = new URLSearchParams(location.search).get("relays");
  if (local && asked) return asked.split(",").filter((u) => /^wss?:\/\/(127\.0\.0\.1|localhost)(:\d+)?\/?$/.test(u));
  return RELAYS;
}

const linkOf = (payload) => `${location.origin}${location.pathname}#j=${payload}`;

function status(msg, bad = false) {
  $("net-status").textContent = msg;
  $("net-status").classList.toggle("error", bad);
}

async function ensureNet() {
  if (net) return net;
  if (!starting) {
    starting = (async () => {
      await init({ module_or_path: new URL("./net/hyades_net_bg.wasm", import.meta.url) });
      const n = await start_net(relayList());
      n.on_change(() => schedule());
      net = n;
      return n;
    })();
  }
  return starting;
}

function schedule() {
  if (drawn) return;
  drawn = requestAnimationFrame(() => {
    drawn = 0;
    render();
  });
}

function el(tag, text, cls) {
  const e = document.createElement(tag);
  if (text !== undefined && text !== null) e.textContent = String(text);
  if (cls) e.className = cls;
  return e;
}

function button(label, onClick, title) {
  const b = el("button", label);
  if (title) b.title = title;
  b.addEventListener("click", () => {
    try {
      onClick();
    } catch (e) {
      status(String(e), true);
    }
  });
  return b;
}

function table(target, head, rows) {
  const t = $(target);
  const tr = el("tr");
  for (const h of head) tr.append(el("th", h));
  t.replaceChildren(tr, ...rows.map((cells) => {
    const r = el("tr");
    for (const c of cells) r.append(c instanceof Node ? (() => { const td = el("td"); td.append(c); return td; })() : el("td", c));
    return r;
  }));
}

function render() {
  if (!net) return;
  const v = JSON.parse(net.view());
  const inRoom = v.mode === "room";
  $("new-start").hidden = inRoom;
  $("new-room").hidden = !inRoom;

  // Open games (§5.3): a plain list, newest first, no ranking.
  const open = [...v.open_rooms].sort((a, b) => a.age_s - b.age_s);
  $("open-list").replaceChildren(...open.map((r) => {
    const b = button(`${r.host ?? "a host"} — ${r.filled}/${r.seats} seats, ${r.round_s} s × ${r.rounds} rounds`, () => openLink(r.link));
    const li = el("li");
    li.append(b);
    return li;
  }));
  $("open-empty").hidden = open.length > 0;

  if (inRoom) renderRoom(v);
  table("relay-table", ["relay", "state", "sent", "accepted", "rate-limited", "rejected", "timeouts", "queued", "waiting", "last hint", "note"],
    v.relays.map((r) => [r.url.replace(/^wss?:\/\//, ""), r.usable ? r.status : `${r.status}, read only`, r.sent, r.accepted, r.rate_limited, r.rejected,
      r.timeouts, r.queued, r.backoff_s ? `${r.backoff_s} s` : "", r.last_hint ?? "", r.note || r.last_reason]));
  $("net-log").textContent = v.log.join("\n");
}

function renderRoom(v) {
  const room = v.room;
  const m = v.match;
  const head = $("room-head");
  head.replaceChildren();
  if (v.link) {
    const full = linkOf(v.link);
    head.append(el("span", v.host ? "Your room. Send this link: " : "Room link: "));
    head.append(el("code", full));
    head.append(" ", button("Copy link", () => navigator.clipboard.writeText(full).then(() => status("Link copied."))));
  }
  const p = v.params;
  if (p) head.append(el("div", `${p.seats} seats · ${p.round_s} s per round · ${p.rounds} rounds · patience ${p.patience_s} s${p.public ? " · listed" : ""}`, "dim"));

  const actions = $("room-actions");
  actions.replaceChildren();
  if (!room) {
    actions.append(el("span", "Looking for the room on the relays…", "dim"));
  } else {
    const place = { seat: "You have a seat.", queue: "You are waiting to play.", kicked: "The host removed you.", none: "" }[room.place];
    if (place) actions.append(el("span", place));
    if (!v.host && room.place === "none" && !v.join_sent) {
      actions.append(button(room.seats.length < (p?.seats ?? 0) && !room.started ? "Take a seat" : "Join the queue", () => {
        net.join($("f-name").value);
        status("Asking the host…");
      }));
    }
    if (v.host && !room.started) {
      const b = button("Start the match", () => net.start(), "Fix the seat table and sign it");
      b.disabled = room.seats.length < 2;
      actions.append(b);
    }
  }
  if (m && m.my_seat !== null && !m.accept_sent) {
    actions.append(button("Accept the parameters and sign", () => net.accept(), "Signing the genesis is your audit of the parameters (§5.2)"));
  }

  const seats = room?.seats ?? [];
  $("room-seats").replaceChildren(...seats.map((s) => {
    const li = el("li", `${s.name}${s.me ? " (you)" : ""}`);
    li.title = s.key;
    if (v.host && !room.started && !s.me) li.append(button("Kick", () => net.kick(s.key)));
    return li;
  }));
  const queue = room?.queue ?? [];
  $("room-queue-box").hidden = queue.length === 0;
  $("room-queue").replaceChildren(...queue.map((s) => el("li", `${s.name}${s.me ? " (you)" : ""}`)));

  $("match-box").hidden = !m;
  if (!m) return;
  const mh = $("match-head");
  const round = Math.min(m.round + 1, m.rounds);
  let line = `Round ${round} of ${m.rounds} — ${m.phase}`;
  if (m.phase === "commit" && m.floor_left_s > 0) line += ` (orders close in ${m.floor_left_s} s)`;
  if (m.my_seat === null) line += " — you are watching";
  else if (m.my_order !== null) line += ` — your order: ${m.my_order === "pass" ? "pass" : `card ${m.my_order + 1}`}`;
  mh.textContent = line;
  if (m.accepted.length < m.names.length) mh.append(el("div", `Signed: ${m.accepted.length} of ${m.names.length}`, "dim"));

  $("match-order").hidden = m.my_seat === null || m.phase === "done";
  $("auto").checked = m.auto;
  $("order").disabled = m.auto;
  if (!$("order").options.length) {
    $("order").append(new Option("pass", "65535"));
    for (let c = 0; c < 18; c++) $("order").append(new Option(`card ${c + 1}`, String(c)));
  }

  const mark = (yes, text = "✓") => el("span", yes ? text : "·", yes ? "ok" : "dim");
  table("match-seats", ["seat", "committed", "revealed", "timed out"], m.seats.map((s, i) => [
    `${s.name}${i === m.my_seat ? " (you)" : ""}`, mark(s.committed), mark(s.revealed), s.defaulted ? el("span", "yes", "bad") : "",
  ]));
  table("match-history", ["round", "orders", "timed out", "checkpoints agree"], m.history.map((h) => {
    const agree = h.agree ? el("span", "yes", "ok") : el("span", Object.keys(h.checkpoints).length ? "no / not all in" : "waiting", "dim");
    return [h.round + 1, h.applied.map((a) => (a === "pass" ? "pass" : a + 1)).join(" "), h.defaulted.map((s) => m.names[s]).join(", "), agree];
  }));
  const alerts = [...m.equivocations.map((e) => `Equivocation: ${e}`), ...m.late_changes.map((e) => `Late change: ${e}`)];
  $("match-alerts").hidden = alerts.length === 0;
  $("match-alerts").textContent = alerts.join("\n");
  if (m.phase === "done") status(m.final_sent || m.my_seat === null ? "The match is over. Save the diagnostics and the match record below and send them back." : "The last round resolved; sending the final checkpoint…");
}

function openLink(link) {
  try {
    net.open_link(link);
    history.replaceState(null, "", `${location.pathname}${location.search}#j=${link.split("#j=").pop()}`);
    status("Room opened. Waiting for the host's room state…");
  } catch (e) {
    status(String(e), true);
  }
}

function save(name, type, text) {
  const a = el("a");
  a.href = URL.createObjectURL(new Blob([text], { type }));
  a.download = name;
  document.body.append(a);
  a.click();
  setTimeout(() => {
    URL.revokeObjectURL(a.href);
    a.remove();
  }, 1000);
}

let wired = false;
function wire() {
  if (wired) return;
  wired = true;
  $("create-form").addEventListener("submit", (e) => {
    e.preventDefault();
    const num = (id) => Number($(id).value);
    try {
      const payload = net.create(num("f-seats"), num("f-round"), num("f-rounds"), num("f-patience"), $("f-public").checked, $("f-name").value);
      history.replaceState(null, "", `${location.pathname}${location.search}#j=${payload}`);
      status("Room made. Send the link; players appear below as they join.");
    } catch (err) {
      status(String(err), true);
    }
  });
  $("join-form").addEventListener("submit", (e) => {
    e.preventDefault();
    openLink($("f-link").value.trim());
  });
  $("auto").addEventListener("change", () => net.set_auto($("auto").checked));
  $("order").addEventListener("change", () => net.choose(Number($("order").value)));
  $("export-diag").addEventListener("click", () => net && save(`hyades-relay-diagnostics-${Date.now()}.tsv`, "text/tab-separated-values", net.diagnostics()));
  $("export-record").addEventListener("click", () => net && save(`hyades-match-record-${Date.now()}.json`, "application/json", net.transcript()));
  try {
    const saved = localStorage.getItem("hyades.name");
    if (saved) $("f-name").value = saved;
  } catch {}
  $("f-name").addEventListener("change", () => {
    try {
      localStorage.setItem("hyades.name", $("f-name").value);
    } catch {}
  });
  setInterval(schedule, 1000);
}

/** Shows the screen; `link` is a room link from the page's address, if any. */
export async function enterNew(link) {
  wire();
  pendingLink = link ?? pendingLink;
  try {
    await ensureNet();
  } catch (e) {
    status(`The network module did not start: ${e}`, true);
    return;
  }
  const v = JSON.parse(net.view());
  if (pendingLink && v.mode !== "room") {
    const l = pendingLink;
    pendingLink = null;
    openLink(l);
  } else if (v.mode === "idle") {
    net.browse();
    status(`Connected to ${relayList().length} relays. Make a link, or open one.`);
  }
  render();
}
