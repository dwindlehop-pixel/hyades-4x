# Hyades — Autopilot: Colonization & Growth
*The autopilot behavior the simulation runs for the **Far Shore** (Expansion) and **The Greening** (Growth) verbs — detailed to the level needed to author those trees' cards and to run the event-scheduled simulation. Production and design (military hulls, the counter-graph) are deliberately **out of scope** here and come next. Companion to `Hyades_simulation_model.md` (sim §), `Hyades_galaxy_and_autopilot.md` (world model), and `Hyades_card_contract.md` (card law). Calls flagged **R-ACn**. **Rev 3:** cards target hexes (player), the autopilot targets planets (execution), a hex resolves onto its contained planets (§1). **Rev 2:** the simulation is **fully decoupled** from the command layer — continuous 3D, each **star system is one point** ('a planet'), **no hexes in the sim**; six cube-face survey directions stand; base productivity step is **20%**. (R-AC1/AC2 resolved; AC3 deferred to the Monte-Carlo phase; AC11 set.)*

---

## 1. Information model (applies to the whole sim)

- **Command view is omniscient over realized state** — planets, structures, and fleets are fully visible to the player for planning. (Hidden simultaneous *orders* are **not** part of realized state; they stay concealed until they resolve, preserving yomi.)
- **Simulation is decoupled and continuous.** The simulation has **no hexes and no boundaries**; it runs in continuous 3D space with each **star system abstracted to a single point** ('a planet'), and the autopilot reasons only over those points. **Hexes exist only in the command view.**
- **Two granularities.** **Cards target hexes** — the player decides **hex by hex**; the **autopilot targets planets** and acts **planet by planet**. A hex-targeted card resolves onto the **planet(s) the hex contains**. The player never micro-targets a planet.
- **The simulation runs on fog of war.** Autopilot **units act only on what they have scanned**; **stealth** can hide what they have not; and every reaction is **light-lagged** (below). The player can see, sooner than their empire can react — and spends cards to close that gap (still bounded by *c*).
- **Three scan tiers.** **Remote:** Biosphere and Habitability are known from interstellar distance (spectroscopy). **Close:** ownership, infrastructure and mineral density require a survey craft to visit. **Inferential (R-SIM3):** some facts are *deducible* at range without being directly read — a **pop-Band-IV world radiates the waste heat of billions**, so it is legible as occupied from home even though *whose* it is still needs a visit. Departure traffic is the second such signal, graded by repeat sightings, and is R-SIM4. Acting on the inferential tier is **off by default** and is something a card enables (§2); early game, an empire flies out and finds out.
- **Relativistic event-scheduling.** Cards issue **instant global orders**, but consequences propagate at light-speed: a response to an observation **N light-years away is queued N years in the future** (`Hyades_card_contract.md` §2). The scan→decide→build→dispatch loop below is therefore a chain of light-lagged scheduled events, not an instant pipeline.

**R-AC1 (resolved):** omniscient over realized state (planets/structures/fleets); hidden simultaneous orders excepted. **R-AC2 (resolved):** the sim is point/continuous with no hexes, so there is nothing to reconcile — the six cube-face directions are simply six headings in space.

---

## 2. Survey (the Explore behavior)

- **Initial production is Light Hulls** — survey craft, the first civilian class.
- The autopilot dispatches **6 Light Vehicles**, one along each of the **six cube-face headings** (±X/±Y/±Z) in continuous space — an omnidirectional fan-out, *expand in all directions*. (No hexes are involved.)
- They travel at **1 g constant acceleration** (relativistic torchships; crossings take the lifetimes the *c*-cap implies).
- **Survey loop:** a Light Vehicle flies to the nearest unscanned planet, **close-scans** it (resolving ownership, infrastructure if any, and mineral density), then finds **another unscanned planet** and repeats. The scan result must travel home before any production center can act on it (light-lag).
- **Target selection is fog-limited by construction, and fog is per player.** The candidate list handed to `Autopilot::choose_survey_target` is `SurveyView { id, position, habitability, biosphere, industrial_signature }` — §1's remote tier plus the one inferential signal — so a target can only be chosen on what an empire's instruments justify. Knowledge is a single set on the player, not per craft: a world any of this empire's scouts has been dispatched to is excluded for all of them, and nothing models an individual ship's private ignorance.

  The list is **not** filtered on ownership, which is a close-scan fact. It *is* filtered on the pop-Band-IV industrial signature — but only once `Doctrine::survey_avoids_inhabited` is set, which is off by default (R-SIM3). So the early-game behavior is deliberately naive: scouts fly at the nearest unvisited world even when it is visibly ablaze with industry, and the wasted hop is part of what early expansion costs. Learning to read the sky is something an empire earns.

