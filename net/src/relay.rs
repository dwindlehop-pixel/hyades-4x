//! One relay, as a state machine with no socket (sessions spec §4.3.1–§4.3.2).
//!
//! Rules implemented here (§4.3.2):
//!
//! 1. A relay's limit is learned from its replies: each relay keeps its own
//!    backoff for writes, for its subscription and for reconnecting.
//! 2. A relay's backoff hint (nostr-protocol/nips#2498, open: an optional
//!    trailing element on `OK` and `CLOSED`) is honored, capped by the caller
//!    at the time left in the current phase. Messages of any length parse;
//!    unknown trailing elements are ignored.
//! 3. Without a hint, wait `BASE_MS`, doubling per consecutive rejection to
//!    `CAP_MS`, each wait drawn ±`JITTER` at random. Absence never means
//!    "retry now".
//! 4. One connection and one subscription per relay, held for the session.
//! 5. Writes leave in priority order, one in flight at a time per relay.

use crate::bytes::Drbg;
use crate::nostr::Event;
use serde_json::Value;

/// *Placeholders*, R-SES17.
pub const BASE_MS: f64 = 15_000.0;
pub const CAP_MS: f64 = 120_000.0;
pub const JITTER: f64 = 0.5;
/// A write with no `OK` after this long is treated as lost.
pub const INFLIGHT_TIMEOUT_MS: f64 = 15_000.0;
/// Even a capped or zero hint waits at least this long, so a relay is never
/// retried in a tight loop.
pub const MIN_WAIT_MS: f64 = 1_000.0;

#[derive(Clone, Debug, Default)]
pub struct Backoff {
    pub failures: u32,
}

impl Backoff {
    /// The wait after a rejection: the relay's hint if it gave one, else our
    /// own jittered doubling; either way capped at `cap` and floored at
    /// `MIN_WAIT_MS`.
    pub fn fail(&mut self, hint_ms: Option<f64>, cap: f64, rng: &mut Drbg) -> f64 {
        self.failures += 1;
        let wait = match hint_ms {
            Some(h) => h,
            None => {
                let nominal = (BASE_MS * 2f64.powi(self.failures as i32 - 1)).min(CAP_MS);
                nominal * (1.0 - JITTER + 2.0 * JITTER * rng.unit())
            }
        };
        wait.min(cap).max(MIN_WAIT_MS)
    }

    pub fn succeed(&mut self) {
        self.failures = 0;
    }
}

/// The machine-readable prefix of an `OK` or `CLOSED` message.
pub fn prefix_of(message: &str) -> &str {
    match message.find(':') {
        Some(i) if message[..i].bytes().all(|b| b.is_ascii_lowercase() || b == b'-') => &message[..i],
        _ => "",
    }
}

/// A hint is a non-negative integer of milliseconds or its decimal string
/// (the unit the proposal settled on during review). Anything else is
/// ignored rather than guessed at; the raw value is kept for the log.
pub fn hint_of(v: Option<&Value>) -> Option<f64> {
    match v? {
        Value::Number(n) => n.as_u64().map(|x| x as f64),
        Value::String(s) if !s.is_empty() && s.len() <= 10 && s.bytes().all(|b| b.is_ascii_digit()) => s.parse().ok(),
        _ => None,
    }
}

