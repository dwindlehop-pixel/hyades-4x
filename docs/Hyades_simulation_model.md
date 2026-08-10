# Hyades — Simulation Model
*Draft section for the design doc. Resolves the tension between the discrete card-tree and the granular, location-based auto-battler.*

## 0. The core tension

Hyades runs on two layers that must stay in lockstep:

- **Command layer** — discrete, deterministic, abstract. Each round the player makes a handful of choices from an ever-expanding **action tree** and **upgrade tree**. There is no shop randomness; this is the deliberate departure from traditional auto-battlers, whose drama comes from RNG draws. In Hyades the only uncertainty is *what the other players chose*.
- **Theater** — granular, location-based, visually simulated. The auto-battler, where fleets actually move, fight, and die in space. This is the visual payload and the source of moment-to-moment interest.

**A card never resolves combat by itself. A card compiles into one or more directives attached to specific units, hexes, or structures.** The theater then executes those directives in space. Designing a mechanic therefore means specifying *both halves*: the discrete card (where it sits on the tree, what it costs, what it decides) **and** the directive it injects into the theater (what the player actually sees happen).

## 0a. The foundational premise — you tune defaults, you don't micro

**The auto-battler carries out all the 4X verbs on its own.** Left alone, your fleets and colonies already explore, expand, exploit, and exterminate — moving, settling, producing, engaging, retreating — under sensible default behaviors. Hyades *is* an auto-battler: a human hand-flying the simulation would play it better, but the strategic layer deliberately forbids that micro.

**Player agency is the power to change the defaults** — for the situation in front of you *and the one you see coming.* A card is not a puppet-string on a single ship; it is an edit to the standing behavior of a fleet, a colony, or the whole board: *fight at range here, refuse this engagement, hold this formation, settle toward that mineral, raze on the way out.* You program the autopilot and watch it run; you do not steer frame by frame.

Everything below — the physics, the planet model, the verbs — *is* the default behavior. Every mechanic in the catalog is a way to **override a default**, conditioned on the present and the anticipated.

## 1. Spatial substrate — hex-zoned board, continuous-space combat

- The strategic board is **hexes**: systems, the planets in them, pop, structures, mineral yields.
- **Combat happens in the void.** Fleets fight in open space in and around a contested system, never on planet surfaces; when a system is contested they maneuver as bodies with position, heading, and velocity. This is what lets retreat "happen in real space."
- **Production is planetside by default.** Colonies on planets generate minerals and build units; the Saar-style mobile dock is the deliberate exception. Systems can differ (open void, asteroid clutter, gravity wells) and that may lightly colour movement or sensors, but there is **no terrain-cover layer for the void fight** — ground cover and battlefield reshaping don't belong. Planets, however, *are* engineerable through a disciplined three-factor model that governs population — see §2a. (This supersedes the earlier blanket "no terraforming": you reshape *planets and their carrying capacity*, never the space battlefield.)
- **Mineral geography** follows the intended 2-D gaussian (Eve moon-mineral style): what you can build is a function of *which planets you hold*. Location drives build; build drives tactics.

## 1a. Theater physics — deterministic by mandate

**The theater is fully deterministic except for the per-ship wreck roll (§4).** Given the same orders and the same board state, the resolution plays out identically every time. This is the load-bearing reason the *command* layer is deterministic: the player chooses among knowable, computable outcomes, and the only uncertainty is the opponents' hidden orders plus the bounded wreck odds — never the physics.

There is not much to the physics, by design:

- **Acceleration** — ships build velocity over time along a heading; this alone governs arrival order, pursuit, and escape (the engines half of the wreck lever).
- **Fire / engagement avoidance** — deterministic rules for dodging fire and for declining or accepting an engagement (kiting, screening, refusing the fight).
- **Fleet formation** — how a fleet arranges itself (line / screen / wedge), which decides who absorbs fire and how splash lands.

