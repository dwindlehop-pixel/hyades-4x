# Hyades — Technology: Design, Components, and Capability

*The design space for **Technology**, the tree that raises what a given mass of
fleet can *do*. Its objective is **capability-years**
(`Hyades_trees_and_card_value.md` §2.3.5). Companion to `Hyades_loadout.md` (the
slot and item model), `Hyades_standing_layer_and_observation.md` (Design as
standing-layer state, and the asymmetric leak), `Hyades_warfare_tree.md` (what
capability is spent on) and `Exotic_matter_technology_inspiration.md`. New calls
continue the **R-TECH n** series.*

**Rev 2 (T-131).** Carries **ratified decisions and open decisions only**
(`CLAUDE.md` §6). This is the **least-built** of the six trees: the standing-layer
half is ratified and shipped (`Roster`, `Class`, `UnlockDesign`), the loadout
model is specified and unbuilt, and **the objective is now specified by the
author** (§4) — a static rating of every Design, earned head-to-head in one test
bed per role — and unbuilt. Most of this file is `OPEN`, and it says so per item
rather than in the prose.

**Rev 2 changes:** §4 rewritten to the author's specification, replacing the
power-mean proposal (R-TREE4, never ratified; appendix §D.9). §5.6 added — a
Technology write reaches no build today. R-TECH5 through R-TECH18 opened.

---

## 0. What this tree is

**Technology raises per-mass effectiveness. Production raises mass.** That split
is the reason the two trees are separate and it is the reason capability is *not*
fleet mass — a capability metric that counted mass would score Production's work
and rank the wrong cards.

**The *Stars!* lineage, and what Hyades takes from it.** *Stars!* runs six
research fields (Energy, Weapons, Propulsion, Construction, Electronics,
Biotechnology), each 0–26, where a level **unlocks components** rather than
granting an ability, and **miniaturization** makes older components cheaper as
the field advances. Three properties of that design are worth keeping and one is
not:

| *Stars!* property | Hyades |
|---|---|
| tech gates **components**, not abilities | **kept** — a card writes the `Roster`, and the roster is what may be built |
| progress is **permanent and monotone** | **kept** — Design never goes stale (std §5) |
| **miniaturization** makes what you have cheaper | **kept, but it is Production's axis** — `eta_works` |
| **26 discrete levels per field, researched with a budget slider** | **dropped** — Hyades has cards, not a research economy, and a slider is micro the design pillar forbids |

**The departure that matters:** in *Stars!* you cannot see a rival's design until
you meet it, and then you can see it forever. Hyades makes that asymmetry
explicit and *tradeable* — **Design leaks spatially and never goes stale**, which
is why it is the most valuable intelligence commodity in the game
(`Hyades_politics_trade_and_intelligence.md` §5.1) and the most damaging to have
published.

---

## 1. Design as standing-layer state

**1.1 `RATIFIED` — Design is the `Roster`: a sorted, idempotent set of
`(HullType, Class)` written only by tree cards.** **R-O28.** Entries are added and
**never removed**, so a scan that reads a roster reads it forever.

**1.2 `RATIFIED` — hull, class and role are three separate things.** The hull is
the object's size and family; the **class** is the specific design of it an empire
has unlocked; the role is what that object is currently being used for. Only the
first two are chosen at production. **R-O29.**

**1.3 `RATIFIED` — Design is permanent and strictly earlier-is-better; Doctrine
dies on retasking.** The **asymmetric leak** (std §5) is what makes the
intelligence price ladder a ladder at all, and it is what makes a Design
disclosure an attack rather than a nuisance.

**1.4 `RATIFIED` — no retroactive refits.** Design law #12 / **R-O47b**. A Design
write never reaches a hull already in the field by fiat; realization is
`on_refit`, so a fleet-wide change lands **staggered by transit time**.

Retroactive would change every acceleration signature at once, laglessly, with no
build to watch for — the instant global state change the light-lagged observation
model exists to rule out. **The recall is itself a signal**, and it cannot be
offset by a thrust write, because what leaks is the movement, not the mass.

**1.5 `RATIFIED` — the seeded roster is LSV(Meadow) + LCV(Tor).**
`SimConfig::enforce_roster` gates production on the roster and **defaults off**,
because the engine has no card system and therefore no unlock path: colonizer and
freighter ride on MSV, which the starting roster excludes, so enforcement forbids
every expansion build permanently — measured, **3 colonies and 18 vehicles against
1,183 and 4,778** over 4,000 yr. **Blocked on cards, not on engine work (T-25).**

