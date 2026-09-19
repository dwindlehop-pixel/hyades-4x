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

**6.4 `OPEN, advanced` — R-WAR2: the card surface.** No Warfare card's *effect*
is authorable until §3.4 and §3.5 land, because the effect is either a *posture*
(needs an accept/decline site) or a *stat* (needs the four resolver layers). What
**is** specifiable now is the shape — empire-scale, legible, placing an object
rather than applying a percentage — and **§7 is the first card specified to it**:
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

## 7. The first Warfare card — the armed coloniser (R-WAR2, advanced)

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

### 7.1 Warfare's licence, and why it is currently worth nothing

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

### 7.2 The card — one Design write, three Doctrine writes

**Author's specification.** The colony ship moves off the Systems family and
onto the Contact family: armed, worse freight per kilotonne of price, and —
crucially — **it is not consumed when it founds**.

| half | write | engine surface | status |
|---|---|---|---|
| **Design** | unlock a Contact coloniser class: `(GeneralContactVehicle, Class)` | `CardEffect::UnlockDesign`, and a per-`Class` drive fraction | `UnlockDesign` **exists**; per-`Class` `φ` is **T-97**, blocked on T-08 |
| **Doctrine** | unladen Contact Vehicles engage nearby enemies | an engagement-posture field on `Doctrine` | **absent** — no such field, and no combat in the sim loop (T-52) |
| **Doctrine** | a Neutral empire is an Enemy empire | a diplomatic stance default on `Doctrine` | **absent** — R-O27/T-11, the field list has never been specified |
| **Doctrine** | a Contact coloniser patrols instead of scrapping itself | the Colonizer role's arrival behaviour (roles §4.2) | the arrival path exists; the branch does not |

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

### 7.3 One third of the stated intent is impossible, and the engine says so exactly

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

### 7.4 What it is evaluated against, and why trees §4 does not cover it

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

### 7.5 What this is blocked on, in order

Stated as a chain because none of it is parallel:

| # | blocker | code | why it binds |
|---|---|---|---|
| 1 | no accept/decline site, and no combat in the simulation loop | §3.4, T-30 / T-12 | `combat::resolve_engagement` is never called from `sim.rs`, so *"engage nearby enemies"* has no execution path and the card's whole Warfare half is inert |
| 2 | no diplomatic fields on `Doctrine` | R-O27 / T-11 | *"a Neutral empire is an Enemy empire"* is a stance default, and there is no stance |
| 3 | `φ` is global, not per-`Class` | T-97, blocked on T-08 | §7.3's Design write cannot be expressed; R-O47b forbids it reaching hulls already in the field |
| 4 | Warfare's objective is unreadable on the standard bed, and `w_ij` is unset | R-TREE8, **R-WAR3** | §7.4 |
| 5 | population is not a factor of production | T-107 | §7.1 — the licence the tree is being given costs its victims nothing |

Items 1–3 are engine work with no open design question left in them — and item 1
is §3.4, which T-104 names the highest-leverage unbuilt thing in the engine.
Items 4 and 5 are design work, specified here and in `Hyades_industry.md` §1.8. **The card is
authorable now and measurable at none of these steps but the last**, which is
why it is written down rather than built.

---

---

## 8. Register

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
| **R-O95** | the empty-to-laden acceleration swing **is** the hull's cargo efficiency, exactly — so a hull cannot be worse at freight, faster empty and slower laden at once (§7.3) |
| — | **only Warfare may carry a population-lethal Doctrine write** (card contract §10), enforced in the card layer rather than by authoring convention |
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
| R-WAR2 | the card surface | **advanced** — §7 specifies the first card's shape and value model; its effects stay blocked on T-30 and R-MC9c |
| **R-WAR4** | **the armed coloniser's magnitudes** — the Contact coloniser class's drive fraction `φ` (0.02 is the smallest value that makes its cargo-inefficiency claim true, and it is a placeholder), the card's mineral cost, and where in the tree the pair sits once §7.4's dispersion prediction is measured | T-97 first, then a measurement on the asymmetric bed |
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
  §4 — card value, the P92 contract and the doubling-time numeraire §7.4 departs from
- `Hyades_card_contract.md` §10 — the write-capability partition §7.1 rests on
- `Hyades_technology_tree.md` — what capability is bought with
- `Hyades_politics_trade_and_intelligence.md` §8 — the time-dependent counter-graph
  edge between these two trees
- `Hyades_industry.md` §1.2 (infrastructure is the war target), §1.8 (population is
  not a factor of production, which is why §7.1's licence is currently inert),
  §9 (captured infra)
- `Hyades_vehicle_roles.md` §4.2 — the Colonizer role and the arrival behaviour
  §7.2 changes
- `src/combat.rs`, `src/arena.rs`, `src/belief.rs`
- CLAUDE.md design laws #2, #3, #4, #7, #8, #10, #11
