//! Matches played through simulated Nostr relays on a simulated clock
//! (sessions spec §4.3.1–§4.3.2, §5.1, netcode §5).
//!
//! Each relay stores events, serves subscriptions by topic, and can be told
//! to rate-limit writes per client address — with or without nips#2498's
//! backoff hint — or to refuse one Nostr key outright. Messages take a fixed
//! latency. Seats sign with ed25519-dalek here where the browser uses
//! WebCrypto; the state machine cannot tell the difference.

use ed25519_dalek::{Signer, SigningKey};
use hyades_net::link::Params;
use hyades_net::session::{Client, Output};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

const LATENCY_MS: f64 = 40.0;

#[derive(Clone, Default)]
struct Policy {
    /// Writes allowed per second per address, and the bucket's size.
    rate: Option<(f64, f64)>,
    /// The nips#2498 hint sent with a rate-limited reply, ms.
    hint_ms: Option<u64>,
    /// Nostr keys whose writes are refused.
    blocked: BTreeSet<String>,
}

struct Relay {
    policy: Policy,
    events: Vec<Value>,
    subs: BTreeMap<usize, Vec<String>>,
    buckets: BTreeMap<usize, (f64, f64)>,
    /// When each address was last told to wait, and until when.
    told_until: BTreeMap<usize, f64>,
    violations: u32,
    rate_limited: u32,
}

impl Relay {
    fn new(policy: Policy) -> Relay {
        Relay {
            policy,
            events: vec![],
            subs: BTreeMap::new(),
            buckets: BTreeMap::new(),
            told_until: BTreeMap::new(),
            violations: 0,
            rate_limited: 0,
        }
    }
}

enum Msg {
    ToRelay { relay: usize, client: usize, text: String },
    ToClient { client: usize, link: usize, text: String },
    Opened { client: usize, link: usize },
}

struct Seat {
    core: Client,
    sk: SigningKey,
    /// The global relay index behind each of this client's relays.
    links: Vec<usize>,
    address: usize,
    alive: bool,
}

struct Net {
    t: f64,
    relays: Vec<Relay>,
    clients: Vec<Seat>,
    queue: VecDeque<(f64, Msg)>,
}

fn topics_of(e: &Value) -> Vec<String> {
    e["tags"].as_array().unwrap().iter().filter(|t| t[0] == "t").map(|t| t[1].as_str().unwrap().to_string()).collect()
}

impl Net {
    fn new(relays: Vec<Policy>) -> Net {
        Net {
            t: 1_800_000_000_000.0,
            relays: relays.into_iter().map(Relay::new).collect(),
            clients: vec![],
            queue: VecDeque::new(),
        }
    }

    fn add(&mut self, links: Vec<usize>, address: usize) -> usize {
        let i = self.clients.len();
        let sk = SigningKey::from_bytes(&[i as u8 + 11; 32]);
        let urls = links.iter().map(|r| format!("wss://relay{r}.test")).collect();
        let core = Client::new(urls, sk.verifying_key().to_bytes(), [i as u8 + 101; 32], self.t);
        self.clients.push(Seat { core, sk, links, address, alive: true });
        i
    }

    fn send(&mut self, at: f64, m: Msg) {
        let pos = self.queue.iter().position(|(t, _)| *t > at).unwrap_or(self.queue.len());
        self.queue.insert(pos, (at, m));
    }

    fn deliver(&mut self, relay: usize, client: usize, text: String) {
        let Some(link) = self.clients[client].links.iter().position(|&r| r == relay) else { return };
        self.send(self.t + LATENCY_MS, Msg::ToClient { client, link, text });
    }

