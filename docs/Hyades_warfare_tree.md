# Hyades — Warfare: Combat, the Arena, and What Is Worth Hitting

*The design space for **Warfare**, the tree that takes colony-years off the
table. Its objective is **colony-years relative to the table, neighbour-weighted**
(`Hyades_trees_and_card_value.md` §2.3.2). Companion to
`Hyades_simulation_model.md` §4 (deterministic combat and the wreck roll),
`Hyades_loadout.md` (what a ship is fitted with),
`Hyades_standing_layer_and_observation.md` (what a fleet leaks by moving) and
`Hyades_technology_tree.md` (what capability is bought with). New calls continue
the **R-WAR n** series.*

**Rev 1, new.** Carries **ratified decisions and open decisions only**
(`CLAUDE.md` §6). The tactical resolver ships and is engine-native; the belief
layer ships and is unwired; the **strategic** half — who fights whom, when, and
what it costs the loser — does not exist. The register reflects that.

---

## 0. What this tree is, and the constraint that shapes all of it

**Warfare is the only tree whose objective is *relative*.**

```text
W_i = ∫₀^T [ C_i(t) − Σ_{j≠i} w_ij · C_j(t) ] dt
```

Every other tree grows a stock of its own. Warfare wins by growing yours **or by
shrinking theirs**, and no aggregate over all seats can express that — which is
why the single global colony count every ratification before
`Hyades_trees_and_card_value.md` used scores a successful war as a *loss*.

**Two consequences that shape the whole design:**

- **A warfare card is measured against a neighbour, not against a bed.** `w_ij`
  is fixed at game start and `Σ_j w_ij = 1`.
- **Warfare is an algebraic zero on the 3-seat measurement bed**, because nothing
  fights. Every number in this file is therefore about mechanism, not balance,
  and the tree cannot be tuned until the engine has an engagement site (§3.4).

**The competitive frame constrains the tactics.** Design pillar: a **meso + macro**
game with a deliberately low micro floor. Deterministic combat removes execution;
hidden simultaneous orders create the meso layer. **There is no in-fight
decision**, so every lever a Warfare card pulls must be pulled *before* the
shooting starts.

---

## 1. The combat/arena split

**1.1 `RATIFIED` — `combat.rs` is engine-native and is the same code both
consumers resolve fights with.** The production game and the Monte-Carlo balancer
share it. It holds the kinematic primitives (fleet trajectories, station-keeping,
Isaacs intercept, lasers with light-lag aim and point defence, dodging
fuel-limited missiles, the BSP targeting index), the tuned weapon parameters
(`CombatConfig`), and the resolution loop.

**1.2 `RATIFIED` — `arena.rs` is a scenario seeder and owns no combat logic.**
Its only job is to **spawn ships outside the constraints of Hyades production** —
no planets, no mining, no colonisation, no mineral budget — place them as two
fleets, and call `combat::resolve_engagement`. **The arena resolves no damage.**
Dependency direction is `arena → combat`, never the reverse.

**Do not reintroduce combat logic into the arena or into an example.** It was in
an example once (`laser_vs_missile`), which meant the balancer and the game could
diverge silently.

**1.3 `RATIFIED` — the Ship Testing Arena is the *required* empirical harness for
per-class `r_eq`.** Design law #4. **These values cannot be derived
analytically**, and the arena exists precisely so that hull supremacy is measured
rather than asserted.

**1.4 `RATIFIED` — everything in `CombatConfig` is a globally Monte-Carlo-tuned
parameter and may not be changed silently.** It requires explicit ratification
(the working agreement). `tests/balance.rs` holds its goldens and reproduces them
bit-for-bit across landings that moved the mass model underneath it.

---

## 2. The tactical model

**2.1 `RATIFIED` — combat is deterministic; the wreck roll is the only stochastic
beat, and it is per ship.** When a ship is defeated it tries to leave. Whether it
is **wrecked** or **gets away** is a single weighted coin-flip. Retreat
*direction* is not random — it follows the board, toward open or friendly space.
**The randomness is only whether the ship survives to use it.**

