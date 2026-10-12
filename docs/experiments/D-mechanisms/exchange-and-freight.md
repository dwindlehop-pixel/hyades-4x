# §D. Mechanisms — the Exchange and freight

*Part of the experiments record. Nothing here is normative. Table of contents: [`../README.md`](../README.md); how the record is organized: [`../AGENTS.md`](../AGENTS.md).*

---

## D.20 T-134 — the Exchange at a spatial equilibrium, the collection capacity it was missing, and one holding per (empire, planet)

*Supports `Hyades_matching.md` §8 and politics §2.14/§2.16. Bed: `examples/work_years`, 3 seats, 4,000 yr, seeds 1, 7, 42, 31337 and the replication set 2, 3, 5, 11; paired log-ratios with standard errors across the 8 seeds unless stated. Every ablation below was a scratch build, measured against the shipped binary, and none landed (`AGENTS.md` §6).*

**The optimal clearing, alone, regressed.** `clear_spatial` reproduced scipy's
LP optimum on 9 seed-1 books to every printed digit and passed every equilibrium
condition on 40 random books, and it cost **−18.89% ± 4.28% work-years, 0/8
seeds up** (89.75M → 74.55M). The Exchange switched off scored 71.30M, so the
shipped greedy wave was worth +25.9% and the optimal clearing kept +4.6% of it,
**while moving the same tonnage** (seed 1 at 1,500 yr: 146,792 → 147,804 kt).

**Five mechanisms refuted by ablation before the cause was found:**

| hypothesis | ablation | result |
|---|---|---|
| sellers now keep their reservation | reservations zeroed | seed 2, 1,500 yr: 5.43M → 5.92M (shipped 7.25M) |
| asks post the bill's share of a bank | asks post only the spare | 72.95M (8 seeds) |
| the buyer's collection leg is unpriced | leg cost + collection leg | 75.71M |
| settlement timed on the whole ask | timed on the sold lot | 76.26M |
| trade itself is harmful | Exchange off | 71.30M — worse |
| deliveries concentrated on few venues | each lot split over 4 venues | loaded 5.5% / 11.1% — unchanged |

**The census that found it** (per-lot tracking of Exchange ore through the
buyer's holding at the venue, proportional attribution on each load, seeds 2
and 7 at 1,500 yr). Every venue is served by one buyer hauler (p10–p90 = 1) with
a laden round trip of ~100–200 yr. Loaded fraction, lots delivered before 1,100 yr:

| lot size | shipped greedy (s2 / s7) | optimal clearing (s2 / s7) |
|---|---|---|
| 10–100 kt | 54.3% / 70.1% | 68.1% / 74.2% |
| 100–1,000 kt | 31.9% / 47.8% | 33.7% / 62.9% |
| ≥ 1,000 kt | none | **1.0% / 4.2%** (117k / 105k kt) |

At equal lot size the optimal clearing's lots are collected as fast or faster;
the whole deficit sits in lots of ≥ 1,000 kt, which the greedy wave never made
because a fill was one bid (≤ ~380 kt). The optimal clearing sends a buyer's
whole demand for a color through a few cheapest legs, mostly to rocks already
rich in that color (91% of kt on seed 2). Refuted along the way by the same
census: the delivered color being unwanted by the venue hauler's destination
(similar shares, similar rates) and longer round trips (medians 142 vs 157 yr).

**The 2×2 that proved it** — capacity = what the buyer's based haulers move in a
400-yr round less what already waits, per color, spilling to the next shared rock:

| | no cap | cap |
|---|---|---|
| greedy wave | 89.75M | 140.87M, **+45.35% ± 3.14%**, 8/8 |
| optimal clearing | 74.55M, −18.89% ± 4.28%, 0/8 | 139.33M, **−1.27% ± 2.84%** against greedy+cap, 4/8 |

The cap carries the whole effect; once it is present the clearing rule does not
move work-years. Loaded fraction under the cap: 36.9% / 37.3% (seeds 2 / 7),
against 18.0% / 24.8% for the shipped wave.

**The landed stages** (engine code, not scratch):

| stage | what | result |
|---|---|---|
| A | one holding per (empire, planet) | **bit-identical** to stage 1 on seeds 1, 2, 7 at 1,500 yr (work-years, colony-years, works, colonies, vehicles, events) |
| B | capacity as route capacities in the LP, then placement | **+47.29% ± 3.04%**, 8/8 against the shipped wave |
| B+C | plus asks from every holding away from a yard | **+51.27% ± 1.71%**, 8/8; against B **+3.98% ± 2.09%, 6/8 — not resolved** |

Colony-years moved by −0.00% under B+C.

**Throughput.** Stage A's first version kept every holding in one ordered map
and was bit-identical at **+19% to +27% per event**; indexing a holding at an
owned planet by the planet removed it (stage A per event 10,353–11,776 ns
against stage 1's 12,175–13,734, 6/6 pairs). Stage B+C's first version summed
each route's room over a sorted venue list per ask — 3.46 s of a 12.8 s run
spent building routes, against 0.010 s solving and 0.013 s placing — and cost
−17% yr/s; computing the shared-rock set and its room once per (seller, buyer,
color) is bit-identical and runs **145.1–156.8 yr/s against the shipped
binary's 128.4–135.7** (3 seats, 1,500 yr, seeds 1 and 7, 3 interleaved rounds;
10,874–11,993 against 12,645–13,116 ns/event). 12-seat combat bench:
11.51–11.59 against 11.67–11.97 yr/s with 9% more events at lower ns/event.