    fn relay_receive(&mut self, r: usize, client: usize, text: &str) {
        let v: Vec<Value> = serde_json::from_str(text).unwrap();
        match v[0].as_str().unwrap() {
            "REQ" => {
                let topics: Vec<String> =
                    v[2]["#t"].as_array().unwrap().iter().map(|x| x.as_str().unwrap().to_string()).collect();
                self.relays[r].subs.insert(client, topics.clone());
                let stored: Vec<Value> = self.relays[r]
                    .events
                    .iter()
                    .filter(|e| topics_of(e).iter().any(|t| topics.contains(t)))
                    .cloned()
                    .collect();
                for e in stored {
                    self.deliver(r, client, json!(["EVENT", "h", e]).to_string());
                }
                self.deliver(r, client, json!(["EOSE", "h"]).to_string());
            }
            "EVENT" => {
                let e = v[1].clone();
                let id = e["id"].as_str().unwrap().to_string();
                let addr = self.clients[client].address;
                let now = self.t;
                let relay = &mut self.relays[r];
                if relay.told_until.get(&addr).is_some_and(|&u| now < u) {
                    relay.violations += 1;
                }
                if relay.policy.blocked.contains(e["pubkey"].as_str().unwrap()) {
                    self.deliver(r, client, json!(["OK", id, false, "blocked: not on this relay"]).to_string());
                    return;
                }
                if let Some((rate, burst)) = relay.policy.rate {
                    let (tokens, last) = relay.buckets.get(&addr).copied().unwrap_or((burst, now));
                    let tokens = (tokens + (now - last) / 1000.0 * rate).min(burst);
                    if tokens < 1.0 {
                        relay.buckets.insert(addr, (tokens, now));
                        relay.rate_limited += 1;
                        let refill_ms = (1.0 - tokens) / rate * 1000.0;
                        let reply = match relay.policy.hint_ms {
                            Some(h) => {
                                relay.told_until.insert(addr, now + h as f64);
                                let _ = refill_ms;
                                json!(["OK", id, false, "rate-limited: slow down", h])
                            }
                            None => json!(["OK", id, false, "rate-limited: slow down"]),
                        };
                        self.deliver(r, client, reply.to_string());
                        return;
                    }
                    relay.buckets.insert(addr, (tokens - 1.0, now));
                }
                if relay.events.iter().any(|x| x["id"] == e["id"]) {
                    self.deliver(r, client, json!(["OK", id, true, "duplicate: have it"]).to_string());
                    return;
                }
                relay.events.push(e.clone());
                let topics = topics_of(&e);
                let subscribers: Vec<usize> = relay
                    .subs
                    .iter()
                    .filter(|(_, ts)| ts.iter().any(|t| topics.contains(t)))
                    .map(|(c, _)| *c)
                    .collect();
                self.deliver(r, client, json!(["OK", id, true, ""]).to_string());
                for c in subscribers {
                    self.deliver(r, c, json!(["EVENT", "h", e]).to_string());
                }
            }
            _ => panic!("unexpected client message {text}"),
        }
    }

    fn pump(&mut self, c: usize) {
        loop {
            let t = self.t;
            let outs = self.clients[c].core.poll(t);
            if outs.is_empty() {
                return;
            }
            for o in outs {
                if !self.clients[c].alive {
                    continue;
                }
                match o {
                    Output::Connect(link, _) => self.send(t + LATENCY_MS, Msg::Opened { client: c, link }),
                    Output::Send(link, text) => {
                        let relay = self.clients[c].links[link];
                        self.send(t + LATENCY_MS, Msg::ToRelay { relay, client: c, text });
                    }
                    Output::Sign(req, bytes) => {
                        let sig = self.clients[c].sk.sign(&bytes).to_bytes();
                        self.clients[c].core.signed(req, sig, t);
                    }
                }
            }
        }
    }

    /// Runs until `until` (simulated ms after the start) or until `done` holds.
    fn run(&mut self, until_ms: f64, mut done: impl FnMut(&mut Net) -> bool) -> bool {
        let end = self.t + until_ms;
        while self.t < end {
            for c in 0..self.clients.len() {
                self.pump(c);
            }
            if done(self) {
                return true;
            }
            let wake = (0..self.clients.len())
                .filter(|&c| self.clients[c].alive)
                .filter_map(|c| self.clients[c].core.next_wake(self.t))
                .fold(f64::INFINITY, f64::min);
            let next_msg = self.queue.front().map_or(f64::INFINITY, |(t, _)| *t);
            let next = wake.min(next_msg).min(end);
            self.t = next.max(self.t + 1.0).min(end.max(self.t + 1.0));
            while self.queue.front().is_some_and(|(t, _)| *t <= self.t) {
                let (_, m) = self.queue.pop_front().unwrap();
                match m {
                    Msg::ToRelay { relay, client, text } => self.relay_receive(relay, client, &text),
                    Msg::ToClient { client, link, text } => {
                        if self.clients[client].alive {
                            let t = self.t;
                            self.clients[client].core.relay_text(link, &text, t);
                        }
                    }
                    Msg::Opened { client, link } => {
                        let t = self.t;
                        self.clients[client].core.relay_opened(link, t);
                    }
                }
            }
        }
        false
    }

    fn view(&self, c: usize) -> Value {
        self.clients[c].core.view(self.t)
    }
}

