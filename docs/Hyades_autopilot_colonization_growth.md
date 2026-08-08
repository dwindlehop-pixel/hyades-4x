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

**R-AC3 (deferred):** whether the six vehicles own fixed sectors or pool on global nearest-unscanned — revisit once the Monte-Carlo system exists and prioritization can be measured. **R-AC4:** scan dwell-time and any close-scan range.

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

**R-AC11:** the value of `X` (and whether it varies by tree/strategy). **R-AC12:** how a center splits output across multiple pending targets, and the cadence (one build per cycle vs. parallel).

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