- **The fan-out is the opening, not the whole survey.** The six bootstrap vehicles are free starting units; every later Light Vehicle is a **paid build from a production center** (§6), charged to that center's stockpile like any other order. Each survey chain ends after `max_survey_hops` worlds and the craft scraps at the nearest friendly colony (`Hyades_vehicle_roles.md` §4.1).

  This is the mechanism that had to exist for expansion to compound, and it was missing. With no replenishment, total exploration is the **fixed product** `players × survey_vehicles × max_survey_hops` — nothing an economy knob can move. Under the pre-ratification settings that product was `3 × 6 × 40 = 720`, and a seed-1 run spent it exactly: 655 distinct worlds of 6,725 (the shortfall being worlds two empires both visited), last scan at t≈2,187 of a 4,000-year horizon. Exploration stopped because the budget was gone, not because time or targets ran out. With paid replenishment and `max_survey_hops = 120`, survey instead scales with the number of production centers, which is what lets the map keep up with the colonies.

**R-AC3 (deferred):** whether the six vehicles own fixed sectors or pool on global nearest-unscanned — revisit once the Monte-Carlo system exists and prioritization can be measured. **R-AC4:** scan dwell-time and any close-scan range. **R-AC16 (resolved):** `Doctrine::survey_reserve`, the frontier size a center tries to keep ahead of itself before spending an otherwise-idle cycle on a scout. **1024**, ratified with the snowball defaults — survey has to scale with the empire or expansion outruns its own map. The knob is deliberately monotone (see §6a) so the offline search can refine it without risk of the collapse an earlier pre-emptive version produced.

---

## 3. Planet ranking (numeric; four classes)

Every close-scanned planet receives a **numeric rank** `R(planet)`; its **class** follows from where the score and its components land:

| Class | Signature | Handled by |
|---|---|---|
| **Production center** | high K-potential **and** hub value (mineral access / centrality) — like the homeworld | colony vehicle (§4) |
| **Colony** | high K-potential (min(hab, bio)), weak hub value | colony vehicle (§4) |
| **Mining outpost** | high mineral density, low K-potential — *can out-rank a colony* | mining vehicle + freighter (§5) |
| **Barren** | low on all | ignored |

Proposed components (sketch): `K_potential = min(hab, bio)` (the ceiling infra can be built to); `mineral_value = Σ density weighted by the civ's current scarcity`; `hub_value = f(K_potential, mineral reach, centrality to holdings)`. **Cards change rank — retroactively** (re-ranking already-scanned planets): terraforming lifts hab/bio → K-potential → a barren world becomes a colony; prospecting raises mineral_value → a colony becomes a mining outpost; etc. Because rank is numeric, all targeting below is a deterministic **argmax** (card-contract §6).

**R-AC5:** the rank formula, component weights, and the class thresholds. **R-AC6:** does ranking read remote data (hab/bio) before a close scan, giving provisional ranks that firm up on visit?

---

## 4. Colonization (the Expand behavior)

- **Production centers produce colony vehicles.**
- A colony vehicle targets the **highest-rank production-center-class** planet. **If no production-center planets remain unclaimed, it targets the highest-rank colony-class** planet.
- On arrival it **founds a colony** (low infrastructure), which begins its own production schedule (§6) and may itself mature into a production center.

So expansion is **production-centers-first, colonies-next**, always by descending rank — a deterministic priority the engine can compute and a card can re-order (by re-ranking, §3).

**R-AC7:** colony-vehicle cost/build-time and founding infrastructure. **R-AC8:** claim/contest rules when two empires target the same planet (resolved by arrival time under light-lag?).

---

## 5. Mining-outpost exploitation (the supply chain)