**2.2 `RATIFIED` — `P(wreck)` is bounded in the open interval (0, 1)** and rises
with incoming damage on a saturating curve. **Never 0%, never 100%.** A
barely-scratched ship can still be lost; an overwhelmed one can still slip away.
So there is always a reason to pile on force and always a live hope for the
cornered fleet. Same logistic form as population growth, which keeps the game's
curved quantities mathematically consistent.

**2.3 `RATIFIED` — there is no tick-based initiative.** No per-round turn order,
no INIT stat deciding who fires first. Each weapon's reach and closing behaviour
is a function of range and the two ships' kinematics, **resolved geometrically**.

Concretely: a torpedo's advantage is that it delivers damage *before the closing
ship reaches beam range*, decided by the distance/closing-speed geometry the
engine already computes for flight — **not by winning a die roll for first
strike.**

**2.4 `RATIFIED` — the four weapon families are a counter-graph, not a damage
ladder.** They differ in the range/kinematics regime where they are strong:

| Family | Strong when | Weak when |
|---|---|---|
| **Beam** | close, all-aspect, instant hit — range-limited hitscan, the reliable backbone | long range (falloff) |
| **Pulse** | point-blank, high burst; tiny, cheap, murderous up close | anything past knife range |
| **Torpedo** | long range, heavy hit; slow projectile, big | close (arming distance), vs. point defence |
| **Missile** | long range, tracking | vs. point defence and jammers — countered by ELEC, unlike torpedoes |

**2.5 `RATIFIED` — station-keeping is a designed periodic motion, not orbital
mechanics.** There is no central mass and nothing integrates a two-body problem.
Ships hold station around the fleet's reference trajectory on a circular path of
fixed radius and angular rate, seeded once per ship.

**The point is informational:** a distant observer resolving only the fleet's
*aggregate* trajectory — which is what every light-lag-delayed detection query in
this engine works from — **cannot predict a specific ship's instantaneous
position or velocity** without a current, close-range scan, because the phase has
evolved unobserved.

**2.6 `RATIFIED` — `dt = 0.0005 yr` (~0.18 days) is required** for missile
guidance convergence. At the old `dt = 0.006 yr`, missiles scored **zero** hits
against a dodging target. This is not a tuning preference; it is a numerical
requirement.

**2.7 `RATIFIED` — fleet synchronisation is an artifact generator.** Launch
schedules must be desynchronised or burst size becomes meaningless.

**2.8 `RATIFIED` — a per-tick budget of < 2 ms, just-in-time for 60 fps.**
Presentation time is decoupled from simulation tick duration.

**2.9 `RATIFIED` — the Lanchester aggregate model is reserved for imperial-scale
resolution.** The individual-missile arena exists only to **calibrate its
parameters**, not to resolve imperial battles.

**2.10 `OPEN` — R-L2: is an engagement a single closing pass, or repeated passes
until one side disengages?** Ties into §2.1's single wreck roll and must be pinned
with it, not separately.

**2.11 `OPEN` — `LASER_HIT_TOLERANCE` changes require dimensional analysis**
against actual hull dimensions before acceptance. It is an **abstracted
fire-control stat, not a literal hull cross-section**, and has been read as the
latter before.

---

## 3. Hull supremacy, and the strategic layer that does not exist

**3.1 `RATIFIED` — hull supremacy must be slot-organic, never
hyperparameter-tuned.** Design law #2. GOU superiority over an equal-cost ROU
fleet (and ROU over LOU) must emerge from **hull slot counts and volume**, never
from tuning a battle constant. Target scale: *Stars!* Dreadnought-vs-Cruiser,
roughly **1 GOU handling 6–45 ROUs**.