**Motion obeys special relativity.** Velocities are bounded by *c* and add relativistically, so engines give diminishing returns as ships go fast and there is a hard ceiling on closing/escape speed. This keeps the velocity lever physically grounded and, helpfully, naturally caps the tempo of intercepts and pursuits. (Gate/wormhole projection, §6, is the only way "around" the light-speed limit.)

> **Future experiment (down the line):** general relativity near a gravity well. Combat in orbit of a star or massive planet could differ enough from deep-space combat — ships that are *better in orbit than in the deep* — to add real tactical texture. This is explicitly an experiment to weigh against simulation overhead, not a committed feature; it pairs naturally with the orbital actions (bombardment, blockade) that already cluster around planets.

## 2. Entities the theater simulates

- **Ships** carry the stats that matter *spatially*: **engine/velocity** (arrival order, pursuit, escape), **weapon profile** (range + single-target burst vs. anti-swarm splash), **armor** (hit absorption), **role/formation slot**. Classes: civilian/production, basic military, advanced, armored, experimental/flagship.
- **Pop / colonies** grow logistically toward a per-planet carrying capacity **K**, then **lock at K** — a locked colony simply can't grow or produce further. The lock is an *economic ceiling, not a defensive state.* K is set by the three-factor planet model (§2a); making a colony hard to dislodge is something a player does with a card or structure, never a free consequence of being full.
- **Structures** are spatial: docks (a planetside build footprint), defenses (interdiction templates / PDS), gate endpoints. Some are mobile.
- **Minerals** — three basics (Cyan / Magenta / Yellow), three supers (Red / Green / Blue), and **Platinum**, the apex ultra-resource. The two color triads are deliberately the CMY and RGB primaries; Platinum is named and rendered as a metallic silver-white so the top tier reads instantly distinct from the six saturated hues. Minerals fund cards and builds.

## 2a. The planet model — three-factor carrying capacity

Borrowed in spirit from *Stars!* (Mare Crisium), which gates population on gravity / radiation / temperature. Hyades keeps the three-factor idea but chooses factors with different *malleability*, all abstracted onto the **same scale as population**:

- **Habitability** — the bundled triplet of gravity, radiation, temperature. The planet's fundamental ceiling. **Hardest to change** (only rare, expensive, late-game engineering moves it at all).
- **Biosphere** — living capacity. **Easy to destroy, hard to improve** — you can crater it in a strike; it recovers only slowly.
- **Infrastructure** — built capacity. **Easiest to build *and* to destroy** — the soft factor and the main wartime lever.

**Changing the factors.** Infrastructure is built and razed freely. Biosphere is slow and asymmetric — quick to wreck, expensive and gradual to restore. **Habitability barely moves at all early on:** shifting the gravity/radiation/temperature triplet is *slow, conditional (it requires an established colony plus a sustained resource commitment), and capped low in the early game.* Late-game **terraforming** technology is exactly what lifts those limits — the mid-to-late payoff that turns a marginal world viable, a deep investment a faction can build a whole strategy around.

Two rules bind the model:

- **Carrying capacity `K = min(habitability, biosphere, infrastructure)`.** Liebig's law of the minimum: the scarcest factor caps the planet, so pop grows logistically toward that minimum and locks there. "Lock at carrying capacity" is now per-planet and can sit well below a planet's potential.
- **Growth rate is shaped by all three factors together.** Even though only the minimum sets the ceiling, the *speed* of filling depends on habitability, biosphere, and infrastructure jointly — so two planets with the same K can fill at very different rates.

**Why the model exists — warfare and trade choices:**

- A **retreating defender** can raze infrastructure and poison biosphere on the way out, dropping K so the conqueror inherits a low-cap, slow-filling husk. Because biosphere recovers slowly, biosphere damage is *durable* denial; infrastructure damage is cheap but quickly repaired. This is the **scorched-earth lever** (the Red Army on the Eastern Front) — see proposal #8.
- An **invader** faces the mirror choice on conquest: **take it intact** (high value, but a large hostile population to hold) or **crater it** (worthless, but pacified). Degrading the world is always a trade of value for control.
- **Trade** moves the soft factors: commerce can raise a partner's infrastructure (and, slowly, biosphere), so trade becomes a way to lift another player's K — and embargo a way to let it decay.

