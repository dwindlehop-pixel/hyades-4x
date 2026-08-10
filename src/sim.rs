//! The **simulation module** — a relativistic discrete-event engine, structured
//! as an **event-driven Entity Component System (ECS)**.
//!
//! ## Architecture
//!
//! Hyades is not tick-based: `Hyades_card_contract.md` §2 mandates *"a
//! discrete-event schedule whose every causal edge carries a light-travel
//! delay."* So this is an **event-driven** ECS — the event queue is the
//! scheduler, and each event dispatches to a **system** that reads/writes
//! **components**:
//!
//! * **Entities** are generational handles ([`Entity`]). Planets, players, and
//!   vehicles are all entities. **Vehicles persist for the whole game** — they
//!   are never despawned. After arriving they keep working: a contact craft flies
//!   on to the next unscanned world, a freighter shuttles cargo home, a colony
//!   vehicle is *recycled into level-1 infrastructure*, a stationed miner holds
//!   position. (Scrap/destroy transitions reuse the same parked-entity machinery.)
//! * **Components** are plain data in typed stores ([`ComponentStore`]).
//!   **Minerals live on planets and ships, never on players**: a planet has an
//!   in-ground `density` ([`MineralField`], which *depletes* as it is mined) and
//!   an extracted `stockpile` ([`Minerals`], the spendable pool); a ship carries
//!   `cargo`. Building a vehicle and upgrading infrastructure both *spend*
//!   minerals from a center's stockpile.
//! * **Systems** are the `sys_*` event handlers.
//! * **Resources** (ECS singletons) are the event queue, seeded RNG, clock,
//!   config, `PopBands`, and the per-seat autopilot policies.
//!
//! ## Continuous, deterministic positions
//!
//! Every spatial entity has an exact `(x, y, z)` at *any* instant, not just at
//! events: planets are fixed points; a ship in flight is placed by the
//! closed-form relativistic flip-and-burn ([`math::position_along`]). Positions
//! are pure functions of deterministic state, so two runs of the same seed agree
//! bit-for-bit — stressed by the determinism tests here and in `tests/`.
//!
//! ## Economy & growth (this slice)
//!
//! Homeworlds start at infrastructure **2** (`K = 2`). Production is gated by
//! development level: **2 = limited** (survey), **3 = medium/rapid** (colony,
//! mining), **4 = all**. Infrastructure deepens one discrete level at a time,
//! costing minerals equal to the *target* level (1→2 costs 2, …). A colony is
//! founded only by a medium/general (colony) vehicle, never an offensive one;
//! founding recycles that vehicle into the new world's level-1 infrastructure.
//! The deepen-vs-colonize-vs-mine trade is an optimal-growth question for the
//! Monte-Carlo balancer (R-AC5/R-AC11); the baseline policy exposes the knobs.
//!
//! ## Interrogation
//!
//! [`Simulation`] carries an optional, off-by-default [`crate::log::SimLog`]
//! (see that module for the design). Call [`Simulation::set_log_filter`] with
//! the categories you want before (or during) a run, then read
//! [`Simulation::log`] to see exactly what each `sys_*` system did and why.

use std::cmp::{Ordering, Reverse};
use std::collections::{BTreeSet, BinaryHeap};

use crate::autopilot::{
    Autopilot, BaselineAutopilot, BuildOrder, Candidate, Doctrine, PlanetView, ProductionContext, RankContext,
    SurveyView, Tasking,
};
use crate::cards::{self, CardEffect, Order, Target};
use crate::galaxy::{Galaxy, PlanetClass, PlanetId, PlayerId, PopBands};
use crate::log::{FreighterLeg, LogEvent, LogFilter, SimLog};
use crate::math::{self, Vec3, G};
use crate::resources::{Archetype, Basic, MineralField, Minerals};
use crate::rng::Rng;
use crate::snapshot::{PlanetSnapshot, PlayerSnapshot, Snapshot, VehicleKind, VehicleSnapshot};

// =====================================================================
// ECS core — a tiny, dependency-free, deterministic world.
// =====================================================================

/// An entity handle: a bare unique ID and nothing else, per the ECS contract
/// (`docs/Hyades_vehicle_roles.md` §1). A `u64` index into the component
/// stores. No generation counter — nothing is despawned (vehicles persist;
/// scrap/destroy will *mark* an entity via a component, not free its slot), so
/// an ID can never dangle or alias a reused slot, and efficient queries cover
/// the rest.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Entity(pub u64);

/// Dense component storage keyed by entity index; iteration is index-ordered for
/// determinism. `None` ⇒ the entity lacks this component.
struct ComponentStore<T> {
    items: Vec<Option<T>>,
}

impl<T> ComponentStore<T> {
    fn new() -> Self {
        ComponentStore { items: Vec::new() }
    }
    fn slot(&mut self, idx: usize) -> &mut Option<T> {
        if idx >= self.items.len() {
            self.items.resize_with(idx + 1, || None);
        }
        &mut self.items[idx]
    }
    fn insert(&mut self, e: Entity, v: T) {
        *self.slot(e.0 as usize) = Some(v);
    }
    fn get(&self, e: Entity) -> Option<&T> {
        self.items.get(e.0 as usize).and_then(|o| o.as_ref())
    }
    fn get_mut(&mut self, e: Entity) -> Option<&mut T> {
        self.items.get_mut(e.0 as usize).and_then(|o| o.as_mut())
    }
    fn contains(&self, e: Entity) -> bool {
        self.get(e).is_some()
    }
}

/// Marker tag for homeworld planet entities.
#[derive(Clone, Copy, Debug)]
struct Homeworld;

/// Carrying-capacity factors of a planet (level-units). Liebig `K = min`.
#[derive(Clone, Copy, Debug)]
struct Factors {
    hab: f64,
    /// **Standing biological mass, in kilotons** — the same unit as population
    /// and minerals, which is what makes the exchange between them exact.
    ///
    /// Unlike minerals this is a *renewable* stock: it regrows logistically
    /// toward [`Self::bio_max`], and it is the only thing in the engine that
    /// increases without being built. Population growth **consumes** it 1:1 and
    /// population decline returns it, which is what closes L6's old exclusion —
    /// people are made of biomass, so population mass is conserved rather than
    /// conjured from an open reservoir.
    bio: f64,
    /// Pristine biosphere: the ceiling `bio` regrows toward. Cards raise or
    /// lower it; a strike that craters a world's ecology lowers this, not just
    /// the standing stock, which is what makes such damage durable.
    bio_max: f64,
    /// Built infrastructure. Integer-valued; deepened one level at a time.
    infra: f64,
}
impl Factors {
    #[inline]
    fn k(&self) -> f64 {
        self.hab.min(self.bio).min(self.infra)
    }
    #[inline]
    fn k_potential(&self) -> f64 {
        self.hab.min(self.bio)
    }
}

/// A vehicle's flight geometry — everything [`math::position_along`] needs to
/// place it at any instant. A parked vehicle has `origin == dest`, `depart ==
/// arrive`, so it reports a fixed point.
#[derive(Clone, Copy, Debug)]
struct Motion {
    origin: Vec3,
    dest: Vec3,
    depart: f64,
    arrive: f64,
    accel: f64,
}

/// A vehicle's semantic destination & survey state.
#[derive(Clone, Copy, Debug)]
struct Voyage {
    target: Entity,
    heading_bias: Option<Vec3>,
    hops: usize,
}

/// Freighter shuttle state: cycle `center → outpost → center`.
#[derive(Clone, Copy, Debug)]
struct Shuttle {
    /// Fixed — the mining site this freighter's own paired Miner works.
    /// Never re-routed: the *source* side of hauling stays 1:1 with the
    /// Miner it was built alongside.
    outpost: Entity,
    /// Where this leg's cargo is headed. **Recomputed on every load**, not
    /// fixed at spawn — confirmed this conversation: "autopilot must haul
    /// minerals to where they are needed," not back to one hardcoded
    /// partner. See `Simulation::most_needed_center`.
    destination: Entity,
    /// `true` while heading out to the outpost to load; `false` heading to
    /// `destination` with cargo.
    outbound: bool,
}

/// Static per-player data. (A player's `PlayerId` is its index in
/// `player_entity`, so it is not duplicated here.)
#[derive(Clone, Copy, Debug)]
struct PlayerInfo {
    scarcity: [f64; 3],
    home: Entity,
}

/// Per-player fog-of-war knowledge (autopilot-doc §1).
#[derive(Clone, Debug, Default)]
struct Knowledge {
    scanned: BTreeSet<PlanetId>,
    /// Worlds a survey craft has been *dispatched to* (marked at launch, so two
    /// scouts never chase the same target). A **bitmap, not a set**: this is
    /// membership-tested once per planet per `survey_candidates` call and never
    /// iterated, and profiling put that one `BTreeSet::contains` at the top of
    /// the whole engine — 63% of instructions once scouts became plentiful.
    /// O(1) indexed access instead of O(log n) pointer chasing.
    visited: VisitedMask,
    targeted: BTreeSet<PlanetId>,
    exploited: BTreeSet<PlanetId>,
}

/// Dense per-planet flag set, indexed by [`PlanetId`]. Grows on demand so a
/// default-constructed [`Knowledge`] needs no galaxy size up front.
#[derive(Clone, Debug, Default)]
struct VisitedMask {
    bits: Vec<bool>,
}

impl VisitedMask {
    #[inline]
    fn insert(&mut self, pid: PlanetId) {
        let i = pid.0 as usize;
        if i >= self.bits.len() {
            self.bits.resize(i + 1, false);
        }
        self.bits[i] = true;
    }

    #[inline]
    fn contains(&self, pid: PlanetId) -> bool {
        self.bits.get(pid.0 as usize).copied().unwrap_or(false)
    }
}

/// A ship's current **role** (`Hyades_vehicle_roles.md` §1/§7) — a Component,
/// per-entity mutable data. Distinct from [`HullType`] (what a ship *is*,
/// fixed at build) — a role describes what mission it's currently attempting,
/// and can change dynamically (only Scout↔Reserve↔Scrapped exist today; the
/// full role catalog's Tribute/RKV-strike/Offensive are not implemented).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Role {
    Scout,
    Colonizer,
    Miner,
    Freighter,
    /// Standing, re-taskable, never auto-scrapped
    /// (`Hyades_vehicle_roles.md` §4.6 — confirmed for e.g. ROU; applied here
    /// to any non-Scout entity with nothing left to do).
    Reserve,
    /// Terminal: recycled for its mineral value. Confirmed only for an
    /// exhausted Scout (§4.1/§4.6) — not a generic idle fallback.
    Scrapped,
}

impl Role {
    fn kind(self) -> VehicleKind {
        match self {
            Role::Scout => VehicleKind::Scout,
            Role::Colonizer => VehicleKind::Colonizer,
            Role::Miner => VehicleKind::Miner,
            Role::Freighter => VehicleKind::Freighter,
            Role::Reserve => VehicleKind::Reserve,
            Role::Scrapped => VehicleKind::Scrapped,
        }
    }
}

/// The size × class × posture hull taxonomy (`Hyades_vehicle_roles.md` §3) —
/// a Component, fixed at build (changes only via docking to refit, not
/// modeled yet). Orthogonal to [`Role`]: a hull type is what a ship *is*; a
/// role is what it's currently *doing*. Only the types the baseline autopilot
/// actually builds are wired to a role below (§ `role_hull_type`); Contact
/// Units and Offensive types exist in the enum for completeness (the spec's
/// full ten-type roster) but nothing spawns them yet — no militarization or
/// combat exists in the engine.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum HullType {
    LimitedSystems,
    MediumSystems,
    GeneralSystems,
    LimitedContactVehicle,
    LimitedContactUnit,
    GeneralContactVehicle,
    GeneralContactUnit,
    LimitedOffensive,
    RapidOffensive,
    GeneralOffensive,
}

/// A named **design** within a hull type — the Banks-convention class name.
///
/// Hull, class and role are three separate things (R-O29,
/// `Hyades_standing_layer_and_observation.md` §7): the hull is the object's
/// size and family, the class is the specific design of it an empire has
/// unlocked, and the role is what that object is currently being used for.
/// Only the first two are chosen at production; the role is assigned after and
/// is reassignable.
///
/// **R-O42b (open): the flavour names are proposed, not authored.** §7.1
/// suggests Meadow for the LSV and Tor for the LCV — small landforms, scaling
/// the Banks convention down to Limited sizes — and lists alternates. They are
/// carried here so the roster has something concrete to hold; renaming them is
/// a one-line change and the author's call.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Class {
    /// LSV — proposed *Meadow*-class (alts: Fen, Holm, Croft, Hollow).
    Meadow,
    /// LCV — proposed *Tor*-class (alts: Spur, Cairn, Shoal, Gully).
    Tor,
    /// A hull type whose classes are not yet authored. Everything the engine
    /// builds beyond the two seeded designs uses this until Design cards give
    /// it a name — it keeps the roster total rather than pretending the rest of
    /// the taxonomy already exists.
    Unnamed,
}

/// The **roster**: which `(hull, class)` designs a player can build at all
/// (R-O28, §5's "Design" — *the roster*, as distinct from Doctrine, which is
/// policy over it).
///
/// Design is **permanent and strictly earlier-is-better**: entries are added by
/// tree cards and never removed, and a scan that reads a roster reads it
/// forever (§5's asymmetric leak — Design never goes stale, Doctrine dies on
/// retasking). That is why this is a set of unlocks rather than a mutable
/// vector of stats.
///
/// Storing it is what unblocks **σ_vector for Design**, which §3 flagged as
/// having no engine component at all: with a roster in the world, the distance
/// between a pre-card and post-card roster is computable.
#[derive(Clone, Debug, Default)]
pub struct Roster {
    /// Sorted, deduplicated — iteration order must be deterministic.
    unlocked: Vec<(HullType, Class)>,
}

impl Roster {
    /// Add a design. Idempotent; keeps the list sorted so iteration is stable.
    pub fn unlock(&mut self, hull: HullType, class: Class) {
        if let Err(at) = self.unlocked.binary_search(&(hull, class)) {
            self.unlocked.insert(at, (hull, class));
        }
    }

    pub fn has(&self, hull: HullType, class: Class) -> bool {
        self.unlocked.binary_search(&(hull, class)).is_ok()
    }

    /// Can this empire build *any* design of this hull type?
    pub fn has_hull(&self, hull: HullType) -> bool {
        self.unlocked.iter().any(|&(h, _)| h == hull)
    }

    /// The first unlocked class for `hull`, if any — what a build order uses
    /// when doctrine names a hull but not a specific design.
    pub fn class_for(&self, hull: HullType) -> Option<Class> {
        self.unlocked.iter().find(|&&(h, _)| h == hull).map(|&(_, c)| c)
    }

    pub fn designs(&self) -> &[(HullType, Class)] {
        &self.unlocked
    }

    pub fn len(&self) -> usize {
        self.unlocked.len()
    }

    pub fn is_empty(&self) -> bool {
        self.unlocked.is_empty()
    }
}

impl HullType {
    /// Hull radius, **in units of shell thickness** — the shell model's single
    /// geometric primitive (R-O58, `Hyades_standing_layer_and_observation.md`
    /// §9.2), and a *derived* quantity, not a new tunable.
    ///
    /// A hull is a shell: dry mass is the material actually bought, so it scales
    /// with **surface area**, and cost §1 already uses area as the cost basis.
    /// Under R-O57 cost *is* dry mass, so `cost ∝ r²` and the radius ladder
    /// falls straight out of the (MC-tunable) cost ladder:
    ///
    /// ```text
    /// r = sqrt(cost / cost_Limited)
    /// ```
    ///
    /// The Limited hull is the unit, `r = 1`, which is the model's other half:
    /// **a Limited hull is all shell and no hold**, which is what makes
    /// [`HullType::cargo_capacity`] come out at exactly zero for it rather than
    /// having to be special-cased. At the shipped 1 : 3 : 9 cost ladder the
    /// ladder is 1 : √3 : 3.
    fn hull_radius(self, cfg: &SimConfig) -> f64 {
        (self.cost_fraction(cfg) / HullType::LimitedSystems.cost_fraction(cfg)).sqrt()
    }

    /// Cargo capacity **as a mass** (R-O58) — kilotons, the same unit as
    /// minerals, population, biosphere and the hull itself (L6).
    ///
    /// Contents scale with the shell's *usable interior*, `(4/3)π(r−t)³`, against
    /// a dry mass that scales with area. Working in shell-thickness units and
    /// normalising to the Medium hull folds the geometric constants into
    /// [`SimConfig::cargo_unit_size`], leaving
    ///
    /// ```text
    /// capacity = cargo_unit_size · (r − 1)³ / (r_Medium − 1)³
    /// ```
    ///
    /// so the reference load a Medium hull hauls is exactly `cargo_unit_size`,
    /// unchanged from the flat constant this replaces.
    ///
    /// **This supersedes the abstract 0 / 1 / 2 unit count** of
    /// `Hyades_vehicle_roles.md` §6. That ladder was confirmed, but as a *unit
    /// count*, and a unit count is not a mass — read as one it puts contents on
    /// a near-linear ladder while the shell model puts them on a cubic one. What
    /// survives, and is asserted in the tests, is its ordinal content: Limited
    /// carries nothing, and each larger hull carries strictly more. The
    /// magnitudes are now geometry (R-O64).
    /// Normalised against [`REFERENCE_MEDIUM_RADIUS`], a **constant**, so this
    /// is well-defined for every cost ladder and nothing diverges. A hull
    /// approaching the unit radius approaches zero capacity, smoothly, because
    /// it is approaching being all shell.
    fn cargo_capacity(self, cfg: &SimConfig) -> f64 {
        const REF_USABLE: f64 =
            (REFERENCE_MEDIUM_RADIUS - 1.0) * (REFERENCE_MEDIUM_RADIUS - 1.0) * (REFERENCE_MEDIUM_RADIUS - 1.0);
        let usable = (self.hull_radius(cfg) - 1.0).max(0.0).powi(3);
        cfg.cargo_unit_size * usable / REF_USABLE
    }

    /// Mineral cost as a fraction of `SimConfig::general_vehicle_cost`, from
    /// the confirmed 1 : 3 : 9 (General : Medium : Limited) progression —
    /// "1 mineral buys 1 GSV, a medium fleet of MSV (starting guess 3), or a
    /// large fleet of LSV (starting guess 9)" (`Hyades_vehicle_roles.md` §6).
    /// The *ratio* (3, 9) is the MC-tunable placeholder, carried on
    /// `SimConfig` (`medium_fleet_size`/`limited_fleet_size`); this just picks
    /// which one applies.
    fn cost_fraction(self, cfg: &SimConfig) -> f64 {
        use HullType::*;
        match self {
            GeneralSystems | GeneralContactVehicle | GeneralContactUnit | GeneralOffensive => 1.0,
            MediumSystems | RapidOffensive => 1.0 / cfg.medium_fleet_size,
            LimitedSystems | LimitedContactVehicle | LimitedContactUnit | LimitedOffensive => {
                1.0 / cfg.limited_fleet_size
            }
        }
    }
}