**Two habits from it.** *A proven-optimal allocation is optimal for the model it
was given* — the LP treated ore dropped at a rock as delivered, and the engine
only moves it at one hauler's rate; the census, not the objective, named the
missing term. And *ablate the pieces of a 2×2 apart before crediting either*:
the clearing rule looked like the cause of a −19% and the cure of a +44%, and it
was neither.

---

## D.21 T-134 stage 2 — the internal duty exchange, and self-trade

*Supports `Hyades_matching.md` §9, `Hyades_warfare_tree.md` §8.20 and roles
§4.5b. Card-free bed: `examples/work_years`, 3 seats, 4,000 yr, the 8 seeds of
§D.20, paired log-ratios against the branch binary before stage 2 (`c5df349`),
standard errors across seeds. Combat bed: `examples/combat_bench`, 12 seats,
both cards at the barrier, 450 yr. Every scratch arm below was measured against
the shipped binary and did not land.*

**Self-trade — the question asked of stage 1's clearing.** An arm that let an
empire's asks fill its own bids scored **−2.51% ± 1.59% work-years, 2/8 seeds
up**, and colony-years **+0.00% ± 0.01%**. Not resolved at 2 SE; the sign leans
harmful. An inference, held at about 60% confidence: a self-trade moves ore an
empire could spend at a center out to its own pile at a shared rock, where it
waits for a hauler. A per-lot census of self-filled lots by where they were
spent would change that. The shipped clearing drops self-routes.

**Why self-trade moves nothing — a census of the book** (`examples/holding_demand`,
3 seats, seeds 1 and 7, every barrier to 2,000 yr; `Simulation::book_census`
reads the book as posted and the fills as struck). There is demand for what the
Holdings hold, and it is small against them: from year 600 on, bids are 1–19%
of what is held away from yards (seed 1, 1,000 yr, Cyan: 42,627 kt bid against
1,806,461 kt held away; seed 1, 1,800 yr, Yellow: 348,940 against 1,869,490).
Centers also bank more in colors they are not short of than all the bids
combined in most rows. Bids are small because a center bids only its shortfall
against its next rung (the same caveat `unmet_color_demand` carries, §6.20).

Fills against bids, shipped (no self-routes) and a scratch build that allows them:

| seed, year, color | bid (kt) | filled, shipped | filled, self allowed | of which self | own haulers' spare room |
|---|---|---|---|---|---|
| 1, 600, C | 11,252 / 13,628 | 11,252 | 13,628 | 1,035 | 13,628 |
| 1, 1,000, C | 42,627 / 43,528 | 42,627 | 43,528 | 4,192 | 43,528 |
| 1, 1,800, C | 226,513 / 216,004 | 22,699 | 20,323 | 8,736 | 20,346 |
| 7, 1,000, Y | 62,025 / 57,538 | 62,025 | 57,538 | 21,388 | 57,538 |
| 7, 1,800, Y | 223,234 / 211,417 | 35,069 | 35,315 | 11,456 | 35,338 |

Two regimes, and self-trade adds nothing in either. **Early**, every bid fills
from rival sellers, and self-fills (7–40% of the volume) displace rivals. **Late**,
fills stop short of bids at the buyer's own haulers' spare room at the rocks it
shares with the seller — with self-trade allowed, fill equals that room to
within 0.4% in every row where it falls short of the bid — and a self-route consumes the same room. Across
rows the fill total moves −11% to +13% between the builds (one run per seed).
An inference: self-trade reassigns fills and burns `exp(−λt)` of what it moves,
which is where the −2.51% ± 1.59% work-years above would come from; a
per-lot account of burned mass would test it.

**The miner freight run alone** (matching §9.1): work-years **+0.63% ± 3.32%,
3/8 up**; colony-years −0.01% ± 0.02%; colonies identical on all 8 seeds. The
run fires: 2,998 and 1,817 runs on seeds 1 and 7 over 1,500 yr, most of them
after year 500.

**With the colony ship's run before embarking** (§9.2): work-years **+0.80% ±
1.89%, 3/8 up**; colony-years **+0.03% ± 0.02%, 7/8 up**; colonies identical on
all 8 seeds. Over 1,000 yr on seeds 1 and 7, miners make 2,723 and 1,460 runs and
colony ships 1,511 and 1,341. What triggers the colony run was measured before
it was built: 22.2% and 22.8% of colony ships launch with fewer settlers than the
hold carries (6,201 and 5,776 launches, seeds 1 and 7, 1,000 yr). Per-seed
work-years swing from −3.8% to +11.5%, which reads as a reordering of a
compounding run rather than a gradient; no default moved on this number.

**The picket sortie** (warfare §8.20). The first predicate — both hulls
standing when the encounter begins — was censused with a scratch print on the
combat bed, seed 1: **0 of 41,770** encounter starts had both standing. By role
pair, 3,240 were picket against picket, the rest a picket against a scout,
miner, freighter, colony ship or withdrawing hull. For picket-against-picket
starts, the shooter's side had another post within 1 ly in 1,228 and the
target's side within 2 ly in 11. With the shipped predicate (both armed, one
standing) and the 2 ly placeholder reach: **3 sorties on seed 1, 0 on seed 7**;
a scratch build at 8 ly flew 105 on seed 1.

**Superseded by the author's ruling on "nearby"** (warfare §8.20): the reach is
now belief about arriving before the battle is decided, with no distance
constant. The 2 ly and 8 ly counts above are a record of the retired rule.
`a_picket_joins_a_battle_it_can_reach_in_time_and_returns_to_its_post` sets
the fight's believed length to twice and half the picket's light-plus-flight
time and asserts one sortie and none. Its first run failed on a real defect —
the arrival check read `world.position`, which a parked hull does not carry, so
every flight time was infinite; it now reads the hull's position through its
motion.

