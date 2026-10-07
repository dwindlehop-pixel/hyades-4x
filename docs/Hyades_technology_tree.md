# Hyades — Technology: Design, Components, and Capability

*The design space for **Technology**, the tree that raises what a given mass of
fleet can *do*. Its objective is **capability-years**
(`Hyades_trees_and_card_value.md` §2.3.5). Companion to `Hyades_loadout.md` (the
slot and item model), `Hyades_standing_layer_and_observation.md` (Design as
standing-layer state, and the asymmetric leak), `Hyades_warfare_tree.md` (what
capability is spent on) and `Exotic_matter_technology_inspiration.md`. New calls
continue the **R-TECH n** series.*

**Rev 2 (T-131).** Carries **ratified decisions and open decisions only**
(`AGENTS.md` §6). This is the **least-built** of the six trees: the standing-layer
half is ratified and shipped (`Roster`, `Class`, `UnlockDesign`), the loadout
model is specified and unbuilt, and **the objective is now specified by the
author** (§4) — a static rating of every Design, earned head-to-head in one test
bed per role — with every bed built and a first table measured (§4.9.1). Most of this file is `OPEN`, and it says so per item
rather than in the prose.

**Rev 2 changes:** §4 rewritten to the author's specification, replacing the
power-mean proposal (R-TREE4, never ratified; appendix §D.9). §5.6 added — a
Technology write reaches no build today. R-TECH5 through R-TECH18 opened. §7 added — technology options and inspirations (R-TECH20, R-TECH21). **T-132** resolved R-TECH14 (the damage model).

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

**1.6 `RATIFIED` — R-O42b: the class flavor names** (approved by the author
after the Tor/Ford split, T-133). On the landform convention (small landforms
for Limited hulls, larger for larger), one class per hull: **Spur** (survey,
LSV), **Tor** (armed survey, LCV), **Cairn** (picket, LCV), **Meadow** (miner,
LSV), **Delta** (colonizer, MSV), **Ford** (freighter, MSV), **Range**
(colonizer, GSV; after the Banks GSV class), **Strait** (freighter, GSV),
**Scarp** (armed colonizer, GCV). Offensive hulls stay unnamed: nothing builds
them. Flavor text is the author's own; renaming is one line per name.

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
its magazines accelerates differently. **Built for missiles (T-139, §9):** the
rounds are `Material::Ordnance` in the hull's hold, so the burn reads the
magazine and a fired round is debris on the ledger. Torpedoes have no family yet.

---

## 3. The counter-graph

**3.1 `RATIFIED` — mineral substitution lives in the counter-graph, not the
mineral ladder.** Design law #1. **Each super is dominant in different trees.**
Within a tree, **the dominant super is the general key** (broad class access) and
**the other two are traversal keys** (specific edges only). So *"Red is the
general key; Blue and Green are traversal keys"* is the Red-dominant case and
holds only in Red-dominant trees (the author's ruling, amending the earlier
unqualified statement). **`OPEN` — R-TECH25:** which super dominates which tree.