/// Which [`HullType`] the baseline autopilot builds for each [`Role`]
/// (`Hyades_vehicle_roles.md` §4's eligibility notes) — a concrete,
/// flagged-placeholder choice among what's *eligible*, not dictated by spec:
/// Scout → LCV (matches the confirmed LCV-scraps-on-exhaustion case, §4.1);
/// Colonizer → MSV (cheapest hull that clears the confirmed 1-pop-cargo
/// floor, §4.2/R-V9); Miner → LSV (spec: "any Systems Vehicle," and the
/// engine already deposits extraction into the outpost's own stockpile, so a
/// Limited miner needs no cargo, §4.3); Freighter → MSV (spec: MSV/GSV,
/// picking the cheaper).
pub fn role_hull_type(role: Role) -> HullType {
    match role {
        Role::Scout => HullType::LimitedContactVehicle,
        Role::Colonizer => HullType::MediumSystems,
        Role::Miner => HullType::LimitedSystems,
        Role::Freighter => HullType::MediumSystems,
        Role::Reserve | Role::Scrapped => HullType::LimitedSystems, // inert; value unused
    }
}

/// The **fixed** reference radius the capacity ladder is normalised against —
/// the Medium hull's radius at the reference cost ladder, `sqrt(9/3) = √3`.
///
/// A *constant*, deliberately, and that is the whole point. Normalising against
/// the live `r_M` made the Medium hull a pivot, and pivoting on a quantity that
/// can approach zero is what produced the apparent explosion in the General :
/// Medium capacity ratio. Against a fixed reference nothing diverges: as the
/// cost ladder narrows, the Medium hull's capacity simply *shrinks toward zero*,
/// which is the physically correct statement — a hull barely larger than the
/// unit radius is nearly all shell and has nearly no hold.
const REFERENCE_MEDIUM_RADIUS: f64 = 1.732_050_807_568_877_2; // √3

/// **Dry mass ≡ mineral cost (R-O57, L6).** Minerals spent become hull, so a
/// hull's price and its empty mass are one number in one unit (kilotons); there
/// is nothing left here to tune independently.
///
/// This resolves the flagged placeholder rather than reconciling it. The former
/// `hull_dry_mass` was a *reconstruction* — a `SimConfig::dry_mass` constant
/// times a size tier of 1 / 2 / 3, a volume-like proxy — and CLAUDE.md §7 asked
/// for it to be checked against git history before anything was built on top.
/// Conservation makes the check moot: no independent value can be correct,
/// because any value other than the cost is mass appearing from or vanishing
/// into the hull.
///
/// What the tier proxy was actually costing: with `dry_mass = 1.0` and
/// `cargo_mass_per_unit = 0.2`, one mineral massed **6.0 units as a hull**
/// (a Medium hull costing 1/3 of a mineral and massing 2.0) and **0.2 units as
/// cargo** — a 30× discrepancy depending only on which side of the airlock the
/// mass was on. That is the contradiction R-O57 exists to remove.
///
/// Combat is untouched by the re-basing. `Combatant::max_accel` is
/// `hull_base_thrust · factor / hull_dry_mass`, and thrust is defined below as
/// thrust-to-mass × dry mass, so the dry mass cancels exactly — empty-hull
/// acceleration depends only on [`hull_thrust_to_mass`], as it did before.
pub fn hull_dry_mass(hull: HullType, cfg: &SimConfig) -> f64 {
    hull.cost_fraction(cfg) * cfg.general_vehicle_cost
}

/// Empty-hull thrust-to-mass, in units of `civilian_accel_g` — Offensive hulls
/// out-accelerate haulers, and the Rapid Offensive Unit is the fastest thing in
/// the game (`Hulls_classes_the_qualitative_counter-graph.md`: "the Culture's
/// fastest ships"). Placeholder magnitudes, monotone by intent (R-ARENA3).
///
/// **R-O65 (new, open): the shell model predicts these should be flat within a
/// family, and they are not.** Under R-O58 thrust scales with surface area and
/// so does dry mass, so empty-hull acceleration is *size-independent* — a
/// Limited and a General Systems hull should accelerate alike when empty, with
/// the whole per-class spread living in what they can carry. The residual
/// 1.2 / 1.1 / 1.0 Systems ladder here predates the shell model and says the
/// opposite. It is deliberately **not** flattened in this change: it is an
/// MC-tuned combat surface, and CLAUDE.md §6 requires explicit ratification
/// before those move. Flattening it is a one-line change once ratified, and it
/// touches nothing in `sim` — civilian motion runs on `civilian_accel_g`, so
/// only `arena`/`combat` read this.
fn hull_thrust_to_mass(hull: HullType) -> f64 {
    use HullType::*;
    match hull {
        GeneralSystems => 1.0,
        MediumSystems => 1.1,
        LimitedSystems => 1.2,
        GeneralContactVehicle => 1.2,
        GeneralContactUnit => 1.3,
        LimitedContactVehicle => 1.4,
        LimitedContactUnit => 1.5,
        GeneralOffensive => 1.6,
        LimitedOffensive => 2.0,
        RapidOffensive => 3.0,
    }
}

/// Thrust force such that `base_thrust / dry_mass · G` reproduces the hull's
/// empty-hull accel — so cargo mass (added in the denominator elsewhere) is the
/// only thing that derates it, matching the loadout acceleration query.
pub fn hull_base_thrust(hull: HullType, cfg: &SimConfig) -> f64 {
    hull_thrust_to_mass(hull) * cfg.civilian_accel_g * hull_dry_mass(hull, cfg)
}

/// Per-hull thrust-factor spread (unit-mean-ish jitter a spawner draws within).
/// The Rapid Offensive Unit is pinned to the top of its range by the arena
/// spawner rather than drawn, so it is always the fastest.
pub fn hull_thrust_multiplier_range(_hull: HullType) -> (f64, f64) {
    (0.85, 1.0)
}

/// Mineral cost of building one ship in `role`, under the "1 CMY mineral = 1
/// fleet" model (`role_hull_type` picks the type, [`HullType::cost_fraction`]
/// picks the fraction of `general_vehicle_cost`).
fn role_cost(role: Role, cfg: &SimConfig) -> f64 {
    role_hull_type(role).cost_fraction(cfg) * cfg.general_vehicle_cost
}

/// The component world: entity bookkeeping plus every typed store.
struct World {
    /// Next entity ID to hand out. Monotonic — IDs are never recycled (nothing
    /// is despawned), so `next` doubles as the live entity count.
    next: u64,

    // planet components
    position: ComponentStore<Vec3>,
    factors: ComponentStore<Factors>,
    density: ComponentStore<MineralField>,
    stockpile: ComponentStore<Minerals>,
    population: ComponentStore<f64>,
    planet_id: ComponentStore<PlanetId>,
    homeworld: ComponentStore<Homeworld>,
    archetype: ComponentStore<Archetype>,

    // shared
    owner: ComponentStore<PlayerId>,

    // vehicle components
    role: ComponentStore<Role>,
    hull_type: ComponentStore<HullType>,
    motion: ComponentStore<Motion>,
    voyage: ComponentStore<Voyage>,
    cargo: ComponentStore<Minerals>,
    /// Population carried as cargo (`Hyades_vehicle_roles.md` §4.2/§6) — only
    /// ever nonzero on a Colonizer, consumed on founding. A separate store
    /// rather than folding into `cargo: Minerals`, since pop isn't a mineral;
    /// the fully generic mineral|pop|embarked-fleet cargo slot the loadout
    /// doc describes is future work (§6 there), this is the minimum that
    /// makes "1 pop as cargo" real.
    pop_cargo: ComponentStore<f64>,
    home_center: ComponentStore<Entity>,
    shuttle: ComponentStore<Shuttle>,

    // player components
    player_info: ComponentStore<PlayerInfo>,
    knowledge: ComponentStore<Knowledge>,
    /// Tunable doctrine knobs (`Hyades_vehicle_roles.md` §9, confirmed this
    /// conversation: "autopilot isn't a resource because these are just
    /// components and systems"). Seeded once from
    /// [`Autopilot::default_doctrine`] at bootstrap; the live, authoritative
    /// value from then on is this component, not anything owned by the
    /// `autopilots` Vec — which is now a stateless per-seat algorithm
    /// selector, not a behavioral-state holder.
    doctrine: ComponentStore<Doctrine>,
    /// Per-player **Design**: the roster of unlocked `(hull, class)` designs
    /// (R-O28). Written only by tree cards; permanent once written.
    roster: ComponentStore<Roster>,
}

impl World {
    fn new() -> Self {
        World {
            next: 0,
            position: ComponentStore::new(),
            factors: ComponentStore::new(),
            density: ComponentStore::new(),
            stockpile: ComponentStore::new(),
            population: ComponentStore::new(),
            planet_id: ComponentStore::new(),
            homeworld: ComponentStore::new(),
            archetype: ComponentStore::new(),
            owner: ComponentStore::new(),
            role: ComponentStore::new(),
            hull_type: ComponentStore::new(),
            motion: ComponentStore::new(),
            voyage: ComponentStore::new(),
            cargo: ComponentStore::new(),
            pop_cargo: ComponentStore::new(),
            home_center: ComponentStore::new(),
            shuttle: ComponentStore::new(),
            player_info: ComponentStore::new(),
            knowledge: ComponentStore::new(),
            doctrine: ComponentStore::new(),
            roster: ComponentStore::new(),
        }
    }

    /// Spawn a fresh entity. Vehicles persist for the game, so IDs are never
    /// recycled — the counter only moves forward, and handles never dangle.
    fn spawn(&mut self) -> Entity {
        let e = Entity(self.next);
        self.next += 1;
        e
    }

    fn entity_at(&self, index: usize) -> Entity {
        Entity(index as u64)
    }

    fn entity_count(&self) -> usize {
        self.next as usize
    }
}

// =====================================================================
// Events — a resource, not part of the component world.
// =====================================================================

#[derive(Clone, Debug)]
enum EventKind {
    /// A contact (survey) craft reaches its target and close-scans it.
    ContactArrive { vehicle: Entity },
    /// A scan result reaches the empire (light-lagged); knowledge updates.
    ScanReport { player: Entity, planet: Entity },
    /// A colony vehicle reaches its target and founds a colony.
    ColonyArrive { vehicle: Entity },
    /// A mining vehicle reaches its outpost and stations to extract.
    MiningArrive { vehicle: Entity },
    /// A freighter completes a shuttle leg (load at outpost / deposit at center).
    FreighterArrive { vehicle: Entity },
    /// A returning vehicle reaches home and parks (e.g. a jilted colony ship).
    ReturnArrive { vehicle: Entity },
    /// A manned outpost extracts ore from its dwindling density.
    MiningTick { outpost: Entity },
    /// A production center completes a build cycle (mine + grow + build).
    ProductionTick { center: Entity },
    /// An exhausted Scout reaches a friendly colony and scraps
    /// (`Hyades_vehicle_roles.md` §4.1/§4.6 — confirmed, LCV only).
    ScrapArrive { vehicle: Entity },
    /// **The round barrier** (`Hyades_netcode.md` §1) — the protocol clock's
    /// only tick. Cards are played here and nowhere else.
    ///
    /// A scheduled event like everything else, because this is a discrete-event
    /// engine: the barrier is not a tick sweep and not a wall-clock timer
    /// (net §1.1 forbids the latter outright — a state transition that depends
    /// on a local clock is the classic lockstep desync).
    RoundBoundary { round: u32 },
}

#[derive(Clone, Debug)]
struct Event {
    time: f64,
    seq: u64,
    kind: EventKind,
}

// Min-heap ordering: earliest time first, ties broken by insertion sequence.
impl PartialEq for Event {
    fn eq(&self, o: &Self) -> bool {
        self.time == o.time && self.seq == o.seq
    }
}
impl Eq for Event {}
impl PartialOrd for Event {
    fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}
impl Ord for Event {
    fn cmp(&self, o: &Self) -> Ordering {
        self.time.partial_cmp(&o.time).unwrap_or(Ordering::Equal).then(self.seq.cmp(&o.seq))
    }
}

// =====================================================================
// Public config / reports.
// =====================================================================

/// Tunable simulation knobs (placeholders pending R-AC/R-M). The MC balancer
/// sweeps these alongside the [`crate::autopilot::Doctrine`].
#[derive(Clone, Copy, Debug)]
pub struct SimConfig {
    pub horizon_years: f64,
    pub cycle_years: f64,
    pub build_years: f64,
    pub civilian_accel_g: f64,
    pub colony_seed_pop: f64,
    pub max_survey_hops: usize,

    /// Minimum development level to build medium vehicles (colony/mining). Per
    /// the production schedule this is **3** (2 = limited, 3 = medium, 4 = all).
    pub medium_min_level: u8,
    /// Minimum development level to build *limited* vehicles — the Scout/LCV
    /// survey craft. The same production schedule that puts medium at 3 puts
    /// limited at **2**; before this field existed the autopilot returned
    /// `Idle` for everything below `medium_min_level`, so the limited tier was
    /// unreachable and a level-2 center could do nothing but hoard minerals.
    pub limited_min_level: u8,
    /// Mineral cost of one General Systems Vehicle — "1 CMY mineral = 1 fleet"
    /// (`Hyades_vehicle_roles.md` §6, confirmed this conversation, revising
    /// the earlier "3 CMY = 3 fleets" anchor to the same ratio in cleaner
    /// units).
    pub general_vehicle_cost: f64,
    /// How many Medium Systems Vehicles one `general_vehicle_cost` buys —
    /// "a medium fleet... starting guess: 3" (§6) — **superseded, see below.**
    ///
    /// **MC-ratified at 4.45** by a verified gradient step (`gradient_probe`
    /// then `gradient_step`): elasticity +32.7 ± 3.6 points per ln, moved α = 0.5 along
    /// the normalised gradient and confirmed on the same CRN seeds. The step as
    /// a whole bought **+10.99 ± 1.86 points** of coverage, 38.26% → 49.25%.
    /// Not a solo optimum — one component of a joint move, and it should be
    /// re-derived jointly if any of the other three changes.
    ///
    /// This also discharges design law #6 for one leg of the ladder: 1 : 3 : 9
    /// was explicitly scaffolding to be replaced rather than a target, and it
    /// has now been replaced by measurement. The ladder is 1 : 4.45 : 9.
    pub medium_fleet_size: f64,
    /// How many Limited Systems Vehicles one `general_vehicle_cost` buys —
    /// "a large fleet... starting guess: 9" (§6). MC-tunable.
    pub limited_fleet_size: f64,
    /// Minerals a homeworld starts with, to seed the first infra deepening.
    /// **Homeworld only** — confirmed this conversation: colonies get no
    /// founding seed. (An earlier turn added `colony_seed_minerals`,
    /// mirroring this field, after diagnosing that colonies were
    /// permanently mineral-starved; corrected — the actual fix is that
    /// mining outposts are supposed to cover the deficiency, with the
    /// autopilot hauling minerals to wherever they're needed, not a founding
    /// windfall. See `sys_freighter_arrive` / `most_needed_center`.)
    pub homeworld_start_minerals: f64,
    /// Enforce the Design roster: refuse to build a hull type the player has
    /// not unlocked (R-O28/R-O42).
    ///
    /// **Defaults to `false`, and that is a stopgap, not a preference.** §7.1
    /// ratifies a starting roster of LSV + LCV only, which is correct for the
    /// card game — but the engine has **no card system**, so there is no unlock
    /// path and enforcement would permanently forbid the Medium hull the
    /// colonizer and freighter are built on. Expansion would halt at turn 0.
    /// The roster is therefore seeded and queryable now (so σ_vector for Design
    /// is measurable) but not yet binding. Flip this on with the card layer, or
    /// with a doctrine that can unlock designs.
    pub enforce_roster: bool,
    /// Fraction of a center's local density mined into its stockpile per cycle.
    pub center_mining_fraction: f64,
    /// Logistic regrowth rate of planetary **biosphere mass** per production
    /// cycle, as a fraction of the remaining deficit below `bio_max`.
    ///
    /// Biosphere is the one renewable stock (L6): population growth consumes it
    /// 1:1 and it grows back on its own. The rate is what turns a razed
    /// ecology into a *durable* wound rather than a momentary one — set it high
    /// and biological damage is cosmetic, set it low and a strike costs the
    /// victim centuries. **Placeholder magnitude** pending MC (R-O63); `0.10`
    /// recovers roughly a third of a deficit over four cycles.
    pub biosphere_regen_rate: f64,
    /// Fraction of an outpost's density extracted per mining tick.
    ///
    /// **MC-ratified at 0.238** by a verified gradient step (`gradient_probe`
    /// then `gradient_step`): elasticity +14.5 ± 5.8 points per ln, moved α = 0.5 along
    /// the normalised gradient and confirmed on the same CRN seeds. The step as
    /// a whole bought **+10.99 ± 1.86 points** of coverage, 38.26% → 49.25%.
    /// Not a solo optimum — one component of a joint move, and it should be
    /// re-derived jointly if any of the other three changes.
    ///
    /// The weakest of the four: 14.5 against 2 SE of 11.6 clears significance
    /// but not comfortably. First candidate to re-check on a wider bed.
    pub outpost_mining_fraction: f64,
    pub mining_tick_years: f64,
    /// Density below which a body is considered mined out.
    pub density_floor: f64,
    /// The **reference hold**: what a Medium hull carries, in kilotons — the
    /// scale factor on [`HullType::cargo_capacity`]'s geometric ladder.
    ///
    /// Every other hull's capacity is this times its usable-volume ratio to the
    /// reference radius (R-O58), so a Limited hull carries nothing and a General
    /// one carries far more than twice as much.
    ///
    /// **This is a floor requirement, not a tuning dial** — measured, not
    /// assumed (`examples/binding_check.rs`, 4 seeds, 4,000 yr):
    ///
    /// | `cargo_unit_size` | 0.05 | 0.2 | 1.0 | 5.0 | 25 | 100 |
    /// |---|---|---|---|---|---|---|
    /// | mean coverage | 2.7% | 20.6% | 36.9% | 38.3% | 38.3% | 38.3% |
    ///
    /// The results at 5, 25 and 100 are **bit-identical**: past roughly 1–5 the
    /// hold stops binding, because `load = cap.min(avail)` and an outpost never
    /// accumulates a full hold between visits. Below that it is a cliff, not a
    /// slope — at 0.2 the economy loses half its coverage and at 0.05 it dies.
    ///
    /// So there is nothing to gain by raising it and a great deal to lose by
    /// lowering it. `gradient_probe` correctly reports its local elasticity as
    /// **exactly zero on every seed**; that is a true statement about this
    /// operating point and not a wiring bug, which is what the wide sweep was
    /// run to establish.
    pub cargo_unit_size: f64,