This is planetside economy, not space-combat terrain: it never adds cover or initiative to the void fight. It is the disciplined version of "terraforming" — fast for infrastructure, slow for biosphere, glacial for habitability.

## 2b. How entities are evaluated — arrival-driven, locally scoped

Entities are **not** stepped by a per-tick sweep. This is a discrete-event
simulation: a ship re-decides what to do next **when it arrives somewhere**
(`ContactArrive`, `FreighterArrive`, `ColonyArrive`, `ScrapArrive`) — the only
moment its situation has actually changed. Production centers are the single
cadence-driven exception: one tick per center per `cycle_years`, because growth
is a rate, not an event. Everything else is arrival-driven.

That choice sets the engine's cost model, and both halves of it matter:

> `cost = entities × arrivals-per-entity × work-per-evaluation`

Fleet size grows without bound as an empire snowballs — that is the *design*, so
the first two factors are not available to trim. **Only `work-per-evaluation` is
ours to control, which is why an entity's decision must be scoped to what it
reads rather than to the size of the galaxy.** A query that walks every planet is
O(galaxy) per arrival and therefore O(entities × galaxy) overall; at full
colonization that is the difference between a sweep that finishes and one that
does not.

The worked example, because it was measured rather than assumed (seed 1, 3 seats,
full colonization, 4,000 yr): survey re-targeting fires on `ContactArrive`, the
correct trigger — but each call built a `PlanetView` for every unvisited world in
the galaxy, **15,653 evaluations × ~4,100–6,725 planets**, to serve a decision
that reads only `id` and `position` and keeps one result. Profiling put that one
query and its view construction at **81% of all engine instructions**. Getting the
trigger right does not by itself make an evaluation cheap.

**R-SIM1 (resolved).** `choose_survey_target` now takes `&[SurveyView]` —
`{ id, position, habitability, biosphere }` — instead of the 88-byte
`PlanetView`. This began as an optimization and turned out to be a **fog-of-war
correctness fix**, which is the more important half:

- The old candidate list filtered on **ground-truth ownership**, so a scout
  silently avoided worlds owned by an empire it had never met. §1 requires a
  close-range scan to learn ownership; that filter was omniscience.
- The old view also carried `minerals`, `owner` and `pop_level` for *unvisited*
  worlds — all three close-scan-only under §1's two-tier model.

`SurveyView` is exactly the remote tier, so the boundary is now enforced by the
type rather than by the policy choosing not to peek.

The two effects pull opposite ways on speed and both should be recorded.
Narrowing the view made each entry cheaper; dropping the ownership filter made
the list *longer*, because owned worlds are no longer skipped — and at full
colonization that is thousands of extra entries. Measured (seed 1, 3 seats):
horizon 2,500 went 1.188 s → 0.859 s (2,104 → 2,910 yr/s), while horizon 4,000
was a wash at ~8.9 s. **Fog of war is not free**, and the light view is roughly
what pays for it.

**Fog is per player, never per unit.** `Knowledge` is a single set on the player
entity — scanned, visited, targeted — so every craft an empire owns draws on the
same map, and a world one scout has been sent to is excluded for all of them.
There is deliberately no per-entity knowledge layer: it would be a second fog
model to keep coherent, it would cost per-unit storage against fleets that run to
thousands (§2b), and it would sit oddly beside a command layer whose cards issue
**instant global orders** (`Hyades_card_contract.md` §2). Where units *should*
lag their empire's knowledge, the mechanism is the existing **light-speed delay**
on event propagation, not a private per-ship map.

**R-SIM3 (resolved).** What may the autopilot know about a world nobody has
visited? Not a binary. **Occupancy is inferable at range, with confidence** —
ownership does not require a visit, it requires evidence:

