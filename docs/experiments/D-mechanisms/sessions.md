# §D. Mechanisms — sessions and relays

*Part of the experiments record. Nothing here is normative. Table of contents: [`../README.md`](../README.md); how the record is organized: [`../AGENTS.md`](../AGENTS.md).*

---

## D.60 Relay load against published relay limits (T-164, R-SES16)

Supports `Hyades_sessions_discovery_and_security.md` §4.3.1. An estimate, not a
measurement: no relay was contacted. The container's network policy refused
every relay host tried (relay.damus.io, nos.lol, relay.primal.net,
relay.nostr.band, nostr.wine, relay.snort.social, offchain.pub, nostr.mom,
relay.nostr.bg, purplepag.es — `CONNECT tunnel failed, response 403`), so no
NIP-11 document of a named relay was read.

**Event size.** A NIP-01 `["EVENT", …]` message carrying one 144-byte frame,
base64-encoded, with three tags (topic, room, NIP-40 expiration), serialized
without whitespace: **629 bytes**; two frames: 821 bytes. Computed with a
throwaway Python script.

**Match shape.** `horizon_years = 4000`, `years_to_first_round = 200`,
`years_per_round = 400` (`src/sim.rs`): barriers at 200, 600, …, 3,800 — **10
per match**. A 30–45 minute match is 180–270 s of wall time per round if the
barriers divide it evenly (estimate).

