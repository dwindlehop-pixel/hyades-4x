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

**Since T-111 there are two callers and the direction is unchanged.**
`sim::Simulation::sys_engagement` resolves through the same function the arena
does, so the production game and the balancer still fight with one model. The
dependency is `sim → combat`, **never `sim → arena`**: the arena's whole job is
spawning outside production, and these ships were paid for. The station-keeping
constants the sweep was tuned at moved from `arena` to `combat` for the same
reason — a tuned number with two copies is an edit waiting to go wrong — and
`arena::ROU_STATION_*` are re-exports so the sweep's own names still resolve.

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

**3.4 ~~`OPEN, and it blocks the rest of the tree`~~ `RATIFIED` — the engine has
an engagement site, and two thirds of this entry was wrong (T-111).**

The original text said: *"`combat::resolve_engagement` is a pure tactical
resolver that takes two fully-specified fleets. There is no round or command
layer for a 'do I take this fight' decision to live in."* Both halves of that
have since stopped being true, and for different reasons:

- **The round layer landed.** `EventKind::RoundBoundary` is a scheduled,
  self-chaining event and `Simulation::apply_orders` is the single inbound
  channel design law #15 requires (T-30, partially done — what is still missing
  is *hidden simultaneity*, which is T-42 and is a different problem). This
  entry was still citing T-30 as though nothing had shipped.
- **The premise that nothing meets was never checked, and it is false.**
  Colonisation is exclusive (R-V3) so colony worlds cannot host two owners, but
  **mining is non-exclusive** (roles §4.3) — and measured on the standard bed
  (`examples/contact_census`, 3 seats, 1,500 yr) **68–75% of all occupied sites
  end up worked by more than one empire**, at ~4,200 contacts per run, up to all
  three seats on one rock. The engine has been producing thousands of contacts a
  run since outposts existed, with everyone politely ignoring each other.

**So what was missing was the call, not the occasion.** `sys_engagement` — the
name `combat.rs`'s own doc comment has used since the combat refactor — now
exists: a miner parking at a rock another empire is working raises
`EventKind::Engagement`, and that resolves through the same
`combat::resolve_engagement` the arena calls. §7 is what it does; §8.5 is what
it still does not.

**What this does and does not unblock.** `belief.rs` is wired, but only at its
*degenerate* end (§5.5 below). R-AC13's "if pressed" trigger at the colonisation
layer and every posture card are **still open** — both need a decision taken at
*range*, and this one is taken at zero range where there is nothing to decide.

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

**5.5 `RATIFIED, wired at one end only` — the rule.** The accept/decline decision
reads a `BeliefAMax` and **never** the other side's true `Combatant::max_accel`.

**§7's engagement site honours this vacuously, and that is worth stating plainly
rather than counting as progress.** A shared mining rock puts the two fleets at
the *same place*, so the range is zero, so the light-lag is zero, and the
observation available at the moment of decision is current — belief equals truth
and `was_surprised` is identically false. §5.4 already exempts close range for
exactly this reason, so this is the degenerate case of the rule and not a hole in
it. **The interesting half is half wired since T-112**: a coloniser diverting from a
picket *is* a decision at range on a light-lagged warning, and whether the
warning beats the ship is exactly the counterplay window. What is still missing
is **belief**: the warning is a fact about the world, not an estimate that can be
wrong, so masking and the one-sided error still need T-33's observation store.
Measured: `committed` runs 49.4–51.6% across eight seeds, which is the coin flip
two identical hulls should produce and carries no information about either side.

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
(`Hyades_politics_trade_and_intelligence.md` §8.2).

**6.3 `RATIFIED` — Warfare's counter into Politics is Interdict**, and Politics'
counter into Warfare is the supply-chain effect: **mobilising against your
supplier cuts the supply the mobilisation is made of.** Neither is a modifier;
both are consequences of state the simulation already tracks, and both are legible
from outside — one as freighter traffic, the other as an acceleration
distribution (§8 of the politics spec).

**6.4 `OPEN, advanced` — R-WAR2: the card surface.** No Warfare card's *effect*
is authorable until §3.4 and §3.5 land, because the effect is either a *posture*
(needs an accept/decline site) or a *stat* (needs the four resolver layers). What
**is** specifiable now is the shape — empire-scale, legible, placing an object
rather than applying a percentage — and **§8 is the first card specified to it**:
the armed coloniser, which places a standing fleet on the board by declining to
consume the hull that founds a colony. Its Design half is blocked on T-97 and its
Doctrine half on §3.4; its *value model* and one refuted design claim are settled.

**6.5 `OPEN` — R-WAR3: `w_ij`, the neighbour weight.** Fixed at game start and
summing to 1, but by what — distance, shared frontier, archetype complementarity?
The objective is not measurable until this is chosen, and the choice decides
whether Warfare is a *positional* tree or a *targeting* one.

**6.6 `OPEN` — R-AC13: the "if pressed" trigger at the colonisation layer** — e.g.
a coloniser re-routing away from a detected threat. Blocked with T-30.

---

## 7. The engagement site — combat in the simulation loop (T-111)

**`RATIFIED` as a seam; every magnitude in it is a placeholder.** What is settled
is that the simulation resolves fights through `combat::resolve_engagement` and
that a destroyed hull's mass is conserved. What is not settled is any number.

### 7.1 Where a fight happens, and why there was already one to have