Prior numerical probing, to be re-derived in-engine: the crossover `N*` scales
~linearly with volume ratio `ρ = V_GOU / V_ROU` (`N* ≈ ρ/3.3`), is nearly
independent of the cross-section constant, and lands in the 6–45 window for
`ρ ∈ [20, 150]` — i.e. **supremacy is slot-organic by construction.**

**3.2 `RATIFIED` — consolidation wins under geometry alone, so any value for
fragmentation must come from combat.** Design law #3. Surface area is the cost
basis and volume the value basis, and the isoperimetric inequality does the rest.
**Strategic value for smaller fleets must therefore come from combat-specific
effects — Lanchester's square law, indivisibility as a liability — not from the
cost curve.**

**3.3 `RATIFIED` — LOUs are chaff in a main battle fleet and genuinely useful for
insurgency.** Design law #8: not useful force projection, but Mao-style harass,
avoid, and strike the resting or retreating enemy. **An LOU that were good in a
line battle would violate 3.2.**

**3.4 `OPEN, and it blocks the rest of the tree` — T-30: there is no accept/decline
site in the engine.** `combat::resolve_engagement` is a *pure tactical resolver*
that takes two fully-specified fleets. There is no round or command layer for a
"do I take this fight" decision to live in.

Three things wait on it: `belief.rs`'s wiring (§5), R-AC13's "if pressed" trigger
at the colonisation layer, and every Warfare card whose effect is a *posture*
rather than a stat.

**3.5 `OPEN` — R-MC9c / T-12: the four layers `resolve_engagement` still needs**,
all as `CombatConfig` fields plus slot-derived stats:

| Layer | Why |
|---|---|
| **HP pools** | a GOU must not die to a single hit |
| **Weapon count as firing units** | count scales with hull slots; **per-hit damage stays a single global constant**, which is what keeps design law #2 intact |
| **Missile AoE** | needed to handle dense LOU swarms; ships spread to avoid it |
| **Magazines** | LOUs model limited missiles, so they are better *laser* platforms than missile platforms |

**Balance-preservation constraint:** keep the AoE radius **below** the baseline
ROU formation spacing, so the AoE term is identically zero in the ROU-vs-ROU case
and the existing laser-vs-missile balance is untouched. Verify by re-running
`--example laser_vs_missile` and comparing.

---

## 4. What is worth hitting

**4.1 `RATIFIED` — infrastructure is the war target.** It is the one stock that is
valuable, visible and destructible **without killing people** — and since
`K = min(hab, bio_max)` it is no longer a term in carrying capacity, so razing it
does not move population. **That separation is what makes an industrial strike a
distinct act from a population strike**, and it is the whole reason infrastructure
left the minimum (`Hyades_industry.md` §1.2).

