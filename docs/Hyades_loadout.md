# Hyades — Loadout & Ship Systems (draft Rev 1)

**Status:** draft, proposed by Claude for Jonathan's ratification. New spec.
Covers the piece `Hyades_vehicle_roles.md` kept deferring: what a ship is
*fitted with*, and how that fit drives movement and real-space combat. The
brief this turn: *"Engine thrust, mass, steering, beams/pulse/torpedoes/
missiles, stealth, armor, anything in the Stars! tech tree that can be
carried into the real-space combat with no tick-based initiative… all this
requires new code, meaning the integration into the existing components and
systems must be thought through and written in the spec. I do want
acceleration to change depending on cargo."*

Scope: the **data model** for loadouts and the **integration plan** into the
existing ECS. It does *not* design the combat resolution algorithm itself
(the wreck roll, accept/decline, elimination-by-manner already live in
`Hyades_simulation_model.md` §4) beyond specifying what loadout-derived
inputs that algorithm reads. Numbers here are provisional MC placeholders.

---

## 1. What a loadout is (ECS binding first)

Recall the four-way split from `Hyades_vehicle_roles.md` §1: **hull type**
(what a ship is), **ship** (an instance), **role** (its current mission),
**loadout** (its specific fit). This doc is the loadout leg.

| Concept | ECS binding |
|---|---|
| **A ship's loadout** | a **Component** — per-ship fit data. `loadout: ComponentStore<Loadout>`. |
| **What each fitted item does** (a beam's damage, an engine's thrust) | a **static lookup** keyed by the item enum — the §3-of-vehicle-roles "Component/lookup split": the *choice* is per-ship data, the *stats* are a shared table. |
| **Ship-level aggregates** (total mass, total thrust, effective accel, total armor, sensor range) | **queries** over the loadout component + hull-type table — computed, never stored, so they can't drift out of sync with the fit. Exactly how `laden_accel` already derives acceleration from dry mass + cargo today. |

So loadout adds **one component type and a pile of static tables**, plus
**query functions**. It adds no new Resource and no new Entity kind — same
shape the engine already has.

**Why aggregates are queries, not stored fields:** a fit changes only at a
production center (below), and mass changes every time cargo is loaded/
unloaded. Storing derived totals would mean invalidating them on every such
event; recomputing on demand from `O(slots)` data (a handful of items) is
cheap and can't desync. This mirrors `k()` on a planet (min of three
factors, recomputed) rather than a cached field.

---

## 2. The slot model (from *Stars!*)

*Stars!* fits a hull by filling **typed slots**: each hull has a fixed set
of slots, each slot accepts a category of item, and the hull's identity is
largely "how many of which slots." Adopting that, adapted to Hyades'
size×class taxonomy.

**Slot categories** (the *Stars!* set, trimmed to what real-space Hyades
combat uses — no separate FTL/STL split, since Hyades has one relativistic
drive model, and no fuel, since flip-and-burn is modeled by proper
acceleration not a fuel budget):

| Slot | Accepts | Drives |
|---|---|---|
| **ENG** | engines | thrust (→ acceleration, with mass) |
| **WPN** | beam / pulse / torpedo / missile | offense in the counter-graph |
| **ARM** | armor | structure / damage absorption |
| **SHD** | shields | regenerating damage buffer (see R-L1) |
| **AoS** | armor *or* shield | flex defense |
| **ELEC** | stealth, sensors, targeting, steering aids | detection, evasion, accuracy |
| **MECH** | cargo pods, colonization gear, mining rigs, mass drivers | the Systems-role tooling `Hyades_vehicle_roles.md` §4 describes |
| **GP** | general purpose — anything except ENG | fill to taste |

**Hull type sets the slot layout.** Larger hulls have more slots and more
GP; the size ladder (General > Medium/Rapid > Limited) is largely a
slot-count ladder, which is what makes a General hull both more capable and
more expensive (1 : ⅓ : ⅑ mineral cost from `Hyades_vehicle_roles.md` §6).
Concrete per-hull slot tables are **R-L0**, deferred to a tuning pass — the
model is the deliverable here, not the balance.