**A shared mining outpost.** Mining is non-exclusive (roles §4.3) and
colonisation is not (R-V3), so a rock is the one site in the shipped engine where
two empires' hulls legitimately stand together. `sys_mining_arrive` raises
`EventKind::Engagement` when a miner parks somewhere another empire is already
working — the arrival *is* the moment the situation changed, which is §4's rule,
and `mine_crew` is already keyed `(seat, outpost)` so the check is `O(seats)`
with no scan.

**The occasion was never the scarce thing** (`examples/contact_census`, 3 seats,
1,500 yr): 68–75% of occupied sites end up shared, ~4,200 contacts per run, up
to three seats on one rock. §3.4 assumed the opposite for as long as it stood.

### 7.2 Who starts it

`Doctrine::engage_neutrals`, default **false** — R-O27/T-11's first diplomatic
field, added because §8.2's card needs exactly it and the rest of the list is
still unspecified. Peace is the design's own default, not a safety catch: §8.2
specifies the first Warfare card as writing *"a Neutral empire is an Enemy
empire"*, which means nothing unless neutrality is where everyone starts.

**Hostility is asymmetric and is not negotiated.** One side's doctrine starts it;
the other is in a fight whether or not it wanted one. What it can do is outrun
them, which is §5's kinematic criterion and not consent.

A second gate, `SimConfig::engagements_enabled`, also defaults off. It is not a
design statement — it is what keeps every coverage, colony-year and work-year
figure this project has measured valid, since all of them were taken on a galaxy
where nothing fought. `combat_off_is_bit_identical` pins it.

### 7.3 What it costs the economy — measured, and it is almost nothing

Eight seeds, 3 seats, 800 yr (`examples/engagement_census`), war against peace:

| | mean | seeds positive |
|---|---|---|
| colony count | **+0.43% ± 0.17** | 5/8 |
| colony-years | **−0.74% ± 0.26** | 3/8 |

Per run: **3,701–4,131 engagements** and **5,488–12,819 hulls destroyed**, for
110–258 kt of slag. Neither aggregate is a finding — `CLAUDE.md` §2 puts the
2-SE bar as a floor for *considering* a number, and a 5/8 or 3/8 sign test is
noise — but the **magnitude** is the point, and it is a corroboration rather than
a surprise:

> **An empire can lose its entire mining fleet several times over and its
> expansion does not notice.** That is the same conclusion §6.19c and R-O92
> reached from the economic side — the binding constraint is `k_high` and the
> colour composition of freight, not hull count — arrived at by destroying the
> hulls instead of by counting them.

**Throughput rose on all eight seeds** (109→126 … 124→137 yr/s) with `ns/event`
*falling* (20,556→18,060). `CLAUDE.md` §2's table reads that pair as "a real
optimisation", and it is not one: nothing got faster per unit of work, there is
simply less work because 11,345 hulls stopped existing. That table assumes a
fixed workload, and this is the row it does not cover.

### 7.4 What it does not do, stated so it is not mistaken for more

- **It is not balanced and cannot be.** Warfare's objective is an algebraic zero
  on the symmetric 3-seat bed (R-TREE8), so nothing here was tuned against a
  galaxy and no magnitude in `CombatConfig` has seen one.
- **The belief layer is wired at its degenerate end only** (§5.5): zero range
  means belief equals truth, and `committed` measures 49.4–51.6% — a coin flip
  between two identical hulls.
- ~~**Only miners ever fight.**~~ **Amended by T-112**: a coloniser arriving at
  a world a rival is *holding* fights too, and the decision to press on or turn
  back is taken at **range**, against a warning that travelled at `c` — the
  thing §5.5 records as the unwired half. Freighters in transit are still
  untouchable, so commerce raiding (§4.4) and blockade remain out of reach.
  What T-112 also establishes is that the denial those fights serve **does not
  work as specified** (§8.6).
- **Nothing is captured and nothing is razed.** §4.1 says infrastructure is the
  war target; this reaches no infrastructure at all.
- **R-WAR5**: the defender takes the laser side and the arriver the missiles,
  and `carrier_accel` reads the laser side's first hull. A convention that
  decides outcomes.

### 7.5 Two arena assumptions the wiring exposed

Worth recording because both were invisible while `resolve_engagement` had one
caller, and both are the shape `CLAUDE.md` §2 warns about — a constant whose
stated reason stopped holding when something else changed.

- **An empty side panicked.** `laser_ships[0]` is safe in a scenario that always
  seeds both fleets; the simulation reaches "nobody is left on that side"
  legitimately, and indexing there is a crash on a legal board state. It now
  returns the walkover.
- **`carrier_accel` assumed one hull per trial.** True of the sweep, false of two
  empires bringing what they built. Carried as R-WAR5 rather than fixed, because
  changing it moves every golden in `tests/balance.rs`.

---

## 8. The first Warfare card — the armed coloniser (R-WAR2, advanced)

*The first concrete answer to **R-WAR2** (§6.4), which said nothing was
authorable until §3.4 and §3.5 land. That still holds for the card's *effects* —
both halves are inert in the engine today — and it does not hold for its
**shape**, its **value model**, or the measurement that refutes part of its
stated intent. This section settles those three and says what is blocked.*

*The contract half — that only Warfare may carry a population-lethal write — is
`Hyades_card_contract.md` §10.*

**Nothing in this section is Monte-Carlo ratified.** Every magnitude is a
placeholder and says so. What is settled is the card's shape, the measurement
that refutes one third of its stated intent, and the bed the head-to-head needs.

### 8.1 Warfare's licence, and why it is currently worth nothing