**1.6 `OPEN` — R-O42b: the class flavor names.** Meadow for the LSV and Tor for
the LCV are proposed, scaling the Banks convention down to Limited sizes. Flavor
text is the author's own; renaming is a one-line change.

**1.7 `OPEN` — R-O47b / T-08: `on_refit` is specified and unbuilt.** Nothing in
the engine stages a design change across transit today, so 1.4 is a law with no
enforcement.

---

## 2. Components — the loadout model

**2.1 `RATIFIED` — a hull is fitted by filling typed slots.**
`Hyades_loadout.md` §2, adapted from *Stars!*:

| Slot | Accepts | Drives |
|---|---|---|
| **ENG** | engines | thrust → acceleration, with mass |
| **WPN** | beam / pulse / torpedo / missile | offense in the counter-graph |
| **ARM** | armor | ablative structure |
| **SHD** | shields | regenerating buffer |
| **AoS** | armor *or* shield | flex defense |
| **ELEC** | stealth, sensors, targeting, steering | detection, evasion, accuracy |
| **MECH** | cargo pods, colonization gear, mining rigs, mass drivers | the Systems-role tooling |
| **GP** | anything except ENG | fill to taste |

**2.2 `RATIFIED` — hull type sets the slot layout, and class *biases* it without
hard-gating.** No class is barred from WPN/ARM/SHD. Systems and Contact hulls have
*fewer* of them and more MECH/ELEC; Offensive hulls are the reverse. **A Systems
Vehicle mounting a defensive beam is a slot filled, not a special case** — the
counter-graph applies to every ship because every ship can carry counter-graph
items.

**2.3 `RATIFIED` — ship-level aggregates are queries, never stored fields.**
Total mass, total thrust, effective acceleration, total armor, sensor range are
computed from the fit on demand. A fit changes only at a production center and
mass changes every time cargo moves, so storing derived totals means invalidating
them on every such event; recomputing from `O(slots)` data cannot desync.

**2.4 `RATIFIED` — acceleration is `thrust / (dry_mass + cargo_mass)`, and thrust
is drawn from *mounted drive*.** **R-MC16 / T-96.** Drive mass is
`structural_drive_fraction · shell + drive_volume_fraction · volume`, so drive and
cargo both scale `r³` and the shell term shrinks away.

**T-96 kept the load-state broadcast and deleted the tax**, and those are two
different things that thrust-∝-dry-mass conflated: a laden GSV flew at **0.317×**
a laden MSV, so design law #3's own cost advantage was being repaid in turnaround.
The round trip is now **1.011** against an equal-cost Medium fleet, while the
*signature* is untouched — a General hull still drops 5.06 g empty to 0.23 g
laden, a **22× swing**, against a Limited hull's 1.00 → 0.70.

**Read a law about a ratio as a claim about the ratio.**

**2.5 `OPEN` — R-L0: the per-hull slot tables.** The model is specified; the
counts are not. This is the single largest unbuilt piece of this tree, and design
law #2 constrains it: **hull supremacy must be slot-organic**, emerging from slot
counts and volume, never from tuning a battle constant.

**2.6 `OPEN` — R-L1: do shields regenerate, and across what?** "No tick-based
initiative" may also mean no in-combat rounds for a shield to regenerate across.
*Recommend* a per-engagement buffer that resets between distinct engagements
rather than regenerating mid-exchange. Tied to `Hyades_warfare_tree.md` §2.

**2.7 `OPEN` — R-O65: `hull_thrust_to_mass` still varies 1.2 / 1.1 / 1.0 across
Systems sizes**, which the shell model says should be flat — empty-hull
acceleration is size-independent. **Not flattened, because it is an MC-tuned
combat surface** and the working agreement requires explicit ratification.