- **Pop-4 industrial signature.** The waste heat and atmospheric chemistry of
  billions is not concealable; spectrometry reads it across interstellar
  distance. A world at the top population band is legible as *taken* without
  anyone going there. This is a third, **inferential** tier sitting between §1's
  remote tier (position, hab, bio) and its close-scan tier (ownership,
  infrastructure, mineral density) — it does not identify *whose* it is.
- **Departure traffic.** Ships observed leaving a world are evidence of activity,
  and repeated sightings raise confidence. Unlike pop-4 this is a *graded* signal
  accumulated over time, so it needs stored light-lagged observations rather than
  a threshold on current state — filed as R-SIM4.

**Crucially, none of it filters anything in the early game.** The default is to
fly out and find out: `Doctrine::survey_avoids_inhabited` is `false`, so a scout
targets the nearest unvisited world even when it is visibly ablaze with industry,
and the wasted hop is part of what early expansion costs. Acting on remote
signatures is something an empire *earns* — the field is exactly the shape a card
edits (`Hyades_card_contract.md` §5: one field, standing behavior), flipped when
the right instruments or board state arrive. The engine always reports the
signature; doctrine decides whether the policy is allowed to use it.

That split is the general answer to the §1 tension. The **command view** stays
omniscient over realized state (R-AC1) because the player is planning; the
**autopilot** acts on what its instruments justify, and what they justify widens
over the course of a game.

**R-SIM4 (new, open):** departure-traffic confidence. Needs per-player
accumulated observations of ship departures, light-lagged, with a confidence
that rises on repeat sightings and presumably decays — plus a decision about
where that state lives, since a per-player × per-planet observation table is the
kind of storage §2b warns about. The pop-4 threshold is deliberately the cheap
half of R-SIM3, taken first because it is exact and needs no new state.

**R-SIM2 (new, open):** the survey scan is still O(planets) per arrival. Position
is static, so a spatial index over planet points with a per-player visited mask
would make nearest-unvisited O(log n) — but `choose_survey_target` is policy, and
the engine cannot accelerate a query it does not define. Resolving this means
deciding whether the trait exposes *"nearest unvisited to a point, optionally
hemisphere-biased"* as an engine-provided primitive rather than a scan the policy
performs itself. Note an incremental frontier *list* was already tried and
reverted: it cut the scanned count 39% and ran slower, because swap-removal cost
more in locality than it saved in count (§2b).

## 3. The round loop

1. **Income & growth** — pop ticks up logistically, minerals accrue per hex, the tree expands, hands refill (deterministically — you choose, you do not roll).
2. **Command phase** *(discrete, hidden, simultaneous)* — every player plays cards face-down: **orders** (move/attack along a heading, encircle, cut retreat), **deployments** (build at docks), **upgrades** (reconfigure unit stats / unlock deeper tree), **reactions** (face-down traps keyed to an enemy move). Hidden + simultaneous = bluff and "modeling of mind."
3. **Resolution phase** *(the theater)* — directives execute in space, played out visually and **deterministically**: fleets accelerate along headings, engage at weapon range, and apply damage by stat + position + range/formation. The damage is fixed; the *one* live roll is per defeated ship — see §4. No dice in the fight itself.
4. **Aftermath** — territory/pop update, wrecks resolve (salvage / reanimation / devour), and board state recomputes card values for the next round. This is the flywheel: some cards scale on color counts, ground held, or enemy state.

## 4. Deterministic combat + the wreck roll

