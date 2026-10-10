# Hyades — sessions, discovery and security (Rev 1)

How a player gets from a link to a seat, how a public game is found, how a
roster restricts the seats, how a match record is verified, and what the
design does and does not detect when a majority of seats colludes. **This spec
amends `Hyades_netcode.md` (Rev 4)**: it replaces the server of netcode §10
with a static site and third-party rendezvous, and it states which attacks are
detectable from which evidence (§7). Netcode's frame, barrier, digest and
determinism rules (§1, §4–§6, §8.1, §9) stand unchanged and are not restated.

Status key: **RATIFIED** — the author's ruling; **OPEN** — a decision not yet
made, with a recommendation where one is attached. Nothing in this spec is
built, and every magnitude in it is a *placeholder* unless it says otherwise.
Calls are flagged **R-SESn**.

---

## 1. The author's rulings

RATIFIED (T-164, the request that opened this spec):

1. **Four entry points.** (a) A **link** pasted in Discord, Reddit or Twitch
   chat: anyone holding it may join until the game is full, and may observe
   after. The link's originator sets the game parameters. At the end,
   participants may rematch, and observers may queue for a seat that a
   participant leaves. (b) **League play**: a predetermined roster; rostered
   players audit the parameters before play; only they may take a seat;
   anyone may observe. (c) **Public practice**: two or three preselected
   modes, and a list of open games in each. (d) **A replay file** from another
   person: verify the game state and outcome, and interrogate the simulation.
2. **No server-side computation is the target; GitHub Pages as the whole
   backend is the acceptable second choice.**
3. **No centralized rank and no matchmaking algorithm.**
4. **The barrier to entry is an itch.io game's**: click a link and play. No
   account, no install, no sign-in.
5. **Game state is managed peer to peer.** A colluding majority of seats may
   succeed in an attack; **a person holding the match record, or an observer,
   must be able to detect that it happened.**

