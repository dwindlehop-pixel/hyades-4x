# Hyades — The Exchange: producer↔consumer matching (draft Rev 1)

**Status:** draft, proposed by Claude for Jonathan's ratification, with a
**tested reference implementation** (`matching.rs`, 6/6 unit tests passing,
compiled under rustc 1.75, dependency-free, `Entity = u64`). Graduates the
matching half of `hyades_todo.md` §1 into a spec; the *policy* half of the
bidding system (when a cross-empire offer succeeds) stays in the todo, per
its "not to be spec'd until articulable" rule — this doc only builds the
machinery that system will run on.

**The brief:** per-cycle production-choice scanning and need-based hauling
(`most_needed_center`) are potentially performance-limiting at scale; design
efficient matching System(s) with minimum query count; an abstract value
scalar ("money") is on the table; check SimCity-adjacent prior art; multiple
Systems allowed since some problems match all producers, some all consumers,
some queue; **change nothing if the existing approach is best.**

---

## 0. Verdict first — is a change warranted?

**Partially yes, and mostly not for raw speed.** Honest accounting:

- **Speed today:** with the 600-planet cap and the 847× throughput margin
  from `bench_hex_size.rs`, the existing O(P) scans are not the bottleneck
  yet. Each freighter load is one ~600-element scan; each production cycle
  likewise. At current entity counts this is microseconds.
- **Speed tomorrow:** the costs are *multiplicative* where it hurts — the
  Monte-Carlo balancer runs thousands of sims × parameter sweeps, and the
  scans multiply per-agent (F freighters × P planets per hauling wave,
  C centers × P per production wave). The reference implementation's full
  worst-case matching wave (600 bids × 300 asks, fully cleared) measures
  **0.68 ms** — and, crucially, it replaces *all* per-agent scans in that
  wave with one pass.
- **Correctness now:** per-agent argmax has a real behavioral bug the
  matcher fixes for free — **herding**. Every freighter loading in the same
  window computes the same `most_needed_center` and dogpiles it, because
  nothing reserves the need a prior match already covered. This is the
  exact documented failure mode of Cities: Skylines' vanilla dispatcher
  (trucks crossing the map past nearer work) and the reason its
  community rebuilt the matcher twice (see §7 refs). Reservation-on-match
  fixes it structurally.
- **Design need:** todo §1's bidding system ("a pirate co-located can
  demand tribute; the other civilization offered my entity a better deal")
  *requires* a common place where unlike offers are compared by one scalar.
  Building the Exchange now means the bidding system later is "post a bid
  into the same book," not a new subsystem.

**Decision D0: adopt the Exchange as an additive module + call-site swap,
not a rewrite.** `most_needed_center` is retained as the test oracle: with
exactly one unit of supply in the book, a matching wave provably reduces to
it (unit test `degenerate_case_equals_most_needed_center`).

---

## 1. Prior art — yes, SimCity's lineage solved this

- **Cities: Skylines `TransferManager`** is the closest existing solution
  and the model adopted here. Every interaction in that game — goods,
  garbage, fire, even romance — is an **incoming offer** (need) or an
  **outgoing offer** (supply) with a *reason*, a *priority 0–7*, an
  *amount*, and a *position*, posted into one central book; a matcher pairs
  them **by priority block first, then proximity**, one reason per
  simulation step. Its two documented weaknesses are instructive: vanilla
  matching largely ignored distance within a block (fixed by the
  MoreEffectiveTransfer / Transfer Manager CE mods, which match
  nearest-within-priority — exactly our policy), and single-unit offers
  caused re-scan churn (fixed by offering remaining capacity — our `qty`
  field). See jkm.dev's decompilation write-up, §7.
- **SimCity 2013 / GlassBox** (Willmott, GDC 2012) is the agent-based
  counterpoint: resources in bins, dumb agents carrying them, 10,000+
  agents kept simple, rules on units not agents. Its lesson is mostly
  negative for us: pure per-agent greedy routing with no central matching
  produced the game's notorious pathologies (agents taking the first
  available job/house). GlassBox validates "keep the mobile agent dumb";
  the TransferManager validates "make the matching central."
- **Algorithm literature.** The problem is the **assignment problem**.
  Optimal matching is the Hungarian method, O(n³) (Kuhn 1955) — rejected:
  too slow per wave and optimality is not a design goal (autopilot defaults
  are meant to be good, legible, and overridable, not optimal).
  **Bertsekas's auction algorithm** (1988) is the concurrent-actors result
  the brief anticipated: agents bid for objects, *prices* rise on
  contested objects until ε-complementary slackness — it is the theoretical
  justification for using one scalar price as the matching axis, and the
  upgrade path if greedy ever proves insufficient. **Contract Net
  Protocol** (Smith 1980) is the same announce–bid–award shape in
  distributed AI. **Weapon–target assignment** is NP-complete in general
  (Lloyd & Witsenhausen 1986) — which is the license to be greedy in
  combat, §5. Gale–Shapley stable matching is noted and rejected: stability
  against preference-list deviation is a two-sided-preferences property we
  don't need (supply has no preferences beyond distance).