**Only Warfare cards may carry Doctrine that kills population** (card contract
§10). That is Warfare's distinguishing capability and the reason five other
trees cannot reach §2.3.2's objective by economic means.

**As the engine stands the licence confers nothing measurable, and the reason is
one line long.** `Sim::extraction_rate` and `Sim::berth_rate` read
`Factors::infra` and the owner's `Works` and nothing else
(`Hyades_industry.md` §1.8). Population is not an argument to any output
function, so a world emptied of people mines and fabricates exactly as fast as a
full one. A population kill costs its target **nothing per year**.

So the first Warfare card is deliberately **not** a population strike. It does
not need the licence, it needs the licence to *exist* before a later card can
use it — and it establishes the tree's board presence while T-107 makes
population a factor of production.

### 8.2 The card — one Design write, three Doctrine writes

**Author's specification.** The colony ship moves off the Systems family and
onto the Contact family: armed, worse freight per kilotonne of price, and —
crucially — **it is not consumed when it founds**.

| half | write | engine surface | status |
|---|---|---|---|
| **Design** | unlock a Contact coloniser class: `(GeneralContactVehicle, Class)` | `CardEffect::UnlockDesign`, and a per-`Class` drive fraction | `UnlockDesign` **exists**; per-`Class` `φ` is **T-97**, blocked on T-08 |
| **Doctrine** | unladen Contact Vehicles engage nearby enemies | an engagement-posture field on `Doctrine` | **absent** — no such field, and no combat in the sim loop (T-52) |
| **Doctrine** | a Neutral empire is an Enemy empire | a diplomatic stance default on `Doctrine` | **absent** — R-O27/T-11, the field list has never been specified |
| **Doctrine** | a Contact coloniser patrols instead of scrapping itself | `Doctrine::picket_after_founding`, `Role::Picket`, `EventKind::ColonyDivert` | **built (T-112)**, and **measured to do the opposite of what it is for** — §8.6 |

**The two halves are one card, and that is a design position.** §5 of
`Hyades_standing_layer_and_observation.md` makes Design permanent and
earlier-is-better while Doctrine is revisable and best played informed — opposite
timing profiles (R-O37). A card carrying both is therefore *not* priced as the
sum of its halves: the Doctrine half wants to be played late and the Design half
early, so the card's value distribution is the narrower of the two, not the
wider. **That is a prediction §4's method can check**, and it is the reason to
ship the first Warfare card as a pair rather than as two cards.