- A mining outpost is **not colonized**. Instead the **nearest production center produces a mining vehicle and a freighter**.
- The **mining vehicle** extracts at the outpost; the **freighter** hauls the minerals back to the production center. This is the physical realization of the synthesis supply chain (world-model R-M5): basics flow from outposts to pop-Band-IV forges.

### 5a. What happens when the rock runs dry — R-AC19 (resolved)

A pair is built for **one** rock: `Shuttle { outpost, .. }` fixes the pickup leg at spawn and only the *delivery* leg is need-routed. Exhaustion therefore used to end both hulls' working lives. Measured on seed 1 at the shipped defaults (`examples/mining_probe -- census`, 3 seats, 4,000 yr):

| | |
|---|---|
| outposts opened | 2,188 |
| outposts mined out | 2,029 (93%) |
| mean productive life of a rock | 808 yr |
| outpost-years spent on a **dead** rock | 1,115,236 of 2,843,918 — **39%** |

Two hulls per dead rock, bought and then idle for a mean of ~550 years each.

**And the freighter never even noticed.** `sys_mining_tick` stops when the *yield* falls to the floor (`density × outpost_mining_fraction ≤ density_floor`, i.e. metallicity 0.042 at the shipped values), while `sys_freighter_arrive` waited for metallicity `< density_floor` = 0.01. In that band the mine is dead and the hauler is not told, so it flies empty round trips for the rest of the match: measured on seed 1, **not one freighter of 2,655 ever reached its stand-down branch.** Both now use the miner's predicate, which is what the branch was always trying to express. **Resolved by `SimConfig::recycle_mining_pairs`:** an exhausted pair goes to **Reserve** — which is already what roles §4.6 says a standing mission that ends does, as against the *completable* mission of an exhausted Scout, which scraps — and the next center ordering a mining pair takes the reserved hulls **nearest its target** instead of buying new ones. No minerals, no build delay, only the flight from wherever the dead rock left them. Measured paired on the standard 4-seed bed: **+0.76 ± 0.42 points of colonies** (49.11% → 49.87%, per-seed [+0.1, −0.0, +1.7, +1.3]) — positive on three seeds of four but short of 2 SE, so **the flag ships off** until it clears.

That first measurement was taken with a **pricing fault in the decision**, since corrected: `ProductionContext::mining_pair_cost` still quoted the full price of a new pair, so a center too poor to buy one sat Idle beside hulls it already owned, and reserved pairs were only ever taken by centers rich enough not to need them. The context now quotes `Simulation::mining_pair_price` — the halves that are *not* in Reserve, which is exactly what the build step charges. The re-measurement under the corrected price is the number that decides whether this becomes the default; the +0.76 above stands as the measurement of the weaker version.

**R-AC9:** "nearest production center" computed under light-lag (true nearest vs. nearest-known). **R-AC10:** mining rate, freighter capacity/cadence, and whether a freighter round-trip is itself a scheduled light-lagged event.

### 5b. The mining knobs are measured, and they are nearly all noise

`examples/mining_probe` puts the whole mining policy through the `gradient_probe` method — CRN, paired central differences, elasticity, an SE on every number — on the standard 4-seed bed with **colonies** as the objective (outposts take no ownership, so coverage counts colonies exactly):

| knob | value | d/dln x | SE | verdict |
|---|---|---|---|---|
| `rank.mineral_high` | 2.0 | **+3.92** | 1.86 | raise it — the only one clearing 2 SE, and barely |
| `rank.mineral_pressure_gain` | 1.0 | −3.61 | 1.88 | ~noise |
| `outpost_mining_fraction` | 0.238 | +3.49 | 2.58 | ~noise |
| `rank.w_mineral` | 0.8 | +2.34 | 1.33 | ~noise |
| `mining_tick_years` | 50 | −1.97 | 2.32 | ~noise |
| `density_floor` | 0.01 | +0.07 | 0.14 | flat — inert here |

Two readings. **`outpost_mining_fraction` has gone quiet**: it measured +14.5 ± 5.8 at 0.20 before the gradient step and +3.49 ± 2.58 at the ratified 0.238, which is what a knob moved onto a local optimum should look like, and an independent confirmation of that step. And **five of six knobs cannot be told from noise**, which is the same verdict T-20 reached for the coverage objective as a whole: this surface is tuned out, and what is left is terms, not values. §5a is one.