    /// **Transit discount rate `λ`, per year** — how fast the value of a
    /// delivery decays with time in flight
    /// (`Hyades_politics_trade_and_intelligence.md` §2.3).
    ///
    /// One constant with two jobs, which is the whole reason it is a single
    /// number: on the Exchange it is the travel-time discount *and* the `$`
    /// sink (the seller receives `E·exp(−λt)` and the remainder is burned); in
    /// the engine today it is what makes freighter routing trade need against
    /// distance instead of chasing the neediest center across the galaxy.
    ///
    /// **Ratified at `0.01` (half-life 69 yr) — R-P2's condition is met, and
    /// by a wide margin.** `examples/lambda_routing.rs`, 3 seats, 3 seeds,
    /// 4,000 yr:
    ///
    /// | λ | half-life | mean coverage |
    /// |---|---|---|
    /// | 0 (`most_needed_center`) | ∞ | 14.35% |
    /// | 0.002 | 347 yr | 27.71% |
    /// | 0.005 | 139 yr | 35.20% |
    /// | **0.010** | **69 yr** | **39.04%** |
    /// | 0.020 | 35 yr | 36.88% |
    /// | 0.050 | 14 yr | 36.74% |
    ///
    /// A genuine interior optimum, not an endpoint, and **2.7× the shipped
    /// baseline** — a larger effect than the entire five-parameter doctrine
    /// search produced. The scale is physically sensible: a laden hop of
    /// 10–30 ly at 1 g takes 20–45 yr, so a 69-year half-life discriminates
    /// exactly at the range real hauls happen.
    ///
    /// `λ = 0` still reduces exactly to `most_needed_center`, which remains the
    /// permanent oracle (design law #5) and is still tested as such.
    ///
    /// Three seeds is thin for a ratified constant; the value is confirmed in
    /// *direction and order of magnitude*, and the precise optimum wants the
    /// ten-seed bed (T-44).
    pub trade_decay_lambda: f64,

    /// **Years before the first round barrier fires** (`Hyades_netcode.md` §1).
    ///
    /// The opening is deliberately card-free: seats bootstrap survey and
    /// colonization from their homeworlds before anyone can play. Default
    /// **200 yr** — two centuries — proposed for the initial implementation and
    /// **to be tuned by Monte Carlo and playtesting** (R-P12).
    pub years_to_first_round: f64,
    /// **Years between round barriers.** Default **400 yr** — four centuries.
    /// At the 4,000-year horizon that is ~10 rounds, which is the board-game
    /// round count the 30–45 minute target implies. Also **MC- and
    /// playtest-tunable** (R-P12); `0.0` disables the round layer entirely.
    pub years_per_round: f64,

    /// Fraction of a scrapped vehicle's `general_vehicle_cost`-equivalent
    /// value recovered into the nearest friendly colony's stockpile
    /// (`Hyades_vehicle_roles.md` §4.6 — the mineral-recovery reasoning is
    /// confirmed, the recovery *rate* is a placeholder pending MC).
    pub scrap_recovery_fraction: f64,

    pub seed: u64,
}

impl SimConfig {
    /// **Is the derived hull ladder usable?** `None` if fine, `Some(reason)` if
    /// the configuration produces a degenerate one.
    ///
    /// Since R-O58 the cost ladder and the *capacity* ladder are the same
    /// object: radius is `sqrt(cost / cost_Limited)` and capacity is
    /// `(r − 1)³` normalised to a fixed reference. So `medium_fleet_size` is no
    /// longer only a price — it sets how much bigger a Medium hull is than a
    /// Limited one, and therefore the entire contents ladder.
    ///
    /// **One failure, and it is a naming contradiction rather than a physical
    /// one.** `medium_fleet_size ≥ limited_fleet_size` makes the Medium hull no
    /// larger than the Limited one (`r_M ≤ 1`), so a hull the taxonomy calls
    /// bigger is in fact smaller. The model cannot express that, so it is
    /// refused.
    ///
    /// **There is no longer a "too close" soft bound**, and the reason is worth
    /// keeping. The earlier version refused `r_M < 1.25` because the General :
    /// Medium capacity ratio blew up there — but that divergence was an artifact
    /// of normalising capacity against the *live* Medium radius, i.e. dividing
    /// by a quantity that goes to zero. Against a fixed reference
    /// ([`REFERENCE_MEDIUM_RADIUS`]) nothing diverges: a narrow cost ladder just
    /// means the Medium hull is nearly all shell and hauls nearly nothing, which
    /// is a real economic consequence rather than a modelling failure. Narrow
    /// ladders are now **allowed and meaningful**, so the search may explore
    /// them.
    pub fn hull_ladder_fault(&self) -> Option<&'static str> {
        let r_m = HullType::MediumSystems.hull_radius(self);
        if !r_m.is_finite() {
            return Some("hull radius is not finite — check the fleet-size ratios");
        }
        if r_m <= 1.0 {
            return Some(
                "medium_fleet_size >= limited_fleet_size: the Medium hull is no larger than the \
                 Limited one, so every hull has zero cargo capacity (R-O58)",
            );
        }
        None
    }

    pub fn new(seed: u64) -> Self {
        SimConfig {
            horizon_years: 4000.0,
            cycle_years: 50.0,
            build_years: 10.0,
            civilian_accel_g: 1.0,
            // "requires 1 pop as cargo to start a new colony" — confirmed,
            // not a placeholder (`Hyades_vehicle_roles.md` §4.2/R-V9).
            colony_seed_pop: 1.0,
            // 120 — ratified with the snowball defaults: a 40-hop chain retired
            // scouts while most of the galaxy was still dark.
            max_survey_hops: 120,
            medium_min_level: 3,
            limited_min_level: 2,
            general_vehicle_cost: 1.0,
            medium_fleet_size: 4.45,
            limited_fleet_size: 9.0,
            homeworld_start_minerals: 3.0,
            enforce_roster: false,
            center_mining_fraction: 0.15,
            biosphere_regen_rate: 0.127,
            outpost_mining_fraction: 0.238,
            mining_tick_years: 50.0,
            density_floor: 0.01,
            cargo_unit_size: 5.0,
            trade_decay_lambda: 0.01,
            years_to_first_round: 200.0,
            years_per_round: 400.0,
            scrap_recovery_fraction: 0.5,
            seed,
        }
    }
}

/// Per-empire end-of-run summary.
#[derive(Clone, Debug, Default)]
pub struct PlayerReport {
    pub planets_owned: usize,
    pub colonies: usize,
    pub mining_outposts: usize,
    pub total_population: f64,
    pub scanned: usize,
}

/// The Monte-Carlo-readable outcome of a run.
#[derive(Clone, Debug)]
pub struct SimReport {
    pub time_years: f64,
    pub events_processed: u64,
    pub players: Vec<PlayerReport>,
    pub planets_scanned_total: usize,
}

/// One group from [`Simulation::fleets_at`]: an owner's same-[`Role`] ships
/// currently in the same theater. A query result, not stored state.
#[derive(Clone, Debug)]
pub struct FleetSummary {
    pub owner: u32,
    pub role: Role,
    pub theater: PlanetId,
    pub ships: Vec<Entity>,
}

// =====================================================================
// Simulation = World (components) + resources (scheduler/policies).
// =====================================================================

/// The simulation: an ECS [`World`] plus the discrete-event resources.
pub struct Simulation {
    world: World,
    config: SimConfig,
    /// Reused buffer for `fill_survey_candidates` — see that method. Not state:
    /// cleared on every use, so it never affects results.
    survey_scratch: Vec<SurveyView>,
    bands: PopBands,

    planet_entity: Vec<Entity>,
    player_entity: Vec<Entity>,

    autopilots: Vec<Box<dyn Autopilot>>,
    queue: BinaryHeap<Reverse<Event>>,
    /// Seeded RNG resource. Reserved for stochastic systems (the wreck roll, sim
    /// §4); the colonization/growth slice is deterministic without it.
    #[allow(dead_code)]
    rng: Rng,
    clock: f64,
    seq: u64,
    events_processed: u64,
    /// Outpost indices with an active mining tick (dedup).
    active_mines: BTreeSet<u64>,
    /// Optional diagnostic event log — off (records nothing) until
    /// [`Simulation::set_log_filter`] enables a category. See [`crate::log`].
    log: SimLog,
    /// Protocol round index (`Hyades_netcode.md` §1's third clock). Distinct
    /// from `clock`, which is sim years, and from anything wall-clock.
    current_round: u32,
    /// Count of card plays whose effect is not implemented yet.
    inert_card_plays: u64,
}

impl Simulation {
    /// Build a simulation, ingesting a generated [`Galaxy`] into the ECS world.
    pub fn new(galaxy: Galaxy, config: SimConfig, autopilots: Vec<Box<dyn Autopilot>>) -> Self {
        // Refuse a degenerate hull ladder rather than simulating one. A config
        // where nothing can carry cargo still *runs* — it produces numbers, and
        // they look like an economy's — which is exactly why this must be loud.
        // Design law #14's rule against benchmarking a broken configuration
        // only helps if the broken configuration is recognisable.
        if let Some(why) = config.hull_ladder_fault() {
            panic!("degenerate hull ladder: {why}");
        }
        let n = galaxy.homeworlds.len();
        assert_eq!(autopilots.len(), n, "need one autopilot per seat");

        let mut world = World::new();

        let mut planet_entity = Vec::with_capacity(galaxy.planets.len());
        for pl in &galaxy.planets {
            let e = world.spawn();
            world.position.insert(e, pl.position);
            world.factors.insert(
                e,
                Factors {
                    hab: pl.habitability,
                    bio: pl.biosphere,
                    // A wild world is at its pristine ceiling by definition.
                    bio_max: pl.biosphere,
                    infra: pl.infrastructure,
                },
            );
            world.density.insert(e, pl.minerals);
            world.stockpile.insert(e, Minerals::default());
            world.population.insert(e, pl.population);
            world.planet_id.insert(e, pl.id);
            if pl.is_homeworld {
                world.homeworld.insert(e, Homeworld);
            }
            if let Some(a) = pl.archetype {
                world.archetype.insert(e, a);
            }
            if let Some(o) = pl.owner {
                world.owner.insert(e, o);
            }
            planet_entity.push(e);
        }

        let mut player_entity = Vec::with_capacity(n);
        for p in 0..n {
            let hw_pid = galaxy.homeworlds[p];
            let home = planet_entity[hw_pid.0 as usize];
            let scarcity = scarcity_for(galaxy.planet(hw_pid).archetype);

            let e = world.spawn();
            world.player_info.insert(e, PlayerInfo { scarcity, home });
            let mut k = Knowledge::default();
            k.scanned.insert(hw_pid);
            k.visited.insert(hw_pid);
            world.knowledge.insert(e, k);
            player_entity.push(e);
        }

        let mut sim = Simulation {
            world,
            config,
            survey_scratch: Vec::new(),
            bands: galaxy.bands,
            planet_entity,
            player_entity,
            autopilots,
            queue: BinaryHeap::new(),
            rng: Rng::new(config.seed),
            clock: 0.0,
            seq: 0,
            events_processed: 0,
            active_mines: BTreeSet::new(),
            current_round: 0,
            inert_card_plays: 0,
            log: SimLog::new(),
        };
        sim.bootstrap();
        sim
    }

    /// Convenience: every seat runs the baseline colonization/growth policy.
    pub fn with_baseline(galaxy: Galaxy, config: SimConfig) -> Self {
        let n = galaxy.homeworlds.len();
        let autopilots: Vec<Box<dyn Autopilot>> =
            (0..n).map(|_| Box::new(BaselineAutopilot::default()) as Box<dyn Autopilot>).collect();
        Simulation::new(galaxy, config, autopilots)
    }

    #[inline]
    pub fn clock(&self) -> f64 {
        self.clock
    }
    #[inline]
    pub fn events_processed(&self) -> u64 {
        self.events_processed
    }
    #[inline]
    fn players(&self) -> usize {
        self.player_entity.len()
    }

    // --- diagnostic log (optional; off by default) --------------------------

    /// Choose which categories to collect from this point on. Defaults to
    /// [`LogFilter::none`] (nothing recorded). Safe to call mid-run — e.g. turn
    /// on `Production` only for the window you're interrogating, then turn it
    /// back off. Does not affect simulation outcomes (see
    /// `tests::logging_does_not_affect_outcomes`).
    pub fn set_log_filter(&mut self, filter: LogFilter) {
        self.log.set_filter(filter);
    }

    /// The diagnostic log collected so far. Empty unless
    /// [`Simulation::set_log_filter`] has enabled at least one category.
    pub fn log(&self) -> &SimLog {
        &self.log
    }

    /// Drop collected records without changing the active filter.
    pub fn clear_log(&mut self) {
        self.log.clear();
    }

    // --- doctrine (a per-player Component; `Hyades_vehicle_roles.md` §9) ---

    /// Player `p`'s live, tunable doctrine. Authoritative from bootstrap
    /// onward — mutating this (via [`Self::set_doctrine`]) is how MC sweeps,
    /// and eventually cards, actually change behavior; the `Autopilot`
    /// object passed to [`Simulation::new`] is never consulted again after
    /// its `default_doctrine()` seeds this at construction.
    pub fn doctrine(&self, p: usize) -> &Doctrine {
        self.world.doctrine.get(self.player_entity[p]).expect("every player has a doctrine")
    }

    /// Replace player `p`'s doctrine outright (e.g. an MC sweep varying
    /// `reinvest_bias`, or — later — a card that changes doctrine mid-game).
    pub fn set_doctrine(&mut self, p: usize, doctrine: Doctrine) {
        self.world.doctrine.insert(self.player_entity[p], doctrine);
    }

    // --- setup -------------------------------------------------------------

    fn bootstrap(&mut self) {
        for p in 0..self.players() {
            let pe = self.player_entity[p];
            let home = self.world.player_info.get(pe).unwrap().home;

            // Seed this seat's Doctrine component — the *only* time
            // `default_doctrine()` is read; the component is authoritative
            // from here on (`Hyades_vehicle_roles.md` §9).
            self.world.doctrine.insert(pe, self.autopilots[p].default_doctrine());

            // Seed this seat's Design roster (R-O42, §7.1). The ratified
            // starting state is **LSV and LCV only, one class each** — so at
            // turn 0 a scout, a settler and a hauler are literally the same
            // object, and the long-range observable carries almost no
            // information because every empire's fleet looks identical.
            // Inscrutability early is total *by construction* rather than by
            // card design, and legibility grows as rosters diverge.
            //
            // See `roster_permits` for why this does not yet *restrict* what
            // production may build: with no card system there is no unlock
            // path, so enforcing the roster would halt colonization outright.
            let mut roster = Roster::default();
            roster.unlock(HullType::LimitedSystems, Class::Meadow);
            roster.unlock(HullType::LimitedContactVehicle, Class::Tor);
            self.world.roster.insert(pe, roster);

            // Seed the homeworld's stockpile so it can begin deepening infra.
            let seed = self.config.homeworld_start_minerals / 3.0;
            let s = self.world.stockpile.get_mut(home).unwrap();
            s.cyan += seed;
            s.magenta += seed;
            s.yellow += seed;

            self.schedule(self.config.cycle_years, EventKind::ProductionTick { center: home });

            // Opening survey fan-out: contact craft, one per cube-face heading,
            // built free as starting units (§2).
            let home_pos = *self.world.position.get(home).unwrap();
            let vehicles = self.world.doctrine.get(pe).unwrap().survey_vehicles;
            for i in 0..vehicles {
                let heading = Vec3::CUBE_FACES[i % 6];
                self.launch_survey(p, home_pos, heading, 0);
            }
        }

        // The round barrier's first tick. Scheduled once here; each boundary
        // schedules its own successor, so the protocol clock is a chain of
        // events rather than anything the run loop knows about.
        if self.config.years_per_round > 0.0 {
            self.schedule(self.config.years_to_first_round, EventKind::RoundBoundary { round: 0 });
        }
    }

    // --- the round layer ---------------------------------------------------

    /// **The round barrier** (`Hyades_netcode.md` §5) — collect this round's
    /// orders, coerce them, apply them in **seat-index order**, schedule the
    /// next boundary.
    ///
    /// Seat order is not a detail: net §5 P2 requires every client to apply
    /// orders in the same sequence, and seat index is the only ordering every
    /// client agrees on before the orders exist.
    fn sys_round_boundary(&mut self, round: u32) {
        self.current_round = round;

        // Collect. In a networked match these arrive over the wire; here the
        // autopilots supply them, which is exactly the dropout path net §5.3
        // specifies (a disconnected seat is handed to `BaselineAutopilot` and
        // generates zero traffic).
        let mut orders: Vec<Order> = Vec::with_capacity(self.player_entity.len());
        for p in 0..self.player_entity.len() {
            let pe = self.player_entity[p];
            let doctrine = *self.world.doctrine.get(pe).unwrap();
            let seat = PlayerId(p as u32);
            let proposed = self.autopilots[p].choose_card(&doctrine, seat, round).unwrap_or(Order::pass(seat));
            orders.push(proposed);
        }

        self.apply_orders(round, &orders);

        let next = round.saturating_add(1);
        if self.clock + self.config.years_per_round <= self.config.horizon_years {
            self.schedule(self.config.years_per_round, EventKind::RoundBoundary { round: next });
        }
    }

    /// **The sole inbound channel** (design law #15, net §11).
    ///
    /// Everything the outside world can do to the simulation goes through here.
    /// It is *total*: every input maps to a legal state transition, and an
    /// illegal order coerces to `pass` rather than being rejected (net §5.1) —
    /// rejection is how a lockstep system desyncs, because one client's
    /// rejection is another's acceptance.
    ///
    /// Public because the presentation layer must be able to reach it. Nothing
    /// else may cross the seam inbound.
    pub fn apply_orders(&mut self, round: u32, orders: &[Order]) {
        let mut sorted: Vec<Order> = orders.to_vec();
        sorted.sort_by_key(|o| o.seat.0);
        for o in sorted {
            let p = o.seat.0 as usize;
            if p >= self.player_entity.len() {
                continue;
            }
            let cost = o.card.and_then(cards::card).map(|c| c.cost).unwrap_or(0.0);
            let affordable = cost <= 0.0 || self.empire_can_afford(p, cost);
            let Some(id) = o.coerce(affordable).card else { continue };
            let Some(c) = cards::card(id) else { continue };
            if c.cost > 0.0 {
                self.empire_spend(p, c.cost);
            }
            self.apply_card_effect(p, c, o.target, round);
        }
    }