**3.2 `RATIFIED` — supers are synthesized, never mined, and only at pop-Band IV.**
Fixed two-basic recipes: `Blue ← C+M`, `Red ← M+Y`, `Green ← Y+C`, **each from its
two basics in a 1:1 ratio by mass** (the author's ruling: color theory; galaxy §4.2). Each archetype
is rich in two basics and poor in the third — **the two precursors of its single
native super** — so every empire self-synthesizes exactly one and must acquire the
other two. **That is the structural reason the Exchange exists.**

*Built* (galaxy §4.5): a forge synthesizes from its own holding when an order or
a sold contract owes a super. **"Exactly one" is a gradient, not a wall** —
measured: on a 3-seat bed with every seat's colonizer and miner Designs billed
25% Red, all three homeworlds made Red from the Magenta and Yellow their banks
held, including the two whose native super is not Red, because freight and the
Exchange put every basic in every bank. A seat poor in a precursor pays more for
the super it is not native to; it is not barred from it. **R-G4 resolved** (the
author's ruling): forges produce supers by color theory, and no archetype is
barred from a super (galaxy §3). Appendix §D.23, §D.27.

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

**3.7 — R-XM*: apex, and what it is for.** **`RATIFIED` (the author's rulings):**
apex is made from Red, Green and Blue in equal parts at `Y_apex = 1/2`, and
**every win condition is conditioned on apex** (trees §4.6). Built: a forge
makes apex from its three supers (and the supers from basics) when an order or a
contract owes it. **`OPEN`:** the win conditions themselves, and whether apex
carries any combat property (R-MC7/R-MC8, the specific-strength ladder).

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

**4.3.1 `RATIFIED` — R-TECH17: the pool is deduplicated by what the engine reads, not
by name.** Two Designs whose every field the beds read is equal are one
candidate. **The long-term goal is that no two named Designs share a loadout**
(the author's note on the approval), so every duplicate this rule merges is a
Design still waiting to be differentiated, and the dedup is expected to merge
nothing once that is done. Under a Bradley–Terry rating (§4.6) a duplicate is not neutral: it
adds a second copy of the same wins and losses, which moves the ratings of every
Design it plays (Balduzzi et al. 2018, §3). **Measured, the current pool has
duplicates:** `LimitedContactVehicle`, `LimitedContactUnit` and
`LimitedOffensive` are identical in dry mass (0.0200 kt), mounts (1 beam) and
structure (20 kJ), and so are the two General Contact hulls (1.0995 kt, 317
beams). Ten hull types are **seven** distinct Designs today — three unarmed
Systems hulls and four armed ones (appendix §D.9). `design_loadout` ignores the
class, **but structure does not since T-133** (`σ` is per Design class), so
two classes on one hull are now two candidates wherever their `σ` differs — the
survey Design (Tor) and the picket Design (Cairn) on the Limited Contact hull
are the first such pair.

### 4.4 `RATIFIED` — one competitive test bed per role, two players, judged by task

Each bed is a **head-to-head**: two players, each fielding an equal-spend fleet
of one candidate Design, in a symmetric scenario where they compete for the same
thing. Every candidate plays every other candidate in the role's pool.

| role | what each side does | judged by (`x`) | start | engine today |
|---|---|---|---|---|
| **Colonizer** | found colonies on a shared field of worlds; a world founded first is gone for the other side | colonies founded in the bed's horizon | symmetric home ports | **built** (§4.4.8) |
| **Miner** | fly to outposts on a shared field of rocks, extract, and deliver ore to be refined into the bank; rocks deplete for both | ore banked — **as built, ore extracted at the fleet's rocks** (§4.4.8) | symmetric home ports | **built** |
| **Picket** | each side has a port launching a fixed schedule of **reference** colony ships, less heavily armed than any picket; pickets strike the rival's launches and defend their own | rival launches destroyed — **as built, rival colony ships wrecked or turned away** (§4.4.8) | blockade stations at the rival port (warfare §8.16) | **built** |
| **Short-range offensive** | a pitched battle | dry mass holding the field (R-TECH19) | **point blank** — both fleets on one reference point | **built**: `examples/design_rating`, fleets generated with the galaxy (§4.4.5), the simulation's own event loop (warfare §8.19.3) |
| **Long-range offensive** | a pitched battle | surviving dry mass | **at a distance**, `D_long` apart | **built**, closing (§4.4.2); beams alone do not reach a standing fleet |

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
**Measured on stationary fleets, beams stop reaching:** ten LCVs a side fight to
a decision out to 3e-3 ly, and at 1e-2 ly and beyond the fight runs the whole
engagement with almost nothing destroyed (appendix §D.10; §D.9 is the same sweep
under the replaced damage model). So a long-range bed whose fleets do not close is
a draw for every beam Design. `D_long` should sit beyond that measured beam
reach; it is a placeholder until a ranged family exists. Ties to **R-L2**
(single pass or repeated passes).

**4.4.3 `RATIFIED` — R-TECH13: every role has a bed** (the author: *"Every
role needs a test bed"*). Scout — a survey race on a shared unscanned field,
judged by worlds reached first. Freighter — a haul race between outpost piles and
each side's own bank, judged by kilotons delivered. With the five above that is
one bed for every role a hull can be tasked with except `Reserve`, which is the
end of a mission rather than one (§4.7.3). Built (§4.4.8).

**4.4.8 `OPEN` — the non-combat beds as built.** Every bed is a two-seat
standard field (`GalaxyConfig::new(2, seed)`), each seat's fleet generated at
its home port at spend `B`, default Doctrine, and a surveyed start of 20 ly
(galaxy §3.1) where the role needs known worlds at `t = 0`. *Recommend* these
judges and horizons (placeholders):

| bed | fleets | judged by `x` | horizon |
|---|---|---|---|
| picket | one per seat, Picket, stationed on the rival's home port | rival colony ships wrecked or turned away | 150 yr |
| colonizer | one per seat, Colonizer | colonies the fleet founds | 80 yr |
| miner | one per seat, Miner, one hull to a rock the seat ranks an outpost | ore extracted at the fleet's rocks, by that seat | 160 yr — three outpost yields at `mining_tick_years` = 50 |
| freighter | a reference Meadow miner fleet and the candidate as Freighter, per seat | kilotons the fleet's freighters deposit | 160 yr |
| scout | one per seat, Scout | worlds the fleet reaches first | 40 yr |

The miner judge counts every crew of that seat at the fleet's rocks, the
autopilot's own included; both seats carry that term, so it is a shared
baseline rather than a bias, and it dilutes the share. The scout bed reads a
hull's price (how many hulls the spend buys) and its drive (the survey leg flies
`laden_accel`, appendix §D.19).

**4.4.5 `RATIFIED` — a bed's fleets are generated with the galaxy** (the
author's ruling, resolving R-WAR36): *"fleets can be optionally generated at
Galaxy generation, with a position and velocity. Equal cost mineral spend per
fleet makes sense."* `Galaxy::generate_with` takes a `FleetSeeding` — one spend
`B` for every fleet, and per fleet a seat, a Design, a role, a position and a
velocity — and the engine builds `round(B / m_d)` hulls of each, parked at rest
or shedding their velocity from the given state. The galaxy is still the only
thing a bed varies; the fight runs on the simulation's own events.

**4.4.6 `RATIFIED` — a pitched battle seats two hostile Doctrines.** Warfare
§8.19.2's ruling: two fleets stand and fight only when both sides' Doctrine is
to kill the other's fleet. Under the default Doctrine a hull that can outrun a
neutral attacker withdraws on the first hit (§8.19.6's third ending), and the
bed measured exactly that: every Scarp left the field under a scratch
(appendix §D.17). So the offensive beds seat both sides with `engage_neutrals`;
R-TECH16's default Doctrine stands for every other role.

**4.4.7 `OPEN` — R-TECH19: "surviving dry mass" is read as dry mass holding the
field.** A pitched battle ends in withdrawals far more than in wrecks (§8.19.6's
second ending sends a hull past its structure home), so a defeated fleet
survives almost whole. *Recommend* `x_A` = the dry mass of A's hulls neither
wrecked nor withdrawn at the horizon. **Horizon:** 1 year, a placeholder; every
measured match was decided long before it.

**4.4.4 `RATIFIED` — R-TECH16: a bed forces the role.** The author: *"need to
force role to test each role."* The bed generates each fleet **in the bed's
role** (galaxy §3.1), whatever the standing layer would task that Design as, so
every Design is tested in every role; Doctrine is otherwise the default, and
hostile on the offensive beds (§4.4.6). The bed rates **Design** — the write
surface this tree owns (§5). The beds run the engine's own role systems, seeded
the way the arena seeds combat: **a bed is a scenario seeder that owns no role
logic**, and the dependency runs `bed → sim`, as `arena → combat` does (§3 of
`AGENTS.md`). Each pair is played on the same seeds (common random numbers) with
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

**4.6.2 `RATIFIED` — R-TECH6: complete separation** (the author: *"Virtual
draw to start is fine"*). A Design whose share is 1
against every opponent has **no finite maximum-likelihood rating** — the
likelihood keeps rising as its rating grows (Hunter 2004 states the condition).
That is not an edge case here: design law #2 *wants* a General Offensive fleet
to beat an equal-spend Rapid one decisively. *Recommend:* a prior of one
virtual draw between every pair, the device Coulom's Whole-History Rating uses
(Coulom 2008) — it keeps every rating finite and moves a well-measured one by
little. **Ratified as the starting prior**: one virtual draw per pair.

**4.6.3 `RATIFIED` — R-TECH7: intransitivity is accepted, and re-evaluated at
every regeneration.** Bradley–Terry places every Design on one line; a
counter-graph with cycles (R-CG*, §3.6) cannot be put on one line without error.
The author's ruling: *"variance in ELO due to counter-graph is ok; every time we
regenerate the static ratings, we must reevaluate this rolling and determine it
still makes sense given the current counter graph."* So every regeneration
reports, per role, the cyclic triads (Kendall & Babington Smith 1940) and the
largest gap between an observed mean share and the share the ratings predict
(`examples/design_rating`'s `INTRANSITIVITY` line), and the regeneration is not
done until someone has read that report against the counter-graph then in force.
If it stops making sense, the replacement on the table is a multidimensional
rating or Nash averaging, which is also invariant to duplicated candidates
(Balduzzi et al. 2018).

### 4.7 `RATIFIED` — capability integrated over the whole fleet; normalization across roles is the author's option

**4.7.1 `RATIFIED` — R-TECH8: the rating is positive, on the ratio scale** (the
author: *"rating must be positive. Convert to ratio scale"*). The harness
reports the Bradley–Terry strength `γ` directly, the role's anchor at 1. Elo points are an **interval** scale: only differences carry meaning,
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

**4.7.2 `OPEN` — R-TECH9: weight each hull by its dry mass.** The author will
decide on seeing the ratio-scale table (§4.9.1), noting that the ratio scale may
already carry mass or volume. *Recommend*
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

**4.7.3 `RATIFIED` — R-TECH10: a hull carries its active role's strength.**
The author's ruling, against the recommendation: *"Evaluate the tree metric on
each ship's active role. If you have a fleet of long range apex destroyers
working a miner role, you didn't have much capability that year."* So
`c_h = γ(d_h, role_h(t))`, read each year from the role the hull is tasked with
at that time. Consequence, stated so it is not mistaken for a defect: a
Doctrine write that retasks hulls toward the roles their Designs rate highest
raises `Q_i` without building anything. Under this ruling that is capability —
a fleet used for what it is good at — and not a metric farm. **Open inside it:**
the strength of `Reserve` (a standing mission that ended; no bed rates it) —
*recommend* zero, since an idle hull does no role's work. Scrapped hulls are not
owned and do not count. *(Superseded recommendation `c(d) = max_r γ(d, r)`:
appendix §D.18.)*

### 4.8 The invariance audit for this objective

| a card could… | move `Q_i` without moving the world? | answer |
|---|---|---|
| unlock a Design | no | the pool is every possible Design; only built hulls count (§4.3) |
| retask hulls (Doctrine) | **yes, if a hull carries its current role's strength** | carry the best role's (§4.7.3) |
| change what a rival flies | no | the rating is static (§4.3) |
| build more hulls | yes, and the world moved | Production's axis inside the stock (§4.7.2) |
| add a Design to the game (authoring, not play) | changes every rating | the table is per engine revision; dedup (§4.3.1, R-TECH17); Nash averaging if cyclic (§4.6.3) |

### 4.9 R-TECH14 — resolved (T-132): the combat beds tied every armed Design, and now discriminate

Under the damage model T-132 replaced, an equal-spend, point-blank round robin
over the armed Designs destroyed both fleets in every one of its 49 matches
(appendix §D.9): one mount delivered a hundred Limited hulls' structure per
tick. **Resolved by the author's damage model** (warfare §8.18): beam power over
time, structure on hull volume. The same round robin now decides most pairings
one way, with mirror fights lasting 28–82 ticks; one GOU beats 40 ROUs and loses
to 50 (appendix §D.10).

**What remains for the beds, and is not this item:** mirror matches are decided
by station-keeping geometry rather than drawn, so a rating needs many seeds with
sides swapped (R-TECH16); and the long-range bed still needs fleets that close
(R-TECH12).

### 4.9.1 The table — every role, the named Designs, ratio scale (appendix §D.19)

`examples/design_rating all`, equal spend `B` = ten General Systems hulls
(13.15 kt), eight seeds, both seatings, 240 matches a role, every leg at its
Design's own drive. The table is `data/design_ratings.tsv`, stamped with the
engine commit (R-TECH15); the raw matches are `data/design_rating_matches.tsv`.
**Measured**, `γ` with the role's anchor at 1 (R-TECH8):

| role (anchor) | order, `γ` |
|---|---|
| short-range (Cairn) | Scarp 7.58 > Cairn 1 > Tor 0.192 > the unarmed three, 0.015 |
| long-range (Cairn) | Scarp 5.49 > Cairn 1 > Tor 0.754 > Delta 0.665 ≈ Meadow 0.660 > Range 0.609 |
| picket (Cairn) | Scarp 2.03 > Cairn = Tor 1 > the unarmed three, 0.037 |
| colonizer (Delta) | Delta 1 > Scarp 0.221 > Range 0.193 > the Limited three, 0.009 (they found nothing) |
| miner (Meadow) | Meadow 1 ≈ Tor = Cairn 1.000 > Delta 0.633 > Scarp 0.206 ≈ Range 0.200 |
| freighter (Delta) | Range 1.81 > Scarp 1.12 > Delta 1 > Meadow 0.181 > Tor = Cairn 0.016 (they carry nothing) |
| scout (Meadow) | Meadow 1 > Tor = Cairn 0.989 > Delta 0.582 > Scarp 0.127 > Range 0.120 |

**Ratios are measured only where pairs are not separated completely.** Every
short-range ratio, and the unarmed-against-armed ratios of the picket and
colonizer beds, are R-TECH6's prior. The rest are bootstrapped; the miner's
General-hull intervals are the widest (0.11–0.35).

**R-TECH7's re-evaluation for this table:** no bed has a cyclic triad. The
largest gap between an observed mean share and the rated one is 0.20 (long-range,
Cairn against Scarp: 0.356 observed, 0.154 rated) and 0.16 (short-range, Tor
against Cairn, where both are the prior's); every other bed is within 0.08.
*Inference:* with one weapon family there is no counter to produce a cycle, so
the scalar rating holds for now; a second family is the observation that would
change this.

**One thing a Design does not reach yet, so the table reads it as a tie:** the
three Limited Designs are one object to the miner bed and near one to the
scout bed (one cost tier, one mass — warfare §8.9.7; they differ only in drive,
0.911 g against 1.00). That is R-O64/R-L0, and the author's long-term goal that
no two named Designs share a loadout (§4.3.1).

### 4.10 `OPEN` — R-TECH15: where the table lives and how it goes stale

*Recommend:* an offline harness writes the table to `data/`, stamped with the
digest of every configuration value the beds read; the harnesses that compute
`T_i` read it and refuse a table whose stamp does not match the running
configuration. The engine does not read it: capability is a measurement, and no
autopilot decision may consult it.

**Cost:** the six-Design pool over all seven beds, both seatings, is 1,680
matches for eight seeds and **7.5 min** on one core (appendix §D.18); the
long-range bed is 42% of it. The round robin is `n(n − 1)` seated matches per
role per seed — 30 for six Designs.

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

**5.4 `OPEN` — R-TECH3: the card surface beyond `UnlockDesign`.** *Advanced:*
the drive factor (§8) is the first write designed past it. Candidates, in
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
table says. That is `AGENTS.md`'s *count the consumers of a write* in its plain
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
| R-O42b | Banks-convention Design names, one class per hull (Spur, Tor, Cairn, Meadow, Delta, Ford, Range, Strait, Scarp) |
| R-TECH6 | one virtual draw per pair, to start (§4.6.2) |
| R-TECH7 | counter-graph intransitivity is accepted; every regeneration reports cyclic triads and the largest residual, and is re-evaluated against the counter-graph then in force (§4.6.3) |
| R-TECH8 | the rating is positive, on the ratio scale, anchor at 1 (§4.7.1) |
| R-TECH10 | a hull carries its **active** role's strength each year (§4.7.3) |
| R-TECH13 | every role has a bed (§4.4.3) |
| R-TECH16 | a bed forces the role (§4.4.4) |
| R-TECH21 | Barrow's scale is loose guidance for this tree and Kardashev's for Production; **card tiers are an instrument of the counter-graph, not of a civilization scale** (§7.2) |
| R-TECH17 | the pool is deduplicated by what the beds read; the goal is that no two named Designs share a loadout (§4.3.1) |
| — | supers are synthesized at pop-Band IV by fixed two-basic recipes |
| — | each super is dominant in different trees; the dominant super is the general key there and the other two are traversal keys — Red-general holds in Red-dominant trees only (law #1, §3.1) |
| — | aggregates are queries, not stored fields |

### Open

| Code | Question | What would settle it |
|---|---|---|
| R-L0 | per-hull slot tables | the arena (design law #4) |
| R-L1 | shield regeneration across what | a decision, with warfare §2 |
| R-O60 / T-04 | magazine mass on ordnance families — **built for missiles (T-139, §2.8)** | torpedoes, when a card builds the family |
| R-O65 | flatten `hull_thrust_to_mass`? | explicit ratification — MC-tuned surface |
| R-O47b / T-08 | `on_refit` is unbuilt | engine work |
| R-CG* | is the counter-graph acyclic? | a design decision |
| R-XM* | apex, and what it is for | a design pass |
| R-TECH1 | **the objective is specified (§4) and not built** — no bed, no table | T-131 |
| R-TECH2 | does capability saturate on the bed? | blocked on R-TECH1 |
| R-TECH5 | fit Bradley–Terry by maximum likelihood on the Elo scale, not sequential updates — *recommended* | author |
| R-TECH9 | weight hulls by dry mass, not count or volume — *recommended*; the author decides on the ratio-scale table, which may already carry mass or volume | author |
| R-TECH11 | equal-spend budget `B` = ten General hulls — *recommended* | author |
| R-TECH12 | `D_long`, and a closing long-range bed | a ranged family or beam falloff, then a sweep |
| ~~R-TECH14~~ | ~~combat beds tie every armed Design~~ — **resolved by T-132**, warfare §8.18 | — |
| R-TECH15 | the table's storage and staleness stamp — `data/design_ratings.tsv`, stamped with the engine commit; no harness reads it back yet | a reader that refuses a stale stamp |
| R-TECH18 | **an unlock reaches no build** — no Design resolver reads the roster. **Advanced (T-139):** the missile card reaches builds through its Doctrine write, which `Standing::design_for(Sentry)` reads; an unlock alone still reaches none (§9.5) | a resolver; T-25 |
| R-TECH19 | the offensive beds' judge read as dry mass **holding the field** (neither wrecked nor withdrawn), horizon 1 yr — *recommended* | author |
| R-TECH3 | the card surface beyond `UnlockDesign` | blocked on R-L0 and R-O65 |
| R-TECH26 | drawing energy from a neutron star (§7.1.3; galaxy R-G7, T-148) | R-G7's choice of where energy enters a mass-conserving model, then a card-workflow pass |
| R-TECH4 | a miniaturization analogue inside this tree? | a decision — *recommend no* |
| T-25 | `enforce_roster` defaults off because there is no unlock path | the card system |
| R-TECH20 | magnetar matter as a Technology option (§7.1.2) | a design pass naming the write and its mass cost |
| R-TECH22 | the drive card's slot, slant and factor `f` — `TIER0[12]` recommended, replacing an unlock that reaches no build (§8.1) | the author; `f` after stage 4 of T-136 |
| R-TECH24 | **the missile card's slot, slant and sentry ratio** — `TIER0[13]` (Balanced) and `ρ = 1e-5` recommended (§9.1, §9.3) | the author; T-140 sweeps `ρ` and its thresholds |
| R-TECH25 | **which super dominates which tree** (§3.1). Structural constraint: each tree's basic feeds exactly two supers (galaxy §4.8, §3.2), so a tree's two cheap supers are fixed by the recipes; whether the dominant super must be one of those two is itself undecided | the author |
| R-TECH23 | where the drive state lives — a `DesignWrite` fold in `CardId` order beside `Works` (recommended) or a `WorksWrite` variant (§8.4) | the author |

---

## 8. The first Technology card — the drive (T-136, design; nothing built)

Taken through `.claude/skills/card-workflow` Stages 0–3. **Every magnitude is a
placeholder and every decision not marked `RATIFIED` is open.**

**8.1 Brief (Stage 0).** The author: *"Obvious first Technology card is one that
improves engines for better acceleration."* *Recommended* slot `TIER0[12]`
(Inscrutable), replacing `UnlockDesign(MediumSystems, Delta)`, which reaches no
build (R-TECH18). Counter-graph edges: not given; the card is the propulsion
half §5.3 says concealment needs, which the cross-tree workflow places.

**8.2 Story and objective (Stage 1).** The Long Dawn's mouth, **First Light**.
The metric is capability-years `T_i` (§4.2). Unlike an unlock, a drive write
reaches every hull built after the play, so it has a path to `Q_i` without the
Design resolver R-TECH18 is waiting on — provided the rating table carries the
upgraded Designs, which it does not yet (8.5, stage 3).

**8.3 Intent as algebra (Stage 2).**

| symbol | name | unit | where it is set |
|---|---|---|---|
| `k` | specific thrust | kt·g per kt of drive | `SimConfig::drive_specific_thrust = 18.21` (R-MC16 anchor) |
| `f` | this empire's drive factor | — | the card; `1.0` before it |
| `m_drv` | a hull's drive mass | kt | `HullType::drive_mass` |
| `M` | dry mass plus load | kt | `thrust_to_mass` |
| `a` | proper acceleration, `G · k · f · m_drv / M` | ly/yr² | `laden_accel` |
| `d` | leg length | ly | — |
| `t` | leg time, `√(d² + 4d/a)` (c = 1) | yr | `math::ship_travel_years` |

- **Every hull of the empire accelerates `f` times harder, empty or laden.** So
  `a_empty / a_laden = 1 + C / M_dry` is untouched (R-O95): the card does not
  change cargo efficiency or the load broadcast (design law #10), only the level.
- **Leg time falls with elasticity `−(2d/a) / (d² + 4d/a)`**, bounded by `−½`
  because the light term `d` does not move. Arithmetic on the formula above,
  not a measurement:

  | hull, leg | `t` at `f = 1` | `f = 1.5` | `f = 2` | elasticity |
  |---|---|---|---|---|
  | laden Medium colonizer, 1 ly | 4.19 yr | 3.47 | 3.05 | −0.47 |
  | laden Medium colonizer, 6.16 ly | 11.84 | 10.30 | 9.44 | −0.37 |
  | laden Medium colonizer, 25 ly | 32.25 | 30.03 | 28.85 | −0.20 |
  | empty Medium, 6.16 ly | 6.93 | 6.68 | 6.56 | −0.11 |
  | empty LCV, 6.16 ly | 8.01 | 7.44 | 7.14 | −0.20 |

  So the gain is concentrated on laden hulls over short legs — freight and
  colonization — and a 1.5x drive cuts any leg by at most 18%.
- **What rivals see:** this empire's acceleration rises by `f` on new hulls, and
  a rival's belief (R-O41, the maximum ever observed) rises when its light
  arrives. Masking stays spend-once.
- **No mass cost** (design law #11 is not touched): `k` is per-mass
  effectiveness, which is what §0 says this tree raises. The price is the
  card's cost.

**8.4 Writes (Stage 3).**

- **A Design write, folded in `CardId` order** like `Works` (float products are
  order-dependent): one per-empire factor `f`.
- **Design law #12 — hulls in the field keep the drive they were built with.**
  `f` is stamped on each hull at construction; `laden_accel` reads the hull's
  stamp, and every forecast of a hull the empire *would* build (a leg priced for
  a build decision, `settler_target`'s discount, the freight score) reads the
  empire's current `f`. One function carries both readings, because a policy
  that prices a leg differently from the engine that flies it chooses against a
  world it does not live in (`thrust_to_mass`'s own comment).
- **Consumers to audit (Stage 4.1):** 13 `thrust_to_mass` call sites and 34
  `laden_accel` call sites, all of which read the drive; the rating beds, which
  must fly the upgraded Design at its own drive.

**8.5 Staged plan (T-136).**

1. **The fold and the per-hull stamp**, bit-identical at `f = 1` on the
   standard bed (a test pins it), and a test that a hull built before the play
   keeps its drive.
2. **The card in its slot**, played at the barrier, with the two-direction
   tests (workflow Stage 6).
3. **Rate the upgraded Designs** in every role bed (`examples/design_rating`) so
   `c(d)` exists for a Design with `f ≠ 1`.
4. **The capability-years harness** that turns the table into `T_i` (T-131's
   missing reader), then the card's value at P92 and its within-tree balance
   against the unlock cards.

**8.6 Voice (Stage 9) — every line a DRAFT for the author.** Role subtitle,
fixed: *Every hull built from now on accelerates harder.*

| context | name (DRAFT) | flavor (DRAFT) |
|---|---|---|
| default | *First Light* | "Every year a ship spends between stars is a year its people are not yet home. We have given some of those years back." — Directorate of Transit Efficiency |
| this empire has played the first Warfare card | *Closing Speed* | "Help that arrives sooner is more help." — Office of Rapid Response |
| this empire has played an Expansion survey card | *The Long Voyage, Shortened* | "The shore has not moved. We have." |

The context rows need R-TREE13 (played cards in `Snapshot`).

**8.7 Open.** R-TECH22 (slot, slant and the magnitude of `f`), R-TECH23 (where
the drive state lives — a `DesignWrite` fold beside `Works`, recommended, or a
`WorksWrite` variant).

---

## 9. The second Technology card — the basic deep-space missile (T-139, built)

Taken through `.claude/skills/card-workflow`. **Every magnitude is a
placeholder.** What the card does to fights and to supply lines is
`Hyades_warfare_tree.md` §8.22; the ordnance book is matching §10.5.

**9.1 Brief (Stage 0).** The author: *"Another tier 1 Technology card should be
basic deep space missile as explored in the arena, but updated to use the new
damage model. … The right Design updates will only change those ships that are
intended for roles a main battle fleet with adequate supply line hardening or to
complete service near a production center. Feels like these should be variation
hulls (cargo at the expense of acceleration) with a subset of available roles
open to them. A long range LOU should stand sentry at production centers."* And:
*"Sentry fleet strength should be built up proportionally to the Holdings and
population increase."* Slot `TIER0[13]` (Balanced), replacing
`UnlockDesign(GeneralSystems, Range)`, which reached no build (R-TECH18). Slot,
slant and magnitudes are R-TECH24. Counter-graph edges: not given; the card meets
the Warfare card's blockade, which is placed by the cross-tree workflow.

**9.2 Story and objective (Stage 1).** The metric is capability-years `T_i`
(§4.2), and no harness reads it (R-TECH1), so the card is measured for **reach**
without a value claim (9.6).

**9.3 Intent as algebra (Stage 2).**

| symbol | name | unit | where it is set |
|---|---|---|---|
| `a_h` | the launcher's empty-hull acceleration | ly/yr² | its drive (R-MC16): 0.94 for a Limited Offensive hull |
| `a` | a round's proper acceleration, `8 · a_h` | ly/yr² | `CombatConfig::missile_accel_multiplier` (arena, tuned) |
| `t_f` | burn time | yr | `missile_fuel_years` = 0.08 (arena) |
| `x(t)` | distance flown, `Δv·t + (√(1 + (a t)²) − 1)/a` | ly | `combat::missile_flight` |
| `R` | reach, `x(t_f)` | ly | 0.0238 from a LOU; a beam's is 7.9e-3 |
| `m_r` | one round's mass | kt | 0.00125 (R-WAR44) |
| `n` | rounds per magazine, 8 per tube | — | R-WAR44 |
| `M` | the Butte's dry mass | kt | 0.020 |
| `ρ` | sentry mass per kilotonne defended | — | `Doctrine::sentry_ratio`; the card writes 1e-5 |
| `D` | a center's holding plus its population | kt | the center |
| `c_s` | one sentry's price, hull plus magazine | kt | 0.030 |

- **Cargo at the expense of acceleration:** a full magazine is `n·m_r = 0.010`
  kt on a 0.020 kt hull, so a Butte flies at `2/3` of its empty acceleration
  full and at its empty acceleration dry. That is R-O95's ratio, with the
  magazine as the load.
- **Three times a beam's reach from the same hull:** `R / 7.9e-3 = 3.0`.
- **Defense in proportion:** a center orders `⌊ρ · D / c_s⌋` sentries, so one
  per 3,000 kt at the placeholder.

**9.4 Writes (Stage 3).**

| write | field | `Standing` question | call sites |
|---|---|---|---|
| `UnlockDesign(LimitedOffensive, Butte)` | the Roster | none while `enforce_roster` is off (R-TECH18) | `roster_permits` |
| `WriteDoctrine(MissileSentries)` | `Doctrine::sentry_ratio` (floor) | `sentries_wanted`, `design_for(Sentry)` | the production policy's fallback, the build order, `assign_role` |
| — (no card) | `Doctrine::missile_pickets` | `design_for(Picket)` → `Mesa` | the room a later card with hardened supply lines writes |
| — (default on) | `Doctrine::point_defense` | `point_defense(role)` | point defense at a round's arrival |
| — (closed) | `Doctrine::ordnance_market` | `trades_ordnance` | the ordnance book (matching §10.5) |

**9.5 Reach audit (Stage 4).**

1. **Consumers.** The Doctrine write is read by the policy's sentry branch
   through `sentries_wanted`, so the card reaches builds: 271 sentries ordered on
   seat 0, seed 1, 3 seats, 800 yr. **This is the first Technology card with a
   path to a build** (R-TECH18).
2. **Defaults against the spec.** `the_sentry_design_is_locked_behind_the_missile_card`
   asserts that no default role is on a missile Design and that no center orders
   a sentry at any defended mass. It then plays the card's writes and asserts the
   proportional count.
3. **Price.** The scratch probe that fixed `R` and the burst also found the two
   policy loops in warfare §8.22.3. The per-round numbers are in 9.3.
4. **Trigger census.** Rounds launched per 800-yr run against a Warfare
   neighbor: 135–306 over 4 seeds (appendix §D.24).
5. **Oracle ceiling.** Not run: there is no metric to read it against (9.2).

**9.6 What it does, measured — reach, not value** (appendix §D.24). 4 seeds,
3 seats, 800 yr. The missile card is on seat 0 and the Warfare card on seat 1 in
both arms. Results are estimates over those 4 seeds:

- 135–306 rounds launched per run; 27–34 hit, 57–75% were shot down and 16–19%
  missed.
- 3–5 seat-1 hulls wrecked by seat 0 per run.
- Seat 0's colonies were **−28.8 ± 6.9** (SE over 4 seeds, 4/4 down) against
  the Warfare-only arm. Colony count is Expansion's metric, and this is recorded
  as a cost the card's seat paid, not as the card's value.

**9.7 Voice (Stage 9) — every line a DRAFT for the author.** Role subtitle,
fixed: *Each center keeps missile sentries in proportion to what it holds.*

| context | name (DRAFT) | flavor (DRAFT) |
|---|---|---|
| default | *Hearth Watch* | "We have built nothing that reaches past our own sky. What we love is here; so are they." — Office of Domestic Assurance |
| a rival has played the first Warfare card | *Porch Light* | "We would never fire first. We have only made sure the first thing anyone sees is how far we can see." |
| this empire has played the first Warfare card | *Both Hands* | "One hand open. The other, as a courtesy, also open, and pointed." |

The context rows need R-TREE13 (played cards in `Snapshot`).

**9.8 Open.** R-TECH24 (slot, slant and `ρ`; the sweep is T-140), R-WAR44
(every missile magnitude), R-WAR48 (a sentry's withdrawal),
R-MX15 (the ordnance book's key and
price).

---

## 7. Technology options and inspirations

*Every item here is `OPEN`: a candidate for what a Technology write could one day
unlock, or a source the tree's design may draw on. None is authorable until the
thing it writes exists (§5.4), and none is a ratified mechanic.*

### 7.1 `OPEN` — options

**7.1.1 Exotic matter** — the families in `Exotic_matter_technology_inspiration.md`
(negative and imaginary mass, degenerate matter, materials at high pressure, the
uncommon condensed states). Bound by §3.5: exotic synthesis is pair production.

**7.1.2 `OPEN` — R-TECH20: magnetar matter — how it differs from regular matter.**
A magnetar's surface field is `~10^10–10^11 T` (Duncan & Thompson 1992; Kouveliotou
et al. 1998). Two thresholds set where matter stops behaving like regular matter,
and both are physics, not placeholders:

| symbol | name | value | what changes above it |
|---|---|---|---|
| `B_0` | atomic field unit | `2.35 × 10^5 T` | the magnetic energy of an electron exceeds its Coulomb binding: atoms are compressed across the field into cylinders along it, bind more tightly, and join into linear molecular chains that condense into a solid of chains (Lai 2001, §§2–4) |
| `B_Q` | quantum-electrodynamic critical field | `4.41 × 10^9 T` | the vacuum itself is birefringent and a photon can split in two (Harding & Lai 2006, §5) |

So the difference from regular matter is **structural and directional**: bonding,
cohesion and the response to light are set by the field's strength and direction
rather than by chemistry alone. **What would settle R-TECH20:** a design pass
saying which Technology write — a component stat, a hull material, a sensor
signature — this becomes, and what it costs under design law #11.

**7.1.3 `OPEN` — R-TECH26: drawing energy from a neutron star.** The
author's idea (T-148, galaxy §4.9's R-G7): a civilization whose whole
technological effort, until recently, went into understanding a neutron star
in its own system and siphoning energy from it. It belongs in the **deep**
tiers of this tree: what a card would write is an inward-precision gain (§0)
reached only through the site. It sits beside R-TECH20, since a magnetar is
a neutron star whose field passes `B_Q`. **What would settle R-TECH26:** R-G7's
choice of where energy enters a mass-conserving model, then a card-workflow
pass naming the Design or Doctrine write, its counter-graph edges (§3) and its
price under design law #11.

### 7.2 `OPEN` — inspirations

- **Iain M. Banks, the Culture novels** — the hull taxonomy and Design names
  (R-O42b; `Hulls_classes_the_qualitative_counter-graph.md`).
- **Exotic matter** — `Exotic_matter_technology_inspiration.md` (§7.1.1).
- **Barrow's inward civilization scale** (Barrow 1998). Where Kardashev ranks a
  civilization by the energy it commands at ever larger scales, Barrow ranks it by
  the **smallest scale it can manipulate**:

  | type | what it manipulates |
  |---|---|
  | I-minus | objects of its own scale — building, mining, joining and breaking solids |
  | II-minus | genes — altering how living things develop |
  | III-minus | molecules and molecular bonds — new materials |
  | IV-minus | individual atoms — nanotechnology |
  | V-minus | atomic nuclei |
  | VI-minus | elementary particles |
  | Omega-minus | the structure of space and time |

  **`RATIFIED` — R-TECH21 (the author's ruling): Barrow's scale is loose
  guidance for this tree, and Kardashev's for Production — nothing more.** §0's
  split (Technology raises per-mass effectiveness, Production raises mass) reads
  as inward precision against outward scale, and that is as far as the analogy
  goes. **Card tiers are an instrument of the counter-graph (§3), not of a
  civilization scale**: a tier is ordered by what it counters and what counters
  it, and a Barrow type never decides where a card sits. The author notes there
  is room in the design space for this and for more cards besides; §7.1's
  options are placed by their counter-graph edges when they are designed.

---

## References

- `Hyades_loadout.md` — the slot model, item families, and the integration plan
- `Hyades_standing_layer_and_observation.md` §2 (concealment by vector), §5 (the
  asymmetric leak), §6.2 (acceleration as the observable), §7 (hull/class/role)
- `Hyades_trees_and_card_value.md` §2.3.5 (capability-years), §2.5, §3.2, §4.3
- `examples/capability_probe` (deleted at T-133: it called the retired resolvers
  directly) — the pool, the per-match cost, beam reach, and the short-range round
  robin (appendix §D.9)
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
- Barrow, J. D. (1998). *Impossibility: The Limits of Science and the Science of
  Limits.* Oxford University Press — the inward civilization scale (§7.2)
- Duncan, R. C. & Thompson, C. (1992). Formation of very strongly magnetized
  neutron stars: implications for gamma-ray bursts. *Astrophysical Journal* 392,
  L9–L13 — magnetars
- Harding, A. K. & Lai, D. (2006). Physics of strongly magnetized neutron stars.
  *Reports on Progress in Physics* 69, 2631–2708 — vacuum birefringence, photon
  splitting
- Kardashev, N. S. (1964). Transmission of information by extraterrestrial
  civilizations. *Soviet Astronomy* 8, 217–221 — the outward scale Barrow inverts
- Kouveliotou, C. et al. (1998). An X-ray pulsar with a superstrong magnetic field
  in the soft γ-ray repeater SGR 1806−20. *Nature* 393, 235–237 — a measured
  magnetar field
- Lai, D. (2001). Matter in strong magnetic fields. *Reviews of Modern Physics*
  73, 629–662 — atoms, chains and condensed matter above `B_0`
- `Exotic_matter_technology_inspiration.md` ·
  `Hulls_classes_the_qualitative_counter-graph.md` (the Banks-convention source)
- `src/sim.rs` — `Roster`, `Class`, `HullType`; `src/cards.rs` — `UnlockDesign`
- AGENTS.md design laws #1, #2, #9, #10, #12, #13