---

## 6. The production schedule (the build cycle = Growth)

Each production center runs a repeating cycle:

1. **Improve own productivity by 20%** (the base value; infrastructure toward the `min(hab,bio)` ceiling, so population grows logistically toward K). This *is* the Greening, planet by planet. The **20%** is a doctrine parameter The Compass can retune.
2. **Build whatever the civ needs to exploit the highest-ranked unexploited planet** — a colony vehicle (colony / production-center target, §4) or a mining vehicle + freighter (mining outpost, §5).
3. **Repeat.**

The base step is **+20% productivity** per cycle (a doctrine parameter The Compass can retune): the share of effort ploughed back into productivity vs. spent reaching outward. Production centers that reach **pop-Band-IV** unlock capitals and synthesis (world-model §5.2) — but that is the Production/Technology frontier, beyond this doc.

### 6a. Tier gates and the order of preference (as implemented)

A center's development level decides which hull classes it may build at all —
**Band II unlocks limited, Band III unlocks medium, Band IV unlocks all**
(`Hyades_mineral_cost_curve.md` §2.6 for the general Band ladder). Both
thresholds are config (`SimConfig::limited_min_level`, `medium_min_level`).
The limited tier matters more than it looks: it is what lets a
Band-II center contribute survey instead of sitting idle, and most centers
spend most of the game at Band II.

Within a cycle the preference order is:

1. **Deepen** toward `K`, if `reinvest_bias` favors depth and the upgrade is funded.
2. **Expand** — colony vehicle or mining pair, by rank (§§4–5) — if funded.
3. **Survey**, as a *fallback* for a cycle that would otherwise be spent Idle,
   when the known frontier is below `survey_reserve` and a scout is affordable.
4. **Idle** (save toward whichever of the above was preferred but unfunded).

**Survey is a fallback, never a pre-emption**, and that ordering is load-bearing.
An earlier revision gave survey priority whenever the frontier was thin, which
made `survey_reserve` non-monotonic: on seed 1 a reserve of 256 reached 1,047
colonies while 4,096 collapsed to 3, because every center scouted every cycle and
none ever colonized. As a fallback the knob is monotone — raising it converts idle
cycles into survey and cannot starve expansion. The one exception is an empty
candidate list, where survey outranks everything because nothing else is possible.

**Deepening uses headroom, not whole levels.** The guard is `infra < k_potential`,
not `infra + 1 <= k_potential`. Since `K = min(hab, bio, infra)`, letting infra
take the last partial step buys nothing directly — but *blocking* it strands the
center below the population bands permanently. A world with `k_potential = 2.86`
sat at infra 2 under the old test, which pinned `K` at 2, which pinned population
at 2, which never crossed the level-3 band edge (≈2.675): it could never build
anything and accumulated minerals it could not spend. Measured on seed 1, 1,050 of
2,435 Idle decisions were centers in exactly that state, several holding 3.5–4.7
minerals against a 3-mineral upgrade.

**R-AC11:** the value of `X` (and whether it varies by tree/strategy). **R-AC12:** how a center splits output across multiple pending targets, and the cadence (one build per cycle vs. parallel). **R-AC17 (resolved — ratified: "the snowball is the design").** `RankWeights::k_high` was miscalibrated against the current galaxy and was the binding constraint on the entire expansion loop. At the old `1.5`, against a galaxy where 99% of planets have `min(hab, bio) ≥ 1.76`, the *Mining outpost* class of §3 was unreachable: measured on seed 1, **zero** mining pairs were ever ordered and **zero** freighter legs ever flew, leaving the need-based hauling of §5 dead code at runtime and colonies unable to afford the 5 minerals that reach infra 3.

**Now `3.2`**, just above the galaxy's median K (~3.22), so the low-K half of the galaxy classifies as mining and the high-K half as colonies. Probing `k_high` alone took seed 1 from 41 colonies to 537; with `survey_reserve = 1024`, `max_survey_hops = 120` and an 8,000-year horizon it reaches **100% of colonizable worlds on 4 of 4 test-bed seeds** (3,435/3,435 · 3,467/3,467 · 3,471/3,471 · 3,516/3,516).

