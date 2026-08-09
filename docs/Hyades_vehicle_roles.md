# Hyades — Vehicle & Planetary Roles (draft Rev 6)

**Status:** draft, proposed by Claude for Jonathan's ratification. Supersedes
Rev 5.

## 0. Changelog from Rev 5

- **R-V9 resolved:** colonization requires **1 pop carried as cargo**, so a
  Colonizer must be **Medium or larger** (Limited has 0 cargo). No longer
  inferred — confirmed, with the mechanism (pop-as-cargo) now stated (§4.2).
- **R-V3 resolved:** **mining is non-exclusive by default; colonizing is
  exclusive by default; cards may change either.** (§4.2, §4.3.)
- **Autopilot is *not* a Resource** — it's components + systems, same as
  everything else. This follows directly from roles being dynamic
  per-entity data: the thing that was modeled as one global policy per seat
  is really per-entity role state (a Component) plus role behavior (Systems).
  §1 and new §9 updated; this is a real correction to the engine's current
  shape, flagged as future refactor work, not yet done in code.
- **Entity is now a plain `u64`** in the engine (done this turn), matching §1's
  "a unique ID, typically a plain integer." The generation counter is gone —
  nothing is despawned, so an ID can't dangle; efficient queries cover the
  rest. §1 updated to describe what's actually there now.
- **New sibling spec:** `Hyades_loadout.md` (draft Rev 1) covers the piece
  this doc kept deferring — engine/mass/steering/weapons/stealth/armor and
  how loadout integrates into components & systems. §6 (Loadout) here now
  just points at it.


---

## 1. The four ECS primitives, precisely

*"An entity has no data and no behavior of its own — usually it only
consists of a unique ID, typically implemented as a plain integer."* —
this conversation. Binding that to what's actually in `sim.rs`:

| Primitive | Definition | This engine |
|---|---|---|
| **Entity** | a bare, unique ID. No data, no behavior. | `struct Entity(pub u64)` — a plain integer ID, exactly the definition above. No generation counter: nothing is despawned (Scrap/Destroy will *mark* an entity via a component, not free its slot), so an ID can never dangle or alias a reused slot, and efficient queries cover what a generation would otherwise guard against. |
| **Component** | pure per-entity data. Nothing else may be attached to an entity except by being a component. | one `ComponentStore<T>` per data type — a dense `Vec<Option<T>>` indexed by entity index. `hull: ComponentStore<Hull>`, `owner: ComponentStore<PlayerId>`, `cargo: ComponentStore<Minerals>`, etc. |
| **System** | behavior. Reads/writes components for the entities it cares about; owns no state of its own. | the `sys_*` functions — `sys_production_tick`, `sys_contact_arrive`, and so on. Each looks up whichever components it needs via `.get()`/`.get_mut()` (always `Option`, since presence isn't guaranteed) and never assumes anything about an entity beyond what those lookups return. |
| **Resource** | global/singleton state, not tied to any one entity. | the event queue, seeded RNG, clock, `SimConfig`, `PopBands`. **Autopilot is *not* one** — see §9; it's the one thing currently mislabeled a Resource that shouldn't be. |

Worth naming even though it isn't a stored primitive: a **query** — reading
across component stores to find or compute something, without storing the
result anywhere. `holdings_centroid` and `survey_candidates` already are
this; **Fleet** (§5) is proposed to be one too.

---

## 2. What "ship" and "planet" actually are

Neither is an ECS primitive. An entity has no inherent "shipness" — strip its
components away and it's indistinguishable from any other bare ID. "Ship" and
"planet" are just names for two component signatures that keep recurring:

- **Informally "planet":** `position`, `factors` (hab/bio/infra),
  `density`, `stockpile`, `population`, `planet_id`, optionally
  `homeworld` (a tag) and `archetype`.
- **Informally "ship":** `owner`, `hull`, `motion`, plus a role-dependent
  subset of `voyage` / `cargo` / `home_center` / `shuttle`.

Nothing in the engine enforces that these two signatures are the only ones,
or that they're mutually exclusive. That's exactly what makes "Role applies
to planets too" (last turn) unremarkable rather than a special case: a
`Role` component is just one more component type, attachable to any entity
regardless of which other components it happens to carry.

---

## 3. Hull types

The size × class × posture space from last turn, restated with the
Component/lookup line drawn explicitly:

| Class | Sizes | Posture |
|---|---|---|
| **Systems** | Limited, Medium, General | Vehicle (civilian baseline; can militarize temporarily — §4.1) |
| **Contact** | Limited, General | both Vehicle *and* Unit exist (below) |
| **Offensive** | Limited, Rapid, General | Unit only — no civilian baseline |

| | Limited | Medium/Rapid | General |
|---|---|---|---|
| **Systems** | LSV | MSV | GSV |
| **Contact** | LCV / LCU | — | GCV / GCU |
| **Offensive** | LOU | ROU | GOU |

**The Component/lookup split:** *which* of these ten values a given ship has
is per-entity data — one more `ComponentStore<HullType>`, exactly like
`hull: ComponentStore<Hull>` already is. What that value *means* — cargo
capacity, accel, cost, build-time — is not per-entity data at all; it's a
static table keyed by the enum value, the same for every MSV that will ever
exist. Nothing here is new relative to the engine's existing style: this is
the same shape as `Basic`/`Super`/`Archetype` in `resources.rs` already.

**LCV confirmed to exist** — not in the original nine-type enumeration, but
used directly this conversation, alongside LCU. Working hypothesis, flagged
rather than asserted: GCV⇄GCU and LCV⇄LCU may just be the Vehicle/Unit name
for the same re-arm/stand-down toggle `command_cards.md` §2 already
established for any hull ("Banks's dROU ⇄ ROU") — i.e. not two independently
built hull designs at each Contact size, but one hull whose name changes
with its posture. If so, does the same toggle exist for Systems sizes (an
armed GSV becoming a "GSU," unnamed so far) or is Contact's Vehicle/Unit
split special because operating in hostile space is a standard enough mode
for scouts specifically to deserve its own name? **R-V8, narrowed to this
question** rather than "is the roster complete" in general.

This resolves the open size-naming placeholder from `Hyades_command_cards.md`
§2/§11 (R-3): General/Medium(Rapid)/Limited replaces "Heavy/Standard/Light."

---

## 4. The role catalog

Every role has a **default behavior** and zero or more **contingent
behaviors** (a trigger condition and what the entity does instead).
Eligibility is a function of which Hull-type value (and sometimes Loadout,
§6) the entity's components show — never of the entity's bare ID, which
carries no such information itself.

### 4.1 Scout — Contact

- **Eligible:** ~~GCV, GCU, LCV, LCU.~~ **Superseded (R-O44,
  `Hyades_standing_layer_and_observation.md` §7): eligibility is permissive with
  varying competence — any hull may take any role, badly or well.** A restricted
  list leaks role from hull, which is exactly what the `BuildOrder::Hull` split
  (R-O29) exists to prevent: if only Contact hulls can scout, seeing a Contact
  hull *is* seeing a scout. An LSV scouts poorly — slow, no dedicated sensor fit
  — but legally. The four hulls above remain the *competent* choices, which is a
  statement about effectiveness, not permission.
- **Default:** fly to nearest unscanned world, close-scan it, repeat
  (`Hyades_autopilot_colonization_growth.md` §2).
- **Contingent — mission exhausted, LCV:** no unknown planets remain →
  return to the nearest friendly colony and **scrap** (confirmed this
  conversation). Correcting Rev 4, which had generalized "idle → Reserve"
  across every role without exception; that was too strong.
- **Contingent — mission exhausted, GCV/GCU/LCU:** not confirmed either way
  — flagged rather than assumed identical to LCV. §4.6 proposes a principle
  (completable vs. standing mission) that would predict Reserve here, since
  a militarized or larger scout plausibly retains standing value an LCV
  doesn't, but that's extrapolation from one data point, not something
  stated.
- **Contingent — hostile space.** Weapons or stealth for operating in
  contested space (point 1, two turns back) — modeled as a Loadout variation
  on the same role, not a different role (§7 explains the reasoning), unless
  §3's Vehicle⇄Unit hypothesis is confirmed, in which case "arming an LCV"
  may just mean "it becomes an LCU" rather than a same-type loadout change.

### 4.2 Colonizer — Systems

- **Eligible: MSV, GSV.** Confirmed (R-V9 resolved): founding requires
  **carrying 1 pop as cargo** to seed the new colony, and Limited has 0 cargo
  capacity (§6), so a Colonizer must be Medium or larger. The pop is expended
  on founding — it becomes the colony's seed population, the same way the hull
  itself becomes the colony's level-1 infrastructure.
- **Default:** target highest-rank ProductionCenter-class world, else
  highest-rank Colony-class (`autopilot_colonization_growth.md` §4).
- **Arrival — uncontested:** founds the colony; the ship **recycles into
  the new colony's level-1 infrastructure** (resolving R-AC7), and the
  carried pop seeds its starting population.
- **Exclusivity (R-V3 resolved): colonizing is exclusive by default.** One
  owner per world; a second colonizer arriving at a claimed world is the
  contested case below. Cards may override this (e.g. shared/condominium
  worlds, forced co-settlement).
- **Contingent — contested** (R-AC8): target already claimed → returns
  toward home, then goes to Reserve (§4.6) like any entity with no further
  task. Because it still carries its pop, that pop returns with it (available
  to re-task at another target) rather than being lost.

### 4.3 Miner — Systems

- **Eligible:** any Systems-class size. A Limited miner has zero cargo (§6)
  and needs a partner to hold what it extracts — "another ship, or
  infrastructure." The engine already does the infrastructure half of this
  correctly: extraction deposits into the **outpost's own `stockpile`
  component**, not the miner's `cargo`, so a Limited miner already works as
  specified with no change needed there.
- **Default:** station at the target outpost; extract on a recurring tick.
- **Exclusivity (R-V3 resolved): mining is non-exclusive by default.**
  Multiple owners may station miners at the same body and draw down its
  shared, finite density concurrently — no ownership claim, first-come or
  otherwise. Cards may override this (e.g. seizing exclusive extraction
  rights, or blockading a rival's outpost). Note this is the deliberate
  asymmetry with colonizing: a world's *surface* is claimed by one, but its
  *ore* is contestable by all until a card says otherwise.

### 4.4 Freighter — Systems

- **Eligible:** MSV, GSV (stated outright).
- **Default:** shuttle cargo between a Miner's outpost and its production
  center, until the outpost is exhausted.
- **Contingent — threatened.** May become **Tribute**: yield the ship/cargo
  to a raider rather than fight or flee into a losing engagement — a link to
  `Privateer` / piracy (`command_cards.md` §8). Confirmed this conversation:
  the trigger requires co-location (a distant pirate can't demand tribute).
  Full mechanics — the "bidding system" for role change generally — aren't
  articulable yet and are deliberately not spec'd here; tracked in
  `hyades_todo.md`.

### 4.5 Relativistic Kill Vehicle (RKV) strike — Systems → sacrificial Offensive

Named for the [established SF/military-futurism term](https://en.wikipedia.org/wiki/Relativistic_kill_vehicle)
for a kinetic weapon whose destructive force is pure velocity, no warhead —
matching last turn's "orbital rail accelerator to .99c, onboard mass driver
for terminal correction only, ship consumed on delivery, maneuver-and-reuse
sacrificed."

- **Eligible:** Systems Vehicle, size TBD (**R-V10**).
- **Requires:** a rail-accelerator structure at the launch system — a new
  infrastructure concept not yet in `Hyades_simulation_model.md` §2's
  structure list (**R-V11**).
- **Default:** none — exclusively contingent, entered under "extremity."
  Exact trigger condition open (**R-V12**).
- Terminal: consumed, no return branch. Damage/accuracy/counters are
  warfare-autopilot / counter-graph territory.

### 4.6 Reserve vs. Scrap — which applies when

Rev 4 generalized "idle → never scrap" across every role from one example.
Confirmed this conversation that's too strong: **LCV scraps** on mission
exhaustion (§4.1); **ROU does not**, even with no enemies nearby (§4.7) —
"for the stated reason," meaning Rev 4's fleet-in-being argument does apply,
just not universally.

**Proposed distinguishing principle**, fitting both data points, flagged as
a hypothesis rather than the confirmed rule: it turns on whether the role's
mission is **completable** or **standing**. Scout-via-LCV is completable —
under current knowledge, a fully-scanned reachable galaxy has categorically
nothing left to explore, so there's no reason to expect the mission to
resume, and reclaiming the mineral value is the rational move. Offensive is
standing — "no enemies nearby right now" isn't "the defense mission is
done," a threat can reappear, and a warship's job is to *be ready*, not to
react to an empty threat list at this instant. Under this reading, whichever
side of that line a given role/hull-type combination falls on decides Scrap
vs. Reserve for it — not a single rule applied to every idle entity alike.

**Reserve, where it applies, composes for free:** it's a role like any
other (§7), so by §5's co-location rule every idle ship of one owner sitting
at one colony automatically forms that colony's Reserve fleet — the
classical naval [**fleet in being**](https://en.wikipedia.org/wiki/Fleet_in_being),
extending influence without ever leaving port.

**R-V16 — resolved: Reserve entities are proactively re-tasked when
circumstances change.** Not left idle until an explicit order — re-tasking
to a fresh opportunity or need is Reserve's own default behavior, the same
way any other role has a default behavior (§7). Confirmed this conversation.

**Still open:** which other role/hull-type combinations land on the Scrap
side vs. the Reserve side of the completable/standing distinction — flagged
per-role rather than assumed (§4.1's GCV/GCU/LCU note; likewise unconfirmed
for an exhausted Miner or a Colonizer bounced by contest).

### 4.7 Offensive — role behavior only, hull types not yet designed

Default unchanged: holds station, engages in-range hostiles, deterministic
accept/decline (`Hyades_galaxy_and_autopilot.md` §6). No combat system
exists in the engine yet.

**Confirmed this conversation:** an idle Offensive unit (example given: ROU)
with no enemies nearby does **not** auto-scrap — it goes to Reserve (§4.6)
and stays there, a standing force regardless of the moment's threat picture.
This is the "standing mission" side of §4.6's completable/standing
distinction — the concrete data point that distinction was built to fit.

---

## 5. Fleet — a query, not a stored thing

*"A ship belongs to a fleet. A fleet consists of one or more ships with the
same role. Changing role changes the fleet."* Precisely: **Fleet is not a
component and not an entity.** There is no `FleetId` anywhere. At any
instant, "my fleet for role R" is the answer to a query — *scan entities,
keep the ones whose `owner` component reads me, whose `Role` component
reads R, and whose position is in the same theater* — computed the same way
`holdings_centroid` already scans `self.planet_entity` and filters by
`owner`.

**Confirmed this conversation: same-role *and* co-located** (same theater —
i.e. the same star system, matching how the engine already treats a system
as a single point). This lines up with the formation/engagement concepts
already on the books — line/screen/wedge, "engage in-range" — which need
spatially-local groupings, not an empire-wide bucket. An empire-wide
same-role count (e.g. whatever `Levée en Masse` scales off,
`command_cards.md` §8) is a separate, still-useful number computed the same
way minus the location filter — not itself "the fleet."

---

## 6. Cargo and the mineral→fleet economy

**Cargo is a component.** Its value is one of: a mineral amount, a
population amount, or **an embarked fleet — a list of entity IDs**. That
last case is exactly why crisp Entity semantics matter: storing `Vec<Entity>`
inside a component is unremarkable *because* an entity is nothing but an ID
— there's no risk of smuggling behavior or hidden state along with it, and
the engine already does precisely this elsewhere (`home_center:
ComponentStore<Entity>`, one `Entity` value per vehicle, today).

**Capacity by size: Limited = 0, Medium = 1, General = 2** (Limited
confirmed this conversation; previously inferred here from "a Limited
Systems Vehicle needs another ship or infrastructure to hold the
minerals" — that inference held up).

**1 CMY mineral = 1 fleet**, broken out by size (revising the earlier "3 CMY
= 3 fleets" anchor to the same ratio in cleaner units):

| Mineral cost | Buys |
|---|---|
| 1 | 1 General Systems Vehicle |
| 1 | a **medium fleet** of Medium Systems Vehicles — starting guess: **3** |
| 1 | a **large fleet** of Limited Systems Vehicles — starting guess: **9** |

A clean 1 : 3 : 9 progression (General : Medium : Limited); per-unit cost 1 /
⅓ / ⅑ mineral. Explicitly provisional pending Monte Carlo — a natural
companion sweep to `experiments.rs` alongside `reinvest_bias`, once both are
implemented.

**Acceleration falls with cargo mass** (implemented this turn): a laden ship
accelerates as `a = base · dry_mass / (dry_mass + μ · cargo)`, so a full
freighter leaves its outpost slower than it returns empty. This is the
`a = thrust / mass` relation; the per-hull `dry_mass` and thrust it needs are
supplied by `Hyades_loadout.md`, and the cargo term will grow to include
pop and embarked-ship mass once those are massed.

---

## 6.5 Loadout — see `Hyades_loadout.md`

Loadout (engine/thrust/mass, steering, beams/pulse/torpedoes/missiles,
stealth, armor, and the *Stars!*-derived tech carried into real-space
combat) is a spec surface of its own, now written up separately in
`Hyades_loadout.md` (draft Rev 1). The one-line ECS binding, so this doc is
self-contained: **a loadout is a Component** (per-ship fit data), its
per-item *stats* are static lookups (§3's Component/lookup split again), and
the ship-level aggregates it produces — total mass, thrust, so effective
acceleration — are **queries** over that component plus the hull-type table,
exactly the way `laden_accel` already derives acceleration from dry mass and
cargo today.

---

## 7. Role-as-Component, Role-behavior-as-System

With §1's vocabulary in hand, the tension you flagged resolves cleanly:

- **Current role = a Component.** Small, mutable, per-entity data — the
  same shape as `owner` or `hull` already are. Attachable to any entity,
  which is what makes "roles apply to planetary production queues too" fall
  out for free: a planet's `PlanetClass` (ProductionCenter / Colony /
  MiningOutpost / Barren) is already this, under a planet-only name.
  Unifying it with ship roles under one `Role` component type is mostly a
  rename plus widening the enum.
- **Role behavior = a System**, not a new, unnamed category. The decision
  procedure for what a role does — default and contingent behavior alike —
  is code that reads the `Role` component and whatever else is present, the
  same way `sys_production_tick` already reads `Hull` and dispatches on it.
  This is what keeps "production-queue-priority isn't Component data" true:
  making Role dynamic never required putting behavior into data: the
  *value* moved into a component, the *logic* stayed exactly where every
  other system's logic already lives.

**R-V15 — resolved, confirmed this conversation: no, plain lookups cover
it.** Every system in `sim.rs` today is already "generic" over entity
composition for free, in the sense that matters: `.get()` on a
`ComponentStore<T>` returns `Option<T>` regardless of what else the entity
carries, so a role-dispatch system can already read `Role` plus whichever
other components happen to be present (ship-shaped or planet-shaped) with
no compile-time type parameter at all — the same pattern used everywhere
already, just one more `ComponentStore`. No `trait Role<Target>`, no
illustrative sketch needed; a plain `enum Role` component and a `sys_role`
function pattern-matching on its value is the shape to build.

---

## 9. Autopilot is not a Resource

You flagged this directly: *"autopilot isn't a resource because these are
just components and systems."* Correct, and it follows straight from §7.

**What the engine does today (to be refactored):** it stores
`autopilots: Vec<Box<dyn Autopilot>>` as a Resource — one boxed policy per
seat — and every system reaches `self.autopilots[p]` to ask "what should
player *p* do here." That was reasonable when a seat's behavior was one
monolithic policy. It stops being reasonable now that behavior is
per-entity role state: the decision *"what does this entity do"* depends on
that entity's `Role` component, not on a per-seat singleton.

**What it should be:**

- The **doctrine knobs** the autopilot currently holds (`growth_rate`,
  `reinvest_bias`, rank weights, …) are per-seat tuning data → a **Component
  on the player entity**, not a Resource. (Players are already entities with
  a `player_info` component; this is one more.)
- The **behavior** — rank a world, choose a production order, pick a survey
  target — is a **System** reading the acting entity's `Role` (and its
  owner's doctrine component), exactly like §7's `sys_role`. There is no
  per-seat policy object at all; there's role state (Component) and role
  behavior (System), the same as everything else.

That leaves the Resource category holding only genuine singletons — the
event queue, RNG, clock, `SimConfig`, `PopBands` — and nothing behavioral.
The `Autopilot` **trait** may still exist as the interface a swappable
policy System implements (baseline vs. a future smarter one), but it is
dispatched per-entity by a system, not stored as per-seat global state.

**Not yet done in code** — flagged as the refactor that lands when Role
becomes a real component (§7). Sequencing note: worth doing *together* with
the Role/`PlanetClass` unification, since both touch the same dispatch sites.

---

## 8. Open ratification points

**Resolved:** R-V0 (roles are dynamic) · Rev 1's R-V1/R-V6 (Scout exhaustion
resolved specifically for LCV, §4.1) · the `command_cards.md` §11 R-3
size-naming placeholder (§3) · **R-V3** (mining non-exclusive, colonizing
exclusive, cards may change either, §4.2/§4.3) · **R-V9** (Colonizer is
Medium+ because it carries 1 pop as cargo, §4.2) · R-V14 (Fleet = same role +
co-located) · R-V15 (no Rust generic needed) · R-V16 (Reserve is proactively
re-tasked).

**R-V13, revised rather than closed:** not "idle units are never scrapped"
(Rev 4, too strong) — confirmed instead that Scrap-vs-Reserve is role/hull-
type-specific (§4.6), with LCV→Scrap and ROU→Reserve as the two confirmed
data points and the completable/standing distinction as the proposed (not
confirmed) rule explaining both.

**Still open:** R-V5 (Freighter 1:1 pairing) · R-V8, narrowed (is
GCV⇄GCU/LCV⇄LCU the same re-arm toggle every hull has, or an independent
build) · R-V10/11/12 (RKV strike's size, structure, trigger) · **new:**
which side of the completable/standing line GCV/GCU/LCU (Scout), an exhausted
Miner, and a contested Colonizer fall on (§4.1, §4.6) · **new (R-V17):** the
autopilot→components/systems refactor (§9) — ratify the doctrine-as-player-
component + behavior-as-system shape before implementing.

**Explicitly out of scope, tracked in `hyades_todo.md` instead:**
nonconsensual role change and the "bidding system" that would govern it —
not yet articulable, so deliberately not spec'd here.

**Still deferred:** the production-queue redesign (expand whenever
affordable, rather than a competing bias dial) — ready to formalize once §7
is ratified, since production choice becomes "what does my current Role's
System say to build," which is a cleaner home for it than the standalone
`BuildOrder` match written before this conversation.