    /// Total basic minerals across an empire's holdings. Cards are paid from
    /// the empire, not from one center — design law #7 puts cards at
    /// empire/macro scale, so a per-center purse would be the wrong grain.
    fn empire_can_afford(&self, p: usize, cost: f64) -> bool {
        let me = PlayerId(p as u32);
        let mut total = 0.0;
        for &e in &self.planet_entity {
            if self.world.owner.get(e).copied() == Some(me) {
                if let Some(s) = self.world.stockpile.get(e) {
                    total += s.basic_total();
                    if total >= cost {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Draw `cost` from the empire's holdings, richest planet first so the draw
    /// is deterministic and does not strand a center that was about to build.
    fn empire_spend(&mut self, p: usize, cost: f64) {
        let me = PlayerId(p as u32);
        let mut holdings: Vec<(Entity, f64)> = self
            .planet_entity
            .iter()
            .filter(|&&e| self.world.owner.get(e).copied() == Some(me))
            .filter_map(|&e| self.world.stockpile.get(e).map(|s| (e, s.basic_total())))
            .filter(|&(_, t)| t > 0.0)
            .collect();
        // Richest first; entity id breaks ties so the order is total.
        holdings.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal).then(a.0 .0.cmp(&b.0 .0)));
        let mut remaining = cost;
        for (e, avail) in holdings {
            if remaining <= 0.0 {
                break;
            }
            let take = remaining.min(avail);
            take_basics(self.world.stockpile.get_mut(e).unwrap(), take);
            remaining -= take;
        }
    }

    fn apply_card_effect(&mut self, p: usize, c: &cards::Card, target: Target, round: u32) {
        match c.effect {
            CardEffect::WriteDoctrine(w) => {
                let pe = self.player_entity[p];
                cards::apply_doctrine_write(self.world.doctrine.get_mut(pe).unwrap(), w);
            }
            CardEffect::UnlockDesign(hull, class) => {
                let pe = self.player_entity[p];
                self.world.roster.get_mut(pe).unwrap().unlock(hull, class);
            }
            CardEffect::DiscloseScans => {
                // politics §5.2/§5.3. The *subject* is whose scan record is
                // published — which need not be the player playing the card.
                // Publishing someone else's holdings is the attack, and it is
                // not opt-in: no consent is sought anywhere on this path.
                let subject = match target {
                    Target::Player(s) => s.0 as usize,
                    Target::None => p,
                };
                if subject >= self.player_entity.len() {
                    return;
                }
                let published: Vec<PlanetId> =
                    self.world.knowledge.get(self.player_entity[subject]).unwrap().scanned.iter().copied().collect();
                for q in 0..self.player_entity.len() {
                    if q == subject {
                        continue;
                    }
                    let k = self.world.knowledge.get_mut(self.player_entity[q]).unwrap();
                    for &pid in &published {
                        k.scanned.insert(pid);
                    }
                }
            }
            CardEffect::NotYetImplemented => {
                self.inert_card_plays += 1;
            }
        }
        self.log.push(self.clock, LogEvent::CardPlayed { player: p as u32, card: c.id.0, round });
    }

    /// The protocol round this run has reached. Presentation-readable.
    pub fn current_round(&self) -> u32 {
        self.current_round
    }

    /// How many card plays resolved to [`CardEffect::NotYetImplemented`] — the
    /// honest measure of how much of the card layer is still scaffolding.
    pub fn inert_card_plays(&self) -> u64 {
        self.inert_card_plays
    }

    // --- scheduling resource ----------------------------------------------

    fn schedule(&mut self, delay: f64, kind: EventKind) {
        self.seq += 1;
        self.queue.push(Reverse(Event { time: self.clock + delay.max(0.0), seq: self.seq, kind }));
    }

    fn schedule_at(&mut self, time: f64, kind: EventKind) {
        self.seq += 1;
        self.queue.push(Reverse(Event { time: time.max(self.clock), seq: self.seq, kind }));
    }

    // --- continuous position ----------------------------------------------

    /// Exact `(x, y, z)` of any spatial entity at absolute time `t`. Planets are
    /// fixed; ships are placed by the relativistic flip-and-burn. `None` for
    /// non-spatial entities (players).
    pub fn position_at(&self, e: Entity, t: f64) -> Option<Vec3> {
        if let Some(m) = self.world.motion.get(e) {
            Some(math::position_along(m.origin, m.dest, m.depart, m.arrive, m.accel, t))
        } else {
            self.world.position.get(e).copied()
        }
    }

    /// Positions of every spatial entity at time `t`, in entity-index order.
    /// Deterministic: a pure function of state and `t`.
    pub fn positions_at(&self, t: f64) -> Vec<Vec3> {
        let mut out = Vec::with_capacity(self.world.entity_count());
        for i in 0..self.world.entity_count() {
            let e = self.world.entity_at(i);
            if let Some(p) = self.position_at(e, t) {
                out.push(p);
            }
        }
        out
    }

    // --- the run loop ------------------------------------------------------

    pub fn step(&mut self) -> bool {
        match self.queue.peek() {
            Some(Reverse(ev)) if ev.time <= self.config.horizon_years => {}
            _ => return false,
        }
        let Reverse(ev) = self.queue.pop().unwrap();
        self.clock = ev.time;
        self.events_processed += 1;
        match ev.kind {
            EventKind::ContactArrive { vehicle } => self.sys_contact_arrive(vehicle),
            EventKind::ScanReport { player, planet } => self.sys_scan_report(player, planet),
            EventKind::ColonyArrive { vehicle } => self.sys_colony_arrive(vehicle),
            EventKind::MiningArrive { vehicle } => self.sys_mining_arrive(vehicle),
            EventKind::FreighterArrive { vehicle } => self.sys_freighter_arrive(vehicle),
            EventKind::ReturnArrive { vehicle } => self.sys_return_arrive(vehicle),
            EventKind::MiningTick { outpost } => self.sys_mining_tick(outpost),
            EventKind::ProductionTick { center } => self.sys_production_tick(center),
            EventKind::ScrapArrive { vehicle } => self.sys_scrap_arrive(vehicle),
            EventKind::RoundBoundary { round } => self.sys_round_boundary(round),
        }
        true
    }

    pub fn run(&mut self) -> SimReport {
        while self.step() {}
        self.clock = self.clock.max(0.0).min(self.config.horizon_years);
        self.report()
    }

    // --- systems -----------------------------------------------------------

    fn sys_contact_arrive(&mut self, vehicle: Entity) {
        let voyage = *self.world.voyage.get(vehicle).unwrap();
        let owner = *self.world.owner.get(vehicle).unwrap();
        let p = owner.0 as usize;
        let pe = self.player_entity[p];
        let here = self.position_at(voyage.target, self.clock).unwrap();
        let reached_pid = *self.world.planet_id.get(voyage.target).unwrap();

        // Scan must travel home before a foundry can act on it (light-lag).
        let home = self.world.player_info.get(pe).unwrap().home;
        let home_pos = *self.world.position.get(home).unwrap();
        let delay = math::signal_delay_years(here.distance(home_pos));
        self.schedule(delay, EventKind::ScanReport { player: pe, planet: voyage.target });

        // Contact unit: continue scouting to the next nearest unscanned world.
        if voyage.hops + 1 < self.config.max_survey_hops {
            let mut cands = core::mem::take(&mut self.survey_scratch);
            self.fill_survey_candidates(p, &mut cands);
            let doctrine = *self.world.doctrine.get(pe).unwrap();
            let next = self.autopilots[p].choose_survey_target(&doctrine, here, voyage.heading_bias, &cands);
            self.survey_scratch = cands;
            if let Some(next_pid) = next {
                self.world.knowledge.get_mut(pe).unwrap().visited.insert(next_pid);
                let next = self.planet_entity[next_pid.0 as usize];
                let accel = doctrine.survey_accel_g * G;
                let dest = *self.world.position.get(next).unwrap();
                let arrive = self.set_leg(vehicle, here, dest, accel, 0.0);
                let v = self.world.voyage.get_mut(vehicle).unwrap();
                v.target = next;
                v.hops += 1;
                self.schedule_at(arrive, EventKind::ContactArrive { vehicle });
                self.log.push(
                    self.clock,
                    LogEvent::ContactArrived { player: p as u32, vehicle, planet: reached_pid, next: Some(next_pid) },
                );
                return;
            }
        }
        self.log
            .push(self.clock, LogEvent::ContactArrived { player: p as u32, vehicle, planet: reached_pid, next: None });
        // LCV, mission exhausted: head for the nearest friendly colony to
        // scrap (`Hyades_vehicle_roles.md` §4.1 — confirmed this
        // conversation). Every other hull type in this role would go to
        // Reserve instead (§4.6); only Scout-via-LCV is confirmed to scrap,
        // and LCV is the only hull type this role currently builds.
        if let Some(dest_e) = self.nearest_owned_planet(p, here) {
            let dest_pos = *self.world.position.get(dest_e).unwrap();
            let accel = self.config.civilian_accel_g * G;
            let arrive = self.set_leg(vehicle, here, dest_pos, accel, 0.0);
            self.schedule_at(arrive, EventKind::ScrapArrive { vehicle });
        } else {
            // No owned world at all (shouldn't happen — the homeworld always
            // is one) — fall back to Reserve rather than lose the ship.
            self.world.role.insert(vehicle, Role::Reserve);
            self.park(vehicle, here);
        }
    }

    fn sys_scan_report(&mut self, player: Entity, planet: Entity) {
        let pid = *self.world.planet_id.get(planet).unwrap();
        self.world.knowledge.get_mut(player).unwrap().scanned.insert(pid);
        self.log.push(self.clock, LogEvent::ScanReceived { player: self.player_index(player), planet: pid });
    }

    fn sys_colony_arrive(&mut self, vehicle: Entity) {
        let voyage = *self.world.voyage.get(vehicle).unwrap();
        let owner = *self.world.owner.get(vehicle).unwrap();
        let p = owner.0 as usize;
        let target = voyage.target;
        let here = self.position_at(target, self.clock).unwrap();
        let target_pid = *self.world.planet_id.get(target).unwrap();

        if !self.world.owner.contains(target) {
            // Found the colony; recycle the vehicle's hull into level-1 infra.
            self.world.owner.insert(target, owner);
            {
                let f = self.world.factors.get_mut(target).unwrap();
                f.infra = f.infra.max(1.0);
            }
            // Seed population from the pop *carried as cargo*
            // (`Hyades_vehicle_roles.md` §4.2/R-V9 — confirmed, not a flat
            // constant applied on arrival regardless of what was brought).
            let carried_pop = self.world.pop_cargo.get(vehicle).copied().unwrap_or(0.0);
            {
                let pop = self.world.population.get_mut(target).unwrap();
                if *pop < carried_pop {
                    *pop = carried_pop;
                }
            }
            // No mineral seed here — confirmed this conversation: "mineral
            // seed is only for homeworld." A colony starts with whatever its
            // own local density and (once built) mining-outpost hauling
            // bring it; see `most_needed_center` / `sys_freighter_arrive`.
            let pid = *self.world.planet_id.get(target).unwrap();
            self.world.knowledge.get_mut(self.player_entity[p]).unwrap().scanned.insert(pid);
            self.schedule(self.config.cycle_years, EventKind::ProductionTick { center: target });
            self.log.push(self.clock, LogEvent::ColonyFounded { player: p as u32, vehicle, planet: target_pid });
            self.park(vehicle, here); // recycled hull, now inert infrastructure
        } else {
            // Contested (R-AC8): a systems vehicle returns home, carried pop
            // and all — nothing is lost, it's available to re-task
            // (`Hyades_vehicle_roles.md` §4.2).
            let home = *self.world.home_center.get(vehicle).unwrap_or(&target);
            let home_pos = *self.world.position.get(home).unwrap();
            let accel = self.config.civilian_accel_g * G;
            let arrive = self.set_leg(vehicle, here, home_pos, accel, 0.0);
            self.schedule_at(arrive, EventKind::ReturnArrive { vehicle });
            self.log.push(self.clock, LogEvent::ColonyContested { player: p as u32, vehicle, planet: target_pid });
        }
    }

    fn sys_mining_arrive(&mut self, vehicle: Entity) {
        let voyage = *self.world.voyage.get(vehicle).unwrap();
        let owner = *self.world.owner.get(vehicle).unwrap();
        let p = owner.0 as usize;
        let outpost = voyage.target;
        let here = self.position_at(outpost, self.clock).unwrap();
        let pid = *self.world.planet_id.get(outpost).unwrap();

        // Miner holds station and extracts; mark the outpost in production.
        self.park(vehicle, here);
        self.world.knowledge.get_mut(self.player_entity[p]).unwrap().exploited.insert(pid);
        self.log.push(self.clock, LogEvent::VehicleParked { player: p as u32, vehicle, role: Role::Miner, at: pid });
        if self.active_mines.insert(outpost.0) {
            self.schedule(self.config.mining_tick_years, EventKind::MiningTick { outpost });
        }
    }

    fn sys_freighter_arrive(&mut self, vehicle: Entity) {
        let sh = *self.world.shuttle.get(vehicle).unwrap();
        let p = self.world.owner.get(vehicle).unwrap().0;
        // Capacity is now the *hull's*, not the role's (R-O58): it is a mass
        // derived from usable interior volume, so it must be read off the ship
        // that is actually here rather than off whatever hull the baseline
        // autopilot happens to pick for Freighters.
        let hull = *self.world.hull_type.get(vehicle).unwrap_or(&HullType::MediumSystems);
        let cap = hull.cargo_capacity(&self.config);

        if sh.outbound {
            // At the outpost: load ore from its stockpile into cargo.
            let avail = self.world.stockpile.get(sh.outpost).unwrap().basic_total();
            let load = cap.min(avail);
            if load > 0.0 {
                let moved = take_basics(self.world.stockpile.get_mut(sh.outpost).unwrap(), load);
                self.world.cargo.get_mut(vehicle).unwrap().add_basics(&moved);
                let outpost_pid = *self.world.planet_id.get(sh.outpost).unwrap();
                self.log.push(
                    self.clock,
                    LogEvent::FreighterTransfer {
                        player: p,
                        vehicle,
                        leg: FreighterLeg::Loaded,
                        amount: load,
                        at: outpost_pid,
                    },
                );
            }
            // Stop shuttling once the outpost is exhausted and empty.
            let dens = self.world.density.get(sh.outpost).unwrap().metallicity();
            if load <= 1e-9 && dens < self.config.density_floor {
                let here = self.position_at(sh.outpost, self.clock).unwrap();
                let outpost_pid = *self.world.planet_id.get(sh.outpost).unwrap();
                self.log.push(
                    self.clock,
                    LogEvent::VehicleParked { player: p, vehicle, role: Role::Freighter, at: outpost_pid },
                );
                self.park(vehicle, here);
                return;
            }
            // Route to whichever owned production center offers the best
            // *discounted* need — confirmed: "autopilot must haul minerals to
            // where they are needed," not back to one hardcoded partner, and
            // (R-P2) not across the galaxy to a marginally needier one either.
            // At the shipped `trade_decay_lambda = 0` this is exactly
            // `most_needed_center`. Falls back to the outpost's own paired home
            // center only if this owner holds no production center at all
            // (shouldn't happen; the homeworld always counts).
            let home = *self.world.home_center.get(vehicle).unwrap_or(&sh.outpost);
            let here = self.position_at(sh.outpost, self.clock).unwrap();
            let dest = self.best_delivery_center(PlayerId(p), here).unwrap_or(home);
            self.world.shuttle.get_mut(vehicle).unwrap().destination = dest;

            let from = self.position_at(sh.outpost, self.clock).unwrap();
            let to = *self.world.position.get(dest).unwrap();
            // Laden run: acceleration derated by the ore just loaded.
            let accel = self.laden_accel(vehicle, self.config.civilian_accel_g);
            let arrive = self.set_leg(vehicle, from, to, accel, 0.0);
            self.world.shuttle.get_mut(vehicle).unwrap().outbound = false;
            self.schedule_at(arrive, EventKind::FreighterArrive { vehicle });
        } else {
            // At the destination: deposit cargo into its stockpile.
            let cargo = *self.world.cargo.get(vehicle).unwrap();
            self.world.stockpile.get_mut(sh.destination).unwrap().add_basics(&cargo);
            {
                let c = self.world.cargo.get_mut(vehicle).unwrap();
                c.cyan = 0.0;
                c.magenta = 0.0;
                c.yellow = 0.0;
            }
            if cargo.basic_total() > 0.0 {
                let dest_pid = *self.world.planet_id.get(sh.destination).unwrap();
                self.log.push(
                    self.clock,
                    LogEvent::FreighterTransfer {
                        player: p,
                        vehicle,
                        leg: FreighterLeg::Deposited,
                        amount: cargo.basic_total(),
                        at: dest_pid,
                    },
                );
            }
            // Return leg always goes back to the fixed mining source — only
            // the delivery side is need-routed, not the pickup side.
            let from = self.position_at(sh.destination, self.clock).unwrap();
            let to = *self.world.position.get(sh.outpost).unwrap();
            let accel = self.laden_accel(vehicle, self.config.civilian_accel_g);
            let arrive = self.set_leg(vehicle, from, to, accel, 0.0);
            self.world.shuttle.get_mut(vehicle).unwrap().outbound = true;
            self.schedule_at(arrive, EventKind::FreighterArrive { vehicle });
        }
    }

    fn sys_return_arrive(&mut self, vehicle: Entity) {
        let here = self.position_at(vehicle, self.clock).unwrap_or(Vec3::ZERO);
        // A ship that bounced home (e.g. a contested Colonizer) has nothing
        // left to do under its old role; it goes to Reserve — standing,
        // re-taskable, never auto-scrapped (`Hyades_vehicle_roles.md` §4.6).
        // Scrapping is confirmed only for an exhausted Scout (§4.1), handled
        // separately in `sys_contact_arrive`.
        self.world.role.insert(vehicle, Role::Reserve);
        if let (Some(&owner), Some(&role), Some(&home)) =
            (self.world.owner.get(vehicle), self.world.role.get(vehicle), self.world.home_center.get(vehicle))
        {
            let pid = *self.world.planet_id.get(home).unwrap();
            self.log.push(self.clock, LogEvent::VehicleParked { player: owner.0, vehicle, role, at: pid });
        }
        self.park(vehicle, here); // idle, available for future tasking
    }

    /// An exhausted Scout reaches a friendly colony and scraps: recycled for
    /// its mineral value (`Hyades_vehicle_roles.md` §4.1/§4.6, confirmed —
    /// "reclaiming the mineral value is the rational move" once a
    /// completable mission is genuinely done). Marked `Role::Scrapped`, not
    /// removed — entities never despawn (this conversation's origin point);
    /// the entity ID stays resolvable, it just stops being tasked.
    fn sys_scrap_arrive(&mut self, vehicle: Entity) {
        let here = self.position_at(vehicle, self.clock).unwrap_or(Vec3::ZERO);
        let owner = self.world.owner.get(vehicle).copied();
        self.world.role.insert(vehicle, Role::Scrapped);
        self.park(vehicle, here);

        if let Some(dest_e) = self.nearest_owned_planet(owner.map(|o| o.0 as usize).unwrap_or(0), here) {
            let recovered = role_cost(Role::Scout, &self.config) * self.config.scrap_recovery_fraction;
            let colors = recovered / 3.0;
            let stock = self.world.stockpile.get_mut(dest_e).unwrap();
            stock.cyan += colors;
            stock.magenta += colors;
            stock.yellow += colors;
            if let Some(o) = owner {
                let pid = *self.world.planet_id.get(dest_e).unwrap();
                self.log.push(self.clock, LogEvent::VehicleScrapped { player: o.0, vehicle, at: pid, recovered });
            }
        }
    }

    fn sys_mining_tick(&mut self, outpost: Entity) {
        let pid = *self.world.planet_id.get(outpost).unwrap();
        let amt = {
            let d = self.world.density.get(outpost).unwrap();
            d.metallicity() * self.config.outpost_mining_fraction
        };
        if amt > self.config.density_floor {
            let extracted = self.world.density.get_mut(outpost).unwrap().extract(amt);
            self.world.stockpile.get_mut(outpost).unwrap().add_basics(&extracted);
            let density_after = self.world.density.get(outpost).unwrap().metallicity();
            self.log.push(
                self.clock,
                LogEvent::MineralsExtracted { planet: pid, amount: extracted.basic_total(), density_after },
            );
            self.schedule(self.config.mining_tick_years, EventKind::MiningTick { outpost });
        } else {
            self.active_mines.remove(&outpost.0); // mined out
            self.log.push(self.clock, LogEvent::MiningExhausted { planet: pid });
        }
    }

    fn sys_production_tick(&mut self, center: Entity) {
        let owner = match self.world.owner.get(center).copied() {
            Some(o) => o,
            None => return,
        };
        let p = owner.0 as usize;
        let pe = self.player_entity[p];
        let center_pid = *self.world.planet_id.get(center).unwrap();
        let doctrine = *self.world.doctrine.get(pe).unwrap();

        // 1) Local mining: the center works its own density into its stockpile.
        let amt = {
            let d = self.world.density.get(center).unwrap();
            d.metallicity() * self.config.center_mining_fraction
        };
        if amt > 0.0 {
            let extracted = self.world.density.get_mut(center).unwrap().extract(amt);
            self.world.stockpile.get_mut(center).unwrap().add_basics(&extracted);
            let density_after = self.world.density.get(center).unwrap().metallicity();
            self.log.push(
                self.clock,
                LogEvent::MineralsExtracted { planet: center_pid, amount: extracted.basic_total(), density_after },
            );
        }

        // 2) Grow: population logistic toward K = min(hab, bio, infra), paid for
        // out of the planet's biological mass.
        //
        // Population is **not** an exception to mass conservation (L6, amended):
        // a kiloton of people is a kiloton of biosphere that stopped being
        // biosphere. Growth draws from `bio` 1:1 and is capped by what is
        // actually standing; decline returns it. Biosphere then regrows
        // logistically toward `bio_max`, which makes it the only self-replenishing
        // stock in the engine and puts a *rate* — not just a ceiling — between an
        // empire and its population.
        //
        // Note the feedback this creates: drawing biosphere down lowers
        // `K = min(hab, bio, infra)`, so a world that grows too fast throttles
        // itself and then recovers. That is the intended ecology, not a bug.
        let growth = doctrine.growth_rate;
        let regen = self.config.biosphere_regen_rate * doctrine.biosphere_regen_bonus;
        let k = self.world.factors.get(center).unwrap().k();
        {
            let pop_now = *self.world.population.get(center).unwrap();
            let start = if pop_now < 0.01 { 0.01 } else { pop_now };
            let mut delta = 0.0;
            if k > 0.0 {
                let grown = (start + growth * start * (1.0 - start / k)).clamp(0.0, k);
                delta = grown - start;
            }
            let f = self.world.factors.get_mut(center).unwrap();
            if delta > 0.0 {
                // Cannot grow more people than there is biomass to make them of.
                delta = delta.min(f.bio.max(0.0));
                f.bio -= delta;
            } else {
                // Decline returns mass to the biosphere.
                f.bio -= delta;
            }
            *self.world.population.get_mut(center).unwrap() = start + delta;

            // Logistic regrowth toward the pristine ceiling.
            if f.bio_max > 0.0 && regen > 0.0 {
                f.bio += regen * f.bio.max(0.0) * (1.0 - f.bio / f.bio_max);
                f.bio = f.bio.clamp(0.0, f.bio_max);
            }
        }
        self.log.push(
            self.clock,
            LogEvent::PopulationStep { planet: center_pid, population: *self.world.population.get(center).unwrap(), k },
        );

        // 3) Build: weigh deepen-vs-expand under the mineral budget & level gate.
        let (infra, k_potential) = {
            let f = self.world.factors.get(center).unwrap();
            (f.infra, f.k_potential())
        };
        let level = self.bands.level(*self.world.population.get(center).unwrap());
        let center_pos = *self.world.position.get(center).unwrap();
        let stock_total = self.world.stockpile.get(center).unwrap().basic_total();
        let target_level = infra.round() + 1.0;

        let info = *self.world.player_info.get(pe).unwrap();
        // Live mineral pressure for this center: 1 when broke for its next infra
        // upgrade, 0 when it can comfortably afford it. Drives the ranking toward
        // mining when the empire is short.
        let mineral_pressure = self.mineral_pressure_of(center);
        let rctx =
            RankContext { scarcity: info.scarcity, holdings_centroid: self.holdings_centroid(p), mineral_pressure };
        let mut cands: Vec<Candidate> = Vec::new();
        {
            let knowledge = self.world.knowledge.get(pe).unwrap();
            for &pid in &knowledge.scanned {
                if knowledge.targeted.contains(&pid) {
                    continue;
                }
                let e = self.planet_entity[pid.0 as usize];
                if self.world.owner.contains(e) {
                    continue;
                }
                let view = self.view_of(e);
                let ranked = self.autopilots[p].rank(&doctrine, &view, &rctx);
                if ranked.class != PlanetClass::Barren {
                    cands.push(Candidate { view, ranked });
                }
            }
        }
        // Built after `cands`, so the survey decision can see how much frontier
        // this empire has left to aim at.
        let ctx = ProductionContext {
            center_pos,
            level,
            infra,
            k_potential,
            stockpile_total: stock_total,
            medium_min_level: self.config.medium_min_level,
            limited_min_level: self.config.limited_min_level,
            infra_cost: target_level,
            colonizer_cost: role_cost(Role::Colonizer, &self.config),
            mining_pair_cost: role_cost(Role::Miner, &self.config) + role_cost(Role::Freighter, &self.config),
            light_vehicle_cost: role_cost(Role::Scout, &self.config),
            candidate_count: cands.len(),
        };

        let order = self.autopilots[p].production_choice(&doctrine, &ctx, &cands);
        self.log.push(
            self.clock,
            LogEvent::ProductionDecision {
                player: p as u32,
                center: center_pid,
                pop_level: level,
                infra,
                k_potential,
                stockpile: stock_total,
                infra_cost: target_level,
                colonizer_cost: ctx.colonizer_cost,
                mining_pair_cost: ctx.mining_pair_cost,
                mineral_pressure,
                candidates_seen: cands.len() as u32,
                chosen: order,
            },
        );
        self.apply_build_with(p, center, center_pos, order, &cands);

        self.schedule(self.config.cycle_years, EventKind::ProductionTick { center });
    }

    // --- build application -------------------------------------------------

    /// Apply a production order. `candidates` is the empire's current candidate
    /// list, used to **task** a finished hull — production decides *what object*
    /// to make, role assignment decides *what it is for* (R-O29).
    fn apply_build_with(
        &mut self,
        p: usize,
        center: Entity,
        center_pos: Vec3,
        order: BuildOrder,
        candidates: &[Candidate],
    ) {
        let center_pid = *self.world.planet_id.get(center).unwrap();
        match order {
            BuildOrder::Idle => {}
            BuildOrder::UpgradeInfrastructure => {
                let target = self.world.factors.get(center).unwrap().infra.round() + 1.0;
                if self.world.stockpile.get_mut(center).unwrap().try_spend_total(target) {
                    self.world.factors.get_mut(center).unwrap().infra += 1.0;
                    let stockpile_after = self.world.stockpile.get(center).unwrap().basic_total();
                    self.log.push(
                        self.clock,
                        LogEvent::BuildApplied {
                            player: p as u32,
                            center: center_pid,
                            order,
                            cost: target,
                            stockpile_after,
                        },
                    );
                }
            }
            BuildOrder::Hull { hull_type, class } => {
                if !self.roster_permits(p, hull_type) {
                    return;
                }
                let doctrine = *self.world.doctrine.get(self.player_entity[p]).unwrap();
                // The job is chosen here, after the object exists — not in the
                // order, which is all a rival could read off the shipyard.
                let tasking = self.autopilots[p].assign_role(&doctrine, hull_type, class, candidates);
                let Some(Tasking { role, target }) = tasking else {
                    return; // nothing worth building this hull for right now
                };

                // A Miner is produced together with the Freighter that hauls for
                // it (roles §5: the nearest center produces both), so the pair is
                // one economic act even though it is two objects.
                let paired_freighter = role == Role::Miner;
                let mut cost = role_cost(role, &self.config);
                if paired_freighter {
                    if !self.roster_permits(p, role_hull_type(Role::Freighter)) {
                        return;
                    }
                    cost += role_cost(Role::Freighter, &self.config);
                }

                if let Some(t) = target {
                    self.mark_targeted(p, t);
                }
                if !self.world.stockpile.get_mut(center).unwrap().try_spend_total(cost) {
                    return;
                }
                match (role, target) {
                    (Role::Scout, _) => self.launch_survey(p, center_pos, Vec3::ZERO, 0),
                    (r, Some(t)) => {
                        let te = self.planet_entity[t.0 as usize];
                        self.spawn_courier(p, r, center, center_pos, te);
                        if paired_freighter {
                            self.spawn_freighter(p, center, center_pos, te);
                        }
                    }
                    (_, None) => {}
                }
                let stockpile_after = self.world.stockpile.get(center).unwrap().basic_total();
                self.log.push(
                    self.clock,
                    LogEvent::BuildApplied { player: p as u32, center: center_pid, order, cost, stockpile_after },
                );
            }
        }
    }

    /// May player `p` build `hull`? Always yes while `enforce_roster` is off —
    /// see that field for why it is off. Kept as one predicate so turning
    /// enforcement on is a config change rather than a code change.
    fn roster_permits(&self, p: usize, hull: HullType) -> bool {
        if !self.config.enforce_roster {
            return true;
        }
        self.world.roster.get(self.player_entity[p]).map(|r| r.has_hull(hull)).unwrap_or(false)
    }

    fn mark_targeted(&mut self, p: usize, target: PlanetId) {
        self.world.knowledge.get_mut(self.player_entity[p]).unwrap().targeted.insert(target);
    }

    /// Spawn a colony/mining courier flying `center → target`.
    fn spawn_courier(&mut self, p: usize, role: Role, center: Entity, from: Vec3, target: Entity) {
        let dest = *self.world.position.get(target).unwrap();
        let accel = self.config.civilian_accel_g * G;
        let e = self.world.spawn();
        self.world.owner.insert(e, PlayerId(p as u32));
        self.world.role.insert(e, role);
        self.world.hull_type.insert(e, role_hull_type(role));
        self.world.voyage.insert(e, Voyage { target, heading_bias: None, hops: 0 });
        self.world.cargo.insert(e, Minerals::default());
        // A Colonizer carries its founding population as cargo, consumed on
        // arrival (`Hyades_vehicle_roles.md` §4.2 — confirmed this
        // conversation: "1 pop as cargo").
        self.world.pop_cargo.insert(e, if role == Role::Colonizer { self.config.colony_seed_pop } else { 0.0 });
        self.world.home_center.insert(e, center);
        let arrive = self.set_leg(e, from, dest, accel, self.config.build_years);
        let ev = match role {
            Role::Colonizer => EventKind::ColonyArrive { vehicle: e },
            _ => EventKind::MiningArrive { vehicle: e },
        };
        self.schedule_at(arrive, ev);
        let target_pid = *self.world.planet_id.get(target).unwrap();
        self.log
            .push(self.clock, LogEvent::VehicleSpawned { player: p as u32, vehicle: e, role, from, to: target_pid });
    }

    fn spawn_freighter(&mut self, p: usize, center: Entity, from: Vec3, outpost: Entity) {
        let dest = *self.world.position.get(outpost).unwrap();
        let accel = self.config.civilian_accel_g * G;
        let e = self.world.spawn();
        self.world.owner.insert(e, PlayerId(p as u32));
        self.world.role.insert(e, Role::Freighter);
        self.world.hull_type.insert(e, role_hull_type(Role::Freighter));
        self.world.cargo.insert(e, Minerals::default());
        self.world.home_center.insert(e, center);
        self.world.shuttle.insert(e, Shuttle { outpost, destination: center, outbound: true });
        let arrive = self.set_leg(e, from, dest, accel, self.config.build_years);
        self.schedule_at(arrive, EventKind::FreighterArrive { vehicle: e });
        let outpost_pid = *self.world.planet_id.get(outpost).unwrap();
        self.log.push(
            self.clock,
            LogEvent::VehicleSpawned { player: p as u32, vehicle: e, role: Role::Freighter, from, to: outpost_pid },
        );
    }

    fn launch_survey(&mut self, p: usize, from: Vec3, heading: Vec3, hops: usize) {
        let mut cands = core::mem::take(&mut self.survey_scratch);
        self.fill_survey_candidates(p, &mut cands);
        let bias = if heading == Vec3::ZERO { None } else { Some(heading) };
        let doctrine = *self.world.doctrine.get(self.player_entity[p]).unwrap();
        let accel = doctrine.survey_accel_g * G;
        let picked = self.autopilots[p].choose_survey_target(&doctrine, from, bias, &cands);
        self.survey_scratch = cands;
        if let Some(target_pid) = picked {
            self.world.knowledge.get_mut(self.player_entity[p]).unwrap().visited.insert(target_pid);
            let target = self.planet_entity[target_pid.0 as usize];
            let dest = *self.world.position.get(target).unwrap();
            let e = self.world.spawn();
            self.world.owner.insert(e, PlayerId(p as u32));
            self.world.role.insert(e, Role::Scout);
            self.world.hull_type.insert(e, role_hull_type(Role::Scout));
            self.world.voyage.insert(e, Voyage { target, heading_bias: bias, hops });
            self.world.cargo.insert(e, Minerals::default());
            let arrive = self.set_leg(e, from, dest, accel, 0.0);
            self.schedule_at(arrive, EventKind::ContactArrive { vehicle: e });
            self.log.push(
                self.clock,
                LogEvent::VehicleSpawned { player: p as u32, vehicle: e, role: Role::Scout, from, to: target_pid },
            );
        }
    }

    /// Set a vehicle's flight leg and return its arrival time.
    fn set_leg(&mut self, e: Entity, origin: Vec3, dest: Vec3, accel: f64, build_delay: f64) -> f64 {
        let dist = origin.distance(dest);
        let travel = if dist > 0.0 { math::ship_travel_years(dist, accel) } else { 0.0 };
        let depart = self.clock + build_delay;
        let arrive = depart + travel;
        self.world.motion.insert(e, Motion { origin, dest, depart, arrive, accel });
        arrive
    }

    /// Acceleration (ly/yr²) for a vehicle setting out *now*, derated for the
    /// mass it is currently carrying: `a = base_g·G · dry / (dry + cargo)`.
    /// This is the `a = thrust / mass` relation with thrust ∝ `base_g` and mass
    /// = dry + laden cargo, so a fully-loaded freighter leaves its outpost
    /// slower than it returns empty.
    ///
    /// **Every mass here is in one unit, kilotons (R-O57/L6).** There is no
    /// cargo-mass coefficient any more: a mineral in the hold masses exactly
    /// what that mineral massed when it was hull, which is the whole content of
    /// conservation. The two constants this replaces — a flat `dry_mass` and a
    /// `cargo_mass_per_unit` of 0.2 — disagreed by 30× about what a mineral
    /// weighs, and the flat dry mass additionally derated a Limited hull exactly
    /// as hard as a General one.
    ///
    /// The consequence is that laden spreads get much wider, which is R-O58's
    /// point rather than a side effect: a Medium freighter under a full hold
    /// carries 15× its own dry mass and accelerates at 1/16 g, while an empty
    /// hull of any size does 1 g. Large hulls broadcast their load state; small
    /// ones do not (§9.2's non-combat source of small-fleet value).
    fn laden_accel(&self, e: Entity, base_g: f64) -> f64 {
        // **Colony cargo mass ≡ mineral cargo mass** (R-O32,
        // `Hyades_standing_layer_and_observation.md` §6.2). A hold full of
        // settlers weighs what a hold full of ore weighs, so the burn cannot be
        // used to tell a colonizer from a freighter. Before this, `pop_cargo`
        // was massless and a laden colony ship accelerated exactly like an empty
        // hull — a free read on the one thing §6.2 exists to conceal, since
        // acceleration is the long-range observable.
        let minerals = self.world.cargo.get(e).map(|m| m.basic_total()).unwrap_or(0.0);
        let pop = self.world.pop_cargo.get(e).copied().unwrap_or(0.0);
        let hull = self.world.hull_type.get(e).copied().unwrap_or(HullType::MediumSystems);
        let dry = hull_dry_mass(hull, &self.config).max(1e-9);
        let factor = dry / (dry + minerals + pop);
        base_g * G * factor
    }

    /// Park a vehicle at `pos` (degenerate motion ⇒ fixed position, not in flight).
    fn park(&mut self, e: Entity, pos: Vec3) {
        let accel = self.config.civilian_accel_g * G;
        self.world.motion.insert(e, Motion { origin: pos, dest: pos, depart: self.clock, arrive: self.clock, accel });
    }

    /// Fill `out` with this player's unowned, unvisited worlds. Takes a caller-
    /// owned buffer rather than returning a fresh `Vec`: this runs on every
    /// scout hop and every launch, and the list can hold thousands of
    /// `PlanetView`s, so allocating one per call was pure churn (memcpy alone
    /// was 4% of engine instructions). Callers `mem::take` the scratch buffer,
    /// fill it, and put it back.
    fn fill_survey_candidates(&self, p: usize, out: &mut Vec<SurveyView>) {
        out.clear();
        let visited = &self.world.knowledge.get(self.player_entity[p]).unwrap().visited;
        for &e in &self.planet_entity {
            let pid = *self.world.planet_id.get(e).unwrap();
            if visited.contains(pid) {
                continue;
            }
            // Remote tier only (autopilot-doc §1): position plus the K-ceiling
            // factors, which spectroscopy reads at interstellar range.
            //
            // Fog is **per player, not per unit** — `Knowledge` is one shared set
            // on the player entity, so a world any of this empire's craft has
            // been sent to is excluded for all of them. Nothing here models an
            // individual scout's ignorance; that would be a second, per-entity
            // fog layer, and cards issue *instant global* orders (card-contract
            // §2) so the command layer already acts empire-wide.
            //
            // Ownership is not filtered because it is not in the remote tier —
            // an unscanned world's owner is unknown to the *empire*, not merely
            // to the craft. Whether the autopilot should instead see realized
            // ownership everywhere (R-AC1's omniscient command view) is R-SIM3.
            let f = self.world.factors.get(e).unwrap();
            // Inferential tier (R-SIM3): a pop-4 world radiates the waste heat of
            // billions, which spectrometry reads across interstellar distance.
            // Reported unconditionally; `Doctrine::survey_avoids_inhabited`
            // decides whether the policy acts on it, and defaults to off.
            let pop = *self.world.population.get(e).unwrap();
            let industrial_signature = self.bands.level(pop) >= 4;
            out.push(SurveyView {
                id: pid,
                position: *self.world.position.get(e).unwrap(),
                habitability: f.hab,
                biosphere: f.bio,
                industrial_signature,
            });
        }
    }

    fn view_of(&self, e: Entity) -> PlanetView {
        let f = self.world.factors.get(e).unwrap();
        PlanetView {
            id: *self.world.planet_id.get(e).unwrap(),
            position: *self.world.position.get(e).unwrap(),
            habitability: f.hab,
            biosphere: f.bio,
            minerals: *self.world.density.get(e).unwrap(),
            owner: self.world.owner.get(e).copied(),
            pop_level: self.bands.level(*self.world.population.get(e).unwrap()),
        }
    }

    fn holdings_centroid(&self, p: usize) -> Vec3 {
        let me = PlayerId(p as u32);
        let mut sum = Vec3::ZERO;
        let mut n = 0.0;
        for &e in &self.planet_entity {
            if self.world.owner.get(e).copied() == Some(me) {
                sum = sum.add(*self.world.position.get(e).unwrap());
                n += 1.0;
            }
        }
        if n > 0.0 {
            sum.scale(1.0 / n)
        } else {
            let home = self.world.player_info.get(self.player_entity[p]).unwrap().home;
            *self.world.position.get(home).unwrap()
        }
    }

    /// Live mineral pressure for `center`: `1.0` when broke for its next
    /// infra upgrade, `0.0` when it can comfortably afford it. Callable for
    /// *any* owned production center, not just from within its own
    /// production tick — a Query (`Hyades_vehicle_roles.md` §1), computed
    /// fresh, never stored, which is what lets [`Self::most_needed_center`]
    /// compare need across the whole empire.
    fn mineral_pressure_of(&self, center: Entity) -> f64 {
        let infra = self.world.factors.get(center).map(|f| f.infra).unwrap_or(0.0);
        let stock = self.world.stockpile.get(center).map(|s| s.basic_total()).unwrap_or(0.0);
        let target_level = infra.round() + 1.0;
        (1.0 - stock / target_level.max(1.0)).clamp(0.0, 1.0)
    }

    /// The owned production center with the highest live mineral pressure —
    /// "autopilot must haul minerals to where they are needed for
    /// infrastructure upgrades and ship building," confirmed this
    /// conversation. This is the query a Freighter re-runs every time it
    /// loads cargo, so delivery tracks *current* need empire-wide rather
    /// than a route fixed at build time. Deterministic: ties broken by
    /// entity id. `None` only if this owner holds no production center at
    /// all (shouldn't happen — the homeworld always counts).
    fn most_needed_center(&self, owner: PlayerId) -> Option<Entity> {
        self.planet_entity.iter().copied().filter(|&e| self.world.owner.get(e).copied() == Some(owner)).max_by(
            |&a, &b| {
                self.mineral_pressure_of(a)
                    .partial_cmp(&self.mineral_pressure_of(b))
                    .unwrap_or(core::cmp::Ordering::Equal)
                    .then(a.cmp(&b))
            },
        )
    }

    /// **Where a laden freighter should actually take its ore** — need,
    /// discounted by how long it takes to get there.
    ///
    /// `score = mineral_pressure(center) · exp(−λ · t_transit)`
    ///
    /// This is the *same* `λ` the Exchange discounts a trade by
    /// (`Hyades_politics_trade_and_intelligence.md` §2.3), and that is the
    /// claim R-P2 conditions its ratification on: one constant should price a
    /// delivery whether the counterparty is your own colony or a rival's.
    /// Internal haulage is just a trade you clear with yourself, so if the
    /// discount is right for one it should be right for the other.
    ///
    /// **`λ = 0` reduces exactly to [`most_needed_center`]**, which is the
    /// shipped default and keeps this behaviour-neutral until it is ratified.
    /// That degeneracy is also why design law #5 keeps `most_needed_center`
    /// permanently: it was already the oracle for single-supply matching, and
    /// it is now the oracle for zero-discount routing too — the same function
    /// checking two different generalisations.
    fn best_delivery_center(&self, owner: PlayerId, from: Vec3) -> Option<Entity> {
        let lambda = self.config.trade_decay_lambda;
        if lambda <= 0.0 {
            return self.most_needed_center(owner);
        }
        let accel = self.config.civilian_accel_g * G;
        let mut best: Option<(Entity, f64)> = None;
        for e in self.planet_entity.iter().copied() {
            if self.world.owner.get(e).copied() != Some(owner) {
                continue;
            }
            let d = from.distance(*self.world.position.get(e).unwrap());
            let t = math::ship_travel_years(d, accel);
            let score = self.mineral_pressure_of(e) * (-lambda * t).exp();
            // Entity id breaks ties so the choice is total and deterministic.
            let better = match best {
                None => true,
                Some((be, bs)) => score > bs || (score == bs && e.0 < be.0),
            };
            if better {
                best = Some((e, score));
            }
        }
        best.map(|(e, _)| e)
    }

    /// The nearest planet owned by player `p` to `from` — used to send an
    /// exhausted Scout home to scrap (`sys_contact_arrive`). `None` only if
    /// the player owns nothing at all (shouldn't happen; the homeworld always
    /// counts).
    fn nearest_owned_planet(&self, p: usize, from: Vec3) -> Option<Entity> {
        let me = PlayerId(p as u32);
        self.planet_entity.iter().copied().filter(|&e| self.world.owner.get(e).copied() == Some(me)).min_by(|&a, &b| {
            let da = from.distance(*self.world.position.get(a).unwrap());
            let db = from.distance(*self.world.position.get(b).unwrap());
            da.partial_cmp(&db).unwrap_or(core::cmp::Ordering::Equal)
        })
    }

    /// The nearest planet (any owner, or none) to `from` — the "co-located"
    /// half of Fleet's definition (§5): a ship's *theater* is the system
    /// nearest to its current position.
    fn nearest_planet_id(&self, from: Vec3) -> PlanetId {
        self.planet_entity
            .iter()
            .copied()
            .min_by(|&a, &b| {
                let da = from.distance(*self.world.position.get(a).unwrap());
                let db = from.distance(*self.world.position.get(b).unwrap());
                da.partial_cmp(&db).unwrap_or(core::cmp::Ordering::Equal)
            })
            .map(|e| *self.world.planet_id.get(e).unwrap())
            .expect("galaxy has at least one planet")
    }

    /// Resolve a player *entity* handle back to its seat index (≤12 players,
    /// so the linear scan is trivial). Only events that originate from a raw
    /// player `Entity` (currently just `ScanReport`) need this.
    fn player_index(&self, e: Entity) -> u32 {
        self.player_entity.iter().position(|&pe| pe == e).expect("not a player entity") as u32
    }

    // --- reporting ---------------------------------------------------------

    pub fn report(&self) -> SimReport {
        let mut players = vec![PlayerReport::default(); self.players()];
        for (p, rep) in players.iter_mut().enumerate() {
            let me = PlayerId(p as u32);
            for &e in &self.planet_entity {
                if self.world.owner.get(e).copied() == Some(me) {
                    rep.planets_owned += 1;
                    if !self.world.homeworld.contains(e) {
                        rep.colonies += 1;
                    }
                    rep.total_population += *self.world.population.get(e).unwrap();
                }
            }
            let k = self.world.knowledge.get(self.player_entity[p]).unwrap();
            rep.mining_outposts = k.exploited.len();
            rep.scanned = k.scanned.len();
        }
        SimReport {
            time_years: self.clock,
            events_processed: self.events_processed,
            planets_scanned_total: players.iter().map(|r| r.scanned).sum(),
            players,
        }
    }

    /// A read-only picture for the presentation / command layer at the current
    /// instant, including every ship's exact position.
    pub fn snapshot(&self) -> Snapshot {
        let planets = self
            .planet_entity
            .iter()
            .map(|&e| {
                let f = self.world.factors.get(e).unwrap();
                let pop = *self.world.population.get(e).unwrap();
                PlanetSnapshot {
                    id: *self.world.planet_id.get(e).unwrap(),
                    position: *self.world.position.get(e).unwrap(),
                    habitability: f.hab,
                    biosphere: f.bio,
                    infrastructure: f.infra,
                    k: f.k(),
                    population: pop,
                    pop_level: self.bands.level(pop),
                    density: *self.world.density.get(e).unwrap(),
                    stockpile: *self.world.stockpile.get(e).unwrap(),
                    owner: self.world.owner.get(e).map(|o| o.0),
                    is_homeworld: self.world.homeworld.contains(e),
                }
            })
            .collect();

        // Vehicles are every entity carrying a role.
        let mut vehicles = Vec::new();
        for i in 0..self.world.entity_count() {
            let e = self.world.entity_at(i);
            if let Some(&role) = self.world.role.get(e) {
                let m = self.world.motion.get(e).unwrap();
                vehicles.push(VehicleSnapshot {
                    owner: self.world.owner.get(e).map(|o| o.0).unwrap_or(0),
                    kind: role.kind(),
                    position: self.position_at(e, self.clock).unwrap(),
                    cargo: *self.world.cargo.get(e).unwrap_or(&Minerals::default()),
                    in_flight: m.arrive > self.clock,
                });
            }
        }

        let players = (0..self.players())
            .map(|p| {
                let me = PlayerId(p as u32);
                let pe = self.player_entity[p];
                let mut snap = PlayerSnapshot::default();
                for &e in &self.planet_entity {
                    if self.world.owner.get(e).copied() == Some(me) {
                        snap.planets_owned += 1;
                        snap.total_population += *self.world.population.get(e).unwrap();
                        snap.stockpiled_total += self.world.stockpile.get(e).unwrap().basic_total();
                    }
                }
                snap.ships = vehicles.iter().filter(|v| v.owner == p as u32).count() as u32;
                let k = self.world.knowledge.get(pe).unwrap();
                snap.mining_outposts = k.exploited.len() as u32;
                snap.planets_scanned = k.scanned.len() as u32;
                snap
            })
            .collect();

        Snapshot { time_years: self.clock, players, planets, vehicles }
    }

    /// Fleet = same owner + same [`Role`] + co-located, computed fresh —
    /// **not** stored anywhere (`Hyades_vehicle_roles.md` §5, confirmed this
    /// conversation: "same-role and co-located is correct for fleets").
    /// "Co-located" = nearest system to a ship's position at `t`
    /// ([`Self::nearest_planet_id`]) — the same theater granularity the rest
    /// of the engine already uses (a star system is one point).
    pub fn fleets_at(&self, t: f64) -> Vec<FleetSummary> {
        let mut groups: std::collections::BTreeMap<(u32, Role, u32), Vec<Entity>> = std::collections::BTreeMap::new();
        for i in 0..self.world.entity_count() {
            let e = self.world.entity_at(i);
            let (Some(&role), Some(&owner)) = (self.world.role.get(e), self.world.owner.get(e)) else {
                continue;
            };
            let Some(pos) = self.position_at(e, t) else { continue };
            let theater = self.nearest_planet_id(pos);
            groups.entry((owner.0, role, theater.0)).or_default().push(e);
        }
        groups
            .into_iter()
            .map(|((owner, role, theater), ships)| FleetSummary { owner, role, theater: PlanetId(theater), ships })
            .collect()
    }
}

fn basic_index(b: Basic) -> usize {
    match b {
        Basic::Cyan => 0,
        Basic::Magenta => 1,
        Basic::Yellow => 2,
    }
}

/// Scarcity weights from a homeworld archetype: the one *poor* basic is scarce
/// (weighted up), so the autopilot values outposts that supply it.
fn scarcity_for(archetype: Option<Archetype>) -> [f64; 3] {
    let mut s = [1.0; 3];
    if let Some(a) = archetype {
        let (_, _, poor) = a.alignment();
        s[basic_index(poor)] = 2.0;
    }
    s
}

/// Remove `amount` total basics from a bank, in proportion to holdings, and
/// return what was removed (a freighter loading at an outpost).
fn take_basics(bank: &mut Minerals, amount: f64) -> Minerals {
    let total = bank.basic_total();
    let take = amount.min(total).max(0.0);
    let mut out = Minerals::default();
    if total <= 0.0 || take <= 0.0 {
        return out;
    }
    let f = take / total;
    out.cyan = bank.cyan * f;
    out.magenta = bank.magenta * f;
    out.yellow = bank.yellow * f;
    bank.cyan -= out.cyan;
    bank.magenta -= out.magenta;
    bank.yellow -= out.yellow;
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::CardId;
    use crate::galaxy::GalaxyConfig;

    /// Unit tests exercise *mechanics*, not the full expansion arc. The shipped
    /// defaults now snowball to thousands of vehicles across the 4,000-year
    /// horizon — that is the design, but it turns every full `run()` into a
    /// multi-second sim and the suite from 6 s into 315 s. So tests pin a short
    /// horizon; long-run coverage questions belong in an example or the offline
    /// search, not here.
    fn test_cfg(seed: u64) -> SimConfig {
        let mut cfg = SimConfig::new(seed);
        cfg.horizon_years = 600.0;
        cfg
    }

    #[test]
    fn only_an_inverted_hull_ladder_is_refused() {
        // The fault is now **one** condition, not two, and the story of how it
        // got to be two is the useful part.
        //
        // R-O58 coupled the cost ladder to the capacity ladder, so
        // `medium_fleet_size` is not just a price — it also sets how much
        // bigger a Medium hull is than a Limited one. Capacity used to be
        // normalised against the *live* Medium radius, i.e. divided by a
        // quantity that goes to zero as the ladder narrows, which made the
        // General : Medium ratio appear to diverge. That looked like a physical
        // absurdity and got a guard (`r_M < 1.25`). It was an artifact of the
        // normaliser. Against a fixed reference nothing diverges and narrow
        // ladders are perfectly meaningful, so the guard is gone.
        let mut cfg = SimConfig::new(1);

        cfg.medium_fleet_size = 3.0;
        assert!(cfg.hull_ladder_fault().is_none(), "the shipped ladder must be valid");

        // **Narrow is legal.** The Medium hull is nearly all shell and hauls
        // almost nothing; the General hull is entirely unaffected by that,
        // which is the tell that the old "explosion" was in the denominator.
        cfg.medium_fleet_size = 8.0;
        assert!(cfg.hull_ladder_fault().is_none(), "a narrow ladder is meaningful, not a fault");
        let m = HullType::MediumSystems.cargo_capacity(&cfg);
        let g = HullType::GeneralSystems.cargo_capacity(&cfg);
        assert!(m > 0.0 && m < 0.01, "Medium is nearly all shell: {m}");
        assert!(g > 100.0, "General is untouched by the Medium hull shrinking: {g}");

        // **Inverted is not.** A "Medium" hull cheaper — and therefore smaller
        // — than a "Limited" one is a contradiction in the naming, not a
        // physics the model can express, so it is still refused.
        cfg.medium_fleet_size = 9.0;
        assert!(cfg.hull_ladder_fault().is_some(), "equal costs means Medium is not larger");
        cfg.medium_fleet_size = 12.0;
        assert!(cfg.hull_ladder_fault().is_some(), "Medium cheaper than Limited inverts the ladder");
    }

    #[test]
    #[should_panic(expected = "degenerate hull ladder")]
    fn constructing_a_sim_on_a_degenerate_ladder_panics() {
        let galaxy = Galaxy::generate(GalaxyConfig::new(3, 1)).unwrap();
        let mut cfg = test_cfg(1);
        cfg.medium_fleet_size = 12.0;
        let _ = Simulation::with_baseline(galaxy, cfg);
    }

    #[test]
    fn the_round_layer_is_behaviour_neutral_while_everyone_passes() {
        // The baseline autopilot's `choose_card` returns `None`, so adding the
        // round layer must move *nothing*. This is what keeps every coverage
        // number in the tree — and the offline search resting on them — valid
        // across the card layer landing. Verified at the shipped defaults:
        // seed 1 / 3 seats / 4 kyr gives 1,044 colonies with and without.
        let galaxy = Galaxy::generate(GalaxyConfig::new(3, 1)).unwrap();
        let mut with_rounds = Simulation::with_baseline(galaxy, test_cfg(1));
        let a = with_rounds.run();

        let galaxy = Galaxy::generate(GalaxyConfig::new(3, 1)).unwrap();
        let mut cfg = test_cfg(1);
        cfg.years_per_round = 0.0; // disables the layer entirely
        let mut without = Simulation::with_baseline(galaxy, cfg);
        let b = without.run();

        for (pa, pb) in a.players.iter().zip(b.players.iter()) {
            assert_eq!(pa.colonies, pb.colonies);
            assert_eq!(pa.planets_owned, pb.planets_owned);
            assert_eq!(pa.total_population.to_bits(), pb.total_population.to_bits());
        }
        assert_eq!(a.planets_scanned_total, b.planets_scanned_total);
    }

    #[test]
    fn round_boundaries_fire_on_the_specified_cadence() {
        // 200 yr to the first, 400 yr between: at a 600 yr test horizon that is
        // rounds 0 and 1. The barrier is a scheduled event, so this also pins
        // that it chains itself rather than being swept for.
        let galaxy = Galaxy::generate(GalaxyConfig::new(3, 1)).unwrap();
        let mut cfg = test_cfg(1);
        cfg.years_to_first_round = 200.0;
        cfg.years_per_round = 400.0;
        let mut sim = Simulation::with_baseline(galaxy, cfg);
        sim.run();
        assert_eq!(sim.current_round(), 1, "600 yr horizon should reach round 1 and stop");

        // And it stops at the horizon rather than running away. Pinned at
        // 1,400 yr, not the 4,000 default: design law #14 — a full-length run
        // costs seconds, and this asserts a cadence, not a long-run property.
        // (1400-200)/400 = 3, so the last barrier is round 3.
        let galaxy = Galaxy::generate(GalaxyConfig::new(3, 1)).unwrap();
        let mut cfg = test_cfg(1);
        cfg.horizon_years = 1400.0;
        let mut long = Simulation::with_baseline(galaxy, cfg);
        long.run();
        assert_eq!(long.current_round(), 3, "(1400-200)/400 = 3, so the last barrier is round 3");
    }

    #[test]
    fn apply_orders_is_total_and_an_illegal_order_costs_nothing() {
        // net §5.1 / design law #15. Every input maps to a legal transition:
        // a bogus seat, a bogus card and an unaffordable card must all leave
        // the sim untouched rather than panicking or half-applying.
        let galaxy = Galaxy::generate(GalaxyConfig::new(3, 1)).unwrap();
        let mut sim = Simulation::with_baseline(galaxy, test_cfg(1));
        let before = sim.world.doctrine.get(sim.player_entity[0]).unwrap().growth_rate;

        sim.apply_orders(
            0,
            &[
                Order { seat: PlayerId(99), card: Some(CardId(3)), target: Target::None }, // no such seat
                Order { seat: PlayerId(0), card: Some(CardId(999)), target: Target::None }, // no such card
                Order { seat: PlayerId(1), card: None, target: Target::None },             // pass
            ],
        );

        assert_eq!(sim.world.doctrine.get(sim.player_entity[0]).unwrap().growth_rate, before);
    }

    #[test]
    fn a_politics_card_publishes_a_rivals_scans_without_asking() {
        // politics §5.3 / §6 — disclosure is the attack, and it is not opt-in.
        // Player 0 publishes player 1's scan record; player 2, who is not
        // involved at all, learns everything player 1 knew.
        let galaxy = Galaxy::generate(GalaxyConfig::new(3, 1)).unwrap();
        let mut sim = Simulation::with_baseline(galaxy, test_cfg(1));
        sim.run();

        let scanned_of = |s: &Simulation, p: usize| s.world.knowledge.get(s.player_entity[p]).unwrap().scanned.clone();
        let victim = scanned_of(&sim, 1);
        let bystander_before = scanned_of(&sim, 2);
        assert!(!victim.is_empty(), "the victim must know something worth publishing");
        assert!(!victim.is_subset(&bystander_before), "and the bystander must not already know it");

        // Fund the attacker so the play is affordable, then publish.
        let attacker_home = sim
            .planet_entity
            .iter()
            .copied()
            .find(|&e| sim.world.owner.get(e).copied() == Some(PlayerId(0)) && sim.world.stockpile.contains(e));
        if let Some(h) = attacker_home {
            let s = sim.world.stockpile.get_mut(h).unwrap();
            s.cyan += 100.0;
        }
        sim.apply_orders(1, &[Order { seat: PlayerId(0), card: Some(CardId(2)), target: Target::Player(PlayerId(1)) }]);

        let bystander_after = scanned_of(&sim, 2);
        assert!(victim.is_subset(&bystander_after), "everything the victim knew is now public");
        assert!(bystander_after.len() > bystander_before.len(), "and the bystander strictly gained");
    }

    fn run_default(players: usize, seed: u64) -> (Simulation, SimReport) {
        let galaxy = Galaxy::generate(GalaxyConfig::new(players, seed)).unwrap();
        let mut sim = Simulation::with_baseline(galaxy, test_cfg(seed));
        let report = sim.run();
        (sim, report)
    }

    #[test]
    fn survey_scans_planets() {
        let (_sim, report) = run_default(3, 42);
        assert!(report.planets_scanned_total > 3, "exploration stalled: {report:?}");
    }

    #[test]
    fn empires_expand_beyond_the_homeworld() {
        let (_sim, report) = run_default(3, 42);
        let total_colonies: usize = report.players.iter().map(|p| p.colonies).sum();
        assert!(total_colonies > 0, "no colonies founded: {report:?}");
    }

    #[test]
    fn population_grows_past_the_starting_towns() {
        let (_sim, report) = run_default(3, 42);
        // 3 homeworlds start at pop 2 each (=6); growth must exceed that.
        let total_pop: f64 = report.players.iter().map(|p| p.total_population).sum();
        assert!(total_pop > 6.5, "population did not grow: {total_pop}");
    }

    #[test]
    fn deterministic_same_seed_same_outcome() {
        let (_a, ra) = run_default(6, 7);
        let (_b, rb) = run_default(6, 7);
        assert_eq!(ra.events_processed, rb.events_processed);
        assert_eq!(ra.planets_scanned_total, rb.planets_scanned_total);
        let pa: Vec<usize> = ra.players.iter().map(|p| p.planets_owned).collect();
        let pb: Vec<usize> = rb.players.iter().map(|p| p.planets_owned).collect();
        assert_eq!(pa, pb);
    }

    #[test]
    fn vehicles_persist_they_do_not_despawn() {
        // Every spawned entity is still present at the end (planets + players +
        // every vehicle ever built); the engine never recycles indices.
        let (sim, _r) = run_default(3, 5);
        let vehicles =
            (0..sim.world.entity_count()).filter(|&i| sim.world.role.get(sim.world.entity_at(i)).is_some()).count();
        assert!(vehicles >= 18, "expected the 6×3 opening scouts to persist, got {vehicles}");
    }

    #[test]
    fn minerals_deplete_when_mined() {
        // The homeworld mines its own density every cycle, so its metallicity
        // must fall over the game.
        let galaxy = Galaxy::generate(GalaxyConfig::new(3, 11)).unwrap();
        let hw = galaxy.homeworlds[0];
        let before = galaxy.planet(hw).minerals.metallicity();
        let mut sim = Simulation::with_baseline(galaxy, test_cfg(11));
        sim.run();
        let after = {
            let e = sim.planet_entity[hw.0 as usize];
            sim.world.density.get(e).unwrap().metallicity()
        };
        assert!(after < before, "density did not deplete: {before} -> {after}");
    }

    #[test]
    fn positions_are_deterministic_and_continuous() {
        // Two identical runs must agree bit-for-bit on every entity's position
        // across a grid of times; and motion must never exceed c between samples.
        let mk = || {
            let g = Galaxy::generate(GalaxyConfig::new(3, 23)).unwrap();
            Simulation::with_baseline(g, SimConfig::new(23))
        };
        let mut a = mk();
        let mut b = mk();
        // advance both identically by stepping
        for _ in 0..2000 {
            a.step();
            b.step();
        }
        let t = a.clock();
        let pa = a.positions_at(t);
        let pb = b.positions_at(t);
        assert_eq!(pa.len(), pb.len());
        for (x, y) in pa.iter().zip(pb.iter()) {
            assert_eq!(x.x.to_bits(), y.x.to_bits());
            assert_eq!(x.y.to_bits(), y.y.to_bits());
            assert_eq!(x.z.to_bits(), y.z.to_bits());
        }
        // continuity / sub-luminal: small time step ⇒ small displacement (≤ c·dt).
        let dt = 0.5;
        let p0 = a.positions_at(t);
        let p1 = a.positions_at(t + dt);
        for (u, v) in p0.iter().zip(p1.iter()) {
            assert!(u.distance(*v) <= dt + 1e-6, "entity moved faster than light");
        }
    }

    #[test]
    fn logging_is_silent_by_default() {
        let (sim, _r) = run_default(3, 17);
        assert!(sim.log().is_empty(), "no category was enabled; log should be empty");
    }

    #[test]
    fn enabling_a_category_captures_real_events() {
        let galaxy = Galaxy::generate(GalaxyConfig::new(3, 17)).unwrap();
        let mut sim = Simulation::with_baseline(galaxy, test_cfg(17));
        sim.set_log_filter(crate::log::LogFilter::all());
        sim.run();

        assert!(!sim.log().is_empty(), "expected records once logging is enabled");
        assert!(sim.log().by_category(crate::log::LogCategory::Production).count() > 0);
        assert!(sim.log().by_category(crate::log::LogCategory::Mining).count() > 0);
        assert!(sim.log().by_category(crate::log::LogCategory::Vehicles).count() > 0);
        assert!(sim.log().by_category(crate::log::LogCategory::Scanning).count() > 0);
        // every player appears in at least one record
        for p in 0..3u32 {
            assert!(sim.log().by_player(p).count() > 0, "no records for player {p}");
        }
    }

    #[test]
    fn logging_does_not_affect_outcomes() {
        // The whole point of the diagnostic seam is that it's a pure side
        // channel: turning it on must not change a single bit of the
        // simulation's deterministic results.
        let mk = |logging: bool| {
            let g = Galaxy::generate(GalaxyConfig::new(6, 2024)).unwrap();
            let mut s = Simulation::with_baseline(g, test_cfg(2024));
            if logging {
                s.set_log_filter(crate::log::LogFilter::all());
            }
            s
        };
        let mut quiet = mk(false);
        let mut loud = mk(true);
        let rq = quiet.run();
        let rl = loud.run();

        assert_eq!(rq.events_processed, rl.events_processed);
        assert_eq!(rq.planets_scanned_total, rl.planets_scanned_total);
        for (pq, pl) in rq.players.iter().zip(rl.players.iter()) {
            assert_eq!(pq.planets_owned, pl.planets_owned);
            assert_eq!(pq.colonies, pl.colonies);
            assert_eq!(pq.mining_outposts, pl.mining_outposts);
            assert_eq!(pq.total_population.to_bits(), pl.total_population.to_bits());
        }
        assert!(!loud.log().is_empty());
        assert!(quiet.log().is_empty());
    }

    #[test]
    fn a_logged_vehicle_entity_resolves_back_to_a_position() {
        // The point of logging Entity (not just PlanetId) on vehicle events is
        // that a consumer can correlate a log line with the continuous-position
        // seam. Confirm that round trip actually works.
        let galaxy = Galaxy::generate(GalaxyConfig::new(2, 5)).unwrap();
        let mut sim = Simulation::with_baseline(galaxy, test_cfg(5));
        sim.set_log_filter(crate::log::LogFilter::none().with(crate::log::LogCategory::Vehicles));
        sim.run();

        let spawn = sim
            .log()
            .by_category(crate::log::LogCategory::Vehicles)
            .find_map(|r| match r.event {
                crate::log::LogEvent::VehicleSpawned { vehicle, .. } => Some(vehicle),
                _ => None,
            })
            .expect("expected at least one VehicleSpawned record");
        assert!(sim.position_at(spawn, 0.0).is_some(), "logged entity should resolve to a position");
    }

    #[test]
    fn cargo_derates_acceleration() {
        // A laden vehicle accelerates more slowly than an empty one, and an
        // empty hull of any size gets the full rate (R-O58: thrust and dry mass
        // both scale with area, so a_empty is size-independent).
        let galaxy = Galaxy::generate(GalaxyConfig::new(2, 1)).unwrap();
        let mut sim = Simulation::with_baseline(galaxy, SimConfig::new(1));
        let base = sim.config.civilian_accel_g;

        // fabricate a throwaway entity id with no cargo component → empty
        let empty = Entity(u64::MAX); // no cargo store entry ⇒ 0 cargo
        let a_empty = sim.laden_accel(empty, base);
        assert!((a_empty - base * G).abs() < 1e-12, "empty ship should get full accel");

        for hull in [HullType::LimitedSystems, HullType::MediumSystems, HullType::GeneralSystems] {
            let e = sim.world.spawn();
            sim.world.hull_type.insert(e, hull);
            let a = sim.laden_accel(e, base);
            assert!((a - base * G).abs() < 1e-12, "{hull:?} empty should get full accel, got {a}");
        }

        // an entity carrying cargo should accelerate strictly less
        let mut sim2 = Simulation::with_baseline(Galaxy::generate(GalaxyConfig::new(2, 1)).unwrap(), SimConfig::new(1));
        let laden = sim2.world.spawn();
        let m = Minerals { cyan: 5.0, ..Minerals::default() };
        sim2.world.cargo.insert(laden, m);
        sim2.world.hull_type.insert(laden, HullType::MediumSystems);
        let a_laden = sim2.laden_accel(laden, base);
        assert!(a_laden < a_empty, "laden accel {a_laden} should be < empty {a_empty}");

        // R-O57: one mass unit, so the derate is exactly dry/(dry+cargo) with no
        // conversion coefficient in between. An MSV costs 1/3 and hauls 5.
        let dry = hull_dry_mass(HullType::MediumSystems, &sim2.config);
        assert!((a_laden - base * G * dry / (dry + 5.0)).abs() < 1e-12);

        // The same load on a bigger hull derates *less* — dry mass is in the
        // denominator, so the spread is a statement about how full the hold is.
        let big = sim2.world.spawn();
        sim2.world.cargo.insert(big, m);
        sim2.world.hull_type.insert(big, HullType::GeneralSystems);
        assert!(sim2.laden_accel(big, base) > a_laden);
    }

    #[test]
    fn shell_model_ladders_are_derived_not_tuned() {
        // R-O58. The radius ladder falls out of the cost ladder (cost ∝ area),
        // and capacity falls out of the radius ladder (contents ∝ usable
        // volume). Neither adds a tunable.
        //
        // Pins the **reference** cost ladder (1:3:9) explicitly rather than
        // reading `medium_fleet_size` off the shipped defaults — this is a
        // structural invariant of the shell model, not a statement about
        // whatever value MC ratification has currently landed on
        // (`hull_type_cost_derives_from_the_fleet_size_config` covers that).
        let mut cfg = SimConfig::new(1);
        cfg.medium_fleet_size = 3.0;
        let (l, m, g) = (HullType::LimitedSystems, HullType::MediumSystems, HullType::GeneralSystems);

        // r = sqrt(cost ratio to Limited): 1 : √3 : 3 at the shipped 1:3:9.
        assert!((l.hull_radius(&cfg) - 1.0).abs() < 1e-12);
        assert!((m.hull_radius(&cfg) - 3f64.sqrt()).abs() < 1e-12);
        assert!((g.hull_radius(&cfg) - 3.0).abs() < 1e-12);

        // R-O57: dry mass *is* the cost, in one unit.
        for hull in [l, m, g] {
            let cost = hull.cost_fraction(&cfg) * cfg.general_vehicle_cost;
            assert!((hull_dry_mass(hull, &cfg) - cost).abs() < 1e-12);
        }

        // What survives of roles §6's 0/1/2: the *ordinal* content. A Limited
        // hull is all shell and carries nothing; each larger hull carries
        // strictly more. The magnitudes are geometry now, not the unit count —
        // General is ~20× Medium, not 2× (R-O64).
        assert_eq!(l.cargo_capacity(&cfg), 0.0);
        assert!((m.cargo_capacity(&cfg) - cfg.cargo_unit_size).abs() < 1e-12);
        assert!(g.cargo_capacity(&cfg) > m.cargo_capacity(&cfg));
        let ratio = g.cargo_capacity(&cfg) / m.cargo_capacity(&cfg);
        assert!((ratio - (2.0 / (3f64.sqrt() - 1.0)).powi(3)).abs() < 1e-9, "G:M capacity ratio {ratio}");

        // Design law #3: consolidation must win under geometry alone. The
        // pre-shell model failed this — a General hull cost 9× a Limited and
        // hauled 2 units where a Medium cost 3× and hauled 1, i.e. 0.100 vs
        // 0.067 cost per unit hauled, so *fragmenting* was cheaper.
        let per_kt = |h: HullType| h.cost_fraction(&cfg) * cfg.general_vehicle_cost / h.cargo_capacity(&cfg);
        assert!(per_kt(g) < per_kt(m), "bigger hull must be cheaper per kt hauled");
    }

    #[test]
    fn combat_acceleration_is_untouched_by_the_dry_mass_rebasing() {
        // `Combatant::max_accel` divides thrust by dry mass, and thrust is
        // defined as thrust-to-mass × dry mass, so the re-basing cancels
        // exactly. This is why R-O57/R-O58 need no combat re-certification —
        // pinned so a future change to `hull_base_thrust` cannot quietly break
        // the laser-vs-missile balance.
        let mut cfg = SimConfig::new(1);
        for hull in
            [HullType::LimitedSystems, HullType::MediumSystems, HullType::GeneralSystems, HullType::RapidOffensive]
        {
            let a = hull_base_thrust(hull, &cfg) / hull_dry_mass(hull, &cfg);
            cfg.general_vehicle_cost = 17.0; // any scale at all
            let b = hull_base_thrust(hull, &cfg) / hull_dry_mass(hull, &cfg);
            cfg.general_vehicle_cost = 1.0;
            assert!((a - b).abs() < 1e-12, "{hull:?}: empty accel must not depend on the mass scale");
        }
    }

    #[test]
    fn roster_unlocks_are_idempotent_and_deterministically_ordered() {
        let mut r = Roster::default();
        assert!(r.is_empty());
        r.unlock(HullType::MediumSystems, Class::Unnamed);
        r.unlock(HullType::LimitedSystems, Class::Meadow);
        r.unlock(HullType::LimitedSystems, Class::Meadow); // repeat is a no-op
        assert_eq!(r.len(), 2, "unlocking twice must not duplicate");
        assert!(r.has(HullType::LimitedSystems, Class::Meadow));
        assert!(r.has_hull(HullType::MediumSystems));
        assert!(!r.has_hull(HullType::GeneralOffensive));
        assert_eq!(r.class_for(HullType::LimitedSystems), Some(Class::Meadow));
        assert_eq!(r.class_for(HullType::GeneralSystems), None);
        // Sorted, so iteration is stable across runs — Design is per-player
        // state the balancer compares, and an unstable order would break that.
        let mut sorted = r.designs().to_vec();
        sorted.sort();
        assert_eq!(r.designs(), &sorted[..]);
    }

    #[test]
    fn seats_start_with_the_ratified_lsv_plus_lcv_roster() {
        // R-O42/§7.1: LSV and LCV only, one class each — at turn 0 a scout, a
        // settler and a hauler are the same object.
        let galaxy = Galaxy::generate(GalaxyConfig::new(3, 1)).unwrap();
        let sim = Simulation::with_baseline(galaxy, test_cfg(1));
        for p in 0..3 {
            let r = sim.world.roster.get(sim.player_entity[p]).expect("every seat has a roster");
            assert_eq!(r.len(), 2, "seat {p} roster should hold exactly the two seeded designs");
            assert!(r.has(HullType::LimitedSystems, Class::Meadow));
            assert!(r.has(HullType::LimitedContactVehicle, Class::Tor));
            assert!(!r.has_hull(HullType::MediumSystems), "MSV must not be unlocked at start");
        }
    }

    #[test]
    fn enforcing_the_starting_roster_forbids_the_medium_hull_which_is_why_it_is_off() {
        // The reason `enforce_roster` defaults to false, pinned so the tradeoff
        // is not rediscovered by surprise. The colonizer and freighter ride on
        // MSV, which §7.1's starting roster does not include, and the engine has
        // no card system to unlock it — so enforcement forbids every expansion
        // build permanently. Measured over a full 4,000-year run that is 3
        // colonies and 18 vehicles against 1,183 and 4,778.
        let galaxy = Galaxy::generate(GalaxyConfig::new(2, 3)).unwrap();
        let mut cfg = test_cfg(3);
        cfg.enforce_roster = true;
        let mut sim = Simulation::with_baseline(galaxy, cfg);

        // Seeded roster: the Limited pair is buildable, the Medium hull is not.
        assert!(sim.roster_permits(0, HullType::LimitedSystems));
        assert!(sim.roster_permits(0, HullType::LimitedContactVehicle));
        assert!(!sim.roster_permits(0, HullType::MediumSystems), "MSV must be forbidden by the seeded roster");

        // Unlocking it is the only thing that changes the answer, so the gate is
        // demonstrably the roster and not something incidental.
        sim.world.roster.get_mut(sim.player_entity[0]).unwrap().unlock(HullType::MediumSystems, Class::Unnamed);
        assert!(sim.roster_permits(0, HullType::MediumSystems));

        // And with enforcement off — the shipped default — nothing is gated.
        let galaxy2 = Galaxy::generate(GalaxyConfig::new(2, 3)).unwrap();
        let sim2 = Simulation::with_baseline(galaxy2, test_cfg(3));
        assert!(sim2.roster_permits(0, HullType::MediumSystems), "default config must not gate anything");
    }

    #[test]
    fn population_growth_is_paid_for_out_of_biosphere() {
        // L6, amended: population is no longer an exception to mass
        // conservation. Every kiloton of people is a kiloton of biosphere that
        // stopped being biosphere, so pop + bio is invariant across a growth
        // step once regrowth is switched off.
        let galaxy = Galaxy::generate(GalaxyConfig::new(2, 5)).unwrap();
        let mut cfg = test_cfg(5);
        cfg.biosphere_regen_rate = 0.0; // isolate the exchange from the regrowth
        let mut sim = Simulation::with_baseline(galaxy, cfg);
        let home = sim.world.player_info.get(sim.player_entity[0]).unwrap().home;
        sim.world.factors.insert(home, Factors { hab: 4.0, bio: 4.0, bio_max: 4.0, infra: 4.0 });
        *sim.world.population.get_mut(home).unwrap() = 1.0;

        let before = *sim.world.population.get(home).unwrap() + sim.world.factors.get(home).unwrap().bio;
        sim.sys_production_tick(home);
        let f = sim.world.factors.get(home).unwrap();
        let pop = *sim.world.population.get(home).unwrap();

        assert!(pop > 1.0, "population should have grown, got {pop}");
        assert!(f.bio < 4.0, "biosphere should have been drawn down, got {}", f.bio);
        assert!((pop + f.bio - before).abs() < 1e-9, "pop+bio not conserved: {before} -> {}", pop + f.bio);
    }

    #[test]
    fn biosphere_regrows_toward_its_pristine_ceiling_but_never_past_it() {
        let galaxy = Galaxy::generate(GalaxyConfig::new(2, 6)).unwrap();
        let mut sim = Simulation::with_baseline(galaxy, test_cfg(6));
        let home = sim.world.player_info.get(sim.player_entity[0]).unwrap().home;
        // A cratered ecology: standing mass far below the pristine ceiling.
        sim.world.factors.insert(home, Factors { hab: 4.0, bio: 0.5, bio_max: 4.0, infra: 4.0 });
        *sim.world.population.get_mut(home).unwrap() = 0.01;

        let mut last = 0.5;
        for _ in 0..40 {
            sim.sys_production_tick(home);
            let bio = sim.world.factors.get(home).unwrap().bio;
            assert!(bio <= 4.0 + 1e-9, "biosphere exceeded its ceiling: {bio}");
            last = bio;
        }
        assert!(last > 0.5, "a razed biosphere should recover over time, got {last}");
    }

    #[test]
    fn a_dead_biosphere_stays_dead_when_doctrine_zeroes_regrowth() {
        // The hostile-card case: reducing regen to zero makes the wound durable
        // rather than momentary.
        let galaxy = Galaxy::generate(GalaxyConfig::new(2, 7)).unwrap();
        let mut sim = Simulation::with_baseline(galaxy, test_cfg(7));
        let pe = sim.player_entity[0];
        sim.world.doctrine.get_mut(pe).unwrap().biosphere_regen_bonus = 0.0;
        let home = sim.world.player_info.get(pe).unwrap().home;
        sim.world.factors.insert(home, Factors { hab: 4.0, bio: 0.0, bio_max: 4.0, infra: 4.0 });
        *sim.world.population.get_mut(home).unwrap() = 0.01;

        for _ in 0..20 {
            sim.sys_production_tick(home);
        }
        assert_eq!(sim.world.factors.get(home).unwrap().bio, 0.0, "regen_bonus=0 must leave the biosphere dead");
    }

    #[test]
    fn exhausted_scouts_scrap_and_recover_minerals() {
        // "An LCV should scrap itself at the nearest friendly colony after
        // there's no unknown planets" — confirmed this conversation.
        // Constructed directly rather than run organically: at hex-derived
        // scale, individual hops take long enough (hundreds of years) that
        // a scout genuinely exhausting a few-hundred-planet galaxy can take
        // tens of thousands of years — real, and flagged separately as a
        // pacing question, but this test only needs to demonstrate the
        // mechanism, not depend on emergent full-galaxy exploration timing.
        let galaxy = Galaxy::generate(GalaxyConfig::new(2, 3)).unwrap();
        let mut sim = Simulation::with_baseline(galaxy, SimConfig::new(3));
        sim.set_log_filter(crate::log::LogFilter::all());

        let target = sim.planet_entity[10];
        let home = sim.world.player_info.get(sim.player_entity[0]).unwrap().home;

        // Mark every planet already visited by player 0 except the one this
        // scout is about to "arrive" at — so survey_candidates() is empty
        // the moment it lands, exactly the exhausted-mission condition.
        {
            let knowledge = sim.world.knowledge.get_mut(sim.player_entity[0]).unwrap();
            for &pid in &sim.planet_entity.iter().map(|&e| *sim.world.planet_id.get(e).unwrap()).collect::<Vec<_>>() {
                knowledge.visited.insert(pid);
            }
        }

        let v = sim.world.spawn();
        sim.world.owner.insert(v, PlayerId(0));
        sim.world.role.insert(v, Role::Scout);
        sim.world.hull_type.insert(v, HullType::LimitedContactVehicle);
        sim.world.voyage.insert(v, Voyage { target, heading_bias: None, hops: 0 });
        sim.world.cargo.insert(v, Minerals::default());
        let here = *sim.world.position.get(target).unwrap();
        sim.park(v, here);

        sim.sys_contact_arrive(v); // should find no candidates, head for scrap

        // Confirmed it's now traveling, not parked — motion should lead
        // toward its nearest owned planet (the homeworld here).
        let motion_dest = sim.world.motion.get(v).unwrap().dest;
        let home_pos = *sim.world.position.get(home).unwrap();
        assert!((motion_dest.x - home_pos.x).abs() < 1e-9 && (motion_dest.y - home_pos.y).abs() < 1e-9);

        // Run the sim forward to let the scheduled ScrapArrive actually fire.
        for _ in 0..10_000 {
            if !sim.step() {
                break;
            }
        }

        assert_eq!(sim.world.role.get(v).copied(), Some(Role::Scrapped));

        let recovered_events: Vec<_> = sim
            .log()
            .by_category(crate::log::LogCategory::Vehicles)
            .filter_map(|r| match r.event {
                crate::log::LogEvent::VehicleScrapped { recovered, vehicle, .. } if vehicle == v => Some(recovered),
                _ => None,
            })
            .collect();
        assert_eq!(recovered_events.len(), 1);
        assert!(recovered_events[0] > 0.0, "scrap should recover a positive amount");
    }

    #[test]
    fn mining_is_non_exclusive_between_owners() {
        // "mining is non-exclusive by default" — confirmed this conversation.
        // Two different owners' miners can both station at the same outpost.
        let galaxy = Galaxy::generate(GalaxyConfig::new(2, 3)).unwrap();
        let mut sim = Simulation::with_baseline(galaxy, SimConfig::new(3));
        let outpost = sim.planet_entity[10];

        for p in 0..2u32 {
            let v = sim.world.spawn();
            sim.world.owner.insert(v, PlayerId(p));
            sim.world.role.insert(v, Role::Miner);
            sim.world.voyage.insert(v, Voyage { target: outpost, heading_bias: None, hops: 0 });
            sim.world.cargo.insert(v, Minerals::default());
            sim.sys_mining_arrive(v); // must not reject the second owner
        }

        let pid = *sim.world.planet_id.get(outpost).unwrap();
        for p in 0..2usize {
            let pe = sim.player_entity[p];
            assert!(
                sim.world.knowledge.get(pe).unwrap().exploited.contains(&pid),
                "player {p} should have registered the outpost as exploited"
            );
        }
    }

    #[test]
    fn most_needed_center_picks_highest_pressure_not_nearest_or_first() {
        // Confirmed this conversation: "autopilot must haul minerals to
        // where they are needed" — the query itself, in isolation.
        let galaxy = Galaxy::generate(GalaxyConfig::new(2, 3)).unwrap();
        let mut sim = Simulation::with_baseline(galaxy, SimConfig::new(3));
        let home = sim.world.player_info.get(sim.player_entity[0]).unwrap().home;

        // A second owned "colony" — same owner, but starved (0 stockpile,
        // infra 1) vs. the homeworld, which we give a full stockpile so its
        // pressure reads ~0.
        let colony = sim.planet_entity[15];
        sim.world.owner.insert(colony, PlayerId(0));
        sim.world.factors.insert(colony, Factors { hab: 3.0, bio: 3.0, bio_max: 3.0, infra: 1.0 });
        sim.world.stockpile.insert(colony, Minerals::default());

        {
            let s = sim.world.stockpile.get_mut(home).unwrap();
            s.cyan = 10.0;
            s.magenta = 10.0;
            s.yellow = 10.0;
        }

        let home_pressure = sim.mineral_pressure_of(home);
        let colony_pressure = sim.mineral_pressure_of(colony);
        assert!(colony_pressure > home_pressure, "test setup should make colony strictly needier");

        let picked = sim.most_needed_center(PlayerId(0));
        assert_eq!(picked, Some(colony), "should route to the needier colony, not the funded homeworld");
    }

    #[test]
    fn freighter_delivers_to_need_not_its_original_pairing() {
        // The end-to-end version: a freighter built for one center still
        // delivers to a *different*, needier center once loaded.
        let galaxy = Galaxy::generate(GalaxyConfig::new(2, 3)).unwrap();
        let mut sim = Simulation::with_baseline(galaxy, SimConfig::new(3));
        let home = sim.world.player_info.get(sim.player_entity[0]).unwrap().home;

        let colony = sim.planet_entity[15];
        sim.world.owner.insert(colony, PlayerId(0));
        sim.world.factors.insert(colony, Factors { hab: 3.0, bio: 3.0, bio_max: 3.0, infra: 1.0 });
        sim.world.stockpile.insert(colony, Minerals::default());
        {
            let s = sim.world.stockpile.get_mut(home).unwrap();
            s.cyan = 10.0;
            s.magenta = 10.0;
            s.yellow = 10.0;
        }

        let outpost = sim.planet_entity[40];
        {
            let mut field = MineralField::default();
            field.set(Basic::Cyan, 5.0);
            field.set(Basic::Magenta, 5.0);
            field.set(Basic::Yellow, 5.0);
            sim.world.density.insert(outpost, field);
        }
        sim.world.stockpile.insert(outpost, Minerals::default());

        // Build a freighter "paired" with the homeworld (its home_center),
        // as apply_build would, but the homeworld is the *less* needy side.
        let from = *sim.world.position.get(home).unwrap();
        sim.spawn_freighter(0, home, from, outpost);
        let freighter = Entity(sim.world.entity_count() as u64 - 1);

        // Give the outpost stockpile something to load, then run the load leg.
        {
            let s = sim.world.stockpile.get_mut(outpost).unwrap();
            s.cyan = 5.0;
            s.magenta = 5.0;
            s.yellow = 5.0;
        }
        sim.sys_freighter_arrive(freighter); // loads, routes to most-needed

        let sh = *sim.world.shuttle.get(freighter).unwrap();
        assert_eq!(sh.destination, colony, "freighter should re-route to the needier colony");
        assert_ne!(sh.destination, home, "not back to its original pairing, which is well-funded");
    }

    #[test]
    fn colonizing_is_exclusive_second_claimant_is_contested() {
        // "colonizing is exclusive by default" — confirmed this conversation.
        // A second Colonizer arriving at an already-founded world bounces
        // rather than founding a duplicate claim.
        let galaxy = Galaxy::generate(GalaxyConfig::new(2, 4)).unwrap();
        let mut sim = Simulation::with_baseline(galaxy, SimConfig::new(4));
        let target = sim.planet_entity[20];
        let home0 = sim.world.player_info.get(sim.player_entity[0]).unwrap().home;
        let home1 = sim.world.player_info.get(sim.player_entity[1]).unwrap().home;

        let mk = |sim: &mut Simulation, p: u32, home: Entity| {
            let v = sim.world.spawn();
            sim.world.owner.insert(v, PlayerId(p));
            sim.world.role.insert(v, Role::Colonizer);
            sim.world.voyage.insert(v, Voyage { target, heading_bias: None, hops: 0 });
            sim.world.cargo.insert(v, Minerals::default());
            sim.world.pop_cargo.insert(v, sim.config.colony_seed_pop);
            sim.world.home_center.insert(v, home);
            v
        };
        let first = mk(&mut sim, 0, home0);
        let second = mk(&mut sim, 1, home1);

        sim.sys_colony_arrive(first);
        assert_eq!(sim.world.owner.get(target).copied(), Some(PlayerId(0)));

        sim.sys_colony_arrive(second);
        // still owned by player 0 — the second claimant did not overwrite it
        assert_eq!(sim.world.owner.get(target).copied(), Some(PlayerId(0)));
        // and the second vehicle is on its way home, not recycled into infra
        assert_ne!(sim.world.role.get(second).copied(), Some(Role::Scrapped));
    }

    #[test]
    fn colonizer_consumes_its_pop_cargo_on_founding() {
        let galaxy = Galaxy::generate(GalaxyConfig::new(2, 4)).unwrap();
        let mut sim = Simulation::with_baseline(galaxy, SimConfig::new(4));
        let target = sim.planet_entity[20];
        let home0 = sim.world.player_info.get(sim.player_entity[0]).unwrap().home;

        let v = sim.world.spawn();
        sim.world.owner.insert(v, PlayerId(0));
        sim.world.role.insert(v, Role::Colonizer);
        sim.world.voyage.insert(v, Voyage { target, heading_bias: None, hops: 0 });
        sim.world.cargo.insert(v, Minerals::default());
        sim.world.pop_cargo.insert(v, 1.0);
        sim.world.home_center.insert(v, home0);

        sim.sys_colony_arrive(v);
        let pop = *sim.world.population.get(target).unwrap();
        assert!((pop - 1.0).abs() < 1e-9, "colony should be seeded with the carried 1.0 pop, got {pop}");
    }

    #[test]
    fn colony_founding_seeds_no_minerals_only_pop() {
        // Corrected this conversation: "No mineral seed for colonies.
        // Mineral seed is only for homeworld." A freshly founded colony
        // with zero local density must start with exactly zero stockpile —
        // whatever it gets from here on comes from its own mining and/or
        // hauled-in freighter deliveries, never a founding windfall.
        let galaxy = Galaxy::generate(GalaxyConfig::new(2, 4)).unwrap();
        let mut sim = Simulation::with_baseline(galaxy, SimConfig::new(4));
        let target = sim.planet_entity[20];
        let home0 = sim.world.player_info.get(sim.player_entity[0]).unwrap().home;

        sim.world.density.insert(target, MineralField::default());

        let v = sim.world.spawn();
        sim.world.owner.insert(v, PlayerId(0));
        sim.world.role.insert(v, Role::Colonizer);
        sim.world.voyage.insert(v, Voyage { target, heading_bias: None, hops: 0 });
        sim.world.cargo.insert(v, Minerals::default());
        sim.world.pop_cargo.insert(v, 1.0);
        sim.world.home_center.insert(v, home0);

        sim.sys_colony_arrive(v);
        let stock = sim.world.stockpile.get(target).unwrap().basic_total();
        assert!(stock.abs() < 1e-9, "colony should start with zero minerals, got {stock}");
    }

    #[test]
    fn hull_type_cost_derives_from_the_fleet_size_config() {
        // Design law #6: 1:3:9 was explicit scaffolding, not a target, and it
        // has since been superseded on one leg by the gradient-step
        // ratification (medium_fleet_size: 3 -> 4.45). This asserts the
        // *derivation* — cost_fraction reads straight off the config fields —
        // rather than pinning a magnitude that MC ratification is expected to
        // keep moving. Structural invariants belong in
        // `shell_model_ladders_are_derived_not_tuned`, which uses an explicit
        // reference config rather than the shipped defaults.
        let mut cfg = SimConfig::new(1);
        for mfs in [3.0, 4.45, 6.0] {
            cfg.medium_fleet_size = mfs;
            let general = HullType::GeneralSystems.cost_fraction(&cfg) * cfg.general_vehicle_cost;
            let medium = HullType::MediumSystems.cost_fraction(&cfg) * cfg.general_vehicle_cost;
            let limited = HullType::LimitedSystems.cost_fraction(&cfg) * cfg.general_vehicle_cost;
            assert!((general - 1.0).abs() < 1e-9);
            assert!((medium - 1.0 / mfs).abs() < 1e-9);
            assert!((limited - 1.0 / cfg.limited_fleet_size).abs() < 1e-9);
            // 1 mineral buys exactly `mfs` mediums or `limited_fleet_size` limiteds.
            assert!((medium * mfs - general).abs() < 1e-9);
            assert!((limited * cfg.limited_fleet_size - general).abs() < 1e-9);
        }
    }

    #[test]
    fn fleets_group_by_owner_role_and_theater() {
        let galaxy = Galaxy::generate(GalaxyConfig::new(3, 9)).unwrap();
        let mut sim = Simulation::with_baseline(galaxy, test_cfg(9));
        sim.run();

        let fleets = sim.fleets_at(sim.clock());
        assert!(!fleets.is_empty(), "expected at least one fleet grouping");
        for fl in &fleets {
            assert!(!fl.ships.is_empty());
            // every ship in a fleet really does share owner + role.
            for &ship in &fl.ships {
                assert_eq!(sim.world.owner.get(ship).map(|o| o.0), Some(fl.owner));
                assert_eq!(sim.world.role.get(ship).copied(), Some(fl.role));
            }
        }
        // no ship appears in two different fleets.
        let mut seen = std::collections::BTreeSet::new();
        for fl in &fleets {
            for &ship in &fl.ships {
                assert!(seen.insert(ship), "ship {ship:?} appeared in more than one fleet");
            }
        }
    }
}