---

## 2. The model — Bids, Asks, pressure, one Book per (empire, commodity)

**D1 — Two-sided offers.** Consumers post **Bids** (need); producers post
**Asks** (supply). `Offer = { entity, price, qty, pos }`. One live bid and
one live ask per entity per book; re-posting replaces (the dirty-flag
update path).

**D2 — Yes to the abstract value scalar; its name is `pressure`, not
money.** One f64 lets unlike offers be compared — a center's mineral
hunger, a planet's exploitation rank, later a rival's tribute bid. For
Minerals it *is* the existing `mineral_pressure_of` ∈ [0,1], unchanged; for
BuildTarget it is the planet rank `R(planet)`
(`Hyades_autopilot_colonization_growth.md` §3). It is **not player-facing
currency** — no diegetic money enters the warm-hard-SF register — it is
the matching scalar, exactly the auction algorithm's price variable.
**R-MX1:** whether the bidding system (todo §1) later *surfaces* pressure
diegetically as trade value, or keeps it internal.

**D3 — Books are a Resource; posting and matching are Systems.** This does
not contradict "autopilot is not a Resource": the Book is inert
infrastructure like the event queue and the RNG — a genuine singleton data
structure with no behavior. The behavior (what to post, when to match, what
a fill *does*) lives in Systems reading `Role` + doctrine, per
`Hyades_vehicle_roles.md` §7/§9. The Exchange never mutates world state; a
`Fill` is turned into scheduled events by the calling System.

**D4 — Event-driven posting; scheduled matching (the minimum-query-count
property).** No system scans per cycle. Offers are posted/updated only when
underlying state changes: stockpile delta or infra level-up re-posts a
Minerals bid; a scan result or card re-rank posts/re-prices a BuildTarget
bid; a freighter finishing a leg posts an Ask. Matching runs as **one
scheduled discrete event per (empire, commodity)** — a "market tick" on the
existing event queue, same discipline as everything else — collapsing F
per-freighter scans and C per-center scans into a single O((B+A)·min(B,A))
wave (0.68 ms at the worst-case cap, §0). Each state change touches the
book once; nothing polls. **R-MX2:** market-tick cadence (per production
cycle? on-demand when a book goes nonempty on both sides?), and whether it
is itself light-lagged per participant (leaning yes: a fill's realization
event is scheduled at the light distance between bid and ask, consistent
with contract §2).

**D5 — Matching policy: price-block, nearest-within, reserve, queue.**
Bids in descending pressure (ties → lower entity id); each bid consumes the
nearest remaining ask (distance ties → lower id); partial fills; **matched
qty is reserved on both sides** (the anti-herding fix); unmatched remainder
**stays queued** in the book for the next wave — satisfying the brief's
"some producers and some consumers unmatched." Greedy, not optimal, by
design (§1). Fully deterministic: total order on every comparison, no
HashMap, entity-id tiebreaks — identical books yield identical fills
(unit-tested).

---

## 3. Instantiation 1 — Minerals (replaces per-freighter `most_needed_center`)

- **Bids:** owned production centers; price = `mineral_pressure_of`
  (unchanged formula), re-posted on stockpile/infra change events.
- **Asks:** freighters at load-complete; qty = cargo on board.
- **Fill →** schedule the freighter's delivery leg to the bid entity
  (replacing the argmax call in `sys_freighter_arrive`); `Shuttle.outpost`
  pairing stays fixed exactly as today — only the *destination* side goes
  through the book.
- Net behavior change: freighters spread across the top-k needy centers
  instead of all converging on the top-1. **R-MX3:** whether distance
  should also *discount* price (a needy center 400 ly away vs. a
  slightly-less-needy one 20 ly away) — leaning yes eventually, via
  `effective_price = price − λ·distance` with λ a doctrine knob, but
  shipping Rev 1 with pure price-then-distance to preserve oracle
  equivalence with `most_needed_center`.

## 4. Instantiation 2 — BuildTarget (replaces per-center per-cycle target scans)

- **Bids:** close-scanned, unexploited planets; price = rank; posted by the
  scan-arrival event and **re-priced by card re-rank events** — this is
  where "cards change rank retroactively" (autopilot doc §3) becomes an
  O(changed planets) book update instead of an O(P) rescan by every center.
- **Asks:** production centers with free build output at step 2 of their
  cycle; qty = 1 build slot.
- **Fill →** the center builds what the target class demands (colony
  vehicle, or miner+freighter pair). Colonization fills are **exclusive**
  (bid qty 1, consumed on match — no two centers target the same colony
  world, resolving the duplicate-targeting the current independent argmax
  permits); mining bids are **non-exclusive** per `Hyades_vehicle_roles.md`
  (post with qty = number of concurrent exploiters allowed; **R-MX4:** that
  number). This also gives R-AC12 (multi-target output split) its natural
  answer: the split is whatever the wave assigns.

