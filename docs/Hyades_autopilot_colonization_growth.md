# Hyades — Autopilot: Colonization & Growth
*The autopilot behavior the simulation runs for the **Far Shore** (Expansion) and **The Greening** (Growth) verbs — detailed to the level needed to author those trees' cards and to run the event-scheduled simulation. Production and design (military hulls, the counter-graph) are deliberately **out of scope** here and come next. Companion to `Hyades_simulation_model.md` (sim §), `Hyades_galaxy_and_autopilot.md` (world model), and `Hyades_card_contract.md` (card law). Calls flagged **R-ACn**. **Rev 3:** cards target hexes (player), the autopilot targets planets (execution), a hex resolves onto its contained planets (§1). **Rev 2:** the simulation is **fully decoupled** from the command layer — continuous 3D, each **star system is one point** ('a planet'), **no hexes in the sim**; six cube-face survey directions stand; base productivity step is **20%**. (R-AC1/AC2 resolved; AC3 deferred to the Monte-Carlo phase; AC11 set.)*

---

## 1. Information model (applies to the whole sim)

- **Command view is omniscient over realized state** — planets, structures, and fleets are fully visible to the player for planning. (Hidden simultaneous *orders* are **not** part of realized state; they stay concealed until they resolve, preserving yomi.)
- **Simulation is decoupled and continuous.** The simulation has **no hexes and no boundaries**; it runs in continuous 3D space with each **star system abstracted to a single point** ('a planet'), and the autopilot reasons only over those points. **Hexes exist only in the command view.**
- **Two granularities.** **Cards target hexes** — the player decides **hex by hex**; the **autopilot targets planets** and acts **planet by planet**. A hex-targeted card resolves onto the **planet(s) the hex contains**. The player never micro-targets a planet.
- **The simulation runs on fog of war.** Autopilot **units act only on what they have scanned**; **stealth** can hide what they have not; and every reaction is **light-lagged** (below). The player can see, sooner than their empire can react — and spends cards to close that gap (still bounded by *c*).
- **Two scan tiers.** **Biosphere and Habitability are known from interstellar distance** (remote spectroscopy). **Ownership, infrastructure, and mineral density require a close-range scan** — a survey craft must visit. So a fresh planet is *partly* legible from home (its K-ceiling factors) and *fully* legible only after a visit.
- **Relativistic event-scheduling.** Cards issue **instant global orders**, but consequences propagate at light-speed: a response to an observation **N light-years away is queued N years in the future** (`Hyades_card_contract.md` §2). The scan→decide→build→dispatch loop below is therefore a chain of light-lagged scheduled events, not an instant pipeline.

**R-AC1 (resolved):** omniscient over realized state (planets/structures/fleets); hidden simultaneous orders excepted. **R-AC2 (resolved):** the sim is point/continuous with no hexes, so there is nothing to reconcile — the six cube-face directions are simply six headings in space.

---

## 2. Survey (the Explore behavior)

- **Initial production is Light Hulls** — survey craft, the first civilian class.
- The autopilot dispatches **6 Light Vehicles**, one along each of the **six cube-face headings** (±X/±Y/±Z) in continuous space — an omnidirectional fan-out, *expand in all directions*. (No hexes are involved.)
- They travel at **1 g constant acceleration** (relativistic torchships; crossings take the lifetimes the *c*-cap implies).
- **Survey loop:** a Light Vehicle flies to the nearest unscanned planet, **close-scans** it (resolving ownership, infrastructure if any, and mineral density), then finds **another unscanned planet** and repeats. The scan result must travel home before any production center can act on it (light-lag).

- **The fan-out is the opening, not the whole survey.** The six bootstrap vehicles are free starting units; every later Light Vehicle is a **paid build from a production center** (§6), charged to that center's stockpile like any other order. Each survey chain ends after `max_survey_hops` worlds and the craft scraps at the nearest friendly colony (`Hyades_vehicle_roles.md` §4.1), so without replenishment total exploration is the fixed product `players × survey_vehicles × max_survey_hops`. Measured on seed 1 at the defaults, that product is `3 × 6 × 40 = 720` and the run spends it exactly, reaching 655 distinct worlds of 6,725 (the difference is worlds two empires both visit) with the last scan landing at t≈2,187 of a 4,000-year horizon. Exploration stops because the budget is gone, not because time or targets ran out — which is why colonization cannot compound without the paid-build path above.