RATIFIED (T-164, the author's ruling on the first draft):

6. **The host can kick a player. Before the game starts, an observer takes
   the kicked player's spot** (§5.1).

RATIFIED (T-164, the author's rulings on the second draft):

7. **No IP address is exposed in chat.** The link is the GitHub Pages site's
   address with a payload that carries no network address, and no message a
   person can read by holding the link carries one either (§4.4, §5.1).
8. **League standings, rankings and results are outside the game design.** The
   project computes none and hosts none. The game's part in league play ends at
   restricting the seats to a roster and handing every participant a
   verifiable match record (§5.2); what an organizer does with records is
   theirs.

RATIFIED (T-164, the author's ruling on the threat model):

9. **The adversary to design against is a very popular streamer's audience.**
   The streamer plays with viewers as participants; the audience is large,
   parasocial, and wants to learn everything it can about the streamer. Every
   participant and every spectator in a streamer's room is that adversary
   (§4.5).

RATIFIED (T-164, the author's ruling on the relay-load estimate):

10. **The recommended fixes are adopted**: the relay rules of §4.3.1, relay-only
    transport as the default for seats (§4.4), and binding a direct
    connection's DTLS fingerprint to the seat key (§4.4). Every magnitude in
    them stays a *placeholder*. §7.5 records what they change in the two
    threat models.

---

## 2. Terms

| term | meaning |
|---|---|
| **seat** | a participant in a match, bound to one Ed25519 public key in the genesis descriptor (netcode §7) |
| **observer** | anyone who runs the simulation on a match's frames and holds no seat. Netcode §3.2.1's two kinds stand: an **observer seat** is a weighted voter in the player overlay; a **spectator** is unweighted and in the gossip tier. In this spec "observer" means a spectator unless it says otherwise |
| **originator** (host) | the person who made a link. Their key signs the room's parameters and, until genesis, the seat table |
| **room** | the pre-genesis state of a match: parameters, joiners, seat table. A room becomes a **session** when every seat has signed the genesis descriptor |
| **rendezvous** | a public publish/subscribe channel used to find peers and exchange WebRTC signaling. Not a game data plane by default (§4.3) |
| **transcript** | genesis descriptor + its signature set + every signed frame a client holds. The verifiable record of a match (netcode §8.4) |
| **replay** | the viewer's JSON recording (`Hyades_interface.md` §3). A *derived* file: a client produces it from a transcript by running the engine. A replay alone verifies nothing |
| **match record** | the file a person hands another person: a transcript, optionally with a replay beside it (§5.4) |

**"Replay file" in ruling 1(d) means the match record**, because only the
transcript can be verified. A replay JSON received alone is shown with an
**unverified** label.

---

## 3. Architecture

```
GitHub Pages (static) ─── client: page, JS, engine.wasm, viewer.wasm, mode table
        │                    (content-addressed by hash, every version kept, §8)
        ▼
browser tab ── Ed25519 identity (WebCrypto) ── engine in a Worker (apply_orders only)
        │
        ├── rendezvous: public Nostr relays (find peers, signaling, room lists)
        ├── WebRTC data channels (frames; netcode §3–§4, §9)
        └── relay transport fallback (frames over the rendezvous, §4.3)
```

**Nothing we operate computes anything.** Pages serves files. The rendezvous
is third-party public infrastructure that carries opaque bytes. STUN uses
public servers. There is no TURN (§4.3). The only optional computation is a
relay a user chooses to run themselves.

**One trust rule covers every transport: authority comes from the Ed25519
signatures inside a payload, never from the transport.** A relay's own event
signature, a WebRTC peer's DTLS identity, and the page's origin authenticate
nothing about the game. This is netcode §3.2's "every relay is untrusted",
extended to the rendezvous and to the lobby.

### 3.1 Why Nostr for the rendezvous — OPEN (R-SES1)

Recommendation: **Nostr relays** ([NIP-01](https://github.com/nostr-protocol/nips/blob/master/01.md)),
publishing to and subscribing from 3–5 relays at once (*placeholder*), with
the relay list pinned in the client build. **A link cannot name a relay**
(§4.5): a relay named by whoever made a link learns the IP address of
everyone who opens it.

- Many independent operators run public relays, and a client can use several,
  so the loss of one relay costs nothing.
- Events carry tags and can be filtered by them, which gives the room list of
  §5.3 without a server. A BitTorrent tracker gives a peer list without
  metadata.
- Expiring events ([NIP-40](https://github.com/nostr-protocol/nips/blob/master/40.md))
  bound how long an abandoned room stays listed, and proof-of-work
  ([NIP-13](https://github.com/nostr-protocol/nips/blob/master/13.md)) prices
  spam (§7.4).
- Nostr signs events with secp256k1 Schnorr, which WebCrypto does not provide,
  so the client links a library for it (permitted: `Hyades_interface.md`
  ruling 14). **The Nostr key is a throwaway per tab**; it is not the game
  identity (the trust rule above).

Prior art: [Trystero](https://github.com/dmotz/trystero) does serverless
WebRTC matchmaking over BitTorrent trackers, Nostr, MQTT and others behind one
interface. The client's rendezvous should be the same shape — `publish(topic,
bytes)`, `subscribe(topic) → bytes` — so a second backend (WebTorrent
trackers) can be added without touching anything above it.

What would settle R-SES1: a prototype measuring, over a week, the fraction of
room joins that complete through the pinned relays, and whether any relay
rate-limits a session's signaling burst at 18 seats. **No such measurement
exists yet.**

---

## 4. Identity, transport and privacy

### 4.1 Identity — OPEN (R-SES2)

Recommendation:

- **Casual and practice play: a fresh Ed25519 key per tab**, generated with
  WebCrypto on page load, non-extractable, kept for the tab's lifetime so a
  rematch (§5.1) keeps the same seat key. No prompt.
- **League play: a persistent key**, stored in IndexedDB, with an explicit
  **backup file** offered at creation. Safari deletes script-writable storage,
  IndexedDB included, after seven days of Safari use without interaction with
  the site ([Apple's 7-day cap](https://docs.didomi.io/releases-and-announcements/announcements/apple-implements-7-day-cap-on-script-writable-storage)),
  so a league key that is never backed up will be lost on some players'
  machines. A player who loses a key sends a new one to whoever makes the
  next roster link (§5.2).
- **A display name is self-chosen and unauthenticated.** The client shows a
  short fingerprint of the key beside it (*placeholder*: 4 words or a glyph),
  so two players named alike are distinguishable.

### 4.2 Transport ladder

A frame is self-authenticating, so every transport below carries the same
144-byte frames and the receive discipline of netcode §4.3 applies unchanged:

1. **WebRTC direct**, on netcode §3.1's circulant overlay.
2. **WebRTC through another seat** — netcode §9.2's flood-with-dedup relay.
3. **The rendezvous itself** (§4.3), for a seat that reaches no peer.

Under §4.4's default (ruling 10), a seat in a link or practice game uses rung 3
alone; the first two rungs apply when every party has opted in to direct
connections.

### 4.3 No TURN; frames over the rendezvous as the fallback — OPEN (R-SES3)

Netcode §3.3 and §10 provision TURN. TURN requires a credential issuer and
metered relay bandwidth, which are server computation and server cost, and
ruling 2 excludes them.

Recommendation: **a seat that reaches no peer publishes and reads its frames
as rendezvous events.** The volume is small because the protocol is
round-barrier lockstep (netcode §1). Computed: one Nostr event carrying one
base64-encoded frame is **629 bytes**, so 18 seats with three frame kinds each
publish **~34 kB per round** across all seats, and a seat on this path reads
the same amount. Measured against no relay; §4.3.1 compares it with the
relays' published limits.

Two limits:

- **Spectators do not use this path at scale.** A popular match fanned out
  through a free public relay would make that relay the server netcode Rev 4
  removed. A spectator uses the gossip tier (netcode §3.2); the relay path is
  for seats, and for a spectator only when it reaches no gossip peer.
- **Frames on a relay are stored events with an expiration**, not ephemeral
  events, so a seat that reconnects can backfill from the relay as well as
  from peers (netcode §9.3). Whether relays honor the expiration is the
  relay's choice; frames are public by design (netcode §2(c)), so a relay
  keeping them leaks nothing. (strfry, a common relay, rejects ephemeral
  events older than 60 s and deletes them after 300 s by default, which rules
  out ephemeral events for frames a reconnecting seat must fetch.)

#### 4.3.1 Load against published relay limits — RATIFIED (ruling 10, R-SES16); magnitudes placeholders

Estimates and sources: appendix §D.60. No public relay this container could
reach published its limits (the network policy refused every relay host), so
the comparison is against the **default and example configurations of four
relay implementations** and NIP-11's example document — what an operator gets
without changing anything, not what any named relay runs.

| limit | source | value | this design |
|---|---|---|---|
| writes per IP | khatru `ApplySaneDefaults` | 2 per 3 min, burst 10 | **binding** below |
| writes per IP | noteguard (Damus's strfry plugin) README example | 8 per min | 2.7x–18x headroom per seat |
| writes per IP | a khatru relay's documented defaults | 30 per min, burst 60 | ample |
| writes | nostr-rs-relay example config | 5 per s, averaged over a minute | ample |
| writes | strfry default config | none | — |
| event size | NIP-11 example / strfry / nostr-rs-relay | 16 KiB / 64 KiB / 128 KiB | 629 B |
| proof-of-work | NIP-11 example `min_pow_difficulty` | 30 | **excludes the relay** for frames: every frame would cost ~2³⁰ hashes |

**What binds is the write rate, per IP address, against the wall time of a
round.** A match is 10 barriers (a 4,000-year horizon, the first at 200 years,
then one every 400), and a 30–45 minute match therefore has **180–270 s of
wall time per round** (estimate). At three frames per round a seat writes 1
event per minute at 180 s; khatru's defaults allow two-thirds of one, so its
burst of 10, less the four lobby events, empties in **~18 minutes** and the
relay rejects the seat mid-match. At the spec's earlier placeholder of 60 s per
round it empties in ~3 minutes.

Rules (ruling 10):

1. **Two events per seat per round, not three.** `CHECKPOINT(r)` rides in the
   same event as `COMMIT(r+1)`; `REVEAL(r+1)` follows the barrier alone. Commit
   and reveal cannot share an event, because the barrier separates them, so two
   is the floor. At 180 s per round that is two-thirds of an event per minute:
   inside khatru's defaults with **no margin**, and 12x inside noteguard's
   example. **The last barrier's `CHECKPOINT` goes out alone**, with the
   match's end, because no `COMMIT` follows it — otherwise the final state
   would never be signed.
2. **A floor on wall time per round of 180 s** (*placeholder*), which the
   30–45 minute target implies anyway, and which replaces §6's earlier 60 s
   placeholder. It is a floor on when a seat's client emits its commit, never
   a clock the state reads (netcode §1.1).
3. **Relays are chosen by their NIP-11 document.** The client reads each pinned
   relay's document at startup (a fetch to a pinned origin, §4.5 rule 2) and
   uses a relay for frames only if it requires no proof-of-work, no NIP-42
   authentication and no payment, and its `max_content_length` holds a frame.
   The pinned list favors relays whose operators state a write allowance of at
   least 2 events per minute per IP.
4. **Seats behind one address share a bucket.** Per-IP limits count every seat
   behind one carrier-grade NAT or one LAN together: at 8 per minute and 180 s
   rounds, 12 batched seats fit behind one address; at khatru's defaults, one.
   **Every event goes to every usable pinned relay.** A relay that answers
   `rate-limited:` (NIP-01's prefix) is skipped for that seat until its bucket
   would have refilled, and the event still reaches the others; a seat is cut
   off only if every relay rejects it.
5. **Spectators read from relays only as a fallback.** Reads are not
   rate-limited in any configuration above, but they are bandwidth no operator
   prices for us: one 18-seat match read by **50,000 spectators** through one
   relay is **~1.7 GB per round, ~76 Mbit/s sustained at 180 s rounds, ~17 GB
   per match** (estimate), against **~0.6 MB per round** for the 18 seats
   alone. No implementation above publishes a bandwidth allowance; the
   inference is that a relay operator would treat the first figure as abuse.
   Spectators therefore gossip (netcode §3.2), and §4.4's relay-only default
   applies to seats. A spectator reads a relay in three cases only: to
   bootstrap before it has a gossip peer, when it has none (the fallback), and
   for one targeted event (rule 7).
6. **Join and `QUEUE` requests go to the host's inbox, not the room topic.**
   A request is tagged with the host's key and only the host's tab subscribes
   to that tag. On the room topic, each of 50,000 requests would be delivered
   to every one of 50,000 subscribers — ~1.6 TB at 629 bytes each (estimate).
   The host publishes the room's state (filling, full, queue full) on the room
   topic, and a client reads it before writing a request, so a full room draws
   no writes. A tag is a filter, not access control: anyone may subscribe to
   the host's inbox, and a request therefore carries nothing a stranger may not
   read — a key, a display name, proof-of-work, no address.

7. **A spectator fetches a missing reveal directly.** When a spectator's log
   holds a timeout quorum against seat *s* for round *r* and no reveal from
   *s*, it requests that one event, by seat and round, from a relay. One event
   per incident, so it costs nothing at any audience size; it is what keeps an
   observer a witness to censorship when its gossip peers are not honest
   (§7.5).

Still OPEN under R-SES16, the magnitudes: the 180 s floor, the write
allowance the pinned list requires, and the pinned list itself. What would
settle them: the NIP-11 documents and stated policies of the candidate relays,
read from a network that can reach them; then the R-SES1 prototype running one
18-seat match at 180 s rounds through each and counting `rate-limited:`
replies.

### 4.4 IP addresses — RATIFIED (rulings 7 and 10); magnitudes placeholders

There are three places an address could appear, and they need separate rules.

| where | who can read it | rule |
|---|---|---|
| **the link** | everyone in the chat | RATIFIED: carries no address. It is the Pages URL, a room id, an engine hash, the host's public key and parameters (§5.1) |
| **rendezvous messages** (room descriptor, join requests, announcements, signaling) | anyone holding the link, and the relay operators | RATIFIED: carry no address in readable form. Signaling, which contains WebRTC's SDP and ICE candidates and therefore addresses, is **encrypted to its one recipient** (X25519 from WebCrypto's Secure Curves, or NIP-44) |
| **a WebRTC connection** | the one peer at the other end | RATIFIED (ruling 10): none for a seat by default; below |

**A WebRTC connection reveals each end's public IP address to the peer it
connects to.** That is a property of WebRTC. In a link game the peers are
strangers from the chat, so a direct connection to them gives a stranger the
address even though the chat never sees it.

**Relay-only is the default for seats in link games and public practice**
(ruling 10). The client opens no WebRTC connection and sends every frame over
the rendezvous (§4.3); no participant or spectator learns another's address,
and the relays learn each client's address as any web server does. Direct
WebRTC is offered only when every party has opted in — the natural case is a
league whose roster knows each other. Cost: every frame takes a relay round
trip, once per barrier. **The default covers seats, not spectators**: a large
audience reading relays costs ~76 Mbit/s per relay for one popular match
(§4.3.1), so spectators gossip among themselves by default. A spectator's
gossip links expose its address to other spectators, not to any seat; a
spectator who opts into relay-only accepts the fallback's cost.

**A direct connection's DTLS fingerprint is signed by the seat key** (ruling
10). WebRTC's DTLS proves only that a link reached whoever answered the
signaling, and whoever carries the signaling — a relay — can substitute both
fingerprints and sit in the middle (RFC 8827 §7). Each side's signaling
message therefore carries its DTLS fingerprint signed with its Ed25519 seat or
spectator key, and a client closes any connection whose negotiated fingerprint
does not match the signed one. Frames were already end-to-end signed, so a
relay in the middle could never forge a move; the binding removes its ability
to read the link and drop frames selectively. It applies only where direct
connections are used, so it does not touch a relay-only seat.

What would settle the remaining magnitudes: R-SES1's prototype measuring relay
latency per barrier at 18 seats.

### 4.5 The streamer's audience as the adversary — OPEN (R-SES15)

Ruling 9 sets the threat model. The streamer is one client among up to 17
viewer-participants and an audience of any size, any of whom may collect
what the client lets them collect. What they want, in order of harm: the
streamer's **IP address** (location, ISP, a target for denial of service or
swatting), **identifiers that link sessions** (other games, other accounts,
the streamer's friends), and **behavior** (when the streamer plays, on what
hardware).

**What reaches a member of the audience, by channel**, under the
recommendations below:

| channel | what crosses it | rule |
|---|---|---|
| the link | Pages URL, room id, engine hash, host key, parameters | no address (ruling 7) |
| rendezvous events | signed frames, a throwaway Nostr key, the event's client-set timestamp | no address; signaling encrypted (§4.4) |
| WebRTC | — | **the streamer's client opens no peer connection** (below) |
| the match record | keys, display names, frames | keys are per session (below) |
| any URL the client fetches | the client's IP address, to that host's operator | **the client fetches only from origins pinned in its build** (below) |

**Recommended rules, each closing one path:**

1. **No peer connection from the streamer's client, ever.** Privacy is a
   property of one client, so the rule is per client: a client in private mode
   creates no `RTCPeerConnection` and reaches the match through the relays
   alone (§4.3). Other participants and the audience may still connect to
   each other directly, which keeps spectator fan-out off the relays (§4.3's
   limit); the streamer's address is in none of those connections. §4.4
   recommends private mode as the default for everyone in link and practice
   games; this rule is the minimum that holds whatever that default becomes.
2. **The client fetches only from origins pinned in its build**: the Pages
   origin, the pinned relays, nothing else. A link that named a relay, an
   image, a font or an engine mirror would let a viewer who made the link log
   the IP address of everyone who opened it — including a streamer who opens
   a viewer's room. The CSP's `connect-src`, `img-src` and `font-src` (§8)
   enforce this in the browser rather than by review, and `default-src
   'self'` refuses everything else.
3. **A fresh key per session.** The streamer's Ed25519 key and Nostr key are
   generated for the room and discarded after its rematch chain ends, so
   match records, which publish freely (netcode R-NET9), do not link one
   session to the next. Persistent keys exist only for roster play (§4.1), and
   the client warns when one is about to be used in a room the user did not
   make.
4. **No free text in the protocol.** Frames carry a card and a target
   (netcode §4); there is no chat, and none should be added — a chat channel
   is a harassment channel addressed to the streamer. Display names are the
   only text: bounded length and character set, rendered as text, never as
   markup, and a streamer-mode switch replaces other players' names with seat
   colors on screen.
5. **Joining costs the joiner, not the streamer.** An audience of tens of
   thousands can open the link at once. A Nostr key costs nothing to make, so
   a per-key rate limit (§7.4) bounds nothing; instead each join and `QUEUE`
   request carries NIP-13 proof-of-work and goes to the streamer's inbox, not
   the room topic (§4.3.1 rule 6); the streamer's tab keeps a bounded queue
   (*placeholder*: 64 entries), drops the rest unread, publishes "full" on the
   room topic, and stops reading its inbox once the seats and queue are full.
6. **Stream sniping is a game-integrity issue in the same scenario.** A
   participant who watches the stream sees the streamer's pending order
   before the reveal, which deletes the yomi of netcode §2(b). Rule: the
   streamer-mode UI does not draw the pending order in the shared view; the
   streamer sees it in a panel they keep out of the capture. A stream delay
   longer than a round's commit window also works, but with 180 s rounds
   (§4.3.1) that is a delay of minutes, so the panel is the default.

**Residual exposure, stated so it is not mistaken for zero:**

- **Relay operators see the streamer's IP address**, as every web server the
  streamer visits does. The pinned list is chosen by the project, not by the
  audience (rule 2). A streamer who wants more runs the client over a VPN, or
  adds a relay they operate to their own settings.
- **GitHub sees the page load**, as for any Pages site.
- **Commit timing** shows when the streamer acts each round and, coarsely,
  how fast their machine simulates. The first is on the stream anyway; the
  second is one bit of hardware class. Not mitigated; a client could hold its
  commit to a fixed minimum time if it matters.
- **Latency-based geolocation** needs round-trip times to the target from
  several vantage points. Under rule 1 the only party exchanging packets with
  the streamer's client is a relay, so a participant sees the streamer's
  frames delayed by two relay hops of unknown length. This is an inference,
  not a measurement; the R-SES1 prototype could measure how well frame
  arrival times on the relay predict the streamer's distance.
- **The audience's own addresses**: spectators gossip directly (§4.3.1 rule
  5), so each spectator's address reaches the spectators it links to. No
  seat's address does, the streamer's included. A spectator who objects opts
  into relay-only.
- **Sybil participants**: the audience can fill every seat but the streamer's
  with one coordinated group (§5.1). That is a game-integrity problem; it
  learns nothing about the streamer that rules 1–4 leave visible.

What would settle R-SES15: the author's ruling on rules 1–6, and a test that
opens a streamer-mode room from a viewer-made link and records every host the
streamer's browser contacts.

---

## 5. The four entry points

### 5.1 The link — OPEN (R-SES5)

**The link is the game's GitHub Pages site.** For this repository:

```
https://dwindlehop-pixel.github.io/hyades-4x/#j=<payload>
```

Clicking it loads the client from Pages, the same page as the menu, and the
client reads the payload. The payload is in the URL **fragment**, which the
browser does not send to the server, so GitHub Pages never sees which room a
person opened. **It contains no network address** (ruling 7): the room is
found through the rendezvous by `room_id`, and addresses, if any are exchanged
at all, travel encrypted to one recipient (§4.4). Recommended payload,
base64url:

| field | bytes | purpose |
|---|---|---|
| `version` | 1 | link format |
| `engine_sha256` | 32 | which engine build the room runs (§8); a link outlives a site deploy |
| `room_id` | 16 | random; the rendezvous topic is derived from it |
| `host_pubkey` | 32 | the originator's Ed25519 key — only it can sign the room's parameters and seat table |
| `params` | ~8 | mode id or preset, seat count, private-mode flag, a few overrides |

Estimated length: **89 bytes → 167 characters** with this repository's Pages
host (computed; the full parameter set is not yet fixed). Twitch chat's
message limit is 500 characters.

**Joining.**

1. The client loads the engine build named by `engine_sha256` (§8) and
   subscribes to the room topic.
2. It fetches the originator's signed **room descriptor** — the full
   parameters, whose hash matches `params` or extends it — and shows them
   before the player commits to anything.
3. The player sends a signed **join request** with their key and name.
4. **The originator's tab sequences the room.** It admits joiners in the order
   it receives them until the seats are full, may kick a seated joiner before
   genesis (ruling 6, below), and signs the seat table. There is no fairness rule on admission:
   the link is the originator's, and anyone who disagrees with the originator's
   admissions can decline to sign genesis.
5. Genesis proceeds per netcode §7: the jointly random galaxy seed (§7.1),
   every seat signing `session_id`. The originator's authority ends at genesis;
   after it the originator is one seat.

**Full rooms.** A person who opens the link after the seat table is full
becomes a spectator: the client joins the gossip tier through the room topic
(netcode §3.2's rendezvous sample, now taken from the rendezvous instead of a
lobby server). If the originator's tab closes before genesis, the room does
not start. Recommendation: the client says so, and offers to make a new link
with the same parameters.

**Waiting spectators and the kick** (ruling 6). While the room fills and after
it is full, a spectator may sign a `QUEUE` request, sent to the originator's
inbox (§4.3.1 rule 6) only while the room's published state says the queue has
room; the originator's tab keeps the requests in the order it received them.

- **Before genesis**, the originator signs a `KICK` naming a seated key. The
  seat is removed from the seat table and offered to the **first spectator in
  the queue**, who takes it by signing a join request. If the queue is empty,
  the seat stays open to anyone holding the link. A kicked key cannot rejoin
  that room.
- A kick before genesis changes no game state: there is no game yet, and the
  seat table every seat signs at genesis is the one in force. A kicked player's
  only record of it is the originator's signed `KICK`.
- **After genesis — OPEN (R-SES14).** The seat table is inside `session_id` and
  a seat's key cannot be swapped without a new genesis, so an observer cannot
  take a seat mid-match. Recommendation: a host `KICK` frame after genesis
  hands the seat to the autopilot at the next barrier, exactly as a dropout
  does (netcode §5.3); it is in the transcript, so every verifier sees who
  kicked whom and when. The cost of the recommendation is a host power over an
  opponent mid-match, which netcode does not otherwise give any seat; the
  alternative is that the host cannot kick after the start, and a disruptive
  player is handled by the timeout and dropout paths alone.

**Rematch.** At the match's end every seat may sign a `REMATCH` intent (yes or
no), and every spectator may sign a `QUEUE` request to the new host's inbox.

- The new room's parameters are the old room's. The new galaxy seed is jointly
  random again (netcode §7.1).
- The new host is the old originator if they said yes, else the
  lowest-indexed seat that said yes. Continuing seats keep their keys.
- **Vacant seats are filled from the queue in the order the new host received
  the requests** — the same sequencing rule as the first room.
- The new genesis carries `rematch_of: <previous session_id>`, so a series of
  rematches is a verifiable chain.

**Sybil seats.** One person can open several tabs, take several seats, and
form a majority alone. No mechanism without identity can detect this from
signatures. It is netcode §2(d)'s collusion case, out of scope for any
protocol, and the link's originator is the defense (step 4).

### 5.2 League play — OPEN (R-SES6)

The game provides two things for a league and nothing else (ruling 8):
**seats restricted to a roster**, and **a verifiable match record for every
participant** (§5.4). Who organizes a league, where its fixtures and results
live, and how it ranks anyone are outside the design. Recommendation:

- **A roster link is a link game whose seat table is fixed in the link.** The
  person who makes it (the organizer, or any rostered player) lists the
  rostered Ed25519 public keys; the client prints each player's key for them to
  send to whoever makes the link, by any means.
- **The roster rides in the link** when it fits: 32 bytes per key, so 18 keys
  add 576 bytes and the link is 935 characters (computed for this repository's
  host) — within Discord's
  limit, over Twitch's 500. A **watch link**, without the roster, is the short
  form for chat (§5.1's payload with a spectator flag); spectators need no
  roster because the signed genesis carries it.
- **Only rostered players may take a seat**: the genesis seat table is the
  roster, and netcode §3.1's seat binding refuses every other key. No new
  mechanism.
- **Auditing the parameters is signing genesis.** The client shows the
  parameters, the engine hash and the card list hash before a rostered player
  signs, and a player who has not signed is not seated. A signature is
  non-repudiable consent (netcode §7).
- **Verification policy** defaults to netcode §8.2's Ranked preset; the link
  may name observer seats (Refereed).
- **Anyone observes** through the watch link.
- **After the match** every participant and spectator can export the match
  record and verify it (§5.4). That is the end of the game's involvement.

### 5.3 Public practice — OPEN (R-SES7)

- **The modes are a static table on the site** (two or three entries: seat
  count, preset, verification policy Public). Changing them is a commit.
- **An open room announces itself** on the rendezvous: an event tagged with
  the protocol version and mode id, carrying the room's link and seat fill,
  refreshed by the originator's tab every ~60 s (*placeholder*) with an
  expiration of a few minutes. When the room starts it publishes one final
  event marking it in progress, with the spectator link.
- **The list is a subscription** filtered by mode, shown sorted by age or by
  fill. **There is no ranking and no assignment**: the player picks a row, and
  joining is §5.1. If no room is open, the player creates one, and it appears
  in other players' lists.
- **Spam**: a listed room costs its announcer proof-of-work (§7.4). The client
  shows at most one room per host key, and drops a row whose host does not
  answer a join request within a few seconds.

### 5.4 The match record — OPEN (R-SES8)

**The file.** A transcript in a fixed binary layout — the canonical CBOR
genesis descriptor, its signature set, then 144-byte frames — with an optional
replay JSON beside it. Estimated size (netcode §8.4's formula, `144 × N × 3 ×
R` bytes): **78 kB** at 18 seats and 10 rounds, **467 kB** at 18 seats and 60
rounds. At `years_per_round = 400` and a 200-year opening, a 4,000-year match
has 10 barriers.

**Verification**, in order, each step a pass or a stated failure:

1. Every frame's signature is valid and its `session_id` is the genesis's.
2. Every seat signed `session_id`.
3. The client obtains the engine build whose hash is in genesis (§8).
4. The engine replays the transcript headless and computes the state root at
   every checkpoint round.
5. The verifier compares its root against every signed `CHECKPOINT` in the
   file, and reports per seat and per round (§7.2).
6. The outcome is the result of the replay. **No outcome claim in a file is
   read**; there is none to read.

**Interrogation.** After a transcript verifies, the client records a replay
from it with the in-browser engine (`Hyades_interface.md` R-UI5's path) and
opens the viewer on it, with every log category enabled — tactical mode, the
log filter, seek to any time. The engine and the viewer stay two modules: the
viewer reads the replay and does not link the engine (interface §2).

---

## 6. The engine in the browser

Every story but a replay-only viewing needs `hyades-engine` compiled to wasm32
and running in a Worker, reached through `apply_orders` alone (netcode §11).
This does not exist yet (T-165).

**Throughput is the feasibility question, and it is not measured.** Two
quantities set it:

- **Live play.** Between barriers every seat simulates a round. At
  `years_per_round = 400` and a wall time per round of *T* seconds, a seat
  must sustain `400 / T` simulated years per second. At *T* = 180 s
  (placeholder, §4.3.1) that is 2.2 yr/s, which T-24's floor of 2.5 yr/s
  covers; at 60 s it would be 6.7 yr/s.
- **Late spectators.** A spectator who arrives at round *r* must replay *r*
  rounds before it sees the present. If it simulates at the same rate the
  seats do, it never catches up while the match runs. **A spectator arriving
  mid-match needs a replay rate several times the live rate, or netcode
  R-NET7's snapshot-assisted catch-up** — which needs the state digest (T-32)
  to check the snapshot.

The missing measurement: the wasm32 build's simulated years per second at 18
seats, on a mid-range phone and a laptop, over a full match horizon (T-166).
Native figures in `AGENTS.md` §7 are not a substitute: they are a different
target, and several were taken on 3 seats.

**Unverified spectating — OPEN (R-SES9).** A phone in a Twitch audience may
not sustain the engine. Option: a **light spectator** that renders replay
frames streamed by a peer without running the simulation, shown with an
unverified label. It can be lied to, and it cannot detect any attack in §7.
Recommendation: build it only if T-166 shows phones cannot keep up.

---

## 7. What a majority can do, and who detects it

Netcode §8.3 states that the vote is divergence detection, not Byzantine
consensus. This section states what each attack changes and what evidence
exposes it. **The rule that makes detection possible: every client's displayed
verdict is its own replay.** A quorum decides how the protocol continues for
the seats; it never decides what an observer or a verifier believes the state
is.

### 7.1 The three classes

| class | what the colluders do | effect in the match | detectable from |
|---|---|---|---|
| **A. State forgery** | sign `CHECKPOINT` roots that the pinned engine does not produce — to eject a conforming minority under `ContinueWithQuorum`, or to play on in an altered state | the minority is ejected, or the colluders play a game nobody else's engine reproduces | **any single transcript containing their checkpoints**, by anyone with the pinned engine; and live, by every observer |
| **B. Equivocation** | sign two different frames for one `(seat, round, kind)`, sent to different peers | different peers act on different orders until the pair meets | **any transcript holding both frames** — non-repudiable proof (netcode §4.4) |
| **C. Censorship** | cast `TIMEOUT_VOTE`s against a seat that did reveal, so its order defaults to `pass`; or withhold its frames from the overlay | the victim's orders are discarded while the protocol runs correctly | **not from a transcript the colluders assembled alone.** Detectable by comparing it with **one witness transcript** — the victim's, or any observer's that received the victim's reveals; or by querying the relays for the victim's reveal while they keep it (§7.5) |

**Class A cannot succeed silently even at 100%.** The outcome of a match is
the replay of its inputs, so colluders who sign false roots gain no outcome
that a verifier will compute; and colluders who omit their checkpoints from a
file publish a file whose outcome is the conforming one. The only way to change
the outcome is to change the inputs, which is class C.

**Class C is the 51% attack that succeeds, and no protocol change removes it.**
On a network with no bound on delivery time, a frame that was withheld and a
frame that was lost leave the same log at the receiver, so a colluding majority
that consistently claims it never received a seat's frames signs nothing that
contradicts that claim. The evidence of censorship is the victim's frame in
someone else's hands. So:

- **Every client keeps its whole transcript**, including frames that arrived
  after a timeout quorum already defaulted their seat.
- **An observer's client flags it live**: "seat *s*'s reveal for round *r* is
  in this client's log; seats {…} voted it timed out." One such round is
  consistent with ordinary packet loss. The client shows the count of such
  rounds per seat, since a seat defaulted in many rounds while its reveals
  reach observers is the pattern a censoring majority produces.
- **A match record may carry witness transcripts**: the verifier merges
  them, and reports every round where a seat's reveal exists in a witness and
  the main transcript defaulted it.
- **The relays are a witness too** (§7.5): a relay-only seat's reveal sits on
  every usable pinned relay, so a reader can query them for any seat and round
  a timeout quorum defaulted.

### 7.2 The verifier's report — OPEN (R-SES10)

Recommendation, per round, one of:

- **agrees** — the replayed root equals every signed root;
- **seats {…} signed roots this engine does not produce** — class A, naming
  them, with the frames as evidence;
- **no seat's signed root is reproduced** — the transcript is missing frames
  or the engine build is wrong; this accuses nobody, because a file with one
  frame removed produces exactly this;
- **equivocation by seat *s*** — class B, with the frame pair;
- **seat *s* defaulted while a witness holds its reveal** — class C, from a
  merged witness;
- **seat *s* defaulted while a pinned relay holds its reveal** — class C, from
  a relay query (§7.5).

**R-SES11 — OPEN.** Should each seat also sign, per round, a hash of the order
set it applied (a `ROUND_INPUTS` frame, or an inputs leaf in netcode §8.1's
digest)? It would let the third verdict above say *which* round's inputs a
file is missing instead of only that the roots disagree. It adds one frame per
seat per round (+33% transcript size, estimated from the three kinds now) and
does not make class C detectable from one file. Recommendation: add it as a
digest leaf when T-32 lands, not as a frame.

### 7.3 Ejection under a forged majority — OPEN (R-SES12)

Under netcode §8.2's Public preset, a class-A majority ejects the conforming
minority, who then cannot continue: the timeout quorum needs a majority they
do not have. Recommendation: the ejected seats' clients **keep replaying** the
inputs that still arrive and keep showing the conforming state, so spectators
see two series of roots and which one their own engine reproduces. Nothing in
the protocol changes; this is presentation.

### 7.4 Denial of service without a server

There is no server to flood. What remains:

- **Room-list spam** (§5.3): announcements carry NIP-13 proof-of-work at a
  difficulty set in the client (*placeholder*: one to two seconds on a laptop).
  Joining a room costs nothing, because the originator's tab admits joiners and
  can remove them.
- **Join floods against an originator**: join and `QUEUE` requests carry
  proof-of-work and go to the originator's inbox rather than the room topic,
  so the audience does not receive each other's requests; the originator's tab
  keeps a bounded queue, drops the rest unread, and publishes the room's state
  so clients stop writing once the seats and queue are full (§4.3.1 rule 6,
  §4.5 rule 5). A per-key rate limit bounds nothing, because a
  Nostr key costs nothing to make.
- **Frame floods**: netcode §4.3 is unchanged — the length check, the window
  and the token bucket come before signature verification.
- **Relay censorship**: a relay that drops a room's events is one of several
  (R-SES1); a room is lost only if every relay the participants share drops it.


### 7.5 What the relay rules change in the two threat models — RATIFIED (ruling 10) where it states a rule; analysis otherwise

Ruling 10 adopts §4.3.1's rules, relay-only seats (§4.4) and the fingerprint
binding (§4.4). Each was chosen for load or privacy; this section checks each
against the majority model (§7.1) and the streamer model (§4.5). Statements
marked *inference* are reasoning, not measurement.

| fix | majority model (§7.1) | streamer model (§4.5) |
|---|---|---|
| relay-only seats | **class C gets a public witness**, below; a new censor appears, below | no seat address reaches anyone but relay operators; unchanged for the streamer, who was relay-only under rule 1 already |
| two events per round | **class A detection is delayed** by one decision: `CHECKPOINT(r)` is published when the seat commits for `r+1`, not when it finishes resolving. Equivocation proof is unchanged, because each frame inside an event is still signed alone. The last barrier's checkpoint goes out alone (§4.3.1 rule 1) | none |
| 180 s floor per round | none: the floor gates when a client emits, never what the state becomes | stream delay stops being a practical sniping defense; the out-of-capture panel is the default (§4.5 rule 6) |
| relays chosen by NIP-11 | fewer usable relays concentrate the transport: a seat is censored by the transport only if every usable relay drops it, and fewer relays make that cheaper (*inference*) | none: NIP-11 is fetched from pinned origins only |
| spectators gossip, relays as fallback | **an observer can be fed by colluders**: if every gossip peer and bridge an observer has is run by the majority, they can withhold the victim's reveal and the observer loses its witness. Rule 7 closes this: a timeout quorum without a reveal triggers a one-event relay fetch | **spectators' addresses reach other spectators** (§4.5 residuals); the streamer's does not |
| join requests to the host's inbox | none: queue order was trusted to the host before and still is | the audience no longer receives each other's requests; anyone can still read the inbox, so a request carries nothing private |
| DTLS fingerprint signed by the seat key | removes a relay's ability to sit inside a direct link and drop frames selectively, which was a way to manufacture class C without a colluding seat | none for the streamer, who opens no direct link |

**Class C gets a public witness.** A relay-only seat publishes its reveal to
every usable pinned relay (§4.3.1 rule 4). So whenever a majority votes a seat
timed out, anyone — an observer during the match, a verifier holding only the
colluders' file afterwards, while the relays keep the events — can query the
relays for that seat's reveal for that round. Finding it does not prove the
majority received it; it proves the seat sent it to public infrastructure the
majority also reads. One round of it is consistent with relay trouble; many
rounds against one seat is the pattern of censorship (*inference*). This
changes §7.1's statement that class C is "not detectable from a transcript the
colluders assembled": it still is not detectable **from the file**, and is now
detectable **from the file plus the relays**, for as long as the relays keep the
events. The verifier gains a sixth verdict (R-SES10): *seat s defaulted while a
pinned relay holds its reveal*.

**A new censor: the transport.** Under relay-only, a seat reaches the match
only through relays, so a relay that drops a seat's events — by policy, by
fault, or because someone sharing the seat's IP address has spent its per-IP
bucket — makes the seat time out with no colluding seat at all. Every
conforming seat then votes the timeout correctly. Two things bound it: the
event goes to every usable relay, so all of them must drop it; and the same
relay query above tells a reader whether the seat's reveal reached any relay,
which separates "the seat sent nothing" from "the relays the voters read did not
carry it". A neighbor exhausting a shared carrier NAT's bucket is the one form
that needs no relay operator's cooperation; it works only against seats behind
a shared address and only on relays whose bucket it can drain (*inference*).

**Net effect.** The majority model is stronger: class C, the one attack that
succeeds, becomes visible to anyone who can query the relays. Class A is seen
one decision later. The streamer model is unchanged for the streamer and costs
the audience their addresses among themselves. One threat is new — censorship
by the transport — and it is bounded by publishing to every relay and detected
by the same relay query.

---

## 8. The site and its supply chain

- **Engines are content-addressed and kept.** Every build's engine module is
  published at `engine/<sha256>.wasm` and never deleted, so a link or a match
  record made against an old build still loads it. The page loads it with
  Subresource Integrity on the hash from the link or genesis (netcode H1).
  OPEN (**R-SES13**): retention — the size of one engine build is not
  measured, and the site grows by one build per engine change that reaches
  `main`.
- **GitHub Pages cannot set response headers.** Netcode §11's CSP is therefore
  a `<meta http-equiv="Content-Security-Policy">`, which the CSP specification
  permits but which cannot express `frame-ancestors` or a report endpoint
  ([Helme](https://scotthelme.co.uk/launching-report-uri-js/)). The policy
  allows `wasm-unsafe-eval`, `connect-src` to the pinned relays and STUN
  servers, and nothing else. Netcode H6 (no threads) means no COOP/COEP header
  is needed; Pages could not send one.
- **No runtime CDN.** Every upstream package (the secp256k1 library, a CBOR
  encoder) is vendored into the site at build time, so the site's content is
  exactly the repository's.
- **The trust root is the repository's `main` branch and GitHub.** A modified
  client harms only its own user: every other client checks its frames and
  roots. A compromised site could serve a malicious client to everyone; a
  player who distrusts the deployment can run the site from a local checkout,
  and their client interoperates as long as the engine hash matches.

---

## 9. Amendments to other specs

| spec | what it said | what this spec does |
|---|---|---|
| netcode §10 | three services: lobby/API, signaling, STUN/TURN | **replaced** by Pages + public rendezvous + public STUN (§3); no TURN (§4.3) |
| netcode §10 (Rev 4) | "no protocol frame ever transits the server" | still true of any server we run; frames **may** transit a third-party relay for a seat that reaches no peer (§4.3) |
| netcode §3.2 | the spectator rendezvous sample comes from the lobby | it comes from the room's rendezvous topic |
| netcode §3.3 | TURN must exist and be metered | **withdrawn** (§4.3); the relay transport is the last rung |
| netcode §7 | "the lobby publishes the descriptor with the full signature set" | the originator publishes it on the rendezvous; genesis gains `room_id`, `host_pubkey` and `rematch_of` |
| netcode §10 | room creation gated by a challenge token | room announcement gated by proof-of-work (§7.4) |
| netcode §11 | CSP as a response header | a meta tag; Pages sends no custom headers (§8) |
| netcode §8.3 | the vote's failures "are identifiable — the transcript names who signed what" | true of classes A and B; **false of class C from one transcript**, true of it from a transcript plus the relays (§7.1, §7.5) |
| interface §8.1 | New game "shown, not built yet" | the menu's play entries are §5's four |

These are amendments **proposed** by an OPEN spec. Netcode is not edited to
match until the author ratifies the R-codes involved; netcode's header carries
a pointer here meanwhile.

---

## 10. Register

| code | decision | status | what would settle it |
|---|---|---|---|
| **R-SES1** | Nostr as the rendezvous; relay count; a second backend | OPEN — recommended (§3.1) | a week-long prototype measuring join completion through the pinned relays and any rate limiting at an 18-seat signaling burst |
| **R-SES2** | identity: per-tab keys for casual play, persistent backed-up keys for roster play; name plus fingerprint | OPEN — recommended (§4.1) | the author's ruling; a test of key survival in Safari |
| **R-SES3** | no TURN; frames over the rendezvous as the last transport | OPEN — recommended (§4.3) | the R-SES1 prototype, plus the fraction of seats that reach no peer |
| **R-SES4** | relay-only seats by default in link and practice games; direct WebRTC only when every party opts in; DTLS fingerprints signed by the seat key | **RATIFIED** (ruling 10); the link and public messages carrying no address is RATIFIED (ruling 7) | — (relay latency per barrier is measured under R-SES1) |
| **R-SES5** | link format, originator sequencing, the spectator queue, rematch host and queue order; the pre-genesis kick is RATIFIED (ruling 6) | OPEN — recommended (§5.1) | the author's ruling; whether first-received queue order is acceptable or a lottery is wanted (a lottery seeded by the rematch's joint seed resists timing races but not sybils) |
| **R-SES6** | roster links: the seat table fixed in the link, a short watch link for chat; standings and results outside the design (ruling 8) | OPEN — recommended (§5.2) | one roster match run end to end |
| **R-SES7** | practice modes as a static table; room announcements; no ranking of the list | OPEN — recommended (§5.3) | the author choosing the two or three modes |
| **R-SES8** | match record format and verification steps | OPEN — recommended (§5.4) | T-32 (the digest) and T-165 (the engine in the browser) |
| **R-SES9** | light, unverified spectators | OPEN — build only if needed (§6) | T-166 |
| **R-SES10** | the verifier's six verdicts | OPEN — recommended (§7.2) | the author's ruling |
| **R-SES11** | a per-round inputs hash | OPEN — recommended as a digest leaf (§7.2) | T-32 |
| **R-SES12** | ejected seats keep replaying and showing their roots | OPEN — recommended (§7.3) | the author's ruling |
| **R-SES13** | retention of old engine builds on Pages | OPEN | the repository's size after a year of builds |
| **R-SES14** | a host kick after genesis: the seat goes to the autopilot, or no kick after the start | OPEN — recommended: autopilot (§5.1) | the author's ruling |
| **R-SES15** | the streamer threat model's six rules: no peer connection from a private client, fetches only from pinned origins, fresh keys per session, no free text, proof-of-work on joins, the pending order kept off the shared view | OPEN — recommended (§4.5); the threat model itself is RATIFIED (ruling 9) | the author's ruling; a test recording every host a streamer-mode browser contacts when it opens a viewer-made link |
| **R-SES16** | relay load: two events per seat per round (the last checkpoint alone), a 180 s floor per round, relays chosen by NIP-11, every event to every usable relay, spectators off the relays but for a targeted reveal fetch, join requests to the host's inbox | **RATIFIED** (ruling 10); the magnitudes (180 s, the required write allowance, the pinned list) are placeholders | the candidate relays' NIP-11 documents and policies, read from a network that reaches them; one 18-seat match through each, counting `rate-limited:` replies |

Engine work this spec depends on, already tracked: **T-30/T-42** (commit and
reveal in the engine), **T-31** (cards; R-NET4's field widths), **T-32** (the
state digest — classes A and C both read it), **T-37** (netcode outside the
crate, which this spec now gives a design). New: **T-164** (this spec),
**T-165** (the engine as a browser module and the transcript verifier),
**T-166** (wasm32 throughput at 18 seats on a phone), **T-167** (the
rendezvous layer and the link flow).

---

## References

- `Hyades_netcode.md` (Rev 4) — every section cited above
- `Hyades_interface.md` — §2 (the seam), §3 (the replay format), §8 (the client), R-UI5
- Nostr NIP-01 (protocol), NIP-13 (proof of work), NIP-40 (expiration), NIP-44 (encrypted payloads): https://github.com/nostr-protocol/nips
- Trystero — serverless WebRTC matchmaking over public networks: https://github.com/dmotz/trystero
- Apple's 7-day cap on script-writable storage (Safari 13.1 / iOS 13.4): https://docs.didomi.io/releases-and-announcements/announcements/apple-implements-7-day-cap-on-script-writable-storage
- CSP by meta tag on hosts without header control, and its limits: https://scotthelme.co.uk/launching-report-uri-js/ · https://htmhell.dev/adventcalendar/2023/7/
- WICG Secure Curves in WebCrypto (Ed25519, X25519): https://wicg.github.io/webcrypto-secure-curves/
- Fischer, Lynch & Paterson, *Impossibility of Distributed Consensus with One Faulty Process*, J. ACM 32(2), 1985 — the asynchronous model §7.1's class C argument is made in
