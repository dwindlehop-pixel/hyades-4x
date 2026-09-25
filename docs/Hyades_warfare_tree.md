# Hyades — Warfare: Combat, the Arena, and What Is Worth Hitting

*The design space for **Warfare**, the tree that takes colony-years off the
table. Its objective is **colony-years relative to the table, neighbor-weighted**
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

- **A warfare card is measured against a neighbor, not against a bed.** `w_ij`
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
Isaacs intercept, lasers with light-lag aim and point defense, dodging
fuel-limited missiles, the BSP targeting index), the tuned weapon parameters
(`CombatConfig`), and the resolution loop.

**1.2 `RATIFIED` — `arena.rs` is a scenario seeder and owns no combat logic.**
Its only job is to **spawn ships outside the constraints of Hyades production** —
no planets, no mining, no colonization, no mineral budget — place them as two
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
no INIT stat deciding who fires first. Each weapon's reach and closing behavior
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
| **Torpedo** | long range, heavy hit; slow projectile, big | close (arming distance), vs. point defense |
| **Missile** | long range, tracking | vs. point defense and jammers — countered by ELEC, unlike torpedoes |

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
  Colonization is exclusive (R-V3) so colony worlds cannot host two owners, but
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
*degenerate* end (§5.5 below). R-AC13's "if pressed" trigger at the colonization
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
renormalizes are all unspecified.

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

**§7's engagement site honors this vacuously, and that is worth stating plainly
rather than counting as progress.** A shared mining rock puts the two fleets at
the *same place*, so the range is zero, so the light-lag is zero, and the
observation available at the moment of decision is current — belief equals truth
and `was_surprised` is identically false. §5.4 already exempts close range for
exactly this reason, so this is the degenerate case of the rule and not a hole in
it. **The interesting half is half wired since T-112**: a colonizer diverting from a
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
**War Sun is the gold standard**: flavourful, legible, places a behavior-rich
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
the armed colonizer, which places a standing fleet on the board by declining to
consume the hull that founds a colony. Its Design half is blocked on T-97 and its
Doctrine half on §3.4; its *value model* and one refuted design claim are settled.

**6.5 `OPEN` — R-WAR3: `w_ij`, the neighbor weight.** Fixed at game start and
summing to 1, but by what — distance, shared frontier, archetype complementarity?
The objective is not measurable until this is chosen, and the choice decides
whether Warfare is a *positional* tree or a *targeting* one.

**6.6 `OPEN` — R-AC13: the "if pressed" trigger at the colonization layer** — e.g.
a colonizer re-routing away from a detected threat. Blocked with T-30.

---

## 7. The engagement site — combat in the simulation loop (T-111)

