# Hyades — Client/Server & Peer Protocol

*The normative spec for how a Hyades match is distributed: lobby, peer transport, wire
protocol, commit–reveal, state verification, and server hardening. Companion to
`Hyades_card_contract.md` (card §), `Hyades_simulation_model.md` (sim §), and the
`hyades-engine` determinism contract (`tests/determinism.rs`). Calls flagged **R-NETn**.
**Rev 2:** the command layer presents **perfect information**; Rev 1's fog/maphack
limitation (§2c) is withdrawn as a misreading, and §2.1 replaces it with the rule that
actually matters — the presentation seam is one-directional. **Rev 3:** seat count raised to **18**. Full mesh is retired as the default topology
(§3.1), the rebroadcast rule is corrected from "everything you hold" to difference-only
(§9.1) — at 18 seats the Rev 2 rule cost 3.7 MB/s upload per client — the `HAVE` bitmap
widths were too narrow (§4.2), and `Unanimous + CancelMatch` no longer survives as the
ranked preset (§8.3). The live-seat set is now explicitly a sim fact, not a network fact
(§5.4).

**Rev 4:** **R-NET2 resolved — no server relay.** Spectator fan-out is peer-to-peer gossip
with capped ingress (§3.2); the server now has no game data plane at all (§10). Forces the
new §3.2.1 split between weighted **observer seats** (overlay members) and unweighted
**spectators** (gossip tier).*

---

> ## Engine status — implementation blockers, audited against the tree
>
> *Added when this spec landed in `docs/`. Audit run against the engine at the
> R-O57/R-O58 shell-model landing. Blockers are carried in `hyades_todo.md`
> under the T-codes cited.*
>
> **The host-isolation and determinism foundation is clean.** Everything §6
> H1–H8 and §10 assume about the engine already holds, verified rather than
> asserted:
>
> | Requirement | Status |
> |---|---|
> | H5 — no host clock, no host entropy | **clean** — no `SystemTime`, `Instant`, `std::time`, or `rand` anywhere in `src/` |
> | H6 — no threads | **clean** — no `std::thread`, `Mutex`, `RwLock`, `Arc`, `SharedArrayBuffer` |
> | §0 — no filesystem, network, process, env | **clean** — none in `src/`; no stdout either |
> | Deterministic iteration order | **clean** — **zero** `HashMap`/`HashSet` in the engine; all `BTreeMap`/`BTreeSet`/`Vec` |
> | H7 — RNG stream is a serialization point | **holds** — `Rng::fork(entity_index)`, drawn in index order |
> | §8.1 — `f64` as `to_bits()` | **established** — the determinism suite already compares this way |
> | WASM target | **gated in CI** — `cargo check --lib --target wasm32-unknown-unknown` |
> | §2.1 — autopilot cannot read global state | **structurally enforced** — `Autopilot` takes `PlanetView`/`SurveyView`/`RankContext`/`ProductionContext` and is never handed `&Simulation` or `&World`. It *cannot* reach global state; the question is only what the views are filled with (B4/B5 below) |
>
> **Cleared while landing this spec:**
>
> - **R-NET14, determinism half** — `tests/determinism.rs` topped out at 12
>   seats; it now covers **18** (`[2, 3, 6, 12, 18]`), bit-identical, +6 s.
>   18 became admissible when R-O12 landed. The circulant-offset half of
>   R-NET14 is genesis work and stays open.
> - **H3 / R-NET11, seed of the discipline** —
>   `no_nan_or_infinity_reaches_replicated_state` asserts no NaN or infinity
>   reaches the report or the snapshot. When the digest lands this check belongs
>   *inside* it, as a fatal error rather than a test.
>
> **Five real blockers. The first three are the same blocker wearing three
> hats: the engine has no command layer at all.**
>
> - **B1 · No round structure (T-30).** Sim §3's four phases and this spec's
>   protocol clock `r` have **no engine counterpart**. `Simulation::run()` drains
>   a discrete-event queue to a horizon; there is no round boundary to
>   synchronize at, no hidden simultaneous order, and no `apply_orders`. §5's
>   entire barrier — the keystone of the design — has nothing to attach to.
>   This is simulation work, not network work, and everything else waits on it.
> - **B2 · No card system (T-31).** §4.2's payload is
>   `card_id ‖ target_kind ‖ target_ref`; the engine has no cards, so `card_id`
>   indexes nothing. This is the same blocker that already stalls roster
>   enforcement (T-25), and R-NET4's field widths are blocked behind R-C1 in any
>   case.
> - **B3 · No state digest (T-32).** §8.1 names seven leaves; **two of them have
>   no engine representation at all** — `exchange_books` (matching.rs is built
>   but not wired into `lib.rs`, T-01) and `counter_graph` (T-13). The other five
>   exist but have no canonical encoding. Verification is this spec's core and it
>   is the furthest from buildable.
> - **B4 · `Knowledge` records *that* you scanned, not *what* you saw (T-33).**
>   This one the audit turned up, and it is the most interesting. `scanned` is a
>   `BTreeSet<PlanetId>`; every read through `view_of` then re-reads *current*
>   ground truth — `factors`, `density`, `population`, `owner`. So a scan from
>   500 years ago yields today's values with **zero lag**. That is precisely
>   §2.1's causal failure: an agent acting on information that has not reached
>   it. It is **not** a desync risk — every client computes the same wrong thing
>   — so it will not show up in a checkpoint. It is a game-correctness bug that
>   §2.1 promotes to a design-law violation, and the fix (store observed values
>   with an as-of round) is the same storage R-SIM4/T-26 flags as expensive at
>   fleet scale.
> - **B5 · Colonization filters on instantaneous global ownership (T-34).**
>   `sim.rs`'s candidate loop skips a world when `world.owner.contains(e)`,
>   regardless of whether the player has observed the claim. The *survey* path
>   documents its equivalent choice and defers it to R-SIM3; the colonization
>   path does not, and it is the one that matters — it is a read of a rival's
>   state with no observation behind it.
>
> **Not blockers, deliberately.** §3 topology, §4 wire format, §7 genesis, §9
> relay/reconnection and §10 server posture need no engine change — they are
> client and service work sitting outside the crate, and the engine's only
> obligations to them (determinism, one inbound entry point, no host access) are
> met or cleared above.

---

## 0. Premises, restated as constraints