**The belief rule on the card bed.** Two interleaved rounds against the fixed
reach, seeds 1 and 7, 450 yr: **0 sorties on both seeds** (3 and 0 before), and
`ns/event` 3,522–3,568 against 3,461–3,553 on seed 1 and 4,561–4,655 against
4,706–4,777 on seed 7 — within the run-to-run spread. A scratch census of every
pitched encounter on seed 1 (3,366; 3,165 Cairn against Cairn, 201 Cairn against
Tor): believed fight length p50 0.019 yr, maximum 0.026 yr; distance from the
battle to the nearest post of either side p05 5.94 ly, p50 14.06 ly, p95 49.7 ly;
encounters with any post nearer than the fight's length, in light-years: 0.
So the predicate cannot be true at current beam and structure magnitudes.

**Throughput.**

| bed | before stage 2 | stage 2 | reading |
|---|---|---|---|
| 3 seats, 1,500 yr, seeds 1 and 7, 3 interleaved rounds, ns/event | 11,815–12,124 and 11,881–12,170 | 12,116–13,857 and 11,575–12,403 | min-of-3 +2.5% and −2.6%: not resolved; events +0.9% and +1.1% |
| combat bench, seed 1, 2 interleaved rounds | 11.17 / 11.34 yr/s, 3,205 / 3,157 ns/event, 12,574,563 events | 10.87 / 10.92 yr/s, 3,408 / 3,393 ns/event, 12,144,877 events | about −3% yr/s: fewer events, each dearer |

Callgrind on the combat bed (seed 1, 300 yr) puts the new code's own
instructions at `best_delivery_center` 0.30% (regular haulers included),
`embark` 0.06% and `call_to_battle` 0.002%, against `fill_survey_candidates` at
45.5%. An inference: the combat bed's per-event rise is the changed run — which
hulls fly where, and so which survey scans run — and not the cost of the side
duties. A per-function comparison against the old binary's profile would test
it.

---

## D.22 R-MX8 — a center's abundance hauled to a center with demand

*Supports `Hyades_matching.md` §8.5. Card-free bed: `examples/work_years`,
3 seats, 4,000 yr, the 8 seeds of §D.20, paired log-ratios against `main` at
`c831aad`, standard errors across seeds. All four binaries ran at once on a
4-core container, so the `yr/s` column compares like with like and not with
other entries.*

**The objective.** Colony-years **+1.29% ± 0.22, 8/8 seeds up** (5.9 SE).
Work-years **+6.22% ± 3.62, 5/8 up** — 1.7 SE, not resolved; per seed −2.1% to
+28.9%, which reads as a reordering of a compounding run rather than a gradient.
Colonies are identical on all 8 seeds, because the bed is saturated at `k_high`
(`AGENTS.md` §7). Vehicles +10.1% ± 2.3, 7/8 up.

An inference: the colony-years gain comes from ore reaching young colonies'
first rungs sooner. Confidence about 70%. A per-colony census of the time from
founding to first build in both arms would confirm or refute it.

**Mechanism check** (a scratch counter over `FreighterTransfer` loads, keyed on
whether the planet was owned by the loading empire at that time; 3 seats,
1,500 yr). Loads at owned centers: **8.19% and 7.25% of loaded tonnage** on
seeds 1 and 7 (6,050 and 5,996 loads), against **0.00%** on `main`, which is
the control that says the counter reads what it claims to. On 6 seats, seed
31337, the first such load is at 32.4 yr. `examples/bank_mix` (seed 1, 800 yr),
`main` → R-MX8: freight's share of banked ore 8.88% → 9.97%, of which some is
ore banked twice (a center-to-center load re-enters a bank); median payable
fraction 0.054 → 0.059; infrastructure builds 523 → 650.

**What it cost, and where.** The first build ran `ns/event` **+32.4% ± 2.7, 8/8
seeds**, with events +2.6% — the "slower per unit" row of `AGENTS.md` §2's
reading table. Callgrind (seed 1, 1,000 yr, the three freight functions kept
out of line in a scratch build): program 43.5 G → 58.4 G instructions, of which
`next_pickup` went 5.1 G → 17.4 G — the offer priced for every owned center on
every milk-run stop (7.4 G) and the walk over every planet to find them
(about 4.9 G). Two changes, each bit-identical to the first build at 1,000 yr
on seeds 1 and 7 (events, colonies and population to the printed digit):

| build, 1,000 yr, seeds 1 and 7, 2 interleaved rounds | `ns/event` | `yr/s` | events |
|---|---|---|---|
| `main` | 14,915–16,633 | 128.4–136.9 | 468,107 / 512,219 |
| R-MX8, first build | 18,366–21,232 | 86.7–98.8 | 543,252 / 551,095 |
| + the buyer's side read once, abundance tested before the leg | 18,084–19,926 | 91.6–100.3 | same |
| + `owned_planets`, a per-seat index of owned planets | **14,276–15,080** | **122.1–127.1** | same |