**Published limits, as configured by default or in an example** (none is a
named relay's live setting):

| source | limit |
|---|---|
| khatru `policies/sane_defaults.go`, `ApplySaneDefaults` | `EventIPRateLimiter(2, time.Minute*3, 10)` — read as 2 tokens per 3 minutes, bucket 10 (token-bucket semantics inferred from the parameter names; the implementation was not read) |
| the same function | `FilterIPRateLimiter(20, time.Minute, 100)` (subscription filters per IP) and `ConnectionRateLimiter(1, time.Minute*5, 100)` (connections per IP), read the same way |
| noteguard README (Damus's strfry write-policy plugin) | `posts_per_minute = 8`, per IP |
| rzazo24/nostr-relay-khatru README | 30 events per minute per IP, burst 60 |
| nostr-rs-relay `config.toml` | `messages_per_sec = 5`, averaged over one minute (example) |
| strfry `strfry.conf` | no write rate limit; `maxEventSize = 65536`; ephemeral events rejected after 60 s, deleted after 300 s |
| NIP-11 example document | `max_message_length = 16384`, `max_content_length = 8196`, `min_pow_difficulty = 30` |

**Per-seat write rate** (events per minute) against those limits; lobby burst
of 4 events (join, seed commit, seed reveal, genesis signature):

| wall time per round | frames unbatched (3/round) | batched (2/round) |
|---|---|---|
| 60 s | 3.00 — khatru defaults reject after ~3 min | 2.00 — after ~4 min |
| 180 s | 1.00 — khatru defaults reject after ~18 min | 0.67 — at the limit, no margin |
| 270 s | 0.67 — at the limit | 0.44 — inside |

Every other source above admits every row; noteguard's example by 2.7x–18x.
Seats behind one IP at 8 per minute: 2 / 4 (60 s), 8 / 12 (180 s), 12 / 18
(270 s), unbatched / batched.

**Read fan-out per relay**, 18 seats, 54 events per round, at 180 s rounds:

| readers beyond the seats | per round | sustained | per match |
|---|---|---|---|
| 0 | 0.61 MB | 0.03 Mbit/s | 0.01 GB |
| 1,000 | 34.6 MB | 1.5 Mbit/s | 0.35 GB |
| 50,000 | 1.70 GB | 76 Mbit/s | 17 GB |

A join or `QUEUE` request on the room topic is delivered to every subscriber:
50,000 requests to 50,000 subscribers at 629 bytes is ~1.6 TB (estimate). Writes
per relay for a whole 18-seat match: 612 events.

Sources: [strfry.conf](https://github.com/hoytech/strfry/blob/master/strfry.conf),
[nostr-rs-relay config.toml](https://github.com/scsibug/nostr-rs-relay/blob/master/config.toml),
[khatru sane_defaults.go](https://github.com/fiatjaf/khatru/blob/master/policies/sane_defaults.go),
[noteguard](https://github.com/damus-io/noteguard),
[nostr-relay-khatru](https://github.com/rzazo24/nostr-relay-khatru),
[NIP-01](https://github.com/nostr-protocol/nips/blob/master/01.md),
[NIP-11](https://github.com/nostr-protocol/nips/blob/master/11.md).

---

## D.61 The relay field test: protocol (T-169, R-SES16, R-SES17, R-SES18)

Supports `Hyades_sessions_discovery_and_security.md` §11. **A protocol; no
results yet.** The question: on the pinned public relays, with real players on
real networks, how often is a write rate-limited or refused, does any relay
send nips#2498's backoff hint, how often is a seat timed out, and does a round
ever resolve differently on two clients (R-SES18)?

**Bed.** The deployed site (GitHub Pages, from `main`), the five pinned relays
(`web/newgame.js`), three to eighteen players, each in their own browser on
their own network. Placeholders as shipped: 180 s per round, 10 rounds,
patience 120 s. A shorter arm (30 s per round, patience 60 s) loads the relays
harder per minute and finishes faster; run it as a second match.

**Steps.**

1. The host opens New game, types a name, sets seats to the number of players,
   and presses **Make a link**, then sends the link (Discord or any chat).
2. Each player opens the link, types a name, and presses **Take a seat**.
3. When the seats are full, the host presses **Start the match**; every player
   presses **Accept the parameters and sign**.
4. Leave **Play automatically** on (or pick orders by hand) and keep the tab
   open and in front until the round table shows every round. A background tab
   may have its timers slowed by the browser, which is itself worth recording.
5. *Optional, in one match only:* after round 2 the host presses **Kick** beside
   one player's name (ruling 12). That player's seat passes from the next round
   the host has not ordered in, and the round table on every tab should show
   it as removed from the same round. The kicked player keeps the tab open to
   the end; it still verifies and signs its checkpoints.
6. Every player presses **Save diagnostics (.tsv)** and **Save match record
   (.json)** and sends both files back, saying which browser and network
   (home, mobile, VPN) they used.

**What the files hold.** The diagnostics TSV has one row per relay reply and
protocol event: `connect`, `open`, `closed`, `screen-pass`/`screen-exclude`/
`screen-unavailable`, `accepted` (with latency), `rate-limited` (with the
relay's message, its hint as sent or `none`, and the wait chosen), `rejected`,
`write-timeout`, `sub-closed`, `notice`, `carry`, `timeout-vote`,
`round-resolved` (with its duration and who was timed out), `late-change`,
`equivocation`. The match record holds the genesis, every acceptance and every
frame the player held.

**What settles what.**

| quantity, from the TSVs | settles |
|---|---|
| `rate-limited` rows per relay per player-minute, with and without a hint | R-SES16's write allowance and pinned list; whether hints are sent at all |
| `rejected` rows by message prefix (`blocked`, `restricted`, `pow`, …) | which pinned relays refuse kind 7860 or this client |
| `screen-*` rows | which relays publish NIP-11 with CORS, and what they demand |
| `round-resolved` durations, and seats timed out per round | whether 180 s and 120 s are enough (R-SES17's magnitudes) |
| `carry` rows | how often the carrier is needed |
| `late-change` rows; checkpoint agreement in the round table | R-SES18 |
| in the kick match: `kicked_from` equal on every tab, and any `late-change` in the round it names | whether a kick arriving after a round resolved happens in practice (ruling 12, §5.1) |

**Data.** Files go in `docs/experiments/data/D.61-<date>-<player>.tsv` and
`.json`, per `docs/experiments/AGENTS.md`, with the browser and network noted
in this entry when results are added.
