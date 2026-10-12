# §D. Mechanisms — supers, apex and forging

*Part of the experiments record. Nothing here is normative. Table of contents: [`../README.md`](../README.md); how the record is organized: [`../AGENTS.md`](../AGENTS.md).*

---

## D.23 T-138 — supers, apex, the refined Exchange, priced production, and a reachable Band IV

*Supports galaxy §3.0 and §4.5, technology §3.2 and §3.7, trees §4.6, matching
§8.7 and §10.4. Card-free bed: `examples/work_years`, 3 seats, 4,000 yr, the 8
seeds of §D.20, against the R-MX8 build (`e7e883b`), all four binaries at once.*

**Population Band IV was unreachable.** The top population edge
(`PopBands::from_weibull(1.4, 4.0)`) is the `Band IV` mass, 715,541.75 kt, and
so was every homeworld's `K` (habitability and pristine biosphere both `Band
4.0`); no world had a ceiling above it (0 of the 200-planet test galaxy, 19 at
it). The closed-form logistic approaches its ceiling without reaching it, so no
world could hold population `Band IV` and the synthesis gate could never open;
a homeworld stood at 61% of it at 600 yr. The same comparison feeds
`industrial_signature`, which could never be true either.

**Calibrating the ceiling to the author's schedule** (a scratch harness, never
landed: seat 0's homeworld population against the `Band IV` edge every 10 yr;
seeds 1, 7, 42, 31337; 1,600 yr; rounds at 200 / 600 / 1,000 / 1,400 yr):

| ceiling (Band) | card-free, every seat | Growth card ×1.6 at round 1, seat 0 | Expansion card ×8 at round 1, seat 0 |
|---|---|---|---|
| 4.05 | 693–745 yr | 503–522 yr | identical to card-free |
| 4.1 | 653–705 yr | 483–502 yr | identical to card-free |
| 4.2 | 632–685 yr | 473–482 yr | identical to card-free |
| 4.3 | 623–675 yr | 463–482 yr | identical to card-free |

At 4.2 a growth-dedicated build crosses before round two's card selection and
every card-free seat crosses between rounds two and three, as the ruling asks.
The ceiling moves the crossing by about 70 yr over 4.05–4.3. The Expansion card
draws no population, so no build in the engine drains a homeworld faster than
the default and the ruling's round-four case cannot be measured yet.

**Card-free, the whole layer is inert** — the 4.2 ceiling and every refined
mechanism together: work-years, colony-years, colonies, vehicles and events
**bit-identical on 8/8 seeds**; only the population figure moves (seed 1 at
1,000 yr: 64,267,894 → 68,593,963 kt). An inference, at about 90% confidence:
nothing a card-free run decides reads a homeworld's population above the
`Band III` edge. A census of the decisions that read population would confirm it.

**Throughput.** The first build ran `ns/event` +5.49% ± 1.05 (8/8) on that bed
with identical events — the new code's own cost. With fast paths for a game
with no standing order and for a hauler with no refined want, interleaved at
1,000 yr (3 rounds, seeds 1 and 7, events identical): 15,017–15,287 against
15,110–15,477 and 14,053–14,861 against 14,285–14,788 `ns/event`; min-of-3
+0.6% and +1.7%, inside the run-to-run spread — not resolved. Test targets:
unit 5.6 s, determinism 51.2 s, smoke 35.2 s, telemetry 25.9 s (the
determinism target was 56–58 s at §D.22).

**The mechanism bed** (`a_super_billed_design_is_synthesized_traded_and_built_with_mass_conserved`,
3 seats, 200 planets, 1,500 yr, regrowth off, every seat's colonizer and miner
Designs billed 25% Red — the state a tier-3 Design write leaves): 3.04 kt of
Red synthesized, 393 hulls built carrying Red, mass conserved to 1e-9. Two
defects found on the way, both fixed before landing:

- **A blocked Design stopped the yard.** A seat whose colonizer owed Red it
  could not get chose that order at every decision, declined it, and built
  nothing else — its banks did not move for centuries. Fixed by quoting a
  Design the center cannot pay as unpayable and choosing again (§10.4).
- **Quoting hid the demand.** With the Design quoted unpayable up front, the
  center never ordered it, never declined, and recorded no need: 0.03 kt made,
  one Red hull, nothing for freight or the Exchange to answer. Fixed by
  choosing on plain prices first and recording the need when the choice is
  blocked; needs are re-recorded at each decision so a center that stops
  wanting the order stops bidding for it.

No Red was traded in that bed: every homeworld held some Magenta and Yellow,
including the two archetypes whose native super is not Red, so each forge made
its own (technology §3.2's "exactly one" is a gradient as built; R-G4). Trade is
pinned by `a_forge_sells_the_super_it_can_make_and_makes_it_at_settlement`.

---

## D.25 R-MX10 — synthesis and refined trade, confirmed in a run

*Supports matching §8.7, §9.6 and §10.6. Bed: the §D.23 mechanism bed (3 seats,
200 planets, 1,500 yr, regrowth off, test configuration), every seat's
colonizer (`Delta`) and miner (`Meadow`) Designs billed 25% refined — the state
a tier-3 Design write leaves — on seeds 11, 3, 7 and 42. A scratch census test,
never landed, read the synthesis log, the refined books and every contract.*

**Arms.** Red: 25% Red. R+G+B: 25% split equally over the three supers. Apex:
25% apex. Non-native: each seat billed in another archetype's native super.
Lockout: the R+G+B and apex arms on the same galaxy with seat 2's homeworld
ceiling at `Band 3.5`, so it can never reach population `Band IV` and forge —
a galaxy variant, which is what a bed may vary (T-133 ruling).

**Synthesis and building, 24 of 24 runs.** Every seat at `Band IV` makes what
its bills owe, and the yard builds hulls of it:

| arm | refined kt made per seat (seed 11) | hulls carrying refined (4 seeds) |
|---|---|---|
| Red | 1.97 / 0.66 / 0.62 Red | 291–428 |
| R+G+B | 0.28 / 0.37 / 0.24 of each | 335–464 each |
| apex | 1.58 / 0.77 / 0.41 apex, and the supers it draws | 319–418 |
| non-native | 0.83 / 0.64 / 0.63 of the billed super | 70–299 per super |
| lockout, R+G+B / apex | seat 2 makes nothing | 253–406 / 272–364 |

First synthesis at 605 or 655 yr on every seat that can forge. Mass drift at
most 9.8e-16 of the total.

**Trade, almost none.** Refined kilotonnes delivered between empires: 0.0050
(seed 11, Red), 0.0369 (seed 7, R+G+B), 0.0251 (seed 7, R+G+B, lockout), and 0
in the other 21 runs. Refined bids posted per run 22–255; asks 0–9. Seat 2 in
the lockout arms received refined material in 1 of 8 runs.

*Mechanism, instrumented at posting.* A forge offers capacity from basics
above its next works bill. At every barrier, on every seed, each forge held
less than that bill in at least one basic — a typical forge 0.3 / 27 / 14 kt of
Cyan / Magenta / Yellow against a bill of 260 / 130 / 390 kt — so its capacity
was zero. Apex is never offered: capacity asks cover the three supers only,
and a forge synthesizes only what it owes. Every homeworld holds all three
basics, so every forge makes every super from its own holding, and the only
window with one forge and no other (605–655 yr on seed 11) contains no round
barrier. Open as R-MX16.

**A defect found and fixed.** A capacity ask was sized per precursor against
that color's share of the works bill; settlement (`synthesis_plan`) keeps the
whole bill as a total. On seed 11 at 1,000 yr seat 1's forge held 8.163 /
7.584 / 0.028 kt against a bill of 6.333 / 3.167 / 9.500 kt and offered
2.43 kt of Blue; its total, 15.78 kt, was below the 19.0 kt bill before any
draw, and both Blue contracts struck from the ask defaulted at settlement. The
ask is now capped by the total above the whole bill, and each super's draw
comes off its precursors' room before the next super is counted (Red and Green
share Yellow). `a_forge_offers_only_capacity_it_can_settle` fails on the old
rule (2.11 kt of Red offered that settlement cannot make) and passes on the
new. Card-free runs are bit-identical to `e3e87d0` on seeds 1, 7, 42 and
31337 at 1,000 yr: a card-free game posts no refined bid, so no ask clears.

---

## D.30 Forging is a forge's purpose

*Supports galaxy §4.5 and matching §8.7, §10.4, §10.6, §10.7. The author's
ruling: forging is a high-priced activity, outweighing almost anything but
immediate survival; once a center clears population `Band IV` its primary
purpose is to forge supers and apex. Bed: `examples/forge_census` — card-free,
standard galaxy, 3 seats, 1,500 yr, seeds 1, 7, 42, 31337 — run on this engine
and on `a50ef75` (common random numbers, paired by seed and seat).*

**What it replaced.** Synthesis ran on demand only: when an order a forge was
paying owed a super it lacked, when a contract it had sold came due (capacity
asks, sold from basics above what its standing order left and priced at the
precursors' willingness to pay over the yield, R-MX16), or when a hauler loaded
at a forge for a sister center. A card-free game posts no refined bid, so a
card-free forge synthesized nothing: every seat on `a50ef75` reads 0.00 kt of
every refined material. It was wrong about the forge's purpose, which the
author has now ruled is to forge.

**Economy, paired** (new against `a50ef75`):

| quantity | empire total per seed (n = 4) | per seat (n = 12) |
|---|---|---|
| colony-years | +0.001% ± 0.002 | +0.000% ± 0.013, range −0.08 to +0.09 |
| work-years | +0.66% ± 0.63 | +0.85% ± 0.84, range −2.21 to +9.43 |
| basic kt delivered between empires | +0.45% ± 0.22 | — |

None of the three is resolved at two standard errors (estimates). *Inference:*
a homeworld that stops building at 610–845 yr costs nothing measurable because
colonies carry the expansion loop by then; the mechanism is not instrumented.

**Forge output.** First synthesis at 610–845 yr on every seat. Supers forged
per seat 57.2–6,009.9 kt (7,495.4 kt over 12 seat-runs), apex 3,090.4 kt.
Every forge makes all three supers. The seat's native super is 4.7–93.0% of its
super mass (median 22.7%, pooled 30.9%): a forge makes whatever pair its
freight and the Exchange bring, not its archetype's. **No super or apex is
delivered between empires on any run**: a forge bids for no super, and no
center posts a refined bid card-free (R-MX18).

**Super-billed Designs.** On §D.25's bed (Delta and Meadow billed 25% Red,
200 planets, seed 11, 1,500 yr) each seat holds 1–4 centers; the forges make
Red and no Red-billed hull is built, because the forge no longer builds and no
hauler carries its Red to a colony within the horizon (a scratch census, never
landed: Red held outside the forge 0.00 kt at every century). Before, the
forge built 291–428 such hulls itself (§D.25). Open as R-MX17;
`a_super_billed_design_is_forged_with_mass_conserved` now pins forging and
conservation only.

**Test targets after the change:** unit 5.2 s (302 tests), determinism 43.1 s,
smoke 12.4 s, telemetry 26.5 s (debug, one run each).

---

## D.31 Forges build super-billed Designs and bid for supers; decisions without a cadence

*Supports galaxy §4.5, matching §8.7 and §10.7, autopilot §6.1a. The author's
rulings: a Design paid in supers is priced higher than forging (R-MX17); forges
bid on the supers they have demand for (R-MX18); no decision has a cadence of
its own; each decision is a tree with a short circuit for the common case, and
some are conditioned on an event. Bed: `examples/forge_census` (card-free,
standard galaxy, 3 seats, 1,500 yr, seeds 1, 7, 42, 31337).*

**R-MX17.** On §D.25's bed (Delta and Meadow billed 25% Red) the forge's first
tick had turned its whole stock into apex (0.50 kt of each basic into 0.495 kt
apex at 600 yr) before its yard's next decision at 605 yr, and the
homeworld's income afterwards rounds to zero, so no Red-billed hull was built.
Two defects stood behind it, each found by tracing the decision:
`ProductionContext::price_of` keys on the hull, so a sentry read the picket's
price and a Red-billed miner read the scout's, both priced out at a forge; and
forging ran ahead of the yard. The yard's quote now reads the Design's own
price (`design_price`) and the forge runs after the yard. The bed builds
Red-billed hulls again (`a_super_billed_design_is_forged_and_built_with_mass_conserved`).

**The retry floor.** Moving the forge behind the yard tied forging to T-88's
`decision_retry_years = 50`, which the author ruled is not design. Deleted.
Asking every center with a free berth on every tick doubled the cost of seed 1
(12.3 s → 25.2 s, events +0.9%). Census of 773k decisions on seed 1: 726,129
exited before the scan (bank below every price, or empty pool), 19,228 scanned
and declined over 7.36 M candidates, 13,644 scanned and committed. Callgrind
(800 yr): `commit_one_build` 63% of instructions, `view_of` + `rank` 41.5%. Of
the scanned declines, 17,470 held the price of a colonizer and a mining pair:
the policy preferred deepening and the rung lacked a color, so it waited on a
color. Conditioning every decline on every scan woke 206,695 of 210,500 asks at
800 yr (scouts add a world on almost every tick; nearly every scanned world can
rank as a mining outpost at full pressure). Conditioning a saving decline on
money, level, works and cards only skipped 197,772 asks and let 9,042 through.

| engine, seed 1, 800 yr | time |
|---|---|
| 50-yr retry floor | 4.9 s |
| every tick | 7.5 s |
| conditioned on events | 5.0 s |

Over 4 seeds at 1,500 yr, conditioned against every tick: colony-years
−0.24% ± 0.04 (4/4 lower), work-years +6.07% ± 4.45 (not resolved). Against
`a50ef75`: colony-years +0.12% ± 0.14, work-years +0.92% ± 4.06 (neither
resolved).

**R-MX18.** Supers delivered between empires: 271.9 / 247.1 / 228.2 / 63.4 kt
on seeds 1 / 7 / 42 / 31337 — 810.6 of 22,040 kt forged (3.7%), against 0
before forges bid. A seat's native super is 29.6% of what it forges, pooled
(per seat 0.4–95.9%, median 17.3%). The recommended ask at the precursors' cost
was not adopted: it equals an ordinary center's bid for a super and never
clears once the bid is discounted by transit, which a unit test showed on the
first run.

**Test targets:** unit 4.0 s, determinism 35.9 s, smoke 10.6 s, telemetry
23.2 s (debug, one run each). `shrinking_the_economy_tick_does_not_multiply_decisions`
is retired: it pinned the retry floor.

---

## D.34 Demand for supers in the card-free bed

*Supports T-146 and R-M5. The author's question: is there sufficient demand
for supers in the test bed? Bed: `examples/super_census` (card-free, standard
galaxy, 3 seats, 1,500 yr, seeds 1, 7, 42, 31337) on three color fields;
every figure is summed over the four runs.*

**Where super demand can come from.** A refined bid is a center's declined
order's refined shortfall, or a forge's want for the supers that complete a
balanced set for apex (R-MX18). An order is billed in refined material only by
a Design write (`Roster::bill_for` returns basics otherwise), so in a
card-free game every super bid is a forge's, and its only use is apex. Apex
has no consumer in the engine: its book carried no bid at any barrier in any
run.

| color field (spacing / width, ly) | 10 / 5 | 70 / 38.5 | 140 / 56 |
|---|---|---|---|
| basics mined | 6,257,724 kt | 20,764,703 kt | 27,529,826 kt |
| basics drawn into supers | 93,503 kt (1.49%) | 44,805 kt (0.22%) | 28,950 kt (0.11%) |
| supers forged | 62,335 kt | 29,870 kt | 19,300 kt |
| supers drawn into apex | 48,061 kt (77.1%) | 12,806 kt (42.9%) | 4,981 kt (25.8%) |
| apex made, all held at the horizon | 24,031 kt | 6,403 kt | 2,491 kt |
| supers held at the horizon | 14,267 kt | 17,033 kt | 14,297 kt |
| supers delivered between empires | 651 kt (1.04%) | 1,527 kt (5.11%) | 178 kt (0.92%) |

**The books at the two barriers that follow the first forges** (barriers at
200, 600, 1,000 and 1,400 yr; the first forge stands at 605–765 yr), supers
summed over the three books:

| field | barrier | bid | asked | per-book min(bid, ask) | filled | bid in books with no ask | asked in books with no bid |
|---|---|---|---|---|---|---|---|
| 10 / 5 | 1,000 | 8,708 | 5,935 | 479 | 479 | 5,344 | 0 |
| 10 / 5 | 1,400 | 16,260 | 9,999 | 187 | 172 | 8,018 | 8,022 |
| 70 / 38.5 | 1,000 | 5,316 | 3,272 | 856 | 856 | 3,797 | 964 |
| 70 / 38.5 | 1,400 | 14,026 | 9,775 | 719 | 671 | 9,696 | 5,326 |
| 140 / 56 | 1,000 | 6,553 | 3,850 | 126 | 126 | 5,515 | 2,917 |
| 140 / 56 | 1,400 | 16,662 | 9,807 | 66 | 52 | 15,102 | 6,571 |

Kilotonnes; "per-book min" is taken per seed and book, then summed. The
clearing fills 79–100% of what each book could match. What does not trade is
volume posted where the other side is absent: a forge bids for the supers it
holds least of and asks only the one it holds most of, so where a seed's
forges hold the same super most, that book has asks and no bids and the other
two have bids and no asks.

**Answer to the question: no.** The card-free bed carries no final demand for
supers — no order is billed in them, and the one use, apex, is bid for by no
one — so a measurement of trade in supers on it measures forges completing
sets for a material nothing consumes. A bed in which Designs are billed in
supers through a Design write (as §D.25's 25%-refined bills were) is what can
test it; the refined books also clear only twice after the first forge in a
1,500-yr run, at the 400-yr round cadence.

---

## D.35 The twin bed: Designs paid in supers, built preferentially

*Supports galaxy §3.1 and T-146. The author's direction: build alternate
test hulls that exactly match the basic hulls but are paid in supers, and build
them preferentially. Bed: `examples/super_census` with `SC_TWINS=1` (twin bill
a third each of Red, Green and Blue; card-free otherwise; standard galaxy,
3 seats, 1,500 yr, seeds 1, 7, 42, 31337), summed over the four runs.*

**First build, and why it carried no demand.** A twin want recorded only at a
decision that chose a hull lasted until the center's next decision, and most
decisions choose no hull: at the barriers the orders wanted 0.06–0.45 kt of
supers in all, on seed 1, and 33 of 23,254 hull orders were paid in supers.
The want now outlives decisions that choose no hull (a center keeps wanting the
twin of the hull Design it last chose), which raises it to ~25 kt per super at
each barrier on seed 1.

| color field (spacing / width, ly) | 10 / 5 | 70 / 38.5 | 140 / 56 |
|---|---|---|---|
| hull orders built | 91,010 (10,858 kt) | 93,250 (13,326 kt) | 92,672 (12,030 kt) |
| paid in supers | 293 (35.4 kt, 0.33%) | 229 (31.2 kt, 0.23%) | 205 (29.6 kt, 0.25%) |
| supers forged | 49,515 kt | 32,329 kt | 15,254 kt |
| drawn into apex | 28,506 kt (58%) | 11,804 kt (37%) | 3,536 kt (23%) |
| supers held at the horizon | 20,214 kt | 18,684 kt | 11,332 kt |
| supers delivered between empires | 725 kt (1.46%) | 1,648 kt (5.10%) | 348 kt (2.28%) |
| orders' want at 1,000 / 1,400 yr | 297 / 300 kt | 246 / 240 kt | 271 / 265 kt |
| supers at forges at 1,000 / 1,400 yr | 9,178 / 17,010 kt | 4,801 / 15,286 kt | 4,926 / 10,279 kt |

The supers forged are 1.3–4.6× all the kilotonnes of hulls built in the run,
and at each barrier the forges hold 16–57× what the orders want; 0.23–0.33% of
hull kilotonnes are paid in supers.

**Inference, stated as one:** within an empire, supers do not move from forges
to the yards that want them. A hauler takes refined material from a center
only as a stop on its own route to the center it serves, and the forges are a
few homeworlds among hundreds of wanting yards. Confidence about 70%; a census
of hauler stops at forges, and of refined kilotonnes delivered to centers by
freight, would test it directly.

**Card-free runs are unchanged**: `forge_census` on seeds 1 and 7 at 800 yr
reproduces `12e59f9` to every printed digit and event count.

**The freight census** (the same bed; `super_census` now reads the refined
part of every freight transfer, `LogEvent::FreighterTransfer::refined`).
Forges are the worlds at population `Band IV` at the horizon — 3 per run, the
homeworlds. Summed over the four seeds:

| color field (spacing / width, ly) | 10 / 5 | 70 / 38.5 | 140 / 56 |
|---|---|---|---|
| hauler pickups at forges | 585 of 328,634 (0.18%) | 905 of 320,533 (0.28%) | 1,236 of 243,139 (0.51%) |
| supers loaded at forges | 36.0 kt | 50.5 kt | 70.0 kt |
| supers loaded elsewhere (rocks, other centers) | 195.3 kt | 372.7 kt | 98.2 kt |
| supers delivered to worlds other than forges | 166.9 kt | 260.2 kt | 156.7 kt |
| hull kilotonnes paid in supers | 35.4 kt | 31.2 kt | 29.6 kt |
| supers held at forges at 1,400 yr | 17,010 kt | 15,286 kt | 10,279 kt |

**Measured:** freight takes 36–70 kt of supers out of the forges over a whole
run, against 10,279–17,010 kt standing at them at the last barrier — under 0.7%.
This confirms §D.35's inference that supers do not reach the yards from the
forges within an empire.

**A second gap, an inference:** 157–260 kt of supers reach yards by freight,
yet 30–35 kt of hulls are paid in them. A twin owes all three supers at once,
and a delivery carries what one forge, rock or Exchange fill held, usually one
super — the same conjunction as a works bill over the three basics (T-91).
Confidence about 60%; the composition of each yard's refined holding when its
twin is declined would test it.

---

## D.36 Supply runs for supers, and the Growth card on the twin bed

*Supports galaxy §4.5 (supply runs) and T-146. The author's rulings: forges
deliver to their own empire's yards; haulers buy at rival forges; both are
Doctrine. Then: play the Growth card on every seat of the twin bed. Bed:
`examples/super_census` with `SC_TWINS=1`, and `SC_CARD=3` (the Inscrutable
Growth card, `growth_rate` × 1.6, 0.5 kt) played on every seat at the first
barrier, 200 yr; 3 seats, 1,500 yr, seeds 1, 7, 42, 31337, summed.*

**Supply runs, no card.** On seed 1 (10 / 5 ly field): 1,784 runs from own
forges carrying 56.2 kt, 840 to rival forges buying 17.7 kt for 26.6 `$`;
hull kilotonnes paid in supers 11.5 → 17.6. Over the four seeds, 0.33–0.46% of
hull kilotonnes are paid in supers on the three fields. **When hulls are built
is the limit, measured:** on seed 1, 2,345 of 2,812 kt of hulls (83%) are
built before 600 yr, and the first forge forges at 605–765 yr; from 700 yr the
empires build 999 / 1,108 / 1,024 kt of hulls on the three fields and pay
4.3% / 4.2% / 3.0% of it in supers. Card-free runs are bit-identical to
`9a30e39` (`forge_census`, seeds 1 and 7, 800 yr).

**The Growth card on every seat** against no card, same bed and seeds:

| color field (spacing / width, ly) | 10 / 5 | 70 / 38.5 | 140 / 56 |
|---|---|---|---|
| hull kilotonnes paid in supers | 51.7 → 125.4 | 50.6 → 160.4 | 41.2 → 94.5 |
| seeds higher with the card | 4/4 | 4/4 | 4/4 |
| hull kilotonnes built | 11,183 → 11,414 | 13,667 → 13,597 | 12,345 → 12,087 |
| supers forged | 59,931 → 119,252 kt | 32,865 → 93,341 kt | 19,042 → 58,638 kt |
| supers delivered between empires | 724 → 1,733 kt | 1,494 → 3,768 kt | 247 → 1,227 kt |
| supplied from own forges | 224.9 → 290.3 kt | 203.9 → 279.8 kt | 226.2 → 236.1 kt |
| bought at rival forges | 32.0 → 23.4 kt | 39.5 → 16.8 kt | 28.7 → 17.6 kt |
| first forge, per seat | 460–670 yr | 460–540 yr | 455–690 yr |

The card brings the first forge from 605–765 yr to 455–690 yr and raises the
hull kilotonnes paid in supers by 2.3–3.2× on every field; the share of hull
kilotonnes paid in supers is 0.8–1.2%.

**Why forging starts near 600 yr, derived.** A homeworld starts at population
`Band II` (31.6 kt) under a ceiling of `Band 4.2` (2,163,979 kt); a forge needs
`Band IV` (715,542 kt), 22,644× the start. The logistic runs at
`r = growth_rate / rate_reference_years = 0.873 / 50 = 0.01746` per year (a
doubling time of 39.7 yr), so `t = ln[x₁(K − x₀) / (x₀(K − x₁))] / r` =
`ln(33,830) / 0.01746` = **597 yr**. Measured card-free: 605–765 yr. The
author set the 4.2 ceiling for this timing (§D.23: a growth-dedicated build
across before round two's selection at 600 yr, most builds by round three).
With the card `r` is 0.0279, at which the same formula from the starting
population gives 373 yr; played at 200 yr, measured 455–690 yr.

**What spreads the first forge across seats: emigration from the homeworld,
proven by ablation.** Twin bed, card-free, 900 yr, the four seeds (a scratch
harness, never landed). The first forge follows the homeworld's crossing of
`Band IV` by 0–15 yr in every seat, so the spread is population, not basics.
Population at 200 yr runs 17–907 kt against the logistic's ~1,036 kt; every
colony ship a homeworld launches carries 1 kt of its people, and the seats
that sent 174–270 ships before 200 yr cross at 721–761 yr or not by 900 yr,
two of them falling from 39 → 26 kt and 17 → 10 kt between 200 and 300 yr.
Seats that sent 8–148 cross at 605–641 yr. **Ablation:** with a homeworld's
population not debited for the settlers it launches (a scratch build), all 12
seats cross at 601 yr and forge at 600–603 yr, population 952 kt at 200 yr in
every seat.

**Why seats launch so differently — measured as far as the early freight.**
Same bed, 200 yr, the four seeds (scratch harnesses, never landed):

- **Not the neighborhood's colonizable worlds.** Worlds `k_high` admits
  within 25 ly of a homeworld: 46–76 per seat, with no order matching the
  ships launched (seed 1: 64 → 8 ships, 66 → 270).
- **The homeworld's spending.** It launches colony ships out of what it
  spends, and in 10 of 12 seats what it spends in 200 yr is within 3 kt of the
  freight delivered to it (2.7 to 190.6 kt). Seed 1 seat 0 spent 5.7 kt and
  sent 8 ships; the others spent 49–135 kt.
- **Not the ore within 25 ly**, by total or by color: it is at least 98% one
  color in 10 of 12 seats and ranges 32,685–1,028,834 kt with no order
  matching the freight.
- **The first freight home.** Its time runs 37–136 yr and its size 0.30–31.23
  kt; delivered home by 100 yr runs 0–69 kt. The seats with 42–69 kt by 100 yr
  launched 100–270 ships before 200 yr; those with 0–2 kt launched 8–116.
  Traced on seed 1: seat 1 crewed a 932-kt rock 1.2 ly from home with five
  miners at 5 yr and landed 31.23 kt at 70 yr, then put ten new crews out;
  seat 0 crewed its two companions and a 1.3-kt rock one miner each, its
  haulers carried 0.1–0.9 kt a load, mostly to a colony, and the first 0.91 kt
  reached home at 114 yr.

**Inference, stated as one:** the spread is the early economy compounding on
the first outposts — a rich rock in reach of the first decision gets a large
crew and a General hauler (31.6-kt hold, T-98), and a seat without one hauls
in Medium holds (0.91 kt) for its first century. Confidence about 60%. The
hold and crew of each seat's first outposts, and an arm with every homeworld
given one equal rich rock in reach, would settle it.

---

## D.42 The first forge's date: planted outposts, starting fleets, and the starting population

*Supports galaxy §3 and T-147. The author's direction: plant three mining
outposts, 120° apart, within the color regions, `Band I`, single color,
identical, 5 ly out; add starting freighters, miners and colony ships to
accelerate the time to forge; the target, a mean of 400 yr with a 5-yr standard
deviation. Bed: `examples/forge_time` (card-free, 3 seats, each homeworld's
population read once per simulated year; the economy ticks every 5 yr), on
`Homeworlds::ColorCentered` with the outposts; starting fleets through
`FleetSeeding` (one miner, one freighter and one Medium colonizer fleet per
seat, 0.3 kt each, a 15-ly surveyed start).*

**What sets the date.** `is_forge` is population at `Band IV` and nothing else,
and population follows the logistic toward the homeworld's `K` (`Band 4.2`,
2,163,979 kt) at `growth_rate / rate_reference_years` = 0.01746 per yr. From the
shipped start (`Band 2.0`, 31.6 kt) that is 597.3 yr to `Band IV`
(715,542 kt); minerals do not enter it. A colony ship's settlers are debited
from the homeworld (`embark`), so emigration can only delay it.

**First forge, `Identical` ground** (mean, standard deviation over all seats,
mean standard deviation within a galaxy; seeds 1, 7, 42, 31337):

| arm | mean | sd | within |
|---|---|---|---|
| trio (9 of 12 seats forged by 800 yr) | 688.4 | 17.7 | 12.0 |
| color-centered, outposts | 624.0 | 15.2 | 1.8 |
| + starting fleets (6 of 12 seats forged by 800 yr) | 676.5 | 4.2 | 4.1 |
| outposts, start `Band 2.75` | 408.2 | 2.5 | 0.5 |
| + starting fleets | 410.7 | 0.5 | 0.5 |
| outposts, start `Band 2.8`, with or without fleets | 395.7 | 0.5 | 0.5 |

The logistic alone predicts 404.2 yr at `Band 2.75` and 391.3 at `Band 2.8`.

**At `Band 2.785` with the starting fleets** (seeds 1, 7, 42, 31337, 2, 3, 5,
11): first forge **400.7 yr, standard deviation 0.5 yr** on `Random`,
`Identical` and `ColorRotated` ground alike — seat 0 at 400 and seats 1 and 2
at 401 in every galaxy, the order in which their economy ticks fall within a
year. Spread of colonies between empires at 1,500 yr on the same configuration
(12 galaxies, mean): `Random` 131.9, `Identical` 25.9, `ColorRotated` 55.0,
against 60.1 / 24.3 / 31.7 with the trio (§D.40).

---

## D.46 A forge's price falls with what it holds

*Supports T-147 and galaxy §4.5. The author's ruling: "forge price should vary
with its holding." Built as `forge_premium · B / (B + H)` — `H` everything the
forge holds, every tier, kt; `B` the price of a whole Band IV works stock, kt.
Bed: `examples/empire_spread`, 3 seats, 1,500 yr, the 12 seeds of §D.37,
against the engine with per-color prices (§D.45). Paired over galaxies: cv
difference and relative level change, mean ± standard error.*

| ground | Growth level | Growth cv | supers level | supers cv | apex level | apex cv | stalled center-years |
|---|---|---|---|---|---|---|---|
| `Random` | +172.1% ± 7.1 | −0.266 ± 0.081 (11/12 lower) | −53.3% ± 5.0 | −0.084 ± 0.024 | −52.2% ± 3.8 | +0.057 ± 0.028 | +10.4% ± 1.4 |
| `ColorRotated` | +162.3% ± 10.1 | −0.129 ± 0.072 | −56.9% ± 5.8 | −0.087 ± 0.024 | −55.2% ± 6.8 | +0.099 ± 0.053 | +11.7% ± 1.7 |
| `Identical` | +23.5% ± 3.7 | −0.018 ± 0.036 | −33.4% ± 3.8 | +0.002 ± 0.010 | −38.7% ± 6.7 | +0.108 ± 0.027 | +9.2% ± 2.8 |

Expansion and Production move by under 0.5% in level and 0.003 in cv.

**The mechanism, by trace** (seed 1, `ColorRotated`, seat 0, `ES_TRACE`): the
forge's share of its seat's freight is 81% in the 500s on both engines, then
falls under the holding-priced premium to 61% in the 800s, 27% in the 1000s and
2–5% from 1,200 yr, against 67–85% throughout on the old engine; freight
delivered elsewhere rises (in the 1,200s, 7,517 kt against 5,554). The basics a
forge stops drawing go to ordinary centers, which buy Bands (Growth) and then
wait on larger bills (stalls). **Inference, stated as one:** forging falls by
half because nothing card-free consumes supers or apex, so every forge's holding
only grows and its price only falls. Confidence about 75%; a bed with final
demand for supers (T-146's twin bed) would show whether the price recovers when
a forge's holding is drawn down.

**And it moved the Exchange.** `clearing_strikes_escrowed_contracts_without_moving_the_world`
failed under this change alone: a forge that has sold supers keeps them back
from apex, apex synthesis leaves slag, so the seller holds more and its price
reads it. The test now pins the premium at zero, which prices a forge at
nothing whatever it holds, and states that channel.

---

## D.48 Three defects the forge-price sweep surfaced

*Supports T-147. Found running `examples/forge_sweep` (twin bed) at low forge
premiums; each is fixed, and each is pinned by a test.*

1. **NaN from an empty bank.** `Minerals::try_take_total` admits an amount up
   to 1e-9 against a bank of exactly zero, and divided by the zero total. An
   order paid wholly in supers leaves its basic part as a rounding residue of
   `price − owed` (~1e-18); a forge that had synthesized its last basic held
   exactly zero; every color became `0 · ∞`. Seen as `supers inf apex inf` in 3
   of 80 sweep runs, first at 1,421.88 yr on seed 42 (premium 0.8), located by
   checking `Simulation::mass_ledger` after every event. The defect predates
   T-147; the twin bed at a low premium is what drains a forge to zero. Fixed:
   an empty bank pays a crumb with nothing (`an_empty_bank_pays_a_crumb_with_nothing`).
2. **A Limited freighter order resolved to the miner role.** `role_of` read
   the freighter Design only on the Medium and General hulls, and the T-98
   sizing builds Limited haulers too. Fixed by reading it on every Systems
   hull (`a_hauler_is_built_where_its_trip_is_worth_its_minerals`).
3. **A barrier storm.** With (2) fixed, seed 1 at the shipped premium built
   25,301 → 648,743 freighters between 1,000 and 1,060 yr, while 707 Exchange
   contracts settled and ore held at rocks rose 222 → 274 Mt. The backlog
   hauler was sized by `freighter_hull`, which reads the rock's mining rate, so
   a delivered pile at a small rock got a Limited hull whose hold is a sliver
   of the pile. Fixed: a backlog hauler takes the Systems hull, of those the
   center can pay for now, that lifts the most of the backlog per kilotonne
   (`Simulation::backlog_hull`). After the fix, seed 1 holds 27,000–31,700
   freighters to 1,500 yr at premiums 10 and 0.6, 87 s a run.

---

## D.49 The forge's price, swept on the twin bed

*Supports T-147. The author's ruling: forging cannot be evaluated without
demand from the alt Designs paid in supers, and the forge-price decisions are
made to maximize the tree metrics on the standard bed with those Designs, then
made default. Bed: `examples/forge_sweep` — the standard galaxy (`Random`
ground, trio homeworlds), 3 seats, 1,500 yr, every seat seeded twin Designs
billed a third each in Red, Green and Blue. Objective: the geometric mean over
Expansion, Growth and Production of each tree's stock divided by its value at
the old default (`premium 10, floor 0, scale 1`) on the same seed (`AGENTS.md`
§2's composite). Engine: after §D.48's fixes; the first two passes ran on the
defective engine and are discarded.*

Screen, seeds 1, 7, 42, 31337 (mean ± standard error of the composite):

| premium | floor | scale | composite | Growth | Production | supers | apex |
|---|---|---|---|---|---|---|---|
| 0.3 | 0 | 1 | +9.5% ± 1.8 | +33.8% | −1.8% | −93.3% | −93.5% |
| 1 | 0 | 1 | +9.0% ± 2.1 | +31.5% | −1.4% | −85.9% | −92.3% |
| 0.6 | 0 | 0.3 | +8.5% ± 2.2 | +27.2% | +0.5% | −93.0% | −94.3% |
| 0.6 | 0.3 | 1 | +8.4% ± 2.2 | +28.8% | −0.9% | −92.4% | −93.4% |
| 1 | 0.5 | 1 | +8.4% ± 2.4 | +28.0% | −0.4% | −80.5% | −85.1% |
| 0.6 | 0 | 1 | +8.1% ± 2.0 | +29.0% | −1.8% | −92.0% | −93.1% |
| 0.6 | 0 | 3 | +7.0% ± 2.1 | +25.3% | −2.1% | −91.5% | −93.3% |
| 3 | 0 | 1 | +5.5% ± 0.9 | +19.0% | −1.1% | −61.2% | −67.8% |
| 3 | 0.5 | 1 | +3.5% ± 1.0 | +12.2% | −1.0% | −37.0% | −37.6% |
| 10 | 0 | 1 | 0 | 0 | 0 | 0 | 0 |

Expansion moves by under 0.25% in every arm. Replication on seeds 2, 3, 5, 11
(chosen against nothing) and the pooled eight:

| config | seeds 1, 7, 42, 31337 | seeds 2, 3, 5, 11 | pooled, n = 8 |
|---|---|---|---|
| premium 0.3 | +9.47% ± 1.82, 4/4 | +7.99% ± 1.26, 4/4 | **+8.73% ± 1.06, 8/8** |
| premium 1 | +9.01% ± 2.12, 4/4 | +8.04% ± 0.89, 4/4 | +8.52% ± 1.08, 8/8 |
| premium 1, floor 0.5 | +8.37% ± 2.45, 4/4 | +7.24% ± 1.52, 4/4 | +7.80% ± 1.35, 8/8 |

**Shipped: premium 0.3, floor 0, scale 1** — the highest pooled composite;
premium 1 is inside its noise. The surface is a plateau below a premium of
about 1 and falls above it: every arm that raises supers and apex lowers
Growth, Expansion does not move, and Production moves by under 2.1% because
the hulls paid in supers are a small part of the fleet (183–234 kt of hull per
seat at the old default). **Inference, stated as one:** the tree composite
prices forging only through those hulls, so it prefers basics spent on works
to basics spent on supers; a final demand for supers larger than the twin
hulls' would move the optimum up. Confidence about 70%; a bed whose Designs
bill more of their price in supers would settle it.
