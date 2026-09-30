# Hyades — The Exchange: producer↔consumer matching (Rev 2)

**Rev 2 (T-134).** §8 added: the **cross-empire** Exchange clears at a spatial
price equilibrium, no deeper than the buyer can move on, and every holding can
sell. Those items are `RATIFIED` (R-MX7, the author's approval of the proposal
and of the census-derived capacity; the one-quantity ruling on holdings). §0–§7
below are the Rev 1 draft of the **intra-empire** matcher and keep their status;
where they say optimality is not a goal, that is about the intra-empire wave and
not about §8.

**Status:** draft, proposed by Claude for Jonathan's ratification, with a
**tested reference implementation** (`matching.rs`, 6/6 unit tests passing,
compiled under rustc 1.75, dependency-free, `Entity = u64`). Graduates the
matching half of `hyades_todo.md` **T-29** (formerly §1) into a spec; the *policy* half of the
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

- **Speed today:** ~~with the 600-planet cap and the 847× throughput margin
  from `bench_hex_size.rs`, the existing O(P) scans are not the bottleneck
  yet. Each freighter load is one ~600-element scan; each production cycle
  likewise. At current entity counts this is microseconds.~~
  **Superseded — both premises have expired.** (a) There is no 600-planet
  cap any more: `GalaxyConfig::new(3, seed)` generates **6,725** planets, so
  every O(P) scan is ~11× the length this paragraph assumed. (b) The 847×
  margin cites `examples/bench_hex_size.rs`, which **is not present in this
  tree** (it is also cited by `galaxy.rs`, `tests/smoke.rs` and
  `tests/determinism.rs`); until it is restored the margin is unverified.
  (c) "At current entity counts" was written when colonization stalled at a
  few dozen colonies — the expansion fixes take a 3-seat game to thousands
  of vehicles, and the scans multiply per agent exactly as "Speed tomorrow"
  below predicts. Treat that paragraph as the live one and re-measure before
  concluding the scans are cheap. See AGENTS.md §7 for the throughput watch.
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
- **Design need:** todo **T-29**'s bidding system ("a pirate co-located can
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
**R-MX1:** whether the bidding system (todo **T-29**) later *surfaces* pressure
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

