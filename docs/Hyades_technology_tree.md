# Hyades — Technology: Design, Components, and Capability

*The design space for **Technology**, the tree that raises what a given mass of
fleet can *do*. Its objective is **capability-years**
(`Hyades_trees_and_card_value.md` §2.3.5). Companion to `Hyades_loadout.md` (the
slot and item model), `Hyades_standing_layer_and_observation.md` (Design as
standing-layer state, and the asymmetric leak), `Hyades_warfare_tree.md` (what
capability is spent on) and `Exotic_matter_technology_inspiration.md`. New calls
continue the **R-TECH n** series.*

**Rev 1, new.** Carries **ratified decisions and open decisions only**
(`CLAUDE.md` §6). This is the **least-built** of the six trees: the standing-layer
half is ratified and shipped (`Roster`, `Class`, `UnlockDesign`), the loadout
model is specified and unbuilt, and the objective is a proposal with a knob in it.
Most of this file is `OPEN`, and it says so per item rather than in the prose.

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

**1.6 `OPEN` — R-O42b: the class flavour names.** Meadow for the LSV and Tor for
the LCV are proposed, scaling the Banks convention down to Limited sizes. Flavour
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

**3.2 `RATIFIED` — supers are synthesised, never mined, and only at pop-Band IV.**
Fixed two-basic recipes: `Blue ← C+M`, `Red ← M+Y`, `Green ← Y+C`. Each archetype
is rich in two basics and poor in the third — **the two precursors of its single
native super** — so every empire self-synthesises exactly one and must acquire the
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

**3.7 `OPEN` — R-XM*: apex, and what it is for.** Synthesised from supers. Present
in `resources.rs` and in nothing else.

---

## 4. The objective — capability-years

**4.1 `RATIFIED in form, OPEN in content` — the objective is an integral of a
capability stock.**

```text
T_i = ∫₀^T Q_i(t) dt
```

Every tree's objective is an `X`-years, which is the form colony-years already has
and the reason it is the guard the project trusts: **count at a horizon is a weak
invariant; the integral falls the moment anything slows down.**

**4.2 `OPEN` — R-TREE4: `Q_i`'s axes, weights, references and aggregator.**
The proposal, and the reasoning matters more than the constants: capability is a
per-mass effectiveness times the mass, naturally a **vector**:

| axis | meaning | measurable as |
|---|---|---|
| projection | deliverable combat mass at range | combat mass × reach under `a_max` within a response window |
| defense | combat mass within response time of owned colonies | the same, evaluated against own holdings |
| acquisition | ore delivered per year | freighter deliveries — **already logged** |

Aggregate with a **power mean**, not a sum and not a hard minimum:

```text
Q = ( Σ_a  s_a · (q_a / q_ref,a)^ρ ) ^ (1/ρ)
```

- A **sum** (`ρ = 1`) lets Technology farm whichever axis is cheapest.
- A **hard Liebig minimum** (`ρ → −∞`) matches the economic thesis — the best war
  machine needs all three basics and all three supers, which *is* a minimum — and
  forces the cross-tree engagement design law #7 requires.
- But a hard minimum makes a card's value depend entirely on whether it happens
  to raise the **currently weakest** axis, which is enormous variance, and
  `Hyades_trees_and_card_value.md` §4.3 requires tier-1 cards to have the
  *lowest* dispersion of any tier.

So **`ρ` is a measured knob**, and "how Liebig is capability" becomes one number
to ratify rather than a binary to argue about. *Recommend* starting near
`ρ = −1` (harmonic mean: strongly penalises a weak axis without zeroing the card
that missed it). `s_a`, `q_ref,a` and `ρ` are all placeholders.

**4.3 `OPEN` — R-TECH1: capability is undefined today and the tree therefore has
no objective at all.** `examples/tree_gradient` **excludes** Technology from its
composite for this reason, and says so — a composite over an unstated subset is
worse than a single metric. Instrumenting `Q_i` is the prerequisite for measuring
a single Technology card.

**4.4 `OPEN` — R-TECH2: does capability saturate on the measurement bed?**
`Hyades_trees_and_card_value.md` §3.2 requires this as the *first* measurement.
Unknown, because 4.3.

---

## 5. What a Technology card writes

**5.1 `RATIFIED` — the write surface that exists.** `CardEffect::UnlockDesign(
HullType, Class)` adds a roster entry. That is the whole of it today, and it is
real: the engine stores it, `enforce_roster` reads it, and a disclosure publishes
it.

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
| — | supers are synthesised at pop-Band IV by fixed two-basic recipes |
| — | Red is the general key; Blue and Green are traversal keys (law #1) |
| — | aggregates are queries, not stored fields |

### Open

| Code | Question | What would settle it |
|---|---|---|
| R-L0 | per-hull slot tables | the arena (design law #4) |
| R-L1 | shield regeneration across what | a decision, with warfare §2 |
| R-O60 / T-04 | magazine mass on ordnance families | engine work |
| R-O65 | flatten `hull_thrust_to_mass`? | explicit ratification — MC-tuned surface |
| R-O42b | class flavour names | the author |
| R-O47b / T-08 | `on_refit` is unbuilt | engine work |
| R-CG* | is the counter-graph acyclic? | a design decision |
| R-XM* | apex, and what it is for | a design pass |
| R-TREE4 | `Q_i`'s axes, weights and `ρ` | instrument, then MC |
| R-TECH1 | **the tree has no objective until `Q_i` exists** | R-TREE4 |
| R-TECH2 | does capability saturate on the bed? | blocked on R-TECH1 |
| R-TECH3 | the card surface beyond `UnlockDesign` | blocked on R-L0 and R-O65 |
| R-TECH4 | a miniaturization analogue inside this tree? | a decision — *recommend no* |
| T-25 | `enforce_roster` defaults off because there is no unlock path | the card system |

---

## References

- `Hyades_loadout.md` — the slot model, item families, and the integration plan
- `Hyades_standing_layer_and_observation.md` §2 (concealment by vector), §5 (the
  asymmetric leak), §6.2 (acceleration as the observable), §7 (hull/class/role)
- `Hyades_trees_and_card_value.md` §2.3.5 (capability-years), §3.2, §4.3
- `Hyades_warfare_tree.md` — what capability is spent on, and the arena that sets it
- `Hyades_production_tree.md` §5 (the hull ladder this fits onto)
- `Exotic_matter_technology_inspiration.md` ·
  `Hulls_classes_the_qualitative_counter-graph.md` (the Banks-convention source)
- `src/sim.rs` — `Roster`, `Class`, `HullType`; `src/cards.rs` — `UnlockDesign`
- CLAUDE.md design laws #1, #2, #9, #10, #12, #13