**4.2 `RATIFIED` — biosphere is the durable target, and it is a rate.** Biosphere
is a mass in kilotons and the one *renewable* stock, regrowing logistically toward
`bio_max`. **Ecology is a rate rather than an exemption**, which is what makes
biological damage durable and gives Warfare a target that is neither hulls nor
infrastructure (design law #11).

> **Open, and it is an engine gap rather than a design one:** nothing in the
> shipped engine moves `bio_max`, so the durable target is currently
> untouchable — and `biosphere_regen_rate` is bit-identically inert for the same
> reason. **A Warfare card that lowers a world's pristine ceiling is the first
> thing that makes either real**, and it also makes the coverage denominator
> playable again, which is the failure mode `CLAUDE.md`'s metric-invariance rule
> exists to catch.

**4.3 `RATIFIED` — population collapse is design content; the undershoot was
not.** Under the Euler logistic an over-capacity world overshot *toward zero*
rather than settling at the cut ceiling, and the severity of that was a function
of `cycle_years` — **so the outcome of an attack on a world's habitability was
being set by a performance knob.** The closed form removes it: an over-capacity
world now decays *toward* the ceiling (31.62 → 8.88 kt in one tick, a 72%
die-off). **T-94/R-O93.**

**4.4 `RATIFIED` — interdiction needs no new mechanism.** A matched contract's
cargo in flight is a laden freighter in open space, which is observable and can be
shot (`Hyades_politics_trade_and_intelligence.md` §2.6). **Blockade is not a
special rule either** — it is a fleet sitting on a route that has to be flown.

**And a laden hull broadcasts.** A General hull drops 5.06 g empty to 0.23 g
laden, a 22× swing, against a Limited hull's 1.00 → 0.70, so **large hulls
announce their load state and small ones do not** (std §9.2). That is what makes
commerce raiding a targeting problem rather than a search problem.

**4.5 `OPEN` — R-IND8: captured infrastructure.** What an occupier gets when it
takes a developed world. The join with Production, and unanswered.

**4.6 `OPEN` — R-WAR1: elimination.** The design pillar is **hard win conditions
with progressive player elimination**, and nothing in the engine eliminates
anyone. What ends a seat, what happens to its holdings, and whether `w_ij`
renormalises are all unspecified.

---

## 5. Belief — deciding on what you have seen

**5.1 `RATIFIED` — acceleration is the observable, and it is one-sided.**
Design law #10. `a = thrust / (dry_mass + cargo_mass)` is one scalar over three
latents, so the inverse problem is under-determined at range. **A ship may fly
below peak and never above it**, so an observation is a *lower bound*, and the
best estimate from a history of observations is their **maximum** — the fastest
burn a target has ever been caught making.

**5.2 `RATIFIED` — surprise attack falls out of the physics.** A target that has
been under-burning has a true reachable set **larger** than the one an observer
computes, so its actual destination can lie outside the cone. An intercept solved
against a believed `a_max` can simply *fail* against a target that was masking.
A 4,851-case sweep pins that the estimator errs **only by optimism**, which *is*
the surprise attack.

**5.3 `RATIFIED` — belief is monotone, so masking is spend-once.** A capability
once displayed cannot be un-displayed. **The first time you burn hard in view of
someone, you have told them, permanently** — the physical form of the standing
layer's asymmetric-leak rule.

**5.4 `RATIFIED` — missile terminal guidance is exempt and stays on true
kinematics.** Close range is exactly where the degeneracy breaks, so a missile a
few thousand km out is *not* working from a lower bound.

**5.5 `RATIFIED` — the rule when T-30 lands.** The accept/decline decision reads a
`BeliefAMax` and **never** the other side's true `Combatant::max_accel`.

**5.6 `OPEN` — R-O40 / T-09: throttle fraction, and observing `a` from the
trajectory rather than the stat block.** Today the engine reads the stat block,
which means a ship cannot actually choose to under-burn — so 5.1's whole
asymmetry is available to the *estimator* and not yet to the *pilot*.

**5.7 `OPEN` — R-O31 / T-05: `min_time_search` as a reachability-cone query.**
Same function, reverse direction. It is what makes tier-1 movement intelligence
(`Hyades_politics_trade_and_intelligence.md` §5.1) worth buying.

---

## 6. What a Warfare card writes

**6.1 `RATIFIED` — cards operate at empire/macro scale only.** Design law #7.
**War Sun is the gold standard**: flavourful, legible, places a behaviour-rich
board object. Combo cards are the connective tissue between trees — **if
single-tree play can win without cross-tree engagement, combo cards are
undercosted.**

**6.2 `RATIFIED` — the stance and conduct table is where posture lives.**
`Conduct::engage` feeds `belief::decide_engagement`'s `Unobserved` policy, so
"what do I assume about a fleet I cannot measure" is a per-stance Doctrine
decision rather than a constant
(`Hyades_politics_trade_and_intelligence.md` §7.2).

**6.3 `RATIFIED` — Warfare's counter into Politics is Interdict**, and Politics'
counter into Warfare is the supply-chain effect: **mobilising against your
supplier cuts the supply the mobilisation is made of.** Neither is a modifier;
both are consequences of state the simulation already tracks, and both are legible
from outside — one as freighter traffic, the other as an acceleration
distribution (§8 of the politics spec).

**6.4 `OPEN` — R-WAR2: the card surface.** Nothing is authorable until §3.4 and
§3.5 land, because a Warfare card's effect is either a *posture* (needs an
accept/decline site) or a *stat* (needs the four resolver layers). What can be
specified now is the **shape**: empire-scale, legible, and placing an object
rather than applying a percentage.

