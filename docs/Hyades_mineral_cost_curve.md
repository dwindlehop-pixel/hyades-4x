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

> **Works have their own, wider ratio system** (`Hyades_industry.md` §5.1).
> Infrastructure purchases may go all the way to **`1:0:0` — a single colour** —
> which card costs may not, because a card is a choice made once and a
> colour-locked card is *unavailable* to an archetype poor in that colour (design
> law #13). A work is a repeated purchase **with alternatives**, so a sole-colour
> route is a specialisation rather than a lock. That is the mechanism which makes
> the galaxy's mineral distribution bite on *development* and not only on cards.

**Design guidance (R-MC1, open):** peak ratio for signature, single-tree
cards — they should feel expensive off-domain and cheap in it. Floor ratio
for cross-tree combo cards, since the combo-backbone law (`Hyades_command_
cards.md` §4) already requires these to not overpunish any one domain — a
combo card meant to be playable from several builds shouldn't be
domain-locked by its own cost curve. Where a given node sits on this
continuum is a per-card/per-node tuning call, not fixed here.

### 1.2 Super ratio

> **Domain column corrected — this table was carrying the pre-swap colors.**
> `Hyades_galaxy_and_autopilot.md` §4.8 (Rev 8) ratifies "the color swap":
> Politics↔Yellow and Expansion↔Cyan, not the Politics↔Cyan/Expansion↔Yellow
> pairing an earlier draft of this table still had. The swap is a named,
> motivated, already-ratified decision (R-M8: "Politics↔Yellow, economic
> leverage... intended") that was never propagated here. It also happens to
> resolve a standing open problem — see the callout below the table.

Each tree has exactly one native super (the R-CG16/Option-B assignment —
undocumented in the current tree; if a fuller derivation exists in prior
conversation history, restore it), one **domain-sharing foreign** super
(shares the tree's own domain mineral), and one **zero-sharing foreign**
super (shares nothing):

| Tree | Domain | Native (1st) | Domain-sharing (2nd) | Zero-sharing (3rd) |
|---|---|---|---|---|
| Warfare | Magenta | Red | Blue | Green |
| Technology | Magenta | Blue | Red | Green |
| Growth | Cyan | Blue | Green | Red |
| Politics | **Yellow** | Green | **Red** | **Blue** |
| Expansion | **Cyan** | Green | **Blue** | **Red** |
| Production | Yellow | Red | Green | Blue |

This ranking isn't chosen — it's forced by the color algebra
(`resources.rs`): the 2nd-place super is whichever of the two non-native
supers still contains the tree's own domain mineral, and the 3rd is the one
that doesn't. The **native (1st) column needs no change under the swap** —
Politics's and Expansion's native super, Green, is `Yellow + Cyan` by
recipe (§4.2), so it contains *both* colors these two trees trade between,
and stays valid whichever one is currently "their" domain. Only the 2nd/3rd
ranks — which are keyed to the domain color itself — swap places for these
two trees.

> **This closes R-O8.** `Hyades_standing_layer_and_observation.md` design law
> L1 forbids a categorical classification (like the Latent group, §3 below)
> from being co-extensive with a color domain, and flagged exactly one
> violation as open: *"Latent is exactly the Cyan domain."* That was true
> under the stale pre-swap table (Growth = Cyan, Politics = Cyan) and is
> **false** under the corrected one (Growth = Cyan, Politics = **Yellow**) —
> Latent is mixed-domain like the other two groups, so a Red-type homeworld
> (rich Magenta/Yellow, poor Cyan) is no longer locked out of the whole
> Latent group: Growth stays expensive for it, but Politics is now one of
> its *cheap* trees. L1 was never violated by design; it was violated by a
> stale table. R-O8 is resolved by this correction, not by new work.

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

> **R-MC10 resolved, and the answer is no — dry mass goes on *area*, not
> volume** (R-O57/R-O58, `Hyades_standing_layer_and_observation.md` §9). The
> premise "assuming roughly uniform hull density" is what fails: a hull is a
> **shell**, so the material bought is the skin, and this very document already
> puts cost on surface area (§2). Once mass is conserved, cost and dry mass are
> the same number, and a volume-scaled mass against an area-scaled price would
> mean a General hull massing more than was paid for it.
>
> Everything else in this passage stands, including the diagnosis. The flat
> `SimConfig::dry_mass` was exactly the gap described — the engine could not
> tell a GSV's mass from a Scout's — and it is now **deleted** rather than
> replaced by a ladder: `hull_dry_mass = cost_fraction × general_vehicle_cost`.
> `V_reference` needs no pinning, because there is no free scale left to pin;
> the mass unit is the mineral. This closes `Hyades_loadout.md`'s R-L5 as a side
> effect, as predicted, though with the opposite ladder to the one proposed.
>
> The volume ratios above keep their job — they are the **capacity** ladder, not
> the mass ladder. Contents scale with the shell's usable interior, which is
> where 33.9 : 6.3 : 1-style super-linearity belongs and where it does the work
> §2 wants from it.

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

#### `V_reserved` per size **and** per role — the model, with numbers

*Added this conversation, and it supersedes the single-lane `V_reserved =
V_LSV` sketch above.* Setting the reserved volume equal to a whole Limited
hull made "Limited carries nothing" a **definition**; the engine inherited
it as the literal `(r − 1)³` with a shell thickness of exactly one Limited
radius for every class (`hyades_todo.md` T-56). Both are now replaced by a
model with two role-indexed constants and one geometric output.

**Geometry.** Work in normalized radius, `V = r³` (the cube-root-of-volume
convention the engine already uses in `HullType::hull_radius`), with **`τ`
the absolute shell thickness** and the hold the concentric interior:

```
V      = r³                       total displaced volume
hold   = (r − τ)³                 pressurised interior — this is what sits on a Band rung
shell  = r³ − (r − τ)³            the material actually bought
cost   = dry mass = shell / η     η = η(role, size), §2.2
V_reserved(role, V) = a_role + b_role · V
capacity = max(0, hold − V_reserved)
```

Three things about this are worth stating plainly because they change what the
earlier sections claimed:

- **§2's "cost ∝ surface area" is the constant-thickness special case.** The
  general law is *cost ∝ shell volume* — the material bought is area × thickness
  — and it reduces to `cost ≈ 3r²τ/η ∝ area` exactly when `τ` is held fixed.
  This is also where §2.6's ratified `F_mass = F_cost^(3/2)` comes from: at
  fixed `τ`, cost tracks `r²` and hold tracks `r³`.
- **The Band rung is the *hold*, not the usable capacity.** The `3/2` exponent
  is exact on hold volume and only approximate on capacity, so the ladder is
  read on the geometric quantity and `V_reserved` is a role deduction applied
  after it. That is what lets a Limited hull sit honestly at `Band Empty` and
  still carry almost nothing.
- **`V_reserved` has two terms, and they do different jobs.** `a_role` is the
  absolute engine/crew/avionics core every hull of that role needs regardless of
  size — it is what turns a `Band Empty` hold into a token cargo, *as a result
  rather than a definition*. `b_role` is the role's own volume-proportional
  payload — weapons, magazines, armour backing, sensor arrays — and it is what
  keeps an Offensive hull from becoming a freighter simply by being large.

**Ratified constants (this conversation).** Offensive reserves the most,
Systems the least, Contact in between — on both terms, and on shell thickness:

| role | `a_role` (absolute core) | `b_role` (payload share of `V`) | `τ` relative to Systems |
|---|---|---|---|
| **Systems** | 0.079 | **0.00** | **1.0×** |
| **Contact** | 0.132 | 0.15 | 1.8× |
| **Offensive** | 0.194 | **0.45** | **3.0×** |

`b_Systems = 0` is deliberate: a Systems hull's only non-cargo volume is its
fixed core, which is what makes its hold ladder purely geometric and makes it
the row the ladder is *solved on*. The other two rows are then derived, not
independently fitted.

#### The ladder is imposed; shell thickness is what it outputs

Cost is `general_vehicle_cost` divided by the existing fleet-size knobs rather
than a new field, per the anchoring directed this conversation:

```
cost(General) = general_vehicle_cost = 1        (= cost Band II)
cost(Medium)  = general_vehicle_cost / medium_fleet_size    = 0.1
cost(Limited) = general_vehicle_cost / limited_fleet_size   = 0.02
```

With the R-MC15 ladder ratified in §2.6 (`F_cost` = 5, 10, 20, 40 and
`F_mass = F_cost^(3/2)`), both the cost and the hold of every Systems hull are
fixed by its rung, so `τ` is not a free dial at all — it is the **output**:

```
r = (cost·η + hold)^(1/3)        τ = r − hold^(1/3)
```

| hull | rung | cost | hold (kt) | **`τ`** |
|---|---|---|---|---|
| LSV | `Band Empty` | 0.02 | 0.0894 | **0.0270** |
| MSV | `Band I` | 0.10 | **1.000** | **0.0317** |
| GSV | `Band II` | 1.00 | 31.62 | **0.0330** |

**A Medium hull's hold is exactly 1.000 kt**, which is the point of anchoring
`Band I` at a round kiloton: `cargo_unit_size` — the engine's reference hold —
becomes **1.0**, and it already agrees with `units::KILOTONS_AT_BAND_I`, which
has been `1.0` since the type landed. The population anchor absorbs the change
(a `Band I` town is ~3,333 people at ~300 kg each rather than 3,200), and it is
the anchor with the loosest tolerance — the requirement on it is an order of
magnitude wide.

**Shell thickness comes out `Limited < Medium < General`, which is the ordering
the design arc asks for, and it was not imposed.** The mechanism is `η`: a
larger hull is rounder (0.86 → 0.98 → 1.000, §2.2), so it gets more interior per
unit of skin, and that surplus is what pays for a thicker skin at the same rung
spacing.

**And the mechanism runs out, which is why Supers gate the mid-game General.**
Once the GSV is a literal sphere there is no more shape to spend, and `τ`
flattens: a `Band III` General would need `τ = 0.0333` and a `Band IV` General
`τ = 0.0333` — no thicker than the `Band II` hull. But at a *fixed Design
level* thickness must keep rising with size, and at a `Band III` hold the cost
is very nearly linear in `τ`:

| `τ` at a `Band III` hold (2,828 kt) | cost | vs the `Band III` price |
|---|---|---|
| 0.0330 (a `Band II` skin) | 19.8 | 0.99× |
| 0.05 | 30.1 | 1.51× |
| 0.10 | 60.4 | **3.02×** |
| 0.20 | 121.7 | 6.09× |

**So a mid-game `Band III` General Systems Hull must be a Design that requires
Supers, and a late-game `Band IV` one a Design that requires apex** — the
Super/apex tier is what *resets the absolute thickness* back down the ladder.
Without that reset the hull is buildable but costs 3–6× its rung, which is the
"cost skyrockets" outcome. This is the concrete mechanical content of "Design
level resets thickness," and it makes the Supers gate a consequence of geometry
rather than a balance decision.

#### The full hull table

Every hull is read off at its size's cost, its `η(role, size)` from §2.2, and
its role's `τ` multiplier and `V_reserved`:

| hull | cost (min) | `η` | `τ` | `r` | `V` | `t/r` | hold | `V_reserved` | **cargo (kt)** | `E` = cargo/cost |
|---|---|---|---|---|---|---|---|---|---|---|
| LSV | 0.02 | 0.86 | 0.0270 | 0.474 | 0.107 | 0.057 | 0.0894 | 0.079 | **0.0104** | 0.52 |
| MSV | 0.10 | 0.98 | 0.0317 | 1.032 | 1.098 | 0.031 | **1.000** | 0.079 | **0.921** | 9.21 |
| GSV | 1.00 | 1.00 | 0.0330 | 3.195 | 32.62 | 0.010 | 31.62 | 0.079 | **31.54** | 31.5 |
| LCV | 0.02 | 0.75 | 0.0486 | 0.345 | 0.041 | 0.141 | 0.0260 | 0.138 | **0** | 0 |
| GCV | 1.00 | 0.96 | 0.0594 | 2.351 | 13.00 | 0.025 | 12.04 | 2.081 | **9.95** | 9.95 |
| LOU | 0.02 | 0.64 | 0.0810 | 0.269 | 0.019 | 0.301 | 0.0066 | 0.203 | **0** | 0 |
| ROU | 0.10 | 0.73 | 0.0950 | 0.553 | 0.169 | 0.172 | 0.0961 | 0.270 | **0** | 0 |
| GOU | 1.00 | 0.93 | 0.0990 | 1.819 | 6.020 | 0.054 | 5.090 | 2.903 | **2.19** | 2.19 |

Seven things this gets right that the superseded sketch did not, and they are
the reason to adopt it:

1. **A Limited Systems hull carries 1.0% of `Band I`** — ~0.0104 kt, a scout's
   sample locker and not a freight capacity. Its *hold* is honestly at
   `Band Empty`; `a_Systems` eats 88% of it.
2. **LCV, LOU and ROU carry exactly zero**, by `max(0, ·)` reached honestly:
   their reserved volume exceeds their hold. This is the *capability* half of
   roles §4's permissive rule (`CLAUDE.md` §7 item 8) — a Limited hull has no
   cargo hold as a fact, not as a competence penalty.
3. **A General Offensive Unit still carries 2.19 kt** — troops, ordnance,
   prize crews — so "Offensive has little to zero cargo" is size-dependent
   rather than a flat zero, which is what the Culture fiction's GOU depicts.
4. **Contact sits between Systems and Offensive on every axis** — `t/r`,
   `V_reserved`, carry efficiency — because it was derived that way, not fitted.
5. **Design law #3 holds in every role.** `E` rises monotonically with size:
   0.52 → 9.21 → 31.5 (Systems), 0 → 9.95 (Contact), 0 → 2.19 (Offensive).
   Consolidation wins under geometry alone, and now by a margin that *grows*
   with class rather than inverting as the pre-R-O58 ladder did.
6. **Shell thickness orders Offensive > Contact > Systems at every size**
   (`t/r`: 0.301 / 0.141 / 0.057 at Limited; 0.054 / 0.025 / 0.010 at General),
   which is the armour statement expressed as geometry rather than as a combat
   constant — design law #2 stays intact.
7. **R-MC12 resolves in lane (a): cost is a function of size alone; role
   changes what you get for the money.** A GOU and a GSV both cost
   `general_vehicle_cost`, but the GOU's lower `η` and 3× thicker skin buy it
   `r_eq = 1.82` against the GSV's `3.20` — a 1.76× linear gap in the direction
   the fiction describes (50 km GSV vs 2–3 km GOU), reached without a per-role
   cost table. **R-MC9 resolves the same way:** the Systems `r_eq` ladder comes
   out `1 : 2.18 : 6.74`, with `r_eq(GSV)/r_eq(MSV) = 3.10` — above lane 1's 1.8
   and below the fiction's literal 6.6, and *derived* from the ladder rather
   than chosen between two lanes.

#### R-MC16 (new): thrust scales with volume as a *capacity*, not as a stat

Asked directly this conversation: *should thrust be proportional to volume?*
**Yes — but as the ceiling a hull can mount, not as a derived acceleration.**
Writing `a_empty = thrust / dry mass` with `dry mass = cost` and an arbitrary
common constant, under three candidate laws:

| hull | `thrust ∝ V` | `thrust ∝ V^(2/3)` | `thrust ∝ dry mass` |
|---|---|---|---|
| LSV | 5.4 | 11.3 | 1.0 |
| GSV | 32.6 | 10.2 | 1.0 |
| LOU | 1.0 | 3.6 | 1.0 |
| ROU | 1.7 | 3.1 | 1.0 |
| GOU | 6.0 | 3.3 | 1.0 |

**Every purely geometric law makes the ROU no faster than the GOU**, which
contradicts the class's role and its name. No choice of exponent fixes it,
because the ROU's advantage was never geometric — a Rapid Offensive Unit
*spends* its reserved volume and its mineral cost on drive where a General
spends it on hold and armour. So:

- **Thrust capacity ∝ `V`.** A hull's ENG slots scale with its volume
  (`Hyades_loadout.md` §3.1), so volume sets the maximum drive it can mount.
  This is the affirmative answer to the question as asked.
- **Realized thrust is a Design quantity, drawn from `b_role · V` and paid for
  in minerals** — not a per-`HullType` constant. A ship flying at its
  volume-proportional ceiling has bought the drive; one that did not is slower,
  and *that* is the acceleration a distant observer reads.
- **This is what design law #10 requires.** `a = thrust/(dry + cargo)` is one
  scalar over three latents, and concealment is a combo property: arming a
  fleet is loud unless you also buy thrust. If thrust were a fixed function of
  hull type, the inverse problem would collapse — observed `a` would name the
  hull class outright, and the observation model with it.
- **It also gives R-O65 its resolution, and its timing.** `hull_thrust_to_mass`'s
  1.2 / 1.1 / 1.0 Systems ladder is a stand-in for a design decision the engine
  cannot yet express, because no Design write reaches thrust. Flatten it *when*
  that write lands (`hyades_todo.md` T-08, `on_refit`), not before — it is
  MC-tuned combat surface (`CLAUDE.md` §6).

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

### 2.6 The Band ladder — a unified magnitude scale

Every quantity in this design that grows over a game — population,
infrastructure, biosphere, radiation, gravity-as-tolerance
(`Hyades_habitability.md` §2.3–2.4), cargo capacity, and hull-class
production cost — has been described somewhere in the docs with a bare
digit: "pop 4," "infra 1 → 4," "3 CMY," a "0–4 sub-score." Bare digits read
as a literal unit count, and for none of these quantities are they one: a
colony at "pop 2" does not have twice the population of "pop 1," any more
than a General hull's cargo hold is twice a Medium's (§2.3 above puts the
real ratio near 6×). **This section names the relationship the digits were
standing in for, so no doc or field has to reuse a bare integer to mean two
different things again.**

**Band, not level.** A **Band** — `Band Empty`, **Band I**, **Band II**, **Band
III**, **Band IV** (Roman numerals, deliberately not the glyphs a literal
count would use) — is a discrete, *multiplicative* magnitude tier. *(Not to
be confused with `hyades_todo.md`'s unrelated "Band A–E" — that is a
design-readiness classification for todo entries, lettered rather than
numbered precisely so it doesn't collide with this one. Same word, unrelated
concept, different alphabet on purpose.)* Crossing
from one Band to the next is not "one more unit"; it is a jump of several
times the previous Band's magnitude. `Band Empty` is the bottom rung — **a
positive magnitude below the first threshold, not zero** (ratified: there is
no `Band 0`, and `Band Empty > 0`); a quantity that is genuinely absent is off
the ladder rather than at its bottom. `Band I` is the first crossed threshold and that quantity's own reference
scale — a small town of ~3,333 people, a Medium hull's reference cargo hold of
exactly one kiloton, a tenth of a General hull's price. **Every quantity anchors its own `Band I`
independently — what has to be shared across quantities is not the
absolute value at `Band I`, it is the *ratio* between consecutive Bands.**
This directly renames the existing population/infrastructure system
(`Hyades_galaxy_and_autopilot.md` §5.1's "integer level 0–4," Gibrat-spaced,
"each level a fixed multiplicative jump") rather than replacing it — that
system was always a Band ladder; it just spelled `Band` as a bare digit.

**The anchor.** `Band I` for mineral spend is *defined*, not measured:
`general_vehicle_cost = 1.0` (`sim.rs`) **is** one unit of minerals, **is**
the cost of one General-class hull, *(amended below — the R-MC15 candidate
moves the General hull up one rung, to cost `Band II`, and puts `Band I` at
0.25 mineral units; the field keeps its value and its name, only its Band
label changes)*, and **is** an abstract quantity
standing for many kilotons of real mass — §2.1–§2.3's shell-model geometry
is what gives that abstraction its physical content.
`medium_fleet_size`/`limited_fleet_size` (`sim.rs`) are not independent
knobs against this anchor; they are **`Band I` expressed in smaller
hulls** — how many Medium- or Limited-class fleets that same one unit of
mineral spend buys.

**The constraint — law of conservation of mass, via the shell model.** Name
the `Band I → Band II` step factor **`F₁`** and the `Band II → Band III`
step factor **`F₂`**. Per this ratification: **`F₁` and `F₂` must each be a
rational number in `[4, 8]` — **superseded and withdrawn by the ratified
R-MC15 block at the end of this section; the constraint is now on how the step
factors grow, `1 < F₍ₙ₊₁₎/Fₙ < 10`** — and the *same* `F₁`, `F₂` must govern every
quantity listed above** — population, infrastructure, biosphere, radiation,
gravity, cargo capacity, and hull production cost. *(Amended by the R-MC15
block at the end of this section: the second half of that sentence is
provably impossible — cost and capacity cannot share a step factor — and the
shared-ratio rule now binds **within** each of two ladders, mass and mineral
cost, rather than across all quantities.)* This is not a style
preference; it is what "the shell model applies uniformly" means in
practice. A hull one Band bigger has `√F` more radius (cost ∝ area, so
`r ∝ √cost`) and therefore a specific, non-arbitrary multiple more volume,
more population capacity, more infrastructure headroom — the same
geometric relationship every one of these quantities inherits, because they
are all consequences of the same conserved-mass body. `[4, 8]` is the range
conservation actually permits here: narrower and a Band step stops reading
as a real tier jump (indistinguishable from noise against the seed-to-seed
variance §"How to search" already documents); wider and two Bands stop
being *adjacent* in any useful sense — a step wider than `8×` should have
had its own Band in between.

**`Band III → Band IV` is unconstrained**, and does not need to be the same
factor across quantities. *(Also withdrawn by R-MC15: `F₃` is now constrained
exactly like its siblings, by the same growth rule. The paragraph is kept
because its reasoning — that the last step before a ceiling is where tuning
freedom belongs — is why the ratified `F₃ = 253` is the largest step rather
than the smallest.)* This is a deliberate release valve, not an
oversight: it is the last Band before a quantity's absolute ceiling
(population's "many billions," a hull's largest practical class), so it is
the natural place for a **balance-tuning knob** rather than a
physically-derived constant. Requiring `[4, 8]` there too would over-constrain
exactly the place tuning most needs freedom.

**A worked — and still-open — example: does the hull ladder satisfy its own
constraint?** Reading hull class onto the ladder as `Limited = Band I`,
`Medium = Band II`, `General = Band III` turns `medium_fleet_size` and
`limited_fleet_size` into direct claims about `F₁`/`F₂`, since cost is the
*square* of the radius ratio: `F₂ = medium_fleet_size` (cost, General over
Medium) and `F₁ = limited_fleet_size / medium_fleet_size` (cost, Medium
over Limited). Checking every cost ladder this project has used or
proposed against `[4, 8]`:

| Ladder | `medium_fleet_size` | `limited_fleet_size` | `F₁` | `F₂` | Both in `[4,8]`? |
|---|---|---|---|---|---|
| 1:3:9 (design law #6, retired placeholder) | 3.0 | 9.0 | 3.0 | 3.0 | no — uniform, but both 25% under the floor |
| **1:4.45:9 (shipped, MC-ratified)** | **4.45** | **9.0** | **2.02** | **4.45** | **no — `F₂` clears, `F₁` badly fails** |
| 1:3.31:16 (shell-model target, T-19) | 3.31 | 16.0 | 4.83 | 3.31 | no — `F₁` clears, `F₂` narrowly fails |

*(This table is now historical: the `[4, 8]` column it is scored against is
withdrawn. Under the ratified rules the shipped 1:4.45:9 satisfies the growth
rule — its step factors 2.02 then 4.45 are increasing and within a decade —
and fails elsewhere instead. The shipped `BAND_STEP = 4.0` puts `Pop Band IV`
at `3,333 × 4³ = 213,300` people, four orders of magnitude short of the
required 1–10 billion.)*

**No ladder this project has ever shipped or proposed satisfies the
constraint**, and that is a genuine new finding, not a restatement of T-19
— T-19 asks whether the shell-model radius prediction beats the original
placeholder; this asks whether *either* candidate, or the shipped value,
is a self-consistent Band ladder at all. Notably, a **uniform** ladder
sitting exactly at the floor (`F₁ = F₂ = 4`) would need
`medium_fleet_size = 4.0`, `limited_fleet_size = 16.0` — which lands on the
same `limited_fleet_size` the shell-model radius prediction independently
produced. That convergence is worth investigating, not yet a conclusion:
the shell-model prediction and "sit exactly at the floor" are different
reasons to land near the same number, and one data point does not
distinguish them.

> **Contradicted by the engine, measured — R-O71 (new, open).** Cargo capacity
> is named in that list, and the engine does not put it on this ladder at all.
> `HullType::cargo_capacity` derives the hold from the shell model's usable
> interior `(r − 1)³` (R-O58), which at the shipped cost ladder steps
> **Medium → General by 106.35x** (`examples/cargo_units`) against the `[4, 8]`
> this section requires — 13 to 26 times outside it. Under the Band reading of
> `Hyades_vehicle_roles.md` §6's 0 / 1 / 2 the holds would be 0.25 / 1.0 / 4.0
> kt, steps of exactly `F`. Both sides are ratified, so this is recorded rather
> than resolved: either capacity leaves this list, or the capacity ladder is
> re-derived from `F`, which would pin `medium_fleet_size` hard and therefore
> lands on top of R-MC15 and T-19. **Resolved by the R-MC15 ratification
> below, and it took the second option in an amended form:** capacity stays on
> a ladder, but on the *mass* ladder rather than the cost one, tied to it by
> `F_mass = F_cost^(3/2)`. `medium_fleet_size` is indeed pinned hard — to 10. `hyades_todo.md` T-53 carries the options and
> `the_cargo_ladder_is_geometric_not_banded` pins the disagreement so it cannot
> drift silently.
>
> Two smaller things fell out of the same measurement and are already fixed in
> code: `laden_accel`'s terms are all genuinely kilotons (no Band is standing in
> for a mass there — the suspicion that started this was not borne out), and
> `SimConfig::cargo_unit_size` was documented as "what a Medium hull carries"
> when it is the hold at the *reference* radius √3 — at the ratified
> `medium_fleet_size = 4.45` a Medium hull carries 0.959 kt against the field's
> 5.0.

**The engine now depends on this number.** `src/units.rs` implements the
Band↔kiloton bridge as `KT(b) = KT_I · BAND_STEP^(b−1)` — the *only* place
the two units meet — and population growth pays for itself across it, so
`BAND_STEP` sets how much biomass a Band of people actually costs. It ships
at **`4.0`, explicitly as a placeholder pinned to this R-code** *(R-MC15 now
ratifies the mass ladder's `I → II` factor at 31.62, and makes the bridge
piecewise because the factor differs per rung)*. Two things follow. Ratifying `F₁`
settles `BAND_STEP`; and any `F₁ ≠ F₂` needs the bridge to become
piecewise, because a single exponential cannot express two different step
factors. A test (`a_band_step_is_multiplicative_not_additive`) pins the
value inside `[4, 8]` so a future edit cannot quietly leave the range.

**R-MC15 (open, new):** decide `F₁`/`F₂` (in `[4, 8]`) and re-derive
`medium_fleet_size`/`limited_fleet_size` from them via the gradient-step
methodology (`CLAUDE.md` §"How to search"), rather than the two fields
being chosen independently as they are today — **`medium_fleet_size` is a
globally MC-tuned parameter (`CLAUDE.md` §6) and is not to be changed
here** without that re-derivation and explicit re-ratification.
`hyades_todo.md` T-19 is the concrete offline-search task this folds into.

#### R-MC15 — **ratified.** Two ladders, a growth constraint, and the `3/2` tie

*Ratified this conversation. It supersedes the `[4, 8]` window above, which is
withdrawn — see the amendment note at the head of this section.*

> **T-64 note — the two ladders now ride on the type.** `units::Qty<S>` is one
> `f64` of kilotons with a zero-sized `Scale` marker (`Mass`, `Cost`) saying
> which of the ladders below its Band *reading* is taken on. This is the
> engine's expression of "the shared-ratio rule binds **within** a ladder": the
> same amount reads a different rung on each, and the shipped defaults make that
> concrete — `general_vehicle_cost = 1.0` is kilotons under R-O57 and reads
> `Band I` as a mass, `Band II` as a cost. Crossing is `Qty::on_scale`, a no-op
> on the bits and explicit at the call site, because a cost *is* a mass. Pinned
> by `units::tests::the_same_amount_reads_a_different_rung_on_each_ladder`, so
> the two cannot be collapsed back into one by anyone who has not read this
> paragraph.

**There are two `F` ladders, not one, and R-O71 is why.** The block above
required a single `F₁`/`F₂` to govern population, infrastructure, biosphere,
radiation, gravity, cargo capacity *and* hull cost. That is provably
impossible: cost is shell volume and capacity is hold volume, so
`F_cost < F_mass^(2/3)` for any ladder at all, and `F_cost ≥ 4` would already
force `F_mass > 8`. The shared-ratio rule therefore binds **within** a ladder:

| ladder | quantities on it |
|---|---|
| **mass** | population, biosphere, `bio_max`, habitability and infrastructure (they are `min`-ed against a mass in `Factors::k`), cargo hold |
| **mineral cost** | hull production cost, card cost, anything priced in minerals |

**The three ratified rules.**

1. **`1 < F₍ₙ₊₁₎/Fₙ < 10`, on both ladders.** Step factors are strictly
   increasing — each Band is a bigger jump than the last — and grow by less
   than a decade per rung. The old `[4, 8]` bound on the factors themselves is
   *superseded and removed*, along with the "`F₃` is unconstrained" release
   valve: `F₃` is now constrained exactly like its siblings.
2. **`F_mass = F_cost^(3/2)`.** Ratified as an identity, not a coincidence. It
   is the shell model's own exponent — at fixed shell thickness cost tracks `r²`
   and hold tracks `r³` — which is why the two ladders can be distinct and still
   be one geometry. Note the consequence: the mass ladder's growth constraint is
   the binding one, since `F_mass` ratios are cost ratios raised to `3/2`, so a
   cost-step ratio must stay under `10^(2/3) = 4.64`.
3. **There is no `Band 0` on the ladder.** The bottom *rung* is
   **`Band Empty`**, and **`Band Empty > 0`** — a positive magnitude beneath
   `Band I`'s threshold, not an absence. Zero is off the ladder entirely.
   **`Band Zero` exists only as a comparison sentinel**, one past the bottom
   the way `Band V` is one past the top: nothing in a game is ever at it, and
   its job is to give a "none of this quantity" check a rung to name instead of
   a bare `0.0`. `BandTier::Zero` sits at ladder position `−1`, outside
   `BAND_FLOOR`, so adding it moved no position above it.

**The ratified default progression.** A uniform step-ratio of 2 satisfies both
rules and lands the population anchor where it is required:

| step | `F_cost` | `F_mass` |
|---|---|---|
| `Empty → I` | 5 | **1000** *(amended — the floor, see below)* |
| `I → II` | 10 | 31.62 |
| `II → III` | 20 | 89.44 |
| `III → IV` | 40 | 252.98 |

Ratios are 2 on the cost ladder and 2.83 on the mass ladder — inside `(1, 10)`
on both, **across `I → II → III → IV`**.

##### Amendment — `Band Empty` is the ladder's *floor*, and its width is set per quantity (T-63)

*Directed: "the setting of `Band Empty` is way too high. Let's set `Band Empty`
to 1 metric ton… I want to change the **width** of `Band Empty`, not reset the
ladder from there."*

`Band Empty` is not a rung of the ratified ladder — it is where the ladder stops
naming magnitudes. Rules 1 and 2 above are statements about how the ladder
*grows*, and they hold across the playable rungs `I → II → III → IV`; how far
below `Band I` the ladder keeps counting is a different question, and §2.6
already answers it per-quantity ("every quantity anchors its own `Band I`; what
is shared is the ratio"). So:

- **The mass ladder's floor is one metric tonne** — `KT(Empty) = 0.001 kt`,
  `F_mass(Empty→I) = 1000`. `Band I` does not move, and neither does any rung
  above it.
- **The cost ladder's floor is untouched** at `F_cost(Empty→I) = 5`. That step
  is the Limited hull's price, a real rung on a real ladder, and the two floors
  are not tied to each other.
- Consequently `F_mass = F_cost^(3/2)` and `1 < F₍ₙ₊₁₎/Fₙ < 10` are claims about
  `I → II → III → IV`. `1000` is neither `5^1.5` nor within a decade of `31.6`,
  and the tests say so in those words
  (`units::tests::a_band_step_is_multiplicative_not_additive`).

**Why.** Everything below `Band I` is read on this one segment, and at the old
`1/11.18 ≈ 0.089 kt` floor the segment was far too narrow to resolve anything:
a 0.1 kt quantity read as **`Band 0.046`**, a rounding error from nothing. At
one tonne the same quantity reads **`Band 0.667`**. Sub-`Band I` colonies,
holds and ore fields are the whole population of that segment, and they were
being flattened against the floor.

**What it costs, and it is a real cost.** *A Limited hull's hold no longer sits
on `Band Empty`.* Hull holds are geometry — the Limited → Medium step is
`5^1.5 = 11.18` because that is the cost ratio raised to the shell exponent —
and that step used to coincide with the mass ladder's floor width. It no longer
does, so a Limited hull's 0.089 kt hold reads **`Band 0.65`**. The Medium and
General holds still land exactly on `Band I` and `Band II`; only the bottom leg
of the "a Limited sits at `Band Empty` on both ladders" claim below is retracted.

**What it does not cost:** the standard bed is unmoved — 10,021,989 →
10,004,297 colony-years on seed 1 and 9,941,898 → 9,930,182 on seed 7 (−0.2% and
−0.1%), with colony counts identical. Almost nothing in the live economy
operates below `Band I`, which is precisely the complaint the change answers. It
does make genuinely poor worlds *barren* — a trace field now falls under
`density_floor` instead of hovering just above it — which is a partial answer to
T-62's "there are no barren worlds any more". `F₃ = 253` is 2.4 orders of magnitude, comfortably inside the "not four
or more" constraint. **This progression is a default, not a measurement**: it is
subject to Monte-Carlo verification that it does not create a colony-years
bottleneck (`examples/colony_years`), and that verification is Stage 3's job.

**Where the rungs land.** `Band I` is **one kiloton**, and everything else is
read off it: a small town of ~3,333 people at ~300 kg of person, possessions and
pressurised volume each, and a Medium hull's reference hold. The opening General
hull is `Band II` on both ladders:

| rung | mineral cost | hold (kt) | population | what sits there |
|---|---|---|---|---|
| `Band Empty` | 0.02 | 0.001 | 3.3 | the ladder's **floor** (T-63); a Limited hull *costs* this rung |
| `Band I` | 0.10 | **1.00** | 3,333 | one **Medium** hull |
| `Band II` | 1.00 | 31.6 | 105,400 | one **General** hull, opening Design |
| `Band III` | 20.0 | 2,828 | 9.43 M | General, **Supers** Design (mid-game) |
| `Band IV` | 800 | 715,500 | **2.38 B** | General, **apex** Design (late-game card) |

**`KT(I) = 1.0` is chosen, not derived, and it is the right thing to choose.**
Every ratio on both ladders is fixed by the step factors; the only freedom left
is where `Band I` sits in real kilotons, and putting it at a round one makes the
engine's reference hold `cargo_unit_size = 1.0` — which is already the value of
`units::KILOTONS_AT_BAND_I`. The population anchor takes up the slack (3,333
people rather than 3,200), and it is the anchor that can afford it: its
requirement is an order of magnitude wide.

`Pop Band IV = 2.38 billion` sits inside the required 1–10 billion, and it is a
*consequence* of the step-ratio rather than a fitted value. A General hull
**costs `Band II` and holds `Band II`**; a Medium **holds `Band I` and costs
`Band I`**; a Limited ~~sits at `Band Empty` on both~~ **costs `Band Empty` and
holds `Band 0.65`** — the hold leg is retracted by the T-63 amendment above,
which widened the mass ladder's floor without moving the cost ladder's. The step factors differ
(5/10/20/40 against 11/32/89/253) and the rungs still correspond, because each
quantity anchors its own `Band I` — which is what this section said from the
start, now with the ratios it actually implies.

**The Supers gate falls out of this, it is not bolted on.** §2.3 shows shell
thickness coming out `Limited < Medium < General` from `η` alone, and then
*flattening* once the GSV is a literal sphere. So a `Band III` hold at a fixed
Design level's thickness costs ~3× its rung. **A mid-game `Band III` General
Systems Hull must therefore be a Design requiring Supers, and a `Band IV` one a
Design requiring apex** — the Super/apex tier is what resets absolute thickness
and keeps the hull at its rung's price.

**What ratification moves in the engine:**

| field | today | ratified | why |
|---|---|---|---|
| `medium_fleet_size` | 4.45 | **10** | the `Band I → II` cost step |
| `limited_fleet_size` | 9.0 | **50** | `5 × medium_fleet_size`, the `Empty → I` step |
| `cargo_unit_size` | 5.0 | **1.0** | the `Band I` hold *is* the reference hold, so this becomes `KT(I)` — and `units::KILOTONS_AT_BAND_I` is already 1.0 |
| `units::BAND_STEP` | 4.0 | **31.62** | it bridges Bands to **kilotons**, so it is the mass ladder's `I → II` factor, not the cost ladder's |

Three consequences to carry into the code change rather than discover in it:

- **`medium_fleet_size` and `limited_fleet_size` are globally MC-tuned**
  (`CLAUDE.md` §6). Moving 4.45 → 10 and 9.0 → 50 must be measured on
  **colony-years**, not colony count, because the acceptance test for this whole
  ladder is whether making General hulls *more expensive* still speeds
  colonisation up.
- **`BAND_STEP` is no longer a single exponential.** The ratified progression
  has a different factor per rung, so the `KT(b) = KT_I · BAND_STEP^(b−1)`
  bridge in `src/units.rs` becomes **piecewise**, and
  `a_band_step_is_multiplicative_not_additive`'s `[4, 8]` assertion is replaced
  by the growth check `1 < F₍ₙ₊₁₎/Fₙ < 10` plus the `3/2` tie to the cost ladder.
- **The absolute mass of one mineral unit is still unpinned** (R-O72,
  `hyades_todo.md` T-54). This section fixes every *ratio* on both ladders and
  no absolute scale except population's; `general_vehicle_cost = 1.0` remains an
  abstract unit until that lands.

## 3. The group-level super premium — Kinetic/Potential/Latent

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
uniform across the six trees. A tree is **double-stacked** when its own
individual zero-sharing (3rd, §1.2) super happens to equal its *group's*
missing super — the two penalties then land on the same target instead of
two different ones. **Recomputed under the corrected §1.2 table** (this was
wrong under the stale one, and materially so — Expansion and Politics were
previously double-stacked and no longer are):

| Tree | Group | 3rd (§1.2) | Group-missing | Double-stacked? |
|---|---|---|---|---|
| Warfare | Kinetic | Green | Blue | no |
| Expansion | Kinetic | Red | Blue | no |
| Technology | Potential | Green | Green | **yes** |
| Production | Potential | Blue | Green | no |
| Growth | Latent | Red | Red | **yes** |
| Politics | Latent | Blue | Red | no |

**Two trees double-stacked, not four, and no group carries both of its
members.** The stale table had Growth and Politics *both* double-stacked
(the "Latent is exactly Cyan" fact this section shared with R-O8) plus one
each from Kinetic and Expansion — four of six trees carrying a compounded
penalty. Under the corrected domains only Technology and Growth do, one per
group at most, which is a much softer and more even penalty landscape than
the stale table implied. **R-MC6 (open, re-scoped):** size the group-level
premium against this corrected map — the sizing question is the same, but
the stakes (who gets hit twice) have changed, so any prior sizing intuition
built against the old table should be re-checked rather than carried over.

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

**"3 CMY = 2 RGB = 1 Platinum = 3 fleets" was one piece of notation doing
three jobs, and conflating them is exactly the bare-digit confusion §2.6
exists to end.** Unpacked into its actual parts:

### 5.0 Unpacking the compressed rule

**"= 3 fleets" is trivial linear scaling, not a ratio worth naming.**
Spending `N` units of mineral cost buys `N` General-hull fleets (or the
Medium-/Limited-hull equivalent, §2.6) — this falls straight out of
`general_vehicle_cost = 1.0` being a flat cost per fleet, and it says
nothing about supers or apex. "3 minerals → 3 fleets" is `Band I` bought
three separate times — **addition, not a Band step** (§2.6) — and is not to
be confused with "`Band III`," a single quantity several times larger in
one lump. Retiring this clause removes exactly the ambiguity §2.6 flags.

**"3 CMY → 2 RGB → 1 Platinum" is a literal, mass-conserving refining
ratio, and it already lives in `Hyades_galaxy_and_autopilot.md` §4.2, not
here:** `3` units of mineral (basics) mass refine to `2` units of super
mass plus `1` unit of slag; `2` units of super mass refine to `1` unit of
apex mass plus `1` unit of slag (`§4.2`'s "ladder 3 basics → 2 supers → 1
apex with wastage"). Two yield fractions, named rather than left as bare
literals every time this ratio is cited:

- **`Y_super` = mineral-mass → super-mass yield ≈ `2/3`** (3 in, 2 out, 1
  to slag).
- **`Y_apex` = super-mass → apex-mass yield = `1/2`** (2 in, 1 out, 1 to
  slag).

End to end, `Y_super · Y_apex = 1/3`: refining a fixed mineral mass all the
way to apex keeps a third of it and loses two-thirds to slag — recoverable
per R-O59 (`Hyades_standing_layer_and_observation.md` §9.3), not destroyed.
This ratio answers *"how much less mass do I have after refining,"* a
bookkeeping question settled by conservation. It says nothing yet about
whether refining was **worth** it — that is §5.1. **R-M2
(`Hyades_galaxy_and_autopilot.md`) still owns whether `2/3` and `1/2` are
the ratified values; this section only names them so they stop being typed
as bare `3:2:1` wherever they're cited.**

### 5.1 The value-equivalence heuristic — a different question, on a different axis

**Stripped of the "3 fleets" clause and disentangled from the literal
refining yield, what's left is an order-of-magnitude *value* heuristic, and
it lives on a different axis than §2's hull size and §5.0's refining yield
entirely.** Per this conversation: it never meant 3 units of base-mineral
spend buys a fleet of equal *value* to 2 units of super spend or 1 unit of
Platinum spend — that would just be §5.0's refining ratio read backwards,
and refining yield is not the same fact as combat value. It means something
closer to *three orders of magnitude* of base-mineral spend sitting
alongside *two orders of magnitude* of super spend and *one order of
magnitude* of Platinum spend — without committing to literal powers of ten
or any single fixed progression, and without claiming those "orders of
magnitude" line up with the Band ladder's `[4, 8]` steps (§2.6): §2.6's
Bands quantize *size within one material*, this heuristic compares *value
across materials*, and the two need not share a scale factor. The concrete
example given: a negative-mass keel (an exotic, Super-or-higher component)
should improve force projection so much via better acceleration that a
fleet an order of magnitude smaller **in mass** matches a base-mineral
fleet's value.

**Why less mass can be worth more: `Hyades_standing_layer_and_observation.md`
§9.5's specific-strength ladder (R-O61) is the mechanical grounding for
this heuristic, derived independently from the shell model itself.**
Higher mineral tiers carry more capability per unit mass — since dry mass
is cost and thrust scales with area, a super-built hull delivers *higher
acceleration at equal capability* than a basics-built one, purely from
conservation, no separate "supers are better" rule required. §9.5 calls
this "a cleaner reading of the '3 CMY = 2 RGB = 1 Platinum' heuristic than
a conversion ratio" — which is this section's point restated from the mass
side: the payoff for refining up the tier ladder is real despite losing
mass to slag (§5.0), because the mass that survives refining does
disproportionately more per kilogram.

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
- **R-MC3a (candidate on the table, §2.3):** pin actual per-class semi-axis
  ratios (§2.1's example values are illustrative). The `η(role, size)` table
  in §2.2 is ratified and §2.3 now derives an `r_eq` ladder from it —
  Systems `1 : 2.10 : 9.16`, and `4.60 / 3.36 / 2.58` across
  GSV / GCV / GOU at equal cost. What remains is the semi-axis ratios that
  *produce* those `η`, which is arena work (§6), not geometry.
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
- **R-MC9: resolved in §2.3, and it picked neither lane.** The size ladder
  is no longer chosen between "balance-preserving" and "fiction-faithful" —
  it is *derived* from the cargo Bands and the shell model, landing at
  `r_eq(GSV)/r_eq(MSV) = 4.36`, above lane 1's 1.8 and below the fiction's
  literal 6.6. The arena (§6.3–6.4) still measures the
  consolidation/fragmentation boundary, but it is now a check on a derived
  ladder rather than the tie-breaker between two guesses.
- ~~**R-MC10:** confirm `dry_mass ∝ hull volume` and pin `V_reference`~~
  **resolved, negatively** — dry mass scales with **surface area**, because a
  hull is a shell and cost *is* dry mass (R-O57/R-O58). `SimConfig::dry_mass` is
  deleted, not re-laddered, and there is no `V_reference` left to pin. Closes
  `Hyades_loadout.md`'s **R-L5** as predicted, with the opposite answer. See §2's
  amendment block.
- **R-MC11:** formalize hull shape (`κ`/`λ`) as a function of loadout slot
  composition rather than a fixed per-`HullType` constant.
- **R-MC12: resolved in §2.3 — cost is a function of size alone.** Role
  changes what the money buys, through `η(role, size)`, the role's shell
  thickness multiplier, and `V_reserved(role, V)`. At equal cost a GOU comes
  out at `r_eq = 2.58` against a GSV's `4.60`, which is the fiction's
  direction without a per-role cost table. Still testable via §6 once
  Contact/Offensive slot tables exist (R-L0).
- **R-MC13:** build the Ship Testing Arena as a `montecarlo.rs` sibling —
  same harness pattern, two placed fleets instead of a generated galaxy.
- **R-MC14:** the actual distance/velocity/`N` sweep ranges for §6.3.
- **R-MC15 (open, new):** decide the Band-ladder step factors `F₁`
  (`Band I → II`) and `F₂` (`Band II → III`), each in `[4, 8]` (§2.6), and
  re-derive `medium_fleet_size`/`limited_fleet_size` from them via the
  gradient-step methodology rather than choosing the two independently —
  no cost ladder this project has shipped or proposed currently satisfies
  the constraint. `medium_fleet_size` is globally MC-tuned
  (`CLAUDE.md` §6) and is not touched by this spec; `hyades_todo.md` T-19
  is the concrete offline-search task this folds into.
  **RATIFIED (§2.6).** Two ladders, mass and mineral cost, tied by
  `F_mass = F_cost^(3/2)`. The `[4, 8]` window on the step factors is
  superseded and removed; the constraint is `1 < F₍ₙ₊₁₎/Fₙ < 10` on both
  ladders, and `F₃` is no longer exempt. Ratified default progression:
  `F_cost` = 5, 10, 20, 40 ⇒ `medium_fleet_size = 10`,
  `limited_fleet_size = 50`, `cargo_unit_size = 1.0`, and `units::BAND_STEP`
  becomes the piecewise mass ladder starting at 31.62. The progression is a
  default subject to Monte-Carlo verification that it creates no colony-years
  bottleneck; the *rules* are settled.
- **R-MC16 (new, §2.3): thrust scales with volume as a *capacity*, not as a
  per-`HullType` acceleration.** Volume sets the ENG-slot ceiling a hull can
  mount; realized thrust is a Design quantity drawn from `b_role · V` and paid
  for in minerals. Every purely geometric thrust law tested makes the ROU
  slower than the GOU, which is why the quantity cannot be a hull constant.
  Gives R-O65 its resolution and its timing: flatten
  `hull_thrust_to_mass`'s 1.2/1.1/1.0 Systems ladder **when** a Design write
  reaches thrust (`hyades_todo.md` T-08), not before.
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