**Class biases the layout, doesn't hard-gate it.** Per this conversation
(*"All ships participate in the counter-graph… Contact Vehicles may carry
weapons… Systems Vehicles may mount defensive weapons"*), no class is barred
from WPN/ARM/SHD slots. Systems and Contact hulls simply have *fewer* of
them and more MECH/ELEC; Offensive hulls are the reverse. A Systems Vehicle
mounting a defensive beam to deter piracy is a GP or WPN slot filled, not a
special case — the counter-graph applies to every ship because every ship
can carry counter-graph items.

---

## 3. Item families

Each family is an enum; each variant has a static stat row. Families map to
the counter-graph weapon types `Hyades_command_cards.md` §2 already names
(beam/pulse/torpedo/missile) plus the support systems that make them land or
miss.

### 3.1 Engines (ENG) — thrust & mass

- **Stat row:** `thrust` (force units), `mass` (mass units), tech tier.
- A hull's total thrust = Σ engine thrust; total dry mass = hull base mass +
  Σ item mass. **Effective acceleration is a query:**
  `a = total_thrust / (dry_mass + cargo_mass)`.
- This is the general form of the `laden_accel` already in the engine, which
  currently hard-codes thrust ∝ `base_accel_g` and a single `dry_mass`
  config knob. The loadout version replaces both with fit-derived sums. **The
  cargo term is exactly the "acceleration changes depending on cargo"
  requirement** — already implemented for the mineral case, generalized here
  to any cargo mass (pop, embarked ships).

### 3.2 Weapons (WPN) — the four counter-graph families

Damage in real space with **no tick-based initiative** (the hard constraint):
there is no per-round turn order, no INIT stat deciding who fires first.
Instead each weapon's *reach* and *closing behavior* is a function of range
and the two ships' kinematics, resolved geometrically, not by a turn counter.
The four families differ in the range/kinematics regime where they're strong
— which is precisely what makes them a counter-graph rather than a damage
ladder:

| Family | Strong when | Weak when | Notes |
|---|---|---|---|
| **Beam** | close, all-aspect, instant hit | long range (falloff) | no travel time — range-limited hitscan; the reliable backbone |
| **Pulse** | point-blank, high burst | anything past knife-range | tiny, cheap, murderous up close; good on fast closers |
| **Torpedo** | long range, heavy hit | close (arming distance), vs. point-defense | slow projectile; unguided-ish, big |
| **Missile** | long range, tracking | vs. point-defense / jammers | guided; countered by ELEC, unlike torpedoes |

"No initiative" concretely means: a torpedo's advantage is that it delivers
damage *before the closing ship reaches beam range*, decided by the
distance/closing-speed geometry the engine already computes for flight (§5),
not by winning a die-roll for first strike. **R-L2:** the exact geometric
resolution — is it a single closing-pass exchange, or repeated passes until
one disengages — ties into `Hyades_simulation_model.md` §4's single wreck
roll and needs to be pinned with that section, not here.

### 3.3 Defense (ARM / SHD / AoS)

- **Armor:** flat structure added; ablative (stays gone once lost).
  Damage-absorption stat, mass, tier.
- **Shield:** regenerating buffer absorbed-first; from the *Stars!* model,
  shields recharge between engagements. **R-L1:** does "no tick-based
  initiative" also mean no in-combat rounds for shields to regen across?
  Likely shields are a per-engagement buffer that resets between distinct
  engagements rather than regenerating mid-exchange — flagged, tied to §5/R-L2.
- **AoS** slots let a hull trade armor for shield to bias its defense toward
  the threat it expects (kinetic vs. energy), a fit-time counter-graph choice.

### 3.4 Electronics (ELEC) — stealth, sensors, steering

- **Stealth / cloak:** reduces the range at which the ship is detected;
  enables a Scout to survey hostile space (`Hyades_vehicle_roles.md` §4.1)
  and a raider to close before being seen. A detection query compares one
  ship's cloak against another's sensors.
- **Sensors / targeting:** raise detection range and hit accuracy (esp. vs.
  cloak and vs. tracking-countered missiles).
- **Steering aids:** improve turn/closing agility — relevant precisely
  because combat is geometric, not turn-ordered: out-maneuvering to hold
  beam range or deny torpedo range is how a fight is won without an
  initiative stat.

### 3.5 Mechanical (MECH) — the Systems tooling

Cargo pods (raise cargo capacity → interacts with the mass/accel query),
colonization gear (the "1 pop as cargo" a Colonizer needs,
`Hyades_vehicle_roles.md` §4.2), mining rigs (extraction rate), and the
**mass driver / rail-accelerator payload** an RKV strike uses
(`Hyades_vehicle_roles.md` §4.5). A given *role* may require a given MECH
item — "some roles may require specific loadouts" (two turns back) — e.g.
Miner needs a mining rig, Colonizer needs colonization gear.

---

## 4. Upgrading a loadout — cards unlock, docking installs

Two-step, per `Hyades_vehicle_roles.md` (this conversation's origin):
**"Hulls are upgraded by cards being played, but ships must dock at a
production center to upgrade."**

1. **Card play unlocks a hull-type/item tier** at the empire level — raises
   what *can* be fitted. This is the tech-tree progression; cards are the
   only source of new loadout options (`Hyades_command_cards.md`).
2. **A ship installs a fit only while docked** at a friendly production
   center. So a loadout change is an event with a place and a delay, not a
   free instantaneous respec — an existing fleet must physically return to
   upgrade, which is a real strategic cost and a natural rally-point pressure.

**ECS integration:** unlocking is per-player state (a component on the player
entity, or a resource of unlocked tiers — **R-L3**, leaning player-component
to match §9-of-vehicle-roles' push to get behavior/tuning off Resources).
Installing is a **System** triggered by a dock event that rewrites the ship's
`loadout` component, after a `refit_years` delay, only if the target tier is
unlocked and the center qualifies. Mirrors how `sys_colony_arrive` already
rewrites components on an arrival event.

---

## 5. Integration into movement & combat systems

**Movement (exists today, needs generalizing):**

- `Motion.accel` is currently set from a flat `civilian_accel_g * G` or the
  `laden_accel` derate. Under loadout, the per-leg accel becomes a **query**:
  `total_thrust(loadout) / (dry_mass(hull, loadout) + cargo_mass(cargo))`.
  Same call site (`set_leg`), richer inputs. Nothing about the relativistic
  flip-and-burn math (`math::position_along`) changes — it already takes
  accel as a parameter.
- **This is the one part already partly built:** `laden_accel` this turn is
  the cargo term. Loadout replaces its two hard-coded config knobs
  (`dry_mass`, implicit thrust) with fit-derived sums. Clean, incremental.

**Combat (new — the big addition):**

- A new **System**, `sys_engagement`, triggered when two mutually-hostile
  ships (or fleets — `Hyades_vehicle_roles.md` §5, same-role + co-located)
  come within detection/weapon range — a condition the continuous-position
  seam (`positions_at`) can already evaluate at any instant.
- It reads each side's loadout-derived aggregates (thrust/agility for the
  geometry, weapon family reach for who damages whom at what range, armor/
  shield for absorption), resolves the exchange **geometrically with no
  initiative order**, and applies `Hyades_simulation_model.md` §4's single
  bounded wreck roll and elimination-by-manner. The wreck roll stays the
  *only* stochastic element (`Hyades_galaxy_and_autopilot.md`); everything
  loadout contributes is deterministic.
- **Determinism:** all loadout math is pure functions of component data →
  the existing determinism guarantees and stress tests extend to it
  unchanged. The engagement system must iterate ships/fleets in entity-index
  order like every other system, so combat outcomes stay bit-reproducible.

**No tick-based initiative, concretely:** `sys_engagement` is scheduled by
*spatial proximity events* on the existing discrete-event queue, not by a
combat-round clock. Who "goes first" is decided by range and closing speed
(geometry), and the whole exchange is one scheduled resolution (or a short
sequence of proximity-driven ones), never a per-tick loop. This is the same
event-driven discipline the rest of the engine already uses; combat is not a
special sub-simulation with its own time model.

---

## 6. What this turn already implemented (down payment)

Cargo-dependent acceleration — the one loadout-adjacent behavior explicitly
asked for — is **done and tested** in `sim.rs`:
`a = base · dry_mass / (dry_mass + cargo_mass_per_unit · cargo)`, applied to
laden freighter legs, with `dry_mass`/`cargo_mass_per_unit` as tunable
`SimConfig` knobs and a `cargo_derates_acceleration` unit test. It's the
degenerate case of §3.1's thrust/mass query (one implicit engine, one dry-
mass constant); the full loadout model generalizes it without changing the
call site.

---

## 7. Open ratification points

- **R-L0** — concrete per-hull slot tables (deferred to tuning; the model is
  the deliverable, not the numbers).
- **R-L1** — shield behavior with no in-combat rounds: per-engagement buffer
  that resets between engagements, rather than mid-exchange regen? (ties to
  §5/R-L2.)
- **R-L2** — the geometric engagement resolution: single closing-pass
  exchange vs. repeated passes; how it feeds `simulation_model.md` §4's wreck
  roll. Belongs with that section, cross-referenced here.
- **R-L3** — where unlocked-tier state lives: player-component vs. resource
  (leaning component, per `Hyades_vehicle_roles.md` §9).
- **R-L4** — do ELEC steering aids and hull size affect the flight model
  (agility) only in combat geometry, or also cruise acceleration? (Simplest:
  combat-only; flag if cruise should differ.)
- ~~**R-L5** — is `dry_mass` per hull-type (General heavier than Limited),
  which the size ladder implies, and does that interact with the 1:3:9 cost
  ratio deliberately (bigger = pricier *and* less nimble laden)?~~
  **Resolved (R-O57/R-O58, `Hyades_standing_layer_and_observation.md` §9).**
  Yes to per-hull-type, and the interaction with 1:3:9 is not merely deliberate
  — it is an identity. Cost *is* dry mass, both scaling with surface area, so
  the cost ratio and the mass ratio are the same ratio and `SimConfig::dry_mass`
  is deleted rather than laddered.
  "Bigger = pricier *and* less nimble laden" holds, but only the second half by
  size: **empty hulls of every size accelerate alike** (thrust and dry mass both
  ∝ area), and the entire spread is in the load, since capacity scales with the
  shell's usable interior. So the penalty is a statement about how full a hull
  is, not about how big it is — which is what makes load state, rather than
  size, the thing a laden burn leaks.

**Sequencing:** this spec + the `Role`/autopilot refactor
(`Hyades_vehicle_roles.md` §7/§9) are the two things gating a return to code.
Loadout is additive (new component, tables, queries, one new combat system)
and doesn't require the Role refactor first — but combat reads Fleet, and
Fleet is cleanest once Role is a real component, so doing Role first is the
tidier order.