**6.5 `OPEN` — R-WAR3: `w_ij`, the neighbour weight.** Fixed at game start and
summing to 1, but by what — distance, shared frontier, archetype complementarity?
The objective is not measurable until this is chosen, and the choice decides
whether Warfare is a *positional* tree or a *targeting* one.

**6.6 `OPEN` — R-AC13: the "if pressed" trigger at the colonisation layer** — e.g.
a coloniser re-routing away from a detected threat. Blocked with T-30.

---

## 7. Register

### Ratified

| Code | Decision |
|---|---|
| — | `combat.rs` is engine-native; `arena.rs` seeds scenarios and resolves no damage |
| law #4 | the Arena is the required empirical harness for per-class `r_eq` |
| law #2 | hull supremacy is slot-organic; 1 GOU ≈ 6–45 ROUs |
| law #3 | consolidation wins under geometry; fragmentation's value must come from combat |
| law #8 | LOUs are chaff in a line, useful for insurgency |
| law #10 | acceleration is the observable; concealment is a combo property |
| R-O41 | belief is the max ever observed; masking is spend-once; error is one-sided |
| R-O93 | the population logistic is solved, so attack outcomes are tick-independent |
| R-MC16 | thrust is drawn from mounted drive; the load-state broadcast survives |
| — | the wreck roll is the only stochastic beat, bounded in (0, 1) |
| — | no tick-based initiative; `dt = 0.0005 yr`; < 2 ms per tick |

### Open

| Code | Question | What would settle it |
|---|---|---|
| **T-30** | **no accept/decline site exists** — blocks belief wiring, R-AC13, and every posture card | engine work; the round/command layer |
| R-MC9c / T-12 | HP pools, weapon count, missile AoE, magazines | engine work, then the arena |
| R-L0 | per-hull slot tables | the arena |
| R-L2 | single closing pass or repeated passes? | pin with the wreck roll |
| R-O40 / T-09 | throttle fraction; observe `a` from the trajectory | engine work |
| R-O31 / T-05 | `min_time_search` as a reachability cone | engine work |
| R-O65 | flatten `hull_thrust_to_mass`? | explicit ratification |
| R-IND8 | captured infrastructure | a design pass, with Production |
| R-WAR1 | **elimination** — nothing eliminates anyone | a design pass |
| R-WAR2 | the card surface | blocked on T-30 and R-MC9c |
| R-WAR3 | `w_ij`, the neighbour weight | a decision; the objective needs it |
| R-AC13 | "if pressed" at the colonisation layer | blocked on T-30 |
| — | nothing moves `bio_max`, so the durable target is untouchable | a Warfare card, and the guard in §4.2 |

---

## References

- `Hyades_simulation_model.md` §4 (deterministic combat, the wreck roll), §5
- `Hyades_loadout.md` §3.2 (the four weapon families), §3.3 (defence), §5
- `Hyades_standing_layer_and_observation.md` §3 (σ), §6.2 (acceleration as the
  observable), §6.4–6.5 (belief and SPRT), §9.2 (laden hulls are conspicuous)
- `Hyades_trees_and_card_value.md` §2.3.2 — the relative objective and `w_ij`
- `Hyades_technology_tree.md` — what capability is bought with
- `Hyades_politics_trade_and_intelligence.md` §8 — the time-dependent counter-graph
  edge between these two trees
- `Hyades_industry.md` §1.2 (infrastructure is the war target), §9 (captured infra)
- `src/combat.rs`, `src/arena.rs`, `src/belief.rs`
- CLAUDE.md design laws #2, #3, #4, #7, #8, #10, #11