**"Colonizable" there means *at or above `k_high`*, and that is the whole of T-20's gap.** The denominator in those four fractions is the set this threshold admits — 3,435 of seed 1's 6,725 planets, 51.1% of them — not the coverage objective's `min(hab, bio) > 0.01`, which is effectively every planet in the galaxy. Measured by `examples/reach_limit.rs` on the standard bed at the shipped 4,000-year horizon, the run reaches **90–100% of the above-gate set** (97.5% · 99.6% · 89.6% · 93.0%) while scoring 46–51% of the objective. So the limiter on coverage is this classification, not survey (0–2 worlds above the gate go unscanned) and not the mineral economy. Two consequences worth stating. The threshold is **not quite a fixed set**: population is paid for out of biosphere (L6) and `k_potential = min(hab, bio)`, so 240 / 207 / 286 / 275 worlds per seed end the run below a gate they started above — a settled world can drop out of the class that made it settleable. And lowering the gate to widen the ceiling directly shrinks the Mining-outpost class that funds expansion — the R-AC17 failure, run in the other direction. **R-AC18:** should the Colony class carry a K floor at all, or should `k_high` order *preference* rather than gate *eligibility*? The ceiling curve is in T-20; the two knobs must move together.

**The stalled configuration is no longer the reference.** Treat the snowball as the baseline the engine is tuned and benchmarked against; a run that plateaus at a few dozen colonies is now a regression, not a starting point. `centrality_scale` has *not* been recalibrated and remains open from the same ~25 ly-era tuning.

---

## 7. Defense (only the hook)

Default posture is **expand in all directions, defending if pressed**: a hostile detected within reach schedules a defensive reaction (light-lagged by the distance to the responder). The substance — formations, engagement, the wreck roll — belongs to the **warfare** autopilot and is out of scope here. **R-AC13:** the trigger and reaction for "if pressed" at the colonization layer (e.g., a colony vehicle re-routing away from a detected threat).

---

## 8. The civilian hull classes introduced here

This doc seeds the **civilian** end of the hull taxonomy (military classes wait for design):
**Light Hull / Light Vehicle** (survey, 1 g) · **Colony Vehicle** (founds colonies) · **Mining Vehicle** (extracts) · **Freighter** (hauls). **R-AC14:** their stats (accel, capacity, cost, build-time, pop-gate to produce), pending the production model.

---

## 9. Card hooks — what Far Shore & Greening cards will set

With this autopilot defined, the two trees' cards become authorable as **orders that edit it** (each instant-global, light-lag-realized, deterministic-best-target, per the contract):

- **The Far Shore (Expansion).** Survey volume/speed/range (more or faster Light Vehicles, longer close-scan range), directional or sector bias, colonization priority, claim rate, contest resolution.
- **The Greening (Growth).** The infrastructure step `X`, growth-rate, **K-ceiling lifts (terraforming hab/bio)** — which **retroactively re-rank** planets (§3) — and carrying-capacity effects (Liebig, world-model §2).

**R-AC15:** lock §§2–6 enough to write the first depth-1 beats of both trees (e.g., *Landfall*, *The Flourishing*) with real outcomes and costs.

---

## 10. Ratification points
- **R-AC1 resolved** (omniscient over realized state; orders excepted) · **R-AC2 resolved** (sim is point/continuous, no hexes) · **R-AC3 deferred** to Monte-Carlo phase · **R-AC4** scan dwell/range
- **R-AC5** rank formula + thresholds · **R-AC6** provisional remote ranks · **R-AC7** colony-vehicle cost/founding infra · **R-AC8** contested-claim resolution
- **R-AC9** nearest-center under lag · **R-AC10** mining/freighter rates · **R-AC11 resolved** (base step +20% productivity) · **R-AC12** multi-target output split/cadence
- **R-AC13** "if pressed" at the colonization layer · **R-AC14** civilian hull stats · **R-AC15** lock enough to author depth-1 Far Shore & Greening beats
- **R-AC16 resolved** (`survey_reserve` = 1024; §2) · **R-AC17 resolved** (`k_high` = 3.2, ratified — the snowball is the design; §6a). `centrality_scale` remains open from the same obsolete-extent tuning.
