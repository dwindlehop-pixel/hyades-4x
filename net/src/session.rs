//! The room and round protocol (sessions spec §5.1, netcode §5), with the
//! relay rules of §4.3.1–§4.3.2, as one state machine that does no I/O.
//!
//! The host gives it relay messages, signatures and the time; it answers with
//! `Output`s: connect to a relay, send text to a relay, sign bytes. Signing is
//! asked for rather than done, because a seat's Ed25519 key lives in WebCrypto
//! and never enters this module (§4.1).
//!
//! What this build is: the transport test of T-169. Seats play rounds of
//! hidden simultaneous orders over public relays, through commit, reveal,
//! timeout votes and checkpoints, while every relay reply is logged. What it
//! is not: a game. The engine is not in the browser (T-165), so an order is a
//! card number nobody applies, and a checkpoint is the hash of the orders a
//! seat applied (R-SES11's inputs leaf) rather than a state root.

use crate::bytes::{b64, canonical_json, from_b64, hex, hex_array, sha256, Drbg};
use crate::frame::{self, Frame, FrameKey, Kind, Order, Phase, CARD_COUNT, FRAME_LEN, PASS};
use crate::link::{LinkData, Params};
use crate::nostr::{id_and_pow_ok, make_event, topic, Event, NostrKey, EVENT_KIND, EXPIRY_SECONDS};
use crate::relay::{self, Action, Link, Outcome, Queued, RelayMsg, Status, CAP_MS, MIN_WAIT_MS};
use ed25519_dalek::{Signature, VerifyingKey};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

