# Hyades — Mineral Cost Curve: Permutation Ratios & Hull Scaling

**Scope.** This spec turns the locked mineral/super/Platinum permutations
(A/B/C×D tree framework, color-transposition domain table) into actual
numbers: how much of each color a card costs, and how hull class modulates
that cost against a ship's value — including where that value math breaks
down (§2.5) and what empirical harness (§6) resolves it. It does **not**
define the counter-graph itself — which ship+class dominates which at
equivalent mineral cost remains open work per `Hulls & classes; the
qualitative counter-graph`. This is the cost infrastructure, and the
measurement tool, that work will sit on top of.

---

## 1. Card costs — the permutation-ratio system

Each tree's locked permutations name *which* colors a card draws on, in
priority order. This section specifies *how much* of each.

### 1.1 Mineral (CMY) ratio

A card's total mineral cost `T` splits across the tree's own
(primary, secondary, tertiary) ranking — the locked full C/M/Y
ordering — in a ratio `a:b:c` (a ≥ b ≥ c), chosen per node along a
continuum:

| Point | Ratio | Spread (a/c) | Reads as |
|---|---|---|---|
| **Peak** | 4:2:1 | 4.00 | pure — leans hard into the tree's own domain color |
| **Default** | 3:2:1 | 3.00 | the anticipated, typical case |
| **Floor** | 5:4:3 | 1.67 | flattest Hyades permits — never a true 1:1:1 |

`primary = round(T·a/(a+b+c))`, `secondary = round(T·b/(a+b+c))`,
`tertiary = round(T·c/(a+b+c))`.

**Design guidance (R-MC1, open):** peak ratio for signature, single-tree
cards — they should feel expensive off-domain and cheap in it. Floor ratio
for cross-tree combo cards, since the combo-backbone law (`Hyades_command_
cards.md` §4) already requires these to not overpunish any one domain — a
combo card meant to be playable from several builds shouldn't be
domain-locked by its own cost curve. Where a given node sits on this
continuum is a per-card/per-node tuning call, not fixed here.

### 1.2 Super ratio