/// Seats a room of `seats` (the host first) plus one queued spectator,
/// starts it, and has every seat sign the genesis.
fn seat_a_room(net: &mut Net, seats: usize, params: Params) -> Vec<usize> {
    let now = net.t;
    let link = net.clients[0].core.create(params, "host", now);
    for c in 1..net.clients.len() {
        net.clients[c].core.open_link(&link, now).unwrap();
    }
    let mut joined = BTreeSet::new();
    let ok = net.run(60_000.0, |n| {
        for c in 1..n.clients.len() {
            if !joined.contains(&c) && n.view(c)["room"].is_object() {
                let t = n.t;
                n.clients[c].core.join(&format!("p{c}"), t).unwrap();
                joined.insert(c);
            }
        }
        let r = &n.view(0)["room"];
        r["seats"].as_array().unwrap().len() == seats && r["queue"].as_array().unwrap().len() == n.clients.len() - seats
    });
    assert!(ok, "the room filled");
    let t = net.t;
    net.clients[0].core.start(t).unwrap();
    let ok = net.run(60_000.0, |n| {
        for c in 0..n.clients.len() {
            let v = n.view(c);
            if v["match"]["my_seat"].is_number() && v["match"]["accept_sent"] == false {
                let t = n.t;
                n.clients[c].core.accept(t).unwrap();
            }
        }
        (0..n.clients.len())
            .all(|c| n.view(c)["match"]["phase"] != "waiting for every seat to sign" && n.view(c)["match"].is_object())
    });
    if !ok {
        for c in 0..net.clients.len() {
            eprintln!("client {c}: {}\n{}", net.view(c)["match"], net.clients[c].core.diagnostics_tsv());
        }
    }
    assert!(ok, "every client saw every seat sign");
    (0..net.clients.len()).collect()
}

fn finished(n: &mut Net, live: &[usize]) -> bool {
    live.iter().all(|&c| {
        let v = n.view(c);
        let m = &v["match"];
        m["phase"] == "done" && (m["my_seat"].is_null() || m["final_sent"] == true)
    })
}

fn quick() -> Params {
    Params { seats: 3, round_s: 2, rounds: 4, patience_s: 6, public: false }
}

fn diag(n: &Net, c: usize, what: &str) -> usize {
    n.clients[c].core.diagnostics_tsv().lines().filter(|l| l.split('\t').nth(2) == Some(what)).count()
}

/// Every seat's checkpoint for every round, as each live client holds them.
fn checkpoints_agree(n: &Net, live: &[usize]) {
    let views: Vec<Value> = live.iter().map(|&c| n.view(c)).collect();
    let rounds = views[0]["match"]["rounds"].as_u64().unwrap() as usize;
    for r in 0..rounds {
        let mines: BTreeSet<String> =
            views.iter().map(|v| v["match"]["history"][r]["mine"].as_str().unwrap().to_string()).collect();
        assert_eq!(mines.len(), 1, "round {r}: every client resolved the same orders: {mines:?}");
    }
    for v in &views {
        assert!(v["match"]["equivocations"].as_array().unwrap().is_empty());
        assert!(v["match"]["late_changes"].as_array().unwrap().is_empty(), "{}", v["match"]["late_changes"]);
    }
}

#[test]
fn a_match_through_generous_relays_resolves_every_round_alike() {
    let mut net = Net::new(vec![Policy::default(), Policy::default()]);
    for c in 0..4 {
        net.add(vec![0, 1], c);
    }
    let all = seat_a_room(&mut net, 3, quick());
    let seats = [0, 1, 2];
    assert!(net.run(10.0 * 60_000.0, |n| finished(n, &all)), "the match finished");
    // Let the final checkpoints arrive everywhere.
    net.run(10_000.0, |_| false);
    checkpoints_agree(&net, &all);
    for &c in &seats {
        let v = net.view(c);
        for h in v["match"]["history"].as_array().unwrap() {
            assert!(h["defaulted"].as_array().unwrap().is_empty(), "nobody timed out on generous relays");
            assert_eq!(h["agree"], true, "every seat signed the same root: {h}");
        }
    }
    let spectator = net.view(3);
    assert_eq!(spectator["room"]["place"], "queue");
    assert!(spectator["match"]["my_seat"].is_null());
    assert_eq!(diag(&net, 0, "rate-limited"), 0);
}

#[test]
fn rate_limits_with_and_without_hints_are_waited_out_and_the_match_still_resolves() {
    let tight_with_hint = Policy { rate: Some((1.0 / 3.0, 3.0)), hint_ms: Some(3_500), ..Default::default() };
    let tight_without = Policy { rate: Some((1.0 / 4.0, 2.0)), ..Default::default() };
    let mut net = Net::new(vec![tight_with_hint, tight_without]);
    for c in 0..3 {
        net.add(vec![0, 1], c);
    }
    let all = seat_a_room(&mut net, 3, Params { patience_s: 30, ..quick() });
    assert!(net.run(30.0 * 60_000.0, |n| finished(n, &all)), "the match finished under rate limits");
    net.run(60_000.0, |_| false);
    checkpoints_agree(&net, &all);
    let limited: u32 = net.relays.iter().map(|r| r.rate_limited).sum();
    assert!(limited > 0, "the limits were exercised");
    assert_eq!(net.relays[0].violations, 0, "no client wrote to a relay before its hint ran out");
    let logged: usize = all.iter().map(|&c| diag(&net, c, "rate-limited")).sum();
    assert_eq!(logged as u32, limited, "every rate-limited reply is in the diagnostics");
    for &c in &all {
        for h in net.view(c)["match"]["history"].as_array().unwrap() {
            assert_eq!(h["agree"], true, "{h}");
        }
    }
}