**2.8 `OPEN` — R-O60 / T-04: magazine mass on ordnance families.** Expended
ordnance must leave the fleet lighter (design law #11), which makes a magazine a
mass that is spent — and therefore a *readable* one, since a fleet that has shot
its magazines accelerates differently.

---

## 3. The counter-graph

**3.1 `RATIFIED` — mineral substitution lives in the counter-graph, not the
mineral ladder.** Design law #1. **Red is the general key** (broad class access);
**Blue and Green are traversal keys** (specific edges only).

**3.2 `RATIFIED` — supers are synthesized, never mined, and only at pop-Band IV.**
Fixed two-basic recipes: `Blue ← C+M`, `Red ← M+Y`, `Green ← Y+C`. Each archetype
is rich in two basics and poor in the third — **the two precursors of its single
native super** — so every empire self-synthesizes exactly one and must acquire the
other two. **That is the structural reason the Exchange exists.**

**3.3 `RATIFIED` — no categorical strategic classification may be co-extensive
with a color domain.** Design law #13 / **R-O34**. It would lock out exactly the
archetype poor in that color. Continuous classifications expressed as magnitude
are exempt.

**3.4 `RATIFIED` — the counter-graph is a per-player ladder disrupted by cards**,
not a global table (standing layer §0). A counter is something you *hold*, and a
card can take it away.

**3.5 `RATIFIED` — exotic synthesis is pair production.** Negative and imaginary
mass are **not** exceptions to conservation (design law #11) — which is exactly
why they must be made in pairs.

**3.6 `OPEN` — R-CG*: is the counter-graph acyclic?** Intransitive
("nontransitive") balance is explicitly on the table. This is a design decision
with large consequences for card authoring and it has not been made.

**3.7 `OPEN` — R-XM*: apex, and what it is for.** Synthesized from supers. Present
in `resources.rs` and in nothing else.

---

## 4. The objective — capability-years, rated statically per role

*Author's specification (T-131): "Evaluate fleet capability integrated over time
statically, assigning an ELO score to each possible Design/hull/class/role. For
each candidate, pit a fleet [10–1000], depending upon size, against all other
candidates in a test bed per role. So a colony ship would need to colonize
planets, a picket would need to intercept less heavily armed targets, a
battleship would need to engage in a pitched battle, and a miner would need to
fly to a mining outpost and refine ore. Each test bed is competitive, with two
players in a head-to-head competition. Competitions are judged according to
task. ELO is calculated (possibly with normalization across roles) the usual
way, and then the tree metric is ELO-as-capability integrated over the whole
fleet, for every year."* And: *"Long range offensive roles should start at a
distance. Short range offensive roles should start at point blank range."*

This replaces the power-mean proposal (R-TREE4), which was never ratified; what
it was and why it is replaced is in appendix §D.9.

### 4.1 Terms

| symbol | name | unit | where it is set |
|---|---|---|---|
| `d` | a **Design**: a `(HullType, Class)` and the loadout `design_loadout` stamps on it at construction | — | §1.1, warfare §8.17 |
| `r` | a **role** that has a test bed | — | §4.4 |
| `𝒟` | the **candidate pool**: every distinct Design the engine and the card table can produce | set | §4.3 |
| `m_d` | dry mass of one hull of `d`, which is its mineral cost (R-O57) | kt | `hull_dry_mass` |
| `B` | the **equal-spend budget** each side of a match receives | kt | §4.5, placeholder |
| `N_d` | hulls of `d` a side fields: `round(B / m_d)` | count | §4.5 |
| `x_A` | side A's **task score** in one match | per role, §4.4 | the bed |
| `s_AB` | A's **score share**, `x_A / (x_A + x_B)`, or `½` when both are zero | in `[0, 1]` | §4.6 |
| `R(d, r)` | `d`'s **rating** in role `r` | Elo points | §4.6 |
| `γ(d, r)` | `d`'s **strength** in role `r`, `10^(R/400)` scaled so the role's anchor reads 1 | ratio | §4.7 |
| `d₀(r)` | the **anchor** Design of role `r` | — | §4.7 |
| `c(d)` | a hull's **capability**: the strength its Design carries into the stock | ratio | §4.7 |
| `Q_i(t)` | player `i`'s **capability stock**: `Σ_{h ∈ i} m_h · c(d_h)` | kt of anchor-equivalent | §4.7 |
| `T_i` | **capability-years**: `∫₀^T Q_i(t) dt`, sampled yearly | kt·yr | §4.2 |
| `D_long` | starting separation of the long-range offensive bed | ly | §4.4, placeholder |

### 4.2 `RATIFIED` — capability-years, sampled every year

```text
T_i = ∫₀^T Q_i(t) dt        Q_i(t) = Σ_{h owned by i at t}  m_h · c(d_h)
```

The form every tree's objective takes (trees §2.2): an integral of a stock, so
it falls the moment anything slows down. Sampled on a fixed yearly grid (trees
§3.3), never on events.

### 4.3 `RATIFIED` — the rating is static, and the pool is every possible Design

**Static** means the rating table is computed offline, once per engine revision,
and `Q_i` is a lookup during a run. Three properties follow, and the first is
the reason this is a measurement rather than a target:

- **An opponent cannot move it.** A hull's capability does not depend on what the
  rival is flying this game, so a Technology card is not scored by the luck of
  the matchup.
- **Unlocking a Design moves no rating.** The pool is every *possible* Design,
  not the ones anyone has unlocked, so a roster write changes `Q_i` only when
  hulls of the new Design are built (§4.8).
- **It costs nothing at run time.** `Q_i` is one multiply-add per owned hull per
  sample.

**4.3.1 `OPEN` — R-TECH17: the pool is deduplicated by what the engine reads, not
by name.** *Recommend:* two Designs whose every field the beds read is equal are
one candidate. Under a Bradley–Terry rating (§4.6) a duplicate is not neutral: it
adds a second copy of the same wins and losses, which moves the ratings of every
Design it plays (Balduzzi et al. 2018, §3). **Measured, the current pool has
duplicates:** `LimitedContactVehicle`, `LimitedContactUnit` and
`LimitedOffensive` are identical in dry mass (0.0200 kt), mounts (1 beam) and
structure (20 kJ), and so are the two General Contact hulls (1.0995 kt, 317
beams). Ten hull types are **seven** distinct Designs today — three unarmed
Systems hulls and four armed ones (appendix §D.9). `design_loadout` ignores the
class, so a class adds no candidate yet.

### 4.4 `RATIFIED` — one competitive test bed per role, two players, judged by task

Each bed is a **head-to-head**: two players, each fielding an equal-spend fleet
of one candidate Design, in a symmetric scenario where they compete for the same
thing. Every candidate plays every other candidate in the role's pool.

| role | what each side does | judged by (`x`) | start | engine today |
|---|---|---|---|---|
| **Colonizer** | found colonies on a shared field of worlds; a world founded first is gone for the other side | colonies founded in the bed's horizon | symmetric home ports | the role system exists; the bed does not |
| **Miner** | fly to outposts on a shared field of rocks, extract, and deliver ore to be refined into the bank; rocks deplete for both | ore banked | symmetric home ports | the role system exists; the bed does not |
| **Picket** | each side has a port launching a fixed schedule of **reference** colony ships, less heavily armed than any picket; pickets strike the rival's launches and defend their own | rival launches destroyed | blockade stations at the rival port (warfare §8.16) | the strike exists; the bed does not |
| **Short-range offensive** | a pitched battle | surviving dry mass | **point blank** — both fleets on one reference point | `resolve_beam_engagement`; degenerate at current magnitudes (§4.9) |
| **Long-range offensive** | a pitched battle | surviving dry mass | **at a distance**, `D_long` apart | as above; beams alone do not reach (§4.9) |

**4.4.1 `RATIFIED` — the starting geometry belongs to the role, never to the
Design.** A short-range Design entered in the long-range bed starts at a
distance and is judged there. This is warfare §8.17's rule — *a loadout is a
property of the Design, never of the fight* — applied to where the fight starts:
the bed sets the geometry per role, and every candidate in that role meets the
same geometry.

*Inference, stated separately:* splitting pitched battle by range moves warfare
§2.4's largest counter-graph cycle — long-range families beat close families at
range and lose to them close — **between** pools rather than inside one, which is
the case a single-axis rating handles worst (§4.6.3). Confidence is moderate; a
measured cyclic-triad count inside each offensive pool (R-TECH7) would confirm or
refute it once a second weapon family exists.

**4.4.2 `OPEN` — R-TECH12: `D_long`, and whether the fleets close.**
*Recommend:* the long-range bed starts the fleets `D_long` apart **closing at a
fixed relative velocity**, so a match passes through every range from `D_long`
to zero — the torpedo's advantage in warfare §2.3 is damage delivered *before*
the brawler reaches beam range, which needs the brawler to be approaching.
**Measured on stationary fleets, beams stop reaching:** ten LCVs a side destroy
each other completely out to 1e-3 ly, survivors appear at 3e-3 ly, and at 0.1 ly
no shot lands (appendix §D.9). So a long-range bed whose fleets do not close is
a draw for every beam Design. `D_long` should sit beyond that measured beam
reach; it is a placeholder until a ranged family exists. Ties to **R-L2**
(single pass or repeated passes).

**4.4.3 `OPEN` — R-TECH13: the Scout and Freighter beds.** Not named by the
author. *Recommend:* Scout — a survey race on a shared unscanned field, judged by
worlds scanned first. Freighter — a haul race between shared outpost piles and
each side's own bank, judged by kilotons delivered.

**4.4.4 `OPEN` — R-TECH16: what is held fixed in a bed.** *Recommend:* Doctrine
at the default for every candidate, so the bed rates **Design only** — the write
surface this tree owns (§5). The beds run the engine's own role systems, seeded
the way the arena seeds combat: **a bed is a scenario seeder that owns no role
logic**, and the dependency runs `bed → sim`, as `arena → combat` does (§3 of
`CLAUDE.md`). Each pair is played on the same seeds (common random numbers) with
the two sides' positions mirrored, so a side has no geometric advantage. Seeds
per pair and each non-combat bed's horizon are placeholders.

### 4.5 `RATIFIED` — fleet size follows hull size, 10 to 1,000 a side

**4.5.1 `OPEN` — R-TECH11: sizes set by equal mineral spend.** *Recommend:*
`N_d = round(B / m_d)` with `B` = ten General hulls' price. On the shipped ladder
(1 : 1/10 : 1/50) that fields **10 General Systems, 12 General Contact, 13
General Offensive, 120–132 Medium/Rapid and 654–658 Limited** hulls — inside the
author's range. Equal spend, not equal count, because a rating earned at equal
count would score a design for being cheap: design law #3 is a claim about equal
spend, and so is law #2's target (1 GOU against 6–45 ROUs).

### 4.6 `RATIFIED` — the rating is Elo

**4.6.1 `OPEN` — R-TECH5: fit, do not update.** *Recommend:* fit the
**Bradley–Terry model** by maximum likelihood over the whole round robin, on the
Elo scale. Elo's expected score, `1 / (1 + 10^(−ΔR/400))`, *is* the
Bradley–Terry win probability with strengths `10^(R/400)` (Bradley & Terry 1952;
Elo 1978), so the scale and meaning are Elo's. What changes is the estimator:
the sequential update the chess federations use depends on the order the games
are played in and on a step size `K`, and a static round robin has no order.
The maximum-likelihood fit is unique when it exists (Zermelo 1929), converges by
the minorization–maximization iteration (Hunter 2004), and is deterministic.

The outcome of a match is the score share `s_AB`. Elo already accepts fractional
scores (a draw is ½), and the share keeps the margin a bare win would throw
away.

**4.6.2 `OPEN` — R-TECH6: complete separation.** A Design whose share is 1
against every opponent has **no finite maximum-likelihood rating** — the
likelihood keeps rising as its rating grows (Hunter 2004 states the condition).
That is not an edge case here: design law #2 *wants* a General Offensive fleet
to beat an equal-spend Rapid one decisively. *Recommend:* a prior of one
virtual draw between every pair, the device Coulom's Whole-History Rating uses
(Coulom 2008) — it keeps every rating finite and moves a well-measured one by
little.

**4.6.3 `OPEN` — R-TECH7: intransitivity.** Bradley–Terry places every Design on
one line; a counter-graph with cycles (R-CG*, §3.6) cannot be put on one line
without error. *Recommend:* report, per role, the count of cyclic triads
(Kendall & Babington Smith 1940) and the share of variance in the pairwise
logit-shares that the fitted rating differences explain. If that share is low,
replace the scalar rating with a multidimensional one or with Nash averaging,
which is also invariant to duplicated candidates (Balduzzi et al. 2018).

### 4.7 `RATIFIED` — capability integrated over the whole fleet; normalization across roles is the author's option

**4.7.1 `OPEN` — R-TECH8: Elo points must become strengths before they are
summed.** Elo points are an **interval** scale: only differences carry meaning,
the zero is arbitrary, and a rating can be negative. A sum of ratings over a
fleet changes when the zero moves, so it measures nothing. *Recommend:* convert
to the Bradley–Terry strength `10^(R/400)` — a **ratio** scale, positive, with
`P(A beats B) = γ_A / (γ_A + γ_B)` — before summing. The stock is then positive,
which is also what the composite's geometric mean needs (trees §2.4, R-TREE9).

**Normalization across roles**, the author's option: a rating is comparable only
inside its own role's pool. *Recommend:* scale each role so its anchor Design
`d₀(r)` has `γ = 1`, which makes every strength read "multiples of the anchor in
this role". *Recommend* the anchor be the Design the default standing layer
assigns to that role (T-121), fixed at game start. **Open inside it:** the
default layer is unarmed, and an unarmed Design scores zero in a combat bed, so
its strength is zero and cannot be the unit. For the offensive and picket roles
the anchor must be an armed Design — the Warfare card's Limited Contact Design is
the candidate.

**4.7.2 `OPEN` — R-TECH9: weight each hull by its dry mass.** *Recommend*
`Q_i = Σ_h m_h · c(d_h)`, in kilotons of anchor-equivalent capability. The
rating was earned at equal **spend**, so it is a per-kiloton quantity, and the
weight that makes it a fleet total is spend — which since R-O57 is dry mass. Two
alternatives, and why not:

- **By count** scores a Limited hull equal to a General one of the same
  strength — the fragmentation reward design law #3 forbids.
- **By volume** counts consolidation **twice**: the equal-spend bed already
  measured how much better a large hull does per kiloton, and that is inside
  `γ`. Volume is Production's stock (trees §2.3.4) for exactly the reason it is
  wrong here.

Consequence, stated so it is not mistaken for a defect: `Q_i` is fleet mass times
a mass-weighted mean strength, so building more hulls raises it. That is
Production's axis appearing in Technology's stock. It does not mis-cost a card —
a card is valued on its own tree's stock (trees §2.4) — but mass enters the
composite geomean twice (R-TREE9).

**4.7.3 `OPEN` — R-TECH10: which role's strength a hull carries.** A hull's role
changes during a game; its Design does not. If a hull carried its **current**
role's strength, a Doctrine write that retasked hulls toward whichever role
rates them highest would raise `Q_i` without building anything — the
invariance rule's failure case (trees §2.5). *Recommend:*
`c(d) = max_r γ(d, r)`, the best role the Design can fill. `Q_i` then depends
only on which Designs an empire has built, which is what Technology writes.
Idle hulls in `Reserve` count; scrapped hulls are not owned and do not.

### 4.8 The invariance audit for this objective

| a card could… | move `Q_i` without moving the world? | answer |
|---|---|---|
| unlock a Design | no | the pool is every possible Design; only built hulls count (§4.3) |
| retask hulls (Doctrine) | **yes, if a hull carries its current role's strength** | carry the best role's (§4.7.3) |
| change what a rival flies | no | the rating is static (§4.3) |
| build more hulls | yes, and the world moved | Production's axis inside the stock (§4.7.2) |
| add a Design to the game (authoring, not play) | changes every rating | the table is per engine revision; dedup (§4.3.1, R-TECH17); Nash averaging if cyclic (§4.6.3) |

### 4.9 `OPEN` — R-TECH14: the combat beds rate every armed Design equal today

**Measured** (`examples/capability_probe`, appendix §D.9): an equal-spend,
point-blank round robin over the four distinct armed Designs — seven hull names —
ends every one of its 49 matches with **both fleets destroyed**, surviving mass
0.0000 kt summed over all of them. Every share is ½, so every rating is equal
and the short-range bed cannot rank anything.

*Inference from the magnitudes, not an instrumented count:* one beam mount
delivers `laser_shots_per_tick × beam_shot_energy_kj` = 40 × 50 = 2,000 kJ per
tick against a Limited hull's 20 kJ of structure, and fire is simultaneous, so
each side's first tick of fire exceeds the other side's whole structure and
both die together. Confidence is high; a per-tick casualty count would confirm
it, and a match that survived its first tick would refute it.

**What would settle it:** R-WAR19's placeholders (shot energy, structure per
kilotonne) set so that an equal-spend fight lasts many ticks — the condition
under which design law #2's slot-organic supremacy can express at all — plus a
closing long-range bed (§4.4.2). Until then the offensive ratings are all equal
and add nothing that `m_h` does not.

### 4.10 `OPEN` — R-TECH15: where the table lives and how it goes stale

*Recommend:* an offline harness writes the table to `data/`, stamped with the
digest of every configuration value the beds read; the harnesses that compute
`T_i` read it and refuse a table whose stamp does not match the running
configuration. The engine does not read it: capability is a measurement, and no
autopilot decision may consult it.

**Cost, bounded from below:** a point-blank combat match at 500 hulls a side
costs **0.032–0.049 s** over three runs on the current resolver, measured on a fight that ended at
once; a fight that runs its whole 0.5-yr engagement horizon is 1,000 ticks and
costs more. The round robin is `n(n − 1)/2` pairs per role times seeds — 21 pairs
for seven Designs. The colonizer, miner and picket beds are short simulation
runs and are not yet priced.

### 4.11 `OPEN` — R-TECH2: does capability saturate on the measurement bed?

Trees §3.2 requires per-stock saturation as the first measurement. Unknown until
the table exists.

---

## 5. What a Technology card writes

**5.1 `RATIFIED` — the write surface that exists.** `CardEffect::UnlockDesign(
HullType, Class)` adds a roster entry. That is the whole of it today: the engine
stores it, `enforce_roster` reads it when enforcement is on, and a disclosure
publishes it. §5.6 says what it does not reach.

**5.2 `RATIFIED` — legibility is σ read from the other side.** Design law #9. A
card's slant *is* how much it would only be worth playing if you meant it, so the
σ→value curve must be **convex** or everyone opens inscrutable and the yomi
channel carries nothing. **The physical cause is that commitment shifts and
narrows a fleet's acceleration distribution** — which is precisely what a
Technology unlock does.

**5.3 `RATIFIED` — concealment is a combo property, not a card property.**
Design law #10. `a = thrust / (dry_mass + cargo_mass)` is one scalar over three
latents, so the inverse problem is under-determined at range — but **arming a
fleet is loud unless you also buy thrust.** A Technology card that adds weapons
without a propulsion partner announces itself.

**5.4 `OPEN` — R-TECH3: the card surface beyond `UnlockDesign`.** Candidates, in
rough order of how well the engine could support them today: component stat
tables (needs §2.5), `hull_thrust_to_mass` (needs R-O65 ratified), sensor and
stealth ranges (needs the detection query), the counter-graph edges themselves
(needs R-CG*). **None is authorable until the thing it writes exists.**

**5.5 `OPEN` — R-TECH4: is there a miniaturization analogue *inside* this tree?**
`eta_works` is Production's and should stay there (§0). The Technology version
would be "an unlocked component gets cheaper as the tree deepens", which is a
second multiplicative axis on the same bill and risks making the two trees
substitutes rather than complements. *Recommend* no — and record the reasoning,
because it is the obvious thing to add later.

**5.6 `OPEN` — R-TECH18: an unlock reaches no build, so every Technology card is
worth zero on §4's metric by construction.** Two readers are missing, and either
one alone is enough:

- `roster_permits` returns `true` before it reads the roster whenever
  `enforce_roster` is off, which is the shipped default (T-25, §1.5).
- `Standing::design_for` — the only function that chooses which Design a role is
  built on — reads **Doctrine only**. Nothing that decides a build reads the
  roster, so unlocking the General Contact Design (card 14) never causes one to
  be built, and `Q_i` counts only built hulls (§4.3).

So `T_i` for a Technology card is the pass arm's `T_i` exactly, whatever the
table says. That is `CLAUDE.md`'s *count the consumers of a write* in its plain
form, and it is found by reading, not by a bed.

**What would settle it:** a Design resolver that picks, per role, among the
Designs the roster holds. Which one it picks is a policy question, and **it must
not read the capability table** — a policy that maximizes the metric would turn
the measurement into the target (§4.10).

---

## 6. Register

### Ratified

| Code | Decision |
|---|---|
| R-O28 | Design is the `Roster`, written only by tree cards |
| R-O29 | hull, class and role are three separate things |
| R-O34 | no classification co-extensive with a color domain (law #13) |
| R-O42 | seats are seeded LSV(Meadow) + LCV(Tor) |
| R-O47b | no retroactive refits; realization is `on_refit` (law #12) |
| R-MC16 | thrust is drawn from mounted drive |
| — | supers are synthesized at pop-Band IV by fixed two-basic recipes |
| — | Red is the general key; Blue and Green are traversal keys (law #1) |
| — | aggregates are queries, not stored fields |

### Open

| Code | Question | What would settle it |
|---|---|---|
| R-L0 | per-hull slot tables | the arena (design law #4) |
| R-L1 | shield regeneration across what | a decision, with warfare §2 |
| R-O60 / T-04 | magazine mass on ordnance families | engine work |
| R-O65 | flatten `hull_thrust_to_mass`? | explicit ratification — MC-tuned surface |
| R-O42b | class flavor names | the author |
| R-O47b / T-08 | `on_refit` is unbuilt | engine work |
| R-CG* | is the counter-graph acyclic? | a design decision |
| R-XM* | apex, and what it is for | a design pass |
| R-TECH1 | **the objective is specified (§4) and not built** — no bed, no table | T-131 |
| R-TECH2 | does capability saturate on the bed? | blocked on R-TECH1 |
| R-TECH5 | fit Bradley–Terry by maximum likelihood on the Elo scale, not sequential updates — *recommended* | author |
| R-TECH6 | complete separation: a prior of one virtual draw per pair — *recommended* | author |
| R-TECH7 | intransitivity: cyclic-triad count and explained variance per role; Nash averaging if low | the first real table |
| R-TECH8 | Elo points → strength `10^(R/400)`; per-role anchor at `γ = 1`; which anchor for combat roles | author |
| R-TECH9 | weight hulls by dry mass, not count or volume — *recommended* | author |
| R-TECH10 | a hull carries its Design's best-role strength, not its current role's — *recommended* | author |
| R-TECH11 | equal-spend budget `B` = ten General hulls — *recommended* | author |
| R-TECH12 | `D_long`, and a closing long-range bed | a ranged family or beam falloff, then a sweep |
| R-TECH13 | Scout and Freighter beds | author |
| R-TECH14 | **combat beds tie every armed Design at current lethality** | R-WAR19 |
| R-TECH15 | the table's storage and staleness stamp | engine work |
| R-TECH16 | beds hold Doctrine at the default and rate Design only — *recommended* | author |
| R-TECH17 | the pool deduplicated by what the beds read — *recommended* | author |
| R-TECH18 | **an unlock reaches no build** — no Design resolver reads the roster | a resolver; T-25 |
| R-TECH3 | the card surface beyond `UnlockDesign` | blocked on R-L0 and R-O65 |
| R-TECH4 | a miniaturization analogue inside this tree? | a decision — *recommend no* |
| T-25 | `enforce_roster` defaults off because there is no unlock path | the card system |

---

## References

- `Hyades_loadout.md` — the slot model, item families, and the integration plan
- `Hyades_standing_layer_and_observation.md` §2 (concealment by vector), §5 (the
  asymmetric leak), §6.2 (acceleration as the observable), §7 (hull/class/role)
- `Hyades_trees_and_card_value.md` §2.3.5 (capability-years), §2.5, §3.2, §4.3
- `examples/capability_probe` — the pool, the per-match cost, beam reach, and the
  short-range round robin (appendix §D.9)
- Balduzzi, D., Tuyls, K., Pérolat, J. & Graepel, T. (2018). Re-evaluating
  evaluation. *Advances in Neural Information Processing Systems 31.* —
  intransitivity, redundant candidates, Nash averaging
- Bradley, R. A. & Terry, M. E. (1952). Rank analysis of incomplete block
  designs: I. The method of paired comparisons. *Biometrika* 39(3/4), 324–345
- Coulom, R. (2008). Whole-History Rating: a Bayesian rating system for players
  of time-varying strength. *Computers and Games*, LNCS 5131, 113–124 — the
  virtual-draw prior
- Elo, A. E. (1978). *The Rating of Chessplayers, Past and Present.* Arco
- Hunter, D. R. (2004). MM algorithms for generalized Bradley–Terry models.
  *Annals of Statistics* 32(1), 384–406 — existence condition and iteration
- Kendall, M. G. & Babington Smith, B. (1940). On the method of paired
  comparisons. *Biometrika* 31(3/4), 324–345 — cyclic triads
- Zermelo, E. (1929). Die Berechnung der Turnier-Ergebnisse als ein
  Maximumproblem der Wahrscheinlichkeitsrechnung. *Mathematische Zeitschrift* 29,
  436–460 — uniqueness of the fit
- `Hyades_warfare_tree.md` — what capability is spent on, and the arena that sets it
- `Hyades_production_tree.md` §5 (the hull ladder this fits onto)
- `Exotic_matter_technology_inspiration.md` ·
  `Hulls_classes_the_qualitative_counter-graph.md` (the Banks-convention source)
- `src/sim.rs` — `Roster`, `Class`, `HullType`; `src/cards.rs` — `UnlockDesign`
- CLAUDE.md design laws #1, #2, #9, #10, #12, #13