**The third Doctrine write is the expensive one and it is the point.** Roles
§4.2 has a coloniser recycle into the new colony's `Band I` infrastructure, and
T-70 makes that exact: `founding_infra = hull_cost`, because a hull's mass *is*
its cost (R-O57, design law #11). A coloniser that patrols instead of scrapping
keeps its minerals in the hull, so **the colony it founds starts with only what
the hold carried as endowment** (`Hyades_industry.md` §1.7). Mass is conserved
either way — the trade is *where the minerals stand*, not whether they exist.

### 8.3 One third of the stated intent is impossible, and the engine says so exactly

The design intent was three properties at once, against the Medium Systems hull
the coloniser rides today: **less cargo per kilotonne of cost, lower laden
acceleration, higher dry acceleration.**

**Two of the three already hold as shipped, and the third is backwards.**
Evaluated at the shipped configuration — these are exact values of deterministic
functions, not estimates and not bounds:

| | MSV (today) | GCV (as shipped) | GCV ÷ MSV |
|---|---|---|---|
| cost = dry mass | 0.1092 kt | 1.0995 kt | 10.07× |
| cargo capacity | 0.9118 kt | 9.8549 kt | — |
| **cargo per kt of cost** | **8.349** | **8.963** | **+7.4%** ✗ *(wanted lower)* |
| **dry acceleration** | **2.3694 g** | **2.4767 g** | **+4.5%** ✓ |
| **laden acceleration** | **0.2534 g** | **0.2486 g** | **−1.9%** ✓ |
| founding seed capacity | 1.000 kt | 12.036 kt | 12.0× |

**And the third cannot be fixed, because the three quantities are two.** Since
T-96 thrust is a property of the mounted drive and does not vary with the load,
and since R-O57 a hull's cost *is* its dry mass. So for any hull, at any
configuration:

```text
a_dry / a_laden  =  (T / M_dry) / (T / (M_dry + C))  =  1 + C / M_dry
```

and `C / M_dry` is cargo capacity per kilotonne of price. **The empty-to-laden
acceleration swing and the hull's cargo efficiency are the same number.**
Asserted to a residual below `1e-12` across five hull types and a ten-fold span
of `drive_volume_fraction` by
`sim::tests::the_acceleration_swing_is_the_cargo_efficiency` (R-O95).

Read as a constraint: cutting cargo efficiency *narrows* the swing, so it raises
laden acceleration relative to dry. A hull cannot be simultaneously worse at
freight, faster empty and slower laden — the triple is over-determined by one
constraint. Measured on the one Design write that could move it, a per-`Class`
drive fraction (T-97):

| `φ` for the Contact coloniser class | cost | cargo/cost | dry `a` | laden `a` |
|---|---|---|---|---|
| **0.01** (shipped, global) | 1.0995 kt | 8.963 | 2.4767 g | 0.2486 g |
| **0.02** | 1.1991 kt | **8.136** | **3.7828 g** | **0.4141 g** |
| 0.05 | 1.4977 kt | 6.314 | 6.6595 g | 0.9105 g |
| 0.10 | 1.9954 kt | 4.490 | 9.5405 g | 1.7379 g |

`φ = 0.02` is the smallest Contact-class drive that makes the cargo-inefficiency
claim true against the MSV, and it raises laden acceleration by **+63%** rather
than lowering it.

**Decision: drop "lower laden acceleration"; take the narrowed signature
instead.** The design was reaching for *the armed coloniser is worse freight*,
and the identity says the compensation for that is not a slower voyage — it is
**concealment**, which is a better fit for the tree that gets it. §9.2 of
`Hyades_standing_layer_and_observation.md` reads the empty-to-laden swing as the
load-state broadcast: a large hull announces whether it is laden and a small one
does not. The identity says the *width* of that broadcast is exactly the hull's
cargo efficiency, so a Contact coloniser at `φ = 0.02` swings 9.14× where the
MSV swings 9.35× and the GSV swings 24.7×. **A warship built out of a freighter's
budget is quieter than the freighter**, and that is a Warfare property obtained
without a single combat constant.

This **extends** design law #10 rather than contradicting it. The law says
*arming a fleet is loud unless you also buy thrust* — a claim about the **level**
of observed acceleration, which rises with a bigger drive. The identity is about
the **spread** between a hull's two load states, which falls as the hold does.
Two different readings of the same observable; neither is the other, and a card
that moves both moves them in opposite directions. **That is the concealment
combo T-97 predicts, arrived at from the other side.**

### 8.4 What it is evaluated against, and why trees §4 does not cover it

**Author's specification: evaluate against a Growth card that raises the growth
rate** — `TIER0` slots 3 and 4, `WriteDoctrine(GrowthRate(1.15 | 1.35))`.

`Hyades_trees_and_card_value.md` §2.4's numeraire makes that comparison well-posed in principle: card value is
the fractional reduction in the doubling time of **its own tree's** stock, which
is dimensionless and therefore comparable across trees. The Growth card is read
on work-years (§2.3.3), the Warfare card on §2.3.2's neighbour-weighted colony
contrast, and `Hyades_trees_and_card_value.md` §4.3 asks whether two tier-1 cards land at
similar P92.

**Three things break that, and each has a concrete answer.**

**1. Warfare's stock has no doubling time.** `W_i = ∫ [C_i − Σ_j w_ij C_j] dt`
is a difference that may be zero or negative, and `t₂ = ln 2 / g` requires a
positive compounding stock. R-TREE9 already flags that a tree which can go
negative has no log; the first Warfare card is where it stops being hypothetical.

> **Decision (R-TREE12): cost a Warfare card as the fractional *increase* it
> causes in the **target's** doubling time**, measured on the target's own tree
> stock, against a CRN counterfactual in which the card was not played. That is
> the same dimensionless quantity §2.4 defines — a fractional change in a
> doubling time — with the sign and the subject reversed, which is exactly what
> §0 says Warfare is for. It keeps trees §4.3's tier-equality claim well-posed and
> needs no new numeraire.

**2. The standard bed cannot read Warfare at all.** R-TREE8: at 3 seats the
homeworlds are equilateral (pairwise 77.942 ly, spread 0.000% on every seed), so
`w_ij` is doubly stochastic and `Σ_i W_i` is an **algebraic** zero — not a small
noisy number, and no count of seeds or years touches it. The head-to-head
therefore runs on the **asymmetric bed R-TREE8 calls for**: one seat carrying the
card under test, every other seat at the default, paired across seeds by CRN.
Under R-TREE12's per-target reading the asymmetric bed is not an extra
requirement — it is the only bed on which either card's value is defined.

**3. The Growth card's own ratified numbers do not transfer.** `growth_rate` was
measured on **colony count** — Expansion's objective, not Growth's — and
`CLAUDE.md` §2 records that the 2,000-year screen overstated it by ~3.7× against
the 4,000-year objective while preserving the ranking. Worse, T-94/T-95 consumed
the surface outright: the closed-form logistic made the answer independent of the
tick, and R-O84's plateau map was built by counting 50-year cycles to a Band
edge. **Re-measure the Growth card on work-years at the 3-kyr bed (trees §3.1); do not
carry a colony-count figure into this comparison.**

**What the comparison should show, stated in advance so the measurement can
refuse it.** The Growth card compounds and the Warfare card does not: a growth
multiplier acts on a rate forever, while the armed coloniser's value is an
inventory of hulls that would otherwise have been scrapped. So the prediction is
**Growth's P92 above Warfare's at earliest legal play, and Warfare's dispersion
higher** — and §4.3 requires tier-1 dispersion to be the *lowest* of any tier,
so if that holds, the first Warfare card is not a tier-1 card and the pair does
not belong at the same depth. That is a falsifiable claim about where the card
sits in the tree, and it is the first thing the head-to-head should be asked.

### 8.5 What this is blocked on, in order

Stated as a chain because none of it is parallel:

| # | blocker | code | why it binds |
|---|---|---|---|
| 1 | ~~no accept/decline site, and no combat in the simulation loop~~ — **half cleared (T-111)** | §7 | `resolve_engagement` *is* called from `sim.rs` now, and hostility is a Doctrine write, so the card's second and third writes have somewhere to land. What is still missing is the **posture**: an engagement fires when a miner parks on a shared rock, not when an unladen hull *chooses* to close on a contact it can see. *"Engage nearby enemies"* needs a detection query at range, which is R-AC13's shape and T-33's data |
| 2 | ~~no diplomatic fields on `Doctrine`~~ — **cleared for this card (T-111)** | R-O27 / T-11 | `Doctrine::engage_neutrals` is the stance *"a Neutral empire is an Enemy empire"* writes. The **rest** of R-O27's field list is still unspecified, so T-11 stays open; this card no longer waits on it |
| 3 | `φ` is global, not per-`Class` | T-97, blocked on T-08 | §8.3's Design write cannot be expressed; R-O47b forbids it reaching hulls already in the field |
| 4 | Warfare's objective is unreadable on the standard bed, and `w_ij` is unset | R-TREE8, **R-WAR3** | §8.4 |
| 5 | population is not a factor of production | T-107 | §8.1 — the licence the tree is being given costs its victims nothing |

Items 1–3 are engine work with no open design question left in them. **Two of
the five moved at T-111 and the chain is shorter than it was**: the card's
Doctrine half now has a field to write and a resolver to reach, and what remains
of item 1 is a *posture* — closing on a contact you can see — rather than the
whole of combat. Items 4 and 5 are design work, specified here and in
`Hyades_industry.md` §1.8. **The card is authorable now and measurable at none of
these steps but the last**, which is why it is written down rather than built.

---

---

### 8.6 Denial measured — the card does the opposite of what it is for (T-112)

**`OPEN`, and the direction is refuted rather than untuned.** The mechanic is
built, gated off, and measured on the asymmetric bed R-TREE8 asks for: seat 0
plays the card, seats 1–2 are at the default doctrine.

#### What was built

A coloniser under `Doctrine::picket_after_founding` is **not consumed when it
founds**. It flies on to the nearest unclaimed world it has scanned and holds
it; nobody else founds there while it does. A rival coloniser already in flight
is warned at `established_at + distance(world, its home centre)` — card contract
§2's rule verbatim, *"a reaction to a detected fleet is scheduled by the distance
to the responder"* — and turns back if the warning beats it there. If it does
not, it arrives into a fight it did not choose, and founds only if it wins.

That is the author's specification implemented as stated, and it is the first
thing in the engine that takes a decision at **range** on a light-lagged edge.

#### What it does, on eight seeds at 800 yr (`examples/denial_census`)

| | mean | seeds |
|---|---|---|
| **neighbour colonies** | **+4.40% ± 1.09** (4.0 SE) | 1/8 negative |
| own colonies | −9.55% ± 2.61 (3.7 SE) | 7/8 negative |
| `W_0 = C_0 − mean(C_j)` | **−133.2 colonies ± 35.3** (3.8 SE) | 1/8 positive |

**The neighbours expand *more*.** The stated goal was to curb their growth; the
card accelerates it, at 4.0 SE with seven of eight seeds agreeing. Warfare's own
objective moves the wrong way by 133 colonies.

#### The mechanism, and it is placement rather than magnitude

**Pickets are fielded and almost never used.** 493–793 pickets per run produce
**3–12 diversions** — a utilisation of one to two percent. A picket is placed on
the world nearest the colony that just founded, which is the *picketing empire's
own* frontier: it squats on ground its own colonisers were about to take and
threatens nobody.

**Aiming it at the neighbour is worse, and the reason is structural.** Placing
the picket on the scanned unclaimed world nearest the closest rival homeworld
gives **24–32 pickets and zero diversions** on every seed. A world that this
empire has scanned and a rival has not yet claimed barely exists: near a rival,
everything visible is already owned. **There is no contested frontier to stand
on** — colonisation is exclusive (R-V3) and worlds go from unscanned to owned
without pausing in between.

So the two placements fail for opposite reasons, and neither is a tuning
question.

#### And a 1:1 trade would still lose, which is arithmetic rather than a measurement

Suppose placement were solved and every picket denied exactly one neighbour
colony. Seat 0 spends `k` colonisers to field `k` pickets, so `C_0` falls by `k`
and the denied neighbour's count falls by `k`. Then

```text
ΔW_0 = −k + w_0A · k = −k(1 − w_0A)
```

and `Σ_j w_0j = 1` by construction (§0), so `w_0A < 1` whenever there is more
than one neighbour and **ΔW_0 < 0**. A denial bought with a whole coloniser is a
losing trade against the tree's own objective at any table wider than two seats,
however well it is aimed.

Measured, the realised trade is far past that bound: seat 0 gave up **718
colonies** across the bed and the neighbours gained 696 — a ratio of **−0.97
denied per spent**, where even +1.00 would not have been enough.

#### What would have to change

Stated as the design question rather than left as a failure:

- **A picket must cost less than a colony.** The card spends a coloniser — the
  most expensive object in the expansion loop — on an object that produces
  nothing. A cheap dedicated hull breaks the 1:1 bound above; a recycled
  coloniser cannot.
- **Or it must deny more than one world per hull.** A blockade that covers an
  approach rather than a point, or one that persists against many arrivals, is
  the shape that clears `1 − w_0A`.
- **Or denial must happen before the wave, not behind it.** Picketing *after*
  founding places hulls where expansion has already been; the contested ground
  is ahead of it. That is a production order, not a founding side effect, and it
  is a different card.

**R-WAR6** carries the two magnitudes this left behind: the founding rung a
departing picket leaves (§8.7) and the picket's own cost.

### 8.7 The first arm was an absorbing zero, and the floor is why

Worth recording separately because the failure was *mine and not the design's*,
and because it is a shape that recurs.

The first implementation charged the card honestly: roles §4.2 makes a colony's
`Band I` stock the **recycled hull**, so a hull that leaves should leave nothing.
It debited the founding rung to zero. Measured, that took seat 0 from **769
colonies to 10** and *raised* the neighbours **+20.0%**: the empire had removed
itself from the game, and the neighbours' gain was one fewer competitor rather
than any denial at all.

**The line is `employment_rate`** (`src/sim.rs`): it returns exactly `0.0` for a
stock of zero, and `fabrication_rate` is `slips × berth_rate`, so a colony
founded at infra 0 can never mine, never build and never recover. Zero is not a
price, it is an **absorbing state**.

The founding rung is now floored at the ladder's own floor (`Band Empty`,
design law #11/T-63) — the smallest rung that is a real quantity. Three arms,
same bed, and the monotonicity is the evidence that the floor is the term that
matters:

| founding rung under the card | own colonies | neighbour colonies | `W_0` |
|---|---|---|---|
| zero | −57.2% | **+20.0%** | −688 |
| **floor (shipped)** | −9.6% | **+4.4%** | −133 |
| unchanged `Band I` | +2.6% | −0.7% | +31 |

**The neighbours' gain tracks the card-player's self-harm one for one**, across a
25-point range, which is what says the measured effect is self-harm and not
denial. Even the arm that charges the card *nothing* only moves the neighbours
−0.71% ± 0.54 — inside noise, and nowhere near "greatly decrease".

`a_picketed_founding_still_leaves_a_workable_colony` pins the floor against the
absorbing zero. **R-WAR6**: whether `Band Empty` is the right rung, or whether a
departing picket should instead found at whatever its mineral endowment buys
(§1.7), is unratified.

### 8.8 Claiming instead of denying — three arms, and all three are bounded by supply (T-113)

**`OPEN`.** §8.6 refuted denial *as arithmetic*: a denial bought with a whole
coloniser gives `ΔW_0 = −k(1 − w_0A)`, and `Σ_j w_0j = 1`, so it loses at any
table wider than two seats however well it is aimed. §8.6's own "what would have
to change" list named three exits. This section measures the first two.

The author's specification: *"Add LOU to the Doctrine and Design as a cheap
picket to claim the frontier to support the Contact Vehicles. Load colony ships
with a mix of minerals for Infra and population so they do not suffer from the
no recycling policy,"* and *"Warfare tree should colonize worlds with a
picket."*

#### What was built

| | Doctrine write | What it is for |
|---|---|---|
| **cheap picket** | `picket_reserve: usize` | pickets as a *purpose*, on `HullType::LimitedOffensive` — §8.6's "a picket must cost less than a colony" |
| **claim the target** | `picket_claims_target: bool` | a centre that has named an outward world it cannot yet pay for holds it while the bank fills |
| **erect the hold** | `founding_infra_share: f64` | part of a coloniser's mineral endowment becomes the new colony's stock instead of its bank |
| **settle held ground** | — (unconditional; self-gating) | a coloniser prefers a world this empire's pickets already hold over a better one nobody holds |

`Class::Unnamed` on `LimitedOffensive` is a Design write, so TIER0 card 15
(Warfare / Inscrutable) is now `UnlockDesign` rather than `NotYetImplemented`.

#### What they measure (`examples/denial_census`, 8 seeds, 3 seats, 800 yr)

Seat 0 plays the card; seats 1–2 at the default doctrine. `own` and
`neighbours` are percentage changes against the peace arm on the same seed
(CRN); `W_0` is in colonies. **Neighbours should fall and `W_0` should rise.**

| arm | own | neighbours | `W_0` | diverts | pickets |
|---|---|---|---|---|---|
| T-112 coloniser pickets, **+ settle held ground** | −8.30 ± 2.69 | +4.07 ± 1.09 | −120 ± 34 | 59 | 5,415 |
| + hold erects infra (`share = 0.5`) | −8.33 ± 2.56 | +4.04 ± 1.04 | −119 ± 32 | 60 | 5,411 |
| + cheap LOU pickets (`reserve = 128`) | −9.01 ± 2.62 | +4.42 ± 1.12 | −129 ± 34 | 56 | 5,402 |
| LOU pickets alone | −1.57 ± 1.21 | +0.93 ± 0.38 | −21 ± 12 | 2 | 95 |
| LOU pickets **+ claims its target** | −1.62 ± 4.47 | +1.10 ± 1.83 | −35 ± 61 | 2 | 188 |

**Nothing here reverses the direction §8.6 measured.** The last two arms are
inside 2 SE on every column, which is *not* a result and is recorded as one
only because of what the picket column says.

The `LOU pickets alone` row is −1.57 where §8.6's run of the same arm read
−1.21: that difference is the held-ground preference below, now firing.

#### Settling held ground is the one arm that moved, and it needed an engine fix first

The preference is worth **+1.25 points of own colonies and +13 on `W_0`** —
arm 1 was −9.55 / +4.40 / −133 before it (§8.6) and is −8.30 / +4.07 / −120
after. It recovers about a seventh of the card's self-harm. It does not change
the sign, and on the tree's own objective the card is still a losing trade.

**It read as exactly zero first, and that was a defect rather than a finding.**
`sim::commit_one_build` reduces the scanned pool to the per-class argmax before
the policy sees it (R-O70), which is exact for a consumer reading the argmax of
a **class**. This preference reads the argmax of a **subset** of a class, and
`max(S)` does not carry `max(S′)` for `S′ ⊂ S` — so a held world reached the
policy only when it already won its class outright, and the preference was
inert by construction. The reduction now carries six slots (per-class winner,
plus per-class winner among held ground);
`the_candidate_reduction_carries_held_ground_without_duplicating_it` and
`a_coloniser_prefers_held_ground_to_a_better_unheld_world` pin both halves.

#### The cheap picket is not measurably cheaper, because it is not built

§8.6's arithmetic says a picket costing a fraction of a colony clears the
`1 − w_0A` bound. The measurement cannot see that, because the hull is fielded
**95 times across eight seeds** — twelve per run, against 5,400 coloniser
pickets in the arms above.

**The mechanism is the branch's position, and it is one predicate.** The picket
sits in the production fallback behind survey:

```text
wants_survey = survey_frontier > 0 && candidate_count < survey_reserve
```

R-O86 measured the second term a **constant `true`** — median `candidate_count`
is 0 and its maximum over a run is 164, against a ratified `survey_reserve` of
1024. So the picket branch is reachable only once the survey frontier is
*exhausted*, which on this bed happens late and rarely. It is behind survey on
purpose: a picket ahead of survey cost its own player **24.5%** of its colonies
in the first version of this branch, because survey is what finds worlds to
picket at all.

#### Giving it its own state doubles the supply and changes nothing

`picket_claims_target` opens the one other state where a cheap hull is the best
thing a centre can do with a cycle: it has **named an outward world and cannot
pay for it yet**. Those cycles are already spent saving, and a picket standing
on the target is what makes the voyage safe to have committed to. It is gated
on the target not already being held *or claimed* — `Simulation::picket_inbound`
counts hulls already under way — so a centre lays down one picket per world
rather than one per decision for the whole voyage.

Supply goes **95 → 188 pickets**, and every objective column stays inside 2 SE.

Without the in-flight guard the same arm reads **−4.87 ± 5.65 / +1.05 ± 2.75 /
−58 ± 85 on 177 pickets**. The 3.3-point difference in own colonies is smaller
than either arm's error bar, so **the guard's value is not resolved at eight
seeds**; it is kept because laying down a hull per decision for the length of a
voyage is waste whether or not this bed can price it.

**The state is rare**: expansion on this bed is not bound by the coloniser's
price, so centres seldom sit in "named it, cannot buy it". That is consistent
with `Hyades_industry.md` §6.19c, which measures the mineral constraint as a
*colour* conjunction on the infrastructure bill rather than a shortfall against
hull prices.

**Diversions stay at 2 per eight seeds**, which is §8.6's structural finding
unchanged: there is no contested frontier to stand on, so more pickets standing
on it would not obviously help either.

#### Erecting the hold is flat because the hold has nothing in it

`founding_infra_share` measures −8.33 against −8.30: no change. §8.7 establishes
that the founding rung matters enormously — floor against an unchanged `Band I`
is a twelve-point swing — so flat here is a claim about the *quantity*, and
`examples/founding_stock` measures it directly (seed 1, 800 yr, seat 0's own
foundings, floor rung `Band Empty` = 0.020 kt):

| `founding_infra_share` | foundings | median stock | mean | max | above the floor |
|---|---|---|---|---|---|
| 0.00 | 650 | 0.020 | 0.020 | **0.020** | **0.0%** |
| 0.50 | 649 | 0.020 | 0.051 | 0.313 | 21.0% |
| 1.00 | 649 | 0.020 | 0.087 | 0.625 | 22.5% |

At share 0 the **maximum** is the floor: no coloniser under this doctrine lands
with an endowment worth anything. At 0.5 the median is still the floor and only
a fifth of foundings clear it — and **the whole hold erected (1.00) adds 1.5
points**, so the axis is exhausted rather than undertuned.

**A coloniser's hold is nearly all settlers.** `settler_target` sizes the
people to the destination's carrying capacity (R-IND12) and the minerals take
whatever volume is left, which is usually none. So *"load colony ships with a
mix of minerals for Infra and population"* is not a share of the hold — it is a
**reservation against the hold**, and that is a change to `settler_target`, not
to this write. **R-WAR7** carries it.

#### Where this leaves the card

Two of §8.6's three exits are now measured and neither works at the magnitudes
reachable today:

- **"A picket must cost less than a colony"** — built, and the cost is not what
  binds. Twelve to twenty-three hulls a run is a supply problem, and its two
  causes are both load-bearing elsewhere (survey's priority, and expansion not
  being coloniser-price-bound).
- **"Denial must happen before the wave"** — `picket_claims_target` is the
  cheapest form of it and fires in a state that is rare.

The third exit — **deny more than one world per hull** — is untouched and is
the only one whose arithmetic clears the `1 − w_0A` bound by construction rather
than by magnitude. That is a blockade over an approach, and it needs a spatial
object the engine does not have.

**What did land on its own merits** is the held-ground preference and the
reduction fix behind it: those are Expansion machinery, they help any doctrine
that holds ground for any reason, and they are the only part of this section not
waiting on a magnitude.


---

## 9. Register

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
| **R-O95** | the empty-to-laden acceleration swing **is** the hull's cargo efficiency, exactly — so a hull cannot be worse at freight, faster empty and slower laden at once (§8.3) |
| — | **only Warfare may carry a population-lethal Doctrine write** (card contract §10), enforced in the card layer rather than by authoring convention |
| — | the wreck roll is the only stochastic beat, bounded in (0, 1) |
| **T-113** | the per-class candidate reduction (R-O70) is exact only for a consumer reading the argmax of a **class**; a consumer reading the argmax of a *subset* needs its own slot, and got one (§8.8) |
| **T-112** | a colony founded at infrastructure **zero** is an absorbing state, not a price — `employment_rate` returns exactly `0.0` there, so it can never mine or build. A departing picket leaves the ladder's floor rung instead (§8.7) |
| **T-111** | `combat::resolve_engagement` is called from `sim.rs`; the simulation and the arena fight with one model, and the arena still seeds no production |
| law #11 | a destroyed hull's mass becomes **slag at the site** — inert (R-O59), conserved, and never a salvage yield |
| — | no tick-based initiative; `dt = 0.0005 yr`; < 2 ms per tick |

### Open

| Code | Question | What would settle it |
|---|---|---|
| ~~**T-30**~~ | ~~no accept/decline site exists~~ — **closed as stated (T-111).** The round layer had already landed and the engine was producing ~4,200 co-locations per run unremarked; `sys_engagement` now resolves them. R-AC13 and posture cards are **not** unblocked: both need a decision at *range* | done; see §3.4 and §7 |
| **R-WAR7** | **a coloniser's hold is nearly all settlers**, so erecting a share of it as the new colony's stock moves the median founding not at all and clears the floor rung in 21–22% of foundings at *any* share (§8.8). Loading a colony ship with a mix is a **reservation against the hold** — a change to `settler_target` (R-IND12) — not a share of what is left over | a `settler_target` that reserves mineral volume, then the same census |
| **R-WAR6** | **the denial magnitudes** — the founding rung a departing picket leaves (`Band Empty` shipped, or the mineral endowment instead, §8.7), and what a picket ought to cost. §8.6's arithmetic says a denial bought with a whole coloniser loses at any table wider than two seats, so this is a *design* question before it is a magnitude. **T-113 built the cheaper hull and it did not settle the question**: the hull is fielded 12–23 times a run because its branch sits behind a survey test R-O86 measured a constant `true`, so cost is not what binds (§8.8). The remaining exit is a denial covering more than one world, which needs a spatial object the engine does not have | a blockade over an approach rather than a point |
| **R-WAR5** | **which side carries which weapon.** `resolve_engagement` is laser-side-vs-missile-side because that is the sweep it was tuned on; the sim gives the defender the lasers and the arriver the missiles, and `carrier_accel` reads the laser side's first hull whatever the attacker flies. A convention that decides outcomes | R-L0's per-hull slot tables, then the arena |
| **T-111** | **the engagement magnitudes** — `engagement_horizon_years`, `engagement_volley_period_years`, and whether a shared rock is the right occasion for a fight at all | a bed on which Warfare's objective is readable (R-TREE8) |
| R-MC9c / T-12 | HP pools, weapon count, missile AoE, magazines | engine work, then the arena |
| R-L0 | per-hull slot tables | the arena |
| R-L2 | single closing pass or repeated passes? | pin with the wreck roll |
| R-O40 / T-09 | throttle fraction; observe `a` from the trajectory | engine work |
| R-O31 / T-05 | `min_time_search` as a reachability cone | engine work |
| R-O65 | flatten `hull_thrust_to_mass`? | explicit ratification |
| R-IND8 | captured infrastructure | a design pass, with Production |
| R-WAR1 | **elimination** — nothing eliminates anyone | a design pass |
| R-WAR2 | the card surface | **advanced** — §7 specifies the first card's shape and value model; its effects stay blocked on T-30 and R-MC9c |
| **R-WAR4** | **the armed coloniser's magnitudes** — the Contact coloniser class's drive fraction `φ` (0.02 is the smallest value that makes its cargo-inefficiency claim true, and it is a placeholder), the card's mineral cost, and where in the tree the pair sits once §8.4's dispersion prediction is measured | T-97 first, then a measurement on the asymmetric bed |
| **R-TREE12** | Warfare card value has no doubling time of its own; decided as the fractional *increase* in the **target's** | trees §5; the target set is still open |
| R-WAR3 | `w_ij`, the neighbour weight | a decision; the objective needs it |
| R-AC13 | "if pressed" at the colonisation layer | blocked on T-30 |
| — | nothing moves `bio_max`, so the durable target is untouchable | a Warfare card, and the guard in §4.2 |

---

## References

- `Hyades_simulation_model.md` §4 (deterministic combat, the wreck roll), §5
- `Hyades_loadout.md` §3.2 (the four weapon families), §3.3 (defence), §5
- `Hyades_standing_layer_and_observation.md` §3 (σ), §6.2 (acceleration as the
  observable), §6.4–6.5 (belief and SPRT), §9.2 (laden hulls are conspicuous)
- `Hyades_trees_and_card_value.md` §2.3.2 — the relative objective and `w_ij`;
  §4 — card value, the P92 contract and the doubling-time numeraire §8.4 departs from
- `Hyades_card_contract.md` §10 — the write-capability partition §8.1 rests on
- `Hyades_technology_tree.md` — what capability is bought with
- `Hyades_politics_trade_and_intelligence.md` §8 — the time-dependent counter-graph
  edge between these two trees
- `Hyades_industry.md` §1.2 (infrastructure is the war target), §1.8 (population is
  not a factor of production, which is why §8.1's licence is currently inert),
  §9 (captured infra)
- `Hyades_vehicle_roles.md` §4.2 — the Colonizer role and the arrival behaviour
  §8.2 changes
- `src/combat.rs`, `src/arena.rs`, `src/belief.rs`
- CLAUDE.md design laws #2, #3, #4, #7, #8, #10, #11