- **The fight is deterministic.** Given positions, stats, formations, and weapon profiles, the damage math is fixed. Skill lives in the card choices and the pre-positioning, not in luck.
- **There is no initiative — no "who goes first."** Exchanges resolve simultaneously, so advantage comes from position, range, and build, never from acting earlier in the tick. (No mechanic may grant turn-order priority.)
- **The wreck roll is the only stochastic beat — and it is per ship, not a fleet scatter.** When a ship is defeated, it tries to leave. Whether it is **wrecked** (destroyed) or **gets away** (retreats intact) is a single weighted coin-flip. Retreat *direction* is not random; it follows the board, toward open or friendly space. The randomness is only *whether the ship survives to use it.*
- **P(wreck) rises with the incoming damage on that ship and is bounded in the open interval (0, 1).** More damage → higher wreck odds, on a saturating curve that **never reaches 0% and never reaches 100%.** A barely-scratched ship can still be lost; an overwhelmed one can still slip away. So there is always a reason to pile on force (push the odds up) and always a live hope for the cornered fleet.
- **The two levers move the odds, never the direction.** This is what "cutting retreat off in real space" actually does:
  - **Strategy / encirclement** — occupy the egress hexes so defeated ships can't clear the kill-zone; they keep eating damage instead, driving P(wreck) toward — but never to — 1.
  - **Tactics / engines** — speed sets exposure time. A fast defeated ship clears the danger zone before damage piles up (lower P(wreck)); a faster pursuer holds it in weapons range longer (higher P(wreck)).
- This is the signature Hyades drama: a deterministic slaughter whose *body count* you shape but never fully dictate.

## 5. The counter-graph ("wolverine")

> **Answered (R-O48, `Hyades_standing_layer_and_observation.md` §8): both,
> sequentially.** The counter-graph is **per-player state with a total-order
> initial condition** — a strict ladder at turn 0, ordered by hull size and
> armament — and **cards are the only source of intransitivity**. So it is a DAG
> early and carries cycles by the mid-game, player-authored rather than designed
> in. Three consequences: it is per-*matchup* rather than global, which is what
> makes close-scan investment pay; the early game is pure economy because there
> is no counter-play yet, recapitulating the *Stars!* arc; and hull-size ordering
> is fine as a default precisely because it is the substrate to be disrupted.

The weapon / armor / engine / weapon-tech profiles form a **directed counter-graph**. A cheap hull running the exact counter shreds an expensive generalist — *when the spatial setup is right* (it has the range, the flank, the formation). Counters are positional, not merely numeric, which is why the theater has to be legible.

**It is not assumed acyclic.** The original note already hedged — "directed (acyclic?) graphs" — and the hedge stands: deliberate cycles may be the right way to stop any one build from dominating. Whether the finished graph is a DAG or carries intransitive loops is an **open question to settle by experiment**, not an assumption.

**Archetypes — a starting draft, informed by *Stars!* but not copied.** *Stars!* runs short/long beams, torpedoes, and missiles on a discrete battle board with tick-based initiative and per-fleet tactic selection. **Hyades keeps none of that board:** movement is continuous, there is no initiative (§4), and the player sets policy through the command layer instead of micro-ing tactics. So Hyades reinterprets the weapon *contrasts* in its own continuous, relativistic space:

| Archetype | Strength | Built-in weakness | Hyades' own levers |
|---|---|---|---|
| **Pulse** (short-range energy) | highest sustained output; **beats beam** | must close fast → light shields, thin armor (glass cannon) | high **acceleration** / tight **turn radius** to close the gap |
| **Beam** (long-range energy) | ranged, never has to close | less effective overall; **loses to pulse** up close | range to kite the brawler |
| **Torpedo** (alpha) | highest alpha strike; superb **capital-killer** | worst accuracy; overkill does **not** carry to nearby targets → bleeds value vs. **swarms** | — |
| **Missile** (guided) | ranged generalist | range limited by **fuel before exhaustion**; less effective overall | tech for **longer range before fuel-out** |
| **Exotic** (apex) | "sci-fi" dark-energy / quantum-foam / black-hole / pulsar weapons | undefined; likely Platinum/super-gated, possibly graph-breaking | where cycles and surprises are allowed to live |

Underneath, the hull stats that differentiate everything: **acceleration, turning radius, and armor-vs-shields** (heavy armor is slow; light shields are fast), with **pulse-beats-beam** as a seed counter.