Each tree has exactly one native super (the R-CG16/Option-B assignment),
one **domain-sharing foreign** super (shares the tree's own domain mineral),
and one **zero-sharing foreign** super (shares nothing):

| Tree | Domain | Native (1st) | Domain-sharing (2nd) | Zero-sharing (3rd) |
|---|---|---|---|---|
| Warfare | Magenta | Red | Blue | Green |
| Technology | Magenta | Blue | Red | Green |
| Growth | Cyan | Blue | Green | Red |
| Politics | Cyan | Green | Blue | Red |
| Expansion | Yellow | Green | Red | Blue |
| Production | Yellow | Red | Green | Blue |

This ranking isn't chosen — it's forced by the color algebra
(`resources.rs`): the 2nd-place super is whichever of the two non-native
supers still contains the tree's own domain mineral, and the 3rd is the one
that doesn't.

Ratio band: **3:2:1 to 4:2:1** (spread 3.00–4.00) — narrower than minerals,
and its *floor* is minerals' *peak*. Supers never flatten toward the
mineral floor of 1.67, because `Hyades_galaxy_and_autopilot.md` §4.6
requires supers to stay "non-interchangeable specialists" even at their
least differentiated. A super-tier card is always more domain-locked than a
mineral-tier one at the same node depth.

### 1.3 Platinum ratio

The locked Platinum mixture — `[domain mineral · 2nd-ranked mineral ·
Platinum]` — splits **~4:2:1, held close to constant** (Platinum : primary
mineral : secondary mineral). Platinum should dominate an apex card's cost
essentially always, not just at a design's sharpest setting — it's the
scarcest tier by construction, so its cost-share shouldn't visibly soften
the way mineral and super ratios are allowed to. **R-MC2 (open):** how much
variance "close to constant" actually permits — a fixed value, or a tight
band like 4:2:1 to 4.5:2:1.

**This is a different ratio from "3 CMY = 2 RGB = 1 Platinum."** That line
is a fleet-*value*-equivalence heuristic, not a card's internal cost split
— see §5. The two happen to rhyme numerically; they aren't the same claim.

---

## 2. Hull costs — surface area vs. volume

### 2.1 The scaling law — confirmed non-self-similar by design

**Ratified this conversation: hull classes do *not* scale self-similarly.**
General hulls are **spheroids**, Medium/Rapid hulls are **ellipsoids**,
Limited hulls are **cylinders** — three genuinely different shapes, chosen
specifically so that value-per-mineral-spent grows **super-linearly** in
size, not just linearly. This supersedes the self-similar treatment in the
first draft of this spec outright.

**Why changing shape (not just size) buys super-linear growth.** The
isoperimetric inequality guarantees the sphere has the *least* surface area
of any solid enclosing a given volume — any elongation away from a sphere
*costs* surface area for the same volume. Define **shape efficiency**
`η = SA_sphere-of-equal-volume / SA_actual` (η = 1 for a true sphere, η < 1
for anything else). Since `V/SA_sphere-of-equal-volume = r_eq/3` (r_eq being
that equal-volume sphere's radius), any shape's value-per-cost is:

```
V/SA = η(shape) × r_eq/3
```

Holding shape fixed, this is the old linear-in-`r_eq` law. But if the shape
*itself* gets rounder (higher η) at the same time `r_eq` grows — which is
exactly what cylinder → ellipsoid → spheroid does — **both factors in the
product increase together**, and the combined growth outpaces `r_eq` alone.
That's the mechanism: not a different exponent, but two increasing factors
multiplying instead of one.

**Worked example** (illustrative axis ratios — the actual per-class values
are a separate design choice, R-MC3a):

| Class | Shape | Example axes | η (shape efficiency) |
|---|---|---|---|
| Limited | Cylinder | radius ρ, length ≈ 4 diameters (matches the Gangster-class flavor source, 200 m × 50 m) | **≈ 0.73** |
| Medium/Rapid | Triaxial ellipsoid | semi-axes 3:2:1 | **≈ 0.85** |
| General | Spheroid | polar axis 1.5× equatorial | **≈ 0.97** |

(Cylinder: `SA = 2πρ²(1+2κ)`, `V = 2πκρ³` for aspect ratio `κ = L/2ρ`;
`η = 2(1.5κ)^(2/3)/(1+2κ)`, giving 0.73 at κ=4. Spheroid: exact trigonometric
surface-area formula for a prolate spheroid gives η ≈ 0.97 at a 1.5×
elongation. Triaxial ellipsoid has no elementary closed form — Knud
Thomsen's approximation, `SA ≈ 4π((ab)^p+(ac)^p+(bc)^p)^(1/p)/3^(1/p)` with
`p ≈ 1.6075`, accurate to within 1.061% for any axis ratio, gives η ≈ 0.85
at 3:2:1. See references.)

Illustrating the compounding with placeholder sizes `r_eq = 1, 2, 3`:

| Class | η | r_eq | V/SA = η·r_eq/3 | Step-up vs. previous |
|---|---|---|---|---|
| Limited | 0.73 | 1 | 0.243 | — |
| Medium | 0.85 | 2 | 0.567 | **×2.33** (pure-linear would be ×2.00) |
| General | 0.97 | 3 | 0.970 | **×1.71** (pure-linear would be ×1.50) |

Both step-ups beat the pure-linear baseline — a direct, checkable
demonstration of the super-linear growth this shape scheme was built to
produce, independent of exactly which `r_eq` values are eventually chosen.

**R-MC3a (open):** pin actual per-class semi-axis ratios (the example
values above are illustrative, not specified). **R-MC3b (open):** pin actual
per-class `r_eq` (overall size), which together with R-MC3a sets the real
cost ratios in §2.3.

### 2.2 Role is a second sphericity axis, on top of size

**Ratified this conversation: sphericity varies by role, not just by size
class.** In decreasing roundness: **Systems Vehicles > Contact Vehicles/
Units > Offensive Units.** A General Offensive Unit is *not* as spherical as
a General Systems Vehicle — both are "General" in the size-tier sense, but
Offensive's whole role sits at a lower η than Systems' at every tier. **A
General Systems Vehicle is a literal sphere** (η = 1.0 exactly) — the anchor
point the other nine hull types are measured against.

This makes shape efficiency a function of two things, not one:
`η(role, size)`, decreasing as you move down either axis:

| Role (roundest → least round) | General | Medium/Rapid | Limited |
|---|---|---|---|
| **Systems** | **1.000** (sphere, by ratification) | ≈0.97–0.99 (mild spheroid) | ≈0.86 (modest cylinder) |
| **Contact** | ≈0.95–0.97 | *(no Medium tier — only Limited/General exist, `sim.rs`'s `HullType`)* | ≈0.75 (moderate cylinder) |
| **Offensive** | ≈0.93 (spheroid, λ≈2) | **≈0.73 at the Gangster anchor** (real data: 200 m × 50 m, κ=4) | ≈0.64 (κ≈7, elongated cylinder) |

**The ROU row isn't one number — it's a range**, per this conversation:
*the fastest ROU classes are elongated to roughly a Limited Offensive
Unit's ratio* (κ high, ~7, η≈0.64 — thin and slippery, prioritizing
acceleration over volume), while *more heavily armed ROU classes
foreshorten toward the General Offensive end* (κ lower, ~2–3, η rising
toward ≈0.85–0.93 — bulkier, more room for weapons/armor mass). The
Gangster-class real data (κ=4, η≈0.73) reads as roughly the *center* of that
range, not either extreme.

This is the first hint that **a hull's shape shouldn't be a fixed constant
per `HullType` at all — it should be a function of loadout mix.** More
ENG-slot fraction (`Hyades_loadout.md` §3.1) → more elongated (faster, less
frontal cross-section); more WPN/ARM/SHD-slot fraction → more foreshortened
(bulkier, more internal volume per unit length). **R-MC11 (open):** formalize
`κ(loadout)` or `λ(loadout)` as an actual function of slot composition
rather than a fixed per-`HullType` lookup — this is a real engine change,
not just a cost-curve tuning number, since it means shape (and therefore
cost) becomes computed from a ship's specific build rather than looked up
from its `HullType` alone.

### 2.3 Reconciling with the existing placeholder — superseded, and calibrated

**R-MC4: resolved.** The 1:3:9 mineral-cost ratio and the 0:1:2 cargo ladder
are both superseded outright, per this conversation — they were placeholders
in their source docs, and §2.1–§2.2's geometry now determines both instead
of a flat lookup.

**Working the actual numbers, three ways.**

*1. Solving backward from the current MSV cost.* Since
`SA = 4π·r_eq²/η`, holding `general_vehicle_cost` fixed and asking "what
`r_eq` ratio would reproduce the *existing* 1:3:9 cost fractions, given the
η values in §2.2's Systems row?" — `(r_eq_M/r_eq_G)² = η_M/3` and
`(r_eq_L/r_eq_G)² = η_L/9`. Using η_M≈0.97, η_L≈0.86 (Systems row, §2.2):
`r_eq_M/r_eq_G ≈ 0.57`, `r_eq_L/r_eq_G ≈ 0.31`. In other words, a General
Systems Vehicle need only be about **1.8× a Medium's linear
(equal-volume-sphere) size, and 3.2× a Limited's**, for the current cost
placeholder to already be geometrically consistent. That's a real,
actionable target — not a range, a specific ratio — if the goal is to keep
existing balance while adopting real geometry underneath it.

*2. Checking that against the literal Culture-class descriptions.* The
Culture wiki excerpts (`Hulls & classes` doc) give real numbers for two
Systems-row anchors: the Plate-class GSV at 50 km × 20 km × 4 km, and the
Desert-class MSV at "slightly over 3 km" long. Treating the Plate-class's
literal dimensions as an ellipsoid and converting to GSV's ratified
sphere-of-equal-volume gives `r_eq(GSV) ≈ 7.9 km`. Giving Desert-class a
modest Systems-row elongation (λ≈1.4) off its 3 km length gives
`r_eq(MSV) ≈ 1.2 km`. That's a **linear ratio of roughly 6.6, not 1.8** —
and because cost scales with the *square* of that ratio, the literal
Culture-fiction size gap implies a Medium-to-General cost fraction near
**1/43**, not 1/3. **This is the central finding: Culture's in-universe
scale gap between GSV and MSV is roughly an order of magnitude too extreme
to use literally as a game-balance target.** It's excellent shape/flavor
inspiration (§2.1–§2.2's aspect ratios lean on it directly) but its
*absolute* size gaps read as narrative grandeur, not a cost curve — using
them verbatim would make Medium and Limited hulls dramatically cheaper
(proportionally) than anything currently balanced around.

*3. The decision this surfaces.* **R-MC9 (open, the real remaining
question):** pick a lane —
  - **(a) Balance-preserving:** adopt the ~1.8 / 3.2 linear ratios from
    calculation 1, keeping cost fractions close to the current 1:3:9 feel
    while running on real geometry underneath.
  - **(b) Fiction-faithful:** adopt something closer to Culture's literal
    scale gap, accept that Medium/Limited hulls become dramatically cheaper
    relative to General than they are today, and re-tune `medium_fleet_size`/
    `limited_fleet_size` upward (toward the 40s/hundreds) to match — a much
    bigger swing in actual play than a spec change should make unilaterally.
  - **(c) Something between,** chosen by playtest feel rather than either
    anchor.

  This spec doesn't pick for you — it computes what each lane actually
  costs, which is the concrete deliverable asked for.

**Cargo capacity is volume, not a separate ladder — R-MC5, resolved
differently than the first draft proposed.** Per this conversation, cargo
*is* meant to scale with volume; Limited's zero isn't a special-cased floor,
it's what volume-scaling *predicts* once a fixed "reserved" volume (engines,
structure, crew — the non-cargo baseline every hull needs regardless of
size) is subtracted out: `cargo ∝ max(0, V − V_reserved)`. Setting
`V_reserved` equal to a Limited hull's *entire* volume (i.e., Limited hulls
are all reserved space, no surplus — matching the existing "Limited = 0"
fact exactly, not overriding it) and applying the lane-1 (balance-preserving)
`r_eq` ratios from above: `V_GSV : V_MSV : V_LSV ≈ 33.9 : 6.3 : 1` (volume
scales with the *cube* of the linear ratios, so even lane 1's modest 1.8×/
3.2× linear gaps become a much bigger volume gap). Subtracting `V_LSV` as
the reserved baseline: **cargo(GSV) : cargo(MSV) ≈ 6.3 : 1** — General
should carry roughly **six times** Medium's cargo, not the current ladder's
flat 2×. This is exactly "General having substantially more room than
Medium," now with a number attached, and it's a direct, checkable
consequence of the same geometry rather than an independently chosen ladder.

**A second, free consequence: `dry_mass` shouldn't be flat either.**
`SimConfig::dry_mass` is currently a single placeholder value (1.0) applied
to *every* `HullType` regardless of size (`sim.rs`) — a bigger gap than the
cargo ladder, since it means the engine currently can't tell a GSV's mass
from a Scout's. Assuming roughly uniform hull density, `dry_mass ∝ V`, and
the lane-1 volume ratios above give **`dry_mass(GSV) : dry_mass(MSV) :
dry_mass(LSV) ≈ 33.9 : 6.3 : 1`** directly — a ready-to-use replacement for
the flat placeholder, and one that immediately feeds the existing
acceleration formula (`a = thrust/(dry_mass + cargo_mass)`,
`Hyades_loadout.md` §3.1) with real differentiation instead of none.
**R-MC10 (open):** confirm this reading of `dry_mass` (proportional to hull
volume) is correct, and pick the actual `V_reference` (what volume equals
`dry_mass = 1.0`) needed to convert these ratios into concrete numbers.

**Cross-role sizing is still genuinely open.** `HullType::cost_fraction`
currently gives *every* "General" hull — Systems, Contact, Offensive alike
— the same cost (`1.0 × general_vehicle_cost`), regardless of role. §2.2
establishes that Offensive is systematically less spherical than Systems at
every size tier, but that alone doesn't say whether a General Offensive
Unit is *smaller* than a General Systems Vehicle in absolute terms (which
the Culture fiction's GSV-vs-GOU gap — 50 km vs. ~2–3 km — strongly implies)
or the same size and simply less efficient. **R-MC12 (open):** this is a
second instance of the same lane-1-vs-lane-2 choice from R-MC9, now applied
across roles rather than within Systems alone, and it's the piece needed
before Contact/Offensive hulls get real numbers rather than the illustrative
η values in §2.2.

### 2.4 Class modulates the permutation, it doesn't replace it

A hull's mineral cost still runs through the §1 permutation machinery — the
class scale factor is a multiplier on top of the permutation-derived base,
not a competing system:

```
hull_cost(tree, hull_type) = permutation_cost(tree, ratio) × class_scale(hull_type)
```

where `class_scale(hull_type) = SA(hull_type) / SA(reference)` — now the
full §2.1–§2.3 shape-and-role-dependent surface area, normalized against
General Systems as the reference (`class_scale = 1.0`), not a bare `r²`.
This keeps the domain-color cost spine (§1) fully intact at every hull size
and role; class only scales the total, it never reshuffles which color is
cheap for whom.

### 2.5 The consolidation penalty — the mathematical form of the gap you flagged

**Diagnosed, not just acknowledged: under geometry alone, consolidation
always wins, and by a derivable amount.** Hold shape (η) fixed and ask what
happens to total mineral cost when a fixed total volume `V` is split into
`N` equal hulls instead of built as one. Each piece has volume `V/N`, hence
`r_eq(V/N) = r_eq(V)/N^(1/3)`, hence `SA` per piece `= SA(V, one hull) /
N^(2/3)`. Summed over `N` pieces:

```
Total_SA(N pieces) = SA(V, one hull) × N^(1/3)
```

Splitting one hull into 8 equal pieces costs **2× the mineral**, not 8× and
not 1× — this is the same relationship that governs why crushing a solid
into powder increases its total surface area at constant mass (a real,
measured effect in materials science — see references). Inverting: **for a
*fixed* mineral budget, the total volume achievable shrinks as `N^(−1/2)`**
as you split it across more hulls. A hundred equal-cost Limited hulls
deliver only about **1/10th the total volume** of one equivalent-cost
consolidated hull — before cargo's reserved-volume penalty (§2.3) is even
applied. Add that in — each of the `N` hulls pays its own `V_reserved`,
not a shared one — and total cargo falls *faster* than `N^(−1/2)`, potentially
to zero once `N` is large enough that every hull is all overhead. **This is
exactly the effect you flagged: under §1–§2.4 alone, there is no volume- or
cargo-derived quantity for which a swarm of small hulls ever beats an
equivalent-cost consolidated hull. Geometry has only one lane, and it always
points toward consolidation.**

**The gap is real, and the fix can't come from more geometry — it has to
come from combat specifically.** The project's own flavor doc already names
the missing counterweight without formalizing it: a GSV is "an easy target"
and, unlike an equivalent-value GOU fleet, "lacks mission flexibility since
it cannot split" (`Hulls & classes` doc). Two mechanisms live outside pure
volume-accounting entirely:

- **Lanchester's square law** (already a project reference,
  `Hyades_command_cards.md`): under aimed-fire attrition, fighting strength
  scales with the *square* of the number of independent firing units. A
  single GSV is always `N=1` no matter how voluminous; a hundred LSVs are
  `N=100` and can concentrate fire in a way one hull structurally cannot.
  This is a real counterweight to the `N^(1/3)` cost penalty above — the
  two need to be measured against each other, not reasoned about in the
  abstract.
- **Indivisibility as a liability, not just a flavor line.** One hull is a
  single point of failure; a fleet degrades gradually. This has no clean
  closed-form the way §2.5's cost math does — it depends on the actual
  wreck-roll and engagement mechanics (`Hyades_simulation_model.md` §4),
  which are specified but not yet implemented.

**Neither of these is solvable from a formula.** They depend on how
engagements actually resolve — which is exactly why R-MC3b needs a
playtest arena rather than a derivation, and why R-MC9's lane choice
should follow the arena's results rather than precede them.

Each A/B/C group's two trees cover exactly two of the three supers between
them (established this conversation) — meaning each group has exactly one
**group-missing** super that *no* tree in that group natively touches:

| Group | Trees | Natives covered | Group-missing |
|---|---|---|---|
| **Kinetic** (A) | Warfare, Expansion | Red, Green | **Blue** |
| **Potential** (B) | Technology, Production | Blue, Red | **Green** |
| **Latent** (C) | Growth, Politics | Blue, Green | **Red** |

**Proposal:** layer a group-level premium on top of the tree-level §1.2
premium — building a hull with the group-missing super costs *more* than
§1.2 alone would predict, reflecting that no tree in that whole strategic
family has ever natively invested in that color's infrastructure.

This is a genuinely second layer, not a restatement of §1.2, and it isn't
uniform across the six trees. Latent is exactly the whole Cyan domain, so
for both Growth and Politics the group-missing super (Red) *is* their own
individual zero-sharing super (§1.2) — the two penalties stack on the same
target. Kinetic and Potential each blend one Magenta-domain and one
Yellow-domain tree, so only one member of each pair gets that
double-stacked penalty (Expansion for Kinetic, Technology for Potential);
the other (Warfare, Production) has a group-penalty that lands on a
*different* super than its own individual worst one. **R-MC6 (open):** size
the group-level premium, and decide whether the doubled-penalty cases
(Growth, Politics) should be capped rather than fully additive.

---

## 4. Counter-graph legibility — directional only

The existing `Hulls & classes` doc defines the counter-graph as which
ship+class dominates which other ship+class **at equivalent mineral cost**.
The intent from this conversation: as a player's mineral supply shifts
(new territory, a synthesis chain coming online, a raid), the *correct*
counter-build against a known enemy fleet-in-being should become
legible directly from what's currently cheap for that player — not require
consulting an external matchup chart. Concretely, this means the §1–§3 cost
machinery should be built so that "what beats what" and "what's cheap for
me" tend to move together often enough that the dominant response is
visible at a glance, not so tightly that the counter-graph collapses into
pure mineral-richness (which would make combat matchups a foregone
conclusion rather than a real decision).

**Not solved here, by design** — per this conversation, the counter-graph's
actual ship+class dominance table is separate, future work. This section
only records the target property so §1–§3 can be checked against it once
that table exists.

---

## 5. Mineral tier as a second, independent axis

**"3 CMY = 2 RGB = 1 Platinum" is not a cost ratio — it's an
order-of-magnitude value heuristic, and it lives on a different axis than
§2's hull size entirely.** Per this conversation: it doesn't mean 3 units of
base-mineral spend buys a fleet of equal value to 2 units of super spend or
1 unit of Platinum spend. It means something closer to *three orders of
magnitude* of base-mineral spend sits alongside *two orders of magnitude*
of super spend and *one order of magnitude* of Platinum spend — without
committing to literal powers of ten or any single fixed progression. The
concrete example given: a negative-mass keel (an exotic, Super-or-higher
component) should improve force projection so much via better acceleration
that a fleet an order of magnitude smaller **in mass** matches a
base-mineral fleet's value.

**Size (§2) and tier (this section) are orthogonal.** A hull's shape/size
class (Limited/Medium/General) sets its `V/SA` efficiency. Its material
tier (Base/Super/Platinum) is a *separate* question: how much combat value
a given mass of hull delivers, once built from exotic rather than ordinary
matter. A "Super-tier Limited fleet" and a "Base-tier Medium fleet" aren't
directly comparable through §2 alone — that comparison is the actual
counter-graph question, and it needs a mechanism, not just a ratio.

**The mechanism already exists in the engine: acceleration.**
`Hyades_loadout.md` §3.1 defines `a = total_thrust / (dry_mass +
cargo_mass)` — effective mass is already load-bearing for combat-relevant
performance. An exotic mass-reducing component (the negative-mass keel)
doesn't need a new abstract "value multiplier" bolted on; it's a direct
multiplier on `a` for the *same* thrust and *same* nominal hull. That's a
real mechanical lever this spec can hand to card design, rather than an
arbitrary "supers are worth 1.5× as much" number.

**Lanchester's square law is the right tool for turning a per-unit
multiplier into a fleet-mass equivalence** — it's already in `Hyades_
command_cards.md`'s own reference list, and it's built for exactly this
question. Under aimed-fire attrition, relative fighting strength scales
with the *square* of numbers (or, equivalently, of per-unit effectiveness):
to match a baseline fleet, a fleet with `k×` the per-unit effectiveness only
needs `1/√k` the numbers (or mass, if mass is the scarce resource being
compared). So:

```
mass_ratio_for_equal_value = 1 / √(effectiveness_multiplier)
```

Hitting a full order-of-magnitude mass reduction (0.1×, matching the
negative-mass-keel example) needs an effectiveness multiplier of **100×**
— two orders of magnitude of per-unit combat effectiveness to buy one order
of magnitude of mass. That's a real, checkable target for whatever stat the
negative-mass keel (or any Super/Platinum exotic component) actually grants
— acceleration, evasion, alpha-strike damage, whatever the combat model
ends up rewarding — rather than a number this spec invents unilaterally.

**R-MC7 (open):** confirm effectiveness-multiplier ↔ mass-ratio via
Lanchester's square law is the intended mechanism (vs. linear, vs. some
other attrition model — Lanchester's *linear* law applies to unaimed/area
fire instead, and would need only `1/k` mass reduction for the same `k×`
effectiveness, a much less dramatic result). **R-MC8 (open):** once R-MC7
is settled and combat mechanics are further along, specify what stat(s)
"effectiveness" actually reads off (a single number, or several
combat-model terms multiplied together) — this is the concrete "how
powerful is a Super-class Limited fleet vs. a Super-class Medium fleet"
question, and it can't be fully answered until the combat model (currently
unimplemented, per `sim.rs`) exists to plug into.

---

## 6. The Ship Testing Arena — requirements

**Resolves R-MC3b, and reframes R-MC9.** Per this conversation: `r_eq` per
class is set empirically, via a dedicated combat sandbox, not derived from
geometry or calibrated against the existing cost placeholder alone (§2.3's
two "lanes" were both geometry-only estimates — this is the actual
tie-breaker between them, and may land somewhere neither predicts).

### 6.1 What it is

A **stripped-down simulation scenario**: no planets, no production, no
mining, no colonization — none of `montecarlo.rs`'s economy layer. Just:

- **Starting distance** between two (or more) fleets.
- **Starting velocity** for each fleet (closing, receding, or orthogonal).
- **Fleet compositions** — specific counts of specific hull types and
  roles, assigned per side.

This is a scenario-configuration layer on top of the *existing* engine, not
a new one — it reuses the same discrete-event `Simulation`/`SimConfig`
machinery `montecarlo.rs` already drives, just seeded with two placed
fleets instead of a generated galaxy.

### 6.2 What it depends on — the real prerequisite list

**This arena cannot run against an empty combat model — building it
*is* the first real implementation of combat**, not a test of an existing
one. `Hyades_simulation_model.md` §4–5 already specifies what's needed:
acceleration-governed approach, engagement at weapon range with no
initiative (simultaneous resolution by geometry, not turn order), damage by
stat/position/formation, and the per-ship wreck roll as the sole stochastic
element. `Hyades_loadout.md` §6 specifies *how* it schedules:
`sys_engagement` fires on spatial-proximity events on the existing
discrete-event queue, not a combat-round clock. None of this is undesigned
— it's designed and unimplemented, which is a smaller gap than starting
from nothing, but still a real one. Concretely, the arena is blocked on:

- **R-L0** (`Hyades_loadout.md`) — concrete per-hull slot tables. Without
  these, hulls have geometry (this spec) but no weapons/armor/engines to
  actually fight with.
- **R-L1** — shield behavior between engagements (resets vs. regenerates).
- **R-L2** — single-closing-pass vs. repeated-pass engagement resolution —
  this one directly shapes whether a numerous fleet's Lanchester advantage
  (§2.5) can even manifest, since repeated passes are what let concentrated
  numbers compound their effect across multiple exchanges.
- **R-L5** (`Hyades_loadout.md` §7) — already asks the same question this
  spec's R-MC10 answers (`dry_mass` per hull-type, not flat); resolving
  R-MC10 closes R-L5 as a side effect.

### 6.3 What it measures

For a fixed mineral budget, split into (a) one General-class hull and (b) N
equal-cost Limited-class hulls of the same tree/role: run the engagement
at a matrix of starting distances and starting velocities, and record —

- **Outcome** (which side is eliminated, or neither within a time bound).
- **Survivor count/fraction** on the winning side.
- **Time-to-resolution.**
- **Damage dealt vs. received**, to separate "won but gutted" from "won
  clean."

Sweep `N` (how fragmented side (b) is) and the candidate `r_eq` values from
§2.1–§2.3 together — the target isn't a single winner, it's the *shape* of
the boundary: at what distance/velocity/N does consolidation stop winning
and fragmentation take over, and does that boundary sit somewhere that
produces a real strategic choice rather than a foregone conclusion in
either direction.

### 6.4 What gets decided from the results

- **R-MC3b:** the actual `r_eq` per class — chosen so the consolidation-vs-
  fragmentation boundary (§6.3) falls in a range that produces genuine,
  situational strategic tension, not a one-sided result.
- **R-MC9 (reframed):** rather than picking "balance-preserving" or
  "fiction-faithful" from geometry alone, run both candidate size ladders
  through the arena and let the measured boundary decide which (or what
  blend) actually plays well.
- **R-MC12:** cross-role sizing (General Offensive vs. General Systems)
  becomes testable the same way once Contact/Offensive hulls have slot
  tables (R-L0) to fight with.

**R-MC13 (open):** build the arena as a sibling to `montecarlo.rs`
(`combat_arena.rs` or similar) — same harness pattern (seeded runs, printed
per-seat outcomes), but seeded with two placed fleets and zero economy
instead of a generated galaxy. **R-MC14 (open):** the actual distance/
velocity/N sweep ranges — informed by, but not fixed by, this spec.

---

## R-code roundup

- **R-MC1:** where each card/node sits on the mineral ratio continuum
  (4:2:1 peak → 5:4:3 floor) — a per-card tuning call.
- **R-MC2:** how much variance "platinum ratio close to 4:2:1 always"
  actually permits.
- **R-MC3a:** pin actual per-class semi-axis ratios (§2.1's example values
  are illustrative).
- **R-MC3b: resolved as methodology, not a value.** `r_eq` per class is set
  empirically via the Ship Testing Arena (§6), not derived from geometry —
  per this conversation.
- **R-MC4: resolved.** 1:3:9 cost and 0:1:2 cargo are both superseded by the
  geometry in §2.1–§2.4.
- **R-MC5: resolved.** Cargo capacity is volume-proportional
  (`cargo ∝ max(0, V − V_reserved)`), not a separate ladder — Limited's zero
  falls out of the geometry rather than being a special case. §2.3 gives a
  concrete General:Medium ratio (≈6.3:1) under one candidate lane; §2.5
  shows this ratio degrades further, not better, as a fleet fragments.
- **R-MC6:** size the Kinetic/Potential/Latent group-level super premium,
  and decide whether Growth/Politics's doubled penalty needs a cap.
- **R-MC7:** confirm Lanchester's square law (vs. linear, vs. another
  attrition model) as the mechanism converting a per-unit exotic-tier
  effectiveness multiplier into a fleet-mass equivalence. Directly relevant
  to §2.5/§6 too — it's the same law proposed there as the counterweight
  to the consolidation penalty.
- **R-MC8:** once R-MC7 is settled and a combat model exists, specify what
  stat(s) "effectiveness" reads off.
- **R-MC9 (reframed, deferred to §6):** no longer a geometry-only pick
  between balance-preserving and fiction-faithful size ladders — the Ship
  Testing Arena's measured consolidation/fragmentation boundary (§6.3–6.4)
  is the actual tie-breaker.
- **R-MC10:** confirm `dry_mass ∝ hull volume` and pin `V_reference` — this
  resolves `Hyades_loadout.md`'s own open **R-L5** as a side effect (same
  question, asked from the loadout side first).
- **R-MC11:** formalize hull shape (`κ`/`λ`) as a function of loadout slot
  composition rather than a fixed per-`HullType` constant.
- **R-MC12:** resolve cross-role absolute sizing — testable via §6 once
  Contact/Offensive hulls have slot tables (R-L0).
- **R-MC13:** build the Ship Testing Arena as a `montecarlo.rs` sibling —
  same harness pattern, two placed fleets instead of a generated galaxy.
- **R-MC14:** the actual distance/velocity/`N` sweep ranges for §6.3.
- **Blocking prerequisites, not this spec's to resolve, but load-bearing
  for §6:** `Hyades_loadout.md`'s **R-L0** (hull slot tables), **R-L1**
  (shield behavior), **R-L2** (single- vs. repeated-pass engagement).

---

## References

- Card mineral/super/Platinum permutations, the A/B/C×D grid, and the
  native-super derivation — established earlier this conversation.
- Combo-backbone undercosting law — `Hyades_command_cards.md` §4 (project
  file).
- Super non-interchangeability ("native only within a super's own
  counter-graph aspects") — `Hyades_galaxy_and_autopilot.md` §4.6 (project
  file).
- 3 CMY = 2 RGB = 1 Platinum exchange rate and the counter-graph's
  equivalent-cost definition — `Hulls & classes; the qualitative
  counter-graph` (project file), including its own "GSV lacks mission
  flexibility, cannot split" observation, now formalized in §2.5.
- Existing hull cost/cargo/mass placeholders (1:3:9 mineral cost, 0:1:2
  cargo, flat `dry_mass`) and the acceleration formula
  (`a = thrust/(dry_mass+cargo_mass)`) — `Hyades_vehicle_roles.md` §6,
  `Hyades_loadout.md` §2–3.1, §6–7 (R-L0/R-L1/R-L2/R-L5), `sim.rs`
  (`HullType`, `SimConfig`) (project files).
- The deterministic combat model (acceleration, no-initiative engagement,
  per-ship wreck roll) and the counter-graph's "settle by experiment, not
  assumption" framing — `Hyades_simulation_model.md` §4–5 (project file).
- Monte-Carlo balancing philosophy and the still-open "counter-graph
  numbers" dependency — `Hyades_card_contract.md` §7–8 (project file).
- Existing MC harness pattern (seeded runs, per-seat printed outcomes) —
  `montecarlo.rs` (project file).
- Real Culture-class dimensions used as anchors — Plate-class GSV (50 km ×
  20 km × 4 km), Desert-class MSV ("slightly over 3 km" long), Gangster-class
  ROU (200 m × 50 m) — `Hulls & classes; the qualitative counter-graph`
  (project file, Culture-wiki-sourced flavor text).
- Square-cube law (surface-area-to-volume scaling under uniform
  enlargement) — [Wikipedia: Square–cube law](https://en.wikipedia.org/wiki/Square%E2%80%93cube_law).
- Isoperimetric inequality (the sphere minimizes surface area for a given
  volume) — [Wikipedia: Isoperimetric inequality](https://en.wikipedia.org/wiki/Isoperimetric_inequality).
- Specific surface area increasing under subdivision at constant volume/mass
  (the real-world basis for §2.5's `N^(1/3)` fragmentation-cost law,
  including a worked cube-splitting example matching the derivation exactly)
  — [Wikipedia: Surface-area-to-volume ratio](https://en.wikipedia.org/wiki/Surface-area-to-volume_ratio), [Particle Technology Labs: "An Introduction to Surface Area"](https://particletechlabs.com/ptl-press/introduction-to-surface-area/).
- Sphere, prolate spheroid, and cylinder surface-area/volume formulas —
  [Wolfram MathWorld: Sphere](https://mathworld.wolfram.com/Sphere.html),
  [Wolfram MathWorld: Spheroid](https://mathworld.wolfram.com/Spheroid.html),
  [Wolfram MathWorld: Cylinder](https://mathworld.wolfram.com/Cylinder.html).
- Knud Thomsen's triaxial-ellipsoid surface-area approximation (p≈1.6075,
  max relative error 1.061%) — [John D. Cook: "Simple approximation for surface area of an ellipsoid"](https://www.johndcook.com/blog/2021/03/24/surface-area-ellipsoid/).
- Lanchester's laws (square law for aimed-fire attrition; linear law for
  unaimed/area fire) — [Wikipedia: Lanchester's laws](https://en.wikipedia.org/wiki/Lanchester%27s_laws)
  (already in `Hyades_command_cards.md`'s own reference list).
