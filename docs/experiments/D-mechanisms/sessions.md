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