> **Superseded in direction by §8.19 (author's ruling, T-133): there are no
> engagements and no fight sites.** This section describes what the engine
> still does until T-133 lands, and stays as that record.

**`RATIFIED` as a seam; every magnitude in it is a placeholder.** What is settled
is that the simulation resolves fights through `combat::resolve_engagement` and
that a destroyed hull's mass is conserved. What is not settled is any number.

### 7.1 Where a fight happens, and why there was already one to have

**A shared mining outpost.** Mining is non-exclusive (roles §4.3) and
colonization is not (R-V3), so a rock is the one site in the shipped engine where
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
> color composition of freight, not hull count — arrived at by destroying the
> hulls instead of by counting them.

**Throughput rose on all eight seeds** (109→126 … 124→137 yr/s) with `ns/event`
*falling* (20,556→18,060). `CLAUDE.md` §2's table reads that pair as "a real
optimization", and it is not one: nothing got faster per unit of work, there is
simply less work because 11,345 hulls stopped existing. That table assumes a
fixed workload, and this is the row it does not cover.

### 7.4 What it does not do, stated so it is not mistaken for more

- **It is not balanced and cannot be.** Warfare's objective is an algebraic zero
  on the symmetric 3-seat bed (R-TREE8), so nothing here was tuned against a
  galaxy and no magnitude in `CombatConfig` has seen one.
- **The belief layer is wired at its degenerate end only** (§5.5): zero range
  means belief equals truth, and `committed` measures 49.4–51.6% — a coin flip
  between two identical hulls.
- ~~**Only miners ever fight.**~~ **Amended by T-112**: a colonizer arriving at
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

## 8. The first Warfare card — the armed colonizer (R-WAR2, advanced)

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

### 8.1 Warfare's license, and why it is currently worth nothing

**Only Warfare cards may carry Doctrine that kills population** (card contract
§10). That is Warfare's distinguishing capability and the reason five other
trees cannot reach §2.3.2's objective by economic means.

**As the engine stands the license confers nothing measurable, and the reason is
one line long.** `Sim::extraction_rate` and `Sim::berth_rate` read
`Factors::infra` and the owner's `Works` and nothing else
(`Hyades_industry.md` §1.8). Population is not an argument to any output
function, so a world emptied of people mines and fabricates exactly as fast as a
full one. A population kill costs its target **nothing per year**.

So the first Warfare card is deliberately **not** a population strike. It does
not need the license, it needs the license to *exist* before a later card can
use it — and it establishes the tree's board presence while T-107 makes
population a factor of production.

### 8.2 The card — one Design write, three Doctrine writes

**Author's specification.** The colony ship moves off the Systems family and
onto the Contact family: armed, worse freight per kilotonne of price, and —
crucially — **it is not consumed when it founds**.

| half | write | engine surface | status |
|---|---|---|---|
| **Design** | unlock a Contact colonizer class: `(GeneralContactVehicle, Class)` | `CardEffect::UnlockDesign`, and a per-`Class` drive fraction | **shipped (T-121)** — `TIER0[15]` unlocks it, and `DoctrineWrite::ArmedFrontier` puts the colonizer ladder on it; per-`Class` `φ` is still **T-97**, blocked on T-08 |
| **Doctrine** | unladen Contact Vehicles engage nearby enemies | an engagement-posture field on `Doctrine` | **absent** — no such field, and no combat in the sim loop (T-52) |
| **Doctrine** | a Neutral empire is an Enemy empire | a diplomatic stance default on `Doctrine` | **absent** — R-O27/T-11, the field list has never been specified |
| **Doctrine** | a Contact colonizer patrols instead of scrapping itself | `Doctrine::picket_after_founding`, `Role::Picket`, `EventKind::ColonyDivert` | **built (T-112)**, and **measured to do the opposite of what it is for** — §8.6 |

**The two halves are one card, and that is a design position.** §5 of
`Hyades_standing_layer_and_observation.md` makes Design permanent and
earlier-is-better while Doctrine is revisable and best played informed — opposite
timing profiles (R-O37). A card carrying both is therefore *not* priced as the
sum of its halves: the Doctrine half wants to be played late and the Design half
early, so the card's value distribution is the narrower of the two, not the
wider. **That is a prediction §4's method can check**, and it is the reason to
ship the first Warfare card as a pair rather than as two cards.

**The third Doctrine write is the expensive one and it is the point.** Roles
§4.2 has a colonizer recycle into the new colony's `Band I` infrastructure, and
T-70 makes that exact: `founding_infra = hull_cost`, because a hull's mass *is*
its cost (R-O57, design law #11). A colonizer that patrols instead of scrapping
keeps its minerals in the hull, so **the colony it founds starts with only what
the hold carried as endowment** (`Hyades_industry.md` §1.7). Mass is conserved
either way — the trade is *where the minerals stand*, not whether they exist.

### 8.3 One third of the stated intent is impossible, and the engine says so exactly

The design intent was three properties at once, against the Medium Systems hull
the colonizer rides today: **less cargo per kilotonne of cost, lower laden
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

| `φ` for the Contact colonizer class | cost | cargo/cost | dry `a` | laden `a` |
|---|---|---|---|---|
| **0.01** (shipped, global) | 1.0995 kt | 8.963 | 2.4767 g | 0.2486 g |
| **0.02** | 1.1991 kt | **8.136** | **3.7828 g** | **0.4141 g** |
| 0.05 | 1.4977 kt | 6.314 | 6.6595 g | 0.9105 g |
| 0.10 | 1.9954 kt | 4.490 | 9.5405 g | 1.7379 g |

`φ = 0.02` is the smallest Contact-class drive that makes the cargo-inefficiency
claim true against the MSV, and it raises laden acceleration by **+63%** rather
than lowering it.

**Decision: drop "lower laden acceleration"; take the narrowed signature
instead.** The design was reaching for *the armed colonizer is worse freight*,
and the identity says the compensation for that is not a slower voyage — it is
**concealment**, which is a better fit for the tree that gets it. §9.2 of
`Hyades_standing_layer_and_observation.md` reads the empty-to-laden swing as the
load-state broadcast: a large hull announces whether it is laden and a small one
does not. The identity says the *width* of that broadcast is exactly the hull's
cargo efficiency, so a Contact colonizer at `φ = 0.02` swings 9.14× where the
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
on work-years (§2.3.3), the Warfare card on §2.3.2's neighbor-weighted colony
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
multiplier acts on a rate forever, while the armed colonizer's value is an
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
| 5 | population is not a factor of production | T-107 | §8.1 — the license the tree is being given costs its victims nothing |

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

> **Every number in §8.6, §8.7 and §8.8 was measured before R-WAR9 (§8.9.6),
> and the engine that produced them no longer exists.** The colonization leg was
> flown at the empty-hull rate, so a colony ship crossed 25 ly in 26.9 years
> where it now takes 32.3 — the expansion loop is slower everywhere and the
> whole bed moves. Re-measured after the fix, the first arm below reads
> **−21.46% / +9.25% / −281** against the −9.55 / +4.40 / −133 it records.
> **The direction is unchanged and the magnitudes are not.** Read these sections
> for their mechanisms; read §8.9 for the magnitudes.

**`OPEN`, and the direction is refuted rather than untuned.** The mechanic is
built, gated off, and measured on the asymmetric bed R-TREE8 asks for: seat 0
plays the card, seats 1–2 are at the default doctrine.

#### What was built

A colonizer under `Doctrine::picket_after_founding` is **not consumed when it
founds**. It flies on to the nearest unclaimed world it has scanned and holds
it; nobody else founds there while it does. A rival colonizer already in flight
is warned at `established_at + distance(world, its home center)` — card contract
§2's rule verbatim, *"a reaction to a detected fleet is scheduled by the distance
to the responder"* — and turns back if the warning beats it there. If it does
not, it arrives into a fight it did not choose, and founds only if it wins.

That is the author's specification implemented as stated, and it is the first
thing in the engine that takes a decision at **range** on a light-lagged edge.

#### What it does, on eight seeds at 800 yr (`examples/denial_census`)

| | mean | seeds |
|---|---|---|
| **neighbor colonies** | **+4.40% ± 1.09** (4.0 SE) | 1/8 negative |
| own colonies | −9.55% ± 2.61 (3.7 SE) | 7/8 negative |
| `W_0 = C_0 − mean(C_j)` | **−133.2 colonies ± 35.3** (3.8 SE) | 1/8 positive |

**The neighbors expand *more*.** The stated goal was to curb their growth; the
card accelerates it, at 4.0 SE with seven of eight seeds agreeing. Warfare's own
objective moves the wrong way by 133 colonies.

#### The mechanism, and it is placement rather than magnitude

**Pickets are fielded and almost never used.** 493–793 pickets per run produce
**3–12 diversions** — a utilization of one to two percent. A picket is placed on
the world nearest the colony that just founded, which is the *picketing empire's
own* frontier: it squats on ground its own colonizers were about to take and
threatens nobody.

**Aiming it at the neighbor is worse, and the reason is structural.** Placing
the picket on the scanned unclaimed world nearest the closest rival homeworld
gives **24–32 pickets and zero diversions** on every seed. A world that this
empire has scanned and a rival has not yet claimed barely exists: near a rival,
everything visible is already owned. **There is no contested frontier to stand
on** — colonization is exclusive (R-V3) and worlds go from unscanned to owned
without pausing in between.

So the two placements fail for opposite reasons, and neither is a tuning
question.

#### And a 1:1 trade would still lose, which is arithmetic rather than a measurement

Suppose placement were solved and every picket denied exactly one neighbor
colony. Seat 0 spends `k` colonizers to field `k` pickets, so `C_0` falls by `k`
and the denied neighbor's count falls by `k`. Then

```text
ΔW_0 = −k + w_0A · k = −k(1 − w_0A)
```

and `Σ_j w_0j = 1` by construction (§0), so `w_0A < 1` whenever there is more
than one neighbor and **ΔW_0 < 0**. A denial bought with a whole colonizer is a
losing trade against the tree's own objective at any table wider than two seats,
however well it is aimed.

Measured, the realized trade is far past that bound: seat 0 gave up **718
colonies** across the bed and the neighbors gained 696 — a ratio of **−0.97
denied per spent**, where even +1.00 would not have been enough.

#### What would have to change

Stated as the design question rather than left as a failure:

- **A picket must cost less than a colony.** The card spends a colonizer — the
  most expensive object in the expansion loop — on an object that produces
  nothing. A cheap dedicated hull breaks the 1:1 bound above; a recycled
  colonizer cannot.
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
colonies to 10** and *raised* the neighbors **+20.0%**: the empire had removed
itself from the game, and the neighbors' gain was one fewer competitor rather
than any denial at all.

**The line is `employment_rate`** (`src/sim.rs`): it returns exactly `0.0` for a
stock of zero, and `fabrication_rate` is `slips × berth_rate`, so a colony
founded at infra 0 can never mine, never build and never recover. Zero is not a
price, it is an **absorbing state**.

The founding rung is now floored at the ladder's own floor (`Band Empty`,
design law #11/T-63) — the smallest rung that is a real quantity. Three arms,
same bed, and the monotonicity is the evidence that the floor is the term that
matters:

| founding rung under the card | own colonies | neighbor colonies | `W_0` |
|---|---|---|---|
| zero | −57.2% | **+20.0%** | −688 |
| **floor (shipped)** | −9.6% | **+4.4%** | −133 |
| unchanged `Band I` | +2.6% | −0.7% | +31 |

**The neighbors' gain tracks the card-player's self-harm one for one**, across a
25-point range, which is what says the measured effect is self-harm and not
denial. Even the arm that charges the card *nothing* only moves the neighbors
−0.71% ± 0.54 — inside noise, and nowhere near "greatly decrease".

`a_picketed_founding_still_leaves_a_workable_colony` pins the floor against the
absorbing zero. **R-WAR6**: whether `Band Empty` is the right rung, or whether a
departing picket should instead found at whatever its mineral endowment buys
(§1.7), is unratified.

### 8.8 Claiming instead of denying — three arms, and all three are bounded by supply (T-113)

**`OPEN`.** §8.6 refuted denial *as arithmetic*: a denial bought with a whole
colonizer gives `ΔW_0 = −k(1 − w_0A)`, and `Σ_j w_0j = 1`, so it loses at any
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
| **claim the target** | `picket_claims_target: bool` | a center that has named an outward world it cannot yet pay for holds it while the bank fills |
| **erect the hold** | `founding_infra_share: f64` | part of a colonizer's mineral endowment becomes the new colony's stock instead of its bank |
| **settle held ground** | — (unconditional; self-gating) | a colonizer prefers a world this empire's pickets already hold over a better one nobody holds |

`Class::Unnamed` on `LimitedOffensive` is a Design write, so TIER0 card 15
(Warfare / Inscrutable) is now `UnlockDesign` rather than `NotYetImplemented`.

#### What they measure (`examples/denial_census`, 8 seeds, 3 seats, 800 yr)

Seat 0 plays the card; seats 1–2 at the default doctrine. `own` and
`neighbors` are percentage changes against the peace arm on the same seed
(CRN); `W_0` is in colonies. **Neighbors should fall and `W_0` should rise.**

| arm | own | neighbors | `W_0` | diverts | pickets |
|---|---|---|---|---|---|
| T-112 colonizer pickets, **+ settle held ground** | −8.30 ± 2.69 | +4.07 ± 1.09 | −120 ± 34 | 59 | 5,415 |
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
`a_colonizer_prefers_held_ground_to_a_better_unheld_world` pin both halves.

#### The cheap picket is not measurably cheaper, because it is not built

§8.6's arithmetic says a picket costing a fraction of a colony clears the
`1 − w_0A` bound. The measurement cannot see that, because the hull is fielded
**95 times across eight seeds** — twelve per run, against 5,400 colonizer
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
thing a center can do with a cycle: it has **named an outward world and cannot
pay for it yet**. Those cycles are already spent saving, and a picket standing
on the target is what makes the voyage safe to have committed to. It is gated
on the target not already being held *or claimed* — `Simulation::picket_inbound`
counts hulls already under way — so a center lays down one picket per world
rather than one per decision for the whole voyage.

Supply goes **95 → 188 pickets**, and every objective column stays inside 2 SE.

Without the in-flight guard the same arm reads **−4.87 ± 5.65 / +1.05 ± 2.75 /
−58 ± 85 on 177 pickets**. The 3.3-point difference in own colonies is smaller
than either arm's error bar, so **the guard's value is not resolved at eight
seeds**; it is kept because laying down a hull per decision for the length of a
voyage is waste whether or not this bed can price it.

**The state is rare**: expansion on this bed is not bound by the colonizer's
price, so centers seldom sit in "named it, cannot buy it". That is consistent
with `Hyades_industry.md` §6.19c, which measures the mineral constraint as a
*color* conjunction on the infrastructure bill rather than a shortfall against
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

At share 0 the **maximum** is the floor: no colonizer under this doctrine lands
with an endowment worth anything. At 0.5 the median is still the floor and only
a fifth of foundings clear it — and **the whole hold erected (1.00) adds 1.5
points**, so the axis is exhausted rather than undertuned.

**A colonizer's hold is nearly all settlers.** `settler_target` sizes the
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
  being colonizer-price-bound).
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


### 8.9 Two responders, a moving picket, and an armed scout (T-115)

**`OPEN`.** Four corrections, three of them to things §8.6–8.8 got wrong rather
than to things that were merely untuned. The author's specification: *"Pickets
should move to the frontier if the site they patrol is colonized. LOU can
replace LSV as scout in this Doctrine… I think the enemy colony ships should be
reacting in addition to enemy home centers. Also, pickets should actively close
the distance to targets they can intercept prior to founding a colony."*

#### The bed, re-measured (`examples/denial_census`, 8 seeds, 3 seats, 800 yr)

Seat 0 plays the card; seats 1–2 at the default doctrine. `own` and `neighbors`
are percentage changes against the peace arm on the same seed (CRN); `W_0` is in
colonies. **Neighbors should fall and `W_0` should rise.** Every row is measured
*after* R-WAR9 (§8.9.6), so this table and §8.6–8.8's are from different
engines and must not be read against each other.

| arm | own | neighbors | `W_0` | diverts | pickets |
|---|---|---|---|---|---|
| T-112 colonizer pickets | −21.46 ± 4.41 | +9.25 ± 2.78 | −281 ± 72 | 15 | 4,974 |
| + hold erects infra (`share = 0.5`) | −21.25 ± 4.48 | +9.16 ± 2.74 | −276 ± 70 | 14 | 5,012 |
| + cheap LOU pickets (`reserve = 128`) | −21.02 ± 4.55 | +8.86 ± 2.52 | −267 ± 63 | 21 | 5,079 |
| LOU pickets alone | −2.87 ± 2.11 | +1.06 ± 0.58 | −29 ± 19 | 1 | 188 |
| + claims its target | −2.64 ± 2.38 | +1.76 ± 0.80 | −49 ± 25 | 2 | 333 |
| **+ the LOU scouts** | **−2.87 ± 2.11** | **+1.06 ± 0.58** | **−29 ± 19** | **1** | **188** |
| + and intercepts | −2.29 ± 1.99 | +0.85 ± 0.62 | −22 ± 20 | 0 | 195 |

*(An `intercepts` column was added after this run and is measured separately;
see §8.9.5. The objective columns reproduce exactly with logging on, which is
`tests/telemetry.rs`'s bound holding.)*

**The direction §8.6 measured is unchanged and the magnitudes roughly doubled**:
a colonizer that keeps its hull now costs its own player 21.5% of its colonies
where it cost 9.6, because a slower expansion loop makes a spent colonizer dearer.
The cheap-hull arms are the mild ones and the last three are inside 2 SE on every
column. The `LOU scouts` row reproducing the one above it **to every printed
digit** is §8.9.7.

#### 8.9.1 The colony ship hears it itself, not only through its capital

§8.6 scheduled one warning, at `now + distance(world, the ship's home center)`,
on card contract §2's rule that *a reaction to a detected fleet is scheduled by
the distance to the responder*. That rule is right and the reading of it was too
narrow: **there are two responders.** The home center issues an order; the crew
notices a picket. A hull flying at the world the picket took is closing on the
source, so the light reaches it *sooner* than it reaches anything standing
still at its launch range.

Both are now scheduled. The ship-side instant is the root of

```text
f(t) = (t − now) − |p(t) − world|
```

— where the light's travelled distance equals the hull's current range — found
by 48 bisections, because the trajectory is a relativistic 1 g brachistochrone
with a departure delay and has no elementary inverse. Bisection uses comparison,
addition and `sqrt` only, all exactly specified by IEEE 754, where a fitted
approximation would be a per-platform divergence in replicated state (design
law #16).

#### 8.9.2 Which forces a second question, and it is kinematic

Light outruns a colony ship, so with the ship-side channel the warning **always**
arrives before the hull does. *"Did the warning arrive in time"* therefore stops
discriminating, and left alone it would turn every ship back and delete the
arrival fight §8.6 built.

The question that replaces it costs no constant. A leg accelerates to the
midpoint and decelerates from it, so **before turnover the hull's velocity still
points somewhere it can choose; after turnover it is shedding velocity it has
already spent and arrives whatever it decides.** The gate is
`now < (depart + arrive) / 2`.

So the counterplay window is a **margin** rather than a race, and both channels
still matter: the crew's own sighting is early, and the order from home may or
may not land before the hull is past turning.

#### 8.9.3 A picket whose world gets settled goes back to the frontier

A picket's whole product is a colony somebody does not found. Once the world it
stands on *is* a colony, a parked hull is a hull doing nothing — and worse, one
the empire keeps counting, so `picket_reserve` throttles the replacements it
should be buying. `Simulation::vacate_picket` takes it off station and off the
books in one function, because every path that removes a picket has to move both
and the one that forgot would leave a permanent phantom holding.

This is the situation T-113's held-ground preference *creates*: settling ground
you hold is the recommended play, and until now it stranded the hull that held
it.

#### 8.9.4 The armed hull takes the survey slot

§8.8 measured the cheap picket's problem as supply: **95 hulls across eight
seeds**, because the picket branch sits behind a survey test R-O86 measured a
constant `true`. `Doctrine::picket_claims_target` answered that by finding the
picket a new branch. `Doctrine::scout_hull_offensive` answers it the opposite
way — **put the picket in the branch that already runs constantly.** An empire
that wants ground held is already building an unarmed light hull in the slot
where it could be building an armed one.

The survey *cadence* is untouched; only the hull it names changes.

**The price was expected to be paid at the yard, and it is not** — an LOU and an
LCV cost the same 0.020 kt, so the write is bit-identically inert. §8.9.7 has
the two reasons and what would make either bite.

**A scout built this way carries `Class::Tor`.** The class is the design
(R-O28/R-O42b), so a Tor on an offensive shell is a survey design mounted on a
fighting hull — which is what a Design write does — and it is how `assign_role`
tells a scouting LOU from a picketing one without a second piece of state.

> **Note on the specification.** The author wrote *"LOU can replace LSV as
> scout"*. The engine's scout rides the **LCV** (`role_hull_type(Role::Scout)`,
> `Class::Tor`, §7.1's starting roster); the LSV is the mining hull. The write
> replaces the LCV.

#### 8.9.5 Interception, and the geometry that decides when it fires

`Doctrine::picket_intercepts` lets a picket leave station for a world it has
seen a colony ship launched at, when it can be on the ground first:

```text
t_see   = depart + |origin − station|              // light, c = 1
t_reach = t_see  + flight(station → world)
go iff t_reach < colony_arrival
```

Every term is an existing quantity — no reaction time, no interception radius,
and nothing the picket knows that light has not delivered. Cost is
`O(pickets held)` per colony-ship launch, and one `is_empty()` where nobody
plays the card.

**The window is the colony ship's travel time minus light's**, because light
carries the news over the same ground the ship crosses. Measured
(`examples/intercept_probe`), with the colony ship flown at the rate its load
implies (§8.9.6):

| range | light | laden colony ship | **window** |
|---|---|---|---|
| 5 ly | 5.000 yr | 10.395 yr | 5.395 |
| 25 ly | 25.000 | 32.253 | 7.253 |
| 100 ly | 100.000 | 107.986 | 7.986 |
| 200 ly | 200.000 | 208.139 | **8.139** |

A laden Medium colonizer makes **0.241 ly/yr²** against an empty hull's 2.446,
so its overhead over light **saturates at ≈8.14 years**. An empty LOU picket
(0.940 ly/yr²) covers **6.29 ly** in that time, against a median
nearest-neighbor spacing of **6.16 ly** on the shipped field:

| nearest-neighbor spacing (6,725 planets) | min | p10 | median | p90 |
|---|---|---|---|---|
| distance | 0.345 ly | 2.911 ly | **6.162 ly** | 15.469 ly |
| picket hop | 1.260 yr | 4.567 yr | **8.012 yr** | 17.468 yr |

**The median world sits just inside the window**, so roughly half the field is
interceptable from a neighboring station — and the margin at the median is
0.13 years, which is thin enough that the reach is a property of the drive
ladder rather than a comfortable constant.

**And the sign of one term is the design.** A station 0.5 ly *beyond* the
contested world loses the race; the same 0.5 ly on the *near* side wins it,
because the far side pays that distance twice — once in light to hear the
launch and once in flight. **Interception is a forward-deployment mechanic**,
not a reaction one.

#### What it actually does, counted

`LogEvent::PicketIntercept` exists because neither the objective nor the picket
column can see this mechanic: an interception **moves** a hull rather than
building one, so the census's picket count barely registers it. Counted
directly (`intercepts` column, 8 seeds):

| | value |
|---|---|
| interceptions launched | **81** (≈10 a run) |
| picket arrivals on station | 195 |
| **diversions caused** | **0** |
| `W_0` against `LOU pickets alone` | −22 ± 20 against −29 ± 19 |

**It fires, on about 40% of the pickets that exist** — the geometry of §8.9.5 is
real and the mechanic reaches it. What it does *not* produce is diversions, and
that follows from §8.9.2 rather than being a separate failure: a picket that
arrives just ahead of a colony ship arrives when that ship is long past
turnover, so the warning lands on a hull that cannot turn. **Interception
converts an uncontested founding into a fight at the destination**, which is
what `resolve_picket_fight` then resolves.

Whether that pays is **not resolved at eight seeds**: −22 ± 20 against −29 ± 19
is a 7-colony improvement with error bars three times its size, and the two arms
share seeds so the difference is paired — the missing measurement is more seeds
on an arm that is still supply-starved at 24 pickets a run.

#### 8.9.6 R-WAR9 — the colonization leg was flown unladen, and it is fixed

**Resolved.** `Simulation::spawn_courier` computed the leg's acceleration as
`civilian_accel_g · G` **before** the hold was loaded and never re-read it, so a
colony ship flew at the empty-hull rate while every `laden_accel` call site in
the engine was freight. `CLAUDE.md` §7 records standing-layer item 5 (R-O32) as
closing exactly that — *"it was massless, so a laden colony ship flew like an
empty hull"* — and it had closed it for the arena and not for this dispatcher.

The leg now reads `laden_accel` after the hold is loaded, which also stops it
pretending every hull mounts the same drive (T-96); `launch_picket` and the
interception's own feasibility test read the same function, so the decision and
the leg cannot disagree about who wins a race. Guarded by
`the_colonization_leg_is_flown_at_the_rate_its_own_load_implies`.

**It contradicts every transit-dependent magnitude measured before it**, this
document's own §8.6–8.8 arms included: a colony ship is 20% slower over 25 ly
and 56% slower over 5, so the expansion loop is slower everywhere and the whole
bed moves. §8.9's table is measured after the fix; §8.6–8.8's are not, and the
comparison between them is between two different engines.

**How it was found is the reusable part.** Nothing was looking for it. The
interception mechanic needed a number — how long a colony ship takes compared to
light — and the number turned out to be a property of a defect rather than of
the design. `CLAUDE.md` §3's *"a second caller is a cheap audit of the first"*
holds for physics as well as for panics.

**And the first answer this document gave was wrong because of it.** §8.9.5
originally reported a 1.92-year window and concluded interception would fire on
fewer than a tenth of worlds. That figure was the *empty* hull's overhead,
computed from the same defect that had already been written up one subsection
below. **A probe inherits every assumption of the code it measures**; when one of
those assumptions has just been found wrong, the probe is part of what has to be
re-derived.

#### 8.9.7 The armed scout is bit-identically inert, and the reason is the cost ladder

`Doctrine::scout_hull_offensive` reproduces the arm without it **to every
printed digit** — 188 pickets, −2.87% own, +1.06% neighbors, `W_0` −29, on all
eight seeds, and it did so before the R-WAR9 fix as well as after it. That is
the *exactly zero* verdict rather than the *inside noise* one (`CLAUDE.md` §2),
and its cause is one line:

**`hull_dry_mass(LimitedContactVehicle) == hull_dry_mass(LimitedOffensive) ==
0.020 kt.`** Under R-O57 cost *is* dry mass, and `hull_dry_mass` reads the cost
*tier*, which groups every Limited hull together. So an LOU and an LCV are the
same object economically — same price, same mass, same shell — and swapping one
for the other changes nothing the simulation reads.

The two hulls *do* differ in `shell_thickness` and `drive_mass` — and
**neither reaches a scout.** `launch_survey` flies its leg at
`doctrine.survey_accel_g · G`, a flat constant, where the colonization and
picket legs now read `laden_accel` and therefore read the hull's own drive
(§8.9.6). So the survey leg has exactly the defect R-WAR9 just closed for the
other two, and closing it there as well is what would make the drive half of
this write bite.

So the write is inert for two independent reasons and is kept for both: it
becomes live in **price** when hull types carry differentiated cost (**R-O64**,
roles §6's 0/1/2 was a unit count and not a mass ladder; **R-L0**, per-hull slot
tables), and live in **speed** when the survey leg reads the hull it is flying.
Until then *"the LOU takes the survey slot"* is a Design statement with no
mechanical content.

It also weakens §8.6's framing. *"A picket must cost less than a colony"* is
satisfied — 0.020 kt against a Medium colonizer's 0.109 — but the cheap armed
hull is not cheaper than the cheap unarmed one, so *arming* the frontier is
free and the trade §8.6's arithmetic describes is not the trade being made.

#### 8.9.8 What is still not modeled — `R-WAR10`

**`OPEN`.** `offer_interception` is handed the colony ship's **destination**,
read from the launch. That is not what a picket can see. What light delivers is
a *trajectory* — a departure point, a time, and a heading — from which a
destination is an inference, and design law #10 already says the observable is
acceleration rather than intent.

So the destination should be **inferred**, from the heading and the worlds along
it, which makes a picket's response wrong sometimes — and a colony ship that
launches on a bearing it does not intend to keep is then running a feint, which
is yomi content rather than a modeling gap. That is the whole of R-WAR10.

**And the thing it is *not* is worth recording, because it was the obvious next
idea and one probe refuted it.** *"Meet the ship anywhere on its path rather than
racing it to a world"* sounds like a strictly larger set and is a narrower one,
because **the interception window is back-loaded.** On a 25 ly voyage
(`examples/intercept_probe`), the perpendicular offset a station can sit at and
still make the meeting is:

| point on the track | 12.5% | 25% | 50% | 75% | 87.5% | **destination** |
|---|---|---|---|---|---|---|
| tolerable offset | 1.24 ly | 1.62 | 1.95 | 2.25 | 2.62 | **4.96 ly** |

The slack a picket lives on is `t_ship(along) − along` — how far behind light the
ship is at that point — and it **accumulates**, because the hull spends the back
half of a brachistochrone decelerating. Early in the voyage there is almost
none. So the destination is where the whole window is, and the shipped criterion
is already taking it; meeting on the path would trade the mechanic's entire
margin for generality it does not need.

**What would settle R-WAR10**: a heading-based candidate set — the worlds within
some cone of the observed bearing — in place of the true target, and a measure of
how often the inference is wrong. It does **not** need the reachability cone
(R-O31/T-05) or a deep-space engagement site, which is what this subsection
originally claimed.

### 8.10 Why `W_0` is negative — it is one cost, and it compounds (T-116)

**`OPEN` as a design question; the mechanism is closed.** Six arms across §8.6–8.9
agreed on the sign and none of them said why. `examples/warfare_why` decomposes
it, and the answer is a single term.

#### The decomposition

`W_0 = C_0 − mean_j C_j` moves for two reasons a single number cannot separate:
the card's player founds fewer colonies, and its neighbors found more. Those are
not independent — colonizable worlds are a shared pool, so a world seat 0 does
not take is one somebody else can — so the harness reports **the transfer
ratio** `Δneighbors / −Δown` beside the total. At 1.0 the card is a pure
handover; below it, some of the loss is worlds nobody takes.

4 CRN seeds, 3 seats, 800 yr, seat 0 alone playing. Per-seed means in colonies;
`MSV`/`Gen` are seat 0's hull builds over the whole bed.

| arm | own | neighbors | `W_0` | MSV | Gen | founded | transfer |
|---|---|---|---|---|---|---|---|
| peace | — | — | — | 6,889 | **0** | 3,562 | — |
| Design write alone | +0.0 | +0.0 | **+0.0** | 6,889 | **0** | 3,562 | — |
| `SettlersPerMineral` alone | +287.8 | −107.1 | **+394.9** | 8,836 | 469 | 4,713 | — |
| …+ the Design write | +220.0 | −93.1 | **+313.1** | 7,415 | 639 | 4,442 | — |
| **Doctrine write alone** | **−197.2** | **+90.5** | **−287.8** | 5,665 | 0 | 2,773 | **0.46** |
| …with the founding rung restored *(ablation)* | −1.0 | +1.0 | **−2.0** | 6,492 | 0 | 3,558 | 1.00 |
| the card as §8.2 specifies it | −197.2 | +90.5 | −287.8 | 5,665 | 0 | 2,773 | 0.46 |
| **both halves, both live** | **−337.5** | **+153.6** | **−491.1** | 4,162 | 637 | 2,212 | 0.46 |

The card as §8.2 specifies it reproduces the Doctrine write **to every printed
digit**, which is the first row saying the same thing: at the shipped policy the
card *is* its Doctrine half.

#### The cause, by ablation

**Restoring the founding rung takes the Doctrine write from −287.8 to −2.0.**
That is `SimConfig::ablate_picket_founding_cost`, which violates conservation and
must never ship — its only job is to remove the suspected cause and watch the
effect go, which is the one method that can refute (`CLAUDE.md` §2).

So the chain is:

```text
colonizer keeps its hull
  → the colony gets no recycled stock: floor rung 0.020 kt, not 0.109  (5.5x)
  → that colony produces less, forever
  → the empire builds 18% fewer colony ships   (6,889 → 5,665)
  → it founds 22% fewer colonies               (3,562 → 2,773)
  → neighbors pick up 46% of them              (transfer 0.46)
  → W_0 = −(1 + 0.46) x the loss
```

**Everything else the card does is worth about two colonies.** Pickets,
engagements, diversions and kills are all still present in the ablation arm —
5,000-odd pickets, 20,000-odd kills — and with the founding rung restored the
whole apparatus nets −2.0. The denial mechanic is not *losing* the objective; it
is not *touching* it.

#### Which corrects §8.6's arithmetic in the direction that matters

§8.6 bounded a 1:1 denial as `ΔW_0 = −k(1 − w_0A)` and concluded the card loses
at any table wider than two seats. That bound assumed the card **denies**.
Measured, the card's neighbors *gain* 0.46 colonies per colony it costs itself —
so the realized form is `ΔW_0 = −k(1 + t)` with `t = 0.46`, and there is no
denial term in it at all. The card is not a bad trade; it is **not a trade**.

#### What this says about where a Warfare card's price can go

Warfare's objective is a *difference*, so a price that compounds against your own
expansion is charged twice — once in your own count and once in everybody
else's. **A card whose cost is a permanent reduction in every colony's starting
stock cannot win it**, whatever the mechanic on the other side buys, because the
loss grows with the thing the objective measures.

The same finding arrives from the other side in §8.9: a picket bought with a
**dedicated LOU** costs −2.87% of own colonies where one bought with a
colonizer's founding stock costs −21.46%. Same mechanic, price moved, an order
of magnitude less damage.

**R-WAR11 (`OPEN`) — restated, because the first version of it was wrong.** It
claimed the engine has no way to charge a card a price. It has one:
`sim::apply_orders` checks `empire_can_afford` and calls `empire_spend(p,
c.cost)`, so `Card::cost` is a real one-off mineral charge against the whole
empire (design law #7's macro grain). The gap is not a missing mechanism.

The gap is that **a card has two costs and only one of them is priceable.**

| | what it is | bounded? |
|---|---|---|
| `Card::cost` | a one-off mineral charge at play | yes, by the number in the table |
| the mechanic's own cost | here, every colony founded afterwards starts 5.5x thinner, forever | **no** |

The second one cannot be priced, because its size depends on how much the
player goes on to expand. Whatever number sits in `Card::cost`, the card is
cheap for a seat that was going to stall and ruinous for one that was going to
compound — and under a *difference* objective it is charged twice, once in your
own count and once in everybody else's.

So the open decision is **whether a Doctrine write may impose a recurring cost
that scales with its holder's own activity at all**, or whether a card's total
cost must be bounded at play time. It is not Warfare-specific: any write that
degrades a per-unit economic rate has this shape, so it belongs in the card
contract rather than here.

**And note what the measurements in §8.10 did *not* include**: they set the
Doctrine fields directly rather than playing the card through `apply_orders`,
so `Card::cost` was never charged. The measured `W_0` is the effect **without**
its price. Charging it makes the card worse, not better.

#### The two halves are multiplicatively bad together, and that is the same term again

`+313.1` and `−287.8` should compose to about `+25`. Measured together they are
**−491.1** — an interaction of **−516**, larger than either half.

One line explains it. `founding_infra(hull)` **is** `hull_cost(hull)` (T-70,
R-O57: the recycled hull's minerals *are* the stock), so the price the Doctrine
write charges is the mass of whatever hull the colony ship happens to be. The
Design write mounts the colony role on a hull costing **1.0995 kt against a
Medium's 0.1092 — 10.07x** — so the same mechanic forfeits ten times as much.

**The card's price is proportional to the thing its other half makes bigger.**
That is not a tuning collision, it is the two writes reading the same quantity
with opposite intent, and it is why the whole card is worse than either piece.

#### And the Design write is not the problem — it is `+313`

The armed colonizer costs what §8.2 says it should and **wins `W_0` anyway**:
+313.1 against the +394.9 of the policy change it rides on, so the GCV gives up
**67.8 colonies** of that gain — a price, paid, with the objective still far
positive. **The negative `W_0` this document has been tracking since §8.6 comes
entirely from the Doctrine half.**

#### Two things the same run says that are not about this card

- **The Design write is unreachable at the shipped colonizer policy.**
  `CheapestViable` keys on negated price and a Medium hull costs 0.1092 kt
  against any General hull's ~1.1–1.3, so the General slot is selected only if
  the Medium is filtered out for delivering zero settlers — which
  `sim::settler_target` cannot do independently of the hold (its zero cases are
  `hi ≤ 0`, `k_origin ≤ 0`, `k_target ≤ floor`, none of which mention the hold
  beyond it being positive). Measured: **zero General colonizers built across
  four seeds**, and the arm is bit-identical to peace.
  `the_cheapest_viable_policy_never_names_a_general_colonizer` pins the algebra.
  **The Contact ladder has no Medium rung** — `HullType` runs Limited and
  General only for Contact — so a card that mounts the colony role on a Contact
  hull is forced to the General tier, which is the tier nothing selects.
- **`SettlersPerMineral` is worth more than any card measured so far**, at
  **+394.9 `W_0`** unilaterally. See **R-IND11** below.

### 8.11 The standing layer answers; it is not switched on (T-117)

**Ratified as architecture, not as a magnitude.** Three card landings in a row
(T-113, T-115, T-116) each added a Doctrine write, and each added the *same
three branches* to carry it: one in the build order, one in the price the
production context reads, and one in `assign_role`'s match on hull type. Three
readings of one write is how they come to disagree — and twice in this document
they did, silently.

#### What replaced them

`autopilot::Standing` is a borrowed reading of Design and Doctrine that answers
questions instead of exposing flags:

| call | answers |
|---|---|
| `design_for(role)` | the hull **and class** this layer lays down for a role |
| `colonizer_ladder()` | the colonizer rungs, cheapest first |
| `mounts(role, hull)` | whether this layer puts that role on that hull |
| `role_of(hull, class)` | **the inverse** — what a finished design is for |
| `recycles_on_founding()` | whether a colony ship is consumed by its colony |

The load-bearing property is that **`role_of` is derived from `design_for`
rather than written out**, in three passes: an exact design match, then the hull
alone for a class the layer has not named, then a **competence table** (R-O44)
for a hull no write has claimed. So a card that mounts the colony role on a
Contact hull changes one function and `assign_role` follows with no edit.
`role_of_inverts_design_for_every_role` pins it across every combination of the
writes that move a design — they compose, and a resolver correct one write at a
time is not one.

#### What it caught

- **`assign_role` was not total.** A Rapid or General Offensive hull matched
  `_ => None`, so a hull the yard had already been charged for could come back
  with no mission. The same shape had already cost the engine real work twice:
  `launch_survey` discarding a paid-for hull (T-116), and `apply_build_with`
  spending minerals on objects that never existed (R-O86).
  `every_hull_has_a_role_under_every_doctrine` requires totality.
- **A `max` standing where a sum belonged.** A colony credited its founding
  stock from the recycled hull and *then* from the hold with
  `f.infra = f.infra.max(erected)` — so a colony that recycled a hull **and**
  landed minerals got whichever was larger and the other vanished. It is `+=`
  now. Mass is conserved either way only because the shipped
  `founding_infra_share` is 0.0 and the second term was always zero.

#### The hold erects in ratio, not in total — `R-IND20`

**An infrastructure rung is billed per color** (`works_bill` splits it by
`works.mix_share(c)`; `can_pay_bill` tests each color separately), so minerals
standing in a colony ship's hold are worth only what their **scarcest** color
allows:

```text
erectable = eta_works · min over c of ( aboard[c] / mix_share(c) )
```

`sim::erectable_from` is that conjunction. A hold that is all Cyan erects
**nothing**, however much of it there is — which is the same conjunction
`Hyades_industry.md` §6.19c measured as the mineral economy's binding
constraint, arriving at founding instead of at a production decision. What the
ratio cannot use is banked rather than lost, so `consumed + remainder == aboard`
exactly.

The version this replaces treated the hold as a scalar total, which let a
single-color hold stand up as infrastructure **no center could have bought with
the same minerals** — a rung obtainable by arriving that was not obtainable by
paying for it.

**`eta_works` multiplies here because `works_bill` divides by it**, which is
what makes a Production card's works efficiency reach a founding as well as a
build.

#### What it cost, measured

**Nothing.** Colonies and total population reproduce **bit-identically on all
four CRN seeds** against the pre-refactor binary (2,946 / 2,849 / 2,861 / 2,955
colonies; 4,788,955 / 9,786,675 / 8,831,325 / 9,456,757 kt). Two configurations
resolve differently than the switch did — a `Tor`-classed Limited Offensive hull
with the scout write *off* now reads as a picket, and the two large Offensive
hulls read as pickets where the switch returned `None` — and **nothing builds
either pairing today**, which is why the bed does not move.

**R-IND20 (`OPEN`)**: whether founding should erect from the hold at all is a
magnitude question (`founding_infra_share`, default 0.0, measured flat at
§8.8 because a colony ship's hold is nearly all settlers, R-WAR7). The *rule*
— that whatever it erects respects the works mix — is ratified here.

### 8.12 A picket guesses, and can be feinted (R-WAR10, resolved — T-120)

**Ratified.** §8.9.8 left `offer_interception` being *handed* the colony ship's
destination, which is not what a picket can see. Light delivers a departure
point, a departure time and a **bearing**; the destination is an inference from
them, and design law #10 already says acceleration is the observable and intent
is not.

#### What a picket now knows

```text
t_see   = depart + |origin − station|          // light, c = 1
bearing = the direction the hull is actually moving
guess   = a world this picket has scanned, within `intercept_cone` of that
          bearing, that it can stand on before the ship arrives
```

**Candidates come from the picketing empire's own `scanned` set.** A world it
has never surveyed is not a world it can guess at, which keeps the inference
inside what this player knows rather than inside what the board holds (design
law #15). It backs the world **nearest the origin along the bearing**: a colony
ship is slow and expensive, so the near world on a line is the cheaper errand
and the better prior. It is a prior, not knowledge.

#### Which makes deception a move rather than a wish

Two worlds on one bearing are indistinguishable at range. A colony ship aimed at
the far one has spent **nothing** to put a picket on the near one —
`a_picket_backs_the_near_world_on_a_bearing_and_can_be_feinted` constructs
exactly that and asserts the picket takes the bait.

And the second half is what makes the feint cost something to sustain.
`EventKind::PicketReassess` has the picket re-read the trajectory every
`intercept_reassess_years`, at the light-lagged position — **what it sees is
where the quarry was when the light left it**, which is why a ship that has
already turned still looks, for `distance` years, like it is going where it was
going. If the bearing now points at a different world the picket re-aims from
where it is; the distance already flown is not refunded, so a picket that took
the feint has paid for it.

#### Two magnitudes, both placeholders — `R-WAR12`

| knob | default | what it sets |
|---|---|---|
| `intercept_cone_radians` | 0.15 rad ≈ 8.6° | how wide a guess is allowed to be |
| `intercept_reassess_years` | 25 yr | how often the guess is revised |

Neither is physical. The cone is a **legibility** knob: wider makes a decoy
bearing likelier to catch the world the ship actually wants, so it sets what a
feint is worth. The cadence sets how long one stays bought. They are the first
two magnitudes in this tree whose job is to price a *bluff* rather than a
kinetic outcome, and they should be tuned against the yomi channel rather than
against `W_0`.

#### What it does not do

The picket commits on the bearing it saw and cannot ask the ship anything. There
is no model of a ship *choosing* a deceptive bearing — the autopilot aims at the
world it wants, and a feint is available to a human or a future policy, not to
`BaselineAutopilot`. So the channel exists and nothing in the engine currently
uses it; that is the correct order (mechanism before policy) and it is stated
here so the absence is not read as a measurement.

---

### 8.13 The head-to-head, and the card has nothing to tune (T-120)

> **Measured inside the card-free opening (T-122).** `examples/card_table`
> played cards at `t ≈ 0`, before the round-0 barrier at 200 yr, so every card
> here paid its price out of the bootstrap bank. The mechanism reasoning below
> stands; the numbers describe no legal game. Re-measured at legal play in
> appendix §D.1.
>
> **Superseded in its conclusion by §8.14 (T-121), and correct as a record of
> the engine that produced it.** Everything below about *why* the card reached
> no decision is why §8.14 exists; the card now carries a Doctrine write and two
> Design unlocks, and the default standing layer is unarmed, so **the Warfare
> row of the table below no longer describes a playable card.** The Growth row
> and the bed itself carry forward. `R-WAR13`, opened here, is resolved there.

*§8.4 asks for the first Warfare card costed against the first Growth card on
one bed. `examples/card_table` is that bed. **The comparison is well-posed and
the Warfare side of it is empty**, for a reason that is structural rather than
statistical and was settled in the code before the runs finished.*

#### The bed

Twelve seats, three arms, cards played at earliest legal play (§2.4 of
`Hyades_trees_and_card_value.md`), three CRN seeds. Seats alternate so neither
tree sits systematically nearer the middle of the galaxy.

| arm | even seats | odd seats |
|---|---|---|
| `Pass` | — | — |
| `GrowthOnly` | — | Growth card (`TIER0[3]`) |
| `Both` | Warfare card (`TIER0[15]`) | Growth card |

Each card's effect is read against the arm that differs from it by that card
alone — Growth is `GrowthOnly − Pass`, Warfare is `Both − GrowthOnly`. Landing
both against `Pass` would charge each card with the other's effect on the same
galaxy.

`g` is fitted by **least squares on `ln X(t)`** over `[150, 600]` yr at a 25-yr
stride, which §2.4 requires and an endpoint pair does not satisfy; `R²` is
reported so a badly chosen window shows as curvature rather than hiding.

#### What it measures

| quantity | value | n | mean `R²` |
|---|---|---|---|
| Growth card value, `1 − t½ ratio` | **+0.1134 ± 0.0592** | 18 | 0.911 |
| Growth work stock at 600 yr | +1.3405 ± 0.9150 | 18 | — |
| Warfare card value, `1 − t½ ratio` | **−0.0371 ± 0.1609** | 9 | 0.791 |
| Warfare `ΔW_i` at 600 yr, colonies | −32.879 ± 46.075 | 18 | — |

**These are estimates, not bounds, and three caveats bound how far they go.**
`n` counts **seat-seeds**, and the six seats inside one galaxy share that
galaxy, so the independent replicate count is **3**, not 18 — every standard
error above is optimistic by an unmeasured factor. `W_i` is a difference and has
no logarithm where it is negative (R-TREE9), so the Warfare row is computed on
the **9 of 18 seat-seeds whose contrast was positive at both ends** — a set
selected by the quantity being measured, which is §2's mix rule and is the
reason the raw `ΔW_i` row is printed beside it. And the Growth row's `R²` of
0.911 says the window is in the exponential regime; Warfare's 0.791 on a
difference says rather less.

**Inference: the Growth card reduces its own tree's doubling time by about a
tenth, and the Warfare card does not measurably move `W_0` in either
direction.** Warfare's −0.0371 is inside one standard error of zero on this bed
and the raw `ΔW_i` is inside one standard error of zero as well.

#### The Warfare card reaches no decision, and that is not a statistical claim

**`TIER0[15]` is `UnlockDesign(HullType::LimitedOffensive, Class::Unnamed)`, and
the Roster has exactly one consumer outside tests.** `Sim::roster_permits`
returns `true` before it looks the roster up whenever `enforce_roster` is
false — which is the shipped default, and item 10 of the standing-layer roadmap
records why: with no unlock path, enforcement forbids every expansion build
permanently. So the card's write lands in a component nothing reads.

**A second, independent confirmation:** `role_hull_type(Role::Picket)` returns
`LimitedOffensive` unconditionally (T-113). The hull the card unlocks is one the
engine already hands to anybody who builds a picket. There is no state in which
this card is what makes an LOU available.

**So the card's only channel to the simulation is `Card::cost` — 0.5 kt, debited
from the empire at play.** That is a pure price with nothing bought, which is
why the arm does not reproduce `GrowthOnly` exactly and why the sign of
`ΔW_i` is negative. `the_first_warfare_card_is_a_price_and_not_yet_an_effect`
pins both halves, so this stops being true the moment enforcement lands.

#### Which answers both halves of the question asked

- **Is there a yomi-legible effect on `W_0`?** No, and not because the effect is
  small. Design law #9 makes legibility σ read from the other side; a write that
  reaches no decision changes no acceleration signature, no build order and no
  board state, so there is nothing on the other side to read. A card with no
  effect surface cannot carry a slant.
- **Can it be tuned to match Growth's +0.1134?** Not by moving a magnitude,
  because the card has no magnitude. Matching requires giving the card an effect
  first, and that is a design decision rather than a tuning one — §8.10 measured
  the effect the spec names for this slot (the denial doctrine) at **−287.8
  `W_0`**, so the obvious candidate is known to point the wrong way.

**`TIER0[15]` and the card §8.2 specifies are different objects, and this is the
first measurement that makes the gap concrete.** §8.2's card is one Design write
plus three Doctrine writes; `DoctrineWrite` carries four variants
(`GrowthRate`, `SurveyVehicles`, `BiosphereRegen`, `ReinvestBias`) and **none of
them is a Warfare write**. Everything §8.6 through §8.11 measured was reached by
setting `Doctrine` fields directly, which bypasses both `TIER0` and
`Card::cost`. The Warfare tree therefore has **no playable card that does what
§8 describes** — `R-WAR13`.

#### Two traps this bed exposed, worth having before the next card measurement

- **An unaffordable order coerces to a pass, silently.** `apply_orders` checks
  `empire_can_afford` and `Order::coerce` turns a failure into a pass rather
  than an error, so a bed that issues orders and then runs **cannot distinguish
  a card that did nothing from a card that was never played**. Round 0 happens
  to be legal today because `bootstrap` seeds each homeworld above every tier-0
  price, but that is a relation between two magnitudes nobody reconciled —
  `a_tier_zero_card_is_affordable_at_round_zero` pins it in both directions, and
  the harness records when each card actually landed (3.4 yr, mean over seats
  and seeds) rather than assuming.
- **`Sim::inert_card_plays` counts the wrong thing.** It increments only on
  `CardEffect::NotYetImplemented`, so a card that writes real state into a
  component with no live consumer is counted as working. It measures which match
  arm ran, not whether the write reached a decision — **`R-WAR14`**.

---

### 8.14 The default is unarmed, and the card is the key (T-121)

*§8.13 measured `TIER0[15]` as a price with no effect surface. This closes that
by making the card the only way into the Contact family, and by making the
card-free standing layer unarmed at every role. It is the author's
specification, and it resolves **R-WAR13**.*

#### What the default standing layer lays down now

| role | before | after | why |
|---|---|---|---|
| Scout | `LimitedContactVehicle` / `Tor` | **`LimitedSystems` / `Tor`** | a Contact hull is armed; the default is not |
| Colonizer | `MediumSystems` / `Unnamed` | unchanged | already unarmed |
| Colonizer, upper rung | `GeneralSystems` | unchanged | ditto |
| Miner | `LimitedSystems` / `Meadow` | unchanged | — |
| Picket | `LimitedOffensive` / `Unnamed` | **`LimitedContactVehicle` / `Unnamed`** | one armed family, one key |
| seeded roster | LSV(Meadow) + LCV(Tor) | **LSV(Meadow)** | see the contradiction below |

Every Warfare `Doctrine` field already defaulted to `false` or `0`, so the
**Doctrine** half was unarmed before this change and the **Design** half was
not. That asymmetry is what made the card a price: a seat flew Contact hulls
whether or not it bought the right to.

#### `LimitedOffensive` is no longer a role's hull, and nothing was priced away

The whole Limited tier shares one `cost_fraction`, so `hull_cost(LOU)` equals
`hull_cost(LCV)` exactly and §8.6's denial arithmetic — `ΔW = −k(1 − w_0A)`
with `Σ_j w_0j = 1` — runs on the same price it did. **Two things do move**, and
both follow from geometry rather than from tuning: an LCV has a nonzero hold
where an Offensive hull's is identically zero (§2.3's `b` term), and
`hull_thrust_to_mass` reads 1.4 against 2.0, which `arena` and `combat` read and
civilian motion does not.

`HullType::LimitedOffensive` keeps its place in the enum, its geometry row and
its entry in `competent_role`. No role mounts it.

#### The card carries three writes, and a card is a bundle now

`Card::effect` is `Card::effects: &'static [CardEffect]`, applied in slice
order. §8.2 specifies this card as *"one Design write, three Doctrine writes"* —
a card is a bundle of writes, and the Design/Doctrine pairing is the design
position (R-O37: opposite timing profiles, so the pair's value distribution is
the **narrower** of the two rather than the wider). Seventeen cards carry a
one-element bundle and read identically.

`TIER0[15]` is now:

```text
UnlockDesign(LimitedContactVehicle, Tor)      — Design, permanent
UnlockDesign(GeneralContactVehicle, Unnamed)  — Design, permanent
WriteDoctrine(ArmedFrontier)                  — Doctrine, revisable
```

`DoctrineWrite::ArmedFrontier` sets `scout_hull_offensive` and
`colonizer_general_contact` **together**, because they are one decision: an
empire that arms what it sends out has armed what it sends out, and splitting
them would let a player buy the cheap half of a slant (design law #9).

**§8.2's third Doctrine write is deliberately absent.** A colonizer that patrols
instead of recycling is `picket_after_founding`, which §8.10 measured at
**−287.8 `W_0`**. Bundling it would make the card strictly worse than passing,
which is R-WAR6's design question rather than a magnitude for this card.

#### What it contradicts — R-O42 and standing-layer §7.1

Those ratified the opening roster as **LSV(Meadow) + LCV(Tor)**. It is now
**LSV(Meadow) alone**, because the Contact family is the armed family and this
card is its only key — seeding an LCV hands every seat the thing the card is
meant to sell.

**§7.1's own argument survives and is better served.** The opening roster exists
so that at turn 0 a scout, a settler and a hauler are literally the same object
and the long-range observable carries almost no information; inscrutability
early is then structural rather than bought. One design does that more
completely than two. What is lost is `Class::Tor` as a *seeded* name — it is now
authored by the card that unlocks the hull carrying it.

#### Re-measured on §8.13's bed, and three seeds cannot resolve either card

> **Measured inside the card-free opening (T-122)** — same defect as §8.13.
> At legal play the card is inert (ΔW −103 ± 314) and the price is invisible;
> appendix §D.1. `R-WAR15` is resolved there.

`examples/card_table`, same three arms and same fit, re-run against the armed
card:

| quantity | T-120 (card was a price) | T-121 (card has writes) |
|---|---|---|
| Growth card value, `1 − t½ ratio` | +0.1134 ± 0.0592 (n=18, `R²` 0.911) | **−0.0705 ± 0.1447** (n=18, `R²` 0.937) |
| Growth work stock at 600 yr | +1.3405 ± 0.9150 | +0.9117 ± 0.8141 |
| Warfare card value, `1 − t½ ratio` | −0.0371 ± 0.1609 (n=9, `R²` 0.791) | **−0.4961 ± 0.2842** (n=8, `R²` 0.755) |
| Warfare `ΔW_i` at 600 yr, colonies | −32.879 ± 46.075 (n=18) | −29.364 ± 39.010 (n=18) |

All estimates, not bounds. **`n` counts seat-seeds over 3 independent seeds**,
six seats sharing each galaxy, so every standard error here is optimistic by an
unmeasured factor — and the Warfare row is computed on the 8 of 18 seat-seeds
whose contrast was positive at both ends, a set selected by the quantity being
measured (R-TREE9).

**Read against 2 SE, this bed resolves neither card.** Warfare's −0.4961 is
1.7 SE from zero and its raw `ΔW_i` is 0.75 SE; Growth's −0.0705 is 0.5 SE
while its work stock is 1.1 SE positive. The specific missing measurement is
seed count: three independent replicates against error bars this wide.

**The two columns are not the same bed and must not be differenced.** The
default Design moved in this landing, so the `Pass` and `GrowthOnly` arms both
moved with it; only the within-column comparison is paired. The Growth row
changing sign across the columns is that, not a result about the Growth card.

**The card's writes reach the simulation, and by what path is `R-WAR15`.** Two
of the three have a reason to be nearly inert: an LSV and an LCV share a
`cost_fraction` so the survey hull's *price* is unchanged, a scout carries no
cargo so the hold geometry does not bind, and `hull_thrust_to_mass` is read only
by `arena`/`combat` — while §8.3 already established that the General Contact
colonizer rung is unreachable under `CheapestViable`, because a Medium hull
always wins on price. No mechanism is established here and none is asserted.

#### Three duplications this removed, because they are the failure mode

`scout_hull` was public and read in three places — the build branch, the
affordability test, and `launch_survey` — which is the shape T-117 named and
T-116 paid for (the engine bought a hull and discarded it because the build site
and the spawn site disagreed). It is private now, and:

- **`Standing::scout_order()` is derived from `design_for(Role::Scout)`** rather
  than written beside it, so the hull the yard is charged for and the hull
  `role_of` reads back are the same answer to the same question.
- **`launch_survey` asks `design_for`** instead of the write.
- **`ProductionContext::price_of` maps the whole Limited tier explicitly**, and
  says in a comment that `picket_cost` and `light_vehicle_cost` are numerically
  equal only because today's ladder gives the tier one price (R-O64/R-L0 open).

#### The same merge broke a build order, and the fix is the same shape

Ordering a picket by **naming its hull** — `hull_order(HullType::...)` — stopped
working the moment the picket and the armed scout came to share a shell:
`hull_order` maps a hull to the one class this policy names for it, stamps
`Tor`, and `role_of` reads that back as a **scout**. A yard paying for one thing
and receiving another is T-116's defect exactly.

`Standing::order_for(role)` is the fix, derived from `design_for` the same way
`scout_order` is, and `scout_order` is now one line of it.
`a_build_order_round_trips_to_the_role_that_asked_for_it` asserts the round trip
under both standing layers **and pins the defect itself** — that ordering a
picket by shell still reads back as the armed scout — so if the collision ever
disappears the rule loses its reason visibly rather than silently.

`hull_order` survives for the mining pair and the colonizer ladder, where the
caller is choosing a hull by price and the role is not in question. Its doc
comment now says so.

**Found by enumeration, not by a test failing.** The defect sits behind
`picket_reserve > 0` or `picket_claims_target`, both of which default off and
neither of which `DoctrineWrite::ArmedFrontier` writes — so no bed reaches it
and the suite was green across the change.

#### And the class now disambiguates on both sides of the write

Before, the class separated a scouting hull from a picketing one **only** when
the armed write was on. Now it works unarmed as well: a scouting LSV is `Tor`
and a mining LSV is `Meadow`. That makes `ASSIGNABLE`'s **order** load-bearing —
`role_of`'s second pass takes the first role that *mounts* the hull, and two
roles now mount `LimitedSystems`. Miner precedes Scout, matching
`competent_role`, so the two resolutions cannot disagree. Nothing
production-built reaches that pass (every build stamps a class);
`every_hull_has_a_role_under_every_doctrine` does.

---

### 8.15 At legal play the first Warfare card has no path to `W` (T-122)

*The measurements are appendix §D.1. This section carries what they decide.*

**Decided — the card is inert at legal play.** Played at the round-0 barrier
(`years_to_first_round`, 200 yr — the opening is card-free by protocol),
`TIER0[15]` moves `W_0` by an amount indistinguishable from zero, and the
0.5 kt price is invisible. The effect §8.13 and §8.14 measured was the price,
paid inside the opening where no game plays a card.

**Decided — the reason is structural, and it is four gates in series.** Each
was closed by ablation, not argued:

| # | gate | where | measured |
|---|---|---|---|
| 1 | **Arming changes no fight.** Only two things ever reach combat: miners sharing a rock (`hostile_contact_at` walks `mine_crew`) and a picket defending held ground (`resolve_picket_fight`). A scout, a colony ship or an armed LCV in transit never does | `sim.rs` | the colonizer write is bit-identical on 8/8 seeds; the scout write is noise |
| 2 | **The initiator always loses** (R-WAR5). Every engagement is the arena's ROU duel with the defender on the lasers | `sys_engagement` | with hostility: +1,955 engagements, **+2,960 hulls lost, +0.6 killed**; work-years −1.79 ± 0.07 |
| 3 | **Rock fights cannot move `W` even when won.** Miners do not bind expansion (§7.3: an empire loses its mining fleet several times over and does not notice) | the economy | — |
| 4 | **Held-ground fights never happen.** Pickets are fielded (278 per run) and intercept (497 per run); no enemy colony ship ever reaches ground they hold. Not the guess — handed the true destination, pickets find 18 feasible intercepts and still cause no fight | `offer_interception` | 0 fights, 0.1 diversions, oracle or not |

**So a Warfare card that moves `W` substantially needs a mechanic the engine
does not have.** Tuning cannot reach it: every magnitude on the paths above
was varied, and `picket_reserve` 8 → 32 is bit-identical.

**`OPEN` — which mechanic (`R-WAR16`).** Three candidates, each named by the
gate it removes, and they are different games:

- **Weapons follow the hull (gate 2).** A Systems hull reserves no payload
  (§2.3's `b = 0`), so it cannot fire; an armed hull fighting an unarmed one
  wins. Slot-organic (design law #2) and uses geometry already ratified. Alone
  it moves only rock fights, which gate 3 says do not matter.
- **Armed hulls strike colony ships (gates 1 and 4).** The expansion loop's
  input is the colony ship; a Contact hull that meets one takes it. Needs a
  meeting site the engine lacks — §8.9.5 found in-transit interception
  *narrower* than racing to the destination, so this is a posture question
  (where the armed hull stands), not a kinematic one.
- **Population strike (Warfare's license, card contract §10).** T-107's
  staffing factor now exists (`SimConfig::population_staffs_industry`, off by
  default), so killing a colony's people finally costs its owner output —
  the prerequisite §8.1 named. `W` counts colonies, so this reaches it only
  through the rival's slowed expansion.

**What would settle it:** the author's choice of mechanic, then the same
asymmetric bed (appendix §D.1) with the card carrying it. **Settled at T-123:
armed hulls strike colony ships, at the port (§8.16).**

**Decided — two ablation knobs, never design options.**
`SimConfig::ablate_oracle_intercept` hands a picket the true destination (a
design-law-#15 violation on purpose); `SimConfig::ablate_color_conjunction`
bills a rung in the paying bank's own mix. Each is pinned by a test asserting
it changes exactly the one thing it names.

### 8.16 The port strike — armed hulls meet colony ships where they launch (T-123, R-WAR16 resolved)

*The measurements are appendix §D.2. This section carries what they decide.*

**Decided — the meeting site is the rival's port, not the colony ship's
destination.** A colony ship picks one of thousands of worlds, and §8.15 showed a
picket cannot meet it there even when handed the answer. Its *origin* is one of
the rival's production centers, and a hull standing there meets it at range
zero: nothing to guess, no light-lag to lose to, and no window to be too slow
for. `examples/launch_census` priced the reach before anything was built — a
seat launches from a mean of **213.6** origins after the round-0 barrier, and
its **eight busiest carry 25.5%** in hindsight (appendix §D.2), which bounds
what eight well-placed hulls per rival could meet.

**Decided — the mechanism** (`Doctrine::picket_blockades`, `src/sim.rs`):

| step | what happens | reads |
|---|---|---|
| see | every colony-ship launch is seen at a blockading seat's capital `distance / c` after the drive lights (`EventKind::LaunchSeen`) | light only — design law #15 |
| place | a picket built under the write goes to the rival port this seat has **seen launch the most**, excluding ports it already holds or has a hull flying to (`blockade_target`) | `launches_seen`, its own store |
| hold | on arrival at a port the rival still owns, the hull stands there and counts against `picket_reserve` | — |
| strike | a colony ship launched from a port with a hostile hull standing at it fights that hull at the yard, before its leg is scheduled (`strike_at_port`) | range zero |

The fight is the picket's own (`fight_at`, shared with `resolve_picket_fight`):
the blockader is on station and takes the defender's side, which **R-WAR5's
convention decides** — the arriver always loses. The hull, its settlers and its
cargo become slag at the port (design law #11; `mass_is_conserved_through_the_blockade`).

**Decided — `TIER0[15]` carries it.** `DoctrineWrite::ArmedFrontier` now also
writes `picket_blockades`, `picket_claims_target` (the supply branch that lets a
center saving for a world build a picket ahead of survey, §8.8) and
`picket_reserve ≥ ARMED_FRONTIER_BLOCKADERS`. **8 is a placeholder magnitude**:
played at the round-0 barrier, 8, 16 and 32 score within a tenth of a standard
error of each other, so the reserve is not what binds (appendix §D.2).

**Measured — the card reaches `W`.** On the asymmetric bed, played at the
round-0 barrier: **ΔW +24,024 ± 8,509 colony-years (t 2.82, 7/8 seeds)**; rival
colonies at the horizon **−102.6 ± 25.8 (t −3.98)**; seat 0 kills **306 ± 84**
colony ships and loses none. **On the twelve-seat table, over 11 independent
galaxies: `ΔW_i` at 800 yr +28.26 ± 5.37 colonies (t 5.26), `∫ΔW_i dt` +8,817 ±
1,713 colony-years (t 5.15), positive on 11/11** (appendix §D.2).

**`OPEN` — three things the mechanism does, stated so they are not mistaken
for tuning:**

- **A struck ship's destination stays marked** in its owner's `targeted` set,
  because that set is monotone by construction (T-101's candidate-scan prune
  depends on it). So a strike also withdraws one world from the launcher's
  candidate list for good. A colonizer that dies at a picket already did the
  same. Whether a kill should forfeit the target is a design question
  (**R-WAR17**); the measurement cannot separate the two effects today.
- **Placement counts every launch ever seen, with no recency.** Launch origins
  move as colonies become centers, so a cumulative count favors old ports. Written
  from `t = 0` instead of at the barrier the same writes score **t 4.06** (reserve
  8) and **t 5.57** (reserve 32), which says the hulls placed early — at the ports
  that launched first — are worth more than the ones placed late. A recency rule
  needs a constant, and the engine has no ratified one (**R-WAR18** — implemented
  at T-125 with the picket's reassessment cadence, and measured null, §8.17.5).
- **The rival has no response.** Nothing in the engine lets a center decline to
  launch past a blockader, or send anything to lift one. Mechanism before policy:
  the channel exists and no policy uses it, so a flat result for a defensive
  card later must not be read as proof that a blockade cannot be answered.

### 8.17 Weapons are a Design, not a side (T-125)

*Author's specification: "Load out is a function of a Design and fixed at ship
construction. Lasers versus missiles has nothing to do with aggression."*

**Decided — a hull's weapons are part of its Design and are fixed when it is
built.** `design_loadout(hull, class)` is read once, at construction, and
stamped on the hull (`World::loadout`); nothing re-reads it afterward (design
law #12, no retroactive refits). Which side arrived first, which side is on
station, and which side started the fight decide **nothing** about what either
ship can shoot. **This retires R-WAR5's convention for every fight the
simulation resolves** — the rock fight, the picket's defense and the port
strike all call `combat::resolve_beam_engagement`, in which each ship fires what
it mounts. The arena keeps its laser-side-vs-missile-side resolver, because that
is the sweep `CombatConfig`'s tuned constants were calibrated on and
`tests/balance.rs` pins it bit-for-bit.

**Decided — damage accumulates against structure, and fire is simultaneous.**
A hull dies when the energy it has absorbed reaches its structure. Every shooter
aims at the ships standing at the start of the tick, so neither side shoots
first by index (§2.3, no initiative). **How structure and damage are sized is
§8.18 (T-132)**, which replaced "structure is dry mass" and a per-tick shot
energy; the replaced model is appendix §D.10.

#### 8.17.1 The Design space — what a weapons Design specifies

| family | field | unit | what it sets | engine |
|---|---|---|---|---|
| all | **structure** | kJ | hull volume `r³` × the Design class's `σ` (§8.18, T-133) | **built** — placeholders `10¹¹` (Systems Designs: Meadow, Tor, Delta, Range, Ford) and `10¹²` kJ per hull unit³ (Cairn, Scarp, unnamed armed hulls) (R-WAR19) |
| all | **fire distances** | ly | max distance to fire upon an enemy and upon a neutral; Doctrine may hold fire on either (§8.19) | **built** — placeholder `0.01` for both (R-WAR27) |
| **beam** | **mounts** | count | `b_role · V` over one Limited Contact hull's, floored, at least one — design law #2's slot-organic count | **built** — LCV 1 |
| beam | **power** | MW | energy per unit of time on target; a tick delivers `P · dt` (§8.18) | **built** — placeholder `50 MW` (R-WAR19) |
| beam | **fire-control error** | ly | the tolerance `laser_hit_check` compares predicted against actual target position; smaller is more accurate | **built** — reads `CombatConfig::laser_hit_tolerance` (tuned), not a second copy |
| beam | targets per tick | count | one per mount — a beam points one way (§8.18.3) | **built**; the arena's `laser_shots_per_tick` is no longer read by the simulation |
| beam | range falloff | kJ per ly | §2.4's "weak at long range" | `OPEN` — every simulation fight today is at range zero |
| beam | point defense | — | whether a beam may target an in-flight missile | arena only; `OPEN` for the simulation |
| **missile** | tubes | count | slot-organic, as beams | `OPEN` |
| missile | **acceleration** | multiple of the carrier's, or absolute ly/yr² | how fast it closes | arena only (`missile_accel_multiplier`, tuned) |
| missile | fuel | yr | guided burn time | arena only |
| missile | launch Δv | ly/yr | separation at release | arena only |
| missile | warhead | kJ | damage per hit | `OPEN` — arena missiles kill on contact |
| missile | magazine | count, and its mass | expendable rounds; expended ordnance leaves the fleet lighter (design law #11) | `OPEN` — R-O60/T-04 |
| missile | burst, guidance cap | count | salvo shape; channel limit on missiles in flight | arena only |
| **torpedo** | as missile, heavier warhead, arming distance | kJ, ly | §2.4's long-range heavy hit, weak close | `OPEN` |
| **pulse** | burst energy, knife range | kJ, ly | §2.4's point-blank burst | `OPEN` |

**Only the beam family is built, and only the Warfare card builds it.** A
Systems hull's payload fraction is zero — cargo *is* its payload — so every hull
an empire builds without the card is **unarmed**, structurally rather than by a
flag: `only_the_warfare_card_arms_a_hull` walks every role and the colonizer
ladder of the default standing layer and finds no mount, then plays the card's
writes and finds beams on the scout and the picket.

**Consequence, stated so it is not read as a regression:** every miner is a
Systems hull, so a rock fight between two empires is now two unarmed crews and
kills nothing, hostile doctrine or not. The T-122 measurement that hostility
cost its own player −1.79 work-years was R-WAR5's convention handing the
arriver missiles it never built.

#### 8.17.2 The Doctrine space — how a Design's weapons are used

Doctrine decides **when and where** an armed hull fights; it never changes what
it mounts.

| write | decides | engine |
|---|---|---|
| `engage_neutrals` | hostility — whether a neutral is an enemy at a shared rock | built |
| `picket_blockades` | posture — stand at the rival port seen launching most, strike what leaves | built (§8.16) |
| `picket_after_founding` | an **armed** colonizer keeps its hull after founding and goes to the frontier as a picket; an unarmed one still becomes its colony's stock | built (§8.17.3) |
| `picket_intercepts` | race a seen launch to the guessed destination | built (§8.12) |
| `picket_claims_target`, `picket_reserve` | supply — how many armed hulls to keep on station | built; reserve a placeholder |
| target priority | which enemy a shooter takes first | built as *nearest not already doomed*; alternatives `OPEN` |
| disengage | break off on believed kinematics (R-O41) | computed and logged, not acted on (T-10) |

#### 8.17.3 Armed colonizers hold the frontier after founding

*Author's specification: "Armed Colonizers after founding a colony should move
to the frontier in a picket role."*

**Decided.** `DoctrineWrite::ArmedFrontier` now writes `picket_after_founding`.
One predicate, `stays_armed_after_founding`, decides both halves — whether the
hull is credited as the colony's stock and whether it is dispatched — and it is
true only for a hull whose Design mounts weapons. So a General Contact colonizer
keeps its hull and leaves the colony founded without its recycled stock (the
§8.10 cost, now paid only by armed hulls), and a Medium Systems colonizer still
recycles.

**Decided — the frontier is the rival's port once one has been seen, and the
edge of the empire's own expansion before that.** A freed armed hull goes where
a new blockader would (§8.16's placement); with no port seen yet, it goes to the
nearest unclaimed world its empire has scanned (§8.6's rule).

T-116's −287.8 `W_0` for this write was measured with every colonizer keeping
its hull and nothing able to be struck; it is superseded, not answered. The
twelve-seat measurement is appendix §D.4.

#### 8.17.4 Pickets stack, like mining crews

*Author's specification: "Pickets can accumulate at more than one per site,
like miners."*

**Decided.** Held ground is `world → (holder seat, stack, since)` and a blockade
is `(port, seat) → stack`. Every hull in a stack fights; a rival's hull cannot
join ground another seat holds; an interception or a move takes one hull and
leaves the rest (`detach_picket`, `leave_blockade`); the reserve counts hulls,
not sites (`pickets_stack_on_held_ground_like_a_crew`).

**Decided — cover every seen port before stacking.** One armed hull destroys any
unarmed colony ship, so a second at the same port adds nothing against what most
ports launch. **That premise is false since T-132** for Medium and General
colony ships, which a lone Limited picket cannot finish in one engagement; the
rule stands until R-WAR21 decides what replaces it (§8.18.6). A new blockader therefore goes to an **uncovered** port this seat
has seen launch, ranked by launches seen in the last `intercept_reassess_years`
and then by all launches; only when every seen port is covered does it stack, on
the port with the most recent launches per committed hull. And the blockade-first
branch below builds ahead of expansion **only while a seen port is uncovered**.
Both rules were measured against the alternatives (appendix §D.4): allocating
purely by launches per hull halved the card, and building the reserve out past
coverage cost it 42%.

#### 8.17.5 The blockade's supply and timing

**Decided — three writes and one recall, each with a measured reason** (appendix
§D.4's census, which splits coverage by century):

| rule | what it does | why |
|---|---|---|
| `picket_first` (Doctrine, the card sets it) | a center builds a picket **before** any other order while its empire's pickets — standing *and* in flight — are short of the reserve and a seen port is uncovered | as a fallback the blockade had **no hull on station for the first century** after the card, through the rivals' fastest expansion |
| `recall_seen_launches` | a seat that starts to blockade records every launch whose light has already reached its capital, and schedules the rest | a record that starts empty at the barrier waits a light-crossing before it can place anything; everything recalled is something light had delivered (design law #15) |
| recency (`BlockadeReassess`) | ports rank by launches seen in the last `intercept_reassess_years` (reused, R-WAR12); a blockader whose port launched nothing seen in that window moves to the best uncovered one | origins move as colonies become centers. **Measured null** — kept because it costs no constant and is the rule the census says should matter once latency is shorter |
| the reserve counts hulls in flight | `pickets_held` adds `blockaders_in_flight` | otherwise the blockade-first branch builds one hull per decision while the first is still traveling |

**`ARMED_FRONTIER_BLOCKADERS` is 128** (placeholder, R-WAR19): the card's effect
rose from reserve 8 to 128 and stopped (8 → 32 → 128 → 512 read +0.061 / +0.092
/ +0.116 / +0.118 on the asymmetric bed before stacking).

#### 8.17.6 Where the card stands against the author's target — `OPEN` (R-WAR20)

*Author's target: 1.5x to 2.0x the key tree metric at P92 on the standard
twelve-seat bed. For Warfare the metric is the integral of the seat's colonies
over the rival mean, `S_i = ∫ C_i / mean_{j≠i} C_j dt`, card against pass.*

**Measured: the card does not reach it**, and the reason is measured rather than
guessed. Twelve-seat result in appendix §D.4.

- **The mechanic can reach it at sufficient coverage.** A coverage oracle
  (`SimConfig::ablate_strike_fraction`, ablation only) that destroys a fixed
  fraction of rival launches from the barrier on gives `ln S` **+0.265 at 25%,
  +0.567 at 50%, +2.61 at 100%** on the asymmetric bed — so roughly **30–45% of
  rival launches struck** is the band the target asks for.
- **The port strike reaches ~10% in the century that matters.** Rival expansion
  peaks at 300–400 yr; the card lands at 200 yr; a blockader must see a port
  (light, `distance` years), be built, and fly there (`distance` years again).
  At reserve 128 the census reads **10.8% coverage in 300–400 yr with 3.8 hulls
  on station and 14.9 in flight**, then 28.6% in 400–500 yr, after the peak.
  Hull count, placement recency and the pre-card record each moved the result
  by less than its standard error; supply priority moved it by a quarter.
- **So the binding constraint is transit latency against the expansion clock**,
  which is physics (`c = 1`) and protocol (`years_to_first_round = 200`), not a
  magnitude of this card.

**What would settle it — the author's choice** among changes that are not this
card's to make: an earlier first barrier (R-P12, `years_to_first_round`); a
strike that reaches a colony ship somewhere other than its port or its
destination; a Warfare metric or target that accounts for a card whose effect
cannot begin until a light-crossing after it is played; or accepting this card
below the band.

### 8.18 The damage model — power over time, structure on volume (T-132)

*Author's specification: "The damage model is broken and must be fixed. I think
kilojoules is a reasonable unit. I think damage per tick is not reasonable…
The duration needs to be scaled to allow for Design improvements and hull
distinctions. Additionally, I don't understand the math behind the structure kJ
rating. That is suspect. Realism is nice, and helps players develop intuition
about the game, but it must yield to the requirements of fun. We have to change
the damage model."* Asked what structure should scale with, the author chose
**hull volume**.

This replaces §8.17's "structure is mass" and the per-shot energy and fire rate
in §8.17.1. What the old model was and what it got wrong is appendix §D.10.

#### 8.18.1 Terms

| symbol | name | unit | where it is set |
|---|---|---|---|
| `P` | a beam mount's **power** while it is on target | MW; kJ/yr in the engine | `CombatConfig::beam_power_mw`, placeholder |
| `σ` | **structure per unit of hull volume** | kJ per hull unit³ | `CombatConfig::structure_kj_per_hull_unit3`, placeholder |
| `r³` | a hull's enclosed volume | hull unit³ | `HullType::hull_volume` (R-PROD5's quantity) |
| `S` | a hull's **structure**, `σ · r³` — the energy that wrecks it | kJ | `combat::hull_structure_kj` |
| `dt` | the engagement's integration step | yr | `engagement_dt_years`, 0.0005 — a numerical requirement (§2.6) |
| `H` | the engagement horizon | yr | `engagement_horizon_years`, 0.5 — placeholder (R-WAR5) |
| `τ_L` | time for one mount on target to wreck a Limited Contact hull, `σ · r³_LCV / P` | days | derived |

#### 8.18.2 `RATIFIED` — the author's rulings

- **Energy is in kilojoules.**
- **Damage is not denominated per tick.** A mount's output is a power, and a tick
  on target delivers `P · dt`. Halving `dt` halves each tick's damage and
  doubles the ticks, so a fight lasts the same time
  (`damage_is_a_power_and_the_kill_time_does_not_depend_on_the_step`). Under the
  replaced model damage was per tick, so the fight's length in years was set by
  the integration step.
- **A fight's duration is scaled so Design improvements and hull distinctions
  can act.** A fight decided in one tick cannot tell two Designs apart.
- **Realism yields to fun.** The magnitudes are chosen for how a fight plays,
  and are stated in physical units so players can build intuition from them.
- **Structure scales with hull volume**, `S = σ · r³`. Durability is a value,
  and design law #3 makes volume the value basis — as it already is for beam
  mounts (payload volume) and for the hold.
- **`σ` is per Design class** (T-133, the author's ruling), so structure is a
  property of the Design and a card can write it. A hull with no named Design
  takes its hull class's default.

#### 8.18.3 Decided — how a tick resolves

Every mount aims at the nearest enemy not already doomed by damage landing this
tick, and engages one target per tick: a beam points one way. The fire-control
test (`laser_hit_check`) depends on the shooter, the target and the time, so it
is taken once per target per tick and holds for every mount aimed there; a miss
on the nearest undoomed target wastes the shooter's remaining mounts that tick.
Fire stays simultaneous (§2.3). The resolver reports how long the fight ran.
**The arena's `laser_shots_per_tick` is no longer read by the simulation's
resolver** — a per-tick rate is exactly the denomination this section removes —
and the arena, which still reads it, is unchanged (`tests/balance.rs` passes).

#### 8.18.4 `OPEN` — R-WAR19: the magnitudes

**Placeholders:** `P = 50 MW`, `σ = 10¹² kJ per hull unit³`. A Limited Contact
hull's structure is then 41 TJ and `τ_L` is **9.5 days**. Only `P / σ` reaches an
outcome; `σ` is set so structures read in terajoules and the beam in megawatts.

**The criterion they were chosen against, which is the part to ratify:** in the
equal-spend, point-blank round robin (Technology §4), mirror fights last **28 to
82 ticks** — room for a small Design difference to act — and the slowest fight
between the smallest hulls ends well inside `H` (1,000 ticks). Measured in
appendix §D.10.

#### 8.18.5 What it does, measured (appendix §D.10)

- **The short-range round robin discriminates**: most pairings are decided one
  way, where every one used to destroy both fleets (Technology R-TECH14,
  resolved).
- **Design law #2 in the engine:** one GOU beats 40 ROUs and loses to 50 on all
  three seeds; one ROU beats 10 LOUs and loses to 14. The law's target is 6–45
  ROUs per GOU, so the crossover sits at or slightly past its upper end.
- **Mirror matches are decisive, not draws.** Identical fleets end with one side
  holding up to 164 of 500 hulls, decided by which fleet's station-keeping
  geometry is easier to hit. A rating needs many seeds with sides swapped.
- **A lone Limited picket cannot finish a Medium colony ship**: one mount needs
  254 days against a 183-day engagement (R-WAR21). On the twelve-seat card bed
  at 450 yr, hulls destroyed fell from **1,580 / 2,030 / 2,798 to 356 / 127 /
  369** (seeds 1 / 7 / 42) and total colonies rose by **182 / 250 / 220**. The
  first Warfare card's measured effect (§8.17.6) is from the replaced model and
  has to be re-measured (R-WAR20).

#### 8.18.6 `OPEN` — R-WAR21: the blockade against a Medium colony ship

> **Reframed by §8.19 (T-133).** The 183-day engagement this item measures
> against is to be deleted; under an encounter the exposure is kinematic, and a
> lone Limited mount wrecks an arriving Medium colony ship with 0.047–0.093
> (§8.19.5, appendix §D.11). The question — how a blockade kills one — stands.

A Medium Systems hull encloses 26.8 times a Limited Contact hull's volume, so
structure on volume makes it 26.8 times as hard to wreck while it costs 5.5
times as much. The consequence is the blockade: **§8.17.4's rule "cover every
seen port before stacking" rests on "one armed hull destroys any unarmed colony
ship", and that is now false for every Medium and General colony ship.**
Candidates, none chosen:

- **Accept it.** A blockade must stack three Limited pickets to kill a Medium
  colony ship in one engagement, and the covering rule is rewritten to stack to
  a lethal count before covering the next port.
- **A longer engagement.** At `H = 1 yr` one mount finishes a Medium colony ship
  in 0.70 yr. `H` is also a cost: a fight nobody can finish runs to it.
- **A stronger beam.** At `P ≳ 70 MW` one mount finishes it inside 0.5 yr with
  every tick a hit — and every equal-spend fight shortens by the same factor,
  which is the resolution §8.18.4 was chosen to buy.
- **Structure on the hull less its hold.** Keeps armed hulls as they are and
  makes cargo space not armor, which departs from "hull volume" as ruled.

#### 8.18.7 `OPEN` — R-WAR22: damage does not persist past an engagement

> **Reframed by §8.19 (T-133):** an encounter ends in a wreck roll, so the
> question becomes whether a hull that survives the roll carries its damage into
> the next encounter.

A hull that survives a fight leaves it undamaged: the resolver's damage is local
to one call. Under the replaced model nearly every fight ended in its first tick,
so this never showed; now a fight that runs out `H` discards partial damage, and
a picket that cannot finish a hull in one engagement can never finish it. *A
decision for the author:* persistent damage (and then repair) is new state on
every hull.

### 8.19 Fights happen where trajectories meet, and Doctrine decides them (T-133)

*Author's specification: "Engagements are not part of the design, but rather
cruft to be eliminated. Unless both sides have a Doctrine to kill the enemy
fleet, pitched battles such as the engagement described are not the common
outcome. An unarmed ship should be trying to run away or to complete its
mission, not locked into an engagement for half a year. Just because a ship is
fired upon doesn't mean it submitted to an engagement. The whole point of the
wreck roll is to have combat and transport be resolved together. In the case of
picket vs colony ship, they should be approaching the site at speed and the
colony ship should try to colonize before destruction."* And: *"The point is
fights can happen anywhere, depending on Doctrine. The sim cannot hardwire fight
sites."* And: *"Each Design/hull/class/role should have a 'Max distance to fire
upon an enemy' and a 'Max distance to fire upon a neutral'. Either or both may be
ignored by Doctrine."*

**Built so far (T-133 stages 1–3, and an interim stage 5):** the wreck roll
with its threshold, the pass resolver, the fire distances and hold-fire, and
per-Design-class structure. A colony ship leaving a blockaded port or arriving
at a picketed world now flies through the fire and takes a wreck roll; a
survivor flies on, or founds. **Not built: stage 4, detection.** Those two
places are still where the engine looks for fire, and a shared rock is still a
stationary pitched battle — now only when both crews' Doctrine is hostile.
Until stage 4, fights happen nowhere else.

#### 8.19.1 Terms

| symbol | name | unit | where it is set |
|---|---|---|---|
| — | **encounter** | — | an interval during which a hull whose Doctrine fires on an enemy hull is within beam reach of it, each on its own trajectory |
| `R` | **beam reach** — the separation beyond which a mount cannot hit | ly | `OPEN` (R-WAR23): fire control against station-keeping gives hits out to 3e-3 ly reliably and thinning by 1e-2 ly (appendix §D.10) |
| `d_enemy` | **max distance to fire upon an enemy** | ly | per Design/hull/class/role; placeholder values `OPEN` (R-WAR27) |
| `d_neutral` | **max distance to fire upon a neutral** | ly | per Design/hull/class/role; placeholder values `OPEN` (R-WAR27) |
| `E` | **exposure** — the time an encounter lasts | yr | kinematics: both trajectories, and the smaller of `R` and the applicable fire distance |
| `D` | energy a hull absorbed during the encounter | kJ | `P · dt` per mount per tick on target (§8.18) |
| `S` | the hull's structure, `σ · r³` | kJ | §8.18 |
| `p₀` | wreck odds of a barely-scratched hull | probability | placeholder, R-WAR24 |
| `x½` | the damage ratio `D / S` at even odds | ratio | placeholder, R-WAR24 |
| `P(wreck)` | `p₀ / (p₀ + (1 − p₀)·e^(−κ·D/S))`, with `κ = ln((1 − p₀)/p₀) / x½` | probability | §2.2's logistic, in closed form |

#### 8.19.2 `RATIFIED` — the author's rulings

- **There are no engagements.** A fight is not a place two fleets stop at and
  not a span of time either side commits to.
- **There are no fight sites.** Any hull can be fired upon anywhere its
  trajectory takes it within reach of a hull whose Doctrine fires on it. The
  engine may not decide *where* fights happen; Doctrine decides *whether*, and
  trajectories decide *where and for how long*.
- **Being fired upon is not consent.** A hull under fire keeps doing what its
  mission says: flying its voyage, arriving, founding. An unarmed hull runs or
  completes its mission; it is never held in place by a fight.
- **A pitched battle needs both sides.** Two fleets stand and fight only when
  both sides' Doctrine is to kill the other's fleet.
- **The wreck roll resolves combat and transport together.** It is the outcome
  of an encounter: the hull either continues its trajectory and mission, or is
  wrecked where it is.
- **Picket against colony ship:** both approach at speed; the colony ship tries
  to found its colony before it is destroyed.
- **Every Design/hull/class/role carries two fire distances**: a maximum
  distance to fire upon an enemy (`d_enemy`) and a maximum distance to fire upon
  a neutral (`d_neutral`). **Doctrine may ignore either or both, and ignoring
  one means holding fire on that kind of target at any range** (R-WAR27,
  resolved).
- **No wreck roll until a hull has sustained a threshold of damage**
  (R-WAR24, resolved in form). Below it the hull is not defeated and flies on.
- **A pitched battle ends by any of three rules** (R-WAR26, resolved in form):
  one side has no hull left; a hull past the threshold rolls, and survivors
  withdraw; a side's Doctrine breaks off on believed kinematics (§5).

#### 8.19.3 `OPEN` — R-WAR25: the encounter, as recommended

- **Fire.** For every tick of an encounter, each mount whose Doctrine fires on
  a hull within `R` aims at the nearest such hull not already past its
  structure, and delivers `P · dt` if fire control holds against both hulls'
  actual trajectories (§8.18.3's rule, with both ends moving).
- **Resolution.** When the encounter ends — the target leaves `R`, arrives and
  completes its mission, or the shooter leaves — every hull that absorbed energy
  takes one wreck roll. Wrecked: destroyed where it is, its mass slag there
  (law #11). Survived: continues its trajectory and mission unchanged.
- **Founding is the end of the colonizer's encounter.** A colony ship that
  survives the roll at arrival founds, whoever is on station — which removes the
  turn-back on "both survive" (§8.9).
- **Detection is arrival-driven** (`CLAUDE.md` §4). When a hull starts a
  trajectory or takes station, the engine finds the intervals in which it comes
  within `R` of a hostile hull that fires on it, or that it fires on, and
  schedules an encounter event for each. A stationary shooter against a hull on
  a straight voyage is a closed-form test — the distance from a point to a line,
  then the flight-distance inverse for the times. Armed hulls are the few, so
  the check is `O(armed hostile hulls)` per departure, not `O(hulls)`.
- **Who fires on whom, and from how far, is one standing-layer question**
  (`CLAUDE.md` §6): `Standing::fire_distance(hull, class, role, their standing)`
  returns the applicable `d_enemy` or `d_neutral`, or nothing where Doctrine
  ignores it. A hull fires on a target within the smaller of that distance and
  the physical reach `R`. Today's scattered writes — `engage_neutrals`,
  `picket_blockades`, `picket_intercepts` — become cases of it and of where
  Doctrine sends armed hulls, not of where fights are allowed.
- **A pitched battle** is an encounter in which both sides fire on each other
  and both sides' Doctrine is to stand. It is the one case that keeps
  `resolve_beam_engagement`, and what ends it is R-WAR26.

#### 8.19.4 What the kinematics already say, before any of it is built

A laden Medium colony ship stays within `R` for **58–105 days** leaving a port or
arriving at a world, and for **2–9 days** passing at 0.82–0.97 c mid-voyage
(appendix §D.11, analytic, one mount always on target as an upper bound).
*Inference:* a slow hull is exposed 20–40 times longer than a fast one, so with
no site hardwired, fights still concentrate where hulls are slow — at departures
and arrivals. That is kinematics choosing the place, which is what the ruling
asks for, and the same result T-115 found for interception.

#### 8.19.5 `OPEN` — R-WAR24: the wreck curve's magnitudes

**Decided (the author's ruling): a hull takes no roll below the threshold
`θ`.** Past it, `P(wreck) = p₀ / (p₀ + (1 − p₀)·e^(−κ·(x − θ)))` with
`x = D / S` and `κ = ln((1 − p₀)/p₀) / (x½ − θ)`, so the odds are `p₀` at the
threshold and even at `x½`. **Placeholders:** `θ = 0.25`, `p₀ = 0.02`,
`x½ = 1` (`CombatConfig::wreck_threshold`, `wreck_odds_at_threshold`,
`wreck_even_odds_damage`). The floor that made a graze a 2% lottery (appendix
§D.11) now applies only to a hull that has taken a quarter of its structure.

**Measured at these placeholders** (appendix §D.12): a lone Cairn picket
delivers 2.45–4.05 structures to a Delta colony ship over the 105 days it is in
reach, leaving or arriving, and the roll's odds round to 0.999–1.000. **A lone
picket now kills a Medium colony ship nearly every time**, where under T-132's
single `σ` it could not kill one at all. The lever is `σ_Delta` (and the fire
distance): the author sets how lethal a lone picket is by setting them.

#### 8.19.6 R-WAR26: what ends a pitched battle — all three, not yet built

**Decided (the author's ruling): all three candidates end a pitched battle** —
one side has no hull left; a hull past the wreck threshold rolls, and its
survivors withdraw; a side's Doctrine breaks off on believed kinematics
(R-O41, §5). **Open:** how they compose when more than one applies in a tick,
and R-L2 (single pass or repeated passes). **Not built:** the engine's one
pitched battle, at a shared rock, still runs `resolve_beam_engagement` to one
side's end or the horizon; its crews are unarmed Systems hulls today, so it
destroys nothing.

#### 8.19.7 Fights run on the event loop, concurrently with everything else

*Author's specification: "All fights galaxy wide have to proceed concurrently
with production and travel. The main event loop needs to process weapons
discharge and course adjustment."*

**`RATIFIED`:**
- **A fight is a sequence of events on the main loop**, interleaved in time
  with every production decision, mining tick and arrival in the galaxy. No
  fight is worked out ahead of time and no fight is resolved in one call.
- **Weapons discharge is an event.** So is **course adjustment**.

**This retires the interim pass resolver** (`combat::resolve_pass`, called by
`Simulation::encounter_at`): it integrates a whole encounter at the moment a
ship departs, including time that has not happened yet — the opposite of
concurrent. It stays only until discharge events replace it.

**`OPEN` — R-WAR25, revised: the event design, as recommended.**

| event | when | what it does |
|---|---|---|
| encounter begins | a hull comes within the farthest applicable fire distance of a hull that fires on it — found when either starts a trajectory or takes station (stage 4) | schedules the first discharge of every hull that fires |
| **discharge** | every discharge period while the shooter is alive and a target is within its fire distance | fire control against the target's position *now*; `P ×` period of energy if it holds; then the next discharge |
| **course adjustment** | when a hull's Doctrine responds to fire, or a defeated hull gets away (§2.1) | replaces the hull's trajectory from where it is and how it is moving; every encounter the old trajectory implied is re-derived |
| wreck roll | R-WAR28 | wrecked, or gets away |

A discharge reads positions at its own time, so a target that has already
arrived, founded, turned back or been wrecked is simply not there; nothing is
cancelled. Simultaneous events are ordered by the queue's existing `(time,
sequence)` rule, so determinism is untouched.

**`OPEN` — R-WAR28: when the wreck roll is taken, and what a survivor does.**
The threshold rule says no roll below `θ`. If the roll is taken the moment
damage crosses `θ`, the odds are exactly `p₀` (2% at the placeholder) every
time, whatever the weapon — a hull is always rolled at the threshold, never
after. Candidates: roll when the encounter ends (the damage then is all the
damage); roll at each further multiple of `θ` with rising odds; roll at the
crossing with odds from the damage rate. And §2.1 says a defeated ship that gets
away *leaves*: does a colony ship that survives keep trying to found, or flee?

**`OPEN` — R-WAR29: the discharge period.** A beam's output is continuous, so
the period is a sampling rate as well as a rate of fire; one per day adds
~54–94% to the twelve-seat card bed's event count, one per combat tick (0.18
days) about five and a half times that (appendix §D.13). *Recommend* a Design
field, so a Technology write can raise a rate of fire.

**`OPEN` — R-WAR30: course adjustment needs flight from a moving start.** The
engine's only leg is rest to rest, and the one course change it makes today (a
picket re-aiming, T-120) starts the new leg *from rest* at the current position —
velocity dropped for free. A course adjustment that is not a free stop needs a
trajectory from a nonzero velocity, which `math.rs` does not have.

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
| **R-WAR9** | **the colonization leg is flown at the rate its own load implies** — `spawn_courier` read `civilian_accel_g · G` before the hold was loaded, so a colony ship flew like an empty hull and R-O32 was closed for the arena but not for this dispatcher. A laden Medium colonizer makes **0.241 ly/yr² against 2.446 empty**. It **invalidates every transit-dependent magnitude measured before it**, §8.6–8.8's arms included (§8.9.6) |
| **R-IND21** | **a hull carries the minerals it was built from** and hands them back in the same proportions — scrap salvage, wreckage, the founding ceiling's overflow. So a hull built from supers or apex drops supers or apex, with no rule beyond the composition itself. `World::hull_minerals`, captured by `Minerals::try_take_total` at the moment the bank pays (T-119) |
| **R-IND22** | **the founding center pays the floor-rung top-up**, so the absorbing-zero guard (§8.7) is a *transfer* rather than mass appearing from nowhere. A parent too poor to pay leaves its child at whatever it could afford — the guard degrades rather than conjuring. **Design law #11 now holds with no exceptions**, card played or not |
| **R-WAR13** | **resolved (T-121)** — `DoctrineWrite::ArmedFrontier` is the Warfare write, and `TIER0[15]` carries it alongside the two Design unlocks. §8.2's *third* write, `picket_after_founding`, stays out: §8.10 measured it at −287.8 `W_0`, so it is R-WAR6's question (§8.14) |
| **T-123** | **a blockader strikes the colony ships its rival launches from a port it stands at**, at range zero, as the defender (R-WAR5). Placement reads only launches whose light has reached the seat's capital. `DoctrineWrite::ArmedFrontier` writes it, with the claim-target supply and a reserve floor of `ARMED_FRONTIER_BLOCKADERS = 8` (placeholder) (§8.16) |
| **T-123** | **`fight_at` is the one fight between hulls standing at a site** — a picket's defense and a port strike both call it, and it is bit-identical to the picket fight it was extracted from (the as-shipped arm reproduces T-122 per seed) |
| **T-121** | **the default standing layer is unarmed at every role, and `TIER0[15]` is the only key to the Contact family.** Scout rides `LimitedSystems`, colonization `MediumSystems`/`GeneralSystems`, and the seeded roster carries one design. The card unlocks LCV(Tor) and GCV(Unnamed) and writes `DoctrineWrite::ArmedFrontier`, which moves survey and the colonizer ladder onto them together (§8.14) |
| **T-121** | **the picket rides `LimitedContactVehicle`**, not `LimitedOffensive`. The Limited tier shares one `cost_fraction`, so §8.6's denial arithmetic runs on an unchanged price; what moves is a nonzero hold and `hull_thrust_to_mass` 2.0 → 1.4. No role mounts an Offensive hull (§8.14) |
| **T-121** | **a card is a bundle of writes** — `Card::effects: &'static [CardEffect]`, applied in slice order — which is what §8.2's *"one Design write, three Doctrine writes"* describes. Seventeen cards carry a one-element bundle (§8.14) |
| **T-120** | **`TIER0[15]` — the first Warfare card — reaches no decision.** Its `UnlockDesign(LimitedOffensive, _)` write lands in the Roster, whose only consumer outside tests is `roster_permits`, which returns `true` unconditionally while `enforce_roster` is off; and `role_hull_type(Role::Picket)` already returns that hull to everyone. The card's whole channel is its 0.5 kt price (§8.13) |
| **T-120** | **the head-to-head bed §8.4 asked for exists** — `examples/card_table`, 12 seats, three arms, cards at earliest legal play, `g` fitted by least squares on `ln X(t)` as §2.4 requires. Growth card value **+0.1134 ± 0.0592** (n=18 seat-seeds over 3 independent seeds, mean `R²` 0.911); Warfare **−0.0371 ± 0.1609**, inside one SE of zero (§8.13) |
| **R-WAR10** | **a picket guesses its target from the observed bearing**, out of worlds *it* has scanned, and re-reads the trajectory as fresh light arrives. Two worlds on one bearing are indistinguishable at range, so a colony ship aimed at the far one puts a picket on the near one for free — **deception is a move now, not a wish** (§8.12) |
| **T-117** | **the standing layer answers; it is not switched on.** `Standing::role_of` is *derived* from `design_for`, so a card that moves a role to a different hull needs no edit in `assign_role` — pinned by `role_of_inverts_design_for_every_role` across every combination of the writes (§8.11). Bit-identical on four seeds |
| **T-117** | **a hold erects infrastructure in the works mix, not in total** — `erectable = eta_works · min_c(aboard[c] / mix_share(c))`, the same per-color conjunction `works_bill` charges a rung with, so a single-color hold erects nothing (§8.11) |
| **T-116** | **the card's negative `W_0` is one cost and it compounds** — the forfeited founding rung. Ablated, the Doctrine write goes −287.8 → **−2.0**, so pickets, engagements and kills are together worth about two colonies (§8.10) |
| **T-116** | **`founding_infra` is `hull_cost`**, so the Doctrine write's price scales with the hull the Design write enlarges — 10.07× on a GCV — which is why the whole card (−491.1) is worse than either half or their sum (§8.10) |
| **T-115** | **there are two responders to a picket, not one** — the home center issues an order and the crew notices, and light reaches a closing hull sooner than a standing observer. Because light outruns a colony ship, *whether the warning arrives* stopped discriminating; **turnover** replaces it, and it is kinematics rather than a constant (§8.9.1–8.9.2) |
| **T-115** | **a picket whose world is colonized returns to the frontier**, off station and off the books in one function — which is the situation T-113's held-ground preference creates (§8.9.3) |
| **T-113** | the per-class candidate reduction (R-O70) is exact only for a consumer reading the argmax of a **class**; a consumer reading the argmax of a *subset* needs its own slot, and got one (§8.8) |
| **T-112** | a colony founded at infrastructure **zero** is an absorbing state, not a price — `employment_rate` returns exactly `0.0` there, so it can never mine or build. A departing picket leaves the ladder's floor rung instead (§8.7) |
| **T-111** | `combat::resolve_engagement` is called from `sim.rs`; the simulation and the arena fight with one model, and the arena still seeds no production |
| law #11 | a destroyed hull's mass becomes **slag at the site** — inert (R-O59), conserved, and never a salvage yield |
| — | no tick-based initiative; `dt = 0.0005 yr`; < 2 ms per tick |

### Open

| Code | Question | What would settle it |
|---|---|---|
| ~~**T-30**~~ | ~~no accept/decline site exists~~ — **closed as stated (T-111).** The round layer had already landed and the engine was producing ~4,200 co-locations per run unremarked; `sys_engagement` now resolves them. R-AC13 and posture cards are **not** unblocked: both need a decision at *range* | done; see §3.4 and §7 |
| **R-IND20** | **whether founding should erect from the hold at all** is a magnitude (`founding_infra_share`, default 0.0, measured flat at §8.8 because a colony ship's hold is nearly all settlers — R-WAR7). The *rule*, that whatever it erects respects the works mix, is ratified at T-117 | a `settler_target` that reserves mineral volume (R-WAR7), then the same census |

| **R-WAR11** | **may a Doctrine write impose a cost that scales with its holder's own activity?** A card has two costs: `Card::cost`, a bounded one-off the engine already charges, and the mechanic's own — here, every colony founded afterwards starting 5.5x thinner, forever. The second cannot be priced, because its size depends on how much the player goes on to expand, and a difference objective charges it twice (§8.10). **Not Warfare-specific**, so it belongs in the card contract | a decision on whether a card's total cost must be bounded at play time |
| **R-IND11** | **`SettlersPerMineral` is worth +394.9 `W_0` unilaterally** on the 4-seed asymmetric bed — more than any card measured so far. `Hyades_industry.md` §1.6 records the policy as *"blocked on R-O74"*, the conjured-settlers violation, and **R-O74 closed at §1.7**. So the block is lifted and the answer may have flipped | a **symmetric** re-measure — this one is competitive, not global, and a default is a global question |
| ~~**R-WAR16**~~ | ~~which mechanic gives the first Warfare card a path to `W`~~ **Resolved (T-123): armed hulls strike colony ships at the port they launch from** (§8.16). The author chose the mechanic; the census chose the site. `TIER0[15]` carries it; ΔW **+24,024 ± 8,509 (t 2.82)** on the asymmetric bed, appendix §D.2 | — |
| **R-WAR17** | **does a colony ship struck at its port forfeit its destination?** Today it does, because `targeted` is monotone and T-101's prune depends on that; so a kill costs the launcher a hull, its settlers and a world (§8.16) | a decision; if no, a non-monotone mark and a re-measure of the prune |
| ~~**R-WAR15**~~ | ~~by what path does the armed card reach the simulation?~~ **Resolved (T-122): at legal play it does not.** The effect §8.14 measured was the 0.5 kt price, paid inside the card-free opening; a pure-price control reproduces it and vanishes at the round-0 barrier. The per-write ablation it asked for was run: the colonizer write is bit-identical on 8/8 seeds, the scout write is noise (appendix §D.1). Superseded by R-WAR16 | — |
| **R-WAR14** | **`Sim::inert_card_plays` counts `NotYetImplemented` only**, so a card writing real state into a component with no live consumer reads as working. It measures which match arm ran, not whether the write reached a decision (§8.13) | a definition of "reached a decision" that a counter can test — the candidate is whether the written component is read on a live path |
| **R-WAR12** | **the two guess magnitudes** — `intercept_cone_radians` (0.15 rad) and `intercept_reassess_years` (25 yr). Neither is physical: the cone sets how wide a guess may be and therefore what a feint is worth, and the cadence sets how long one stays bought. They are the first magnitudes in this tree whose job is to price a **bluff** rather than a kinetic outcome | a bed on which the yomi channel is readable — not `W_0`, which a bluff does not move directly |
| **R-WAR8** | **the two supply writes' magnitudes** — `scout_hull_offensive` (the armed hull takes the survey slot) and `picket_intercepts` (a picket leaves station for a race it can win). Both ship off. Note that the first is **bit-identically inert** until hull types carry differentiated cost (§8.9.7, R-O64/R-L0) | the census arms in `examples/denial_census`; for the scout write, a cost ladder that distinguishes Limited hulls |
| **R-WAR7** | **a colonizer's hold is nearly all settlers**, so erecting a share of it as the new colony's stock moves the median founding not at all and clears the floor rung in 21–22% of foundings at *any* share (§8.8). Loading a colony ship with a mix is a **reservation against the hold** — a change to `settler_target` (R-IND12) — not a share of what is left over | a `settler_target` that reserves mineral volume, then the same census |
| **R-WAR6** | **the denial magnitudes** — the founding rung a departing picket leaves (`Band Empty` shipped, or the mineral endowment instead, §8.7), and what a picket ought to cost. §8.6's arithmetic says a denial bought with a whole colonizer loses at any table wider than two seats, so this is a *design* question before it is a magnitude. **T-113 built the cheaper hull and it did not settle the question**: the hull is fielded 12–23 times a run because its branch sits behind a survey test R-O86 measured a constant `true`, so cost is not what binds (§8.8). The remaining exit is a denial covering more than one world, which needs a spatial object the engine does not have | a blockade over an approach rather than a point |
| ~~**R-WAR5**~~ | ~~which side carries which weapon~~ **Resolved for the simulation (T-125): a ship carries what its Design mounts** (§8.17). Every simulation fight uses `resolve_beam_engagement`. The convention survives only in the arena's laser-side-vs-missile-side sweep, where `carrier_accel` still reads the laser side's first hull — kept because it is what `tests/balance.rs`'s goldens were tuned on | — |
| **R-WAR20** | **the first Warfare card is below the author's 1.5–2.0x P92 target** (measured on the damage model T-132 replaced; re-measure after R-WAR21), bound by transit latency: coverage of rival launches must reach ~30–45% and the port strike reaches ~10% in the expansion peak (§8.17.6) | the author's choice among an earlier first barrier, a different meeting site, a latency-aware Warfare target, or accepting the card below the band |
| ~~**R-WAR18**~~ | ~~blockade placement has no recency~~ **Implemented (T-125), measured null**: ranking by launches seen in the last `intercept_reassess_years` and moving a blockader off a port that went quiet changed `ln S` by less than its standard error, because latency, not placement, binds (§8.17.5) | — |
| **R-WAR19** | **the beam and structure magnitudes** — `beam_power_mw = 50`, and `σ` per Design class since T-133: `10¹¹` for Systems Designs, `10¹²` for armed ones (T-132; the per-tick `50 kJ` shot and `1,000 kJ/kt` structure they replace are appendix §D.10). Only `P / σ` reaches an outcome; one mount wrecks a Limited Contact hull in 9.5 days | ratify the duration criterion in §8.18.4 (mirror fights 28–82 ticks, every fight inside `H`), then the arena once it can seed beam-versus-beam fights between loadouts |
| **R-WAR21** | **a lone Limited picket cannot finish a Medium colony ship** (254 days against a 183-day engagement), so §8.17.4's covering rule has lost its premise and the twelve-seat bed's kills fell 77–94% (§8.18.6) | the author's choice: accept and stack, a longer engagement, a stronger beam, or structure less the hold |
| **R-WAR23** | **beam reach `R`** — the separation beyond which a mount cannot hit. It sets every encounter's length (§8.19). Fire control gives hits reliably to 3e-3 ly and thinning by 1e-2 ly | a derived bound from fire control and station-keeping, or a Design field of the beam family |
| **R-WAR24** | **the wreck curve's magnitudes** — the threshold rule is ruled (no roll below `θ`); `θ = 0.25`, `p₀ = 0.02`, `x½ = 1` are placeholders. At them a lone Cairn picket wrecks a Delta colony ship 0.999–1.000 of the time (§8.19.5) | the author: how lethal a lone picket should be, set through `σ_Delta`, the fire distance and these |
| **R-WAR25** | **the encounter** — fire along both trajectories, one wreck roll per damaged hull when it ends, arrival-driven detection, `Standing::fires_on` (§8.19.3) | ratify, then T-133 |
| **R-WAR27** | **the fire distances' values** — "ignored by Doctrine" is ruled to mean **hold fire at any range**; every beam Design carries `0.01` ly for both, a placeholder | a value per Design |
| **R-WAR28** | **when the wreck roll is taken, and what a survivor does** — at the threshold crossing the odds are exactly `p₀` every time (§8.19.7) | the author |
| **R-WAR29** | **the discharge period** — rate of fire, sampling rate and event cost at once; *recommend* a Design field (§8.19.7) | the author, then a placeholder |
| **R-WAR30** | **flight from a moving start** — course adjustment without a free stop needs new kinematics in `math.rs` (§8.19.7) | engine work |
| **R-WAR26** | **what ends a pitched battle** — all three candidates, ruled; how they compose and R-L2 are open, and none is built (§8.19.6) | engine work, then a pitched-battle bed |
| **R-WAR22** | **damage does not persist past an engagement** — a hull no single fight can finish is never finished (§8.18.7) | the author: persistent damage (and repair) is new per-hull state |
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
| **R-WAR4** | **the armed colonizer's magnitudes** — the Contact colonizer class's drive fraction `φ` (0.02 is the smallest value that makes its cargo-inefficiency claim true, and it is a placeholder), the card's mineral cost, and where in the tree the pair sits once §8.4's dispersion prediction is measured | T-97 first, then a measurement on the asymmetric bed |
| **R-TREE12** | Warfare card value has no doubling time of its own; decided as the fractional *increase* in the **target's** | trees §5; the target set is still open |
| R-WAR3 | `w_ij`, the neighbor weight | a decision; the objective needs it |
| R-AC13 | "if pressed" at the colonization layer | blocked on T-30 |
| — | nothing moves `bio_max`, so the durable target is untouchable | a Warfare card, and the guard in §4.2 |

---

## References

- `Hyades_simulation_model.md` §4 (deterministic combat, the wreck roll), §5
- `Hyades_loadout.md` §3.2 (the four weapon families), §3.3 (defense), §5
- `Hyades_standing_layer_and_observation.md` §3 (σ), §6.2 (acceleration as the
  observable), §6.4–6.5 (belief and SPRT), §9.2 (laden hulls are conspicuous)
- `Hyades_trees_and_card_value.md` §2.3.2 — the relative objective and `w_ij`;
  §4 — card value, the P92 contract and the doubling-time numeraire §8.4 departs from
- `Hyades_card_contract.md` §10 — the write-capability partition §8.1 rests on
- `Hyades_technology_tree.md` — what capability is bought with
- `Hyades_politics_trade_and_intelligence.md` §8 — the time-dependent counter-graph
  edge between these two trees
- `Hyades_industry.md` §1.2 (infrastructure is the war target), §1.8 (population is
  not a factor of production, which is why §8.1's license is currently inert),
  §9 (captured infra)
- `Hyades_vehicle_roles.md` §4.2 — the Colonizer role and the arrival behavior
  §8.2 changes
- `src/combat.rs`, `src/arena.rs`, `src/belief.rs`
- CLAUDE.md design laws #2, #3, #4, #7, #8, #10, #11