## 5. Instantiation 3 — combat target acquisition: buckets, no book, no money

**D6 — Combat does not use the Exchange.** Engagement is already scheduled
by spatial proximity events and resolved per theater
(`Hyades_loadout.md` §5), and Fleet is a co-located query — so the theater
*is* the bucket. Target acquisition is: group combatants by theater (one
O(N) pass over position, the same pass `fleets_at` implies), then match
within the theater greedily by threat score in entity-index order. Global
optimal weapon–target assignment is NP-complete (§1) and a global book adds
nothing when every legal pairing is local by definition. The only shared
machinery is the discipline: deterministic order, id tiebreaks.

---

## 6. Complexity summary

| Path | Today | With the Exchange |
|---|---|---|
| Freighter destination | O(P) scan **per freighter load** | O(live offers) upsert per state change + one shared wave |
| Center target choice | O(P) scan **per center per cycle** | same wave; card re-rank = O(changed) re-posts |
| Card re-rank fallout | every center rescans | O(changed planets) book updates |
| Combat acquisition | per-ship nearest scans | O(N) theater bucketing + tiny local matches |
| One full worst-case wave | — | **0.68 ms** measured (600 bids × 300 asks, rustc 1.75 -O) |

## 7. Ratification points

**R-MX1** does pressure ever surface diegetically (feeds todo §1) ·
**R-MX2** market-tick cadence + per-fill light-lag ·
**R-MX3** distance-discounted price (λ doctrine knob) vs. pure
price-then-distance · **R-MX4** mining bid concurrency qty ·
**R-MX5** one Book per (empire, commodity) now; merge into per-commodity
*global* books when cross-empire bidding lands (the pirate's tribute demand
is then just a hostile bid whose distance term enforces todo §1's
co-location rule) · **R-MX6** confirm `most_needed_center` is kept
permanently as the oracle in tests, or deleted after the swap bakes.

**Sequencing:** the module is additive (one file, no engine changes needed
to compile it). The call-site swap touches `sys_freighter_arrive` and the
production-cycle step-2 chooser, and is cleanest **after** the
Role/autopilot refactor already queued (`Hyades_vehicle_roles.md` §7/§9),
since both rewire the same dispatch sites. The engine repo was not present
in this session's container; `matching.rs` ships alongside this doc,
tested standalone, ready to drop into `src/` when the repo is next
uploaded.

---

## References

- Cities: Skylines `TransferManager` internals (offers, priority blocks,
  distance, per-reason cadence — decompiled walkthrough):
  https://jkm.dev/posts/cities-skylines-trading-market/
- MoreEffectiveTransfer (nearest-within-priority matching rework, modes,
  and the documented vanilla pathologies):
  https://github.com/pcfantasy/MoreEffectiveTransfer/wiki/English-UG
- Transfer Manager CE (balanced match mode; capacity-quantity offers fixing
  single-unit churn):
  https://steamcommunity.com/sharedfiles/filedetails/?id=2804719780
- Willmott, A., "Inside GlassBox," GDC 2012 (resources/units/maps/agents;
  10,000+ dumb agents): https://www.andrewwillmott.com/talks/inside-glassbox
  — coverage: https://www.gamedeveloper.com/design/gdc-2012-breaking-down-em-simcity-em-s-glassbox-engine
- Bertsekas, D.P., "The Auction Algorithm: A Distributed Relaxation Method
  for the Assignment Problem," *Annals of Operations Research* 14 (1988)
  105–123; survey: https://www.mit.edu/~dimitrib/Auction_Encycl.pdf
- Kuhn, H.W., "The Hungarian Method for the Assignment Problem," *Naval
  Research Logistics Quarterly* 2 (1955) 83–97:
  https://en.wikipedia.org/wiki/Hungarian_algorithm
- Smith, R.G., "The Contract Net Protocol," *IEEE Transactions on
  Computers* C-29(12) (1980) 1104–1113:
  https://en.wikipedia.org/wiki/Contract_Net_Protocol
- Lloyd, S.P. & Witsenhausen, H.S. (1986), WTA NP-completeness:
  https://en.wikipedia.org/wiki/Weapon_target_assignment_problem
- Gale, D. & Shapley, L.S., "College Admissions and the Stability of
  Marriage," *American Mathematical Monthly* 69 (1962) 9–15:
  https://en.wikipedia.org/wiki/Gale%E2%80%93Shapley_algorithm
- Order matching / price-time priority (the exchange metaphor):
  https://en.wikipedia.org/wiki/Order_matching_system
- Nystrom, R., *Game Programming Patterns* — Event Queue & Dirty Flag
  (the posting discipline): https://gameprogrammingpatterns.com/event-queue.html ·
  https://gameprogrammingpatterns.com/dirty-flag.html