**R-AC3 (deferred):** whether the six vehicles own fixed sectors or pool on global nearest-unscanned — revisit once the Monte-Carlo system exists and prioritization can be measured. **R-AC4:** scan dwell-time and any close-scan range. **R-AC16 (new, open):** `Doctrine::survey_reserve`, the frontier size a center tries to keep ahead of itself before spending an otherwise-idle cycle on a scout. Default **64 — a placeholder**, not a measured optimum; seed-1 probing put the best value nearer 1024, and the knob is deliberately monotone (see §6) so a sweep can tune it safely.

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
- The **mining vehicle** extracts at the outpost; the **freighter** hauls the minerals back to the production center. This is the physical realization of the synthesis supply chain (world-model R-M5): basics flow from outposts to pop-4 forges.

**R-AC9:** "nearest production center" computed under light-lag (true nearest vs. nearest-known). **R-AC10:** mining rate, freighter capacity/cadence, and whether a freighter round-trip is itself a scheduled light-lagged event.

---

## 6. The production schedule (the build cycle = Growth)

Each production center runs a repeating cycle:

1. **Improve own productivity by 20%** (the base value; infrastructure toward the `min(hab,bio)` ceiling, so population grows logistically toward K). This *is* the Greening, planet by planet. The **20%** is a doctrine parameter The Compass can retune.
2. **Build whatever the civ needs to exploit the highest-ranked unexploited planet** — a colony vehicle (colony / production-center target, §4) or a mining vehicle + freighter (mining outpost, §5).
3. **Repeat.**

The base step is **+20% productivity** per cycle (a doctrine parameter The Compass can retune): the share of effort ploughed back into productivity vs. spent reaching outward. Production centers that reach **pop-4** unlock capitals and synthesis (world-model §5.2) — but that is the Production/Technology frontier, beyond this doc.

### 6a. Tier gates and the order of preference (as implemented)

A center's development level decides which hull classes it may build at all —
**2 unlocks limited, 3 unlocks medium, 4 unlocks all**. Both thresholds are
config (`SimConfig::limited_min_level`, `medium_min_level`). The limited tier
matters more than it looks: it is what lets a level-2 center contribute survey
instead of sitting idle, and most centers spend most of the game at level 2.

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

**R-AC11:** the value of `X` (and whether it varies by tree/strategy). **R-AC12:** how a center splits output across multiple pending targets, and the cadence (one build per cycle vs. parallel). **R-AC17 (new, open):** `RankWeights::k_high` is **miscalibrated against the current galaxy** and is the binding constraint on the whole expansion loop. It defaults to `1.5` while 99% of planets have `min(hab, bio) ≥ 1.76`, so the *Mining outpost* class in §3 is effectively unreachable — measured on seed 1, **zero** mining pairs are ever ordered and **zero** freighter legs ever fly, leaving the need-based hauling of §5 dead code at runtime and colonies unable to afford the 5 minerals that reach infra 3. Probing `k_high` alone (nothing else changed) took seed 1 from 41 colonies to 537 at `3.2`, and with `survey_reserve = 1024`, `max_survey_hops = 120` and an 8,000-year horizon to **3,435 of 3,435 colonizable worlds — full coverage**. The value is a Monte-Carlo-tuned rank weight, so per CLAUDE.md §6 it is **flagged here for ratification rather than changed**; CLAUDE.md §7 already lists `k_high`/`centrality_scale` as tuned to an obsolete ~25 ly galaxy extent.

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
- **R-AC16** `survey_reserve` magnitude (default 64 is a placeholder; §2) · **R-AC17** `k_high` recalibration — **the binding constraint on expansion**, flagged for ratification, not yet applied (§6a)