**R-MX1** does pressure ever surface diegetically (feeds todo **T-29**) ·
**R-MX2** market-tick cadence + per-fill light-lag ·
**R-MX3** distance-discounted price (λ doctrine knob) vs. pure
price-then-distance · **R-MX4** mining bid concurrency qty ·
**R-MX5** one Book per (empire, commodity) now; merge into per-commodity
*global* books when cross-empire bidding lands (the pirate's tribute demand
is then just a hostile bid whose distance term enforces todo **T-29**'s
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

---

## 8. The cross-empire clearing (T-134)

| symbol | name | unit | where it is set |
|---|---|---|---|
| `B` | a buyer empire | — | — |
| `j` | an ask: one empire's offer of one color from one planet | — | `post_exchange_offers` |
| `P_B` | buyer `B`'s price for the color this round | `$`/kt | the clearing's dual |
| `a_j` | ask `j`'s reservation | `$`/kt | `willingness_to_pay` at a yard; `0` away from one |
| `b_i` | bid `i`'s value | `$`/kt | `willingness_to_pay`, politics §2.11 |
| `λ` | transit burn rate | 1/yr | `trade_decay_lambda = 0.01` (R-P2) |
| `t_jB` | the seller's laden leg to the shared rock nearest it | yr | `ship_travel_years` on the seller's Freighter Design |
| `H_(B,v)` | what `B`'s haulers based at rock `v` carry away in one round | kt | `haul_per_round`: hold × `years_per_round` / laden round trip |
| `R_(B,v,c)` | `B`'s delivery room at `v` for color `c` | kt | `H_(B,v)` − what `B` already holds there in `c`, floored at 0; unlimited where `B` owns `v` |

**8.1 `RATIFIED` — R-MX7: the book clears at a spatial price equilibrium.** One
price `P_B` per buyer empire; ask `j` ships to the buyer maximizing
`P_B · exp(−λ t_jB)` and only if that is at least `a_j`; a bid buys only if
`b_i ≥ P_B`. Samuelson (1952), Takayama & Judge (1971). In `ln P` the burn is an
additive cost, so the allocation is the optimum of a transportation LP and the
prices are its duals; `matching::clear_spatial` solves it exactly by successive
shortest paths over a graph whose interior nodes are the buyer empires, and
quotes the **least** equilibrium prices (Demange, Gale & Sotomayor 1986). A
reservation of zero is floored at `1e-9` of the book's top offer — a numerical
device for the logarithm, not a tunable. Verified against scipy's LP on 9 books
and by every equilibrium condition on 40 random books — appendix §D.20.

**8.2 `RATIFIED` — a leg carries no more than the buyer can move on.** Each
route's capacity is `Σ_v R_(B,v,c)` over the rocks the seller and buyer both
work; the solver honors arc capacities exactly, and the flows are placed at those
rocks nearest the seller first, each taking at most its remaining room. What
finds no room is not sold. **Why:** ore delivered beyond `H` stays at the rock —
the clearing without this term moved the same tonnage as the greedy wave and lost
18.9% of work-years, and adding it is worth +47.3% ± 3.0% on 8/8 seeds. Appendix
§D.20. No new constant: the hold, the trip and the round interval are the
engine's.

**8.3 `RATIFIED` — a bank, the ore on a rock and a hauler's arrived cargo are one
quantity** (the author's ruling). The engine holds an empire's minerals per
`(empire, planet)` (`Holdings`); a center's "bank" is only the holding at a planet
the empire owns. Two indexes sit behind the one interface for speed, and claiming
a planet moves the claimant's holding there from one to the other.

**8.4 `RATIFIED` — every holding can sell.** A holding where its owner has no
yard asks, at `a_j = 0`, for the part its owner's haulers cannot move this round —
`max(0, held − H_(owner, v))`, split across colors in proportion to what it holds.
A holding at a rock the buyer also works hands over with a zero-length leg.
Worth +3.98% ± 2.09% over §8.2 alone (6/8 seeds) — **not resolved at two standard
errors**; ratified as the ruling's consequence, not on that number.

**8.4b `RATIFIED` (as built, T-134) — an empire does not trade with itself.**
A route from an ask to its own empire is dropped. The reason is measured, not
assumed: every bid fills from rival sellers while supply exceeds demand, and
once fills fall short of bids they stop at the buyer's own haulers' spare room,
which a self-route draws on too. Allowed, self-trade changed who filled a bid
and not how much was filled. Appendix §D.21. Privateering (`Hyades_warfare_tree.md`
§8.21) is the first mechanic that creates self-trade demand the book cannot
already meet, and would reopen this.

**8.5 `RATIFIED` — R-MX8: a hauler carries a center's abundance to a center
with demand, under the Exchange's journey discount** (the author's ruling). At a
planet its empire owns, a hauler serving center `D` takes, per color `c`,

```text
offer_c = min(abundance_c, want_c)   if  wtp(D, c) · exp(−λ t) > wtp(O, c),  else 0
abundance_c = max(0, held_c(O) − bill_c(O))
```

| symbol | name | unit | where it is set |
|---|---|---|---|
| `O` | the center the hauler stands on | — | — |
| `D` | the center the hauler serves | — | the shuttle's destination |
| `held_c(O)` | `O`'s holding in color `c` | kt | `Holdings` (§8.3) |
| `bill_c(O)` | `O`'s next works bill in color `c` | kt | `works_bill(infra_step_price)` |
| `want_c` | `D`'s shortfall against its next bill, net of the hold | kt | `wanted_here` (T-91) |
| `wtp(X, c)` | `X`'s willingness to pay for `c` | `$`/kt | `willingness_to_pay` |
| `t` | the laden leg `O → D` at a full hold | yr | `ship_travel_years` on the hauler's Design |

So a center keeps what its own next rung needs, and ships only where the Exchange
would ship an ask (§8.1's rule with `a_j = wtp(O, c)`). What loads at a center
goes to `D`, the buyer it was priced against, and is never re-routed. A milk run
(T-91) may stop at such a center too, scored the same way as any other pile. A
hauler whose own rock is settled keeps working while `O` offers something, and
stands down as before once it offers nothing. No new constant. Measured on the
8-seed bed: colony-years **+1.29% ± 0.22, 8/8 seeds**; work-years +6.22% ±
3.62, not resolved at 2 SE; appendix §D.22.

**8.6 Throughput.** §8 costs nothing measurable on the 3-seat bed — per-event
cost fell against the pre-T-134 engine in 6/6 interleaved pairs — and the
12-seat combat bench does 9% more events at lower per-event cost. Appendix §D.20. R-MX8
(§8.5) prices an offer at every owned center on every milk-run stop; with the
buyer's side read once and a per-seat index of owned planets, per-event cost on
the 3-seat bed is +2.2% ± 1.1 at 4,000 yr (8 seeds) and at or below the engine's
before it at 1,000 yr. Appendix §D.22.

---

## 9. The internal duty exchange (T-134 stage 2)

§8 prices ore between empires. This section prices **hull time inside one
empire**: a hull standing on one duty weighs a side duty against it, in the same
units, with Doctrine setting the exchange rate. The engine carries the first
three side duties the author specified; the rest of the author's list is open
below, and every magnitude here is a placeholder.

| symbol | name | unit | where it is set |
|---|---|---|---|
| `m` | a miner's mining rate: its crew's lift this tick over crew size and `mining_tick_years` | kt/yr | `sys_mining_tick` |
| `f` | a run's freight rate: what the best center wants of the pile, up to the hold, over the laden-plus-empty round trip | kt/yr | `consider_freight_run` |
| `p_mine`, `p_freight` | the prices of the two duties | — | `Doctrine::duty_price` (`DutyPrices`), both `1.0`, placeholders |

**9.1 `RATIFIED` (the author's direction) — a miner on a large holding runs
freight when there is demand.** After each mining tick, the last miner of a crew
of two or more leaves for one run when `f · p_freight > m · p_mine`
(`Standing::takes_freight_run`). `m` is **zero** when its empire's pile at the
rock already exceeds `H` (§8's per-round haul), because ore lifted there goes
nowhere this round. The run carries what the best center (the score every hauler
uses) is short of, delivers it into that center's holding, and flies back to the
rock, where the miner rejoins its crew. Nothing runs before the first barrier,
when `H` is first measured; `H` is measured at every barrier whether or not the
book posts.

**9.2 `RATIFIED` (the author's direction) — a colony ship whose origin is still
growing runs freight before it embarks.** When the origin's settler split
(`settler_target`, R-IND12), not the hold or the world, limits the settlers a
colony ship would carry now, the hull flies empty to the nearest pile its empire
holds of anything its origin is short of, brings one load home, and then embarks
with what the grown origin sends (`Standing::runs_freight_before_embarking`,
off when `p_freight = 0`). Its launch — the `VehicleSpawned` record, the light a
blockader sees, the offer to rival pickets — is the embarkation, not the build.
About 22% of colony ships launch with fewer settlers than their hold carries
(seeds 1 and 7, 1,000 yr).

**9.3 `RATIFIED` (the author's direction and ruling) — a posted picket goes to
a pitched battle it believes it can reach before the battle is decided.**
Specified in `Hyades_warfare_tree.md` §8.20: both hulls armed and one standing;
the battle's light plus the picket's flight must come before the fight's
believed end; a return to the post when nothing is left in its reach. No
distance constant.

**9.4 What the three duties measured.** Card-free, 8 seeds, 4,000 yr:
work-years **+0.80% ± 1.89** (3/8 seeds up), colony-years **+0.03% ± 0.02**,
colonies identical on every seed — not resolved at two standard errors, and
ratified on the author's direction, not on the number. Over 1,000 yr on seeds
1 and 7, miners make 2,723 and 1,460 runs and colony ships 1,511 and 1,341.
Appendix §D.21.

**9.5 `OPEN` — R-MX9: a Mahan main fleet's quick intercept.** The author's
rule: a main fleet runs a quick intercept **only if** doing so does not change
its empire's belief about how pitched battles go or what the current Doctrine
achieves. The engine has no main-fleet role, no stored belief about battle
outcomes and no record of Doctrine outcomes, so none of the three terms of the
rule can be evaluated yet. **What would settle it:** a main-fleet role (roles §5's
stored fleet, `Hyades_warfare_tree.md` R-WAR32), a per-empire estimate of
pitched-battle success that a detachment would move, then the rule as a
`Standing` question beside `joins_battle`.

**9.6 `OPEN` — R-MX10: the room this leaves for cards.** The author: most of
this Exchange cannot be implemented yet, and the design should leave room for
new cards, Designs and Doctrine to widen what it means. The places a card can
write today, each read through one `Standing` question:

- **Duty prices** (`DutyPrices`) — a card that values freight over mining, or
  sets `p_freight = 0` to keep every hull on its standing duty.
- **How a picket judges a battle** — `Standing::joins_battle` takes the
  arrival and the believed end; a card that changes what a seat believes about
  fights, or how much margin it wants, writes there.
- **A Design's hold** — every side run carries `cargo_capacity`, so a Design
  write that enlarges a hold enlarges every side run it makes.
- **A new duty** — a `SideRun` variant and a `Standing` question are the whole
  interface; the event loop, the conservation ledger and the stand-down on
  withdrawal already cover it.

**What would settle it:** the cards themselves; until then these are the
interfaces, not the designs.

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
- Samuelson, P.A., "Spatial Price Equilibrium and Linear Programming,"
  *American Economic Review* 42(3) (1952) 283–303
- Takayama, T. & Judge, G.G., *Spatial and Temporal Price and Allocation
  Models*, North-Holland (1971)
- Ahuja, R.K., Magnanti, T.L. & Orlin, J.B., *Network Flows*, Prentice Hall
  (1993), §9.7 — successive shortest paths
- Demange, G., Gale, D. & Sotomayor, M., "Multi-Item Auctions," *Journal of
  Political Economy* 94(4) (1986) 863–872 — least equilibrium prices
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