| Premise | Consequence for this spec |
|---|---|
| Every player **and observer** runs the sim | State is replicated, not authoritative-server; the network carries *inputs*, never state |
| Server runs **no** sim | Server work per request is O(1) and independent of game complexity — the single strongest DoS property in the design |
| Game is peer-to-peer; server is lobby only | Browser P2P ⇒ WebRTC, because it is the only P2P transport in the web platform |
| Protocol carries only **card ID + target ID** | Frames are fixed-width; there is no variable-length parser to attack |
| Lobby publishes an agreed card list | The card list is content-addressed and its hash is inside the signed session identity |
| Multiple verification regimes per lobby | Verification is a *policy* pinned in the session descriptor, evaluated as a pure function of the message log |
| Command layer presents **perfect information** | The only secret in the protocol is the in-round simultaneous order; everything else is public by design |
| The simulation must **not** act on that perfect information | The presentation seam is read-only and one-directional (§2.1) |
| Web standards wherever possible | WebRTC / SCTP / DTLS / ICE / STUN / TURN / WebSocket / WebTransport / WebCrypto / WASM / CBOR |

**The enabling design decision is already made.** Card contract §1 — *"a card is `(costs,
target)`, and targets are drawn from a determinable finite set"* — is what makes a
fixed-width, ~20-byte-payload protocol possible at all. A game with free-numeric input,
freeform coordinates, or "choose any N of M" could not have this network architecture.
Everything below is downstream of that rule.

---

## 1. The keystone: round-barrier lockstep, not tick lockstep

Sim §3 gives four phases per round: income & growth → **command** (hidden, simultaneous) →
**resolution** (the theater) → aftermath. The network synchronizes **only at the command
boundary**. Between barriers, every client independently simulates the resolution and
renders it at whatever pace it likes — presentation time is already decoupled from
simulation tick duration.

Three clocks, kept strictly apart:

- **Sim clock** (years) — deterministic, advanced by the discrete-event queue. Never read from the host.
- **Presentation clock** (wall time) — per-client, purely cosmetic, may run fast or slow.
- **Protocol clock** (round index `r ∈ ℕ`) — the *only* clock the network knows.

Consequences worth stating flatly:

- **No rollback, no prediction, no delay-frames, no GGPO.** A 300 ms RTT costs 300 ms once
  per round, not once per frame. This is a turn-based network problem wearing a real-time
  costume, and it tolerates network conditions that would destroy an RTS.
- **Behind-ness is a presentation problem, not a sync problem.** A client still watching
  round *r*'s theater while others are committing to *r+1* is fine; it must only reach the
  barrier before it can commit.

### 1.1 The wall-clock prohibition

**No state transition may depend on any client's local clock.** Every transition —
including timeouts — is derived from the contents of the signed message log. Local clocks
determine only *when a client emits a message*, never what the state becomes. Violating
this is the most common way lockstep architectures desync, and it is not recoverable after
the fact.

---

## 2. Trust model — what full replication can and cannot protect

Three distinct properties, with three different answers. Conflating them is the standard
mistake in P2P anti-cheat design.

**(a) Integrity of shared state — protected.**
Any client that alters resolution produces a different state digest at the next checkpoint.
To hide that, it would have to also compute the honest state, which means running the honest
sim, which means its cheat had no effect. The detection is therefore sound in the direction
that matters: *a cheat that changes the game is a cheat that changes the digest.*

**(b) Confidentiality of in-round orders — protected, by commit–reveal (§5).**
With a mandatory ≥128-bit salt. Not optional: card contract §1 guarantees the order space is
small, finite, and enumerable *by design*, so an unsalted commitment is brute-forced
essentially instantly. The rule that makes the protocol small is the same rule that makes
the naïve commitment worthless.

**(c) Persistent board state — public by design; there is nothing to protect.**

*(Rev 2 — supersedes Rev 1, which invented a fog-of-war secrecy requirement the design does
not have.)* The command layer presents **perfect information**: the board is fully visible,
as in the tabletop lineage the game descends from. There is no fog to hack, so the cheat
class Rev 1 flagged as undetectable does not exist.

The confidentiality surface of the entire protocol is therefore **(b) alone** — one salted
hash per seat per round, protecting the simultaneous-order beat and nothing else. Three
open questions collapse rather than get answered: observer-feed delay, transcript-publication
privacy, and referee authority for ranked play all existed only to manage secrets that are
not kept (§12).

**(d) Collusion between players** (out-of-band voice, shared screens) is out of scope for
any protocol, here or anywhere.

**(e) Binary attestation is weak by itself.** A client self-reports its WASM hash; a
determined cheat reports the honest one. Attestation catches accidental version skew and
lazy cheats. The checkpoint digest catches the rest — of class (a) only.

**(f) Computation advantage — the only cheat class perfect information leaves standing, and
the design already disarms it.** With the whole board visible and a computable argmax (card
contract §6), a modified client could enumerate `V(card, target, state)` over every legal
pairing. But the design already intends the official client to do exactly that — *"the UI
tells you which effects can be played this turn."* **Surfacing the analysis is what removes
the incentive;** withholding it is what would create a solver market. And argmax assumes a
*believed* opponent state, so the hidden simultaneous order — class (b) — is precisely what
no solver resolves. The yomi layer survives full analysis by construction.

### 2.1 The seam is one-directional — perfect information must not flow inward

Perfect information at the presentation layer does **not** license the simulation to act on
it. The sim's in-world agents — autopilots, production centers, fleets — decide from
**light-lagged, player-relative knowledge** only. If they read the global instantaneous
state instead, two things break, in two different senses of the word:

- **Causally.** Card contract §2 models the theater as a discrete-event schedule in which
  every causal edge carries a light-travel delay, and the light-cone gap *is* the
  counterplay window. An agent acting on information that has not yet reached it does not
  merely cheat — it deletes the mechanic.
- **Numerically.** Presentation state is *not* identical across clients: they render at
  different paces, a reconnecting client is mid-catch-up, viewports differ. Anything derived
  from the presentation that flows back into the sim injects per-client nondeterminism
  straight into the hashed state, which is the network-layer failure.

One rule guards both: **the presentation seam is read-only and one-directional.**
`Snapshot` (`src/snapshot.rs`) projects sim → presentation. The *only* inbound channel is
`apply_orders(round, &[(seat, card_id, target_kind, target_ref)])` (§11). No other value
derived from what the player can see may cross back.

Keep the resulting asymmetry deliberate and visible in the UI: **the human has perfect
information and plays under it; the human's empire does not, and executes under light-lag.**
That gap is the game.

**R-NET13** — ratify the one-directional seam as a design law alongside mass conservation,
and audit the autopilot for any read of global state where a player-relative knowledge view
is required.

---

## 3. Topology

### 3.1 Player mesh — WebRTC

Browser peer-to-peer means [WebRTC](https://www.w3.org/TR/webrtc/) `RTCDataChannel`;
WebSocket and WebTransport are both client–server only. Data channels ride SCTP over DTLS
over UDP ([RFC 8831](https://www.rfc-editor.org/rfc/rfc8831),
[RFC 8261](https://www.rfc-editor.org/rfc/rfc8261)), so the transport is encrypted and
authenticated hop-by-hop as a mandatory property, not an option.

- **Deterministic degree-capped overlay, not a full mesh.** Each seat connects to
  `k = min(N−1, 6)` peers on a **circulant graph** whose offsets are pinned in the genesis
  descriptor, so every client computes the identical topology with no negotiation. At
  N ≤ 7 this *is* the full mesh — one code path covers both regimes. At N = 18 the
  recommended offsets are **C₁₈(1, 2, 7)**: 54 links instead of 153, diameter 2.

  | N | offsets | degree | diameter | links (vs. full mesh) |
  |---|---|---|---|---|
  | ≤7 | (1,2,3) | N−1 | 1 | full mesh |
  | 12 | (2,3) | 4 | 2 | 24 (66) |
  | 15 | (1,2,4) | 6 | 2 | 45 (105) |
  | 18 | (1,2,7) | 6 | 2 | 54 (153) |

  A diameter-2 overlay exists at degree ≤6 for every N in 3–18 (verified by exhaustive
  search over circulant offset sets), so the barrier costs at most two hops per broadcast
  regardless of seat count.

- **Why not full mesh at 18.** 153 links means 17 concurrent `RTCPeerConnection`s per
  client and 153 ICE negotiations at match start. At any realistic per-link failure rate,
  a *complete* mesh is statistically unattainable — which is survivable (see §9.2) but
  makes completeness a bad thing to design around. The overlay bounds per-client
  connections at 6 regardless of N, which is the property that actually matters.

- **Robustness.** Under independent per-link establishment failure, C₁₈(1,2,7) remains
  connected in 99.9% of trials at 20% link failure and 98.5% at 30% (4,000-trial Monte
  Carlo). Connectivity, not completeness, is the start condition: **begin the match when
  the overlay is connected and keep negotiating stragglers in the background.**

- **Not a star.** A relay hub is both a trust chokepoint and an availability chokepoint,
  and re-introduces the authoritative middle the design exists to avoid.
- **Seat binding.** A client opens data channels *only* to the Ed25519 public keys listed
  in the session descriptor. Inbound connections from unlisted identities are refused
  before any application data is read.

**R-NET3 — resolved (Rev 3):** 18 seats, degree-capped overlay per the table above.
**R-NET14** — pin the exact offset table in genesis, and extend `tests/determinism.rs`
(currently topping out at 12 seats) to 18.

### 3.2 Observers — fan-out tier, never the mesh

Observers must not join the player overlay: an unbounded observer count would let an
attacker impose O(observers) connection and verification load on each *player*. Because
every frame is signed and the log is self-verifying, **every relay in the observer tier is
untrusted** — it can withhold or delay frames but cannot forge them.

**R-NET2 — resolved (Rev 4): no server relay. The spectator tier is peer-to-peer gossip.**
The server's role stays exactly what it is for players — rendezvous and signaling — and it
carries no game data plane at all (§10).

- **Capped ingress.** Each seat accepts at most `m_ingress` spectator links (recommend 2,
  pinned in genesis) — 36 tier-0 slots at 18 seats. Capped, not zero: the original
  objection was to *unbounded* player-side load, and 2 links carrying 7.6 kB per round is
  negligible and uniform across seats.
- **Unstructured gossip below that.** Each spectator maintains a target degree
  `d ≈ 4–6` to other spectators, plus an ingress link if a slot is free. Frames propagate
  by the *same* flood-with-dedup rule as the player overlay (§9.2) — one code path.
- **Rendezvous, not relay.** A joining spectator asks the lobby for a random sample of
  currently-connected spectators. This is the signaling role the server already performs;
  no frame ever transits it.
- **Backfill is reconnection.** A joining spectator requests missing history from any peer
  holding it — the frames are signed, so any source works (§9.3).

Cost, at 18 seats and 7.6 kB of frames per round:

| | server relay (rejected) | gossip tier |
|---|---|---|
| server egress, 1 000 spectators, 60 rounds | 0.47 GB/match | **0** |
| server egress, 100 000 spectators | 46.7 GB/match | **0** |
| per-spectator upload (d=5, round every 30 s) | — | 1.3 kB/s |
| depth to 100 000 spectators (d=5) | 1 hop | ~8 hops |

Latency is the thing traded away, and it is the thing observers do not need: presentation
is decoupled, spectators have no barrier to meet, and a lagging spectator simply replays.

**Churn and eclipse — the failure mode changes shape.** A server relay fails all-or-nothing;
a gossip mesh *partitions*, stalling a subgroup while the rest stay current. Two
consequences:

- **Self-heal by re-sampling.** A spectator whose round lag exceeds a threshold re-samples
  peers from the rendezvous and rewires. This also detects **eclipse**: an attacker running
  many spectator nodes can surround a target and *withhold* (it can never forge — the
  frames are signed), and the tell is that freshly-sampled peers report a higher round.
- **Hard-NAT spectators stratify into leaves.** A symmetric-NAT spectator can still reach
  the reachable population, so it connects outbound and simply serves no one. TURN demand
  therefore does not scale with spectator count the way it would in a mesh of only hard-NAT
  peers.

**R-NET17** — is a lobby-published liveness beacon (current round + checkpoint root,
signed) worth it as a second eclipse tell? It is a few bytes per round, but it does put
per-round data on the server, which is exactly what this decision removed. Re-sampling
already covers the case.

**R-NET18** — ratify `m_ingress` and spectator degree `d`.

### 3.2.1 Spectators are not observer seats

The no-relay decision forces a distinction that Rev 3 left implicit:

- **Observer seats** (the Refereed policy's weighted voters, §8.2) run the sim, publish
  signed checkpoints, and never submit orders. Their verdict is load-bearing, so they must
  **join the player overlay as full members** — a seat whose vote decides matches cannot
  depend on a tier where withholding is possible.
- **Spectators** carry no weight and live in the gossip tier. Withholding degrades their
  view and nothing else.

A weighted observer placed in the gossip tier is an eclipse target with a vote. Keep the
two populations separate in the genesis descriptor.

**R-NET12 — closed (Rev 2).** No observer-feed delay is needed. Under perfect information
(§2c) an observer sees exactly what every player sees, so out-of-band relay conveys nothing;
and reveals unlock only after every commit is locked, so an observer cannot be ahead of the
barrier either.

### 3.3 NAT traversal

ICE / STUN / TURN per [RFC 8445](https://www.rfc-editor.org/rfc/rfc8445),
[RFC 8489](https://www.rfc-editor.org/rfc/rfc8489),
[RFC 8656](https://www.rfc-editor.org/rfc/rfc8656). Published estimates of the share of
WebRTC sessions that cannot establish a direct path and require a TURN relay span roughly
10–30% and disagree with each other; the figure is population- and network-dependent, so
treat it as a range to provision against rather than a number to plan on.

**At 18 seats, overlay routing largely substitutes for TURN.** With 18 independent seats,
the probability that *at least one* cannot establish a direct path approaches certainty
across that range — but because frames are end-to-end signed, a seat unreachable by one
peer is still reachable through another (§9.2). TURN is therefore needed only for a seat
that can reach **no** peer at all, which is a far rarer condition than "some link failed."
Practical consequence: TURN must exist and be metered, but the design should not budget
for `(TURN-only seats × k)` relayed links.

---

## 4. Wire protocol — one fixed-size frame

The hardening strategy is to have no parser. **Every peer message is exactly 144 bytes.**
No length prefixes, no optional fields, no nesting, no variable-length anything. A message
of any other length is discarded before it is looked at.

### 4.1 Layout (little-endian)

| Offset | Size | Field |
|---|---|---|
| 0 | 1 | `version` — must equal the genesis-pinned value |
| 1 | 1 | `kind` — enum, closed set |
| 2 | 2 | `seat` — u16 index into the genesis seat table |
| 4 | 4 | `round` — u32 protocol round index |
| 8 | 32 | `session_id` — SHA-256 of the genesis descriptor |
| 40 | 32 | `payload` — kind-specific, zero-padded |
| 72 | 8 | `reserved` — MUST be zero |
| 80 | 64 | `signature` — Ed25519 over bytes 0..80 |

Ed25519 is now available natively in WebCrypto across the major engines (Chrome 137 in May
2025 completing the set after Firefox and Safari 17), so no signing library needs to be
bundled — see [RFC 8032](https://www.rfc-editor.org/rfc/rfc8032) and the
[WICG Secure Curves](https://wicg.github.io/webcrypto-secure-curves/) spec.

### 4.2 Payload variants (32 bytes, remainder zero)

| `kind` | Payload |
|---|---|
| `ORDER_COMMIT` | `SHA-256(session_id ‖ round ‖ seat ‖ card_id ‖ target_kind ‖ target_ref ‖ salt)` |
| `ORDER_REVEAL` | `card_id: u16` ‖ `target_kind: u8` ‖ `target_ref: u32` ‖ `salt: [u8;16]` ‖ zeros |
| `CHECKPOINT` | `state_root: [u8;32]` (§8.1) |
| `CHECKPOINT_LEAVES` | leaf digest vector, truncated — divergence localization only, sent on demand |
| `TIMEOUT_VOTE` | `subject_seat: u16` ‖ `phase: u8` ‖ zeros |
| `HAVE` | possession bitmaps: `have_commit: u32` ‖ `have_reveal: u32` ‖ `have_checkpoint: u32` ‖ zeros — **u32, not u16 (Rev 3): 16 bits cannot address 18 seats** |
| `VERDICT_VOTE` | `subject_seat: u16` ‖ `verdict: u8` ‖ `evidence_round: u32` ‖ zeros — only under `HaltAndVote` policy |

`target_kind` + `target_ref` is the encoding of card contract §1's target set
(`self/global`, one neighbor empire, one held hex, one frontier hex, a menu selection,
`none`). **R-NET4** — final field widths are blocked on **R-C1** (the closed list of legal
`target_rule` kinds).

**The card/target constraint is honored.** The only *game* content on the wire is `card_id`
and the target pair. Everything else is envelope (session, seat, round, signature) or
verification metadata (a hash, a bitmap, a vote). No derived game state — no positions, no
resources, no fleet composition — is ever transmitted.

### 4.3 Receive discipline

Checks in this order, cheapest first, so a flood costs the attacker more than the defender:

1. Length **exactly** 144, else drop.
2. `version` matches, else drop.
3. `session_id` matches ours, else drop.
4. `reserved` is zero and the payload's declared padding is zero, else drop. *(Non-zero
   padding is a protocol violation, not a courtesy — it kills covert channels and sloppy
   encoders in one rule.)*
5. `seat < N`, else drop.
6. `round ∈ [current − W, current + 1]` for a fixed window `W` (e.g. 3), else drop. This
   is what bounds memory.
7. Rate: per-peer token bucket. Frames from a peer bearing `seat ≠ that peer` are *relayed*
   frames (§9) and draw on a separate, smaller budget.
8. **Ed25519 verification** — last, because it is the expensive step.
9. Store at most one frame per `(seat, round, kind)`.

**Memory is bounded and preallocated.** `W × N × K × 144` bytes — at W=3, N=12, K=7 that is
≈ 36 KB, allocated once at session start. There is no dynamic-growth path in the receive
path, which is the actual answer to "robust against buffer overflow": there is no buffer to
overflow and no allocator to exhaust.

### 4.4 Equivocation is proof, not opinion

Two *differing* signed frames for the same `(seat, round, kind)` are non-repudiable
evidence of misbehavior — no vote required, no interpretation possible. Both frames are
retained, broadcast as an evidence pair, and trigger the policy's equivocation action
immediately. `ORDER_COMMIT` and `CHECKPOINT` are the meaningful cases; `ORDER_REVEAL`
equivocation is self-limiting since only one reveal can match the commitment.

---

## 5. The round barrier: commit → reveal → resolve → checkpoint

Per round `r`:

- **P0 · COMMIT.** Each seat broadcasts `ORDER_COMMIT(r)`.
- **P1 · REVEAL.** A seat broadcasts `ORDER_REVEAL(r)` **only once it holds commits from
  every live seat.** The gate is a log-derived condition, not a timer — which is what makes
  the simultaneity fair regardless of latency. There is no last-mover advantage because
  there is no last mover.
- **P2 · RESOLVE.** With all reveals in and verified against their commitments, every
  client applies orders **in seat-index order** and runs the resolution. Identical inputs,
  identical order, identical outputs.
- **P3 · CHECKPOINT.** Broadcast `CHECKPOINT(r)`; evaluate policy (§8).

### 5.1 Illegal orders are coerced, never rejected

Legality (card in the published list, target legal given shared state, costs affordable) is
a **pure function of state every client already has**. An illegal order is therefore
replaced by the deterministic default order — `pass` / target `none` — identically by every
client, with no message exchanged. Rejecting instead of coercing is how this class of
system desyncs.

### 5.2 Liveness: the timeout is a message, not a clock

A seat that commits and then declines to reveal would otherwise stall everyone. Any seat
may broadcast `TIMEOUT_VOTE(subject, phase, r)` once its own patience expires. When the log
contains timeout votes from a policy-defined quorum (default: ⌊N/2⌋+1, excluding the
subject), the subject's order for `r` becomes the default order and play continues.

Because the trigger is *"a quorum of signed votes is present in the log,"* every client
makes the same call at the same logical point despite differing wall clocks. The UI
countdown drives when *you* emit a vote; it never drives what the state becomes.

Residual: a seat can commit, watch the reveals, dislike its own order, and eat the default.
Since the default is `pass`, this is strictly worse than revealing in nearly every case, so
it is not an exploit — but the abandoned commit stays in the log, so a serial abuser is
visible in the transcript.

**At 18 seats the timeout path stops being an error path.** The chance that *at least one*
seat times out in a given round is `1 − (1−p)ᴺ`: at a per-seat rate of 2% that is 11.4% of
rounds at N=6 but **30.5% at N=18**. Design it as a routine, low-friction, well-presented
transition — not an exception handler. This also sharpens **R-NET6**: at 18 seats, whether
a defaulted round burns the action is a balance decision that will fire in roughly a third
of rounds, not a corner case.

### 5.4 The live-seat set is a sim fact, not a network fact

Progressive elimination means N *shrinks* during the match, and every quorum in this spec —
reveal gating (§5), timeout quorum (§5.2), checkpoint partition (§8.2) — is taken over
**live seats**. So liveness must be derived from simulation state, which every client
computes identically, and **never from connection state**, which differs per client by
construction.

Concretely: elimination is a deterministic sim event at a specific round; a client whose
peer link to seat 11 is down does *not* thereby treat seat 11 as eliminated. Getting this
backwards makes the timeout quorum itself client-dependent, which diverges the verdict and
then the state — a desync that would present as intermittent and unreproducible.

**Eliminated seats demote out of the overlay into the fan-out tier** (§3.2). They keep
running the sim as observers; they stop holding overlay links and stop counting toward
quorum. Peak network load is therefore at match start and monotonically decreases, which
is the favorable direction.

### 5.3 Dropout: the autopilot is the drop-out policy

A permanently disconnected seat is handed to the engine's deterministic autopilot
(`BaselineAutopilot`) at a specific round pinned by quorum vote. Every client runs the same
autopilot from the same state and gets the same orders — so a dropped player generates
**zero network traffic** and zero divergence risk. This is a free property of having built
the MC balancer's autopilot first.

**R-NET6** — does a defaulted or timed-out round consume the seat's action, or refund it?

---

## 6. Determinism hazards in the browser

The engine is already bit-reproducible under `rustc` (see `tests/determinism.rs`, which
compares `f64::to_bits()` on densely-sampled positions). Porting that guarantee to WASM
across heterogeneous browsers and CPUs introduces hazards that are not in the current test
suite.

**H1 · One binary, hash-pinned.** All seats and observers run the *same* `.wasm` module.
The genesis descriptor pins its SHA-256; a client reporting a different hash is refused.
Serve it with [Subresource Integrity](https://www.w3.org/TR/SRI/) so the browser enforces
the hash independently. *This is the single most important rule in this section* — it makes
every remaining hazard a build-configuration question rather than a runtime one.

**H2 · Relaxed SIMD is forbidden.** The proposal exists specifically to *introduce* local
nondeterminism whose results vary with hardware support — fused-multiply-add rounding,
reciprocal approximations, NaN and out-of-range conversion behavior all become
implementation-defined. Build with it disabled
([proposal overview](https://github.com/WebAssembly/relaxed-simd/blob/main/proposals/relaxed-simd/Overview.md),
[Nondeterminism.md](https://github.com/WebAssembly/design/blob/main/Nondeterminism.md)).

Plain `simd128` is deterministic and, because a single fixed binary is shipped,
autovectorization differences across toolchains cannot arise. **R-NET10** — enable
`simd128` or not.

**H3 · NaN payloads are nondeterministic in core WASM.** The spec picks arithmetic-NaN
payloads nondeterministically, so NaN *bits* differ across runtimes even without relaxed
SIMD. Any NaN that reaches the state digest is a latent, intermittent, unreproducible
desync. `matching.rs::upsert` already `debug_assert!`s finiteness; promote that discipline:
**NaN entering hashed state is a fatal error, not a value.**

**H4 · Transcendentals are fine — because they are in the module.** WASM has no
`sin`/`cos`/`exp`/`pow` instructions; for `wasm32-unknown-unknown` these are compiled into
the binary. Under H1 that makes them bit-identical everywhere, which is *better* than the
native-target situation where the platform libm varies. The corresponding rule: **no float
may cross the JS boundary inbound.** `Math.*` in JS is engine-provided and not required to
be correctly rounded.

**H5 · No host clock, no host entropy.** No `Date.now()`, `performance.now()`, or
`Math.random()` inside the sim. Already satisfied (`log.rs` deliberately stamps with the
sim clock).

**H6 · No threads.** WASM threads plus `SharedArrayBuffer` introduce scheduling
nondeterminism. The sim runs single-threaded in a dedicated Worker. This also avoids the
COOP/COEP header requirements entirely.

**H7 · The RNG stream is a serialization point.** The wreck roll is the only stochastic
element; a divergence in the *number* of draws diverges everything downstream. Draws must
remain structurally deterministic (per-ship, entity-index order) — the existing discipline,
now load-bearing for network integrity rather than only for MC reproducibility.

**H9 · No presentation backflow.** Per §2.1. This is the hazard H1–H7 do not cover, because
it is not a floating-point issue: it is per-client *state* — render pace, catch-up progress,
viewport — reaching the sim. Enforce it structurally by giving the engine exactly one
inbound entry point, not by convention.

**H8 · What is safe:** `f64` `+ − × ÷ √` are correctly rounded and deterministic in WASM.
That, plus H1–H7 and H9, is the whole foundation.

---

## 7. Session genesis and the card-list agreement

The **genesis descriptor** is a canonically-encoded [CBOR](https://www.rfc-editor.org/rfc/rfc8949)
record (deterministic encoding profile) containing:

```
protocol_version        u16
wasm_module_sha256      [u8;32]
cardlist_sha256         [u8;32]
ruleset_sha256          [u8;32]   // SimConfig + GalaxyConfig + all MC-tuned constants
galaxy_seed             u64       // §7.1
sim_seed                u64
seats                   [{ index: u16, pubkey: [u8;32], name: str }]   // ordered
observer_policy         { fanout, delay_rounds }
verification_policy     § 8.2
timing_hints            { ... }   // advisory only; never state-bearing
```

`session_id = SHA-256(canonical CBOR)`. Every seat signs `session_id` with its Ed25519 key,
and **the lobby publishes the descriptor together with the full signature set.**

That signature set *is* the card list agreed to by all players, and it is non-repudiable:
the card list hash is inside `session_id`, so no seat can be running a different list
without failing both attestation and signature verification, and no seat can later claim it
did not agree to the list it signed.

### 7.1 The galaxy seed must be jointly random

If the lobby picks the seed, a colluding lobby operator can shop for a galaxy favorable to
one seat. Use the same commit–reveal machinery: each seat contributes 32 random bytes,
commits during lobby assembly, reveals after all commits land, and
`galaxy_seed = SHA-256(reveals ‖ in seat order)`. No participant controls the galaxy, and a
non-revealer is simply excluded before the match begins.

---

## 8. Verification

### 8.1 The state digest

A Merkle root over per-subsystem leaf digests, so divergence *localizes* instead of merely
being detected:

- **Leaves:** `galaxy`, `players`, `vehicles`, `event_queue`, `rng_cursor`,
  `exchange_books`, `counter_graph` — each SHA-256 over a canonical encoding: fixed-width,
  little-endian, index-ordered, `f64` as `to_bits()`.
- **Root:** SHA-256 over the leaf digests in fixed order.

Two rules that are easy to get wrong:

- **Digest the authoritative state, not the `Snapshot`.** `Snapshot` is the presentation
  seam and is a *projection*; digesting it lets divergence hide in state the projection
  drops. The event queue and RNG cursor in particular must be inside the digest.
- **On mismatch, exchange `CHECKPOINT_LEAVES`.** This converts "desync at round 41" into
  "the `event_queue` leaf diverged at round 41," which is the difference between a bug
  report and a coordinate.

**R-NET5** — final leaf partition and checkpoint cadence.

### 8.2 The checkpoint frame *is* the ballot

No separate voting round is needed for divergence. Every client holds the same set of
signed `CHECKPOINT(r)` frames, partitions seats by root value, and applies the policy
function. The verdict is therefore a **pure function of the log** and identical everywhere.
`VERDICT_VOTE` exists only for the `HaltAndVote` policy, where a human judgment is
deliberately inserted.

```
VerificationPolicy {
  cadence:        EveryRound | EveryKRounds(k),
  quorum:         SimpleMajority | Supermajority(n, m) | Unanimous | ObserverWeighted { weights },
  on_divergence:  ContinueWithQuorum { eject_minority } | HaltAndVote | CancelMatch,
  on_equivocation: Eject | CancelMatch,     // never "continue" — equivocation is proof
  attestation:    Required | Optional,
}
```

Named presets matching the three regimes:

| Lobby | cadence | quorum | on_divergence |
|---|---|---|---|
| **Public** | EveryRound | SimpleMajority | ContinueWithQuorum { eject_minority } |
| **Ranked** | EveryRound | Supermajority(2, 3) | ContinueWithQuorum { eject_minority }; CancelMatch on equivocation |
| **Refereed** | EveryRound | ObserverWeighted (all weight on k designated observer seats) | configurable |

**Rev 3 — `Unanimous + CancelMatch` is retired as the ranked preset.** It does not survive
the jump to 18 seats. A single seat can void a match by publishing a false root, and the
probability that some seat does so is `1 − (1−q)ᴺ`:

| per-seat grief rate q | N=6 | N=12 | N=18 |
|---|---|---|---|
| 0.5% | 3.0% | 5.8% | 8.6% |
| 1% | 5.9% | 11.4% | **16.5%** |
| 2% | 11.4% | 21.5% | **30.5%** |

Voiding one match in six is not a competitive integrity feature. Supermajority(2/3) needs
12 of 18 colluders to force a wrong verdict and 7 to deadlock, and `CancelMatch` is
reserved for equivocation, which is proof rather than opinion (§4.4).

Note the asymmetry the seat count creates: **majority-style quorums get stronger as N
grows** (10 colluders of 18 is a much higher bar than 4 of 6) while **unanimity gets
weaker** (still one veto, more seats holding it). The two policies move in opposite
directions under the same change.

An **observer seat** runs the sim and publishes signed checkpoints but never submits
orders. That is the mechanism for "vest all voting power in one or more observers."

### 8.3 Honest limits of the vote

- **This is divergence detection, not Byzantine consensus.** With f colluding seats,
  `SimpleMajority` fails once f ≥ ⌈N/2⌉ — the colluders can eject an honest player.
  `Unanimous` inverts the failure mode: any single seat can void a match by publishing a
  false root. There is no policy that is robust to both, and pretending otherwise would be
  a design error.
- **But both failure modes are identifiable.** The transcript names who signed what.
- **R-NET8** — under `SimpleMajority`, what happens on an even split? (Recommend `Halt`;
  a seat-order tiebreak rewards whoever bribed seat 0.)

### 8.4 The transcript is the real anti-cheat

`genesis + every frame` is a complete, signed, self-verifying match record. At 144 bytes ×
N seats × ~3 primary kinds × R rounds — e.g. 8 seats, 60 rounds ≈ 207 KB — it is trivially
archivable and shareable.

**Anyone holding the pinned binary can replay it offline and determine the truth
independently of any vote.** That makes enforcement *forensic* rather than *preventive*,
which is the strongest honest guarantee this architecture can offer. Transcript export
should be a first-class client feature, not a debug flag.

**R-NET9 — closed (Rev 2).** Transcripts publish freely. Under perfect information (§2c) a
transcript reveals nothing that was not already on every screen; the only secrets in it are
the per-round commit salts, which are revealed in the same round by construction. Retention
is a storage question, not a privacy one.

---

## 9. Reliability, relay, reconnection

### 9.1 Redundant idempotent broadcast, not SCTP reliability

Configure the data channel **unreliable and unordered** (`ordered: false`,
`maxRetransmits: 0`; see [`RTCDataChannel`](https://developer.mozilla.org/en-US/docs/Web/API/RTCDataChannel))
and supply reliability at the application layer: **each client sends a peer exactly the
frames that peer's `HAVE` bitmap says it lacks, re-sent on a backoff until possession is
confirmed.**

**Rev 3 correction — this must be difference-only, not "rebroadcast everything you hold."**
The Rev 2 phrasing is what breaks first at 18 seats. Naïve rebroadcast of the whole
`N × K × W` window at 4 Hz costs, per client upload:

| N | naïve rebroadcast | difference-only, steady state |
|---|---|---|
| 6 | 0.36 MB/s | ~0 |
| 12 | 1.60 MB/s | ~0 |
| 18 | **3.70 MB/s** | ~0 |

Under difference-only, steady-state traffic is a client's own three frames to its k ≤ 6
peers — 2.6 kB per round at N=18 — plus `HAVE` gossip. Bytes go out only when something
was actually lost.

Why not simply use SCTP's reliable-ordered mode:

- Every frame is idempotent and keyed by `(seat, round, kind)`, so duplicates cost nothing
  and arrival order is irrelevant — the ordering guarantee buys nothing.
- Under 5–10% loss, head-of-line blocking costs more than re-sending a 144-byte frame.
- App-level redundancy survives a peer *restart*, which SCTP retransmission does not.

### 9.2 Any peer may relay any frame

Because frames are end-to-end signed, relaying is safe: **flood-with-dedup over the overlay
is the entire routing algorithm.** A frame is forwarded to any peer whose `HAVE` bitmap
lacks it; duplicates are discarded on the `(seat, round, kind)` key. If A↔B is broken but
A↔C and C↔B work, the round still completes. Under Rev 3's degree-capped overlay (§3.1)
this is no longer a resilience bonus — it is **load-bearing**, because the overlay is
deliberately incomplete and every frame reaches most seats by relay. (Hop-by-hop DTLS
protects each link; the Ed25519 signature is what survives the relay — both are needed.)

### 9.3 Reconnection and late join are the same operation

State is `f(genesis, frame log)`. A reconnecting client — or a late-joining observer —
requests missing frames from any peer, replays the sim headlessly from genesis, and
rejoins. No state transfer, no trust required, no special-case code path.

**R-NET7** — measure headless replay throughput against realistic match length. Headless
replay is much faster than watching (presentation is decoupled), but if a late-join replay
exceeds a few seconds, add snapshot-assisted catch-up: accept an *untrusted* snapshot whose
digest matches the quorum-agreed root at that round, then replay forward. The snapshot need
not be trusted because the root is the check.

---

## 10. Server architecture and DoS posture

Three services, none holding game state, none simulating.

1. **Lobby / API** — HTTPS (HTTP/3). Room lifecycle, genesis assembly, publication of the
   card list and hashes, signature collection. Room records are a few KB with a TTL.
2. **Signaling** — [WebSocket](https://www.rfc-editor.org/rfc/rfc6455) or WebTransport.
   Relays SDP offer/answer and trickle ICE candidates as **opaque size-capped blobs**, and
   serves the spectator rendezvous sample (§3.2). The server does not parse SDP; it has no
   reason to and parsing is attack surface.
3. **STUN / TURN** — ephemeral per-session HMAC-derived credentials (the standard TURN REST
   pattern), with expiry and a per-session byte cap.

**Rev 4: there is no game data plane.** With R-NET2 resolved against a server relay, no
protocol frame ever transits the server — not for players, not for spectators. The server
handles session setup and connection brokerage only. TURN remains the sole
bandwidth-metered service, and per §3.2 spectator TURN demand does not scale with spectator
count.

**The strongest DoS property is structural and free:** the server does no simulation and
holds no per-round state, so no request can make it do work proportional to game
complexity. Maximum work per request is bounded, small, and known in advance.

Layered on top:

- **Anycast/CDN** in front of the lobby API for L3/L4 absorption.
- **HTTP/3 over QUIC** — [RFC 9000 §8](https://www.rfc-editor.org/rfc/rfc9000) address
  validation and the 3× anti-amplification limit are inherited, not implemented.
- **Room creation is the only semi-expensive operation** — gate it behind a challenge token
  bound to the creator's Ed25519 identity; cap rooms per identity and per source.
- **Signaling** — per-connection message rate and size caps, connection caps per source,
  idle timeout, and *close the socket on the first malformed frame* rather than attempting
  recovery.
- **TURN is the only bandwidth-costly service.** Issue credentials only to sessions whose
  ICE actually failed direct connectivity; meter bytes; cap allocations. Given the 10–30%
  spread in published relay-requirement estimates (§3.3), provision for the high end but
  bill and rate-limit as if it were the exception it should be.
- **Store nothing that is not needed.** The lobby holds fixed-schema records only; no
  user-supplied data is ever interpreted as a path, query, or code.

---

## 11. Client hardening

- **Sim in a dedicated Worker.** The UI thread never touches the sim's linear memory
  directly, only projections.
- **The entire network→sim attack surface is one function:**
  `apply_orders(round, &[(seat, card_id, target_kind, target_ref)])`. It is total: every
  input maps to a legal state transition, illegal orders coerce to `pass` (§5.1). Nothing
  else from the network reaches the engine.
- **Frame decoding** uses a fixed-offset `DataView` reader after a length-equality check.
  No JSON, no dynamic schema, no `eval`.
- **Memory ceiling.** WASM memory is allocated with a fixed maximum and never grows during
  a match. Panics abort the session cleanly rather than continuing from indeterminate
  state.
- **CSP** without `unsafe-eval` (WASM needs `wasm-unsafe-eval`, which is a much narrower
  grant), plus SRI on the WASM and JS bundles (H1).
- Rust/WASM is memory-safe within its sandbox and cannot corrupt the host, so "buffer
  overflow" in the classical sense is not the threat — **resource exhaustion and logic
  confusion are**, and §4.3's fixed-size preallocated receive buffer plus the total
  `apply_orders` addresses both.

---

## 12. Open ratification points

| Code | Question | Blocked on |
|---|---|---|
| **R-NET1** | ~~Fog under full replication~~ — **closed (Rev 2):** the command layer presents perfect information; there is no fog to protect. Superseded by **R-NET13**. | — |
| **R-NET2** | ~~Server relay vs. gossip~~ — **resolved (Rev 4):** no server relay; P2P gossip tier with capped ingress (§3.2) | — |
| **R-NET3** | ~~12-seat cap~~ — **resolved (Rev 3):** 18 seats, degree-capped overlay (§3.1) | — |
| **R-NET4** | Frame payload field widths (`card_id`, `target_kind`/`target_ref`) | **R-C1** |
| **R-NET5** | Merkle leaf partition and checkpoint cadence | — |
| **R-NET6** | Does a defaulted/timed-out round consume the action or refund it? | R-2 (action economy) |
| **R-NET7** | Headless replay throughput vs. match length; snapshot-assisted catch-up needed? | — |
| **R-NET8** | Even-split behavior under `SimpleMajority` | — |
| **R-NET9** | ~~Transcript privacy~~ — **closed (Rev 2):** publishable; retention is a storage question. | — |
| **R-NET10** | Enable `simd128` in the pinned build? | — |
| **R-NET11** | Ratify H3 (NaN in hashed state is fatal) as a design law — **seeded:** `no_nan_or_infinity_reaches_replicated_state` guards the report and snapshot today; the fatal-error form waits on the digest | R-NET5 |
| **R-NET12** | ~~Observer feed delay~~ — **closed (Rev 2):** unnecessary under perfect information. | — |
| **R-NET13** | Ratify §2.1's one-directional seam as a design law; audit the autopilot for any read of global state where a player-relative knowledge view is required — **audit done:** the `Autopilot` trait structurally cannot reach global state, but the views handed to it can. Two findings, B4 (`Knowledge` stores membership, not observations, so every read is zero-lag ground truth) and B5 (colonization filters on instantaneous global ownership). See the engine-status block | T-33, T-34 |
| **R-NET14** | Pin the circulant offset table in genesis; ~~extend `tests/determinism.rs` from 12 to 18 seats~~ **done** — 18 seats bit-identical, +6 s | — |
| **R-NET15** | Measure the state-digest cost at an 18-seat galaxy. Planet count at 18 seats is unmeasured — do not extrapolate it from the 12-seat figure. If a full rehash per round is too slow, adopt dirty-chunk Merkle updates rather than lengthening the cadence | R-NET5 |
| **R-NET16** | Match-start admission rule: what overlay connectivity threshold begins play, and how long stragglers keep negotiating before the seat is dropped to autopilot (§5.3) | R-NET14 |
| **R-NET17** | Lobby-published liveness beacon as a second eclipse tell — worth putting per-round data back on the server, or does re-sampling suffice? | R-NET2 |
| **R-NET18** | Ratify `m_ingress` (spectator links per seat) and spectator gossip degree `d` | R-NET2 |

---

## References

**Transport & NAT traversal**
- WebRTC 1.0 (W3C): https://www.w3.org/TR/webrtc/
- `RTCDataChannel` — reliability configuration (`ordered`, `maxRetransmits`, `maxPacketLifeTime`): https://developer.mozilla.org/en-US/docs/Web/API/RTCDataChannel
- RFC 8831 — WebRTC Data Channels: https://www.rfc-editor.org/rfc/rfc8831
- RFC 8832 — Data Channel Establishment Protocol: https://www.rfc-editor.org/rfc/rfc8832
- RFC 8261 — SCTP over DTLS: https://www.rfc-editor.org/rfc/rfc8261
- RFC 8445 — ICE: https://www.rfc-editor.org/rfc/rfc8445
- RFC 8489 — STUN: https://www.rfc-editor.org/rfc/rfc8489
- RFC 8656 — TURN: https://www.rfc-editor.org/rfc/rfc8656
- RFC 6455 — WebSocket: https://www.rfc-editor.org/rfc/rfc6455
- WebTransport (W3C): https://www.w3.org/TR/webtransport/
- WebTransport reaching Baseline with Safari 26.4, March 2026: https://webrtc.ventures/2026/04/webtransport-is-now-baseline-what-it-means-for-real-time-media/
- RFC 9000 — QUIC (§8 address validation / anti-amplification): https://www.rfc-editor.org/rfc/rfc9000
- TURN relay-requirement estimates (sources disagree; ~10–20% vs. ~30%): https://bloggeek.me/webrtcglossary/nat/ · https://www.rtcinsights.com/blog/stun-turn-configuration/

**Cryptography**
- Web Cryptography API: https://www.w3.org/TR/WebCryptoAPI/
- WICG Secure Curves in the Web Cryptography API (Ed25519, X25519): https://wicg.github.io/webcrypto-secure-curves/
- Ed25519 completing major-browser support with Chrome 137 (May 2025): https://blogs.igalia.com/jfernandez/2025/08/25/ed25519-support-lands-in-chrome-what-it-means-for-developers-and-the-web/
- RFC 8032 — EdDSA / Ed25519: https://www.rfc-editor.org/rfc/rfc8032
- Commitment scheme (binding + hiding; why the salt is mandatory): https://en.wikipedia.org/wiki/Commitment_scheme
- Merkle tree: https://en.wikipedia.org/wiki/Merkle_tree

**WebAssembly determinism**
- WebAssembly design — Nondeterminism (the authoritative list of where the spec admits it): https://github.com/WebAssembly/design/blob/main/Nondeterminism.md
- Relaxed SIMD proposal overview (explicitly introduces hardware-dependent results): https://github.com/WebAssembly/relaxed-simd/blob/main/proposals/relaxed-simd/Overview.md
- Core spec — NaN propagation is nondeterministic: https://webassembly.github.io/spec/core/exec/numerics.html#nan-propagation
- Subresource Integrity: https://www.w3.org/TR/SRI/
- CSP `wasm-unsafe-eval`: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Security-Policy/script-src
- IEEE 754: https://en.wikipedia.org/wiki/IEEE_754

**Encoding**
- RFC 8949 — CBOR (§4.2 deterministic encoding): https://www.rfc-editor.org/rfc/rfc8949

**Prior art**
- Bettner & Terrano, *1500 Archers on a 28.8: Network Programming in Age of Empires and Beyond* — the canonical deterministic-lockstep write-up, including the desync-diagnosis problem §8.1 addresses: https://www.gamedeveloper.com/programming/1500-archers-on-a-28-8-network-programming-in-age-of-empires-and-beyond
- Lockstep protocol: https://en.wikipedia.org/wiki/Lockstep_protocol

**Internal**
- `Hyades_card_contract.md` §1 (no arbitrary input — the precondition for this protocol), §6 (deterministic value), §7 (MC balancing)
- `Hyades_simulation_model.md` §3 (round loop), §4 (deterministic combat + wreck roll)
- `hyades-engine` `tests/determinism.rs`, `src/snapshot.rs`, `src/log.rs`, `src/matching.rs`