/// A relay's answer to one write (NIP-01 `OK`), with nips#2498's hint.
#[derive(Clone, Debug, PartialEq)]
pub struct OkReply {
    pub id: String,
    pub ok: bool,
    pub message: String,
    pub hint_ms: Option<f64>,
    pub hint_raw: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum RelayMsg {
    Ok(OkReply),
    Closed { sub: String, message: String, hint_ms: Option<f64>, hint_raw: Option<String> },
    Event { sub: String, event: Event },
    Eose,
    Notice(String),
    Auth,
}

/// Parses a relay message of any array length. `None` for anything
/// malformed; the caller logs and drops it.
pub fn parse(raw: &str) -> Option<RelayMsg> {
    if raw.len() > 1 << 20 {
        return None;
    }
    let v: Vec<Value> = serde_json::from_str(raw).ok()?;
    let text = |i: usize| v.get(i).and_then(Value::as_str).unwrap_or("").to_string();
    let raw_at = |i: usize| v.get(i).map(|x| x.to_string());
    match v.first()?.as_str()? {
        "OK" => Some(RelayMsg::Ok(OkReply {
            id: v.get(1)?.as_str()?.to_string(),
            ok: v.get(2)?.as_bool()?,
            message: text(3),
            hint_ms: hint_of(v.get(4)),
            hint_raw: raw_at(4),
        })),
        "CLOSED" => Some(RelayMsg::Closed {
            sub: v.get(1)?.as_str()?.to_string(),
            message: text(2),
            hint_ms: hint_of(v.get(3)),
            hint_raw: raw_at(3),
        }),
        "EVENT" => Some(RelayMsg::Event {
            sub: v.get(1)?.as_str()?.to_string(),
            event: serde_json::from_value(v.get(2)?.clone()).ok()?,
        }),
        "EOSE" => Some(RelayMsg::Eose),
        "NOTICE" => Some(RelayMsg::Notice(text(1))),
        "AUTH" => Some(RelayMsg::Auth),
        _ => None,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Idle,
    Connecting,
    Open,
}

/// A write waiting for its turn. Lower `priority` leaves first (§4.3.2 rule 5).
#[derive(Clone, Debug)]
pub struct Queued {
    pub priority: u8,
    pub seq: u64,
    pub event_id: String,
    pub text: String,
    /// A later write with the same key supersedes this one (the host's room state).
    pub replace: Option<String>,
    pub what: String,
}

#[derive(Clone, Debug, Default)]
pub struct Stats {
    pub sent: u32,
    pub accepted: u32,
    pub rate_limited: u32,
    pub rejected: u32,
    pub timeouts: u32,
    pub connects: u32,
    pub closes: u32,
    pub sub_closed: u32,
    pub last_hint: Option<String>,
    pub last_reason: String,
}

pub enum Action {
    Connect,
    Send(String),
}

/// What a reply to one of our writes meant.
pub enum Outcome {
    Accepted { queued: Queued, latency_ms: f64 },
    RateLimited { what: String, wait_ms: f64, hint_raw: Option<String>, message: String },
    Rejected { what: String, message: String },
    NotOurs,
}

pub struct Link {
    pub url: String,
    pub status: Status,
    /// False when the NIP-11 screen excluded the relay (§4.3.1 rule 3).
    pub usable: bool,
    pub note: String,
    next_connect_at: f64,
    conn_backoff: Backoff,
    pub write_until: f64,
    write_backoff: Backoff,
    queue: Vec<Queued>,
    inflight: Option<(Queued, f64)>,
    sub_until: f64,
    sub_backoff: Backoff,
    sub_wanted: Option<String>,
    sub_sent: bool,
    pub stats: Stats,
}

impl Link {
    pub fn new(url: String) -> Link {
        Link {
            url,
            status: Status::Idle,
            usable: true,
            note: String::new(),
            next_connect_at: 0.0,
            conn_backoff: Backoff::default(),
            write_until: 0.0,
            write_backoff: Backoff::default(),
            queue: Vec::new(),
            inflight: None,
            sub_until: 0.0,
            sub_backoff: Backoff::default(),
            sub_wanted: None,
            sub_sent: false,
            stats: Stats::default(),
        }
    }

    pub fn queued(&self) -> usize {
        self.queue.len() + self.inflight.is_some() as usize
    }

    pub fn enqueue(&mut self, q: Queued) {
        if let Some(key) = &q.replace {
            self.queue.retain(|x| x.replace.as_ref() != Some(key));
        }
        self.queue.push(q);
    }

    /// The subscription this relay should hold; sent (once) when it changes.
    pub fn subscribe(&mut self, req: String) {
        if self.sub_wanted.as_ref() != Some(&req) {
            self.sub_wanted = Some(req);
            self.sub_sent = false;
        }
    }

    pub fn opened(&mut self) {
        self.status = Status::Open;
        self.conn_backoff.succeed();
        self.stats.connects += 1;
        self.sub_sent = false;
    }

    /// The socket closed: requeue what was in flight and reconnect after a backoff.
    pub fn closed(&mut self, now: f64, rng: &mut Drbg) -> f64 {
        self.status = Status::Idle;
        self.stats.closes += 1;
        self.sub_sent = false;
        if let Some((q, _)) = self.inflight.take() {
            self.queue.push(q);
        }
        let wait = self.conn_backoff.fail(None, CAP_MS, rng);
        self.next_connect_at = now + wait;
        wait
    }

    pub fn poll(&mut self, now: f64, rng: &mut Drbg, log: &mut Vec<(String, String)>) -> Vec<Action> {
        let mut out = Vec::new();
        if self.status == Status::Idle && now >= self.next_connect_at {
            self.status = Status::Connecting;
            out.push(Action::Connect);
            return out;
        }
        if self.status != Status::Open {
            return out;
        }
        if !self.sub_sent && now >= self.sub_until {
            if let Some(req) = &self.sub_wanted {
                out.push(Action::Send(req.clone()));
                self.sub_sent = true;
            }
        }
        if let Some((q, sent_at)) = &self.inflight {
            if now - sent_at >= INFLIGHT_TIMEOUT_MS {
                self.stats.timeouts += 1;
                log.push(("write-timeout".into(), q.what.clone()));
                let (q, _) = self.inflight.take().expect("checked");
                self.queue.push(q);
                self.write_until = now + self.write_backoff.fail(None, CAP_MS, rng);
            }
        }
        if self.usable && self.inflight.is_none() && now >= self.write_until && !self.queue.is_empty() {
            let best =
                (0..self.queue.len()).min_by_key(|&i| (self.queue[i].priority, self.queue[i].seq)).expect("non-empty");
            let q = self.queue.remove(best);
            out.push(Action::Send(q.text.clone()));
            self.stats.sent += 1;
            self.inflight = Some((q, now));
        }
        out
    }

    /// The earliest time `poll` could do something it cannot do now.
    pub fn next_wake(&self) -> Option<f64> {
        match self.status {
            Status::Idle => Some(self.next_connect_at),
            Status::Connecting => None,
            Status::Open => {
                let mut t: Option<f64> = None;
                let mut at = |x: f64| t = Some(t.map_or(x, |y: f64| y.min(x)));
                if let Some((_, sent)) = &self.inflight {
                    at(sent + INFLIGHT_TIMEOUT_MS);
                } else if self.usable && !self.queue.is_empty() {
                    at(self.write_until);
                }
                if !self.sub_sent && self.sub_wanted.is_some() {
                    at(self.sub_until);
                }
                t
            }
        }
    }

    pub fn on_ok(&mut self, reply: OkReply, cap: f64, now: f64, rng: &mut Drbg) -> Outcome {
        let OkReply { id, ok, message, hint_ms, hint_raw } = reply;
        let Some((q, sent_at)) = self.inflight.take_if(|(q, _)| q.event_id == id) else {
            return Outcome::NotOurs;
        };
        let prefix = prefix_of(&message);
        if ok || prefix == "duplicate" {
            self.write_backoff.succeed();
            self.stats.accepted += 1;
            return Outcome::Accepted { latency_ms: now - sent_at, queued: q };
        }
        self.stats.last_reason = message.clone();
        if prefix == "rate-limited" {
            self.stats.rate_limited += 1;
            self.stats.last_hint = hint_raw.clone();
            let wait = self.write_backoff.fail(hint_ms, cap, rng);
            self.write_until = now + wait;
            let what = q.what.clone();
            self.queue.push(q);
            return Outcome::RateLimited { what, wait_ms: wait, hint_raw, message };
        }
        self.stats.rejected += 1;
        Outcome::Rejected { what: q.what, message }
    }

    /// The relay closed our subscription. Re-send it after a backoff, the
    /// relay's hint if it gave one.
    pub fn on_closed(&mut self, hint_ms: Option<f64>, cap: f64, now: f64, rng: &mut Drbg) -> f64 {
        self.stats.sub_closed += 1;
        self.sub_sent = false;
        let wait = self.sub_backoff.fail(hint_ms, cap, rng);
        self.sub_until = now + wait;
        wait
    }

    pub fn on_eose(&mut self) {
        self.sub_backoff.succeed();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn q(priority: u8, seq: u64, id: &str) -> Queued {
        Queued { priority, seq, event_id: id.into(), text: format!("[\"EVENT\",{id}]"), replace: None, what: id.into() }
    }

    fn open_link() -> (Link, Drbg) {
        let mut l = Link::new("wss://r".into());
        let mut rng = Drbg::new([1; 32]);
        let mut log = vec![];
        assert!(matches!(l.poll(0.0, &mut rng, &mut log)[..], [Action::Connect]));
        l.opened();
        (l, rng)
    }

    fn reply(id: &str, ok: bool, message: &str, hint_ms: Option<f64>) -> OkReply {
        OkReply { id: id.into(), ok, message: message.into(), hint_ms, hint_raw: hint_ms.map(|h| h.to_string()) }
    }

    #[test]
    fn parses_ok_of_any_length_and_reads_the_hint() {
        let m = parse(r#"["OK","ab",false,"rate-limited: slow down",2500]"#).unwrap();
        assert!(matches!(m, RelayMsg::Ok(OkReply { ok: false, hint_ms: Some(h), .. }) if h == 2500.0));
        let m = parse(r#"["OK","ab",false,"rate-limited: slow down","2500",{"later":"field"}]"#).unwrap();
        assert!(matches!(m, RelayMsg::Ok(OkReply { hint_ms: Some(h), .. }) if h == 2500.0));
        let m = parse(r#"["OK","ab",false,"rate-limited: slow down",-4]"#).unwrap();
        assert!(matches!(m, RelayMsg::Ok(OkReply { hint_ms: None, hint_raw: Some(_), .. })));
        let m = parse(r#"["OK","ab",true]"#).unwrap();
        assert!(matches!(m, RelayMsg::Ok(OkReply { ok: true, hint_ms: None, .. })));
        let m = parse(r#"["CLOSED","h","rate-limited: easy",900]"#).unwrap();
        assert!(matches!(m, RelayMsg::Closed { hint_ms: Some(h), .. } if h == 900.0));
        assert!(parse(r#"["OK",1,true]"#).is_none());
        assert!(parse("not json").is_none());
        assert!(parse(r#"["WHAT"]"#).is_none());
        assert_eq!(prefix_of("rate-limited: x"), "rate-limited");
        assert_eq!(prefix_of("Hello: x"), "");
    }

    #[test]
    fn backoff_doubles_with_jitter_to_the_cap_and_honors_hints() {
        let mut rng = Drbg::new([2; 32]);
        let mut b = Backoff::default();
        let waits: Vec<f64> = (0..6).map(|_| b.fail(None, f64::INFINITY, &mut rng)).collect();
        for (i, w) in waits.iter().enumerate() {
            let nominal = (BASE_MS * 2f64.powi(i as i32)).min(CAP_MS);
            assert!(*w >= nominal * (1.0 - JITTER) && *w <= nominal * (1.0 + JITTER), "wait {i}: {w}");
        }
        assert_eq!(b.fail(Some(3_000.0), f64::INFINITY, &mut rng), 3_000.0);
        assert_eq!(b.fail(Some(500_000.0), 40_000.0, &mut rng), 40_000.0, "a hint is capped");
        assert_eq!(b.fail(Some(0.0), f64::INFINITY, &mut rng), MIN_WAIT_MS, "never a tight loop");
        b.succeed();
        assert_eq!(b.failures, 0);
    }

    #[test]
    fn a_rate_limited_write_waits_out_the_hint_and_goes_again() {
        let (mut l, mut rng) = open_link();
        let mut log = vec![];
        l.enqueue(q(1, 1, "a"));
        assert!(matches!(&l.poll(10.0, &mut rng, &mut log)[..], [Action::Send(_)]));
        let o = l.on_ok(reply("a", false, "rate-limited: slow down", Some(30_000.0)), CAP_MS, 20.0, &mut rng);
        assert!(matches!(o, Outcome::RateLimited { wait_ms, .. } if wait_ms == 30_000.0));
        assert!(l.poll(29_000.0, &mut rng, &mut log).is_empty(), "nothing before the hint runs out");
        assert_eq!(l.next_wake(), Some(30_020.0));
        assert!(matches!(&l.poll(30_020.0, &mut rng, &mut log)[..], [Action::Send(_)]));
        assert!(matches!(l.on_ok(reply("a", true, "", None), CAP_MS, 30_100.0, &mut rng), Outcome::Accepted { .. }));
        assert_eq!((l.stats.rate_limited, l.stats.accepted), (1, 1));
    }

    #[test]
    fn urgent_writes_leave_first_and_room_state_is_replaced() {
        let (mut l, mut rng) = open_link();
        let mut log = vec![];
        l.enqueue(Queued { replace: Some("room".into()), ..q(3, 1, "s1") });
        l.enqueue(q(4, 2, "carry"));
        l.enqueue(Queued { replace: Some("room".into()), ..q(3, 3, "s2") });
        l.enqueue(q(0, 4, "reveal"));
        let mut order = vec![];
        for t in 1..=3 {
            match &l.poll(t as f64, &mut rng, &mut log)[..] {
                [Action::Send(_)] => {}
                _ => panic!("expected a send"),
            }
            let id = l.inflight.as_ref().unwrap().0.event_id.clone();
            l.on_ok(reply(&id, true, "", None), CAP_MS, t as f64, &mut rng);
            order.push(id);
        }
        assert_eq!(order, ["reveal", "s2", "carry"]);
    }

    #[test]
    fn a_lost_reply_and_a_dropped_socket_requeue_the_write() {
        let (mut l, mut rng) = open_link();
        let mut log = vec![];
        l.enqueue(q(1, 1, "a"));
        l.poll(0.0, &mut rng, &mut log);
        assert!(l.poll(INFLIGHT_TIMEOUT_MS + 1.0, &mut rng, &mut log).is_empty());
        assert_eq!(l.stats.timeouts, 1);
        assert_eq!(l.queued(), 1);
        let wait = l.closed(20_000.0, &mut rng);
        assert!(wait >= BASE_MS * (1.0 - JITTER));
        assert!(l.poll(20_001.0, &mut rng, &mut log).is_empty(), "no reconnect before the backoff");
        assert!(matches!(l.poll(20_000.0 + wait, &mut rng, &mut log)[..], [Action::Connect]));
    }
}