The index also serves `best_delivery_center`, which walked the galaxy on `main`
too (7.8 G of `main`'s 43.5 G). At 1,000 yr the per-event cost is at or below
`main`'s, and the remaining `yr/s` gap is the +16% and +8% more events the
changed run does. **On the full bed** (8 seeds, 4,000 yr, the final build
reproducing the first build's work-years, colony-years, colonies and events on
every seed): `ns/event` **+2.18% ± 1.06, 6/8 up**; `yr/s` −4.64% ± 0.93;
events +2.62% ± 0.22. The residual per-event cost is 2.1 SE and is not
located; a callgrind pair at 4,000 yr would locate it.

**Test budget.** `no_nan_or_infinity_reaches_replicated_state` went 37.4 →
44.4 s at 200 yr and the determinism target 53.9–55.0 → 62.8–62.9 s. Trimmed to
150 yr (probe in the test's comment), the target runs 56.1–58.4 s; at 130 yr it
is 55.0–56.2 s, so that test no longer sets the target's time —
`stepping_in_any_granularity_reaches_the_same_state` (35.6 s on `main`, 36.0 s
here) and scheduling do. The target is under 60 s and above the 54 s the
tolerance band asks a fix to reach; `main` itself runs it at 53.9–55.0 s.

---

## D.47 A hauler priced against the shipping backlog

> **Superseded measurement (§D.48).** The tables below were taken on an engine
> with two defects in this change: a backlog hauler was sized to the rock's
> mining rate, so a pile the Exchange dropped at a small rock got Limited
> hulls and seed 1 built hundreds of thousands of them after the 1,000-year
> barrier; and a Limited freighter order resolved to the miner role. Read the
> tables as a record of that engine; §D.49 measures the corrected one.

*Supports T-147, R-P19 and roles §4.4. The author's direction: "price of
building a new hauler should increase with high demand in shipping", read (the
author's choice) as the hauler's value rising so more are built; the bill stays
the hull's dry mass (design law #11). Built: a freighter Design ordered on its
own; per rock, the wanted ore beyond the holds based there; one trip valued at
the empire's want-weighted prices times the share of haulers sent to that rock
not lost there; the cost valued at the building center's own prices; built in
the fallback slot ahead of survey; Reserve first. Bed as §D.46.*

**First build: it ran away.** The backlog was read off the pile, which cannot
see haulers flying to it: seed 1 built **152,874** freighters by 1,500 yr
against 5,620, events 1.0 M → 3.55 M (`hauler_census`, scratch). Subtracting the
holds of the haulers based at each rock bounded it at **33,217**, events 1.43 M,
ore waiting at outposts 302 Mt against 764 Mt.

**Hauler alone** (paired against §D.45's engine):

| ground | Growth level | Production level | supers level | apex level | supers cv | apex cv | stalled center-years |
|---|---|---|---|---|---|---|---|
| `Random` | +155.4% ± 15.6 | +585.2% ± 5.3 | +416.6% ± 7.9 | +455.6% ± 6.0 | −0.060 ± 0.026 | +0.006 ± 0.033 | +25.0% ± 2.3 |
| `ColorRotated` | +104.4% ± 13.1 | +526.6% ± 4.8 | +357.8% ± 7.0 | +479.6% ± 5.9 | −0.079 ± 0.032 | −0.071 ± 0.026 | +24.3% ± 2.9 |
| `Identical` | −29.8% ± 9.0 | +370.9% ± 7.9 | +55.3% ± 17.5 | −32.0% ± 12.5 | −0.010 ± 0.016 | −0.121 ± 0.055 | +24.4% ± 3.1 |

**Forge price and hauler together** (what lands):

| ground | Growth level | Growth cv | Production level | supers level | apex level | stalled center-years |
|---|---|---|---|---|---|---|
| `Random` | +921.6% ± 18.4 | −0.263 ± 0.068 (11/12 lower) | +693.0% ± 6.0 | +64.0% ± 6.2 | +67.6% ± 4.6 | +31.3% ± 3.3 |
| `ColorRotated` | +633.8% ± 13.9 | −0.175 ± 0.061 (9/12 lower) | +618.0% ± 5.4 | +52.8% ± 5.7 | +72.6% ± 5.6 | +30.2% ± 2.6 |
| `Identical` | +8.5% ± 11.8 | −0.120 ± 0.088 | +391.0% ± 6.3 | +2.0% ± 14.3 | −58.3% ± 13.8 | +38.6% ± 2.8 |

Expansion falls 0.8–1.0% (standard error 0.1–0.2) on every ground and arm.
Production is `∫` fleet volume, so it counts the haulers themselves. No hull is
armed card-free, so the survival share is 1 there; the landed engine
reproduces the measured combined arm bit for bit on seeds 1 and 7.

**The loss in the price.** On the determinism card bed (six seats, Warfare,
Growth and missile cards, 350 yr), seed 7 ran 3.43 M events against 0.97 M:
haulers sent to rocks among the Warfare card's pickets, wrecked or withdrawn,
and replaced while the backlog stood (in the last 50 years, 616 freighters
spawned, encounters 11,063 → 19,397, withdrawals 1,338 → 2,324). Valuing a trip
at the share of haulers that came back (R-WAR47's rule) took it to 2.64 M
events and 459 freighters spawned. **Inference, stated as one:** the rest is
haulers flying among rival pickets, which is the conflict R-P19 asks for,
paid in fire events. Confidence about 55%; a census of encounters by role
would settle it.

**Cost.** Telemetry bed (3 seats, 500 planets, 600 yr, seed 1), release:
29,753 → 44,360 events, 1,455–1,871 → 2,300–2,933 ns per event, 12,455 → 22,147
instructions per event — routing decisions (`best_delivery_center`,
`offer_from`) are each `O(centers)` and there are six times the haulers to make
them. Pricing all three colors from one read of the bill took instructions
1,029 M → 982 M with the run bit-identical; per-color calls had re-read it. Test
targets, unloaded, old → new before the scenery changes: unit 5.9 → 20.4 s,
determinism 40.3 → 105.4 s, telemetry 32.8 → 68.0 s; after (card bed 600 → 400
planets, telemetry 800 → 400): 19.4 / 33.5 / 9.3 / 29.5 s.

---

## D.53 The voyage discount and the stops a leg may make, swept together

*Supports T-147, politics §1.4/§1.8 and industry §6.20. The author's
direction: sweep the levers behind the appendix's shifts larger than 2x, and
commit what improves the centers built. Bed: `examples/forge_sweep`
(`FS_LAMBDA`, `FS_STOPS`; twin bed, 3 seats, 1,500 yr, `completion_exponent =
0.75`). Score: the tree composite (§D.49) and **centers built** — owned worlds
whose works stand at Band IV at the horizon — both against `λ = 0.01`, `2`
stops on the same seed and ground. Screen on seeds 1, 7, 42, 31337, on
`Random` and `ColorRotated` ground. Of the appendix's >2x shifts, `λ`
(§A, 2.7x coverage) and the forge premium (§D.43, done at §D.49) are knobs;
the color-site spacing (§D.41) is the ground, and the backlog hauler (§D.47)
and the hull-sized hauler (T-98) are mechanisms already shipped.*

Mean over both grounds (each cell four seeds per ground):

| `λ` \ stops | 1 | 2 | 3 | 4 | 5 | 6 | 8 |
|---|---|---|---|---|---|---|---|
| 0.0025 | | −9.65% / −25.9% | | | | | |
| 0.005 | | −3.85% / −12.1% | | | | | |
| 0.01 | −43.26% / −65.0% | **0** | +2.98% / +10.3% | +4.58% / +14.9% | +3.36% / +14.2% | +4.42% / +16.4% | +2.62% / +13.8% |
| 0.02 | | +1.33% / +6.8% | | +6.86% / +25.0% | +7.69% / +26.4% | +7.57% / +29.3% | |
| 0.03 | | −1.67% / +3.1% | | +7.94% / +28.5% | +7.50% / +30.8% | +8.34% / +29.8% | |
| 0.04 | | | | +6.50% / +27.5% | +7.63% / +29.9% | **+9.12% / +30.7%** | |
| 0.05 | | −4.54% / −0.5% | | | | +7.94% / +26.8% | |

(composite / centers built.) Expansion falls with stops — about −0.9% at 3,
−1.7% at 4, −2.7% at 5, −3.6% at 6, −5.0% at 8 — and supers and apex forged
fall 60–85% in every arm with four or more stops. **The two levers interact:**
a sharper discount alone gains nothing past `0.02`, and more stops alone peak
near +4.6%; together they reach +9%.

**Replication** on seeds 2, 3, 5, 11, both grounds (16 runs per arm):

| arm | seeds 1, 7, 42, 31337 | seeds 2, 3, 5, 11 | pooled, n = 16 | centers built, pooled |
|---|---|---|---|---|
| `λ 0.04`, 6 stops | +9.12% ± 0.87, 8/8 | +8.47% ± 1.36, 8/8 | +8.79% ± 0.78, 16/16 | +28.2% ± 1.6 |
| `λ 0.03`, 4 stops | +7.94% ± 0.87, 8/8 | +6.88% ± 1.19, 8/8 | +7.41% ± 0.72, 16/16 | +25.5% ± 1.5 |

Paired, `0.04 / 6` minus `0.03 / 4`: composite +1.38% ± 0.60 (12/16), centers
built +2.76% ± 1.01, Expansion −1.40% ± 0.23 (16/16 lower).

**A mass leak the sweep exposed.** At `λ = 0.04` with three or more stops,
`a_twin_bed_builds_from_supers_with_mass_conserved` lost about 1.0 kt.
Checking the ledger after every event named the first loss: a `DutyArrive` at
723.5 yr, where cargo fell 0.1595 kt with nothing gained elsewhere. The hauler
had been parked in Reserve at 689.8 yr *laden*: a milk run of three or more
stops can come back to its own base, and the retirement test there (an
exhausted rock, nothing loaded at this stop) ignored what the earlier stops had
put in the hold; the side run that took it from Reserve wrote over the hold. At
two stops a leg's second stop is never its base, so the shipped engine could
not reach it. **Fixed:** a hauler retires only with an empty hold. `λ 0.01`,
2 stops reproduces the prior binary to every printed digit (seed 1).

**Shipped `λ = 0.04`, 6 stops**, measured on the fixed engine, 16 runs:

| ground | composite | centers built | Growth | Production | Expansion | supers | apex | Growth cv, paired |
|---|---|---|---|---|---|---|---|---|
| `Random` | +8.55% ± 1.23, 8/8 | +29.7% ± 2.4 | +26.2% ± 2.6 | +6.3% ± 1.1 | −3.7% ± 0.5 | −80.6% ± 8.3 | −81.6% ± 8.2 | −0.002 ± 0.020 |
| `ColorRotated` | +9.34% ± 1.02, 8/8 | +28.8% ± 1.7 | +28.0% ± 1.7 | +6.0% ± 1.5 | −2.5% ± 0.7 | −78.3% ± 14.1 | −75.8% ± 11.5 | −0.040 ± 0.021 |
| both | **+8.95% ± 0.78, 16/16** | **+29.3% ± 1.4** | | | | | | |

**Cost.** Seed 1, 800 yr, one run each: 589,595 → 712,729 events (+21%) and
44,247 → 64,443 ns/event (+46%), 26.1 → 45.9 s. More work and dearer work;
the dearer part is `next_pickup`'s scan of every pile and center, run up to
five times per leg. Test targets on this container: unit 27.3 s, smoke 13.2 s,
telemetry 46.6 s, determinism 49.0 s.

**Inference, stated as one:** a sharper discount sends each pile's color to the
nearest center that wants it, and more stops let one leg assemble the colors a
bill lacks from several piles; together a leg finishes more bills per round
trip, and the forges, which want basics without end, receive less. Confidence
about 60%; a per-leg census of how many bills each delivery completes, at both
settings, would test it. The supers and apex lost are the cost the tree
composite does not price (§D.49's caveat).

---

## D.54 Freight routed by `$` at every stop: four arms, none landed

*Supports T-147. The author's direction: the missing unlock is dynamic
routing priced in `$`, not stop count; reprice the next leg at each stop,
deciding whether to take on cargo; throughput is not the goal until the
scheduling is found. Bed: §D.53's (twin bed, 3 seats, 1,500 yr, `λ = 0.04`),
scored against the shipped engine (`max_pickup_stops = 6`) on the same seed.
Each arm is a scratch build (never landed); the patch is kept outside the
repository. A scratch census (`freight_census`) splits freight into deliveries
× kt per delivery and reports the fleet every 250 yr.*

**Arm 1 — the next pile by the empire's posted price.** After each delivery a
hauler goes to the pile whose one hold is worth most at its empire's
want-weighted color prices, discounted by the empty leg; a claim book keeps
haulers off a pile already claimed, and the pile becomes its base.

| stops | composite | centers built | runs |
|---|---|---|---|
| 6 | −6.87% ± 1.18 | −27.0% ± 3.1 | 0/8 positive |
| 2 | −23.33% ± 2.06 | −58.5% ± 4.0 | 0/8 |

Decomposition (seeds 1 and 7, `Random`, per seat, against shipped):
deliveries +65–80%, kt per delivery 6.0–8.1 → 3.0–5.4, total kt delivered 0
to −22%, ore left at outposts −30% to −75%, leg lengths unchanged (30–39 ly).
The stockpiles move; each hold carries half as much.

**Arm 2 — the same, with the four best piles priced at their real buyer
over both legs.** −6.11% ± 1.40, centers built −28.9% ± 3.3, 0/8.

**Arm 3 — the next leg priced at every stop** (the author's direction):
deliver now, or detour to one of the three best piles and deliver the larger
load — whichever is worth more at its best buyer's prices, discounted over
every leg; each pile loads only what that buyer wants; stop cap 16. Seed 1,
500 yr, against shipped: Growth 54k → 371k, 53k → 231k, 71k → 83k kt-years
per seat; Expansion +6% to +51%; centers built 3 → 9; cost 69 s against 6 s.
At 1,500 yr (4 seeds, `Random`): **composite −41.50% ± 3.59, centers built
down about two thirds, 0/4.** The time series (seed 1) says when:

| per seat | shipped | arm 3 |
|---|---|---|
| freight, 250–500 yr | 5.5–10.2 Mt | 19.2–23.5 Mt |
| freight, 750–1,000 yr | 112–133 Mt | 66–82 Mt |
| freighters at 1,000 yr | 9.1k–9.9k | 12.4k–15.8k |
| miners at 1,000 yr | 1.8k–2.6k | 1.6k–1.7k |
| kt per delivery, 750–1,000 yr | 5.9–6.5 | 2.1–3.2 |
| ore at outposts, 1,000 yr | 53–95 Mt | 15–46 Mt |

**Arm 3 with two fleet-size rules.** The hauler order priced a per-rock
backlog net of the haulers *based* at each rock, and a roaming hauler moves
its base every trip.
- *The whole fleet against the whole stock:* freighters fall to about 2,200
  per seat, ore at outposts climbs to 128–158 Mt by 1,000 yr, freight 22–27 Mt
  per 250 yr.
- *A new hauler priced by the route it would fly* (the order fires when a
  fresh hauler's best move from the center is worth more than the hull):
  freighters reach 26k–30k per seat by 750 yr, 0.6–2.9 kt per delivery.

**Inference, stated as one:** pricing each leg at its buyer routes freight
better than the welded base — arm 3 delivers two to four times as much in
its first 500 years — but nothing in these arms prices the *size of the
fleet* correctly once haulers roam. A route's value says what one more trip
earns now, the backlog compares a stock with holds per trip, and the claim
book reserves only piles that haulers are flying to; none of them sees the
rate at which mining refills the piles, which is what a marginal hauler
competes for. Confidence about 60%; a fleet-size rule priced against the
piles' refill rate (or a per-cycle assignment of haulers to piles and
buyers), measured on the same bed, would test it.

**Refuted by §D.55.** The fleet-size and load-size ablations moved
kilotonnes per delivery by about 1 kt; the cause was that these arms priced
outpost piles only, which drops center-to-center freight.

---

## D.55 Why per-stop routing fell behind after 500 years: it never loaded at a center

*Supports T-147. The author's direction: run experiments to explain §D.54's
turnaround after 500 yr, then fix it. Bed: §D.53's twin bed (3 seats,
1,500 yr, `λ = 0.04`), `Random` ground, scored against the shipped engine
(`max_pickup_stops = 6`) on the same seed. Arms are scratch builds; a scratch
census (`freight_census`) reports per seat every 250 yr: ore mined at outposts
and at centers, kilotonnes loaded at outposts and at centers, colonies, and
centers with works at Band III and Band IV.*

**Two hypotheses refuted on seed 1, at 500 yr** (arm 3, stop cap 16):

| arm | kt per delivery, 250–500 yr | freighters at 500 yr |
|---|---|---|
| arm 3 | 2.3–3.1 | 4.7k–6.9k |
| + fill the hold on departure | 3.0–3.4 | 5.0k–5.6k |
| + base not moved (shipped fleet accounting) | 2.4–3.2 | 3.9k–4.2k |
| both | 3.0–3.6 | 3.5k–3.7k |

Neither the load cap nor the fleet count moves kilotonnes per delivery by
more than about 1 kt.

**Extraction is not it either.** Over 500–750 yr arm 3 mines 75–84 Mt per seat
at outposts against shipped's 77–88 Mt (seed 1) and lifts as much or more
from them (68–82 Mt against 65–72 Mt).

**The census that named it — loads by source, per seat per 250 yr:**

| window | shipped, at outposts | shipped, at centers | arm 3, at outposts | arm 3, at centers |
|---|---|---|---|---|
| seed 1, 250–500 | 11–17 Mt | 1.6–3.1 Mt | 30–33 Mt | 0 |
| seed 1, 500–750 | 65–72 Mt | 21–25 Mt | 68–82 Mt | 0.001–0.002 Mt |
| seed 1, 750–1,000 | 54–61 Mt | 54–72 Mt | — | — |
| seed 7, 500–750 | 72–100 Mt | 15–27 Mt | 108–146 Mt | 0.001–0.002 Mt |
| seed 7, 750–1,000 | 92–112 Mt | 61–74 Mt | — | — |

In shipped, freight that moves a center's abundance — its holding above its
own next works bill — to another center short of it (R-MX8) grows from 0.5–3 Mt
before 500 yr to the same size as outpost freight by 1,000 yr. Arm 3's planner
priced outpost piles only, so that freight stopped. Band III centers at 750 yr,
seed 1: shipped 48–77, arm 3 22–30.

**The ablation: centers in the planner.** Each center's abundance, net of what
haulers have claimed there, is ranked by the same upper bound as a pile; the
three best are priced exactly at their best buyer, capped by what the
Exchange's gate lets the center ship to that buyer (`center_offer`). At
750 yr, Band IV centers per seat: seed 1 30 / 26 / 16 against shipped
17 / 12 / 10 (arm 3 13 / 9 / 8); seed 7 25 / 30 / 44 against 19 / 14 / 16.
Loads at centers run 190–334 Mt per seat over 500–750 yr, three to five times
the outpost loads. At 1,500 yr, stop cap 16, against shipped on the same seed
(mean ± standard error over seeds; "runs" counts seed × ground):

| ground | seeds | composite | centers built | runs positive |
|---|---|---|---|---|
| — | arm 3 alone (§D.54), 1, 7, 42, 31337 | −41.50% ± 3.59 | about −65% | 0/4 |
| `Random` | 1, 7, 42, 31337 | +24.07% ± 3.70 | +41.3% ± 7.4 | 4/4 |
| `Random` | 2, 3, 5, 11 (chosen against nothing) | +21.16% ± 1.07 | +37.9% ± 5.7 | 4/4 |
| `ColorRotated` | 1, 7, 42, 31337 | +18.14% ± 2.96 | +35.1% ± 4.5 | 4/4 |
| **both** | **all twelve** | **+21.12% ± 1.64** | **+38.1% ± 3.2** | **12/12** |

Per tree over the twelve: Expansion +2.8% ± 0.5, Growth +39.6% ± 2.8,
Production +21.0% ± 2.9.

**Two variants, against that arm on the same seeds:**

| variant | composite | centers built | runs |
|---|---|---|---|
| stop cap 6 (the shipped `max_pickup_stops`) | −0.44% ± 0.33 | +2.1% ± 1.2 | 1/4 positive |
| a hauler may not detour to the pile it stands on | −2.37%, −1.85%, −2.77% | −2.8%, −6.9%, +2.8% | 0/3 |

Stop cap 6 is inside two standard errors of 16 on both measures and runs
20–28% fewer events, so `max_pickup_stops` stayed at 6 (since lowered to 2
with a kept, netted price table, §D.56); at 6 against shipped,
seeds 1, 7, 42, 31337: composite +23.63% ± 3.86, centers built +43.4% ± 8.6,
4/4. The zero-length detour is how a stop loads past its share of the room
(each color at most its share per stop, `take_for_deficit`), and removing it
costs on all three seeds measured; it is shipped as measured. Bit-identity of
the landed source with the scratch arm: seed 1, 600 yr, identical output.

**Supers and apex.** Under the new routing each seat's one forge holds
1.4–1.9 Mt of a single basic and none of the other two (seed 1), makes
238–520 kt of supers once near 500 yr and nothing after; apex is 0 on 24 of
24 seat-runs at stop cap 16 (shipped: 47–264 kt per seat) and −89.8% ± 8.4 at
6. Arm 3 alone had the same: 0–8 kt apex. The forge bids
`premium · B/(B+H)` at pressure 1 against centers whose completion term raises
theirs, so at the shipped premium 0.3 it wins no delivery of a color it lacks,
and the full-hold top-up that used to carry those colors in incidentally is
gone. Forge premium against 0.3 under the new routing, seeds 1 and 7:

| premium | composite | centers built | supers per seat | apex per seat |
|---|---|---|---|---|
| 1 | +0.04%, +0.83% | +1.1%, −5.7% | 0.5–5.4 Mt | 0–10.5 kt |
| 3 | +0.06%, −1.06% | −4.5%, −3.2% | 5.7–19.0 Mt | 32–623 kt |

A pass routing supers by the planner (each forge's supers priced against every
center owed them) left seed 1 unchanged: apex 0, supers within 45 kt. Not
shipped. Premium stays 0.3; what the card-free bed should forge is the
author's call (galaxy §4.5).

**Inference, stated as one:** the turnaround was the loss of center-to-center
freight, not a fleet-size or load-size effect; the planner priced every source
a hauler could load at except the one whose share of freight grows after
500 yr. Confidence about 85%, from the census on two seeds, an ablation that
reverses the sign on four, and a replication on four seeds and a second
ground; a census of where the planner's center loads go (to which buyers, and
how far each bill was from completion) would raise it or lower it.

---

## D.56 A cheaper freight doctrine: a kept price table, netted as haulers commit

*Supports T-147. The author's direction: approximate the per-stop freight
planner (§D.55) with a cheaper doctrine that improves yr/s at under 5% loss
of work-years. Bed: the twin bed (3 seats, 1,500 yr, `λ = 0.04`), work-years
= the Growth stock, scored against the exact planner (§D.55 at stop cap 6,
shortlist 3, prices read fresh) on the same seed; throughput is wall time of
one run at 800 yr, seed 1, run one at a time on an idle machine, interleaved
over two rounds where stated. Levers are two Doctrine fields,
`freight_shortlist` and `freight_price_age_years`, and
`SimConfig::max_pickup_stops`.*

**The levers alone** (four seeds; work-years vs exact; wall at 800 yr, exact
183.8 / 187.7 s, the engine before the planner 47.7 / 47.9 s):

| stop cap / shortlist / price age | work-years | composite | wall |
|---|---|---|---|
| 2 / 3 / fresh | −3.05% ± 0.70 | +1.09% ± 0.70 | — |
| 6 / 2 / fresh | −2.56% ± 3.18 | −1.85% ± 1.00 | — |
| 6 / 1 / fresh | −5.78% ± 1.16 | −2.56% ± 0.59 | 146.1 / 147.3 s |
| 3 / 3 / fresh | — | — | 140.0 / 143.6 s |
| 6 / 3 / 5 yr, not netted | −4.92% ± 4.34 | −7.27% ± 1.07 | 152.8 s |
| 2 / 1 / 5 yr, not netted | −8.25% ± 0.84 | −5.29% ± 0.71 | 77.2 s |

An exact pruning of the buyer search (sort centers by their best price,
stop at the first that cannot win) was bit-identical and slower, 263.9 s:
the bound ignores the voyage discount, which cuts a score to about a third.

**Netting.** A kept table is the centers' shortfalls and prices as read; when
a hauler turns for a buyer, that buyer's shortfall in the table falls by what
the hauler carries. Same price age, with and without it:

| price age | not netted | netted |
|---|---|---|
| 5 yr | −4.92% ± 4.34 | **+42.73% ± 6.87**, 4/4 |
| 25 yr | — | **+46.78% ± 5.50**, 4/4 |
| 50 yr | — | +34.04% ± 5.29, 4/4 |
| 100 yr | — | +10.05% ± 5.33, 3 seeds |

**Why the exact planner is far from optimal — the overshoot census** (scratch
build, seed 1, 1,500 yr): the share of kilotonnes delivered that exceeded
the buyer's shortfall when the hauler arrived.

| arm | deliveries | kt delivered | beyond the shortfall on arrival |
|---|---|---|---|
| exact | 1,179,000 | 5,660,513 | **0.930** |
| 25 yr, netted | 596,000 | 1,340,576 | **0.592** |

Every hauler prices the same unfilled shortfall, so haulers deciding close
together converge on one buyer and most of what arrives is past what it
needed. Some of it banks toward the next bill, so 0.930 bounds the waste
rather than estimating it.

**Netted, with the cheap levers** (price age 25 yr):

| stop cap / shortlist | work-years | composite | centers built | wall |
|---|---|---|---|---|
| 6 / 3 | +46.78% ± 5.50 | +20.06% ± 2.13 | +37.5% ± 8.4 | 175.5 / 174.9 s |
| 6 / 1 | +42.78% ± 5.53 | +18.90% ± 2.55 | +36.5% ± 9.2 | 136.9 s |
| 2 / 3 | +40.02% ± 5.59 | +20.47% ± 2.80 | +33.3% ± 8.9 | 102.6 s |
| **2 / 1** | **+39.33% ± 5.70** | **+20.11% ± 2.82** | **+33.6% ± 8.3** | **78.6 s** |
| 1 / 1 | +24.01% ± 5.05 | +21.24% ± 2.14 | +25.0% ± 6.4 | 67.1 s |

**Shipped: stop cap 2, shortlist 1, price age 25 yr.** Replicated on seeds
2, 3, 5, 11 (+30.95% ± 7.13 work-years, 4/4) and on `ColorRotated` ground
(+35.92% ± 7.64, 4/4); pooled over twelve runs **work-years +35.40% ± 3.74,
composite +18.78% ± 1.73, centers built +28.5% ± 4.2, 12/12**, at 2.4x the
exact planner's throughput. Against stop cap 6, shortlist 3 with the same
netting (+40.69% ± 3.97 pooled) it gives up 3.8% of work-years for 2.2x the
throughput. Against the engine before the planner, pooled twelve: composite
+40.68% ± 3.08, centers built +67.9% ± 7.5; that engine is still 1.6x faster
per simulated year (47.8 s against 78.6 s). Apex forged over twelve
seat-runs: 81 kt, against 144 exact and 1,370 before the planner (galaxy
§4.5's open question). The default's price age is a placeholder inside the
25-yr peak; the stop cap and shortlist were chosen for throughput.

**Inference, stated as one:** the exact planner's gap is coordination, not
precision — a fresher price helps one hauler and nothing tells the next one
it has been spoken for. Confidence about 80%, from a four-seed ablation at
one price age and the census on one seed; netting in-flight cargo against a
table read fresh at every decision would separate the netting from the
staleness, and would say whether a fresh read is worth its cost at all.