#[test]
fn a_relay_that_refuses_one_seat_still_gets_its_frames_from_the_carrier() {
    let mut net = Net::new(vec![Policy::default(), Policy::default()]);
    for c in 0..3 {
        net.add(vec![0, 1], c);
    }
    // A spectator that reads only relay 1.
    net.add(vec![1], 3);
    let blocked = net.clients[1].core.nostr_pubkey().to_string();
    net.relays[1].policy.blocked.insert(blocked);
    let all = seat_a_room(&mut net, 3, Params { patience_s: 60, ..quick() });
    assert!(net.run(20.0 * 60_000.0, |n| finished(n, &all)), "the match finished");
    net.run(60_000.0, |_| false);
    // Seat 1's events never reach relay 1, yet the spectator on relay 1 holds
    // every one of seat 1's reveals: seat 2 carried them (§4.3.2 rule 6).
    let spectator = net.view(3);
    for h in spectator["match"]["history"].as_array().unwrap() {
        assert!(h["defaulted"].as_array().unwrap().is_empty(), "the spectator saw seat 1 play: {h}");
    }
    assert!(diag(&net, 2, "carry") > 0, "seat 2 carried for its predecessor");
    assert!(diag(&net, 1, "rejected") > 0, "seat 1 was refused by relay 1");
    checkpoints_agree(&net, &all);
}

#[test]
fn a_seat_that_falls_silent_is_timed_out_and_the_rest_play_on() {
    let mut net = Net::new(vec![Policy::default(), Policy::default()]);
    for c in 0..3 {
        net.add(vec![0, 1], c);
    }
    seat_a_room(&mut net, 3, quick());
    assert!(net.run(5.0 * 60_000.0, |n| n.view(0)["match"]["round"].as_u64() >= Some(1)), "round 0 resolved");
    net.clients[2].alive = false;
    let live = [0usize, 1];
    assert!(net.run(20.0 * 60_000.0, |n| finished(n, &live)), "the live seats finished");
    net.run(30_000.0, |_| false);
    checkpoints_agree(&net, &live);
    let v = net.view(0);
    let defaulted_rounds =
        v["match"]["history"].as_array().unwrap().iter().filter(|h| h["defaulted"] == json!([2])).count();
    assert!(defaulted_rounds >= 2, "seat 2 was defaulted once it fell silent: {}", v["match"]["history"]);
    assert!(diag(&net, 0, "timeout-vote") > 0);
}

#[test]
fn the_host_kicks_and_the_first_queued_spectator_takes_the_seat() {
    let mut net = Net::new(vec![Policy::default()]);
    for c in 0..4 {
        net.add(vec![0], c);
    }
    let now = net.t;
    let link = net.clients[0].core.create(Params { seats: 3, ..quick() }, "host", now);
    for c in 1..4 {
        net.clients[c].core.open_link(&link, now).unwrap();
    }
    let mut joined = BTreeSet::new();
    // Join one at a time so the order of arrival is fixed: p1, p2 seated, p3 queued.
    assert!(net.run(60_000.0, |n| {
        for c in 1..4 {
            let ahead_seen = (1..c).all(|a| joined.contains(&a))
                && n.view(0)["room"]["seats"].as_array().unwrap().len()
                    + n.view(0)["room"]["queue"].as_array().unwrap().len()
                    >= c;
            if !joined.contains(&c) && n.view(c)["room"].is_object() && ahead_seen {
                let t = n.t;
                n.clients[c].core.join(&format!("p{c}"), t).unwrap();
                joined.insert(c);
            }
        }
        n.view(0)["room"]["queue"].as_array().unwrap().len() == 1
    }));
    let p1 = net.view(1)["me"].as_str().unwrap().to_string();
    let t = net.t;
    net.clients[0].core.kick(&p1, t).unwrap();
    assert!(net.run(30_000.0, |n| n.view(3)["room"]["place"] == "seat" && n.view(1)["room"]["place"] == "kicked"));
    assert!(net.clients[1].core.join("again", net.t).is_err(), "a kicked key cannot rejoin");
}