**Tie to the minerals.** The graph hangs off the CMY basics and RGB supers: each archetype and upgrade carries a **mineral affinity**, so *local mineral density (the 2-D gaussian) makes some builds natural and cheap where you settle* — and you can **spend extra or substitute minerals to patch a build's built-in weakness** (bolt shields onto a brawler, extend missile fuel), within your resource limits. Geography proposes a build; resources let you bend it.

> **Open questions / proposed experiments** (flagged where we lack data):
> - **Acyclicity:** round-robin the archetype builds in sim, measure win-rates, look for intransitive loops — then decide whether to *engineer* cycles or remove them.
> - **Torpedo-vs-swarm crossover:** sweep swarm size against torpedo accuracy + the no-overkill-splash rule; find where swarms start beating torpedoes.
> - **Brawler close-rate under SR:** vary acceleration / turn radius against beam range — can pulse reliably close, or do snipers kite forever?
> - **Mineral → build mapping:** vary the gaussian; confirm local density creates regional metas without hard-locking strategy.
> - **Exotic tier:** defer until the core loop is stable, then test whether apex weapons are fun or runaway.

## 6. The bridge — the spatial-verb vocabulary

Every card ultimately compiles into one or more of these visible behaviors. Author new mechanics in this vocabulary so they always have a theater expression:

**Footprints are hex templates, never a continuous "radius."** An effect lands on a fixed hex shape: a **single hex** (the footprint for an early-game or especially powerful effect), **all hexes adjacent to a target** (radius-1), a **designated trio**, or **radius-2** (every hex adjacent to the hexes adjacent to the target) — and some effects are simply **global**. Letting a player paint *arbitrary multiple hexes* is possible, but it is exactly the micro the design ethos rejects (see §0a), so it stays a rare exception, not a tool.

| Verb | What the theater shows |
|---|---|
| **Heading / vector** | where a fleet drives; sets flanks and retreat lanes |
| **Velocity / engines** | arrival order, pursuit, escape — *relativistic, bounded by c* (the *tactics* half of retreat-cut) |
| **Engagement range / weapon profile** | kiting, focus-fire, burst vs. splash (anti-swarm) |
| **Formation** | line / screen / wedge; who eats hits, how splash lands |
| **Encirclement** | occupy egress hexes so defeated ships can't clear the kill-zone, driving their wreck odds up (the *strategy* half) |
| **Fortification** | *card- or structure-granted* resistance to dislodging — never a free consequence of a locked colony |
| **Interdiction** | freeze a hex (or AoE template): no movement in or out |
| **Build footprint / dock mobility** | where you can build — planetside, per hex; the mobile dock is the exception |
| **Gate endpoints / projection** | force appearing where it "can't" be |
| **Topology edit** | sever / collapse a system so movement must route around it |
| **Consumption / salvage** | devour or reclaim wrecks across a hex template |

**Design test:** if a proposed mechanic can't be written as a card that injects one of these verbs, it is an abstraction that won't render — and it does not belong in Hyades.

---

*Reference models for tuning the sim: special relativity for motion (a hard *c* cap and relativistic velocity addition); logistic / carrying-capacity growth for pop; **Liebig's law of the minimum** for `K = min(habitability, biosphere, infrastructure)`; the *Stars!* gravity/radiation/temperature scheme as the ancestor of the habitability triplet; Lanchester's laws for deterministic attrition; a directed counter-graph for builds — acyclicity is an open question, with intransitive ("nontransitive") balance explicitly on the table ([intransitivity](https://en.wikipedia.org/wiki/Intransitivity)); a 2-D gaussian for mineral distribution; the Penrose process as the flavor anchor for rift-energy extraction; a logistic/sigmoid curve in the open interval (0, 1) for P(wreck) vs. incoming damage — the same logistic form used for pop growth, which keeps the game's curved quantities mathematically consistent; and, as a flagged future experiment, general relativity for orbital-vs-deep-space combat.*