/// NIP-13 bits a join or queue request must carry (§4.5 rule 5). *Placeholder.*
pub const JOIN_POW: u32 = 8;
/// The host keeps at most this many queued spectators (§4.5 rule 5). *Placeholder.*
pub const QUEUE_CAP: usize = 64;
/// A predecessor's frame missing from a relay this long is carried there (§4.3.2 rule 6). *Placeholder.*
pub const CARRY_AFTER_MS: f64 = 20_000.0;
/// A public room re-announces itself this often (§5.3). *Placeholder.*
pub const ANNOUNCE_EVERY_MS: f64 = 60_000.0;
/// An announcement older than this drops out of the Open games list.
pub const ANNOUNCE_STALE_MS: f64 = 180_000.0;
/// The last checkpoint waits this long after its round resolves, for late votes.
pub const FINAL_GRACE_MS: f64 = 5_000.0;
const LOG_CAP: usize = 100_000;
const SUB_ID: &str = "h";
const MAX_FRAMES_PER_EVENT: usize = 64;
const MAX_CONTENT: usize = 64 * 1024;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Output {
    /// Open a websocket to relay `i` at this URL.
    Connect(usize, String),
    /// Send this text on relay `i`'s socket.
    Send(usize, String),
    /// Sign these bytes with the seat key and call `signed(req, sig)`.
    Sign(u64, Vec<u8>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Member {
    key: [u8; 32],
    name: String,
}

#[derive(Clone, Debug, Default)]
struct RoomState {
    seq: u64,
    seats: Vec<Member>,
    queue: Vec<Member>,
    kicked: Vec<[u8; 32]>,
    started: bool,
    genesis: Option<Value>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Idle,
    Browse,
    Room,
}

#[derive(Clone, Debug, Default)]
struct Round {
    opened_at: Option<f64>,
    gate_at: Option<f64>,
    completed_at: Option<f64>,
    commits: BTreeMap<u16, [u8; 32]>,
    reveals: BTreeMap<u16, [u8; 32]>,
    votes: BTreeMap<(u16, Phase), BTreeSet<u16>>,
    checkpoints: BTreeMap<u16, [u8; 32]>,
    mine: Option<(Order, [u8; 16])>,
    committed: bool,
    revealed: bool,
    voted: BTreeSet<(u16, Phase)>,
}

struct Match {
    genesis: Value,
    session: [u8; 32],
    seats: Vec<[u8; 32]>,
    names: Vec<String>,
    keys: Vec<VerifyingKey>,
    params: Params,
    my_seat: Option<u16>,
    accepts: BTreeMap<u16, Value>,
    accept_sent: bool,
    started_at: Option<f64>,
    frames: BTreeMap<FrameKey, [u8; FRAME_LEN]>,
    first_held: BTreeMap<FrameKey, f64>,
    on_relay: BTreeMap<FrameKey, BTreeSet<usize>>,
    carried: BTreeSet<(FrameKey, usize)>,
    rounds: Vec<Round>,
    current: usize,
    sent_checkpoint: BTreeMap<u32, [u8; 32]>,
    late_logged: BTreeSet<u32>,
    final_sent: bool,
    equivocations: Vec<String>,
    late_changes: Vec<String>,
    auto: bool,
    chosen: Option<u16>,
    /// The seat whose key is the room's host key, which alone signs a kick.
    host_seat: Option<u16>,
    /// Kicked seat → the first round it is handed to the default order:
    /// the earliest such round among the host's kick frames held.
    kicks: BTreeMap<u16, usize>,
    /// The seats this client has signed a kick for, held or not yet.
    kicks_sent: BTreeSet<u16>,
}

impl Match {
    fn n(&self) -> usize {
        self.seats.len()
    }

    /// The timeout quorum: netcode §5.2's ⌊N/2⌋+1 excluding the subject,
    /// capped at the N−1 seats that can vote, so two seats can still time
    /// each other out.
    fn quorum(&self) -> usize {
        (self.n() / 2 + 1).min(self.n() - 1).max(1)
    }

    /// A seat defaults in a round when the host kicked it at or before that
    /// round (R-SES14) or a timeout quorum voted against it.
    fn defaulted(&self, r: usize, s: u16) -> bool {
        if self.kicks.get(&s).is_some_and(|&from| from <= r) {
            return true;
        }
        let rd = &self.rounds[r];
        [Phase::Commit, Phase::Reveal].iter().any(|&p| rd.votes.get(&(s, p)).is_some_and(|v| v.len() >= self.quorum()))
    }

    fn valid_reveal(&self, r: usize, s: u16) -> Option<Order> {
        let rd = &self.rounds[r];
        let (order, salt) = frame::read_reveal(rd.reveals.get(&s)?);
        let commit = rd.commits.get(&s)?;
        (frame::commit_payload(&self.session, r as u32, s, &order, &salt) == *commit).then(|| order.coerce())
    }

    fn commit_gate(&self, r: usize) -> bool {
        (0..self.n() as u16).all(|s| self.rounds[r].commits.contains_key(&s) || self.defaulted(r, s))
    }

    fn complete(&self, r: usize) -> bool {
        (0..self.n() as u16).all(|s| self.defaulted(r, s) || self.valid_reveal(r, s).is_some())
    }

    /// What every seat's order resolved to: a timeout quorum overrides a
    /// reveal, so the result is a function of the set of frames held.
    fn applied(&self, r: usize) -> Vec<Order> {
        (0..self.n() as u16)
            .map(|s| if self.defaulted(r, s) { PASS } else { self.valid_reveal(r, s).unwrap_or(PASS) })
            .collect()
    }

    fn root(&self, r: usize) -> [u8; 32] {
        frame::inputs_root(&self.session, r as u32, &self.applied(r))
    }

    fn floor_ms(&self) -> f64 {
        self.params.round_s as f64 * 1000.0
    }

    fn patience_ms(&self) -> f64 {
        self.params.patience_s as f64 * 1000.0
    }

    fn done(&self) -> bool {
        self.rounds.last().is_some_and(|r| r.completed_at.is_some())
    }
}

enum Pending {
    Frame { batch: u64, slot: usize, body: [u8; frame::BODY_LEN] },
    Lobby { msg: Value, topics: Vec<String>, priority: u8, pow: u32, replace: Option<String>, what: String },
}

struct Batch {
    slots: Vec<Option<[u8; FRAME_LEN]>>,
    priority: u8,
    what: String,
}

/// One event to publish: to every relay, or to `only` (the carrier, §4.3.2 rule 6).
struct Post {
    ev: Event,
    priority: u8,
    replace: Option<String>,
    what: String,
    only: Option<usize>,
    frames: Vec<FrameKey>,
}

struct LogEntry {
    t: f64,
    relay: String,
    what: String,
    detail: String,
}

pub struct Client {
    relays: Vec<Link>,
    rng: Drbg,
    nostr: NostrKey,
    me: [u8; 32],
    t0: f64,
    out: Vec<Output>,
    log: Vec<LogEntry>,
    next_req: u64,
    next_seq: u64,
    next_batch: u64,
    pending: BTreeMap<u64, Pending>,
    batches: BTreeMap<u64, Batch>,
    own_event_frames: BTreeMap<String, Vec<FrameKey>>,
    seen: BTreeMap<String, Vec<FrameKey>>,
    held_events: Vec<(usize, Event)>,
    held_accepts: Vec<(Value, [u8; 32])>,
    /// Signed acceptances by signer: the event content, when first held, and
    /// the relays known to hold it — so the carrier can carry them too.
    accept_content: BTreeMap<[u8; 32], (String, f64)>,
    accept_on_relay: BTreeMap<[u8; 32], BTreeSet<usize>>,
    accept_ids: BTreeMap<String, [u8; 32]>,
    accept_carried: BTreeSet<([u8; 32], usize)>,
    mode: Mode,
    link: Option<LinkData>,
    host: bool,
    room: Option<RoomState>,
    join_sent: bool,
    name: String,
    open_rooms: BTreeMap<String, (Value, f64)>,
    last_announce: f64,
    sub_since: u64,
    mat: Option<Match>,
}

fn sanitize_name(name: &str) -> String {
    let s: String = name.chars().filter(|c| c.is_alphanumeric() || " -_.".contains(*c)).take(24).collect();
    let s = s.trim().to_string();
    if s.is_empty() {
        "player".into()
    } else {
        s
    }
}

fn member_json(m: &Member) -> Value {
    json!({ "key": hex(&m.key), "name": m.name })
}

fn members_from(v: &Value) -> Option<Vec<Member>> {
    v.as_array()?
        .iter()
        .map(|m| {
            Some(Member { key: hex_array(m.get("key")?.as_str()?)?, name: sanitize_name(m.get("name")?.as_str()?) })
        })
        .collect()
}

fn params_json(p: &Params) -> Value {
    json!({ "seats": p.seats, "round_s": p.round_s, "rounds": p.rounds, "patience_s": p.patience_s, "public": p.public })
}

fn params_from(v: &Value) -> Option<Params> {
    let p = Params {
        seats: v.get("seats")?.as_u64()?.try_into().ok()?,
        round_s: v.get("round_s")?.as_u64()?.try_into().ok()?,
        rounds: v.get("rounds")?.as_u64()?.try_into().ok()?,
        patience_s: v.get("patience_s")?.as_u64()?.try_into().ok()?,
        public: v.get("public")?.as_bool()?,
    };
    (p.clamped() == p).then_some(p)
}

impl Client {
    /// `me` is the seat's Ed25519 public key (its private half stays with the
    /// host); `seed` is 32 bytes from the platform's cryptographic source.
    pub fn new(relay_urls: Vec<String>, me: [u8; 32], seed: [u8; 32], now: f64) -> Client {
        let mut rng = Drbg::new(seed);
        let nostr = NostrKey::generate(&mut rng);
        Client {
            relays: relay_urls.into_iter().map(Link::new).collect(),
            rng,
            nostr,
            me,
            t0: now,
            out: Vec::new(),
            log: Vec::new(),
            next_req: 1,
            next_seq: 1,
            next_batch: 1,
            pending: BTreeMap::new(),
            batches: BTreeMap::new(),
            own_event_frames: BTreeMap::new(),
            seen: BTreeMap::new(),
            held_events: Vec::new(),
            held_accepts: Vec::new(),
            accept_content: BTreeMap::new(),
            accept_on_relay: BTreeMap::new(),
            accept_ids: BTreeMap::new(),
            accept_carried: BTreeSet::new(),
            mode: Mode::Idle,
            link: None,
            host: false,
            room: None,
            join_sent: false,
            name: "player".into(),
            open_rooms: BTreeMap::new(),
            last_announce: f64::NEG_INFINITY,
            sub_since: 0,
            mat: None,
        }
    }

    fn note(&mut self, now: f64, relay: &str, what: &str, detail: impl Into<String>) {
        if self.log.len() < LOG_CAP {
            self.log.push(LogEntry { t: now - self.t0, relay: relay.into(), what: what.into(), detail: detail.into() });
        }
    }

    fn secs(now: f64) -> u64 {
        (now / 1000.0).max(0.0) as u64
    }

    // --- topics and the one subscription per relay ------------------------

    fn room_topic(&self) -> Option<String> {
        self.link.map(|l| topic(&format!("room-{}", hex(&l.room))))
    }

    fn inbox_topic(&self) -> Option<String> {
        self.link.map(|l| topic(&format!("inbox-{}", hex(&l.room))))
    }

    fn resubscribe(&mut self, now: f64) {
        let mut topics = vec![];
        match self.mode {
            Mode::Idle => {}
            Mode::Browse => topics.push(topic("open")),
            Mode::Room => {
                topics.extend(self.room_topic());
                if self.host {
                    topics.extend(self.inbox_topic());
                }
            }
        }
        self.sub_since = Self::secs(now).saturating_sub(EXPIRY_SECONDS);
        let req = json!(["REQ", SUB_ID, { "kinds": [EVENT_KIND], "#t": topics, "since": self.sub_since }]).to_string();
        for l in &mut self.relays {
            l.subscribe(req.clone());
        }
    }

    // --- signing and publishing ----------------------------------------------

    fn ask_signature(&mut self, bytes: Vec<u8>, p: Pending) {
        let req = self.next_req;
        self.next_req += 1;
        self.pending.insert(req, p);
        self.out.push(Output::Sign(req, bytes));
    }

    fn send_lobby(
        &mut self,
        msg: Value,
        topics: Vec<String>,
        priority: u8,
        pow: u32,
        replace: Option<&str>,
        what: &str,
    ) {
        let bytes = canonical_json(&msg).into_bytes();
        let replace = replace.map(str::to_string);
        self.ask_signature(bytes, Pending::Lobby { msg, topics, priority, pow, replace, what: what.into() });
    }

    fn send_frames(&mut self, frames: Vec<(Kind, u32, Vec<u8>)>, priority: u8, what: String) {
        let (Some(m), false) = (&self.mat, frames.is_empty()) else { return };
        let Some(seat) = m.my_seat else { return };
        let session = m.session;
        let batch = self.next_batch;
        self.next_batch += 1;
        self.batches.insert(batch, Batch { slots: vec![None; frames.len()], priority, what });
        for (slot, (kind, round, payload)) in frames.into_iter().enumerate() {
            let body = frame::body(kind, seat, round, &session, &payload);
            self.ask_signature(body.to_vec(), Pending::Frame { batch, slot, body });
        }
    }

    fn publish(&mut self, post: Post, now: f64) {
        let Post { ev, priority, replace, what, only, frames } = post;
        let text = json!(["EVENT", ev]).to_string();
        if !frames.is_empty() {
            self.own_event_frames.insert(ev.id.clone(), frames);
        }
        for i in 0..self.relays.len() {
            if only.is_some_and(|j| j != i) {
                continue;
            }
            let seq = self.next_seq;
            self.next_seq += 1;
            self.relays[i].enqueue(Queued {
                priority,
                seq,
                event_id: ev.id.clone(),
                text: text.clone(),
                replace: replace.clone(),
                what: what.clone(),
            });
        }
        let url = only.map_or("*".to_string(), |j| self.relays[j].url.clone());
        self.note(now, &url, "queued", format!("{what} p{priority}"));
    }

    fn frames_content(frames: &[[u8; FRAME_LEN]]) -> String {
        json!({ "v": 1, "f": frames.iter().map(|f| b64(f)).collect::<Vec<_>>() }).to_string()
    }

    /// The host's answer to an `Output::Sign`.
    pub fn signed(&mut self, req: u64, sig: [u8; 64], now: f64) {
        match self.pending.remove(&req) {
            None => {}
            Some(Pending::Lobby { msg, topics, priority, pow, replace, what }) => {
                let content = json!({ "v": 1, "m": msg, "k": hex(&self.me), "s": b64(&sig) }).to_string();
                let ev = make_event(&self.nostr, &topics, content, pow, Self::secs(now), &mut self.rng);
                // Act on our own message now rather than on its echo, which a
                // relay need not send back to its author.
                self.seen.insert(ev.id.clone(), vec![]);
                if msg.get("type").and_then(Value::as_str) == Some("accept") {
                    self.hold_accept(self.me, &ev, None, now);
                }
                self.on_lobby(msg, self.me, &ev, now);
                self.publish(Post { ev, priority, replace, what, only: None, frames: vec![] }, now);
            }
            Some(Pending::Frame { batch, slot, body }) => {
                let f = frame::assemble(&body, &sig);
                self.ingest_frame(&f, None, now);
                let Some(b) = self.batches.get_mut(&batch) else { return };
                b.slots[slot] = Some(f);
                if b.slots.iter().all(Option::is_some) {
                    let b = self.batches.remove(&batch).expect("present");
                    let frames: Vec<[u8; FRAME_LEN]> = b.slots.into_iter().flatten().collect();
                    let keys = self.keys_of(&frames);
                    let Some(room) = self.room_topic() else { return };
                    let ev = make_event(
                        &self.nostr,
                        &[room],
                        Self::frames_content(&frames),
                        0,
                        Self::secs(now),
                        &mut self.rng,
                    );
                    self.publish(
                        Post { ev, priority: b.priority, replace: None, what: b.what, only: None, frames: keys },
                        now,
                    );
                }
            }
        }
    }

    fn keys_of(&self, frames: &[[u8; FRAME_LEN]]) -> Vec<FrameKey> {
        let Some(m) = &self.mat else { return vec![] };
        frames.iter().filter_map(|f| frame::decode(f, &m.session, &m.keys)).map(|f| f.key()).collect()
    }

    // --- relay events ---------------------------------------------------------

    pub fn relay_opened(&mut self, i: usize, now: f64) {
        self.relays[i].opened();
        let url = self.relays[i].url.clone();
        self.note(now, &url, "open", "");
    }

    pub fn relay_closed(&mut self, i: usize, now: f64) {
        let wait = self.relays[i].closed(now, &mut self.rng);
        let url = self.relays[i].url.clone();
        self.note(now, &url, "closed", format!("reconnect in {:.1} s", wait / 1000.0));
    }

    /// The NIP-11 screen's verdict (§4.3.1 rule 3). An excluded relay is
    /// still read, never written to.
    pub fn screen(&mut self, i: usize, usable: bool, note: &str, now: f64) {
        self.relays[i].usable = usable;
        self.relays[i].note = note.into();
        let url = self.relays[i].url.clone();
        self.note(now, &url, if usable { "screen-pass" } else { "screen-exclude" }, note);
    }

    /// How long a relay's hint may hold a write: the time left before this
    /// seat's patience for the current phase runs out (§4.3.2 rule 2).
    fn phase_cap(&self, now: f64) -> f64 {
        let Some(m) = &self.mat else { return CAP_MS };
        let (Some(_), Some(_)) = (m.my_seat, m.started_at) else { return CAP_MS };
        if m.done() {
            return CAP_MS;
        }
        let r = &m.rounds[m.current];
        let opened = r.opened_at.unwrap_or(now);
        let deadline = if !r.committed {
            opened + m.floor_ms() + m.patience_ms()
        } else {
            r.gate_at.unwrap_or(opened + m.floor_ms()) + m.patience_ms()
        };
        (deadline - now).max(MIN_WAIT_MS)
    }

    pub fn relay_text(&mut self, i: usize, text: &str, now: f64) {
        let url = self.relays[i].url.clone();
        let Some(msg) = relay::parse(text) else {
            self.note(now, &url, "unparsed", text.chars().take(120).collect::<String>());
            return;
        };
        match msg {
            RelayMsg::Ok(reply) => {
                let cap = self.phase_cap(now);
                let (id, ok, message) = (reply.id.clone(), reply.ok, reply.message.clone());
                match self.relays[i].on_ok(reply, cap, now, &mut self.rng) {
                    Outcome::Accepted { queued, latency_ms } => {
                        self.note(now, &url, "accepted", format!("{} {:.0} ms", queued.what, latency_ms));
                        if let Some(signer) = self.accept_ids.get(&id) {
                            self.accept_on_relay.entry(*signer).or_default().insert(i);
                        }
                        if let (Some(keys), Some(m)) = (self.own_event_frames.get(&id), self.mat.as_mut()) {
                            for k in keys {
                                m.on_relay.entry(*k).or_default().insert(i);
                            }
                        }
                    }
                    Outcome::RateLimited { what, wait_ms, hint_raw, message } => {
                        let hint = hint_raw.unwrap_or_else(|| "none".into());
                        self.note(
                            now,
                            &url,
                            "rate-limited",
                            format!("{what}: {message} | hint {hint} | wait {:.1} s", wait_ms / 1000.0),
                        );
                    }
                    Outcome::Rejected { what, message } => {
                        self.note(now, &url, "rejected", format!("{what}: {message}"))
                    }
                    Outcome::NotOurs => self.note(now, &url, "ok-unmatched", format!("{id} {ok} {message}")),
                }
            }
            RelayMsg::Closed { message, hint_ms, hint_raw, .. } => {
                let cap = self.phase_cap(now);
                let wait = self.relays[i].on_closed(hint_ms, cap, now, &mut self.rng);
                let hint = hint_raw.unwrap_or_else(|| "none".into());
                self.note(
                    now,
                    &url,
                    "sub-closed",
                    format!("{message} | hint {hint} | resubscribe in {:.1} s", wait / 1000.0),
                );
            }
            RelayMsg::Event { event, .. } => self.on_event(i, event, now),
            RelayMsg::Eose => self.relays[i].on_eose(),
            RelayMsg::Notice(m) => self.note(now, &url, "notice", m),
            RelayMsg::Auth => self.note(now, &url, "auth-asked", "NIP-42 is not supported; writes may be refused"),
        }
    }

    fn on_event(&mut self, i: usize, ev: Event, now: f64) {
        if ev.kind != EVENT_KIND || ev.content.len() > MAX_CONTENT {
            return;
        }
        if let Some(signer) = self.accept_ids.get(&ev.id) {
            self.accept_on_relay.entry(*signer).or_default().insert(i);
        }
        if let Some(keys) = self.seen.get(&ev.id) {
            if let Some(m) = self.mat.as_mut() {
                for k in keys {
                    m.on_relay.entry(*k).or_default().insert(i);
                }
            }
            return;
        }
        let Ok(content) = serde_json::from_str::<Value>(&ev.content) else { return };
        if let Some(list) = content.get("f").and_then(Value::as_array) {
            if self.mat.is_none() {
                if self.held_events.len() < 4096 {
                    self.held_events.push((i, ev));
                }
                return;
            }
            let mut keys = vec![];
            for f in list.iter().take(MAX_FRAMES_PER_EVENT).filter_map(Value::as_str).filter_map(from_b64) {
                keys.extend(self.ingest_frame(&f, Some(i), now));
            }
            self.seen.insert(ev.id, keys);
        } else if let Some(msg) = content.get("m") {
            let signer = content.get("k").and_then(Value::as_str).and_then(hex_array::<32>);
            let sig = content.get("s").and_then(Value::as_str).and_then(from_b64);
            self.seen.insert(ev.id.clone(), vec![]);
            let (Some(signer), Some(sig)) = (signer, sig) else { return };
            let Ok(sig) = <[u8; 64]>::try_from(sig.as_slice()) else { return };
            let Ok(vk) = VerifyingKey::from_bytes(&signer) else { return };
            if vk.verify_strict(canonical_json(msg).as_bytes(), &Signature::from_bytes(&sig)).is_err() {
                let url = self.relays[i].url.clone();
                self.note(now, &url, "bad-signature", "a lobby message failed its Ed25519 check");
                return;
            }
            if msg.get("type").and_then(Value::as_str) == Some("accept") {
                self.hold_accept(signer, &ev, Some(i), now);
            }
            self.on_lobby(msg.clone(), signer, &ev, now);
        }
    }

    fn hold_accept(&mut self, signer: [u8; 32], ev: &Event, relay: Option<usize>, now: f64) {
        self.accept_ids.insert(ev.id.clone(), signer);
        self.accept_content.entry(signer).or_insert((ev.content.clone(), now));
        let on = self.accept_on_relay.entry(signer).or_default();
        if let Some(i) = relay {
            on.insert(i);
        }
    }

    // --- the lobby ------------------------------------------------------------

    fn room_hex(&self) -> String {
        self.link.map(|l| hex(&l.room)).unwrap_or_default()
    }

    fn on_lobby(&mut self, msg: Value, signer: [u8; 32], ev: &Event, now: f64) {
        let ty = msg.get("type").and_then(Value::as_str).unwrap_or("");
        if ty == "announce" {
            if self.mode == Mode::Browse {
                if let Some(room) = msg.get("room").and_then(Value::as_str) {
                    let room = room.to_string();
                    let newer = self.open_rooms.get(&room).is_none_or(|(old, _)| {
                        old.get("seq").and_then(Value::as_u64) < msg.get("seq").and_then(Value::as_u64)
                    });
                    if newer {
                        self.open_rooms.insert(room, (msg, now));
                    }
                }
            }
            return;
        }
        if msg.get("room").and_then(Value::as_str) != Some(&self.room_hex()) {
            return;
        }
        match ty {
            "room" => self.on_room_state(msg, signer, now),
            "join" => {
                let from_inbox = self.inbox_topic().is_some_and(|t| ev.topics().any(|x| x == t));
                if self.host && from_inbox && id_and_pow_ok(ev, JOIN_POW) {
                    let name = sanitize_name(msg.get("name").and_then(Value::as_str).unwrap_or(""));
                    self.admit(Member { key: signer, name }, now);
                }
            }
            "accept" => self.on_accept(msg, signer, now),
            _ => {}
        }
    }

    fn on_room_state(&mut self, msg: Value, signer: [u8; 32], now: f64) {
        let Some(link) = self.link else { return };
        if signer != link.host || self.host {
            return;
        }
        let seq = msg.get("seq").and_then(Value::as_u64).unwrap_or(0);
        if self.room.as_ref().is_some_and(|r| r.seq >= seq) {
            return;
        }
        let (Some(seats), Some(queue)) =
            (msg.get("seats").and_then(members_from), msg.get("queue").and_then(members_from))
        else {
            return;
        };
        let kicked = msg
            .get("kicked")
            .and_then(Value::as_array)
            .map(|a| a.iter().filter_map(|k| hex_array(k.as_str()?)).collect())
            .unwrap_or_default();
        let started = msg.get("started").and_then(Value::as_bool).unwrap_or(false);
        let genesis = msg.get("genesis").filter(|g| !g.is_null()).cloned();
        self.room = Some(RoomState { seq, seats, queue, kicked, started, genesis: genesis.clone() });
        self.note(now, "", "room-state", format!("seq {seq}"));
        if let (Some(g), None) = (genesis, &self.mat) {
            self.setup_match(g, now);
        }
    }

    /// The host admits joiners in the order it receives them (§5.1 step 4):
    /// a seat while one is free and the match has not started, the queue
    /// after that, nothing past the queue's cap.
    fn admit(&mut self, m: Member, now: f64) {
        let Some(link) = self.link else { return };
        let Some(room) = self.room.as_mut() else { return };
        if room.kicked.contains(&m.key) || room.seats.iter().chain(&room.queue).any(|x| x.key == m.key) {
            return;
        }
        let what = if !room.started && room.seats.len() < link.params.seats as usize {
            room.seats.push(m);
            "seated"
        } else if room.queue.len() < QUEUE_CAP {
            room.queue.push(m);
            "queued"
        } else {
            self.note(now, "", "join-dropped", "the queue is full");
            return;
        };
        self.note(now, "", what, "");
        self.publish_room(now);
    }

    fn publish_room(&mut self, now: f64) {
        let Some(room) = self.room.as_mut() else { return };
        room.seq += 1;
        let msg = json!({
            "type": "room",
            "room": self.link.map(|l| hex(&l.room)),
            "seq": room.seq,
            "seats": room.seats.iter().map(member_json).collect::<Vec<_>>(),
            "queue": room.queue.iter().map(member_json).collect::<Vec<_>>(),
            "kicked": room.kicked.iter().map(|k| hex(k)).collect::<Vec<_>>(),
            "started": room.started,
            "genesis": room.genesis.clone(),
        });
        let topics = self.room_topic().into_iter().collect();
        self.send_lobby(msg, topics, 3, 0, Some("room"), "room-state");
        self.last_announce = f64::NEG_INFINITY;
        let _ = now;
    }

    // --- user actions ---------------------------------------------------------

    /// List the Open games (§5.3).
    pub fn browse(&mut self, now: f64) {
        if self.mode == Mode::Room {
            return;
        }
        self.mode = Mode::Browse;
        self.resubscribe(now);
    }

    /// Make a room and its link (§5.1). Returns the link payload.
    pub fn create(&mut self, params: Params, name: &str, now: f64) -> String {
        let params = params.clamped();
        let link = LinkData { room: self.rng.bytes(), host: self.me, params };
        self.name = sanitize_name(name);
        self.link = Some(link);
        self.host = true;
        self.mode = Mode::Room;
        self.room =
            Some(RoomState { seats: vec![Member { key: self.me, name: self.name.clone() }], ..Default::default() });
        self.resubscribe(now);
        self.publish_room(now);
        self.note(now, "", "created", format!("{} seats", params.seats));
        link.encode()
    }

    /// Open a room from its link.
    pub fn open_link(&mut self, link: &str, now: f64) -> Result<(), String> {
        let l = LinkData::decode(link)?;
        if l.host == self.me {
            return Err("this tab made the link; open it in another tab or browser".into());
        }
        self.link = Some(l);
        self.host = false;
        self.mode = Mode::Room;
        self.resubscribe(now);
        self.note(now, "", "opened-link", hex(&l.room));
        Ok(())
    }

    /// Ask the host for a seat, or a place in the queue (§5.1). Written only
    /// after the room's state has been read, and only while it has room
    /// (§4.3.1 rule 6).
    pub fn join(&mut self, name: &str, now: f64) -> Result<(), String> {
        let (Some(link), false) = (self.link, self.host) else { return Err("no room to join".into()) };
        let Some(room) = &self.room else { return Err("the room's state has not arrived yet".into()) };
        if room.kicked.contains(&self.me) {
            return Err("the host removed this player".into());
        }
        let full = (room.started || room.seats.len() >= link.params.seats as usize) && room.queue.len() >= QUEUE_CAP;
        if full {
            return Err("the room and its queue are full".into());
        }
        self.name = sanitize_name(name);
        let msg = json!({ "type": "join", "room": hex(&link.room), "name": self.name });
        let topics = self.inbox_topic().into_iter().collect();
        self.send_lobby(msg, topics, 2, JOIN_POW, None, "join");
        self.join_sent = true;
        self.note(now, "", "join-sent", self.name.clone());
        Ok(())
    }

    /// Kick a seated player. Before the match starts the first queued
    /// spectator takes the seat (ruling 6); after it starts the seat plays
    /// the default order from the next round this host has not committed
    /// (ruling 12, R-SES14).
    pub fn kick(&mut self, key_hex: &str, now: f64) -> Result<(), String> {
        let key: [u8; 32] = hex_array(key_hex).ok_or("not a key")?;
        if !self.host || key == self.me {
            return Err("only the host kicks, and not itself".into());
        }
        let room = self.room.as_mut().ok_or("no room")?;
        if room.started {
            return self.kick_in_match(key, now);
        }
        let before = room.seats.len();
        room.seats.retain(|m| m.key != key);
        if room.seats.len() == before {
            return Err("not seated".into());
        }
        room.kicked.push(key);
        if !room.queue.is_empty() {
            let next = room.queue.remove(0);
            room.seats.push(next);
        }
        self.note(now, "", "kicked", key_hex.to_string());
        self.publish_room(now);
        Ok(())
    }

    fn kick_in_match(&mut self, key: [u8; 32], now: f64) -> Result<(), String> {
        let m = self.mat.as_ref().ok_or("no match yet")?;
        let subject = m.seats.iter().position(|k| *k == key).ok_or("not seated")? as u16;
        if m.kicks_sent.contains(&subject) || m.kicks.contains_key(&subject) {
            return Err("already kicked".into());
        }
        let r = m.current;
        let from = if m.rounds[r].committed { r + 1 } else { r };
        if m.done() || from >= m.rounds.len() {
            return Err("no round left to kick from".into());
        }
        self.mat.as_mut().expect("checked").kicks_sent.insert(subject);
        self.send_frames(
            vec![(Kind::Kick, from as u32, frame::kick_payload(subject).to_vec())],
            0,
            format!("kick r{from}"),
        );
        self.note(now, "", "kicked", format!("seat {subject} from round {from}"));
        Ok(())
    }

    /// Fix the seat table and publish the genesis descriptor (netcode §7).
    pub fn start(&mut self, now: f64) -> Result<(), String> {
        let link = self.link.ok_or("no room")?;
        if !self.host {
            return Err("only the host starts".into());
        }
        let room = self.room.as_mut().ok_or("no room")?;
        if room.started || room.seats.len() < 2 {
            return Err("a match needs at least two seats".into());
        }
        let mut params = link.params;
        params.seats = room.seats.len() as u8;
        let genesis = json!({
            "proto": 1,
            "engine": "none: transport test (T-169)",
            "room": hex(&link.room),
            "host": hex(&link.host),
            "params": params_json(&params),
            "seats": room.seats.iter().map(|m| hex(&m.key)).collect::<Vec<_>>(),
            "names": room.seats.iter().map(|m| m.name.clone()).collect::<Vec<_>>(),
        });
        room.started = true;
        room.genesis = Some(genesis.clone());
        self.publish_room(now);
        self.setup_match(genesis, now);
        Ok(())
    }

    fn setup_match(&mut self, genesis: Value, now: f64) {
        let seats: Option<Vec<[u8; 32]>> = genesis
            .get("seats")
            .and_then(Value::as_array)
            .map(|a| a.iter().filter_map(|k| hex_array(k.as_str()?)).collect());
        let names: Vec<String> = genesis
            .get("names")
            .and_then(Value::as_array)
            .map(|a| a.iter().map(|n| sanitize_name(n.as_str().unwrap_or(""))).collect())
            .unwrap_or_default();
        let (Some(seats), Some(params)) = (seats, genesis.get("params").and_then(params_from)) else {
            self.note(now, "", "bad-genesis", "");
            return;
        };
        let keys: Option<Vec<VerifyingKey>> = seats.iter().map(|k| VerifyingKey::from_bytes(k).ok()).collect();
        let Some(keys) = keys else { return };
        if seats.len() < 2 || seats.len() != params.seats as usize || names.len() != seats.len() {
            self.note(now, "", "bad-genesis", "seat count");
            return;
        }
        let session = sha256(&[canonical_json(&genesis).as_bytes()]);
        let my_seat = seats.iter().position(|k| *k == self.me).map(|i| i as u16);
        let host_seat = genesis
            .get("host")
            .and_then(Value::as_str)
            .and_then(hex_array::<32>)
            .and_then(|h| seats.iter().position(|k| *k == h))
            .map(|i| i as u16);
        self.note(now, "", "genesis", format!("session {} seat {:?}", &hex(&session)[..12], my_seat));
        self.mat = Some(Match {
            genesis,
            session,
            seats,
            names,
            keys,
            params,
            my_seat,
            accepts: BTreeMap::new(),
            accept_sent: false,
            started_at: None,
            frames: BTreeMap::new(),
            first_held: BTreeMap::new(),
            on_relay: BTreeMap::new(),
            carried: BTreeSet::new(),
            rounds: vec![Round::default(); params.rounds as usize],
            current: 0,
            sent_checkpoint: BTreeMap::new(),
            late_logged: BTreeSet::new(),
            final_sent: false,
            equivocations: vec![],
            late_changes: vec![],
            auto: true,
            chosen: None,
            host_seat,
            kicks: BTreeMap::new(),
            kicks_sent: BTreeSet::new(),
        });
        for (msg, signer) in std::mem::take(&mut self.held_accepts) {
            self.on_accept(msg, signer, now);
        }
        for (i, ev) in std::mem::take(&mut self.held_events) {
            self.on_event(i, ev, now);
        }
    }

    /// Sign the genesis descriptor: the seat's audit of the parameters
    /// (sessions spec §5.2, netcode §7).
    pub fn accept(&mut self, now: f64) -> Result<(), String> {
        let m = self.mat.as_mut().ok_or("no match yet")?;
        if m.my_seat.is_none() || m.accept_sent {
            return Err("not a seat, or already accepted".into());
        }
        m.accept_sent = true;
        let msg = json!({ "type": "accept", "room": self.link.map(|l| hex(&l.room)), "session": hex(&m.session) });
        let topics = self.room_topic().into_iter().collect();
        self.send_lobby(msg, topics, 1, 0, None, "accept");
        self.note(now, "", "accept-sent", "");
        Ok(())
    }

    fn on_accept(&mut self, msg: Value, signer: [u8; 32], now: f64) {
        let Some(m) = self.mat.as_mut() else {
            // An acceptance can outrun the genesis it signs; keep it until then.
            if self.held_accepts.len() < 64 {
                self.held_accepts.push((msg, signer));
            }
            return;
        };
        if msg.get("session").and_then(Value::as_str) != Some(&hex(&m.session)) {
            return;
        }
        let Some(seat) = m.seats.iter().position(|k| *k == signer) else { return };
        let signed = json!({ "m": msg, "k": hex(&signer) });
        m.accepts.entry(seat as u16).or_insert(signed);
        if m.accepts.len() == m.n() && m.started_at.is_none() {
            m.started_at = Some(now);
            m.rounds[0].opened_at = Some(now);
            self.note(now, "", "match-start", "every seat signed");
        }
    }

    /// Play orders automatically (a random card or a pass each round), or not.
    pub fn set_auto(&mut self, auto: bool) {
        if let Some(m) = self.mat.as_mut() {
            m.auto = auto;
        }
    }

    /// The order for the current round when not playing automatically:
    /// a card number, or anything ≥ `CARD_COUNT` for a pass.
    pub fn choose(&mut self, card: u16) {
        if let Some(m) = self.mat.as_mut() {
            m.chosen = Some(card);
        }
    }

    // --- frames -----------------------------------------------------------------

    fn ingest_frame(&mut self, bytes: &[u8], relay: Option<usize>, now: f64) -> Option<FrameKey> {
        let m = self.mat.as_mut()?;
        let f: Frame = frame::decode(bytes, &m.session, &m.keys)?;
        let key = f.key();
        if let Some(i) = relay {
            m.on_relay.entry(key).or_default().insert(i);
        }
        match m.frames.get(&key) {
            Some(old) if *old != f.bytes => {
                let what = format!("seat {} round {} {:?}", f.seat, f.round, f.kind);
                m.equivocations.push(what.clone());
                self.note(now, "", "equivocation", what);
                return Some(key);
            }
            Some(_) => return Some(key),
            None => {}
        }
        m.frames.insert(key, f.bytes);
        m.first_held.insert(key, now);
        let r = f.round as usize;
        if r >= m.rounds.len() {
            return Some(key);
        }
        let n = m.n() as u16;
        let rd = &mut m.rounds[r];
        match f.kind {
            Kind::Commit => {
                rd.commits.entry(f.seat).or_insert(f.payload);
            }
            Kind::Reveal => {
                rd.reveals.entry(f.seat).or_insert(f.payload);
            }
            Kind::Checkpoint => {
                rd.checkpoints.entry(f.seat).or_insert(f.payload);
            }
            Kind::TimeoutVote => {
                let (subject, phase) = frame::read_vote(&f.payload);
                if subject < n && subject != f.seat {
                    rd.votes.entry((subject, phase)).or_default().insert(f.seat);
                }
            }
            Kind::Kick => {
                let subject = frame::read_kick(&f.payload);
                if Some(f.seat) == m.host_seat && subject < n && subject != f.seat {
                    let from = m.kicks.entry(subject).or_insert(r);
                    *from = (*from).min(r);
                }
            }
        }
        Some(key)
    }

    // --- the clock ----------------------------------------------------------------

    /// Run everything due at `now` and return what the host must do.
    pub fn poll(&mut self, now: f64) -> Vec<Output> {
        self.advance_match(now);
        self.carry(now);
        self.announce(now);
        for i in 0..self.relays.len() {
            let mut log = vec![];
            let actions = self.relays[i].poll(now, &mut self.rng, &mut log);
            let url = self.relays[i].url.clone();
            for (what, detail) in log {
                self.note(now, &url, &what, detail);
            }
            for a in actions {
                match a {
                    Action::Connect => {
                        self.note(now, &url, "connect", "");
                        self.out.push(Output::Connect(i, url.clone()));
                    }
                    Action::Send(text) => self.out.push(Output::Send(i, text)),
                }
            }
        }
        std::mem::take(&mut self.out)
    }

    /// The earliest time `poll` has something to do, if any.
    pub fn next_wake(&self, now: f64) -> Option<f64> {
        if !self.out.is_empty() {
            return Some(now);
        }
        let mut t: Option<f64> = None;
        let mut at = |x: f64| t = Some(t.map_or(x, |y: f64| y.min(x)));
        for l in &self.relays {
            if let Some(w) = l.next_wake() {
                at(w);
            }
        }
        if let Some(m) = &self.mat {
            if m.started_at.is_some() && !m.final_sent {
                // Phases open on floors and patience; check every second.
                at(now + 1000.0);
            }
            if !m.carried.is_empty() || m.my_seat.is_some() {
                at(now + 5000.0);
            }
        }
        if self.host && self.link.is_some_and(|l| l.params.public) {
            at(self.last_announce + ANNOUNCE_EVERY_MS);
        }
        t.map(|x| x.max(now))
    }

    fn random_order(&mut self) -> Order {
        let r = self.rng.unit();
        if r < 1.0 / 3.0 {
            PASS
        } else {
            Order {
                card: ((self.rng.unit() * CARD_COUNT as f64) as u16).min(CARD_COUNT - 1),
                target_kind: 0,
                target_ref: 0,
            }
        }
    }

    /// The round loop (netcode §5), from this client's seat.
    fn advance_match(&mut self, now: f64) {
        loop {
            let Some(m) = self.mat.as_ref() else { return };
            if m.started_at.is_none() || m.done() {
                break;
            }
            let r = m.current;
            // A kicked seat stops playing from its kick's round on; it still
            // verifies, and signs its checkpoints at the end.
            let seat = m.my_seat.filter(|s| !m.kicks.get(s).is_some_and(|&from| from <= r));
            if m.rounds[r].gate_at.is_none() && m.commit_gate(r) {
                self.mat.as_mut().unwrap().rounds[r].gate_at = Some(now);
            }
            let m = self.mat.as_ref().unwrap();
            let rd = &m.rounds[r];
            let opened = rd.opened_at.unwrap_or(now);
            let mut frames: Vec<(Kind, u32, Vec<u8>)> = vec![];
            let mut priority = 2;
            let mut what = String::new();

            if let Some(seat) = seat {
                // Commit, carrying the checkpoints it deferred (§4.3.1 rule 1, §4.3.2 rule 5).
                if !rd.committed && now >= opened + m.floor_ms() {
                    let order = if m.auto {
                        Some(self.random_order())
                    } else {
                        self.mat.as_ref().unwrap().chosen.map(|c| {
                            if c < CARD_COUNT {
                                Order { card: c, target_kind: 0, target_ref: 0 }
                            } else {
                                PASS
                            }
                        })
                    };
                    if let Some(order) = order {
                        let salt: [u8; 16] = self.rng.bytes();
                        let m = self.mat.as_mut().unwrap();
                        let payload = frame::commit_payload(&m.session, r as u32, seat, &order, &salt);
                        m.rounds[r].mine = Some((order, salt));
                        m.rounds[r].committed = true;
                        m.chosen = None;
                        frames.push((Kind::Commit, r as u32, payload.to_vec()));
                        for rr in 0..r {
                            if !m.sent_checkpoint.contains_key(&(rr as u32)) {
                                let root = m.root(rr);
                                m.sent_checkpoint.insert(rr as u32, root);
                                frames.push((Kind::Checkpoint, rr as u32, root.to_vec()));
                            }
                        }
                        priority = 1;
                        what = format!("commit r{r}");
                    }
                }
                let m = self.mat.as_ref().unwrap();
                let rd = &m.rounds[r];
                // Reveal once every live seat's commit is held (netcode §5 P1).
                if rd.committed && !rd.revealed && rd.gate_at.is_some() && rd.commits.contains_key(&seat) {
                    let (order, salt) = rd.mine.expect("committed");
                    self.mat.as_mut().unwrap().rounds[r].revealed = true;
                    frames.push((Kind::Reveal, r as u32, frame::reveal_payload(&order, &salt).to_vec()));
                    priority = 0;
                    what = format!("reveal r{r}");
                }
                // Timeout votes, no sooner than patience after the phase opened (§4.3.2 rule 7).
                let m = self.mat.as_ref().unwrap();
                let rd = &m.rounds[r];
                let mut votes = vec![];
                for s in (0..m.n() as u16).filter(|&s| s != seat) {
                    if m.defaulted(r, s) {
                        continue;
                    }
                    if !rd.commits.contains_key(&s)
                        && now >= opened + m.floor_ms() + m.patience_ms()
                        && !rd.voted.contains(&(s, Phase::Commit))
                    {
                        votes.push((s, Phase::Commit));
                    }
                    if rd.commits.contains_key(&s)
                        && m.valid_reveal(r, s).is_none()
                        && rd.gate_at.is_some_and(|g| now >= g + m.patience_ms())
                        && !rd.voted.contains(&(s, Phase::Reveal))
                    {
                        votes.push((s, Phase::Reveal));
                    }
                }
                if !votes.is_empty() {
                    let m = self.mat.as_mut().unwrap();
                    for &(s, p) in &votes {
                        m.rounds[r].voted.insert((s, p));
                        frames.push((Kind::TimeoutVote, r as u32, frame::vote_payload(s, p).to_vec()));
                    }
                    if what.is_empty() {
                        what = format!("votes r{r}");
                    }
                    let detail = votes.iter().map(|(s, p)| format!("seat {s} {p:?}")).collect::<Vec<_>>().join(", ");
                    self.note(now, "", "timeout-vote", format!("round {r}: {detail}"));
                }
                if !frames.is_empty() {
                    self.send_frames(frames, priority, what);
                }
            }

            let m = self.mat.as_ref().unwrap();
            if m.rounds[r].completed_at.is_none() && m.complete(r) {
                let defaulted: Vec<u16> = (0..m.n() as u16).filter(|&s| m.defaulted(r, s)).collect();
                let last = r + 1 == m.rounds.len();
                let m = self.mat.as_mut().unwrap();
                m.rounds[r].completed_at = Some(now);
                if !last {
                    m.current = r + 1;
                    m.rounds[r + 1].opened_at = Some(now);
                }
                let took = now - opened;
                self.note(
                    now,
                    "",
                    "round-resolved",
                    format!("round {r} in {:.1} s, defaulted {defaulted:?}", took / 1000.0),
                );
                if !last {
                    continue;
                }
            }
            break;
        }
        self.final_checkpoint(now);
        self.watch_late_changes(now);
    }

    /// The last barrier's checkpoint goes out alone, after a short grace for
    /// late votes (§4.3.1 rule 1).
    fn final_checkpoint(&mut self, now: f64) {
        let Some(m) = self.mat.as_mut() else { return };
        let (Some(_), false) = (m.my_seat, m.final_sent) else { return };
        let Some(done_at) = m.rounds.last().and_then(|r| r.completed_at) else { return };
        if now < done_at + FINAL_GRACE_MS {
            return;
        }
        m.final_sent = true;
        let mut frames = vec![];
        for rr in 0..m.rounds.len() {
            if !m.sent_checkpoint.contains_key(&(rr as u32)) {
                let root = m.root(rr);
                m.sent_checkpoint.insert(rr as u32, root);
                frames.push((Kind::Checkpoint, rr as u32, root.to_vec()));
            }
        }
        self.send_frames(frames, 1, "final checkpoint".into());
        self.note(now, "", "match-end", "");
    }

    /// A vote that arrives after this seat signed its checkpoint changes the
    /// round it resolved: the case the deferral exists to absorb, and the
    /// one a checkpoint mismatch would show (§7.5).
    fn watch_late_changes(&mut self, now: f64) {
        let Some(m) = self.mat.as_mut() else { return };
        let mut found = vec![];
        for (&r, root) in &m.sent_checkpoint {
            if !m.late_logged.contains(&r) && m.root(r as usize) != *root {
                found.push(r);
            }
        }
        let mut notes = vec![];
        for r in found {
            m.late_logged.insert(r);
            let what = format!("round {r} resolved differently after this seat's checkpoint");
            m.late_changes.push(what.clone());
            notes.push(what);
        }
        for what in notes {
            self.note(now, "", "late-change", what);
        }
    }

    /// A predecessor's frame that reached one relay and is missing from
    /// another after `CARRY_AFTER_MS` is republished there, in one event per
    /// relay (§4.3.2 rule 6).
    fn carry(&mut self, now: f64) {
        let Some(m) = self.mat.as_ref() else { return };
        let Some(seat) = m.my_seat else { return };
        let n = m.n() as u16;
        let pred = (seat + n - 1) % n;
        let mut per_relay: BTreeMap<usize, Vec<FrameKey>> = BTreeMap::new();
        for (key, held_at) in &m.first_held {
            if key.seat != pred || now - held_at < CARRY_AFTER_MS {
                continue;
            }
            let Some(on) = m.on_relay.get(key) else { continue };
            if on.is_empty() {
                continue;
            }
            for (j, l) in self.relays.iter().enumerate() {
                if l.usable && l.status == Status::Open && !on.contains(&j) && !m.carried.contains(&(*key, j)) {
                    per_relay.entry(j).or_default().push(*key);
                }
            }
        }
        let Some(room) = self.room_topic() else { return };
        // The predecessor's signed acceptance, the same way: without it a
        // reader of that relay never sees the match start.
        let pred_key = m.seats[pred as usize];
        let mut accept_to = vec![];
        if let (Some((content, held_at)), Some(on)) =
            (self.accept_content.get(&pred_key), self.accept_on_relay.get(&pred_key))
        {
            if !on.is_empty() && now - held_at >= CARRY_AFTER_MS {
                for (j, l) in self.relays.iter().enumerate() {
                    if l.usable
                        && l.status == Status::Open
                        && !on.contains(&j)
                        && !self.accept_carried.contains(&(pred_key, j))
                    {
                        accept_to.push((j, content.clone()));
                    }
                }
            }
        }
        for (j, content) in accept_to {
            self.accept_carried.insert((pred_key, j));
            let ev = make_event(&self.nostr, std::slice::from_ref(&room), content, 0, Self::secs(now), &mut self.rng);
            self.accept_ids.insert(ev.id.clone(), pred_key);
            let url = self.relays[j].url.clone();
            self.note(now, &url, "carry", format!("seat {pred}'s acceptance"));
            self.publish(
                Post {
                    ev,
                    priority: 4,
                    replace: None,
                    what: format!("carry accept {pred}"),
                    only: Some(j),
                    frames: vec![],
                },
                now,
            );
        }
        for (j, keys) in per_relay {
            let m = self.mat.as_mut().unwrap();
            let frames: Vec<[u8; FRAME_LEN]> = keys.iter().take(MAX_FRAMES_PER_EVENT).map(|k| m.frames[k]).collect();
            for k in keys.iter().take(MAX_FRAMES_PER_EVENT) {
                m.carried.insert((*k, j));
            }
            let taken: Vec<FrameKey> = keys.into_iter().take(MAX_FRAMES_PER_EVENT).collect();
            let ev = make_event(
                &self.nostr,
                std::slice::from_ref(&room),
                Self::frames_content(&frames),
                0,
                Self::secs(now),
                &mut self.rng,
            );
            let url = self.relays[j].url.clone();
            self.note(now, &url, "carry", format!("{} frames of seat {pred}", frames.len()));
            self.publish(
                Post {
                    ev,
                    priority: 4,
                    replace: None,
                    what: format!("carry seat {pred}"),
                    only: Some(j),
                    frames: taken,
                },
                now,
            );
        }
    }

    /// A public room announces itself while it fills (§5.3).
    fn announce(&mut self, now: f64) {
        let (true, Some(link), Some(room)) = (self.host, self.link, &self.room) else { return };
        if !link.params.public || room.started || now < self.last_announce + ANNOUNCE_EVERY_MS {
            return;
        }
        self.last_announce = now;
        let msg = json!({
            "type": "announce",
            "room": hex(&link.room),
            "link": link.encode(),
            "seq": room.seq,
            "filled": room.seats.len(),
            "seats": link.params.seats,
            "round_s": link.params.round_s,
            "rounds": link.params.rounds,
            "host": room.seats.first().map(|m| m.name.clone()),
        });
        self.send_lobby(msg, vec![topic("open")], 5, 0, Some("announce"), "announce");
    }

    // --- what the page shows, and what a player sends back ----------------------

    pub fn view(&self, now: f64) -> Value {
        let room = self.room.as_ref().map(|r| {
            let place = if r.seats.iter().any(|m| m.key == self.me) {
                "seat"
            } else if r.queue.iter().any(|m| m.key == self.me) {
                "queue"
            } else if r.kicked.contains(&self.me) {
                "kicked"
            } else {
                "none"
            };
            json!({
                "seq": r.seq,
                "seats": r.seats.iter().map(|m| json!({"key": hex(&m.key), "name": m.name, "me": m.key == self.me})).collect::<Vec<_>>(),
                "queue": r.queue.iter().map(|m| json!({"key": hex(&m.key), "name": m.name, "me": m.key == self.me})).collect::<Vec<_>>(),
                "started": r.started,
                "place": place,
            })
        });
        let mat = self.mat.as_ref().map(|m| {
            let r = m.current;
            let rd = &m.rounds[r];
            let phase = if m.started_at.is_none() {
                "waiting for every seat to sign"
            } else if m.done() {
                "done"
            } else if rd.gate_at.is_none() {
                "commit"
            } else {
                "reveal"
            };
            let opened = rd.opened_at.unwrap_or(now);
            let history: Vec<Value> = (0..m.rounds.len())
                .filter(|&i| m.rounds[i].completed_at.is_some())
                .map(|i| {
                    let mut groups: BTreeMap<String, Vec<u16>> = BTreeMap::new();
                    for (s, root) in &m.rounds[i].checkpoints {
                        groups.entry(hex(&root[..6])).or_default().push(*s);
                    }
                    let mine = hex(&m.root(i)[..6]);
                    json!({
                        "round": i,
                        "applied": m.applied(i).iter().map(|o| if *o == PASS { Value::from("pass") } else { Value::from(o.card) }).collect::<Vec<_>>(),
                        "defaulted": (0..m.n() as u16).filter(|&s| m.defaulted(i, s)).collect::<Vec<_>>(),
                        "mine": mine,
                        "checkpoints": groups,
                        "agree": m.rounds[i].checkpoints.len() == m.n() && m.rounds[i].checkpoints.values().all(|c| hex(&c[..6]) == mine),
                    })
                })
                .collect();
            json!({
                "session": hex(&m.session),
                "names": m.names,
                "my_seat": m.my_seat,
                "accepted": m.accepts.keys().collect::<Vec<_>>(),
                "accept_sent": m.accept_sent,
                "params": params_json(&m.params),
                "phase": phase,
                "round": r,
                "rounds": m.rounds.len(),
                "floor_left_s": ((opened + m.floor_ms() - now) / 1000.0).max(0.0).ceil(),
                "auto": m.auto,
                "chosen": m.chosen,
                "my_order": rd.mine.map(|(o, _)| if o == PASS { Value::from("pass") } else { Value::from(o.card) }),
                "seats": (0..m.n() as u16).map(|s| json!({
                    "name": m.names[s as usize],
                    "committed": rd.commits.contains_key(&s),
                    "revealed": m.valid_reveal(r, s).is_some(),
                    "defaulted": m.defaulted(r, s),
                    "kicked_from": m.kicks.get(&s),
                })).collect::<Vec<_>>(),
                "history": history,
                "equivocations": m.equivocations,
                "late_changes": m.late_changes,
                "final_sent": m.final_sent,
            })
        });
        let relays: Vec<Value> = self
            .relays
            .iter()
            .map(|l| {
                let s = &l.stats;
                json!({
                    "url": l.url,
                    "status": format!("{:?}", l.status).to_lowercase(),
                    "usable": l.usable,
                    "note": l.note,
                    "sent": s.sent, "accepted": s.accepted, "rate_limited": s.rate_limited, "rejected": s.rejected,
                    "timeouts": s.timeouts, "closes": s.closes, "sub_closed": s.sub_closed,
                    "queued": l.queued(),
                    "backoff_s": ((l.write_until - now) / 1000.0).max(0.0).ceil(),
                    "last_hint": s.last_hint,
                    "last_reason": s.last_reason,
                })
            })
            .collect();
        let open: Vec<Value> = self
            .open_rooms
            .values()
            .filter(|(_, seen)| now - seen < ANNOUNCE_STALE_MS)
            .map(|(m, seen)| {
                let mut m = m.clone();
                m["age_s"] = json!(((now - seen) / 1000.0).round());
                m
            })
            .collect();
        let log: Vec<String> = self
            .log
            .iter()
            .rev()
            .take(200)
            .map(|e| format!("{:8.1} {} {} {}", e.t / 1000.0, e.what, e.relay, e.detail))
            .collect();
        json!({
            "mode": format!("{:?}", self.mode).to_lowercase(),
            "me": hex(&self.me),
            "host": self.host,
            "name": self.name,
            "link": self.link.map(|l| l.encode()),
            "params": self.link.map(|l| params_json(&l.params)),
            "join_sent": self.join_sent,
            "room": room,
            "match": mat,
            "relays": relays,
            "open_rooms": open,
            "log": log,
        })
    }

    /// Every relay reply and protocol event, one per line: the data that
    /// settles R-SES16 and R-SES17 (§4.3.2 rule 8). Not part of the transcript.
    pub fn diagnostics_tsv(&self) -> String {
        let mut s = String::from("t_s\trelay\tevent\tdetail\n");
        for e in &self.log {
            let clean = |x: &str| x.replace(['\t', '\n'], " ");
            s.push_str(&format!(
                "{:.3}\t{}\t{}\t{}\n",
                e.t / 1000.0,
                clean(&e.relay),
                clean(&e.what),
                clean(&e.detail)
            ));
        }
        s
    }

    /// The match record (sessions spec §5.4): genesis, the seats' signed
    /// acceptances and every frame held, each frame base64.
    pub fn transcript_json(&self) -> String {
        let Some(m) = &self.mat else { return "null".into() };
        json!({
            "format": "hyades-transcript",
            "version": 0,
            "note": "transport test (T-169): checkpoints are inputs roots, not state roots",
            "genesis": m.genesis,
            "session": hex(&m.session),
            "accepts": m.accepts.values().collect::<Vec<_>>(),
            "frames": m.frames.values().map(|f| b64(f)).collect::<Vec<_>>(),
        })
        .to_string()
    }

    /// Something the host layer saw that the state machine did not: a socket
    /// error, a failed signature, a screen fetch that failed.
    pub fn host_note(&mut self, relay: Option<usize>, what: &str, detail: &str, now: f64) {
        let url = relay.map(|i| self.relays[i].url.clone()).unwrap_or_default();
        self.note(now, &url, what, detail);
    }

    /// This tab's throwaway Nostr key.
    pub fn nostr_pubkey(&self) -> &str {
        &self.nostr.pk
    }
}
