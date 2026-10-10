# Hyades — sessions, discovery and security (Rev 1)

How a player gets from a link to a seat, how a public game is found, how a
league restricts its seats, how a match record is verified, and what the
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
public servers. There is no TURN (§4.3). The only places computation is
optional are a league's own repository (§5.2's GitHub Actions verifier) and a
user who chooses to run their own relay.

**One trust rule covers every transport: authority comes from the Ed25519
signatures inside a payload, never from the transport.** A relay's own event
signature, a WebRTC peer's DTLS identity, and the page's origin authenticate
nothing about the game. This is netcode §3.2's "every relay is untrusted",
extended to the rendezvous and to the lobby.

### 3.1 Why Nostr for the rendezvous — OPEN (R-SES1)

Recommendation: **Nostr relays** ([NIP-01](https://github.com/nostr-protocol/nips/blob/master/01.md)),
publishing to and subscribing from 3–5 relays at once (*placeholder*), with
the relay list pinned in the client build and overridable by relay hints in a
link.

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
  machines. A lost key is replaced by the league organizer re-signing the
  roster (§5.2).
- **A display name is self-chosen and unauthenticated.** The client shows a
  short fingerprint of the key beside it (*placeholder*: 4 words or a glyph),
  so two players named alike are distinguishable.

### 4.2 Transport ladder

A frame is self-authenticating, so every transport below carries the same
144-byte frames and the receive discipline of netcode §4.3 applies unchanged:

1. **WebRTC direct**, on netcode §3.1's circulant overlay.
2. **WebRTC through another seat** — netcode §9.2's flood-with-dedup relay.
3. **The rendezvous itself** (§4.3), for a seat that reaches no peer.

### 4.3 No TURN; frames over the rendezvous as the fallback — OPEN (R-SES3)

Netcode §3.3 and §10 provision TURN. TURN requires a credential issuer and
metered relay bandwidth, which are server computation and server cost, and
ruling 2 excludes them.

Recommendation: **a seat that reaches no peer publishes and reads its frames
as rendezvous events.** The volume is small because the protocol is
round-barrier lockstep (netcode §1). Estimated, at 18 seats with three frame
kinds per seat per round and ~640 bytes per Nostr event carrying one
base64-encoded frame (estimate; not measured against a relay): **~35 kB
published per round across all seats**. A seat on this path reads every
seat's frames from the relay, so its download per round is the same estimate.

Two limits:

- **Spectators do not use this path at scale.** A popular match fanned out
  through a free public relay would make that relay the server netcode Rev 4
  removed. A spectator uses the gossip tier (netcode §3.2); the relay path is
  for seats, and for a spectator only when it reaches no gossip peer.
- **Frames on a relay are stored events with an expiration**, not ephemeral
  events, so a seat that reconnects can backfill from the relay as well as
  from peers (netcode §9.3). Whether relays honor the expiration is the
  relay's choice; frames are public by design (netcode §2(c)), so a relay
  keeping them leaks nothing.

### 4.4 IP addresses — RATIFIED as a hazard, mitigation OPEN (R-SES4)

**A WebRTC connection reveals each end's public IP address to the other.** A
streamer who posts a link in Twitch chat reveals their address to every
stranger who joins or spectates through a direct link. This is a property of
WebRTC, not of this design, and the itch.io barrier (ruling 4) means the
people joining are strangers.

Recommendation: a **private mode**, chosen by the originator in the link and
by any joiner for themselves, in which the client opens **no** WebRTC
connection and uses the relay transport (§4.3) for everything. Cost: every
frame takes a relay round trip, which is acceptable at one barrier per round.
Signaling events are encrypted to their recipient (X25519 from WebCrypto's
Secure Curves, or NIP-44), because SDP carries addresses and a room's topic is
readable by anyone holding the link.

---

## 5. The four entry points

### 5.1 The link — OPEN (R-SES5)

**The link.** `https://<pages-host>/<repo>/#j=<payload>`. The payload is in the
URL **fragment**, which the browser does not send to the server, so GitHub
Pages never sees which room a person opened. Recommended payload, base64url:

| field | bytes | purpose |
|---|---|---|
| `version` | 1 | link format |
| `engine_sha256` | 32 | which engine build the room runs (§8); a link outlives a site deploy |
| `room_id` | 16 | random; the rendezvous topic is derived from it |
| `host_pubkey` | 32 | the originator's Ed25519 key — only it can sign the room's parameters and seat table |
| `params` | ~8 | mode id or preset, seat count, private-mode flag, a few overrides |

Estimated length: **89 bytes → 167 characters** with this repository's Pages
host (computed; the full parameter set is not yet fixed). Twitch chat's
message limit is 500 characters, so the link fits with room for relay hints.

**Joining.**

1. The client loads the engine build named by `engine_sha256` (§8) and
   subscribes to the room topic.
2. It fetches the originator's signed **room descriptor** — the full
   parameters, whose hash matches `params` or extends it — and shows them
   before the player commits to anything.
3. The player sends a signed **join request** with their key and name.
4. **The originator's tab sequences the room.** It admits joiners in the order
   it receives them until the seats are full, may remove a joiner before
   genesis, and signs the seat table. There is no fairness rule on admission:
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

**Rematch.** At the match's end every seat may sign a `REMATCH` intent (yes or
no), and every spectator may sign a `QUEUE` request.

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

A **league** is a GitHub repository (or a directory in one) maintained by its
organizer, served by Pages. Recommendation:

- **The roster is a file**: the rostered Ed25519 public keys and display
  names, and the league's parameters, signed by the organizer's key.
  A player's key reaches the organizer out of band (a Discord message, a pull
  request adding it); the client prints the key to copy.
- **A league match link** carries the league's address and a fixture id
  instead of an originator. The genesis seat table is exactly the fixture's
  rostered keys, in roster order. Netcode §3.1's seat binding already refuses
  every unlisted key, so "only rostered players may participate" needs no new
  mechanism.
- **Auditing the parameters is signing genesis.** The client shows the
  parameters, the engine hash and the card list hash, and a rostered player
  who has not signed is not seated. A signature is non-repudiable consent
  (netcode §7).
- **Verification policy** defaults to netcode §8.2's Ranked preset; a league
  may name observer seats (Refereed).
- **Anyone observes**: spectators join the gossip tier through the fixture's
  rendezvous topic.
- **Results** are submitted as a pull request adding the match record to the
  league repository. A **GitHub Actions workflow verifies it** with the native
  engine, which is bit-identical to the wasm32 build since T-127 (netcode H4a),
  and the merge is the result entering the league's table.
- **Standings are the league's own**, computed from its verified records. There
  is no global rank (ruling 3); two leagues may rank the same player
  differently.

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
  must sustain `400 / T` simulated years per second. At *T* = 60 s
  (placeholder) that is 6.7 yr/s, above T-24's floor of 2.5 yr/s.
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
| **C. Censorship** | cast `TIMEOUT_VOTE`s against a seat that did reveal, so its order defaults to `pass`; or withhold its frames from the overlay | the victim's orders are discarded while the protocol runs correctly | **not from a transcript the colluders assembled.** Detectable by comparing it with **one witness transcript** — the victim's, or any observer's that received the victim's reveals |

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
  merged witness.

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
- **Join floods against an originator**: the originator's tab rate-limits join
  requests per Nostr key and stops reading the room topic once the seats are
  full.
- **Frame floods**: netcode §4.3 is unchanged — the length check, the window
  and the token bucket come before signature verification.
- **Relay censorship**: a relay that drops a room's events is one of several
  (R-SES1); a room is lost only if every relay the participants share drops it.

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
| netcode §7 | "the lobby publishes the descriptor with the full signature set" | the originator (or the league repository) publishes it on the rendezvous; genesis gains `room_id`, `host_pubkey` or the league fixture id, and `rematch_of` |
| netcode §10 | room creation gated by a challenge token | room announcement gated by proof-of-work (§7.4) |
| netcode §11 | CSP as a response header | a meta tag; Pages sends no custom headers (§8) |
| netcode §8.3 | the vote's failures "are identifiable — the transcript names who signed what" | true of classes A and B; **false of class C from one transcript** (§7.1) |
| interface §8.1 | New game "shown, not built yet" | the menu's play entries are §5's four |

These are amendments **proposed** by an OPEN spec. Netcode is not edited to
match until the author ratifies the R-codes involved; netcode's header carries
a pointer here meanwhile.

---

## 10. Register

| code | decision | status | what would settle it |
|---|---|---|---|
| **R-SES1** | Nostr as the rendezvous; relay count; a second backend | OPEN — recommended (§3.1) | a week-long prototype measuring join completion through the pinned relays and any rate limiting at an 18-seat signaling burst |
| **R-SES2** | identity: per-tab keys for casual play, persistent backed-up keys for leagues; name plus fingerprint | OPEN — recommended (§4.1) | the author's ruling; a test of key survival in Safari |
| **R-SES3** | no TURN; frames over the rendezvous as the last transport | OPEN — recommended (§4.3) | the R-SES1 prototype, plus the fraction of seats that reach no peer |
| **R-SES4** | private mode: no WebRTC, relay transport only | OPEN — recommended (§4.4) | the author's ruling on whether a streamer's address is the client's concern |
| **R-SES5** | link format, originator sequencing, rematch host and queue order | OPEN — recommended (§5.1) | the author's ruling; whether first-received queue order is acceptable or a lottery is wanted (a lottery seeded by the rematch's joint seed resists timing races but not sybils) |
| **R-SES6** | leagues as repositories, signed rosters, Actions verification | OPEN — recommended (§5.2) | one league run end to end |
| **R-SES7** | practice modes as a static table; room announcements; no ranking of the list | OPEN — recommended (§5.3) | the author choosing the two or three modes |
| **R-SES8** | match record format and verification steps | OPEN — recommended (§5.4) | T-32 (the digest) and T-165 (the engine in the browser) |
| **R-SES9** | light, unverified spectators | OPEN — build only if needed (§6) | T-166 |
| **R-SES10** | the verifier's five verdicts | OPEN — recommended (§7.2) | the author's ruling |
| **R-SES11** | a per-round inputs hash | OPEN — recommended as a digest leaf (§7.2) | T-32 |
| **R-SES12** | ejected seats keep replaying and showing their roots | OPEN — recommended (§7.3) | the author's ruling |
| **R-SES13** | retention of old engine builds on Pages | OPEN | the repository's size after a year of builds |

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
