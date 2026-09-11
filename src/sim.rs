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
use std::collections::{BTreeMap, BTreeSet, BinaryHeap};

use crate::autopilot::{
    Autopilot, BaselineAutopilot, BuildOrder, Candidate, Doctrine, PlanetView, ProductionContext, RankContext, Ranked,
    SurveyStrategy, SurveyView, Tasking,
};
use crate::cards::{self, CardEffect, Order, Target};
use crate::galaxy::{Galaxy, PlanetClass, PlanetId, PlayerId, PopBands};
use crate::log::{FreighterLeg, LogEvent, LogFilter, SimLog};
use crate::matching;
use crate::math::{self, Vec3, G};
use crate::resources::{Archetype, Basic, MineralField, Minerals};
use crate::rng::Rng;
use crate::snapshot::{PlanetSnapshot, PlayerSnapshot, Snapshot, VehicleKind, VehicleSnapshot};
use crate::units::{self, Band, BandTier, Kilotons, Length, Measure, Price, Volume};

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
    /// Clear the component, returning what was there. Vacating a slot is not
    /// the same as never having filled one — the shipyard-occupancy store
    /// (R-O69) is *presence-as-state*, so it needs to be able to empty.
    fn remove(&mut self, e: Entity) -> Option<T> {
        self.items.get_mut(e.0 as usize).and_then(|o| o.take())
    }

    /// Get the component, creating it from `f` if absent. Added for T-69's
    /// berth list, which is grown one entry at a time rather than written whole.
    fn entry_or_insert_with(&mut self, e: Entity, f: impl FnOnce() -> T) -> &mut T {
        let slot = self.slot(e.0 as usize);
        if slot.is_none() {
            *slot = Some(f());
        }
        slot.as_mut().unwrap()
    }
}

/// Marker tag for homeworld planet entities.
#[derive(Clone, Copy, Debug)]
struct Homeworld;

/// Carrying-capacity factors of a planet. Liebig `K = min`, over **Bands**.
///
/// The two ceilings are Band levels and the biosphere is a mass; they are
/// different types because the engine spent a long time treating them as the
/// same `f64`. See [`crate::units`] for the defect and the bridge that closes
/// it.
#[derive(Clone, Copy, Debug)]
struct Factors {
    hab: Band,
    /// **Standing biological mass, in kilotons** — the same unit as minerals
    /// and hull dry mass, which is what makes the exchange between them exact.
    ///
    /// Unlike minerals this is a *renewable* stock: it regrows logistically
    /// toward [`Self::bio_max`], and it is the only thing in the engine that
    /// increases without being built. Population growth **consumes** it (L6) —
    /// people are made of biomass, so population mass is conserved rather than
    /// conjured from an open reservoir. The draw is the *mass* the step adds,
    /// `KT(pop_after) − KT(pop_before)`, not the Band increment: a Band is a
    /// magnitude tier, and one more tier is four times the people, not four
    /// more of them.
    biomass: Kilotons,
    /// Pristine biosphere: the ceiling `biomass` regrows toward, also a mass.
    /// Cards raise or lower it; a strike that craters a world's ecology lowers
    /// this, not just the standing stock, which is what makes such damage
    /// durable.
    ///
    /// **Write it through [`Self::set_bio_max`]**, which keeps
    /// [`Self::bio_max_band`] in step. The pair is a cache, and a cache with
    /// two write paths is a bug waiting for a card to find it.
    bio_max: Kilotons,
    /// [`Self::bio_max`] read back onto the Band ladder — **cached, because it
    /// is on the hottest path in the engine.**
    ///
    /// `Kilotons::in_bands` is a `ln`, `k_potential` needs it, and
    /// `k_potential` is evaluated once per *scanned planet* per production
    /// decision. That is a transcendental in a loop that runs tens of millions
    /// of times a run, for a quantity nothing has changed since galaxy
    /// generation. Identical value, computed once (R-O70).
    bio_max_band: Band,
    /// **Built infrastructure, stored as the minerals standing in it**
    /// (`Hyades_industry.md` §1.3, T-70).
    ///
    /// It is a `Price` rather than a `Band` because a **Band is a reading, not a
    /// second thing to store** (`CLAUDE.md` §4) — and because the infrastructure
    /// ladder *is* the mineral ladder (R-O80), so the reading has to be taken on
    /// the **Cost** scale. `Price` is kilotons carrying that scale marker, which
    /// is what satisfies §1.3's "stored as a mass in kilotons" without silently
    /// moving every infrastructure threshold onto the mass ladder, where the
    /// rungs are `^1.5` apart and every gate would mean something different.
    ///
    /// Read the rung with [`Factors::infra_band`]; never `in_bands()`, which
    /// would take it on the wrong ladder.
    infra: Price,
}
impl Factors {
    /// Build a set of factors, deriving the cached Band reading of `bio_max`.
    #[inline]
    fn new(hab: Band, biomass: Kilotons, bio_max: Kilotons, infra: Price) -> Factors {
        Factors { hab, biomass, bio_max, bio_max_band: bio_max.in_bands(), infra }
    }

    /// The only way to move the pristine ceiling. Keeps the cached Band reading
    /// consistent — the card that craters an ecology must not leave
    /// `k_potential` reading the old world.
    #[inline]
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "no card mutates bio_max yet; the setter exists so the \
                                                     first one cannot desynchronise the cache"
        )
    )]
    fn set_bio_max(&mut self, m: Kilotons) {
        self.bio_max = m;
        self.bio_max_band = m.in_bands();
    }

    /// Liebig's minimum, **over Bands only**.
    ///
    /// The standing biomass is deliberately *not* a term here, and that is the
    /// unit fix. It used to be, which made `K` collapse as population ate the
    /// biosphere and recover as the biosphere regrew — a feedback loop that
    /// read as ecology but was arithmetic on mismatched units, a mass compared
    /// against two levels.
    ///
    /// The biosphere still binds population, in two ways that survive the fix:
    /// its *pristine ceiling* [`Self::bio_max`] enters as a Band (durable
    /// damage lowers what a world can ever hold), and its *standing stock* is
    /// the mass growth is paid out of (transient damage makes growth slow, not
    /// the ceiling low). Splitting those apart is what design law #11's
    /// "renewable stock" actually asks for.
    /// **Infrastructure is not a term** (`Hyades_industry.md` §1.1, T-67).
    ///
    /// It was, and while it was, infrastructure could not be attacked without
    /// attacking population: T-64's discrete logistic goes strongly negative
    /// above `K`, so cutting a world's ceiling makes its population *overshoot
    /// below* the new one rather than settle at it — `2K → 0.25K` in a single
    /// step, pinned by
    /// `a_colony_seeded_above_its_capacity_crashes_below_it`. Every industrial
    /// strike was a population strike with extra steps, and infrastructure is
    /// precisely the war target the design wants to be survivable.
    ///
    /// Removing the term beats softening the logistic: the crash **stays**
    /// where it is wanted — a habitability or biosphere strike still collapses
    /// a population, because those genuinely are a world's capacity to hold
    /// people — and nothing has to be tuned. No decline rate, no second time
    /// constant, and **no clamp**, which matters because a clamp is exactly
    /// what let T-64's broken logistic keep scoring well.
    #[inline]
    fn k(&self) -> Band {
        self.hab.min(self.bio_max_band)
    }
    /// The ceiling infrastructure is built *toward*.
    ///
    /// **Identical to [`Self::k`] since T-67**, and kept as a separate name
    /// because the autopilot's deepening staircase means something different by
    /// it: `k` is what population may reach, `k_potential` is what a centre is
    /// still allowed to build. They coincide today; they are not the same
    /// question, and the industrial ramp (`Hyades_industry.md` §6) will give
    /// the second one its own answer.
    ///
    /// `bio_max` is read in Bands here. That is exact rather than a
    /// convenience: a population at Band `b` masses `KT(b)` and the pristine
    /// biosphere masses `KT(bio_max_band)`, so this Band minimum and the mass
    /// budget bind at precisely the same place (pinned by
    /// `the_mass_budget_and_the_band_ceiling_bind_together`).
    #[inline]
    fn k_potential(&self) -> Band {
        self.k()
    }

    /// **The infrastructure rung, read off the stock** — the one place the
    /// Cost-ladder reading is taken (T-70).
    ///
    /// `infra` is stored as the minerals standing in it, so the rung is a `ln`
    /// away. That is a conversion and conversions are not free (`CLAUDE.md` §4:
    /// `band()` is 2.2x an arithmetic op), so call it at the **edges** — a
    /// production decision, a log line, a view — and never inside a loop over
    /// entities.
    #[inline]
    fn infra_band(&self, cfg: &SimConfig) -> Band {
        self.infra.band_from(cost_anchor(cfg))
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
    /// Worlds this empire has close-scanned.
    ///
    /// **A sorted `Vec`, not a `BTreeSet`**, because it is *iterated* on the
    /// hottest path — every production decision walks the whole thing, 1.03
    /// billion elements over a run. A B-tree walk is a pointer chase across
    /// boxed nodes; a sorted `Vec` is a sequential read, and §4's finding is
    /// that **locality beats element count**. Insertion is a binary search plus
    /// a memmove, and insertions are four orders of magnitude rarer than
    /// iterations. Order is identical to the `BTreeSet`'s, so nothing about
    /// determinism or results moves (R-O70).
    scanned: ScannedSet,
    /// Worlds a survey craft has been *dispatched to* (marked at launch, so two
    /// scouts never chase the same target). A **bitmap, not a set**: this is
    /// membership-tested once per planet per `survey_candidates` call and never
    /// iterated, and profiling put that one `BTreeSet::contains` at the top of
    /// the whole engine — 63% of instructions once scouts became plentiful.
    /// O(1) indexed access instead of O(log n) pointer chasing.
    visited: VisitedMask,
    /// Worlds already claimed by a build order this empire has placed.
    ///
    /// **A bitmap, for exactly the reason `visited` is one** — and this one was
    /// missed when that lesson was learned. It is membership-tested once per
    /// scanned world per production decision and *never iterated*: profiled at
    /// **1.03 billion `contains` calls** on seed 1 at the shipped horizon,
    /// every one of them an `O(log n)` walk down a B-tree of boxed nodes.
    /// O(1) indexed access instead of pointer chasing (R-O70).
    targeted: VisitedMask,
    exploited: BTreeSet<PlanetId>,
}

/// Dense per-planet flag set, indexed by [`PlanetId`]. Grows on demand so a
/// default-constructed [`Knowledge`] needs no galaxy size up front.
#[derive(Clone, Debug, Default)]
struct VisitedMask {
    bits: Vec<bool>,
}

/// A sorted, deduplicated set of planet ids, kept in a contiguous `Vec` so
/// iteration is a sequential read. See [`Knowledge::scanned`] for why.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct ScannedSet {
    ids: Vec<PlanetId>,
}

impl ScannedSet {
    #[inline]
    fn insert(&mut self, pid: PlanetId) -> bool {
        match self.ids.binary_search_by_key(&pid.0, |p| p.0) {
            Ok(_) => false,
            Err(i) => {
                self.ids.insert(i, pid);
                true
            }
        }
    }
    #[inline]
    fn iter(&self) -> core::slice::Iter<'_, PlanetId> {
        self.ids.iter()
    }
    #[inline]
    fn len(&self) -> usize {
        self.ids.len()
    }
    #[cfg(test)]
    #[inline]
    fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }
    /// Both sides are sorted, so this is a linear merge rather than a lookup
    /// per element. Test-only — the engine never asks this.
    #[cfg(test)]
    fn is_subset(&self, other: &ScannedSet) -> bool {
        let mut it = other.ids.iter();
        self.ids.iter().all(|want| it.by_ref().any(|have| have == want))
    }
}

impl<'a> IntoIterator for &'a ScannedSet {
    type Item = &'a PlanetId;
    type IntoIter = core::slice::Iter<'a, PlanetId>;
    fn into_iter(self) -> Self::IntoIter {
        self.ids.iter()
    }
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

/// The ratified per-hull geometry (`Hyades_mineral_cost_curve.md` §2.3) — the
/// four numbers that turn a mineral price into a ship.
///
/// **These are spec-ratified, not MC-tuned.** `η` comes from §2.2's role×size
/// sphericity table and `τ` from §2.3's thickness solve; the two
/// `V_reserved` terms are §2.3's ratified role constants. None of them is a
/// free parameter for a search to move — they are what the Band ladders
/// *are*, and moving one without re-deriving the rungs breaks the tie
/// `F_mass = F_cost^(3/2)` that R-MC15 ratified.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HullGeometry {
    /// **`η`** — shape efficiency, §2.2. A General Systems Vehicle is a literal
    /// sphere (`1.000`, the anchor the other nine are measured against); every
    /// other hull needs more skin for the same volume, so its cost buys less
    /// ship.
    pub shape_efficiency: f64,
    /// **`τ`** — absolute shell thickness. Rises `Limited < Medium < General`
    /// within a role and by role `Systems < Contact < Offensive`, the second
    /// being the armour statement expressed as geometry rather than as a combat
    /// constant (design law #2).
    pub shell_thickness: Length,
    /// **`a_role`** — the absolute engine/crew/avionics core every hull of this
    /// role carries regardless of size. It is what makes a Limited hull's hold
    /// nearly all core and almost no cargo *as a result* rather than as a
    /// special case.
    pub reserved_core: Volume,
    /// **`b_role`** — the role's own volume-proportional payload: weapons,
    /// magazines, armour backing, sensor arrays. It is what stops an Offensive
    /// hull becoming a freighter simply by being large.
    pub reserved_payload_fraction: f64,
}

// §2.3's thickness ladder, Systems row, and the per-role multipliers on it.
//
// **Written to full precision, not to the spec table's four decimals.** These
// are not free constants: §2.3 *solves* them from the ratified cost and the
// ratified hold, `τ = (cost·η + hold)^(1/3) − hold^(1/3)`, so a rounded value
// misses its Band rung. At four decimals the Medium hull's hold came out
// 0.9977 kt against a `Band I` of 1.000 — 0.2% low, and enough to make a
// Colonizer unable to carry a colony seed defined at exactly that rung. The
// rungs are what the numbers are *for*; carry them at full width.
//
// They reproduce the rungs at the **ratified cost ladder** (1 : 1/10 : 1/50).
// A config that moves `medium_fleet_size` without moving the mass ladder has
// broken `F_mass = F_cost^(3/2)`, and the hold drifts off its rung — which is
// the tie being visible rather than a bug.
const TAU_LIMITED: f64 = 0.027_003_351_217_540_383;
const TAU_MEDIUM: f64 = 0.031_654_111_594_258_35;
const TAU_GENERAL: f64 = 0.032_988_014_798_952_3;
const TAU_CONTACT: f64 = 1.8;
const TAU_OFFENSIVE: f64 = 3.0;

impl HullType {
    /// Every hull type, in declaration order. Iterated rather than matched
    /// wherever a check must cover all ten, so a new variant cannot slip past
    /// it silently — [`SimConfig::hull_ladder_fault`] is the one that matters,
    /// because the hull it misses is the one that puts a NaN in hashed state.
    pub const ALL: [HullType; 10] = [
        HullType::LimitedSystems,
        HullType::MediumSystems,
        HullType::GeneralSystems,
        HullType::LimitedContactVehicle,
        HullType::LimitedContactUnit,
        HullType::GeneralContactVehicle,
        HullType::GeneralContactUnit,
        HullType::LimitedOffensive,
        HullType::RapidOffensive,
        HullType::GeneralOffensive,
    ];

    /// This hull's ratified geometry (§2.3). The table is written out rather
    /// than computed so that every one of the ten hulls is greppable and a new
    /// variant cannot inherit a neighbour's shell by accident.
    pub const fn geometry(self) -> HullGeometry {
        use HullType::*;
        let (eta, tau, core, payload) = match self {
            // Systems — roundest, thinnest-skinned, and the only role whose
            // reserved volume does not scale: cargo *is* its payload, which is
            // what makes its hold ladder purely geometric and makes it the row
            // §2.3 solves the ladder on.
            LimitedSystems => (0.86, TAU_LIMITED, 0.079, 0.00),
            MediumSystems => (0.98, TAU_MEDIUM, 0.079, 0.00),
            GeneralSystems => (1.000, TAU_GENERAL, 0.079, 0.00),
            // Contact — between Systems and Offensive on every axis, because
            // §2.3 derives it that way rather than fitting it.
            LimitedContactVehicle | LimitedContactUnit => (0.75, TAU_LIMITED * TAU_CONTACT, 0.132, 0.15),
            GeneralContactVehicle | GeneralContactUnit => (0.96, TAU_GENERAL * TAU_CONTACT, 0.132, 0.15),
            // Offensive — least round, thickest-skinned, largest reserve. The
            // `b` term is what zeroes a warship's hold at every size a warship
            // is actually built at.
            LimitedOffensive => (0.64, TAU_LIMITED * TAU_OFFENSIVE, 0.194, 0.45),
            RapidOffensive => (0.73, TAU_MEDIUM * TAU_OFFENSIVE, 0.194, 0.45),
            GeneralOffensive => (0.93, TAU_GENERAL * TAU_OFFENSIVE, 0.194, 0.45),
        };
        HullGeometry {
            shape_efficiency: eta,
            shell_thickness: Length::new(tau),
            reserved_core: Volume::new(core),
            reserved_payload_fraction: payload,
        }
    }

    /// **Absolute shell thickness `τ`** — the shell model's one geometric input
    /// (§2.3), and the thing that separates cost from capacity.
    ///
    /// Before T-56 this was the literal `1.0` subtracted inside
    /// `cargo_capacity`, which made "a Limited hull is all shell" a
    /// *definition* and left `medium_fleet_size` doing two jobs: since
    /// `r = sqrt(cost ratio)`, the price knob was also the hold knob, and
    /// `CLAUDE.md` §2 lists the measurement artifacts that came of it. With a
    /// real `τ` the two are independent functions of one body — cost is the
    /// shell, capacity is the hold.
    pub const fn shell_thickness(self) -> Length {
        self.geometry().shell_thickness
    }

    /// Hull radius, solved from the price rather than square-rooted from it.
    ///
    /// §2 puts cost on the material actually bought, which is the **shell
    /// volume** shape-penalised by `η`:
    ///
    /// ```text
    /// cost · η = r³ − (r − τ)³ = 3τr² − 3τ²r + τ³
    /// ```
    ///
    /// a quadratic in `r` with the positive root
    ///
    /// ```text
    /// r = τ/2 + sqrt(12·τ·cost·η − 3τ⁴) / (6τ)
    /// ```
    ///
    /// **This supersedes `r = sqrt(cost / cost_Limited)`**, which was the
    /// constant-`τ` special case written as if it were the law — it reduces to
    /// `cost ≈ 3r²τ/η ∝ area` exactly when the thickness is held fixed, which
    /// is no longer true across roles or sizes.
    ///
    /// The discriminant goes negative only for a hull too cheap to be built at
    /// its own thickness (`4·cost·η < τ³`), which
    /// [`SimConfig::hull_ladder_fault`] rejects at construction — design law
    /// #16 makes a NaN here a fatal error, not a value. At the ratified ladder
    /// the tightest margin is the LOU's, at 24× clear.
    pub fn hull_radius(self, cfg: &SimConfig) -> Length {
        let g = self.geometry();
        let tau = g.shell_thickness.hull_units();
        let material = self.cost_fraction(cfg) * cfg.general_vehicle_cost * g.shape_efficiency;
        let disc = 12.0 * tau * material - 3.0 * tau * tau * tau * tau;
        Length::new(tau / 2.0 + disc.max(0.0).sqrt() / (6.0 * tau))
    }

    /// The hold's radius, `r − τ`, floored at zero.
    ///
    /// The floor is not a special case for small hulls — it is the statement
    /// that a hull whose skin is as thick as it is wide has no interior at all.
    pub fn hold_radius(self, cfg: &SimConfig) -> Length {
        (self.hull_radius(cfg) - self.shell_thickness()).max(Length::ZERO)
    }

    /// The hold's **volume**, `(r − τ)³` — the geometric quantity that sits on a
    /// Band rung (§2.3), as distinct from the cargo it can carry, which is this
    /// minus the role's `V_reserved` and is a mass.
    ///
    /// At the ratified ladder the Systems row lands on the rungs exactly:
    /// `Band Empty` 0.089, `Band I` 1.00, `Band II` 31.6.
    pub fn hold_volume(self, cfg: &SimConfig) -> Volume {
        self.hold_radius(cfg).cubed()
    }

    /// The **shell's** volume, `r³ − (r − τ)³` — the material actually bought,
    /// and therefore (L6/R-O57) the hull's cost and its dry mass.
    ///
    /// Now genuinely the inverse of [`hull_radius`](Self::hull_radius): this
    /// times `1/η` reproduces the cost that produced the radius, and
    /// `the_shell_model_round_trips_cost_through_geometry` pins it.
    pub fn shell_volume(self, cfg: &SimConfig) -> Volume {
        self.hull_radius(cfg).cubed() - self.hold_volume(cfg)
    }

    /// **Reserved volume** — `a_role + b_role · V` (§2.3): the engine/crew core
    /// plus the role's own scaling payload, both deducted from the hold before
    /// anything can be carried.
    pub fn reserved_volume(self, cfg: &SimConfig) -> Volume {
        let g = self.geometry();
        g.reserved_core + self.hull_radius(cfg).cubed() * g.reserved_payload_fraction
    }

    /// Cargo capacity **as a mass** (R-O58) — [`Kilotons`], the same unit as
    /// minerals, population, biosphere and the hull itself (L6). Typed rather
    /// than a bare `f64` because it is consumed by an acceleration term, which
    /// needs a mass and nothing else (§4, R-O66).
    ///
    /// ```text
    /// capacity = max(0, hold − V_reserved) · cargo_unit_size
    /// ```
    ///
    /// The Band rung is the **hold**; `V_reserved` is the role deduction on top
    /// of it, and [`SimConfig::cargo_unit_size`] is the one density that turns a
    /// volume into a mass. Three consequences, all of them §2.3's and none of
    /// them special-cased here:
    ///
    /// - **`max(0, ·)` is reached honestly.** A Limited Contact or Offensive
    ///   hull, and a Rapid Offensive one, have a reserve larger than their
    ///   whole hold, so they carry exactly nothing — the *capability* half of
    ///   roles §4's permissive rule, a fact rather than a competence penalty.
    /// - **A Limited Systems hull carries a little**, ~0.010 kt: a scout's
    ///   sample locker, because its core eats 88% of a `Band Empty` hold.
    /// - **A General Offensive Unit still carries ~2.2 kt** — troops, ordnance,
    ///   prize crews — so "Offensive has little to zero cargo" is size-dependent
    ///   rather than a flat zero.
    ///
    /// **The normaliser is gone.** Capacity used to be `(r − 1)³` scaled against
    /// a fixed reference radius `√3`, which made `cargo_unit_size` the hold of a
    /// hull no ladder actually produced and put a derived quantity in a
    /// denominator — the shape of four of the measurement artifacts in
    /// `CLAUDE.md` §2. A hold is now a volume and a density converts it.
    ///
    /// **This supersedes the abstract 0 / 1 / 2 unit count** of
    /// `Hyades_vehicle_roles.md` §6. That ladder was confirmed, but as a *unit
    /// count*, and a unit count is not a mass — read as one it puts contents on
    /// a near-linear ladder while the shell model puts them on a cubic one. What
    /// survives is its ordinal content: each larger hull carries strictly more.
    /// The magnitudes are geometry now (R-O64).
    pub fn cargo_capacity(self, cfg: &SimConfig) -> Kilotons {
        let usable = self.hold_volume(cfg) - self.reserved_volume(cfg);
        Kilotons::new(usable.max(Volume::ZERO).hull_units_cubed() * cfg.cargo_unit_size)
    }

    /// **The largest founding population this hull could deliver** — the people
    /// its hold holds.
    ///
    /// Since T-64 this is the hold mass itself, with no conversion: settlers
    /// are a mass and a hold is a mass, so the "capacity in Bands" step was
    /// converting a number into its own logarithm and back.
    ///
    /// A *capacity*, not a load: what actually flies is this capped at the
    /// target's carrying capacity, because a seed above `K` crashes rather than
    /// settling (T-56 stage 4). The ladder makes the capacities `Band Empty` /
    /// `Band I` / `Band II` for Limited / Medium / General.
    pub fn colony_seed_capacity(self, cfg: &SimConfig) -> Kilotons {
        Kilotons::new(self.hold_volume(cfg).hull_units_cubed() * cfg.cargo_unit_size)
    }

    /// Mineral cost as a fraction of `SimConfig::general_vehicle_cost` — the
    /// **anchor** the geometry is solved from, not a consequence of it.
    ///
    /// The General hull is the unit; `medium_fleet_size` and
    /// `limited_fleet_size` are the `Band I → II` and `Band Empty → I` steps of
    /// the ratified cost ladder (§2.6, R-MC15), so this is `1 : 1/10 : 1/50`.
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

// The two constants this block used to hold — `REFERENCE_MEDIUM_RADIUS` (√3)
// and `UNIT_SHELL_THICKNESS` (1.0) — are **deleted by T-56 stage 3c.** Both
// were consequences of a geometry that no longer exists: capacity was
// `(r − 1)³` normalised against a fixed reference radius, so the shell
// thickness was a literal `1` and `cargo_unit_size` was the hold of a hull no
// ladder actually produced. Thickness is now a ratified per-hull quantity
// (`HullType::geometry`) and capacity is a volume times a density, so there is
// no normaliser left to put a derived quantity in a denominator — which is
// what four of the measurement artifacts in `CLAUDE.md` §2 had in common.

// `FOUNDING_INFRA_AT_MEDIUM` is **deleted (R-O77 closed)**. It was the anchor a
// subsidised founding rate scaled from: a Medium hull cost 0.1 minerals and
// became a Band of infrastructure the ladder charges 1.0 for, so founding
// conjured 10× the mass that was spent. There is no rate any more — a hull's
// mass *is* the infrastructure it becomes, which is what design law #11 said
// all along.

/// **The infrastructure ladder — priced in minerals, on the mineral ladder
/// (R-O80 closed).**
///
/// Infrastructure is bought with minerals, so its rungs are the *cost* ladder's
/// rungs: **`Infra I` costs `minerals I`**, rung for rung. `general_vehicle_cost`
/// is cost `Band II` (§2.6), which anchors the rest through the ratified
/// factors `5, 10, 20, 40`:
///
/// | rung | minerals to be at it | step from the rung below |
/// |---|---|---|
/// | `Empty` | 0.02 | — |
/// | `I` | 0.10 | 0.08 |
/// | `II` | 1.00 | 0.90 |
/// | `III` | 20.0 | 19.0 |
/// | `IV` | 800 | 780 |
///
/// **This replaces `round(infra) + 1`**, which charged 1 / 2 / 3 / 4 — a linear
/// count of Band *numerals*, and the last Band-additive quantity in the engine.
/// It arrived in the initial import with no derivation beyond its doc line
/// ("infrastructure upgrades cost minerals equal to the target level") and
/// survived because `Band` still had an `Add`.
///
/// The shape of the economy changes with it: the first two rungs get an order
/// of magnitude cheaper and `Band III` becomes a real investment rather than
/// three minerals. That is what a multiplicative ladder means — and it is why
/// the correspondence with hull prices is *exact*. The rungs **are** the hull
/// costs (`Limited` 0.02, `Medium` 0.10, `General` 1.00), so a recycled hull
/// buys precisely the infrastructure its minerals would have bought, and
/// [`Simulation::founding_infra`] needs no separate rate.
/// The cost ladder's anchor for this configuration: minerals at cost `Band I`,
/// one step below `general_vehicle_cost` (which §2.6 puts at `Band II`).
///
/// This is the one runtime parameter the ladder takes. Everything else about it
/// is the ratified `COST_LADDER`, carried on the `Qty<Cost>` type itself.
#[inline]
fn cost_anchor(cfg: &SimConfig) -> f64 {
    cfg.general_vehicle_cost / units::COST_LADDER[1]
}

/// **Workable veins in a deposit** — `N(S) = veins_per_band^(band(S) − 1)`
/// (`Hyades_industry.md` §4.2, T-71).
///
/// Two miners cannot work the same vein, so capacity beyond the first must open
/// another one and the best veins go first. That makes extraction *sublinear* in
/// crew — Lanchester's lesson with the sign reversed, because sites are
/// exclusive where fire concentrates. But **crowding is relative to the body**:
/// a bigger deposit has more veins, so `n` miners crowd a pebble and rattle
/// around a seam.
///
/// | deposit | mass | `N` |
/// |---|---|---|
/// | `Band I` | 1 kt | 1 |
/// | `Band II` | 31.6 kt | 10 |
/// | `Band III` | 2,828 kt | 100 |
/// | `Band IV` | 715,500 kt | 1,000 |
///
/// Floored at one: a body always has somewhere to put the first miner, and the
/// `Band Empty` floor (design law #11) means `band()` never returns a magnitude
/// that would drive this to zero.
fn veins(deposit: Kilotons, cfg: &SimConfig) -> f64 {
    let band = deposit.in_bands().bands();
    // `veins_per_band` below 1 would make `powf` of a negative exponent diverge
    // and put an infinity into a quantity the simulation divides by — design
    // law #16 says an infinity is a fatal value, not a small one, because it
    // becomes a NaN in one subtraction. Clamped at the edge rather than checked
    // at every call site.
    let per_band = cfg.veins_per_band.max(1.0);
    per_band.powf(band - 1.0).clamp(1.0, MAX_VEINS)
}

/// Ceiling on the vein count, and therefore on any crew derived from it.
///
/// Not a design constant — `Hyades_industry.md` §4.2's ladder tops out at 1,000
/// for `Band IV`, four orders below this. It exists so that a mis-set
/// `veins_per_band` produces an absurd number rather than an unbounded one: a
/// crew is allocated as hulls, so an unclamped `N` is an allocation loop, not a
/// wrong answer.
const MAX_VEINS: f64 = 1.0e7;

/// **Work performed at a site** — `W(n, S) = N(S)^(1−β) · n^β` for `n ≤ N(S)`
/// (`Hyades_industry.md` §4.3, T-71).
///
/// `n` is **extraction capacity in miner-equivalents**: hulls on station for an
/// outpost, infrastructure allocated to extraction for a colony, and the whole
/// point of the shape is that it is *one law for both*. §4.4 is explicit that an
/// asymmetry here would make "outpost or colony?" a question about rate shape
/// when it should be a question about commitment.
///
/// Three properties, and they are why this shape rather than a bare `n^β`:
///
/// - **A full crew pays linearly in richness.** `W(N, S) = N`, so a `Band IV`
///   body worked by its thousand miners yields a thousand times a `Band I`
///   body's one.
/// - **A large crew on a rich rock is rational.** On `Band IV` one miner does
///   `W = 31.6` and a thousand do `W = 1000`.
/// - **A poor rock saturates at once.** On `Band I`, `N = 1` and the second
///   miner adds nothing.
///
/// **Past `N` this is flat, and that is a placeholder** (R-IND9). §4.3 says the
/// marginal miner beyond `N` gets the *floor grade* rather than zero — there is
/// always more poor ore — so the cap should be a knee and not a wall. Flat is
/// the cheaper of the two candidates §4.3 lists and it matters only for absurd
/// crews; R-IND9 is open and says to settle it by what reads better in a log.
fn extraction_work(capacity: f64, deposit: Kilotons, cfg: &SimConfig) -> f64 {
    let n_veins = veins(deposit, cfg);
    let n = capacity.clamp(0.0, n_veins);
    if n <= 0.0 {
        return 0.0;
    }
    let beta = cfg.crowding_beta;
    n_veins.powf(1.0 - beta) * n.powf(beta)
}

/// **The share of a deposit's veins a crew effectively works** — `W(n,S)/N(S)`,
/// which reduces to `(n/N)^β` (T-71, R-IND19).
///
/// This is the form the engine multiplies the stock by, and it is **not** what
/// §4.3 writes. §4.3 gives `extraction = ε · S · W`, and that double-counts the
/// deposit: `W` rises with `N` and `N` rises with `S`, so output would go as
/// richness *squared*. Concretely, at `ε = outpost_mining_fraction` a full crew
/// on a `Band III` body would lift **307% of everything present** in one tick —
/// the opposite of §4.3's own "output tracks the stock, so a body depletes
/// asymptotically rather than cliff-edging".
///
/// **Every ratio §4.3 asserts survives the normalisation**, because dividing by
/// `N` is a change of scale that `ε` absorbs, and §4.3's claims are all about
/// ratios:
///
/// - *"one miner does `W = 31.6` and a thousand do `W = 1000` — 31.6x the ore
///   for 1000x the hulls"* on `Band IV`: here `(1/1000)^½ = 0.0316` against
///   `(1000/1000)^½ = 1`. **The same 31.6x.**
/// - *"a full crew pays linearly in the deposit's richness"*: a full crew takes
///   `ε·S` on every body, and `S` is the richness. Linear, once.
/// - *"a poor rock saturates immediately"*: on `Band I`, `N = 1`, so the second
///   miner is clamped away and adds nothing.
///
/// So the disagreement is confined to the absolute scale, which was never
/// measured either way (R-IND18), and the engine takes the reading that keeps a
/// body finite. **R-IND19** carries the spec correction.
fn crowding_factor(capacity: f64, deposit: Kilotons, cfg: &SimConfig) -> f64 {
    extraction_work(capacity, deposit, cfg) / veins(deposit, cfg)
}

/// **One miner-equivalent of extraction capacity, in kilotons** (T-71).
///
/// §4.3 says `n` counts "miner hulls on station **plus** units of
/// Infrastructure allocated to extraction", which only type-checks if the two
/// are commensurable — and under R-O57 they already are, because dry mass *is*
/// mineral cost. So the unit is **a miner hull's own mass**: the baseline
/// autopilot mines with `LimitedSystems`, one hull is one unit by construction,
/// and a colony's infrastructure converts at the same rate rather than at a
/// second constant nobody could calibrate.
///
/// Defining it this way is what makes §4.4's "one law, two ways to buy
/// capacity" literally one law instead of two laws that resemble each other.
fn miner_equivalent(cfg: &SimConfig) -> f64 {
    hull_cost(HullType::LimitedSystems, cfg).kilotons().max(1e-12)
}

/// **How many builds a yard runs at once** — `slips(F) = 1 + floor(F / F_slip)`
/// (T-69, `Hyades_industry.md` §3.2).
///
/// Every `F_slip` of fabrication throughput buys another berth, and a centre
/// always has at least one. **This is the "build wide" axis and it scales
/// without limit**, which is what the design asks for — and it is why the
/// *turnaround* floor below is not a second tuned curve but a consequence of
/// this one.
///
/// Throughput divides among the active slips, so a hull of mass `m` occupies a
/// berth for `t_lead + m / (F / slips(F))`. As `F` grows, `slips` grows with it,
/// `F / slips → F_slip` **from above**, and
///
/// ```text
/// t_build → t_lead + m / F_slip        (approached from above, never reached)
/// ```
///
/// **The soft limit is emergent, not imposed.** No amount of industry rushes one
/// hull below that floor; industry buys *more ships at once*, never *faster
/// ships*. Two consequences the design wants on purpose: a General hull stays a
/// long, visible commitment that a rich empire cannot buy its way out of
/// telegraphing (which is what the observation model trades in), and `m` is dry
/// mass, which under R-O57 *is* mineral cost — so the time ladder and the price
/// ladder are one ladder with no second constant to drift.
///
/// **The `1 +` is load-bearing and reads backwards until you take the
/// reciprocal.** It makes per-berth throughput `F/(1 + floor(F/F_slip))`, which
/// is *strictly below* `F_slip` for every finite `F` and rises toward it. Time
/// is the reciprocal, so `t_build` sits strictly **above** the floor and falls
/// toward it — which is the guarantee §3.2 is about. Dropping the `1 +` to
/// "fix" the boundary inverts it: per-berth throughput would then *exceed*
/// `F_slip` and a rich yard would build a single hull faster than the floor,
/// deleting the design property outright. `§3.2`'s prose says `F/slips → F_slip`
/// *from above*, which is the one line in it that is wrong; the operative
/// sentence beside it — "no amount of industry rushes one hull below that
/// floor" — is the claim, and this form is what satisfies it.
fn slips(fabrication_rate: f64, cfg: &SimConfig) -> usize {
    let per_slip = cfg.slip_throughput.max(1e-12);
    1 + (fabrication_rate / per_slip).floor().max(0.0) as usize
}

/// **The knee of the rate curve: one infrastructure rung's worth of works, as
/// one employment's share of it** (T-74).
///
/// `infra_rung_price(1)` is what standing at rung I costs — the same number a
/// Medium hull costs (R-O80) — and a default allocation splits the stock three
/// ways. So a rung-I centre with no works cards played has `u = half` in every
/// employment, sits exactly at half its ceiling, and reproduces the flat
/// constants T-68 and the mining model shipped.
///
/// **That is the anchor and it is the whole calibration.** No number here was
/// fitted to a run; the curve pivots about the configuration that was already
/// ratified, so what T-74 changes is the *shape* around that point rather than
/// the point itself.
fn works_knee(cfg: &SimConfig) -> f64 {
    infra_rung_price(1, cfg).kilotons() / 3.0
}

/// **A planet's rate for one employment, from its infrastructure** (T-74,
/// `Hyades_industry.md` §6.3).
///
/// ```text
/// u_e    = infra · alloc_w[e] / Σ alloc_w      // this employment's share of the stock
/// rate_e = cap_e · u_e / (u_e + half_e)        // saturating, per planet
/// ```
///
/// **A Michaelis–Menten hyperbola, chosen because its two parameters are exactly
/// the two axes the trees are meant to differ on** — which is why one curve can
/// carry the tall/wide distinction instead of it being imposed:
///
/// - `cap_e` is the **asymptote**, the highest rate this planet can ever reach.
///   **Production raises it** — §5.2's "the highest peak per planet".
/// - `half_e` is the **knee**, the stock at which you are halfway there, so the
///   initial slope is `cap_e / half_e`. **Growth and Expansion lower it** —
///   "better efficiency per kilotonne invested".
///
/// A Production world climbs slowly toward a distant ceiling; an Expansion world
/// reaches most of a nearer one almost at once. Neither is a special case.
///
/// **The ceiling is per planet and the empire total is not capped**, which is
/// why Expansion's cheap low-ceiling works are a strategy rather than a
/// handicap, and why slips growing linearly in `F` (§3.2) does not contradict a
/// bounded `F`: the bound is per yard, and an empire has many yards.
///
/// `cap` and `half` from [`cards::Works`] are **multipliers on the base
/// constants**, base `1.0`, so an empire that has played no works card sits
/// exactly on the shipped curve.
fn employment_rate(infra: Price, works: &cards::Works, e: cards::Employment, base_cap: f64, base_half: f64) -> f64 {
    let u = infra.kilotons() * works.alloc_share(e);
    if u <= 0.0 {
        return 0.0;
    }
    let cap = base_cap * works.cap[e.index()];
    let half = (base_half * works.half[e.index()]).max(1e-12);
    cap * u / (u + half)
}

/// **The works bill for a centre's next rung, split by colour** (T-73,
/// `Hyades_industry.md` §5.1/§6.3).
///
/// ```text
/// total   = infra_step_price(stock) / eta_works      // Design, multiplicative
/// bill[c] = total · mix_w[c] / Σ mix_w               // additive-weight share
/// ```
///
/// **This is the mechanism that makes the galaxy's mineral distribution bite on
/// development.** Until now the field bit only on card costs, and T-62 made it
/// log-normal — so which colours a homeworld sits near was an enormous, almost
/// unexpressed fact about a game. A works bill is payable *in named colours*, so
/// a Yellow-poor empire genuinely cannot take the Yellow route however rich it
/// is in total.
///
/// **A mix card can never lower the total** (§5.4/§6.4): `mix_w` is a *share* of
/// a bill only `eta_works` sets, so moving weight between colours moves where
/// the bill lands and nothing else. The orthogonality is structural — not a rule
/// anyone has to remember in review — and `a_mix_card_cannot_change_the_total`
/// asserts it.
///
/// One function, read by both the decision and the build, because the two
/// disagreeing is this repo's recurring failure: `mining_pair_cost` carries a
/// comment begging them to agree, and `settlers_by_hull` exists because they
/// once did not.
fn works_bill(step: Price, works: &cards::Works) -> [Price; 3] {
    let total = step / works.eta_works.max(1e-12);
    let mut bill = [Price::ZERO; 3];
    for (i, &c) in Basic::ALL.iter().enumerate() {
        bill[i] = total * works.mix_share(c);
    }
    bill
}

/// Can this bank pay a colour-split bill? **Every colour, not the total** —
/// which is the whole content of T-73.
fn can_pay_bill(bank: &Minerals, bill: &[Price; 3]) -> bool {
    Basic::ALL.iter().enumerate().all(|(i, &c)| Price::new(bank.get_basic(c)) + Price::new(1e-9) >= bill[i])
}

/// Pay a colour-split bill. Assumes [`can_pay_bill`]; clamps at zero so a
/// rounding crumb cannot drive a colour negative.
fn pay_bill(bank: &mut Minerals, bill: &[Price; 3]) {
    for (i, &c) in Basic::ALL.iter().enumerate() {
        let have = bank.get_basic(c);
        bank.add_basic(c, -(bill[i].kilotons().min(have)));
    }
}

/// The minerals it takes to *stand at* whole infrastructure rung `n`.
#[inline]
fn infra_rung_price(n: usize, cfg: &SimConfig) -> Price {
    Price::new(Price::rung_from(n, cost_anchor(cfg)))
}

/// Minerals to raise infrastructure from the stock `from` to the next whole rung.
///
/// Takes the **stock** rather than a Band since T-70, so the rung is derived
/// here rather than at every call site — one reading, one place it can be taken
/// on the wrong ladder.
fn infra_step_price(from: Price, cfg: &SimConfig) -> Price {
    let at = infra_rung_of(from, cfg);
    infra_rung_price(at + 1, cfg) - infra_rung_price(at, cfg)
}

/// The whole rung an infrastructure stock stands at.
#[inline]
fn infra_rung_of(stock: Price, cfg: &SimConfig) -> usize {
    stock.band_from(cost_anchor(cfg)).round().bands().max(0.0) as usize
}

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
pub fn hull_dry_mass(hull: HullType, cfg: &SimConfig) -> Kilotons {
    Kilotons::new(hull.cost_fraction(cfg) * cfg.general_vehicle_cost)
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
    hull_thrust_to_mass(hull) * cfg.civilian_accel_g * hull_dry_mass(hull, cfg).kilotons()
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
///
/// **Only for the paths where the role still picks the hull** — the paired
/// Freighter, and the Scout. A build that names its hull is priced by
/// [`hull_cost`], because the two stopped agreeing the moment Doctrine could
/// order a heavier colonizer.
fn role_cost(role: Role, cfg: &SimConfig) -> Price {
    hull_cost(role_hull_type(role), cfg)
}

/// Mineral cost of building one hull. Same number as [`hull_dry_mass`] under
/// L6/R-O57 — cost *is* dry mass — but read at the point of purchase rather
/// than the point of flight.
fn hull_cost(hull: HullType, cfg: &SimConfig) -> Price {
    Price::new(hull.cost_fraction(cfg) * cfg.general_vehicle_cost)
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
    population: ComponentStore<Kilotons>,
    planet_id: ComponentStore<PlanetId>,
    homeworld: ComponentStore<Homeworld>,
    archetype: ComponentStore<Archetype>,
    /// **Shipyard occupancy** — present iff this center has a build under way,
    /// holding the clock time it finishes (R-O69).
    ///
    /// The center is the thing that is busy, not the hull: a build ties up the
    /// yard for `build_years` and the next decision is taken when it clears.
    /// Presence is the whole state — the economy tick skips an occupied center,
    /// and [`EventKind::BuildDecision`] removes it on arrival.
    /// **Occupied berths at a centre — completion times, ascending** (T-69,
    /// `Hyades_industry.md` §3.2).
    ///
    /// Was one `f64`: a yard held exactly one build. Slips make concurrency
    /// linear in fabrication throughput, so a rich centre runs several builds
    /// at once and this is the list of when each clears.
    ///
    /// It stays presence-as-state — an empty list means an idle yard — for the
    /// same reason R-O69 needed a store that can vacate.
    berths: ComponentStore<Vec<f64>>,

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
    pop_cargo: ComponentStore<Kilotons>,
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
    /// **The folded works layer, per empire** (`Hyades_industry.md` §6.2, T-73).
    ///
    /// Recomputed from the played multiset in `CardId` order, never accumulated
    /// at play time — see [`cards::Works::fold`] for why that is a desync and
    /// not merely untidy. Identity until a works card exists.
    works: ComponentStore<cards::Works>,
    /// **The multiset the `works` fold is taken over** (T-75b).
    ///
    /// A card play appends `(CardId, WorksWrite)` here and `works` is then
    /// re-derived from the whole list. Keeping the list is what makes the
    /// derivation possible at all: a running product cannot be re-accumulated
    /// in `CardId` order after the fact, because the order it *was* accumulated
    /// in is already baked into its low bits (§6.5).
    works_writes: ComponentStore<Vec<(cards::CardId, cards::WorksWrite)>>,
    /// **The `$` ledger, per empire** (`Hyades_politics_trade_and_intelligence.md`
    /// §2, T-82).
    ///
    /// `$` is a **claim, not a substance** (R-P1, ratified): it has no mass,
    /// occupies no hold, cannot be mined and cannot be shot down. That is what
    /// makes a faucet legal at all — design law #11 conserves mass with no
    /// exclusions, so a `$` that *were* a commodity would make minting illegal
    /// and the economy a closed barter system.
    ///
    /// It is **replicated state**, so design law #16 applies with no softening:
    /// a NaN here is an unreproducible desync. `credit` is the only writer and
    /// it refuses non-finite deltas.
    purse: ComponentStore<f64>,
}

/// **The cross-empire Exchange — one book per basic colour**
/// (`Hyades_politics_trade_and_intelligence.md` §3.1, T-84).
///
/// A `Resource` in the ECS sense, like the event queue: the Exchange never
/// mutates world state, it produces `Fill`s and the caller turns those into
/// events (§10.0). Indexed in `Basic::ALL` order so the set has a canonical
/// order — which §10.5 needs, because per-round clearing over a *set* is what
/// keeps price from being a function of event ordering.
#[derive(Default)]
struct Exchange {
    books: [matching::Book; 3],
    /// **Contracts in flight** — matched, escrowed, not yet settled (T-85).
    ///
    /// §10.2: *"this is the state §3.3 needs and the engine has no analogue
    /// for — it is the first thing in the engine that is **owed** rather than
    /// owned."* A `Vec` appended in fill order, which is deterministic because
    /// `match_wave` is.
    contracts: BTreeMap<u64, Contract>,
    /// Next contract id. Monotonic — an id is never reused, so a settlement
    /// event cannot be delivered to a different contract than the one that
    /// scheduled it.
    next_id: u64,
    /// Contracts settled, and `$` burned to transit, since the run began.
    /// Diagnostic only — §10.8's guard is a census.
    settled: u64,
    defaulted: u64,
    burned: f64,
    /// **Cumulative offers posted per colour**, `(bids, asks)` — the census
    /// §10.8 asks for, and the only way to see the book's *depth* once clearing
    /// drains it. Live depth after a wave is the unmatched remainder, which
    /// answers a different question.
    posted: [(u64, u64); 3],
    /// **Why a fill did not become a contract**, cumulative: `(self-trade, no
    /// shared venue, no price, no purse)`.
    ///
    /// §10.8's census, at the one place a market can silently do nothing. A
    /// book with deep two-sided depth and zero contracts is indistinguishable
    /// from a book nobody posted to unless the *rejections* are counted — which
    /// is `CLAUDE.md` §2's "instrument the decision", and it is how the first
    /// run of this stage was diagnosed instead of guessed at.
    rejected: [u64; 4],
    /// Fills the matcher produced, before any filter.
    fills: u64,
    /// Kilotons actually delivered, per colour — the volume the market moved.
    /// Without it a census can only say trade *happened*, not whether it
    /// happened at a scale that could move anything.
    traded: [f64; 3],
}

/// **A cleared trade, escrowed and awaiting settlement at its venue** (T-85).
///
/// Everything needed to settle without re-deriving it: who owes whom, what, how
/// much `$` is locked, and **where the goods change hands**. The venue is on the
/// contract rather than looked up later because the parties' shared outposts can
/// change between clearing and settlement — a rock mines out, a crew is
/// retasked — and a contract whose venue moved is a contract neither side
/// agreed to.
#[derive(Clone, Copy, Debug)]
struct Contract {
    buyer: PlayerId,
    seller: PlayerId,
    /// **The centre that owes the ore.** Recorded at match rather than looked
    /// up at settlement, because §3.3's default case turns on *this* bank being
    /// short — re-deriving "some centre of the seller's" at settlement would
    /// make a default impossible to express.
    seller_centre: Entity,
    colour: Basic,
    /// Kilotons of ore the seller owes.
    qty: f64,
    /// `$` locked from the buyer's purse at match (§2.3's `E`).
    escrow: f64,
    /// **Where the seller leaves the ore** (§10.6). A rock both parties work —
    /// **not the buyer's world**, because a foreign hull near a colony is a
    /// card and not the default.
    seller_drop: Entity,
    /// **Where the buyer leaves its side, when its side is goods.**
    ///
    /// A contract has **two locations, not one** (author's ruling): *leave
    /// Yellow at X in exchange for Magenta at Y*. The two legs need not meet at
    /// the same rock, and each is chosen by **the party making that delivery**,
    /// minimising *its own* transit — which supersedes R-P17's symmetric
    /// "minimum summed transit". A shipper pays for its own leg, so a shipper
    /// picks its own drop; a single compromise venue would make each side pay
    /// for the other's geography, and §7.1's default transaction is *balanced*.
    ///
    /// `None` when the buyer settles in `$`, which has no location at all.
    buyer_drop: Option<Entity>,
    /// When the contract was struck — the burn is measured from here to
    /// settlement.
    ///
    /// **There is deliberately no `deliver_at`.** The scheduled `ContractDue`
    /// event *is* the delivery time, and storing it twice would be two facts
    /// that must agree with nothing checking that they do — the same defect
    /// `mining_pair_cost` needed a comment to guard against and
    /// `Candidate::settlers_by_hull` exists to avoid.
    ///
    /// **The obligation is struck instantly; only the goods are light-lagged**
    /// (author's ruling: *debt can travel faster than light*). That is not an
    /// exception to design law #15 — it is R-P1 being taken seriously. `$` and
    /// the claim it denominates are **not substances**: they have no mass,
    /// occupy no hold and cross no distance, so there is nothing for light-lag
    /// to bind. And the contract is struck at the **round barrier**, which is
    /// the protocol clock's synchronisation point (§10.5) rather than an in-world
    /// observation — the same moment cards resolve and the `Works` fold is
    /// recomputed.
    ///
    /// The asymmetry is the design: **the ledger is instant and the freight is
    /// not.** A deal can be agreed across the theatre in a round while the ore
    /// it commits takes decades to arrive, and everything that can happen to
    /// that ore on the way (§8.1 — attack, diversion, theft, blockade) is the
    /// gap between the two.
    struck: f64,
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
            berths: ComponentStore::new(),
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
            works: ComponentStore::new(),
            works_writes: ComponentStore::new(),
            purse: ComponentStore::new(),
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
    /// A production center's **economy** step: mine + grow. Cadence-driven, one
    /// per center per `cycle_years`, because both halves are *rates over an
    /// interval* and an interval is what they need.
    ///
    /// It no longer carries the build decision (R-O69). It still *takes* one
    /// when the yard is free, because an empty yard has no completion event to
    /// wake it — mining is what changes a saving center's situation, so the
    /// mining step is the right place to reconsider.
    ProductionTick { center: Entity },
    /// **A production center decides what to build** — the event that replaced
    /// the cadence (R-O69).
    ///
    /// Fires when the yard clears: `build_years` after a build was committed.
    /// The decision is the moment the center's situation actually changed, which
    /// is the §4 rule the production tick was the last exception to.
    ///
    /// The other trigger the design calls for — **a build interrupted by
    /// hostiles** — has nothing to raise it yet: no combat is wired into the
    /// simulation loop (`combat::resolve_engagement` is never called from
    /// `sim.rs`). When it is, interruption is *this* event scheduled at the
    /// moment of the strike, after clearing `building_until`; the seam is here
    /// so that is a scheduling call and not a redesign. **T-52.**
    BuildDecision { center: Entity },
    /// **A contract's freight leg completes at its drop** (T-77). Carries the
    /// contract id rather than an index, because ids are never reused and
    /// indices shift when a contract settles.
    ContractDue { id: u64 },
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
    /// **Irreducible per-hull lead time, `t_lead`** (`Hyades_industry.md` §3.2,
    /// T-68) — tooling and crew, the part of a build that does not scale with
    /// industry.
    ///
    /// This replaced a flat `build_years = 10.0`, under which a Limited hull and
    /// a General hull took the same ten years despite a **50x** mass ratio. Time
    /// now tracks mass, and because dry mass *is* mineral cost (R-O57) the time
    /// ladder and the price ladder are **one ladder** with no second constant to
    /// tune and no way for them to drift apart.
    ///
    /// **Approved starting value, not MC-ratified** (§3.3).
    pub build_lead_years: f64,
    /// **One slip's fabrication throughput, `F_slip`, in kt/yr** (§3.2, T-68).
    ///
    /// Stage 2 is the single-slip case, so this is simply the rate at which a
    /// yard turns mass into hull: `t_build = t_lead + m / F_slip`. Concurrency
    /// — `slips(F) = 1 + floor(F / F_slip)`, and with it the *soft floor* that
    /// makes this an asymptote rather than a divisor — is stage 3 (T-69). The
    /// constant is introduced now with the meaning it will keep, so that landing
    /// slips changes what divides by it and not what it means.
    ///
    /// **Approved starting value, not MC-ratified** (§3.3).
    pub slip_throughput: f64,
    /// **Fabrication ceiling, kt/yr** — the asymptote `cap_fab` a single planet
    /// can ever reach (T-74, `Hyades_industry.md` §6.3). Production raises it.
    ///
    /// **Anchored, not fitted.** The knee sits at one infrastructure rung
    /// ([`works_knee`]), so a rung-I centre under default doctrine runs at
    /// exactly *half* its ceiling — and this is `2 × slip_throughput`, which
    /// makes that centre's rate identical to the flat constant T-68 shipped.
    /// Below a rung it is slower, above it faster: the ramp, switched on,
    /// pivoting about the configuration that was already ratified.
    ///
    /// **Placeholder magnitude** (R-IND3), like every coefficient in §5.
    pub fab_cap: f64,
    pub civilian_accel_g: f64,
    pub colony_seed_pop: BandTier,
    pub max_survey_hops: usize,

    /// Minimum development level to build medium vehicles (colony/mining).
    ///
    /// **Ratified at `Band II`, down from `Band III` (R-O80, T-61).** It had to
    /// move with the infrastructure ladder. `K = min(hab, bio_max, infra)` caps
    /// population, so this is really a gate on *infrastructure*, and the
    /// corrected ladder charges **20 minerals cumulative to reach `Band III`
    /// against 6 under the old linear one**. An expansion gate cannot sit one
    /// rung above a step that costs twenty times the last one: at `Band III`
    /// the standard bed produced **zero colonies in 4,000 years** on both
    /// seeds, homeworlds stalling at `Band II` and never affording a colonizer.
    ///
    /// At `Band II` the same bed gives 10,105,286 and 10,037,745 colony-years
    /// on seeds 1 and 7 — the best figures this project has recorded, +15% on
    /// the subsidised best, with the first colony founded at 65 yr against 185.
    ///
    /// The production schedule's "2 = limited, 3 = medium, 4 = all" is retired
    /// with it: that schedule was written against a linear price ladder and
    /// does not survive the ladder being multiplicative.
    pub medium_min_level: BandTier,
    /// Minimum development level to build *limited* vehicles — the Scout/LCV
    /// survey craft. The same production schedule that puts medium at
    /// `Band III` puts limited at **2** (`Band II`); before this field existed
    /// the autopilot returned `Idle` for everything below `medium_min_level`,
    /// so the limited tier was unreachable and a `Band II` center could do
    /// nothing but hoard minerals.
    pub limited_min_level: BandTier,
    /// Mineral cost of one General Systems Vehicle — "1 CMY mineral = 1 fleet"
    /// (`Hyades_vehicle_roles.md` §6, confirmed this conversation, revising
    /// the earlier "3 CMY = 3 fleets" anchor to the same ratio in cleaner
    /// units).
    pub general_vehicle_cost: f64,
    /// How many Medium Systems Vehicles one `general_vehicle_cost` buys —
    /// "a medium fleet... starting guess: 3" (§6) — **superseded twice.**
    ///
    /// **Ratified at 10 as the `Band I → II` step of the cost ladder**
    /// (`Hyades_mineral_cost_curve.md` §2.6, R-MC15), replacing the MC-tuned
    /// 4.45. The ladder is now `1 : 10 : 50`, and it is no longer a free
    /// parameter: it *is* a Band step, so it is bound by the ratified growth
    /// rule `1 < F₍ₙ₊₁₎/Fₙ < 10` and by `F_mass = F_cost^(3/2)`.
    ///
    /// **Measured before adopting** (`examples/hull_ladder`, the standard
    /// four-seed CRN bed, 4,000 yr), because T-56's acceptance test is that
    /// making General hulls *relatively more expensive* must not slow
    /// colonisation down:
    ///
    /// | | colony-years | vs shipped | doubling |
    /// |---|---|---|---|
    /// | shipped (4.45 / 9.0) | 8,011,139 | — | 284.5 yr |
    /// | ratified (10 / 50) | 8,697,322 | **+8.6%** | **265.0 yr** |
    ///
    /// Every seed improved (+8.0, +2.9, +13.9, +10.0) and the doubling time
    /// fell by 19.5 years. **The gain is the price, not the hold** — and that
    /// needed an ablation, because `hull_radius` is `sqrt(cost ratio)`, so this
    /// field also moves the Medium hold 0.96 → 24.07 kt. Re-run with
    /// `cargo_unit_size` rescaled to hold the Medium hull's capacity at its
    /// shipped 0.959 kt, the ladder still gives **+7.6%** and 269.3 yr. So
    /// roughly seven of the eight points are the cost change.
    ///
    /// The old MC ratification is superseded, not overturned: its +32.7 ± 3.6
    /// elasticity was measured on the *coverage* objective at a lower operating
    /// point, and it pointed the same way this ladder went.
    ///
    /// This discharges design law #6 for the ladder: 1 : 3 : 9 was scaffolding
    /// to be replaced rather than a target, and it has been.
    pub medium_fleet_size: f64,
    /// How many Limited Systems Vehicles one `general_vehicle_cost` buys —
    /// "a large fleet... starting guess: 9" (§6).
    ///
    /// **Ratified at 50** — `5 × medium_fleet_size`, the `Empty → I` step of
    /// the cost ladder (§2.6, R-MC15). Not independently tunable any more: with
    /// `medium_fleet_size` it forms the two lowest rungs of a Band ladder, and
    /// their ratio is what the growth rule constrains.
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
    /// **Re-task a mining pair when its rock runs dry**, instead of leaving the
    /// miner and its freighter parked on a dead world for the rest of the match.
    ///
    /// A pair is built for one rock: `Shuttle { outpost, .. }` fixes the pickup
    /// leg at spawn and only the *delivery* leg is need-routed, so exhaustion
    /// used to end both hulls' working lives. Measured on seed 1 at the shipped
    /// defaults (`examples/mining_probe -- census`): **2,029 of 2,188 outposts
    /// mine out**, a rock lasts a mean of **808 years**, and **39% of all
    /// outpost-years are spent on an exhausted one** — a miner and a freighter
    /// each, paid for and idle.
    ///
    /// With this on, an exhausted pair goes to Reserve (roles §4.6 — standing
    /// and re-taskable, never auto-scrapped) and the next center that wants a
    /// mining pair takes the reserved hulls nearest its target instead of
    /// buying new ones: no mineral cost, no build delay, only the flight.
    pub recycle_mining_pairs: bool,
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
    /// **The crowding exponent β** (`Hyades_industry.md` §4.3, T-71).
    ///
    /// Work at a site is `W(n, S) = N(S)^(1−β) · n^β`, so β = 1 is no crowding
    /// at all and β = 0 makes every extra unit of capacity worthless.
    /// **Placeholder `1/2`** — §4.3's own value, never measured (R-IND18).
    pub crowding_beta: f64,
    /// **Workable veins per Band of deposit mass** (`Hyades_industry.md` §4.2,
    /// T-71) — `N(S) = veins_per_band^(band(S) − 1)`.
    ///
    /// This is the term whose absence would have killed the mining ramp. A bare
    /// `n^β` crowds a `Band I` pebble and a `Band IV` seam identically, so
    /// nobody would ever put a large crew anywhere; the design wants hundreds or
    /// thousands of miners on a high-value outpost. **Placeholder `10`** — "a
    /// decade per Band", the design's own language for an order of magnitude
    /// (R-IND18).
    pub veins_per_band: f64,
    pub mining_tick_years: f64,
    /// Density below which a body is considered mined out.
    pub density_floor: f64,
    /// The **reference hold**, in kilotons — the scale factor on
    /// [`HullType::cargo_capacity`]'s geometric ladder.
    ///
    /// Every other hull's capacity is this times its usable-volume ratio to
    /// [`REFERENCE_MEDIUM_RADIUS`] (R-O58), so a Limited hull carries nothing
    /// and a General one carries far more than twice as much.
    ///
    /// ~~What a Medium hull carries~~ — **it is the hold of a hull at the
    /// reference radius √3, which the Medium hull only has when
    /// `medium_fleet_size == 3`.** The normaliser is a constant on purpose (see
    /// [`REFERENCE_MEDIUM_RADIUS`]); it is this doc line that was stale.
    ///
    /// **Ratified at 1.0 — `KT(I)`, one kiloton** (§2.6, R-MC15), where the
    /// spec puts `Band I` for every quantity on the mass ladder. At the
    /// ratified cost ladder the Medium radius is `√5` and a Medium hull carries
    /// **4.81 kt**, so this field and the hull's real hold still differ by the
    /// normaliser; making them the same number needs the per-`(role, size)`
    /// thickness and `η` of §2.3 (T-56 stage 3c).
    ///
    /// **Adopting it was free, and provably so.** On top of the ratified cost
    /// ladder, moving this 5.0 → 1.0 cuts every hold five-fold — Medium
    /// 24.07 → 4.81 kt, General 2,852 → 570 kt — and reproduces the four-seed
    /// bed **bit-identically** (`examples/hull_ladder`). Both values sit above
    /// the binding threshold the table below already records.
    ///
    /// **That matters for the table, which is why it is corrected here.** Its
    /// x-axis is this field, not the hold any hull actually has, and since
    /// R-O58 the cost ladder *is* the capacity ladder — so `medium_fleet_size`
    /// silently rescales what every row means. The table was taken at
    /// `medium_fleet_size = 4.45`, where "past roughly 1–5" meant *past roughly
    /// 0.19–0.96 kt of real Medium hold*. At the ratified ladder this field's
    /// 1.0 is a 4.81 kt hold, already clear of it. CLAUDE.md §2: a parameter
    /// that reaches the objective through a derived quantity cannot be swept
    /// alone.
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
    /// **The `$` faucet rate** — `$` minted per kilotonne of fabrication capacity
    /// per year (`Hyades_politics_trade_and_intelligence.md` §2.3, T-82).
    ///
    /// `$_income = base · production`, with **production the works fabrication
    /// rate**, which is what R-P3 ratified: income tracks what an empire can
    /// *make*, not how many people it has, so the biggest empire does not also
    /// automatically hold the deepest purse.
    ///
    /// **R-P16 is resolved rather than shipped.** §10.3 decided to ship this
    /// against infrastructure stock as an explicit placeholder *because T-74 had
    /// not landed and there was no fabrication rate to read*. T-74 has landed,
    /// so the faucet reads the real quantity and the placeholder is not needed.
    ///
    /// **Placeholder magnitude** (R-P2) — nothing spends `$` yet, so no
    /// measurement can price it.
    pub dollar_per_fabrication: f64,

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

/// Default for [`SimConfig::recycle_mining_pairs`]. Kept as a named constant
/// rather than a literal because it is the switch an A/B measurement flips —
/// `examples/mining_probe -- recycle` compares both settings on the same seeds.
///
/// **`true`, ratified on request** ("use the mining outpost strategy that leads
/// to the greatest amount of colonies"). Paired on the **8-seed** bed:
/// **+1.69 ± 0.53 points of colonies** (3.2 SE), 47.67% → 49.36%, per-seed
/// [+0.5, +0.1, +3.0, +1.2, +1.5, +3.7, +0.0, +3.5] — up on every seed, down on
/// none. The standard four could not resolve it (+1.19 ± 0.64, 1.9 SE) because
/// seed 42 alone swings +3.0 points.
///
/// It is by a wide margin the largest effect available on this surface, which
/// says more about the surface than about the effect: five of the six mining
/// *knobs* cannot be told from noise at all
/// (`Hyades_autopilot_colonization_growth.md` §5b). Tuning was exhausted; this
/// is a term.
pub const RECYCLE_MINING_PAIRS_DEFAULT: bool = true;

impl SimConfig {
    /// **Is the derived hull ladder usable?** `None` if fine, `Some(reason)` if
    /// the configuration produces a degenerate one.
    ///
    /// **T-56 stage 3c untied the two ladders**, and this check changed with
    /// them. Radius used to be `sqrt(cost / cost_Limited)` with capacity
    /// `(r − 1)³` against a fixed reference, so `medium_fleet_size` was
    /// simultaneously the price and the hold. Cost is now the shell volume and
    /// capacity the hold volume — two functions of one body, separated by a
    /// ratified per-hull `τ` — so a price change no longer silently rescales
    /// every cargo bay.
    ///
    /// **Two failures, and they are different in kind.**
    ///
    /// - `medium_fleet_size ≥ limited_fleet_size` is a **naming contradiction**:
    ///   a hull the taxonomy calls bigger would cost no more, and (via the
    ///   solve) be no larger. Refused.
    /// - A hull priced below its own skin — `4·cost·η < τ³` — is a **geometric
    ///   impossibility**: the quadratic in `hull_radius` has no real root, and
    ///   design law #16 makes the resulting NaN a fatal error rather than a
    ///   value. Refused here, at construction, so it can never reach hashed
    ///   state. At the ratified ladder the tightest margin is the Limited
    ///   Offensive hull's, 24× clear.
    ///
    /// **There is no "too close" soft bound**, and the reason is worth keeping.
    /// An earlier version refused `r_M < 1.25` because the General : Medium
    /// capacity ratio blew up there — an artifact of normalising capacity
    /// against the *live* Medium radius, i.e. dividing by a quantity that goes
    /// to zero. There is no normaliser at all now. A narrow ladder just means
    /// the Medium hull is nearly all shell and hauls nearly nothing, which is a
    /// real economic consequence rather than a modelling failure.
    pub fn hull_ladder_fault(&self) -> Option<&'static str> {
        if self.medium_fleet_size >= self.limited_fleet_size {
            return Some(
                "medium_fleet_size >= limited_fleet_size: the Medium hull is no larger than the \
                 Limited one, so a hull the taxonomy calls bigger is in fact smaller",
            );
        }
        for hull in HullType::ALL {
            let g = hull.geometry();
            let tau = g.shell_thickness.hull_units();
            let material = hull.cost_fraction(self) * self.general_vehicle_cost * g.shape_efficiency;
            // The discriminant of the radius solve, checked directly: NaN first,
            // because a NaN comparison is false either way round and design law
            // #16 makes it fatal rather than merely wrong.
            let disc = 12.0 * tau * material - 3.0 * tau * tau * tau * tau;
            if disc.is_nan() || disc < 0.0 || !hull.hull_radius(self).is_finite() {
                return Some(
                    "a hull is priced below its own shell (4·cost·η < τ³): the radius solve has no \
                     real root, which would put a NaN in replicated state (design law #16)",
                );
            }
        }
        None
    }

    pub fn new(seed: u64) -> Self {
        SimConfig {
            horizon_years: 4000.0,
            cycle_years: 50.0,
            build_lead_years: 2.0,
            slip_throughput: 0.1,
            fab_cap: 0.2,
            civilian_accel_g: 1.0,
            // "requires 1 pop as cargo to start a new colony" — confirmed,
            // not a placeholder (`Hyades_vehicle_roles.md` §4.2/R-V9).
            colony_seed_pop: BandTier::I,
            // 120 — ratified with the snowball defaults: a 40-hop chain retired
            // scouts while most of the galaxy was still dark.
            max_survey_hops: 120,
            medium_min_level: BandTier::II,
            limited_min_level: BandTier::II,
            general_vehicle_cost: 1.0,
            medium_fleet_size: 10.0,
            limited_fleet_size: 50.0,
            homeworld_start_minerals: 3.0,
            enforce_roster: false,
            recycle_mining_pairs: RECYCLE_MINING_PAIRS_DEFAULT,
            center_mining_fraction: 0.15,
            biosphere_regen_rate: 0.127,
            outpost_mining_fraction: 0.238,
            crowding_beta: 0.5,
            veins_per_band: 10.0,
            mining_tick_years: 50.0,
            density_floor: 0.01,
            cargo_unit_size: 1.0,
            trade_decay_lambda: 0.01,
            dollar_per_fabrication: 1.0,
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
    /// The empire's people, **as a mass** — see
    /// [`crate::snapshot::PlayerSnapshot::total_population`] for why a sum of
    /// Bands would not mean anything.
    pub total_population: Kilotons,
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
    /// Memoised holdings centroid per seat; `None` means "recompute".
    /// Invalidated only by [`Simulation::claim_planet`] — see
    /// [`Simulation::holdings_centroid`] for why this is a memo and not a
    /// running sum.
    centroid_cache: Vec<Option<Vec3>>,

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
    /// Which miners are working which outpost, so exhaustion can find the hulls
    /// it stranded — and so extraction knows how many hands are on the rock
    /// (T-57). Keyed by outpost entity index; `BTreeMap` for deterministic
    /// iteration, like every other collection in here, and the crew is a `Vec`
    /// in arrival order for the same reason.
    ///
    /// **It was one `Entity` until T-57.** A mining outpost could only ever have
    /// a single operator *by data structure*, which is why "how many miners per
    /// outpost" was not a value anyone could tune — there was no term in the
    /// model for it.
    mine_crew: BTreeMap<(u32, u64), Vec<Entity>>,
    /// Ore an outpost has extracted **for one player**, awaiting a freighter.
    ///
    /// Keyed by `(player, outpost)`, and that is the correction: the rock's
    /// `stockpile` component is one pile per *planet*, so two empires working
    /// the same body were filling and drawing from the same heap — either could
    /// haul away what the other's miners dug. Outposts are never claimed
    /// (`claim_planet` is for colonies), so nothing stopped it.
    ///
    /// **The body is shared; the pile is not.** `density` stays per-planet
    /// because a rock is one physical object and every crew on it depletes the
    /// same ore — contesting a field is a real mechanic, and a card that lets
    /// an empire draw from a rival's pile is a *card*, not the default.
    outpost_stock: BTreeMap<(u32, u64), Minerals>,
    /// **The cross-empire Exchange** (T-84). Rebuilt and cleared at the round
    /// barrier (§10.5), never continuously — a continuous book makes price a
    /// function of event ordering, and two clients that tie-break a match
    /// differently clear at different prices, which is an unreproducible desync.
    exchange: Exchange,
    /// Whether the round barrier posts to the Exchange at all (T-84).
    ///
    /// Exists for the inertness ablation and nothing else: `CLAUDE.md` §2 puts
    /// ablation first among the three kinds of proof, and "posting changes
    /// nothing" is only checkable against a run that did not post.
    exchange_posting: bool,
    /// Whether a struck contract is ever scheduled to settle (T-77).
    ///
    /// Separate from `exchange_posting` so the two stages stay separable: with
    /// posting on and this off, contracts are struck and `$` is escrowed and
    /// **no kilotonne moves**, which is what makes §10.7's stage-4 inertness a
    /// checkable claim rather than one T-77 silently invalidated. It is also
    /// the ablation for attributing T-77's own effect.
    exchange_settlement: bool,
    /// Per-player pools of hulls whose rock ran dry, awaiting re-tasking. Push
    /// order is event order, so these are deterministic; selection is by
    /// distance to the new target, not by position in the pool.
    reserve_miners: Vec<Vec<Entity>>,
    reserve_freighters: Vec<Vec<Entity>>,
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
                Factors::new(
                    pl.habitability,
                    // **Every world is at its pristine ceiling at t=0** — one
                    // generated value, read as a mass, used for both the stock
                    // and the ceiling. There is no design reason for a world to
                    // start below its own ceiling, and no generator spread to
                    // reconcile: `biomass == bio_max` here holds by
                    // construction rather than by convention.
                    pl.biosphere.in_kilotons(),
                    pl.biosphere.in_kilotons(),
                    // Galaxy generation states the starting rung; the engine stores
                    // the stock that rung costs (T-70).
                    infra_rung_price(pl.infrastructure.round().bands().max(0.0) as usize, &config),
                ),
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
            centroid_cache: vec![None; n],
            autopilots,
            queue: BinaryHeap::new(),
            rng: Rng::new(config.seed),
            clock: 0.0,
            seq: 0,
            events_processed: 0,
            active_mines: BTreeSet::new(),
            mine_crew: BTreeMap::new(),
            outpost_stock: BTreeMap::new(),
            exchange: Exchange::default(),
            exchange_posting: true,
            exchange_settlement: true,
            reserve_miners: vec![Vec::new(); n],
            reserve_freighters: vec![Vec::new(); n],
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
            self.world.works.insert(pe, cards::Works::default());
            self.world.works_writes.insert(pe, Vec::new());
            self.world.purse.insert(pe, 0.0);

            // Seed the homeworld's stockpile so it can begin deepening infra.
            let seed = self.config.homeworld_start_minerals / 3.0;
            let s = self.world.stockpile.get_mut(home).unwrap();
            s.cyan += seed;
            s.magenta += seed;
            s.yellow += seed;

            self.schedule(self.config.cycle_years, EventKind::ProductionTick { center: home });

            // Opening survey fan-out: contact craft, one per cube-face heading,
            // built free as starting units (§2). Under `SurveyStrategy::GlobalPool`
            // (R-AC3) even the opener gets no heading — every craft, including
            // the fan-out, picks the globally nearest unscanned world.
            let home_pos = *self.world.position.get(home).unwrap();
            let doctrine_here = *self.world.doctrine.get(pe).unwrap();
            let vehicles = doctrine_here.survey_vehicles;
            for i in 0..vehicles {
                let heading = match doctrine_here.survey_strategy {
                    SurveyStrategy::GlobalPool => Vec3::ZERO,
                    SurveyStrategy::OpeningSectors | SurveyStrategy::PersistentSectors => Vec3::CUBE_FACES[i % 6],
                };
                // Bootstrap craft are *seeded*, not built — no yard made them,
                // so they leave at once (autopilot-doc §2).
                self.launch_survey(p, home_pos, heading, 0, 0.0);
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

        // **The Exchange clears at the barrier, not continuously** (§10.5).
        // Cards have just resolved and the `Works` fold has been recomputed, so
        // this is the moment the standing layer is coherent — and a round is a
        // *set*, which is what gives the book a canonical order. Posting is
        // inert until T-85 wires clearing.
        if self.exchange_posting {
            self.post_exchange_offers();
            self.clear_exchange();
        }

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
            let cost = Price::new(o.card.and_then(cards::card).map(|c| c.cost).unwrap_or(0.0));
            let affordable = cost <= Price::ZERO || self.empire_can_afford(p, cost);
            let Some(id) = o.coerce(affordable).card else { continue };
            let Some(c) = cards::card(id) else { continue };
            if c.cost > 0.0 {
                self.empire_spend(p, Price::new(c.cost));
            }
            self.apply_card_effect(p, c, o.target, round);
        }
    }

    /// Total basic minerals across an empire's holdings. Cards are paid from
    /// the empire, not from one center — design law #7 puts cards at
    /// empire/macro scale, so a per-center purse would be the wrong grain.
    fn empire_can_afford(&self, p: usize, cost: Price) -> bool {
        let me = PlayerId(p as u32);
        let mut total = Price::ZERO;
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
    fn empire_spend(&mut self, p: usize, cost: Price) {
        let me = PlayerId(p as u32);
        let mut holdings: Vec<(Entity, Price)> = self
            .planet_entity
            .iter()
            .filter(|&&e| self.world.owner.get(e).copied() == Some(me))
            .filter_map(|&e| self.world.stockpile.get(e).map(|s| (e, s.basic_total())))
            .filter(|&(_, t)| t > Price::ZERO)
            .collect();
        // Richest first; entity id breaks ties so the order is total.
        holdings.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal).then(a.0 .0.cmp(&b.0 .0)));
        let mut remaining = cost;
        for (e, avail) in holdings {
            if remaining <= Price::ZERO {
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
            CardEffect::WriteWorks(w) => {
                // **Record, then re-derive** (T-75b, §6.5). The list is the
                // state; `Works` is a cache of the fold over it. Doing it the
                // other way — multiplying the coefficient in place — would make
                // the result depend on play order in its last bits, which is a
                // desync and not a rounding difference.
                let pe = self.player_entity[p];
                self.world.works_writes.get_mut(pe).unwrap().push((c.id, w));
                let folded = cards::Works::fold(self.world.works_writes.get(pe).unwrap());
                *self.world.works.get_mut(pe).unwrap() = folded;
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
            EventKind::BuildDecision { center } => self.sys_build_decision(center),
            EventKind::ContractDue { id } => self.sys_contract_due(id),
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
            self.claim_planet(target, owner);
            {
                // The hull is what is being recycled, so read it off the ship
                // that actually arrived. Defaulting matches `sys_freighter_arrive`:
                // a courier constructed without one is a test fixture, not a
                // build, and the Medium hull is the baseline colonizer.
                let hull = self.world.hull_type.get(vehicle).copied().unwrap_or(HullType::MediumSystems);
                let founded_at = self.founding_infra(hull);
                let f = self.world.factors.get_mut(target).unwrap();
                f.infra = f.infra.max(founded_at);
            }
            // Seed population from the pop *carried as cargo*
            // (`Hyades_vehicle_roles.md` §4.2/R-V9 — confirmed, not a flat
            // constant applied on arrival regardless of what was brought).
            //
            // **Credited, not assigned** (R-O74). These people were debited from
            // the founding centre at launch, so they are *added* to whatever the
            // world holds rather than `max`'d into place. An unowned world holds
            // zero (`galaxy.rs`), so the two agreed until settlers had to come
            // from somewhere — at which point `max` would have destroyed the
            // difference silently.
            let carried_pop = self.world.pop_cargo.get(vehicle).copied().unwrap_or(Kilotons::ZERO);
            {
                let pop = self.world.population.get_mut(target).unwrap();
                *pop += carried_pop;
            }
            self.world.pop_cargo.insert(vehicle, Kilotons::ZERO);
            // **And the rest of the hold lands with them.** The old rule here
            // was "no mineral seed for colonies, homeworlds only" — superseded:
            // a hold may carry any mix of settlers and minerals, and the
            // minerals jumpstart production (`Hyades_industry.md` §1.7). This
            // is not a new grant, it is the endowment the founding centre
            // already paid for out of its own bank at launch.
            let endowment = self.world.cargo.get(vehicle).copied().unwrap_or_default();
            if endowment.basic_total() > Price::ZERO {
                if let Some(bank) = self.world.stockpile.get_mut(target) {
                    bank.add_basics(&endowment);
                }
                self.world.cargo.insert(vehicle, Minerals::default());
            }
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
        self.mine_crew.entry((p as u32, outpost.0)).or_default().push(vehicle);
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
            // **Its owner's pile, not the rock's.** Outposts are never claimed,
            // so a per-planet heap let either empire haul away what the other's
            // miners dug.
            let stock = self.outpost_stock.entry((p, sh.outpost.0)).or_default();
            // A hold is a mass and a stockpile is a price; the same kilotons,
            // two ladders (R-O57). `on_scale` is the crossing, said out loud.
            let avail = stock.basic_total();
            let load = cap.on_scale::<units::Cost>().min(avail);
            if load > Price::ZERO {
                let moved = take_basics(self.outpost_stock.get_mut(&(p, sh.outpost.0)).unwrap(), load);
                self.world.cargo.get_mut(vehicle).unwrap().add_basics(&moved);
                let outpost_pid = *self.world.planet_id.get(sh.outpost).unwrap();
                self.log.push(
                    self.clock,
                    LogEvent::FreighterTransfer {
                        player: p,
                        vehicle,
                        leg: FreighterLeg::Loaded,
                        amount: load.kilotons(),
                        at: outpost_pid,
                    },
                );
            }
            // Stop shuttling once the outpost is exhausted and empty.
            //
            // **The predicate has to be the miner's, not a stricter one.**
            // `sys_mining_tick` stops when the *yield* falls to the floor
            // (`density × outpost_mining_fraction <= density_floor`), so at the
            // shipped values a rock stops producing at metallicity 0.042 — while
            // this branch used to wait for 0.01. In that band the mine is dead
            // and the freighter does not know it, so it shuttles empty round
            // trips for the rest of the match: measured on seed 1, **not one
            // freighter of 2,655 ever reached this branch.** Same test, same
            // verdict, and the hull becomes re-taskable when its rock dies.
            let dens = self.world.density.get(sh.outpost).unwrap().total_mass().kilotons();
            if load <= Price::new(1e-9) && dens * self.config.outpost_mining_fraction <= self.config.density_floor {
                let here = self.position_at(sh.outpost, self.clock).unwrap();
                let outpost_pid = *self.world.planet_id.get(sh.outpost).unwrap();
                self.park(vehicle, here);
                if self.config.recycle_mining_pairs {
                    self.release_to_reserve(vehicle, Role::Freighter, outpost_pid);
                } else {
                    self.log.push(
                        self.clock,
                        LogEvent::VehicleParked { player: p, vehicle, role: Role::Freighter, at: outpost_pid },
                    );
                }
                return;
            }
            // Route to whichever owned production center offers the best
            // *discounted* need — confirmed: "autopilot must haul minerals to
            // where they are needed," not back to one hardcoded partner, and
            // (R-P2) not across the galaxy to a marginally needier one either.
            // At `trade_decay_lambda = 0` this reduces exactly to
            // `most_needed_center`; the shipped value is 0.01, so the discounted
            // path is live. Since T-81 the need term is **per colour** and reads
            // the cargo actually aboard, so a hauler carrying Yellow goes where
            // Yellow is what is missing. Falls back to the outpost's own paired
            // home center only if this owner holds no production center at all
            // (shouldn't happen; the homeworld always counts).
            let home = *self.world.home_center.get(vehicle).unwrap_or(&sh.outpost);
            let here = self.position_at(sh.outpost, self.clock).unwrap();
            let cargo = self.world.cargo.get(vehicle).copied().unwrap_or_default();
            let dest = self.best_delivery_center(PlayerId(p), here, &cargo).unwrap_or(home);
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
            if cargo.basic_total() > Price::ZERO {
                let dest_pid = *self.world.planet_id.get(sh.destination).unwrap();
                self.log.push(
                    self.clock,
                    LogEvent::FreighterTransfer {
                        player: p,
                        vehicle,
                        leg: FreighterLeg::Deposited,
                        amount: cargo.basic_total().kilotons(),
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
        // **Unload before parking** (R-O74). A bounced Colonizer is carrying
        // people and minerals that were debited from its home centre, and this
        // is where "nothing is lost" stops being a comment and becomes an
        // entry: park it still laden and the endowment sits in a hold forever,
        // which is a slow leak rather than an obvious one.
        if let Some(&home) = self.world.home_center.get(vehicle) {
            let pop = self.world.pop_cargo.get(vehicle).copied().unwrap_or(Kilotons::ZERO);
            if pop > Kilotons::ZERO && self.world.population.contains(home) {
                let at_home = self.world.population.get_mut(home).unwrap();
                *at_home += pop;
                self.world.pop_cargo.insert(vehicle, Kilotons::ZERO);
            }
            let cargo = self.world.cargo.get(vehicle).copied().unwrap_or_default();
            if cargo.basic_total() > Price::ZERO && self.world.stockpile.contains(home) {
                self.world.stockpile.get_mut(home).unwrap().add_basics(&cargo);
                self.world.cargo.insert(vehicle, Minerals::default());
            }
        }
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
            let colors = recovered.kilotons() / 3.0;
            let stock = self.world.stockpile.get_mut(dest_e).unwrap();
            stock.cyan += colors;
            stock.magenta += colors;
            stock.yellow += colors;
            if let Some(o) = owner {
                let pid = *self.world.planet_id.get(dest_e).unwrap();
                self.log.push(
                    self.clock,
                    LogEvent::VehicleScrapped { player: o.0, vehicle, at: pid, recovered: recovered.kilotons() },
                );
            }
        }
    }

    fn sys_mining_tick(&mut self, outpost: Entity) {
        let pid = *self.world.planet_id.get(outpost).unwrap();
        // **Extraction is per-miner now, not per-rock (T-57).**
        //
        // `outpost_mining_fraction` was the fraction of remaining density a
        // *rock* yielded per tick, with the miner standing on it contributing
        // nothing but the schedule: a second hull would have extracted no more,
        // and a better one no more either. It is now the fraction **one miner**
        // works, and a crew of `n` works `n` times as much — capped at the whole
        // remaining field, because a rock cannot yield more than it holds.
        //
        // A crew of one is arithmetically identical to the old expression,
        // which is what keeps the shipped configuration bit-identical.
        //
        // **Each player's crew works the shared rock into that player's own
        // pile.** The body is one physical object and every crew on it depletes
        // the same ore — so a contested field runs out sooner for everybody,
        // which is the mechanic — but the ore each empire lifts is its own.
        let crews: Vec<(u32, usize)> = self
            .mine_crew
            .range((0u32, outpost.0)..=(u32::MAX, outpost.0))
            .filter(|((_, o), _)| *o == outpost.0)
            .map(|((pl, _), c)| (*pl, c.len()))
            .collect();
        let mut any = false;
        for (player, crew) in crews {
            let amt = {
                let d = self.world.density.get(outpost).unwrap();
                // A fraction of the ore actually present — a **mass**, since
                // T-62 made the field's colours Bands and only their masses add.
                //
                // **Crowding, scaled to the deposit** (§4.3, T-71). Extraction is
                // `ε · S · W(n, S)` with `n` the crew in miner-equivalents — one
                // Limited hull, one unit. `ε` is `outpost_mining_fraction`
                // unchanged, which is what pins the calibration: on a `Band I`
                // body `N = 1`, so `W(1, S) = 1` and a lone miner extracts
                // exactly what it did before T-71.
                let stock = d.total_mass();
                let n = crew.max(1) as f64;
                let work = crowding_factor(n, stock, &self.config);
                stock.kilotons() * (self.config.outpost_mining_fraction * work).min(1.0)
            };
            if amt <= self.config.density_floor {
                continue;
            }
            any = true;
            let extracted = self.world.density.get_mut(outpost).unwrap().extract(Kilotons::new(amt));
            self.outpost_stock.entry((player, outpost.0)).or_default().add_basics(&extracted);
            let density_after = self.world.density.get(outpost).unwrap().total_mass().kilotons();
            self.log.push(
                self.clock,
                LogEvent::MineralsExtracted { planet: pid, amount: extracted.basic_total().kilotons(), density_after },
            );
        }
        if any {
            self.schedule(self.config.mining_tick_years, EventKind::MiningTick { outpost });
        } else {
            self.active_mines.remove(&outpost.0); // mined out
            self.log.push(self.clock, LogEvent::MiningExhausted { planet: pid });
            // The rock is done, but the hulls are not. Roles §4.6: a standing
            // mission that ends puts the vehicle in Reserve, re-taskable —
            // only a *completable* mission (an exhausted Scout) scraps. Every
            // player's crew stands down, since the rock is done for all of them.
            let keys: Vec<(u32, u64)> = self
                .mine_crew
                .range((0u32, outpost.0)..=(u32::MAX, outpost.0))
                .filter(|((_, o), _)| *o == outpost.0)
                .map(|(k, _)| *k)
                .collect();
            for k in keys {
                if let Some(crew) = self.mine_crew.remove(&k) {
                    if self.config.recycle_mining_pairs {
                        for miner in crew {
                            self.release_to_reserve(miner, Role::Miner, pid);
                        }
                    }
                }
            }
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

        // 0) **The `$` faucet** (§2.3, T-82). Income accrues where production
        // happens, on the cadence a rate needs — the economy tick is an
        // interval, and `$_income = base · production · dt` is a rate over one.
        // No new cadence and no per-player sweep: summing over centres *is* the
        // empire's production.
        //
        // **Nothing reads the purse yet**, which is the point of this stage: the
        // ledger and the faucet land inert and the bed stays bit-identical, so
        // the first thing that spends `$` is measured against a known baseline
        // (§10.7 stages 1–3).
        self.credit(pe, self.config.dollar_per_fabrication * self.fabrication_rate(center) * self.config.cycle_years);

        // 1) Local mining: the center works its own density into its stockpile.
        let amt = {
            let d = self.world.density.get(center).unwrap();
            let stock = d.total_mass();
            // **Same law as an outpost's crew, different way of buying `n`**
            // (§4.4, T-71). `extraction_rate` returns the centre's capacity in
            // miner-equivalents; the deposit decides what that capacity is worth.
            let work = crowding_factor(self.extraction_rate(center), stock, &self.config);
            stock.kilotons() * (self.config.outpost_mining_fraction * work).min(1.0)
        };
        if amt > 0.0 {
            let extracted = self.world.density.get_mut(center).unwrap().extract(Kilotons::new(amt));
            self.world.stockpile.get_mut(center).unwrap().add_basics(&extracted);
            let density_after = self.world.density.get(center).unwrap().total_mass().kilotons();
            self.log.push(
                self.clock,
                LogEvent::MineralsExtracted {
                    planet: center_pid,
                    amount: extracted.basic_total().kilotons(),
                    density_after,
                },
            );
        }

        // 2) Grow: population logistic toward `K`, paid for out of the planet's
        // standing biological mass.
        //
        // **Two constraints, two units, and they are not the same constraint.**
        // The logistic ceiling `K = min(hab, bio_max, infra)` is a Band — a
        // magnitude tier — and says how large this world's civilization can
        // ever get. The *payment* is a mass in kilotons drawn from `biomass`,
        // and says how fast it can get there. Before the units were typed
        // these were one term: the standing stock sat inside the Liebig
        // minimum, so a mass was being compared against two levels and a
        // world's ceiling fell every time its people ate.
        //
        // Population is **not** an exception to mass conservation (L6,
        // amended): a kiloton of people is a kiloton of biosphere that stopped
        // being biosphere. But the exchange is a *mass* difference, not a Band
        // difference — one more Band is `BAND_STEP` times the people, not
        // `BAND_STEP` more of them — which is why the draw is
        // `KT(after) − KT(before)` and not `after − before`.
        //
        // Biosphere then regrows logistically toward `bio_max`, which makes it
        // the only self-replenishing stock in the engine and puts a *rate* —
        // not just a ceiling — between an empire and its population. A world
        // grown faster than its ecology can regrow stalls until it recovers,
        // without its ceiling moving.
        let growth = doctrine.growth_rate;
        let regen = self.config.biosphere_regen_rate * doctrine.biosphere_regen_bonus;
        let k = self.world.factors.get(center).unwrap().k();
        {
            let pop_now = *self.world.population.get(center).unwrap();
            // **The logistic runs on the people, not on their Band** (T-64).
            //
            // Directed: *"Growth and construction and cost are denominated by
            // mass. All logistic functions are applied to real population, not
            // Bands."* It used to step `s + r·s·(1 − s/K)` with `s` and `K` as
            // *ladder positions*, then convert both ends to mass to pay the
            // biomass draw — which is a logistic in log space wearing a
            // logistic's clothes. A Band difference is not an amount of
            // anything, so `r` there was a rate of change of an exponent: the
            // same `growth_rate` meant a different number of people at every
            // point on the ladder, compounding hardest where the ladder is
            // widest.
            //
            // Now `K`'s mass is the carrying capacity and the step is the
            // textbook one. `growth_rate` finally means what its name says —
            // the fraction a small population adds per cycle — and its ratified
            // value is re-measured rather than carried across (T-64 step 5).
            let cap = units::population_mass(k);
            // The logistic has a fixed point at zero, so a founding population
            // needs a floor to grow off. The floor moves the *population*; the
            // mass baseline below stays at the true prior value, so the bump is
            // paid for like any other growth rather than conjured.
            let start = pop_now.max(units::POPULATION_SEED_FLOOR);
            let mut target = start;
            if cap > Kilotons::ZERO {
                let (s, c) = (start.kilotons(), cap.kilotons());
                target = Kilotons::new((s + growth * s * (1.0 - s / c)).clamp(0.0, c));
            }
            let mass_before = pop_now;
            let f = self.world.factors.get_mut(center).unwrap();
            let mut draw = target - mass_before;
            if draw > f.biomass {
                // Cannot make more people than there is biomass to make them
                // of. Spend the whole standing stock and land wherever that
                // reaches — the shortfall throttles the *rate*, never `K`.
                draw = f.biomass.max(Kilotons::ZERO);
                target = mass_before + draw;
            }
            // A shrinking population returns its mass: `draw` is negative here.
            f.biomass = (f.biomass - draw).max(Kilotons::ZERO);
            *self.world.population.get_mut(center).unwrap() = target;

            // Logistic regrowth toward the pristine ceiling, in kilotons — and
            // the logistic runs on the world's **living mass**, biosphere plus
            // people, not on the biosphere alone.
            //
            // That is design law #11 taken at its word rather than half of it.
            // If people are made of biomass then they are part of the standing
            // ecology, and two things follow that the biosphere-only form got
            // wrong. A world whose population has eaten its biosphere down to
            // nothing is not sterile — `r·0·(1 − 0)` is zero, so under the old
            // form such a world could *never* recover, a trap state a
            // mass-denominated draw can now actually reach. And a world filled
            // to its ceiling with people has no spare ecological niche, so its
            // biosphere must not regrow into one: `living == bio_max` is
            // saturation whether the mass is standing in forests or walking
            // around in cities.
            //
            // The rate stays relative, so `biosphere_regen_rate` keeps its
            // meaning and its ratified value under the rebasing.
            if f.bio_max > Kilotons::ZERO && regen > 0.0 {
                let headroom = 1.0 - f.biomass.kilotons() / f.bio_max.kilotons();
                if headroom > 0.0 {
                    // **The multiplier counts the people; the ceiling does
                    // not.** Both halves are deliberate.
                    //
                    // Counting people in the multiplier is what stops a trap
                    // the mass-denominated draw makes reachable: a population
                    // can now empty its biosphere *exactly*, and `r·0·(1 − 0)`
                    // is zero, so a biosphere-only logistic would leave such a
                    // world permanently sterile. A world with people on it is
                    // not sterile.
                    //
                    // Leaving them out of the ceiling is design law #11 taken
                    // literally: biosphere "regrows logistically toward
                    // `bio_max`", full stop. Making the population share that
                    // ceiling is a *stronger* coupling than the law asks for,
                    // and it does double duty with `K`, which already caps
                    // population at `min(hab, bio_max, infra)`. Measured on the
                    // standard four-seed bed, the two forms are **bit-identical
                    // in colony count** — the mass budget never binds at the
                    // shipped defaults — so this is settled on fidelity to the
                    // spec, not on a number. If a starvation die-back ever
                    // lands (R-O67), the shared ceiling is the form to revisit.
                    let seeded = (f.biomass + target).max(Kilotons::ZERO);
                    // The ceiling applies to the *regrowth* only, never to the
                    // standing stock: a clamp that can push `biomass` downward
                    // is a silent mass sink, and it would quietly absorb any
                    // over-ceiling state a card wrote rather than letting the
                    // conservation checks see it.
                    f.biomass = (f.biomass + seeded * (regen * headroom)).min(f.bio_max.max(f.biomass));
                }
            }
        }
        self.log.push(
            self.clock,
            LogEvent::PopulationStep {
                planet: center_pid,
                population: self.world.population.get(center).unwrap().kilotons(),
                k: k.bands(),
            },
        );

        // 3) Decide, but only if the yard is free.
        //
        // **This is the decoupling (R-O69).** The economy above is a rate over
        // an interval and legitimately wants a cadence; the *decision* is not,
        // and pinning it to the same 50-year tick capped every center at one
        // build per cycle however rich it was. Measured before the change
        // (`examples/cadence_throttle`, seeds 1 and 7): the median funded build
        // fired at **5.5x the cost of what it bought**, 82% of funded builds
        // could have been made at least twice that cycle and 55% at least five
        // times, with a worst case of 112x. That is a serial dependency setting
        // the expansion-loop time constant, not an economy running dry.
        //
        // A center with a build under way is skipped: its decision comes from
        // [`EventKind::BuildDecision`] when the yard clears. A center with an
        // empty yard has no such event pending, so the mining step doubles as
        // its retry — mining is what changes a saving center's situation.
        //
        // **T-88 will sever this call.** `cycle_years` is doing two unrelated
        // jobs — the economic integration step (mine, grow, mint, all rates over
        // an interval, which want a *small* step for fidelity) and the retry
        // cadence for a saving centre (which should not be a cadence at all).
        // Dropping the tick to 1/yr for granularity would multiply decisions
        // 50x along with it, and the decision half is what costs throughput.
        // The retry belongs on the events that actually change a saving
        // centre's situation — minerals arriving — not on a clock.
        if self.free_berths(center) > 0 {
            self.sys_build_decision(center);
        }
        self.schedule(self.config.cycle_years, EventKind::ProductionTick { center });
    }

    /// A center chooses its next build, and occupies its yard if it commits.
    ///
    /// Reached two ways: the economy tick above when the yard is idle, and
    /// `BuildDecision` when a build finishes. Committing spends the minerals and
    /// applies the build immediately — `apply_build_with` is unchanged, so a
    /// vehicle's launch delay and arrival time are exactly what they were — and
    /// then holds the yard for `build_years` before the next decision. **The
    /// only behavioural change is the cadence of decisions**, from
    /// `cycle_years` to `build_years` for a center that keeps finding things to
    /// buy.
    fn sys_build_decision(&mut self, center: Entity) {
        // **Retire the berths that have cleared, not the whole yard** (T-69).
        // Reached two ways — the economy tick with a berth free, and a
        // `BuildDecision` scheduled when one clears — and in both cases the
        // question is how many are still occupied, not whether any are.
        if let Some(b) = self.world.berths.get_mut(center) {
            let now = self.clock;
            b.retain(|&t| t > now + 1e-12);
            if b.is_empty() {
                self.world.berths.remove(center);
            }
        }
        // **Fill every free berth, not one** (T-69). Committing a single build
        // per decision would leave the extra slips permanently idle while still
        // dividing the yard's throughput among them — all of concurrency's cost
        // and none of its benefit, which would measure as a regression for a
        // reason that has nothing to do with the mechanism. The loop terminates
        // because each commit takes a berth and `commit_one_build` returns
        // `false` the moment the centre cannot or will not buy anything more.
        while self.free_berths(center) > 0 {
            if !self.commit_one_build(center) {
                break;
            }
        }
    }

    /// One pass of the production decision: pick an order, commit it if it is
    /// affordable and worth doing, and occupy a berth for its build time.
    /// Returns whether anything was committed — which is what stops
    /// [`Self::sys_build_decision`]'s loop.
    fn commit_one_build(&mut self, center: Entity) -> bool {
        let owner = match self.world.owner.get(center).copied() {
            Some(o) => o,
            None => return false, // lost the world; no yard to run
        };
        let p = owner.0 as usize;
        let pe = self.player_entity[p];
        let doctrine = *self.world.doctrine.get(pe).unwrap();
        let center_pid = *self.world.planet_id.get(center).unwrap();

        let (infra, infra_band, k_potential) = {
            let f = self.world.factors.get(center).unwrap();
            // One reading, taken at the edge — the decision and the log line
            // both want the rung, and `band_from` is a `ln` (`CLAUDE.md` §4).
            (f.infra, f.infra_band(&self.config), f.k_potential())
        };
        let level = self.bands.level(*self.world.population.get(center).unwrap());
        let center_pos = *self.world.position.get(center).unwrap();
        let stock_total = self.world.stockpile.get(center).unwrap().basic_total();
        // Minerals to buy the next whole level, from the stock standing there —
        // and since T-73 the bill is payable *in colours*, so the split and the
        // bank both go into the context.
        let target_level = infra_step_price(infra, &self.config);
        let works = self.world.works.get(pe).copied().unwrap_or_default();
        let infra_bill = works_bill(target_level, &works);
        let bank = self.world.stockpile.get(center).copied().unwrap_or_default();
        let stockpile_by_colour = [
            Price::new(bank.get_basic(Basic::Cyan)),
            Price::new(bank.get_basic(Basic::Magenta)),
            Price::new(bank.get_basic(Basic::Yellow)),
        ];

        let info = *self.world.player_info.get(pe).unwrap();
        // Live mineral pressure for this center: 1 when broke for its next infra
        // upgrade, 0 when it can comfortably afford it. Drives the ranking toward
        // mining when the empire is short.
        let mineral_pressure = self.mineral_pressure_of(center);
        let rctx =
            RankContext { scarcity: info.scarcity, holdings_centroid: self.holdings_centroid(p), mineral_pressure };
        // **Reduce, do not materialize (R-O70).** Both consumers —
        // `production_choice` and `assign_role` — read exactly four things off
        // this list: the per-class argmax by `score_then_id` for
        // ProductionCenter, Colony and MiningOutpost, and the count. They never
        // iterate it for anything else.
        //
        // Building a `Vec` of every non-Barren scanned world to hand over three
        // winners is §4's "do not materialize a collection you only `max_by`
        // over", at the engine's hottest scale: thousands of 112-byte pushes
        // per decision, and decisions now fire on build completion rather than
        // once per 50 years (R-O69), which is what made the waste visible.
        //
        // The reduction is exact, not an approximation. `score_then_id` is a
        // total order (ids are unique), so the maximum of a class is unique and
        // the max of the per-class maxima *is* the max of the whole list —
        // independent of scan order. `candidate_count` still carries the true
        // count, because `survey_reserve` is a threshold on the size of the
        // frontier and not on the size of this slice.
        let mut count = 0usize;
        let mut best: [Option<Candidate>; 3] = [None, None, None];
        {
            let knowledge = self.world.knowledge.get(pe).unwrap();
            for &pid in &knowledge.scanned {
                if knowledge.targeted.contains(pid) {
                    continue;
                }
                let e = self.planet_entity[pid.0 as usize];
                if self.world.owner.contains(e) {
                    continue;
                }
                let view = self.view_of(e);
                let ranked = self.autopilots[p].rank(&doctrine, &view, &rctx);
                let slot = match ranked.class {
                    PlanetClass::ProductionCenter => 0,
                    PlanetClass::Colony => 1,
                    PlanetClass::MiningOutpost => 2,
                    PlanetClass::Barren => continue,
                };
                count += 1;
                let better = match &best[slot] {
                    None => true,
                    Some(cur) => Ranked::score_then_id(&ranked, &cur.ranked).is_gt(),
                };
                if better {
                    // The per-hull settler figure is computed for the three
                    // reduced winners only, not for every scanned world — the
                    // reduction is exactly what makes that affordable (R-O70).
                    let settlers_by_hull = [
                        self.settler_target(center, e, HullType::MediumSystems.colony_seed_capacity(&self.config)),
                        self.settler_target(center, e, HullType::GeneralSystems.colony_seed_capacity(&self.config)),
                    ];
                    let mining_crew = self.mining_crew_for(center, e);
                    best[slot] = Some(Candidate { view, ranked, settlers_by_hull, mining_crew });
                }
            }
        }
        let cands: Vec<Candidate> = best.into_iter().flatten().collect();
        // Built after `cands`, so the survey decision can see how much frontier
        // this empire has left to aim at.
        let ctx = ProductionContext {
            center_pos,
            level,
            infra: infra_band.bands(),
            k_potential: k_potential.bands(),
            stockpile_total: stock_total,
            medium_min_level: self.config.medium_min_level,
            limited_min_level: self.config.limited_min_level,
            infra_cost: target_level,
            infra_bill,
            stockpile_by_colour,
            colonizer_cost: hull_cost(HullType::MediumSystems, &self.config),
            general_colonizer_cost: hull_cost(HullType::GeneralSystems, &self.config),
            medium_seed_capacity: HullType::MediumSystems.colony_seed_capacity(&self.config),
            general_seed_capacity: HullType::GeneralSystems.colony_seed_capacity(&self.config),
            medium_founding_infra: self.founding_infra_band(HullType::MediumSystems),
            general_founding_infra: self.founding_infra_band(HullType::GeneralSystems),
            // The *true* price of a pair to this center right now. With
            // recycling on, a half that comes out of Reserve is not bought, and
            // the context has to say so or the decision is made on a price the
            // build step will not charge: a center too poor for a new pair would
            // sit Idle next to hulls it already owns. `apply_build_with` takes
            // the nearest reserved hull of each kind, so this matches what it
            // will actually spend.
            // **Priced for the crew the chosen rock would get** (T-72). The
            // autopilot only ever spends this on `best_mining`, and since T-71
            // the crew is a property of the deposit — so pricing it from a flat
            // doctrine count would put a number in front of the decision that
            // the build step will not charge, which is the exact defect the
            // comment above is about. `Candidate::mining_crew` is the one
            // computation of the rule; this reads it rather than repeating it.
            mining_pair_cost: self.mining_pair_price(
                p,
                cands.iter().find(|c| c.ranked.class == PlanetClass::MiningOutpost).map_or(1, |c| c.mining_crew),
            ),
            light_vehicle_cost: role_cost(Role::Scout, &self.config),
            candidate_count: count,
        };

        let order = self.autopilots[p].production_choice(&doctrine, &ctx, &cands);
        self.log.push(
            self.clock,
            LogEvent::ProductionDecision {
                player: p as u32,
                center: center_pid,
                pop_level: level,
                infra: infra_band.bands(),
                k_potential: k_potential.bands(),
                stockpile: stock_total.kilotons(),
                infra_cost: target_level.kilotons(),
                colonizer_cost: ctx.colonizer_cost.kilotons(),
                mining_pair_cost: ctx.mining_pair_cost.kilotons(),
                mineral_pressure,
                candidates_seen: count as u32,
                chosen: order,
            },
        );
        // Committing occupies the yard. Only a build that actually spent counts:
        // `apply_build_with` declines on an unaffordable price, a roster gate, or
        // a hull with no job worth doing, and a center that built nothing must
        // not be held busy for it.
        let Some(committed) = self.apply_build_with(p, center, center_pos, order, &cands) else {
            // The yard stays free and the next economy tick retries, which is
            // the right cadence for a center whose situation only changes as it
            // mines. Returning `false` also ends the fill loop — a centre that
            // declined this order will decline it again at the same instant.
            return false;
        };
        let done = self.clock + self.build_time(center, committed);
        let b = self.world.berths.entry_or_insert_with(center, Vec::new);
        b.push(done);
        // Ascending, so the list reads as "when does the next one clear" and
        // stays deterministic however completions interleave.
        b.sort_by(|x, y| x.partial_cmp(y).unwrap_or(core::cmp::Ordering::Equal));
        self.schedule_at(done, EventKind::BuildDecision { center });
        true
    }

    // --- build application -------------------------------------------------

    /// **How long a yard is occupied making `mass`** — `t_build = t_lead + m /
    /// F_slip` (`Hyades_industry.md` §3.2, T-68).
    ///
    /// Takes the **mass actually committed**, which is what makes one expression
    /// cover every order the engine has: a hull, an infrastructure rung, or a
    /// whole mining pair. Under R-O57 dry mass and mineral cost are the same
    /// number, so "what this build costs" and "how much stuff it is" are not two
    /// quantities — and a mining pair drawn partly from Reserve is cheaper *and*
    /// quicker, because there is genuinely less to fabricate.
    ///
    /// It is a `Price` on the way in and kilotons on the way out: the same
    /// amount, read on the ladder each side wants (`units::Qty::on_scale`).
    ///
    /// **Stage 2 is the single-slip case.** `slips(F)` and the emergent soft
    /// floor are T-69; this is deliberately the `slips = 1` specialisation of
    /// the same formula, so landing concurrency changes the divisor rather than
    /// the model.
    fn build_time(&self, center: Entity, mass: Price) -> f64 {
        // **Throughput divides among the slips** (T-69). That is what makes the
        // turnaround floor emergent rather than imposed: as `F` grows, `slips`
        // grows with it and `F / slips` approaches `F_slip` from above, so one
        // hull never builds faster than `t_lead + m / F_slip` however rich the
        // empire is. Industry buys more ships at once, never faster ships.
        let f = self.fabrication_rate(center);
        let per_berth = (f / slips(f, &self.config) as f64).max(1e-12);
        self.config.build_lead_years + mass.on_scale::<units::Mass>().kilotons().max(0.0) / per_berth
    }

    /// **This centre's fabrication throughput, kt/yr** (T-74). It was the flat
    /// `slip_throughput`; it is now what that centre's *works* produce, so
    /// deepening buys build rate and razing takes it away.
    fn fabrication_rate(&self, center: Entity) -> f64 {
        let infra = self.world.factors.get(center).map(|f| f.infra).unwrap_or(Price::ZERO);
        let works = self
            .world
            .owner
            .get(center)
            .and_then(|o| self.world.works.get(self.player_entity[o.0 as usize]))
            .copied()
            .unwrap_or_default();
        employment_rate(infra, &works, cards::Employment::Fabrication, self.config.fab_cap, works_knee(&self.config))
    }

    /// **How many berths this centre has spare** (T-69). Zero means every slip
    /// is busy and the yard cannot commit again until one clears.
    fn free_berths(&self, center: Entity) -> usize {
        let total = slips(self.fabrication_rate(center), &self.config);
        let busy = self.world.berths.get(center).map(|b| b.len()).unwrap_or(0);
        total.saturating_sub(busy)
    }

    /// **This centre's extraction capacity, in miner-equivalents** (T-71).
    ///
    /// ~~The fraction of its own density it works per cycle~~ — T-74 shipped that
    /// as a Michaelis–Menten curve on infrastructure, and §4.3a retired the form:
    /// MM has **no deposit term**, so it would make crowding identical on a
    /// `Band I` pebble and a `Band IV` seam, which is the exact fault §4.2 exists
    /// to correct. Extraction saturates once, through §4.3's crowding law; §6.3's
    /// curve stays where it belongs, on fabrication.
    ///
    /// So this returns `n`, and [`crowding_factor`] turns `n` into a rate
    /// against the body being worked — the **same law an outpost's crew runs
    /// on** (§4.4). The two differ only in how capacity is bought: hulls for an
    /// outpost, infrastructure for a colony.
    ///
    /// The works coefficients keep their tree meanings on §4.3's parameters
    /// (§4.3a): `cap[Extraction]` scales the capacity a given stock buys —
    /// Production's asymptote — and `half[Extraction]` divides it, so lowering
    /// the knee buys more work per kilotonne, which is Growth and Expansion's
    /// signature.
    fn extraction_rate(&self, center: Entity) -> f64 {
        let infra = self.world.factors.get(center).map(|f| f.infra).unwrap_or(Price::ZERO);
        let works = self
            .world
            .owner
            .get(center)
            .and_then(|o| self.world.works.get(self.player_entity[o.0 as usize]))
            .copied()
            .unwrap_or_default();
        let e = cards::Employment::Extraction;
        let stock = infra.kilotons() * works.alloc_share(e);
        if stock <= 0.0 {
            return 0.0;
        }
        let scale = works.cap[e.index()] / works.half[e.index()].max(1e-12);
        stock * scale / miner_equivalent(&self.config)
    }

    /// Apply a production order. `candidates` is the empire's current candidate
    /// list, used to **task** a finished hull — production decides *what object*
    /// to make, role assignment decides *what it is for* (R-O29).
    /// Apply a chosen build, returning **the mass it actually committed** —
    /// `None` for a decline (unaffordable, gated by the roster, or a hull with
    /// no job worth doing), in which case the caller must not occupy the yard
    /// for a build that never happened.
    ///
    /// It returns the *committed* price rather than a bare `bool` because since
    /// T-68 the yard is held for `build_time(mass)`, and the mass is only known
    /// here: recycling means a mining pair's real price depends on what was
    /// sitting in Reserve when the order was placed. Returning `true` and
    /// re-deriving the cost outside would have been a second copy of that rule,
    /// which is how `mining_pair_cost` came to need a comment explaining that it
    /// must match what `apply_build_with` will spend.
    fn apply_build_with(
        &mut self,
        p: usize,
        center: Entity,
        center_pos: Vec3,
        order: BuildOrder,
        candidates: &[Candidate],
    ) -> Option<Price> {
        let center_pid = *self.world.planet_id.get(center).unwrap();
        match order {
            BuildOrder::Idle => None,
            BuildOrder::UpgradeInfrastructure => {
                let target = infra_step_price(self.world.factors.get(center).unwrap().infra, &self.config);
                // **Pay the colour bill, not the total** (T-73). Same
                // `works_bill` the decision was made against.
                let works = self.world.works.get(self.player_entity[p]).copied().unwrap_or_default();
                let bill = works_bill(target, &works);
                let payable = self.world.stockpile.get(center).map(|b| can_pay_bill(b, &bill)).unwrap_or(false);
                // What was actually committed is the bill, not the ladder step:
                // `eta_works` divides the total, so an efficiency card makes the
                // rung genuinely cheaper — and the yard is held for what was
                // built (T-68), which has to be the same number.
                let billed: Price = bill.iter().fold(Price::ZERO, |a, &b| a + b);
                if payable {
                    pay_bill(self.world.stockpile.get_mut(center).unwrap(), &bill);
                    let f = self.world.factors.get_mut(center).unwrap();
                    // **A rung is bought, not incremented.** The stock moves to
                    // exactly what standing at the next rung costs, so the
                    // ladder stays the single source of the number (T-70).
                    let next = infra_rung_of(f.infra, &self.config) + 1;
                    f.infra = infra_rung_price(next, &self.config);
                    let stockpile_after = self.world.stockpile.get(center).unwrap().basic_total();
                    self.log.push(
                        self.clock,
                        LogEvent::BuildApplied {
                            player: p as u32,
                            center: center_pid,
                            order,
                            cost: billed.kilotons(),
                            stockpile_after: stockpile_after.kilotons(),
                        },
                    );
                    Some(billed)
                } else {
                    None
                }
            }
            BuildOrder::Hull { hull_type, class } => {
                if !self.roster_permits(p, hull_type) {
                    return None;
                }
                let doctrine = *self.world.doctrine.get(self.player_entity[p]).unwrap();
                // The job is chosen here, after the object exists — not in the
                // order, which is all a rival could read off the shipyard.
                let tasking = self.autopilots[p].assign_role(&doctrine, hull_type, class, candidates);
                let Some(Tasking { role, target }) = tasking else {
                    return None; // nothing worth building this hull for right now
                };

                // A Miner is produced together with the Freighter that hauls for
                // it (roles §5: the nearest center produces both), so the pair is
                // one economic act even though it is two objects.
                let paired_freighter = role == Role::Miner;
                if paired_freighter && !self.roster_permits(p, role_hull_type(Role::Freighter)) {
                    return None;
                }

                // Recycling is checked *before* pricing, because a hull taken
                // from Reserve is not bought: the pair's cost is only the halves
                // that still have to be built. A center that could not afford a
                // new pair can therefore still open an outpost with idle hulls,
                // which is the whole point — 39% of outpost-years were being
                // spent on exhausted rocks.
                // **The crew is the target's, not the doctrine's** (T-72). Read
                // from the candidate list so it is bit-identical to what
                // `mining_pair_cost` quoted; falling back to the deposit only if
                // the target is not among the candidates.
                let crew = if role == Role::Miner {
                    match target.map(|t| self.planet_entity[t.0 as usize]) {
                        Some(te) => candidates
                            .iter()
                            .find(|c| self.planet_entity[c.view.id.0 as usize] == te)
                            .map_or_else(|| self.mining_crew_for(center, te), |c| c.mining_crew),
                        None => 1,
                    }
                } else {
                    1
                };
                let target_entity = target.map(|t| self.planet_entity[t.0 as usize]);
                let (reused_miners, reused_freighter) = match (self.config.recycle_mining_pairs, role, target_entity) {
                    (true, Role::Miner, Some(te)) => {
                        let at = *self.world.position.get(te).unwrap();
                        // One reserved hull per crew slot, nearest first.
                        let mut taken = Vec::new();
                        while taken.len() < crew {
                            match self.take_nearest_reserve(p, true, at) {
                                Some(e) => taken.push(e),
                                None => break,
                            }
                        }
                        (taken, self.take_nearest_reserve(p, false, at))
                    }
                    _ => (Vec::new(), None),
                };
                // **Price the hull that was ordered, not the hull the role
                // implies.** R-O29 moved the hull choice into `BuildOrder`, but
                // this line kept reading it back off the role — which agreed
                // only because `role_hull_type` happened to invert
                // `assign_role`. It stops agreeing the moment Doctrine can pick
                // a heavier colonizer, and the failure would have been silent:
                // a General hull bought at a Medium hull's price.
                let mut cost = hull_cost(hull_type, &self.config) * (crew - reused_miners.len().min(crew)) as f64;
                if paired_freighter && reused_freighter.is_none() {
                    cost += role_cost(Role::Freighter, &self.config);
                }

                if let Some(t) = target {
                    self.mark_targeted(p, t);
                }
                // **The whole order occupies one yard, so it launches as one
                // unit** (T-68). The delay every hull in this order waits out is
                // the order's own `t_build`, which is also exactly how long the
                // caller holds the yard — one number, computed once, rather than
                // a per-ship time that could drift from the occupancy. Hulls
                // taken from Reserve are already built and leave at once.
                let launch_delay = self.build_time(center, cost);
                if !self.world.stockpile.get_mut(center).unwrap().try_spend_total(cost) {
                    // Put anything taken from Reserve back, or the hulls vanish
                    // on a build that never happened.
                    for e in reused_miners {
                        self.reserve_miners[p].push(e);
                    }
                    if let Some(e) = reused_freighter {
                        self.reserve_freighters[p].push(e);
                    }
                    return None;
                }
                match (role, target) {
                    (Role::Scout, _) => {
                        // Under `PersistentSectors` (R-AC3), a paid-for replenishment
                        // Scout inherits an outward heading from home through the
                        // center that built it, instead of always pooling globally.
                        // `normalized()` returns `ZERO` for a center at home itself,
                        // which degrades to global pool for that one craft — the
                        // correct degenerate case, not a special case to add.
                        let heading = match doctrine.survey_strategy {
                            SurveyStrategy::PersistentSectors => {
                                let home = self.world.player_info.get(self.player_entity[p]).unwrap().home;
                                let home_pos = *self.world.position.get(home).unwrap();
                                center_pos.sub(home_pos).normalized()
                            }
                            SurveyStrategy::GlobalPool | SurveyStrategy::OpeningSectors => Vec3::ZERO,
                        };
                        self.launch_survey(p, center_pos, heading, 0, launch_delay)
                    }
                    (r, Some(t)) => {
                        let te = self.planet_entity[t.0 as usize];
                        // A crew, not an operator (T-57). Reserved hulls first —
                        // they are already paid for — then newly built ones.
                        let mut reused = reused_miners.into_iter();
                        for _ in 0..crew {
                            match reused.next() {
                                Some(e) => self.retask_miner(e, p, center, te),
                                None => self.spawn_courier(p, r, hull_type, center, te, launch_delay),
                            }
                        }
                        if paired_freighter {
                            match reused_freighter {
                                Some(e) => self.retask_freighter(e, p, center, te),
                                None => self.spawn_freighter(p, center, center_pos, te, launch_delay),
                            }
                        }
                    }
                    (_, None) => {}
                }
                let stockpile_after = self.world.stockpile.get(center).unwrap().basic_total();
                self.log.push(
                    self.clock,
                    LogEvent::BuildApplied {
                        player: p as u32,
                        center: center_pid,
                        order,
                        cost: cost.kilotons(),
                        stockpile_after: stockpile_after.kilotons(),
                    },
                );
                Some(cost)
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

    /// What a mining pair costs player `p` this cycle: the halves that are not
    /// already sitting in Reserve. Equals the full price whenever recycling is
    /// off, which is what keeps the flag a clean A/B.
    /// **How many miners to put on this body — the crew that meets the buyer's
    /// unmet demand, and no more** (`Hyades_industry.md` §4.5, T-87).
    ///
    /// **Crew is not a parameter.** It was `miners_per_outpost` (T-57, a flat
    /// hull count) and then `miner_vein_fraction` (T-72, a share of the rock),
    /// and both were policies someone had to tune against an objective that
    /// could not price them. The measured verdict on the second was monotone in
    /// the wrong direction — every crew size scored worse than the one below,
    /// on both seeds and both objectives (§6.15) — and the reason is now the
    /// model rather than a footnote: **ore nobody can spend is a cost.** A crew
    /// sized without reference to demand buys hulls, freight and entity count
    /// to bring forward a resource the empire is not short of.
    ///
    /// So the crew *falls out* of demand. There is no knob.
    ///
    /// **Demand**, in kilotons per year, is what the founding centre can
    /// actually consume and currently cannot get:
    ///
    /// ```text
    /// D = fabrication_rate(centre) · mineral_pressure(centre)
    /// ```
    ///
    /// Both terms already exist and both already run on this decision path.
    /// `fabrication_rate` is the rate the yard turns minerals into mass — the
    /// only sink that consumes ore — and `mineral_pressure` is `1.0` when the
    /// centre is broke for its next rung and `0.0` when it is comfortable. A
    /// centre with a full bank has no unmet demand and opens a mine with one
    /// hull; a starved one crews to its throughput.
    ///
    /// **Supply** is §4.3's law read forwards, so the crew is that law inverted:
    ///
    /// ```text
    /// supply(n) = ε · S · (n/N)^β / T          kt/yr, T = mining_tick_years
    /// n*        = N · (D · T / (ε · S))^(1/β)   clamped to [1, N]
    /// ```
    ///
    /// The sign this produces is the interesting part and it is the opposite of
    /// both retired policies: **a richer rock wants a smaller crew**, because it
    /// meets the same demand with fewer hands. Deposit mass grows as `N^{3/2}`
    /// while the demand target does not grow at all, so `n*` *falls* as the body
    /// gets richer. Under a flat fraction it rose. That inversion is why this is
    /// a model change and not a retuning.
    ///
    /// **R-IND20: demand is read at the founding centre, not empire-wide.** An
    /// outpost feeds the whole empire through freight, so the correct demand is
    /// the empire's unmet total; that is an `O(planets)` scan on a decision path
    /// (`CLAUDE.md` §4) and would need the `holdings_centroid` memo treatment.
    /// The centre that pays for the pair is the defensible local proxy, and the
    /// difference is what R-IND20 is for.
    fn mining_crew_for(&self, center: Entity, planet: Entity) -> usize {
        let stock = self.world.density.get(planet).map(|d| d.total_mass()).unwrap_or(Kilotons::ZERO);
        let ore = stock.kilotons();
        if ore <= 0.0 {
            return 1;
        }
        let demand = self.fabrication_rate(center) * self.mineral_pressure_of(center);
        if demand <= 0.0 {
            return 1;
        }
        let n_veins = veins(stock, &self.config);
        // What one full crew would lift per year, against what is wanted per
        // year. Both sides are kt/yr, which is the check that this is a rate
        // comparison and not two magnitudes that happen to be `f64`.
        let full_crew_rate = self.config.outpost_mining_fraction * ore / self.config.mining_tick_years.max(1e-12);
        let share = (demand / full_crew_rate.max(1e-12)).clamp(0.0, 1.0);
        let beta = self.config.crowding_beta.max(1e-6);
        let want = n_veins * share.powf(1.0 / beta);
        (want.round().clamp(1.0, n_veins.min(MAX_VEINS)) as usize).max(1)
    }

    fn mining_pair_price(&self, p: usize, crew: usize) -> Price {
        let full_miner = role_cost(Role::Miner, &self.config);
        let full_freighter = role_cost(Role::Freighter, &self.config);
        if !self.config.recycle_mining_pairs {
            return full_miner * crew as f64 + full_freighter;
        }
        // Only the hulls that still have to be *built* are priced; the rest come
        // out of Reserve. A crew of `n` can draw up to `n` reserved miners.
        let from_reserve = self.reserve_miners[p].len().min(crew);
        let miners = full_miner * (crew - from_reserve) as f64;
        let freighter = if self.reserve_freighters[p].is_empty() { full_freighter } else { Price::ZERO };
        miners + freighter
    }

    /// A hull whose mission ended puts itself in Reserve and joins its owner's
    /// pool (`Hyades_vehicle_roles.md` §4.6). It keeps its position — Reserve is
    /// a standing state, not a recall — so the flight it will eventually make is
    /// paid from wherever the rock left it.
    fn release_to_reserve(&mut self, vehicle: Entity, role: Role, at: PlanetId) {
        let Some(&owner) = self.world.owner.get(vehicle) else { return };
        let p = owner.0 as usize;
        self.world.role.insert(vehicle, Role::Reserve);
        match role {
            Role::Miner => self.reserve_miners[p].push(vehicle),
            _ => self.reserve_freighters[p].push(vehicle),
        }
        self.log.push(self.clock, LogEvent::VehicleParked { player: owner.0, vehicle, role: Role::Reserve, at });
    }

    /// Take the reserved hull **nearest the target** out of `pool`, if any.
    ///
    /// Nearest rather than first: these hulls are scattered across every rock
    /// the empire has ever worked out, so the choice is a real distance and the
    /// flight is the whole cost of recycling. The scan is `O(idle hulls of one
    /// kind for one player)` — bounded by the fleet, not by the galaxy, and run
    /// only when a center actually orders a mining pair (§4's locality rule).
    fn take_nearest_reserve(&mut self, p: usize, miner: bool, target: Vec3) -> Option<Entity> {
        let pool = if miner { &self.reserve_miners[p] } else { &self.reserve_freighters[p] };
        let mut best: Option<(usize, f64)> = None;
        for (i, &e) in pool.iter().enumerate() {
            let Some(pos) = self.position_at(e, self.clock) else { continue };
            let d = pos.distance(target);
            // Deterministic argmin: distance, tie-broken by pool order.
            if best.map(|(_, bd)| d < bd).unwrap_or(true) {
                best = Some((i, d));
            }
        }
        let (i, _) = best?;
        let pool = if miner { &mut self.reserve_miners[p] } else { &mut self.reserve_freighters[p] };
        Some(pool.remove(i))
    }

    /// Send a reserved hull back out on a fresh mining mission. No build delay:
    /// the ship exists, so only the flight is ahead of it.
    fn retask_miner(&mut self, e: Entity, p: usize, center: Entity, target: Entity) {
        let from = self.position_at(e, self.clock).unwrap_or(Vec3::ZERO);
        let dest = *self.world.position.get(target).unwrap();
        self.world.role.insert(e, Role::Miner);
        self.world.voyage.insert(e, Voyage { target, heading_bias: None, hops: 0 });
        self.world.home_center.insert(e, center);
        let accel = self.config.civilian_accel_g * G;
        let arrive = self.set_leg(e, from, dest, accel, 0.0);
        self.schedule_at(arrive, EventKind::MiningArrive { vehicle: e });
        let target_pid = *self.world.planet_id.get(target).unwrap();
        self.log.push(
            self.clock,
            LogEvent::VehicleSpawned {
                player: p as u32,
                vehicle: e,
                role: Role::Miner,
                from,
                to: target_pid,
                settlers: 0.0,
                endowment: 0.0,
            },
        );
    }

    /// Re-bind a reserved freighter to a new outpost. The pickup leg is the
    /// fixed one (`sys_freighter_arrive`), so re-binding it is exactly what
    /// recycling means for this hull.
    fn retask_freighter(&mut self, e: Entity, p: usize, center: Entity, outpost: Entity) {
        let from = self.position_at(e, self.clock).unwrap_or(Vec3::ZERO);
        let dest = *self.world.position.get(outpost).unwrap();
        self.world.role.insert(e, Role::Freighter);
        self.world.home_center.insert(e, center);
        self.world.shuttle.insert(e, Shuttle { outpost, destination: center, outbound: true });
        let accel = self.config.civilian_accel_g * G;
        let arrive = self.set_leg(e, from, dest, accel, 0.0);
        self.schedule_at(arrive, EventKind::FreighterArrive { vehicle: e });
        let outpost_pid = *self.world.planet_id.get(outpost).unwrap();
        self.log.push(
            self.clock,
            LogEvent::VehicleSpawned {
                player: p as u32,
                vehicle: e,
                role: Role::Freighter,
                from,
                to: outpost_pid,
                settlers: 0.0,
                endowment: 0.0,
            },
        );
    }

    /// **The infrastructure a colony has the instant it is founded — the
    /// recycled hull, converted as a mass** (R-O76).
    ///
    /// `sys_colony_arrive` has always said the colony ship's hull *becomes* the
    /// colony's first infrastructure, and then awarded one Band regardless of
    /// what was recycled — which pinned every new colony's `K` at one Band and
    /// made a heavier colony ship pointless by construction.
    ///
    /// **The hull's mass *is* the infrastructure it becomes — no rate, no
    /// subsidy (R-O77 closed).** Dry mass is the mineral cost (L6/R-O57), so
    /// the conversion is the identity and the only work here is reading the
    /// mass back onto the ladder:
    ///
    /// ```text
    /// infra = band_of(hull_cost)
    /// ```
    ///
    /// | hull | cost / infra mass | founding infra |
    /// |---|---|---|
    /// | Limited | 0.02 kt | `Band 0` — founds nothing |
    /// | Medium | 0.10 kt | **`Band 0.33`** |
    /// | General | 1.00 kt | **`Band I`** |
    ///
    /// **This is what removing the subsidy does, and it is meant to hurt.**
    /// Until now a Medium hull's 0.1 minerals became a whole Band of
    /// infrastructure that the ladder charges 1.0 for — founding conjured 10×
    /// the mass spent, in flat contradiction of design law #11. With the
    /// conjuring gone a Medium colonizer founds at a *third* of a rung, and a
    /// General hull is the only one that reaches `Band I`: the first
    /// configuration in which "a Medium hull unless a General is required"
    /// has ever had a case where a General is required.
    ///
    /// A colony founded at `Band 0.33` is a real colony that is badly short of
    /// everything, which is the intended pressure — it must be supplied rather
    /// than born adequate.
    ///
    /// **Two earlier versions of this were wrong, both in the same way.** The
    /// first summed Band numerals (`b(b+1)/2`, the cumulative `1+2+3+4 = 10` of
    /// the linear price ladder) and put a General hull at `Band IV`. The second
    /// did the arithmetic in kilotons but kept the subsidised rate, giving
    /// `Band 1.67`. `I + II + III + IV` is not `X`, and a 10× conjuring is not a
    /// conversion.
    ///
    /// R-V9 needs no special case at either end: a Limited hull's mass reads
    /// below `Band Empty`, so its colony would have no carrying capacity and
    /// [`Self::colony_seed_for`] declines.
    fn founding_infra(&self, hull: HullType) -> Price {
        // **The recycled hull's minerals *are* the stock** (T-70), so this is a
        // price and no longer a Band — the clamp is to what the top playable
        // rung costs rather than to the rung's index. Same content, one ladder
        // reading fewer, and it lands on the ladder's own value rather than on
        // a Band that has to be converted back the moment it is spent against.
        let ceiling = infra_rung_price(BandTier::MAX_PLAYABLE.band().bands() as usize, &self.config);
        hull_cost(hull, &self.config).min(ceiling).max(Price::ZERO)
    }

    /// [`Self::founding_infra`] as a rung, for the surfaces that want the
    /// reading rather than the stock.
    #[inline]
    fn founding_infra_band(&self, hull: HullType) -> Band {
        self.founding_infra(hull).band_from(cost_anchor(&self.config))
    }

    /// **The carrying capacity a colony will have the moment it is founded** —
    /// the world's own, and nothing to do with the hull that got there (T-67).
    ///
    /// It used to take the hull, because `K` included infrastructure and
    /// founding set infrastructure to what the recycled hull bought
    /// ([`Self::founding_infra`]) — that dependence was the whole of R-O76.
    /// With infrastructure out of `K` there is no such dependence, and the
    /// parameter went with it.
    ///
    /// It is still the number a colony ship must not exceed: a population above
    /// `K` does not settle back to it, it *crashes* below it
    /// (`a_colony_seeded_above_its_capacity_crashes_below_it`). What changed is
    /// that the engine can no longer *cause* that by founding — the seed is
    /// capped here — so the crash is reachable only by an attack on
    /// habitability or biosphere, which is exactly the design intent.
    fn founding_capacity(&self, target: Entity) -> Kilotons {
        let f = self.world.factors.get(target).unwrap();
        // **The hull no longer caps this** (T-67). Infrastructure left `K`, so a
        // colony seeds to the *world's* own ceiling whatever hull founded it;
        // the recycled hull still lands as industrial stock, which is now its
        // whole job. R-O76's "seed depth does not pay" measured the mismatch
        // between what a hull carried and what the colony could hold, and there
        // is no mismatch left to measure.
        units::population_mass(f.k_potential())
    }

    /// **The founding population a colony ship of this hull carries** — the
    /// Band its hold masses (T-56 stage 4b), **capped at the target's
    /// carrying capacity.**
    ///
    /// This is what makes a heavier colonizer worth its price. The hold ladder
    /// is the mass ladder (§2.6, R-MC15), so a General hull's hold is a whole
    /// Band above a Medium's: the colony it founds *starts* a Band further up
    /// the population ladder and compounds from there, against a hull that cost
    /// `medium_fleet_size` times as much. Whether the head start beats the count
    /// is T-56's acceptance test, and it is measured, not assumed.
    ///
    /// **`None` is R-V9 expressed as physics.** "A Colonizer must be Medium or
    /// larger" used to be a rule about hull types; it is now a consequence — a
    /// Limited hull's hold sits at `Band Empty`, below `colony_seed_pop`, so it
    /// cannot carry a viable founding population. `colony_seed_pop` keeps its
    /// ratified value and changes job: from *the* seed to the **floor** a hull
    /// must clear to found anything.
    ///
    /// **Since T-67 the hull no longer sets the colony's `K`.** Infrastructure
    /// left the carrying-capacity minimum, so both viable hulls seed to the
    /// *world's* ceiling and the hold is the only thing that distinguishes
    /// them. R-O76's measured "seed depth does not pay" was about the mismatch
    /// between hold and ceiling; there is no mismatch left, and the hull-choice
    /// question is reopened as R-IND11.
    ///
    /// **R-O74 (new, open): these settlers are conjured, and stage 4b makes
    /// that 31× louder.** Nothing debits the founding center's population or
    /// biosphere for the people put aboard, which was already a design law #11
    /// violation at `Band I` and is a bigger one at `Band II`. It is recorded
    /// rather than fixed here because drawing the seed from the origin is a
    /// behaviour change that would dominate the measurement stage 4 exists to
    /// take — and mixing the two is exactly the confound this staging avoids.
    fn colony_seed_for(&self, hull: HullType, origin: Entity, target: Entity) -> Option<Kilotons> {
        // **R-V9 is enforced on the hold, and since T-67 it has to be.**
        //
        // It used to fall out of the infrastructure coupling: `founding_capacity`
        // was `k_potential.min(infra.max(founding_infra(hull)))`, a Limited
        // hull's `founding_infra` is `Band Empty`, and `population_mass` maps
        // that to *zero* — so the seed was zero and the hull was refused. That
        // was a coincidence of two unrelated rules, and removing infrastructure
        // from `K` dissolved it: a Limited hull would now found a colony of
        // 0.089 kt, quietly, in a build nobody changed on purpose.
        //
        // So the rule is stated where roles §4 says it lives — as **capability,
        // not competence**: a hull founds nothing unless its *hold* can carry a
        // full `colony_seed_pop`. A Medium hull's hold is `Band I` exactly and a
        // Limited hull's is `Band Empty`, so the ratified rule and the geometry
        // now agree without either propping the other up.
        let floor = units::population_mass(self.config.colony_seed_pop.band());
        let hold = hull.colony_seed_capacity(&self.config);
        if hold < floor * (1.0 - 1e-9) {
            return None;
        }
        // What actually flies is the least of three things, and the third is the
        // one R-O74 was missing. The **world** caps it, because a seed above `K`
        // would crash rather than settle. The **hold** caps it, because that is
        // capability. And the **origin** decides it — not as a share of what it
        // has, but as the split that maximises the growth of the combined
        // origin-plus-colony system under a travel discount (R-IND12; see
        // `settler_target`, which also defines every symbol it uses).
        Some(hold.min(self.founding_capacity(target)).min(self.settler_target(origin, target, hold)))
    }

    /// **How many settlers a centre sends, and where the number comes from
    /// (R-IND12).**
    ///
    /// Every symbol, before any of them is used:
    ///
    /// | symbol | meaning | unit |
    /// |---|---|---|
    /// | `x_p` | the origin's current population | kt |
    /// | `K_p` | the origin's carrying capacity, `population_mass(k())` | kt |
    /// | `K_c` | the target's carrying capacity, same function | kt |
    /// | `x_0` | the floor a colony would otherwise start from, [`units::POPULATION_SEED_FLOOR`] | kt |
    /// | `r` | logistic growth rate per cycle, `Doctrine::growth_rate` | 1/cycle |
    /// | `tau` | one-way transit, origin → target, at civilian accel | yr |
    /// | `delta` | discount on a gain that arrives `tau` late | — |
    /// | `S` | settlers put aboard | kt |
    /// | `g(x, K)` | the logistic rate `r·x·(1 − x/K)` — the engine's own step | kt/cycle |
    ///
    /// **The sizing is demand-side, not supply-side.** An earlier version shipped
    /// a *fraction of the parent* and the author's ruling retired it: a fraction
    /// is irrelevant. What decides the amount is `K` against the hold, the
    /// build-out the destination will pay for (see [`Self::endowment_minerals`]),
    /// and the growth of the **combined** origin-plus-colony system under a
    /// travel discount.
    ///
    /// **Total growth is not what varies — timing is.** Both worlds reach their
    /// own ceiling eventually whatever is shipped, so "the total growth of the
    /// combined system" is a statement about *when*, which is exactly what the
    /// colony-years guard measures. So price the seed in time:
    ///
    /// ```text
    /// value(S) = T_c(S) = ln[ (S / (K_c − S)) · ((K_c − x_0) / x_0) ] / r
    ///            -- the time the seed saves the child, floor -> S
    /// cost(S)  = T_p(S) = S / g(x_p − S, K_p)
    ///            -- the time the origin needs to regrow what it gave away
    /// maximise   delta · T_c(S) − T_p(S)
    /// ```
    ///
    /// **`r` cancels.** Both terms carry `1/r`, so the split is independent of
    /// `growth_rate` — which matters, because that knob is separately ratified
    /// (R-O84) and a policy that moved with it would couple two things that were
    /// measured apart.
    ///
    /// **Two rival formulations were tried and rejected on measurement**, which
    /// is the reason this one is written out rather than asserted:
    ///
    /// - *Marginal next-cycle rate*, `g(x_p − S) + delta·g(S)`: closed form, and
    ///   it ships **nothing at all** whenever the origin is below `K_p/2` and the
    ///   target is far. That is precisely the early game, when colonising matters
    ///   most, so it would have stalled expansion by construction.
    /// - *Sum of fill times*: strips a full origin to 99% of itself at zero
    ///   distance, and ships nothing from one at its ceiling when the target is
    ///   far — wrong in both directions at the same operating point.
    ///
    /// **The optimum is found on a fixed grid, not by root-finding**, because
    /// the objective is **not unimodal**: sampled over 4,000 random
    /// configurations it has more than one turning point in ~4.6% of them. A
    /// bisection or golden-section search would silently return a local optimum
    /// in those, and it would do it deterministically, which is the worst kind of
    /// wrong — reproducible and invisible. [`Self::ENDOWMENT_GRID`] evaluations
    /// per launch is a few thousand per run, against a growth step that runs per
    /// planet per cycle; this is not a hot path.
    ///
    /// **The discount is `delta = 1 / (1 + tau / cycle_years)`** — a gain landing
    /// `n` production cycles late is worth `1/(1+n)` of one landing now.
    /// Hyperbolic rather than exponential, and chosen because it needs **no new
    /// constant**: `cycle_years` already exists and is the natural clock for a
    /// quantity denominated per cycle. Whether the form should be exponential is
    /// **R-IND14**, open.
    fn settler_target(&self, center: Entity, target: Entity, hold: Kilotons) -> Kilotons {
        let x_p = self.world.population.get(center).copied().unwrap_or(Kilotons::ZERO);
        let k_p = self.capacity_of(center);
        let k_c = self.capacity_of(target);
        let x_0 = units::POPULATION_SEED_FLOOR;
        let hi = hold.min(k_c).min((x_p - x_0).max(Kilotons::ZERO));
        if hi <= Kilotons::ZERO || k_p <= Kilotons::ZERO || k_c <= x_0 {
            return Kilotons::ZERO;
        }
        // **A destination-limited seed fills the world, and that is the model
        // rather than an exception to it.** `T_c(S)` is `ln(S/(K_c − S) · …)`,
        // which diverges as `S` approaches `K_c`: a colony landed at its own
        // ceiling has no growth left to wait through, so the time it saves is
        // unbounded. The grid below cannot represent its own endpoint — the
        // top point would be an infinity — so the case is taken here, where it
        // is legible, instead of being lost as a 1/32 shortfall nobody notices.
        if hi >= k_c * (1.0 - 1e-12) {
            return hi.min(k_c);
        }
        let delta = self.travel_discount(center, target);
        let (xp, kp, kc, x0) = (x_p.kilotons(), k_p.kilotons(), k_c.kilotons(), x_0.kilotons());
        let head = (kc - x0) / x0;

        let mut best = (f64::NEG_INFINITY, 0.0);
        for i in 1..=Self::ENDOWMENT_GRID {
            let s = hi.kilotons() * (i as f64) / (Self::ENDOWMENT_GRID as f64);
            if s >= kc || s >= xp {
                break;
            }
            // Time the seed saves the child: floor -> S on its own logistic.
            let t_c = ((s / (kc - s)) * head).ln();
            // Time the origin needs to regrow it. An origin already at or over
            // its ceiling has no headroom to regrow *into*, and its people are
            // surplus rather than growth — so the gift costs it nothing.
            let left = xp - s;
            let rate = left * (1.0 - left / kp);
            let t_p = if rate > 1e-15 { s / rate } else { 0.0 };
            let score = delta * t_c - t_p;
            if score > best.0 {
                best = (score, s);
            }
        }
        Kilotons::new(best.1).min(hi)
    }

    /// Grid resolution for [`Self::settler_target`]. Stated as a constant
    /// because it is part of the answer: the objective is not unimodal, so the
    /// split is the best of this many candidates rather than a solved optimum,
    /// and changing it changes results.
    const ENDOWMENT_GRID: usize = 32;

    /// A planet's carrying capacity as a **mass** — `population_mass(k())`,
    /// which since T-67 is `min(hab, bio_max)` and has no infrastructure term.
    fn capacity_of(&self, planet: Entity) -> Kilotons {
        self.world.factors.get(planet).map(|f| units::population_mass(f.k())).unwrap_or(Kilotons::ZERO)
    }

    /// `delta = 1 / (1 + tau / cycle_years)` — see [`Self::settler_target`] for
    /// what the symbols are and why the form is hyperbolic.
    fn travel_discount(&self, center: Entity, target: Entity) -> f64 {
        let (a, b) = (self.world.position.get(center), self.world.position.get(target));
        let tau = match (a, b) {
            (Some(&from), Some(&to)) => {
                let d = from.distance(to);
                if d > 0.0 {
                    math::ship_travel_years(d, self.config.civilian_accel_g * G)
                } else {
                    0.0
                }
            }
            _ => 0.0,
        };
        1.0 / (1.0 + tau / self.config.cycle_years.max(1e-9))
    }

    /// **The minerals a centre sends with the settlers — sized by what the
    /// destination will build, not by what the origin happens to hold.**
    ///
    /// Terms:
    ///
    /// | symbol | meaning | unit |
    /// |---|---|---|
    /// | `H` | the hull's hold capacity | kt |
    /// | `S` | settlers already loaded | kt |
    /// | `I_0` | founding infrastructure, the recycled hull | Band |
    /// | `I_star` | the build-out the site is worth developing to | Band |
    /// | `C(I_0 → I_star)` | cumulative rung price over that range | kt (as `Price`) |
    /// | `K_c` | the target's carrying capacity | Band |
    /// | `D` | the target's mineral abundance | Band |
    ///
    /// A hold is a volume, not a passenger list, and the author's ruling is that
    /// it may carry a mix: *"the cargo hold needn't be filled with space or pop.
    /// It can hold a combination of pop and minerals to jumpstart production."*
    /// The settlers are capped by `K_c`, so the leftover volume is real and the
    /// question is only what it is worth filling with.
    ///
    /// **The demand is the intended build-out**, `E = min(H − S, C(I_0 →
    /// I_star), bank)`. Sending more than the destination will spend is freight
    /// for ore that then sits in a stockpile; sending less means it waits on a
    /// freighter for something the coloniser had room for.
    ///
    /// **`I_star` is where works value enters, and works are superadditive in
    /// `K` and `D`** (§5, author's ruling: *"Works have higher value on high K
    /// world and high mineral density worlds, and the highest on the
    /// combination"*). A product has exactly that property — a positive cross
    /// partial — and a product of masses is a **sum on the Band ladder**, so the
    /// placeholder is the midpoint `I_star = (K_c + D) / 2`, i.e. the geometric
    /// mean of the two masses.
    ///
    /// **Placeholder, and flagged as one: R-IND13.** The real works-value
    /// function is §5's and needs T-73/T-74; this is the cheapest form with the
    /// right cross partial, not a claim about magnitudes.
    fn endowment_minerals(&self, center: Entity, hull: HullType, settlers: Kilotons) -> Price {
        let spare = (hull.colony_seed_capacity(&self.config) - settlers).max(Kilotons::ZERO);
        if spare <= Kilotons::ZERO {
            return Price::ZERO;
        }
        let bank = self.world.stockpile.get(center).map(|s| s.basic_total()).unwrap_or(Price::ZERO);
        let demand = self.build_out_price(center, hull);
        // A hold is a mass and a bank is a price; the same kilotons read on two
        // ladders (`units::Qty::on_scale`), crossed explicitly rather than by
        // arithmetic that happens to typecheck.
        spare.on_scale::<units::Cost>().min(demand).min(bank).max(Price::ZERO)
    }

    /// `C(I_0 → I_star)` — what the destination's intended build-out costs, as
    /// the sum of the infrastructure rungs between the recycled hull's rung and
    /// the works-value rung. See [`Self::endowment_minerals`] for the symbols.
    fn build_out_price(&self, target: Entity, hull: HullType) -> Price {
        let Some(f) = self.world.factors.get(target) else {
            return Price::ZERO;
        };
        let density = self.world.density.get(target).map(|d| d.abundance()).unwrap_or(Band::ZERO);
        // The works rung: geometric mean of capacity and abundance, which on the
        // Band ladder is their midpoint (R-IND13, placeholder).
        let i_star = Band::new((f.k().bands() + density.bands()) * 0.5);
        let from = infra_rung_of(self.founding_infra(hull), &self.config);
        let to = i_star.round().bands().max(0.0) as usize;
        let mut total = Price::ZERO;
        for rung in (from + 1)..=to {
            total += infra_rung_price(rung, &self.config);
        }
        total
    }

    fn mark_targeted(&mut self, p: usize, target: PlanetId) {
        self.world.knowledge.get_mut(self.player_entity[p]).unwrap().targeted.insert(target);
    }

    /// Spawn a colony/mining courier flying `center → target`.
    ///
    /// Takes the **hull that was ordered**, not `role_hull_type(role)`. The two
    /// agreed until Doctrine could choose a colonizer's hull; reading the hull
    /// back off the role would have flown a Medium ship on a General ship's
    /// bill, and nothing would have complained.
    fn spawn_courier(
        &mut self,
        p: usize,
        role: Role,
        hull: HullType,
        center: Entity,
        target: Entity,
        launch_delay: f64,
    ) {
        // The launch point *is* the centre — it was passed in alongside it until
        // T-68 needed a seventh argument, and the two were always the same read.
        let from = *self.world.position.get(center).unwrap();
        let dest = *self.world.position.get(target).unwrap();
        let accel = self.config.civilian_accel_g * G;
        let e = self.world.spawn();
        self.world.owner.insert(e, PlayerId(p as u32));
        self.world.role.insert(e, role);
        self.world.hull_type.insert(e, hull);
        self.world.voyage.insert(e, Voyage { target, heading_bias: None, hops: 0 });
        // A Colonizer carries its founding population as cargo, consumed on
        // arrival (`Hyades_vehicle_roles.md` §4.2), and **how much is what the
        // hull's hold masses** (T-56 stage 4b) rather than a flat constant.
        //
        // **Both halves of the hold are loaded out of the origin** (R-O74
        // closed): the settlers are debited from `center`'s population and the
        // minerals from its stockpile, so a launch moves mass rather than
        // creating it. Load before the ship exists as far as the books are
        // concerned — the debit and the credit are the same statement.
        let (settlers, endowment) = if role == Role::Colonizer {
            let settlers = self.colony_seed_for(hull, center, target).unwrap_or(Kilotons::ZERO);
            let minerals = self.endowment_minerals(center, hull, settlers);
            if settlers > Kilotons::ZERO {
                let pop = self.world.population.get_mut(center).unwrap();
                *pop = (*pop - settlers).max(Kilotons::ZERO);
            }
            let loaded =
                self.world.stockpile.get_mut(center).map(|bank| take_basics(bank, minerals)).unwrap_or_default();
            (settlers, loaded)
        } else {
            (Kilotons::ZERO, Minerals::default())
        };
        self.world.cargo.insert(e, endowment);
        self.world.pop_cargo.insert(e, settlers);
        self.world.home_center.insert(e, center);
        let arrive = self.set_leg(e, from, dest, accel, launch_delay);
        let ev = match role {
            Role::Colonizer => EventKind::ColonyArrive { vehicle: e },
            _ => EventKind::MiningArrive { vehicle: e },
        };
        self.schedule_at(arrive, ev);
        let target_pid = *self.world.planet_id.get(target).unwrap();
        self.log.push(
            self.clock,
            LogEvent::VehicleSpawned {
                player: p as u32,
                vehicle: e,
                role,
                from,
                to: target_pid,
                settlers: settlers.kilotons(),
                endowment: endowment.basic_total().kilotons(),
            },
        );
    }

    fn spawn_freighter(&mut self, p: usize, center: Entity, from: Vec3, outpost: Entity, launch_delay: f64) {
        let dest = *self.world.position.get(outpost).unwrap();
        let accel = self.config.civilian_accel_g * G;
        let e = self.world.spawn();
        self.world.owner.insert(e, PlayerId(p as u32));
        self.world.role.insert(e, Role::Freighter);
        self.world.hull_type.insert(e, role_hull_type(Role::Freighter));
        self.world.cargo.insert(e, Minerals::default());
        self.world.home_center.insert(e, center);
        self.world.shuttle.insert(e, Shuttle { outpost, destination: center, outbound: true });
        let arrive = self.set_leg(e, from, dest, accel, launch_delay);
        self.schedule_at(arrive, EventKind::FreighterArrive { vehicle: e });
        let outpost_pid = *self.world.planet_id.get(outpost).unwrap();
        self.log.push(
            self.clock,
            LogEvent::VehicleSpawned {
                player: p as u32,
                vehicle: e,
                role: Role::Freighter,
                from,
                to: outpost_pid,
                settlers: 0.0,
                endowment: 0.0,
            },
        );
    }

    fn launch_survey(&mut self, p: usize, from: Vec3, heading: Vec3, hops: usize, launch_delay: f64) {
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
            let arrive = self.set_leg(e, from, dest, accel, launch_delay);
            self.schedule_at(arrive, EventKind::ContactArrive { vehicle: e });
            self.log.push(
                self.clock,
                LogEvent::VehicleSpawned {
                    player: p as u32,
                    vehicle: e,
                    role: Role::Scout,
                    from,
                    to: target_pid,
                    settlers: 0.0,
                    endowment: 0.0,
                },
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
        // **Three masses, and each reading is taken explicitly.** Acceleration
        // is `thrust / mass`, so every term below has to be kilotons or the
        // ratio is meaningless — the defect R-O66 found in `K` one module over,
        // in the one place design law #10 makes observable.
        //
        // Minerals convert 1:1 because L6/R-O57 made cost and dry mass one
        // number: a mineral in the hold masses exactly what it massed as hull.
        // That is an identity, not a coefficient, which is why there is no
        // `cargo_mass_per_unit` any more.
        // Cargo in a hold is a mass; the same minerals in a bank are a price.
        let minerals =
            self.world.cargo.get(e).map(|m| m.basic_total()).unwrap_or(Price::ZERO).on_scale::<units::Mass>();
        let pop = self.world.pop_cargo.get(e).copied().unwrap_or(Kilotons::ZERO);
        let hull = self.world.hull_type.get(e).copied().unwrap_or(HullType::MediumSystems);
        let dry = hull_dry_mass(hull, &self.config).max(Kilotons::new(1e-9));
        let laden = dry + minerals + pop;
        base_g * G * (dry.kilotons() / laden.kilotons())
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
            // Inferential tier (R-SIM3): a pop-Band-IV world radiates the waste heat of
            // billions, which spectrometry reads across interstellar distance.
            // Reported unconditionally; `Doctrine::survey_avoids_inhabited`
            // decides whether the policy acts on it, and defaults to off.
            let pop = *self.world.population.get(e).unwrap();
            let industrial_signature = self.bands.level(pop) >= BandTier::IV;
            out.push(SurveyView {
                id: pid,
                position: *self.world.position.get(e).unwrap(),
                habitability: f.hab,
                biosphere: f.bio_max.in_bands(),
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
            biosphere: f.bio_max.in_bands(),
            minerals: *self.world.density.get(e).unwrap(),
            owner: self.world.owner.get(e).copied(),
            pop_level: self.bands.level(*self.world.population.get(e).unwrap()),
        }
    }

    /// Centroid of player `p`'s holdings — **memoised, because it only moves
    /// when a colony is founded** (R-O70).
    ///
    /// The computation is a full walk of `planet_entity`, and it ran once per
    /// production decision: profiled at **1.14 billion iterations** on seed 1
    /// at the shipped horizon, the largest single scan in the engine. Holdings
    /// change roughly 3,400 times in that run, so all but ~0.003% of that work
    /// was recomputing an answer nothing had invalidated.
    ///
    /// **The cache stores the recomputed value rather than a running sum**, and
    /// that is deliberate. An incremental sum would accumulate in *claim* order
    /// while this walks in *planet-id* order, and floating-point addition is
    /// not associative — the centroid would differ in the last bits, every
    /// score with it, and the run would diverge. Recomputing on the same path
    /// keeps the result bit-identical; only the number of recomputations
    /// changes.
    fn holdings_centroid(&mut self, p: usize) -> Vec3 {
        if let Some(c) = self.centroid_cache[p] {
            // A stale cache is a silent, seed-dependent divergence, so debug
            // builds pay for the full recompute and compare. Free in release.
            debug_assert_eq!(
                c,
                self.compute_holdings_centroid(p),
                "stale holdings centroid for player {p}: planet ownership was written outside `claim_planet`"
            );
            return c;
        }
        let c = self.compute_holdings_centroid(p);
        self.centroid_cache[p] = Some(c);
        c
    }

    fn compute_holdings_centroid(&self, p: usize) -> Vec3 {
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

    /// **The only sanctioned way to give a planet an owner.**
    ///
    /// Ownership is what moves [`Self::holdings_centroid`], so the cache is
    /// invalidated here and nowhere else. Writing `world.owner` directly for a
    /// *planet* leaves every subsequent rank reading a stale centre of mass —
    /// silently, and only on some seeds. The `debug_assert` in
    /// `holdings_centroid`'s caller-facing test build catches exactly that.
    fn claim_planet(&mut self, planet: Entity, owner: PlayerId) {
        self.world.owner.insert(planet, owner);
        self.centroid_cache[owner.0 as usize] = None;
    }

    /// **What a centre will pay for one kilotonne of a colour, in `$`/kt**
    /// (politics §3.2, T-84).
    ///
    /// ```text
    /// wtp = base_value[c] · doctrine_demand[c] · mineral_pressure(centre)
    /// ```
    ///
    /// The fourth term §3.2 names — `risk_discount(counterparty)` — is not here
    /// because it is a property of *whom you are trading with*, not of what you
    /// want, so it belongs at match time and needs reputation (T-86).
    ///
    /// Every term is already ratified or already exists: `mineral_pressure_of`
    /// is the engine's, and the two Doctrine fields default to the works mix
    /// (`Hyades_industry.md` §6.10) rather than to a second independent
    /// statement of what an empire wants.
    fn willingness_to_pay(&self, center: Entity, colour: Basic, doctrine: &Doctrine) -> f64 {
        let i = colour as usize;
        doctrine.base_value[i] * doctrine.doctrine_demand[i] * self.mineral_pressure_of(center)
    }

    /// **Post every empire's bids and asks to the cross-empire books** (T-84).
    ///
    /// Runs at the round barrier and rebuilds the books from scratch, because
    /// §10.5 clears a **set** rather than a stream: a book carried across rounds
    /// would make a stale offer's position in it depend on when it was posted,
    /// which is the event-ordering dependence per-round clearing exists to
    /// avoid.
    ///
    /// **A centre bids for what it is short of and asks with what it is long
    /// of**, both read off the same place — its own bank against its own next
    /// works bill. That is what makes this move Yellow from the Yellow-rich to
    /// the Yellow-poor rather than shuffling mass at random: T-73 measured
    /// 1,494 of 1,515 banks single-coloured, so nearly every centre is
    /// simultaneously long one colour and short the other two.
    ///
    /// **Nothing clears yet.** This stage is inert by construction (§10.7
    /// stages 1–3) and the guard is a bit-identical bed.
    fn post_exchange_offers(&mut self) {
        for b in self.exchange.books.iter_mut() {
            *b = matching::Book::new();
        }
        let players = self.player_entity.len();
        for p in 0..players {
            let pe = self.player_entity[p];
            let doctrine = *self.world.doctrine.get(pe).unwrap();
            let owner = PlayerId(p as u32);
            // Planet-id order, which is the engine's canonical iteration order
            // and the only one every client agrees on before the round exists.
            for &e in &self.planet_entity {
                if self.world.owner.get(e).copied() != Some(owner) {
                    continue;
                }
                let deficit = self.colour_deficit(e, owner);
                let bank = self.world.stockpile.get(e).copied().unwrap_or_default();
                let pos = *self.world.position.get(e).unwrap();
                let at = [pos.x, pos.y, pos.z];
                for (i, &c) in Basic::ALL.iter().enumerate() {
                    let short = deficit[i].kilotons();
                    if short > 1e-9 {
                        let price = self.willingness_to_pay(e, c, &doctrine);
                        if price > 0.0 {
                            self.exchange.posted[i].0 += 1;
                            self.exchange.books[i].post_bid(matching::Offer {
                                entity: e.0,
                                price,
                                qty: short,
                                pos: at,
                                owner,
                            });
                        }
                    } else {
                        // Long: whatever this bill does not claim is sellable.
                        // A centre with no bill to pay is long everything it
                        // holds, which is exactly the Yellow-rich empire the
                        // Yellow-poor one needs to reach.
                        let spare = bank.get_basic(c);
                        if spare > 1e-9 {
                            self.exchange.posted[i].1 += 1;
                            self.exchange.books[i].post_ask(matching::Offer {
                                entity: e.0,
                                price: self.willingness_to_pay(e, c, &doctrine),
                                qty: spare,
                                pos: at,
                                owner,
                            });
                        }
                    }
                }
            }
        }
    }

    /// **The outposts an empire has crew standing on**, in outpost-id order
    /// (T-85). The candidate venues it can settle a trade at.
    fn worked_outposts(&self, p: PlayerId) -> Vec<u64> {
        self.mine_crew
            .range((p.0, 0u64)..=(p.0, u64::MAX))
            .filter(|(_, crew)| !crew.is_empty())
            .map(|((_, o), _)| *o)
            .collect()
    }

    /// **Where two empires can hand goods over** — a rock they both work
    /// (`Hyades_politics_trade_and_intelligence.md` §10.6, T-85).
    ///
    /// **R-P17, decided — and not as it was first framed.** The venue is the
    /// shared rock nearest **the party making this delivery**, because a
    /// contract has *two* drops and each side ships its own (see
    /// [`Contract::buyer_drop`]). The first formulation minimised the two
    /// parties' *summed* transit, which is the right answer only if there is one
    /// venue for both legs — and there is not.
    ///
    /// `None` means these two empires cannot trade at all right now, and that is
    /// the mechanic rather than a failure: **geography is the trade
    /// constraint.** An empire with no rock in common with anyone is landlocked,
    /// and reaching one is a reason to go somewhere.
    fn shared_venue(&self, a: PlayerId, b: PlayerId, ship_from: Vec3) -> Option<Entity> {
        let (mine, theirs) = (self.worked_outposts(a), self.worked_outposts(b));
        let mut best: Option<(f64, u64)> = None;
        // Both lists are id-sorted, so this is a linear merge rather than a
        // nested scan — a centre can work thousands of rocks (§4.5).
        let (mut i, mut j) = (0usize, 0usize);
        while i < mine.len() && j < theirs.len() {
            match mine[i].cmp(&theirs[j]) {
                Ordering::Less => i += 1,
                Ordering::Greater => j += 1,
                Ordering::Equal => {
                    let e = Entity(mine[i]);
                    if let Some(&at) = self.world.position.get(e) {
                        let cost = ship_from.distance(at);
                        // Id breaks ties, so the choice is total and does not
                        // depend on which party is named first.
                        if best.is_none_or(|(c, o)| cost < c || (cost == c && mine[i] < o)) {
                            best = Some((cost, mine[i]));
                        }
                    }
                    i += 1;
                    j += 1;
                }
            }
        }
        best.map(|(_, o)| Entity(o))
    }

    /// **Clear every colour book into escrowed contracts** (§10.5, T-85).
    ///
    /// The first stage of the Exchange that is **not** inert, and §10.7 says to
    /// expect it to move the bed rather than assume it will not — it is the same
    /// shape as the industry stage that broke §6.7's neutrality plan, because it
    /// changes what a purchase costs.
    ///
    /// Three filters between a `Fill` and a `Contract`, and each one is a design
    /// statement rather than a guard:
    ///
    /// - **No self-trades.** An empire filling its own ask is a no-op that would
    ///   burn `$` and move nothing. §4's Corner — buying a mineral you have no
    ///   use for so a rival cannot have it — is a *different* play and stays
    ///   legal, because it has a real counterparty.
    /// - **A shared venue, or no trade.** §10.6: goods change hands at a rock
    ///   both parties work. This is what makes trade geographic.
    /// - **What the purse can actually pay.** Escrow is locked at match, so a
    ///   bid beyond the ledger is trimmed to what it can fund rather than
    ///   creating a debt nobody agreed to.
    fn clear_exchange(&mut self) {
        let now = self.clock;
        let mut fills: Vec<(usize, matching::Fill)> = Vec::new();
        for (i, book) in self.exchange.books.iter_mut().enumerate() {
            for f in book.match_wave_cross_empire() {
                fills.push((i, f));
            }
        }
        for (i, f) in fills {
            self.exchange.fills += 1;
            if f.buyer == f.seller {
                self.exchange.rejected[0] += 1;
                continue;
            }
            let (bid_e, ask_e) = (Entity(f.bid), Entity(f.ask));
            let (Some(&b_at), Some(&s_at)) = (self.world.position.get(bid_e), self.world.position.get(ask_e)) else {
                continue;
            };
            // The seller ships, so the seller picks its own drop.
            let Some(seller_drop) = self.shared_venue(f.buyer, f.seller, s_at) else {
                self.exchange.rejected[1] += 1;
                continue; // landlocked with respect to each other this round
            };
            // The buyer's side is `$` in the default transaction, and `$` has no
            // location. A goods counter-leg is the buyer's own contract on
            // another colour's book, with its own drop — which is how "leave
            // Yellow at X in exchange for Magenta at Y" is expressed.
            let buyer_drop = None;
            let _ = b_at;

            // **The price the fill cleared at**, carried on the fill rather
            // than looked up — a wave drops exhausted offers, so a bid that
            // matched in full is already gone from the book.
            let price = f.price;
            let purse = self.purse_of(f.buyer);
            if price <= 0.0 {
                self.exchange.rejected[2] += 1;
                continue;
            }
            if purse <= 0.0 {
                self.exchange.rejected[3] += 1;
                continue;
            }
            let qty = f.qty.min(purse / price);
            let escrow = qty * price;
            if qty <= 1e-9 || !escrow.is_finite() {
                continue;
            }

            let pe = self.player_entity[f.buyer.0 as usize];
            self.credit(pe, -escrow);

            // **The freight leg.** The obligation was instant; the ore is not.
            // Transit is the seller's centre to its own drop, at civilian
            // acceleration — the same `ship_travel_years` every other voyage in
            // the engine uses, so a trade is priced in the same geometry as a
            // colonisation or a haul (§8.1: a trade is a voyage).
            let drop_at = *self.world.position.get(seller_drop).unwrap();
            let t = math::ship_travel_years(s_at.distance(drop_at), self.config.civilian_accel_g * G);
            let id = self.exchange.next_id;
            self.exchange.next_id += 1;
            self.exchange.contracts.insert(
                id,
                Contract {
                    buyer: f.buyer,
                    seller: f.seller,
                    seller_centre: ask_e,
                    colour: Basic::ALL[i],
                    qty,
                    escrow,
                    seller_drop,
                    buyer_drop,
                    struck: now,
                },
            );
            if self.exchange_settlement {
                self.schedule(t, EventKind::ContractDue { id });
            }
        }
    }

    /// **A contract's freight arrives at its drop** (§3.3, §10.6, T-77).
    ///
    /// The ore leaves the seller's bank and lands in the **buyer's pile at the
    /// shared outpost** — not at the buyer's world. The buyer's own freighter
    /// collects it on the need-based route it was already flying, which is why
    /// this needs no cross-empire hull and why a foreign hull near a colony
    /// stays a card rather than the default.
    ///
    /// **Mass is conserved** (design law #11): every kilotonne debited from the
    /// seller is credited to the buyer, and the `$` burn is not mass — `$` was
    /// never in the mass ledger (R-P1).
    ///
    /// **Both outcomes share the burn, which is what §2.3's sink is.**
    ///
    /// | | seller gets | buyer gets | burned |
    /// |---|---|---|---|
    /// | delivered | `E · exp(−λ·t)` | the ore | `E · (1 − exp(−λ·t))` |
    /// | **defaulted** | nothing | `E · exp(−λ·t)` | the same |
    ///
    /// §3.3: *"the buyer loses the burn, the seller loses the cargo, the loss is
    /// shared"* — **which is what makes escorting worth paying for** (R-IND10,
    /// resolved there). A default is not a free option for the seller: it keeps
    /// nothing and forfeits the sale.
    ///
    /// Default here is *inability*, not malice — the bank is short because the
    /// centre spent the ore, or lost the world. Deliberate default is a card
    /// (§7.2), and interdiction is T-86.
    fn sys_contract_due(&mut self, id: u64) {
        let Some(c) = self.exchange.contracts.remove(&id) else {
            return; // already settled or cancelled
        };
        let t = (self.clock - c.struck).max(0.0);
        let keep = (-self.config.trade_decay_lambda * t).exp();
        let paid = c.escrow * keep;
        let burn = c.escrow - paid;

        // Does the seller still have it? `get_basic` is the colour the contract
        // names, not the bank total — a centre rich in Cyan cannot settle a
        // Yellow contract, which is the whole point of the colour axis.
        let held = self.world.stockpile.get(c.seller_centre).map_or(0.0, |b| b.get_basic(c.colour));
        let delivered = held + 1e-9 >= c.qty && self.world.owner.get(c.seller_centre).copied() == Some(c.seller);

        if delivered {
            if let Some(bank) = self.world.stockpile.get_mut(c.seller_centre) {
                match c.colour {
                    Basic::Cyan => bank.cyan -= c.qty,
                    Basic::Magenta => bank.magenta -= c.qty,
                    Basic::Yellow => bank.yellow -= c.qty,
                }
            }
            // **Into the buyer's own pile at that rock** — `outpost_stock` is
            // already keyed `(player, outpost)` and is already what a laden
            // freighter loads from, so the collection leg needs no new code at
            // all. The buyer's hauler picks the ore up on the route it was
            // flying anyway, which is the amendment's whole claim made literal.
            let pile = self.outpost_stock.entry((c.buyer.0, c.seller_drop.0)).or_default();
            match c.colour {
                Basic::Cyan => pile.cyan += c.qty,
                Basic::Magenta => pile.magenta += c.qty,
                Basic::Yellow => pile.yellow += c.qty,
            }
            let se = self.player_entity[c.seller.0 as usize];
            self.credit(se, paid);
            self.exchange.settled += 1;
            self.exchange.traded[c.colour as usize] += c.qty;
        } else {
            let be = self.player_entity[c.buyer.0 as usize];
            self.credit(be, paid);
            self.exchange.defaulted += 1;
        }
        self.exchange.burned += burn;
    }

    /// Contracts in flight, and what the Exchange has settled and burned.
    /// Diagnostic — §10.8's guard is a census, not a scalar.
    pub fn exchange_state(&self) -> (usize, u64, f64) {
        (self.exchange.contracts.len(), self.exchange.settled, self.exchange.burned)
    }

    /// Contracts that **defaulted** — the seller's bank was short the colour it
    /// owed, or it had lost the world, when the freight came due (§3.3).
    pub fn exchange_defaults(&self) -> u64 {
        self.exchange.defaulted
    }

    /// Turn the whole Exchange off — posting, clearing and settlement (T-77).
    ///
    /// The ablation `CLAUDE.md` §2 puts first among the three kinds of proof:
    /// "trade narrowed colour dispersion" is only a claim if there is a run
    /// without trade to compare against.
    pub fn set_exchange_enabled(&mut self, on: bool) {
        self.exchange_posting = on;
        self.exchange_settlement = on;
    }

    /// Kilotons delivered per colour, in `Basic::ALL` order.
    pub fn exchange_traded(&self) -> [f64; 3] {
        self.exchange.traded
    }

    /// **An empire's ore waiting at outposts**, summed over every pile it holds.
    ///
    /// Delivered ore lands here, not in a bank (§10.6) — so a census that reads
    /// only planet stockpiles **cannot see what the Exchange moved**. That is
    /// the metric-blindness `CLAUDE.md` §2 keeps warning about, and it made the
    /// first colour-flow reading look like trade changed nothing.
    pub fn outpost_holdings(&self, p: PlayerId) -> Minerals {
        let mut out = Minerals::default();
        for ((owner, _), m) in self.outpost_stock.iter() {
            if *owner == p.0 {
                out.add_basics(m);
            }
        }
        out
    }

    /// Fills produced, and why each rejected one was: `(self-trade, no venue,
    /// no price, no purse)`.
    pub fn exchange_rejections(&self) -> (u64, [u64; 4]) {
        (self.exchange.fills, self.exchange.rejected)
    }

    /// Cumulative `(bids, asks)` posted per colour, in `Basic::ALL` order.
    pub fn exchange_posted(&self) -> [(u64, u64); 3] {
        self.exchange.posted
    }

    /// **Who is contracted to move what, and where** — the Exchange census
    /// (§10.8, T-85).
    ///
    /// Returns one row per contract in flight: `(buyer, seller, colour,
    /// kilotons, escrow, seller_drop, buyer_drop, struck)`. §10.8 is explicit
    /// that the guard for Exchange work is a **census** and not colony-years,
    /// because colony-years is inverted for anything that changes how minerals
    /// are spent — so the state has to be readable, not just summarised.
    ///
    /// It is also what answers the design's actual question: **does Yellow move
    /// from Yellow-rich empires to Yellow-poor ones?** That is a claim about
    /// direction and counterparties, and no scalar carries it.
    #[allow(clippy::type_complexity)]
    pub fn exchange_contracts(&self) -> Vec<(PlayerId, PlayerId, Basic, f64, f64, PlanetId, Option<PlanetId>, f64)> {
        self.exchange
            .contracts
            .values()
            .map(|c| {
                let drop = *self.world.planet_id.get(c.seller_drop).unwrap();
                let pay = c.buyer_drop.and_then(|e| self.world.planet_id.get(e).copied());
                (c.buyer, c.seller, c.colour, c.qty, c.escrow, drop, pay, c.struck)
            })
            .collect()
    }

    /// How many offers stand on each colour's book. Diagnostic — the interim
    /// guard for Exchange work is a census, not colony-years (§10.8).
    pub fn exchange_depth(&self) -> [(usize, usize); 3] {
        [self.exchange.books[0].len(), self.exchange.books[1].len(), self.exchange.books[2].len()]
    }

    /// **Add to an empire's `$` ledger** — the only writer (T-82).
    ///
    /// Refuses a non-finite delta rather than propagating it. Design law #16 is
    /// explicit that a NaN in replicated state is a *fatal value*, not a small
    /// one: core WASM picks NaN payloads nondeterministically, so a NaN that
    /// reaches the hashed state is an intermittent desync with no reproducer.
    /// The purse is replicated, so the guard belongs at the write and not at
    /// the digest.
    ///
    /// A negative balance is *not* refused — debt is a legitimate state for a
    /// claim, and §2.1 is explicit that `$` is an obligation rather than a
    /// substance. What is refused is a value that is not a number.
    fn credit(&mut self, player: Entity, delta: f64) {
        if !delta.is_finite() {
            return;
        }
        if let Some(p) = self.world.purse.get_mut(player) {
            let next = *p + delta;
            if next.is_finite() {
                *p = next;
            }
        }
    }

    /// An empire's `$` balance. Presentation-readable; the simulation reads it
    /// only through the Exchange.
    pub fn purse_of(&self, player: PlayerId) -> f64 {
        self.player_entity.get(player.0 as usize).and_then(|&e| self.world.purse.get(e)).copied().unwrap_or(0.0)
    }

    /// Live mineral pressure for `center`: `1.0` when broke for its next
    /// infra upgrade, `0.0` when it can comfortably afford it. Callable for
    /// *any* owned production center, not just from within its own
    /// production tick — a Query (`Hyades_vehicle_roles.md` §1), computed
    /// fresh, never stored, which is what lets [`Self::most_needed_center`]
    /// compare need across the whole empire.
    fn mineral_pressure_of(&self, center: Entity) -> f64 {
        let infra = self.world.factors.get(center).map(|f| f.infra).unwrap_or(Price::ZERO);
        let stock = self.world.stockpile.get(center).map(|s| s.basic_total()).unwrap_or(Price::ZERO);
        // The price of this center's *next* rung — the same function the build
        // path charges, rather than a second copy of `round(infra) + 1`.
        let target_level = infra_step_price(infra, &self.config);
        (1.0 - stock / target_level.max(Price::new(1e-9))).clamp(0.0, 1.0)
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
    fn best_delivery_center(&self, owner: PlayerId, from: Vec3, cargo: &Minerals) -> Option<Entity> {
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
            let score = self.bill_completion(e, owner, cargo) * (-lambda * t).exp();
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

    /// **How much of this destination's remaining shortfall the cargo closes**
    /// (R-IND17, `Hyades_industry.md` §6.11).
    ///
    /// ```text
    /// short_before = Σ_c deficit[c]
    /// short_after  = Σ_c max(0, deficit[c] − cargo[c])
    /// completion   = (short_before − short_after) / short_before
    /// ```
    ///
    /// **This replaces T-81's `relief`, which was measured counterproductive.**
    /// That version scored the fraction of the *cargo* that landed on a
    /// deficit, which sends each colour to wherever that colour is scarcest —
    /// by construction a different centre per colour. Paying a three-colour
    /// bill needs ore to **converge**, so scattering it by colour is the
    /// opposite of what the bill wants: measured, infrastructure builds fell
    /// 57 → 31 and bank composition did not move at all.
    ///
    /// **The denominator is the centre's remaining shortfall, not the bill.**
    /// That distinction is the whole mechanism and it is easy to get wrong — the
    /// first written form of R-IND17 divided by `Σ bill`, and worked out on
    /// paper that ties a centre needing only Yellow against one needing
    /// everything, both scoring `0.5` for the same Yellow delivery. No
    /// concentration at all. Dividing by what is left to find instead:
    ///
    /// | destination, given a Yellow cargo | `÷ Σ bill` | `÷ short_before` |
    /// |---|---|---|
    /// | needs only Yellow | 0.500 | **1.000** |
    /// | needs Yellow and Magenta | 0.500 | 0.750 |
    /// | needs everything | 0.500 | 0.500 |
    /// | needs only Magenta | 0.000 | 0.000 |
    ///
    /// So a centre holding two colours and missing the third pulls the third
    /// hardest, and ore concentrates where it can actually be spent.
    ///
    /// Still a dimensionless fraction in `[0, 1]`, because `exp(−λ·t)`
    /// multiplies it — R-O68 is the standing lesson on mixed-unit comparisons.
    ///
    /// **What it cannot do.** No routing rule can give an empire a colour its
    /// own ground does not hold, and the supply is single-coloured: 6,725
    /// sources measured at a mean dominant-colour share of **0.789**, 38% of
    /// them ≥95% one colour. That is §8.1's subject and the Exchange's job
    /// (T-77), with design law #1's counter-graph as the other half.
    fn bill_completion(&self, center: Entity, owner: PlayerId, cargo: &Minerals) -> f64 {
        let deficit = self.colour_deficit(center, owner);
        let short_before = deficit.iter().fold(Price::ZERO, |a, &b| a + b);
        if short_before <= Price::ZERO {
            return 0.0;
        }
        let mut short_after = Price::ZERO;
        for (i, &c) in Basic::ALL.iter().enumerate() {
            short_after += (deficit[i] - Price::new(cargo.get_basic(c))).max(Price::ZERO);
        }
        (short_before - short_after) / short_before
    }

    /// A centre's per-colour shortfall against its **next works bill** — the
    /// quantity T-73 made meaningful and nothing was measuring.
    fn colour_deficit(&self, center: Entity, owner: PlayerId) -> [Price; 3] {
        let Some(f) = self.world.factors.get(center) else {
            return [Price::ZERO; 3];
        };
        let step = infra_step_price(f.infra, &self.config);
        let works = self.world.works.get(self.player_entity[owner.0 as usize]).copied().unwrap_or_default();
        let bill = works_bill(step, &works);
        let bank = self.world.stockpile.get(center).copied().unwrap_or_default();
        let mut out = [Price::ZERO; 3];
        for (i, &c) in Basic::ALL.iter().enumerate() {
            out[i] = (bill[i] - Price::new(bank.get_basic(c))).max(Price::ZERO);
        }
        out
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
                    biosphere: f.biomass.in_bands(),
                    bio_max: f.bio_max.in_bands(),
                    biomass: f.biomass,
                    infrastructure: f.infra_band(&self.config),
                    works: f.infra,
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
                        snap.stockpiled_total += self.world.stockpile.get(e).unwrap().basic_total().kilotons();
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
fn take_basics(bank: &mut Minerals, amount: Price) -> Minerals {
    let total = bank.basic_total();
    let take = amount.min(total).max(Price::ZERO);
    let mut out = Minerals::default();
    if total <= Price::ZERO || take <= Price::ZERO {
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
    use crate::autopilot::Ranked;
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
        cfg.horizon_years = 300.0;
        cfg
    }

    /// For tests that run the sim **twice and compare the two bit-for-bit**.
    ///
    /// Those assert an *arithmetic identity* — logging is a side channel, the
    /// round layer is inert while everyone passes — and `CLAUDE.md` §2 already
    /// settled what that costs: "determinism is a property of the arithmetic,
    /// not of how long you accumulate it." Two runs is two horizons, so these
    /// pay double for a horizon that buys them nothing.
    ///
    /// **Pinned at 250 yr, and the reason it had to move is T-68.** Making
    /// `t_build` track mass took a Medium hull from 10 yr to 3.0, so a centre
    /// decides three times as often, the entity count follows, and the same 600
    /// yr does several times the work it used to. The identity is unchanged;
    /// only the bill was.
    fn paired_cfg(seed: u64) -> SimConfig {
        let mut cfg = SimConfig::new(seed);
        cfg.horizon_years = 250.0;
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
        //
        // **Pins both legs of the ladder**, rather than varying one against
        // whatever the other currently defaults to. This test asserts a
        // *structural* property of the shell model, so inheriting a ratified
        // value makes it a test of that value instead — which is exactly what
        // happened when T-56 stage 3b moved `limited_fleet_size` 9 → 50 and
        // three tests failed for reasons that had nothing to do with what they
        // were checking.
        let mut cfg = SimConfig::new(1);
        cfg.limited_fleet_size = 9.0;

        cfg.medium_fleet_size = 3.0;
        assert!(cfg.hull_ladder_fault().is_none(), "the reference 1:3:9 ladder must be valid");
        let g_wide = HullType::GeneralSystems.cargo_capacity(&cfg);

        let l_hold = HullType::LimitedSystems.hold_volume(&cfg);
        let spread_wide = HullType::MediumSystems.hold_volume(&cfg) / l_hold;

        // **Narrow is legal**, and stage 3c changed what "narrow" does. It used
        // to drive the Medium hull's hold toward *zero*, because capacity was
        // `(r − 1)³` and `r` was a ratio to the Limited hull. Now the hold is
        // set by the price and the hull's own thickness, so a Medium hull
        // priced like a Limited one simply *has a Limited hull's hold*: the two
        // converge instead of collapsing. Same conclusion — a narrow ladder is
        // a real economic statement, not a modelling failure — reached by
        // arithmetic that no longer has a denominator in it.
        cfg.medium_fleet_size = 8.0;
        assert!(cfg.hull_ladder_fault().is_none(), "a narrow ladder is meaningful, not a fault");
        let spread_narrow = HullType::MediumSystems.hold_volume(&cfg) / l_hold;
        assert!(
            spread_narrow < 1.5 && spread_narrow < spread_wide,
            "a Medium hull priced like a Limited one must hold like one: {spread_narrow} vs {spread_wide}"
        );
        // Stated as an invariance rather than a magnitude: the General hull is
        // priced off `general_vehicle_cost` alone, so moving `medium_fleet_size`
        // must leave it *exactly* where it was. An absolute threshold here would
        // only be a test of `cargo_unit_size`, which is what it silently became
        // before stage 3b moved that value.
        assert_eq!(HullType::GeneralSystems.cargo_capacity(&cfg), g_wide, "General is untouched");

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
        // Both legs pinned: the fault is `medium ≥ limited`, and stating only
        // one side makes the test depend on the other's ratified value.
        cfg.limited_fleet_size = 9.0;
        cfg.medium_fleet_size = 12.0;
        let _ = Simulation::with_baseline(galaxy, cfg);
    }

    /// **T-75b: a works card play is recorded and the state re-derived, so the
    /// empire's works are independent of the order its cards were played in.**
    ///
    /// `cards::works_fold_is_order_independent` already pins [`cards::Works::fold`]
    /// itself. This pins the *engine's* half, which is the half that can go
    /// wrong in a way the fold cannot see: `apply_card_effect` could multiply
    /// the coefficient into the live `Works` and every fold test would still
    /// pass. The state that must be order-independent is the one the simulation
    /// reads, so assert it there.
    ///
    /// Bit-identical, not approximately equal — §6.5 is explicit that this is a
    /// desync condition and not a tidiness preference.
    #[test]
    fn playing_works_cards_in_any_order_leaves_the_same_empire_state() {
        use cards::{Card, CardId, Employment, Slant, Tree, WorksWrite};
        let card = |i: u16, w: WorksWrite| Card {
            id: CardId(i),
            tree: Tree::Production,
            slant: Slant::Balanced,
            cost: 0.8,
            effect: CardEffect::WriteWorks(w),
            needs_subject: false,
        };
        // Multiplicative *and* additive writes, with factors chosen so the
        // permutations genuinely differ in float order — an order-insensitive
        // fixture would let a broken implementation through.
        let deck = [
            card(4, WorksWrite::Cap(Employment::Fabrication, 1.3)),
            card(1, WorksWrite::EtaWorks(1.1)),
            card(7, WorksWrite::Cap(Employment::Fabrication, 0.7)),
            card(2, WorksWrite::AllocWeight(Employment::Extraction, 0.1)),
            card(9, WorksWrite::AllocWeight(Employment::Extraction, 1.3)),
            card(3, WorksWrite::MixWeight(Basic::Yellow, 1.7)),
        ];
        let guard: f64 = deck
            .iter()
            .filter_map(|c| match c.effect {
                CardEffect::WriteWorks(WorksWrite::Cap(_, f)) => Some(f),
                CardEffect::WriteWorks(WorksWrite::EtaWorks(f)) => Some(f),
                _ => None,
            })
            .product();
        assert_ne!(guard, 1.3 * 0.7 * 1.1, "fixture must be order-sensitive to be worth running");

        let play = |order: &[usize]| {
            let galaxy = Galaxy::generate(GalaxyConfig::new(2, 4)).unwrap();
            let mut sim = Simulation::with_baseline(galaxy, test_cfg(4));
            for &i in order {
                sim.apply_card_effect(0, &deck[i], Target::None, 0);
            }
            *sim.world.works.get(sim.player_entity[0]).unwrap()
        };

        let want = play(&[0, 1, 2, 3, 4, 5]);
        for order in [[5, 4, 3, 2, 1, 0], [2, 0, 5, 1, 4, 3], [1, 3, 0, 4, 2, 5], [3, 5, 1, 0, 2, 4]] {
            let got = play(&order);
            assert_eq!(
                got.eta_works.to_bits(),
                want.eta_works.to_bits(),
                "eta_works differs under {order:?}: {} vs {}",
                got.eta_works,
                want.eta_works
            );
            for i in 0..3 {
                assert_eq!(got.cap[i].to_bits(), want.cap[i].to_bits(), "cap[{i}] differs under {order:?}");
                assert_eq!(got.half[i].to_bits(), want.half[i].to_bits(), "half[{i}] differs under {order:?}");
                assert_eq!(got.alloc_w[i].to_bits(), want.alloc_w[i].to_bits(), "alloc_w[{i}] under {order:?}");
                assert_eq!(got.mix_w[i].to_bits(), want.mix_w[i].to_bits(), "mix_w[{i}] under {order:?}");
            }
        }

        // And the writes actually landed — an implementation that dropped every
        // one of them would satisfy every assertion above.
        assert_ne!(want, cards::Works::default(), "the plays must have moved the state");

        // The rate the simulation reads moves with it, which is the only reason
        // any of this matters.
        let galaxy = Galaxy::generate(GalaxyConfig::new(2, 4)).unwrap();
        let mut sim = Simulation::with_baseline(galaxy, test_cfg(4));
        let yard = sim.world.player_info.get(sim.player_entity[0]).unwrap().home;
        let before = sim.fabrication_rate(yard);
        sim.apply_card_effect(0, &card(4, WorksWrite::Cap(Employment::Fabrication, 2.0)), Target::None, 0);
        assert!(sim.fabrication_rate(yard) > before, "a cap card must raise the yard's rate");
    }

    #[test]
    fn the_round_layer_is_behaviour_neutral_while_everyone_passes() {
        // The baseline autopilot's `choose_card` returns `None`, so adding the
        // round layer must move *nothing*. This is what keeps every coverage
        // number in the tree — and the offline search resting on them — valid
        // across the card layer landing. Verified at the shipped defaults:
        // seed 1 / 3 seats / 4 kyr gives 1,044 colonies with and without.
        let galaxy = Galaxy::generate(GalaxyConfig::new(3, 1)).unwrap();
        let mut with_rounds = Simulation::with_baseline(galaxy, paired_cfg(1));
        let a = with_rounds.run();

        let galaxy = Galaxy::generate(GalaxyConfig::new(3, 1)).unwrap();
        let mut cfg = paired_cfg(1);
        cfg.years_per_round = 0.0; // disables the layer entirely
        let mut without = Simulation::with_baseline(galaxy, cfg);
        let b = without.run();

        for (pa, pb) in a.players.iter().zip(b.players.iter()) {
            assert_eq!(pa.colonies, pb.colonies);
            assert_eq!(pa.planets_owned, pb.planets_owned);
            assert_eq!(pa.total_population.kilotons().to_bits(), pb.total_population.kilotons().to_bits());
        }
        assert_eq!(a.planets_scanned_total, b.planets_scanned_total);
    }

    #[test]
    fn round_boundaries_fire_on_the_specified_cadence() {
        // 100 yr to the first, 100 yr between: at a 250 yr horizon that is
        // rounds 0 and 1. The barrier is a scheduled event, so this also pins
        // that it chains itself rather than being swept for.
        let galaxy = Galaxy::generate(GalaxyConfig::new(3, 1)).unwrap();
        let mut cfg = paired_cfg(1);
        cfg.years_to_first_round = 100.0;
        cfg.years_per_round = 100.0;
        let mut sim = Simulation::with_baseline(galaxy, cfg);
        sim.run();
        assert_eq!(sim.current_round(), 1, "(250-100)/100 = 1, so the last barrier is round 1");

        // And it chains rather than firing once, and stops at the horizon rather
        // than running away. **Shortened the cadence, not the horizon**
        // (`CLAUDE.md` §2 — cut samples, not the question): this used to buy its
        // extra barriers with a 1,400 yr run, which cost **437 s of a 507 s unit
        // target** once T-68 made hulls 3-4x quicker to build and the entity
        // count followed. A 25 yr cadence at 250 yr exercises **ten** barriers
        // where the long run exercised four, and the property under test — the
        // barrier reschedules itself and the last one lands below the horizon —
        // is exactly the same property, now better covered for 6% of the cost.
        let galaxy = Galaxy::generate(GalaxyConfig::new(3, 1)).unwrap();
        let mut cfg = paired_cfg(1);
        cfg.years_to_first_round = 25.0;
        cfg.years_per_round = 25.0;
        let mut chained = Simulation::with_baseline(galaxy, cfg);
        chained.run();
        assert_eq!(chained.current_round(), 9, "(250-25)/25 = 9, so the last barrier is round 9");
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
        let total_pop: f64 = report.players.iter().map(|p| p.total_population.kilotons()).sum();
        assert!(total_pop > 6.5, "population did not grow: {total_pop}");
    }

    #[test]
    fn deterministic_same_seed_same_outcome() {
        // **The third paired-run test, and the most expensive of them** — six
        // seats, run twice. `paired_cfg` for the same reason as the other two:
        // determinism is a property of the arithmetic, not of how long you
        // accumulate it (`CLAUDE.md` §2), and after T-68 this one run was 65 s
        // of a 68 s unit target on its own. `tests/determinism.rs` is the
        // full-scale guard; this is the in-module smoke version of it.
        let mk = |seed: u64| {
            let galaxy = Galaxy::generate(GalaxyConfig::new(6, seed)).unwrap();
            let mut sim = Simulation::with_baseline(galaxy, paired_cfg(seed));
            let report = sim.run();
            (sim, report)
        };
        let (_a, ra) = mk(7);
        let (_b, rb) = mk(7);
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
        let before = galaxy.planet(hw).minerals.total_mass().kilotons();
        let mut sim = Simulation::with_baseline(galaxy, test_cfg(11));
        sim.run();
        let after = {
            let e = sim.planet_entity[hw.0 as usize];
            sim.world.density.get(e).unwrap().total_mass().kilotons()
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
            let mut s = Simulation::with_baseline(g, paired_cfg(2024));
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
            assert_eq!(pq.total_population.kilotons().to_bits(), pl.total_population.kilotons().to_bits());
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
        let dry = hull_dry_mass(HullType::MediumSystems, &sim2.config).kilotons();
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
        cfg.limited_fleet_size = 9.0;
        let (l, m, g) = (HullType::LimitedSystems, HullType::MediumSystems, HullType::GeneralSystems);

        // **Radius is solved from the price, not square-rooted from it.**
        // `r = sqrt(cost / cost_Limited)` was the constant-`τ` special case
        // (T-56 stage 3c); the law is `cost·η = r³ − (r − τ)³`. Assert the
        // inversion, which holds for every ladder, rather than the three
        // magnitudes one ladder happens to produce.
        for hull in [l, m, g] {
            let recovered = hull.shell_volume(&cfg).hull_units_cubed() / hull.geometry().shape_efficiency;
            let paid = hull.cost_fraction(&cfg) * cfg.general_vehicle_cost;
            assert!((recovered - paid).abs() < 1e-9, "{hull:?}: shell/η = {recovered}, paid {paid}");
        }
        assert!(l.hull_radius(&cfg) < m.hull_radius(&cfg));
        assert!(m.hull_radius(&cfg) < g.hull_radius(&cfg));

        // R-O57: dry mass *is* the cost, in one unit.
        for hull in [l, m, g] {
            let cost = hull.cost_fraction(&cfg) * cfg.general_vehicle_cost;
            assert!((hull_dry_mass(hull, &cfg).kilotons() - cost).abs() < 1e-12);
        }

        // What survives of roles §6's 0/1/2: the *ordinal* content. Each larger
        // hull carries strictly more, and the magnitudes are geometry rather
        // than a unit count (R-O64). The Limited hull's hold is no longer
        // identically zero — §2.3 gives a Limited *Systems* hull a token load
        // and reserves the zero for the roles whose payload fills the hull.
        assert!(l.cargo_capacity(&cfg) < m.cargo_capacity(&cfg));
        assert!(m.cargo_capacity(&cfg) < g.cargo_capacity(&cfg));
        assert_eq!(HullType::LimitedOffensive.cargo_capacity(&cfg), Kilotons::ZERO);

        // Design law #3: consolidation must win under geometry alone. The
        // pre-shell model failed this — a General hull cost 9× a Limited and
        // hauled 2 units where a Medium cost 3× and hauled 1, i.e. 0.100 vs
        // 0.067 cost per unit hauled, so *fragmenting* was cheaper.
        let per_kt = |h: HullType| h.cost_fraction(&cfg) * cfg.general_vehicle_cost / h.cargo_capacity(&cfg).kilotons();
        assert!(per_kt(g) < per_kt(m), "bigger hull must be cheaper per kt hauled");
    }

    /// **R-O71 closed: the hold ladder *is* the mass ladder, tied to the cost
    /// ladder by `F_mass = F_cost^(3/2)`.**
    ///
    /// This test replaces `the_cargo_ladder_is_geometric_not_banded`, which
    /// pinned a disagreement between two ratified specs — §2.6 demanded one
    /// step factor in `[4, 8]` for every Band-laddered quantity, cargo capacity
    /// named among them, while the shell model stepped Medium → General by
    /// ~106×. R-MC15 resolved it in the geometry's favour: the `[4, 8]` window
    /// is withdrawn, capacity sits on the **mass** ladder rather than the cost
    /// one, and the `3/2` exponent is what makes those one geometry instead of
    /// two scales. The old test's failure message asked for exactly this
    /// replacement, and T-56 stage 3c is what triggered it.
    ///
    /// The rung is the **hold**, not the usable cargo: `V_reserved` is a role
    /// deduction applied after the ladder, so cargo steps a little wide of the
    /// factor (34.3 against 31.6) while the hold hits it.
    #[test]
    fn the_hold_ladder_is_the_mass_ladder() {
        let cfg = SimConfig::new(1);
        let (l, m, g) = (HullType::LimitedSystems, HullType::MediumSystems, HullType::GeneralSystems);

        // The `3/2` tie, asserted against the cost ladder that produced it
        // rather than against a copied constant. **The steps are the hull cost
        // steps, not `MASS_LADDER` entries** — those two agreed at the top of
        // the ladder and used to agree at the bottom as well, and the bottom
        // agreement was a coincidence of where the ladder's floor sat. Widening
        // `Band Empty` to one tonne (`units::KILOTONS_AT_BAND_EMPTY`) ended it:
        // `MASS_LADDER[0]` is the floor's width, 1000, while the Limited →
        // Medium hull step is and remains `5^1.5`. Asserting the geometry
        // against the geometry is the claim that was always meant.
        for (cost_step, hold_step) in [
            (cfg.limited_fleet_size / cfg.medium_fleet_size, m.hold_volume(&cfg) / l.hold_volume(&cfg)),
            (cfg.medium_fleet_size, g.hold_volume(&cfg) / m.hold_volume(&cfg)),
        ] {
            // Tolerances are loose because `η` varies across the three sizes by
            // design (§2.2) — the steps are hit to within a couple of percent,
            // not to the bit.
            let want = cost_step.powf(1.5);
            assert!((hold_step / want - 1.0).abs() < 0.05, "hold step {hold_step} vs {cost_step}^1.5 = {want}");
        }

        // **Medium and General holds still land on `Band I` and `Band II`.**
        // That is the part of the T-56 story the floor change does *not* touch,
        // because both rungs are above it. What it does touch is the Limited
        // hull, whose 0.089 kt hold used to sit exactly on `Band Empty` and now
        // reads ~`Band 0.65` — recorded here rather than asserted, because it
        // is a consequence of the floor and not a property of the geometry.
        for (hull, rung) in [(m, 1.0), (g, 2.0)] {
            let hold = Kilotons::new(hull.hold_volume(&cfg).hull_units_cubed() * cfg.cargo_unit_size);
            assert!((hold.in_bands().bands() - rung).abs() < 0.05, "{hull:?} hold reads {:?}", hold.in_bands());
        }

        // **R-V9 is satisfied by geometry rather than contradicted by it.** A
        // Colonizer carries `colony_seed_pop` of settlers, and before T-56 that
        // mass (1.0 kt) *exceeded* the hold of the Medium hull R-V9 names as the
        // smallest able to carry them (0.959 kt) — an inconsistency nothing
        // caught, because capacity gates mineral loading only. The Medium hull's
        // hold is now `Band I` by construction, which is the rung the colony
        // seed is defined at, so the rule and the geometry finally agree.
        let seed_mass = units::population_mass(cfg.colony_seed_pop.band()).kilotons();
        let hold = m.hold_volume(&cfg).hull_units_cubed() * cfg.cargo_unit_size;
        assert!(
            seed_mass <= hold,
            "the colony seed ({seed_mass:.3} kt) must fit a Medium hold ({hold:.3} kt) — R-V9 says a \
             Medium hull is the smallest that can found a colony, and a hold too small to carry the \
             seed makes that rule unsatisfiable"
        );
    }

    /// **T-56: hull geometry carries its units in the type, and the ladder is
    /// the ratified one.**
    ///
    /// The shell model was always three quantities — a radius, a thickness and
    /// the volumes they bound — but only one of them had a name and none had a
    /// type, so `(r − 1)` was a subtraction whose second operand was a literal.
    /// Stage 2 named and typed them; stage 3c gave `τ` its ratified per-hull
    /// values, which is what finally separates cost from capacity.
    #[test]
    fn hull_geometry_is_dimensioned_and_the_shell_closes() {
        let cfg = SimConfig::new(1);
        let (l, m, g) = (HullType::LimitedSystems, HullType::MediumSystems, HullType::GeneralSystems);

        for hull in HullType::ALL {
            // Shell + hold is the whole hull, with nothing unaccounted for.
            // This is the identity that makes cost and capacity two halves of
            // one body rather than two ladders that happen to share a symbol.
            let closed = hull.shell_volume(&cfg) + hull.hold_volume(&cfg);
            let whole = hull.hull_radius(&cfg).cubed();
            assert!(
                (closed - whole).hull_units_cubed().abs() < 1e-12,
                "{hull:?}: shell + hold = {closed} but the hull is {whole}"
            );

            // r − τ, floored. The floor is a statement, not a guard: a hull
            // whose skin is as thick as it is wide has no interior.
            let expect = (hull.hull_radius(&cfg) - hull.shell_thickness()).max(Length::ZERO);
            assert_eq!(hull.hold_radius(&cfg), expect, "{hull:?}");

            // **The radius solve inverts the price.** `hull_radius` solves
            // `cost·η = r³ − (r − τ)³`, so running it backwards must return the
            // cost that went in. If these ever drift apart, the shell has
            // stopped being what is bought.
            let recovered = hull.shell_volume(&cfg).hull_units_cubed() / hull.geometry().shape_efficiency;
            let paid = hull.cost_fraction(&cfg) * cfg.general_vehicle_cost;
            assert!((recovered - paid).abs() < 1e-9, "{hull:?}: shell/η = {recovered} but it cost {paid}");
        }

        // **Shell thickness rises with size**, which is the design arc's
        // requirement, and it is *derived* — §2.3 solves it from the cost and
        // hold the Band rungs fix, and `η` rising toward the sphere is what
        // pays for it. This is also the tripwire stage 2 left behind: it used
        // to assert one unit for every hull.
        assert!(l.shell_thickness() < m.shell_thickness());
        assert!(m.shell_thickness() < g.shell_thickness());

        // **And with armour**, at every size: Offensive > Contact > Systems.
        // The armour statement as geometry rather than as a combat constant,
        // which is what keeps design law #2 intact.
        assert!(l.shell_thickness() < HullType::LimitedContactVehicle.shell_thickness());
        assert!(HullType::LimitedContactVehicle.shell_thickness() < HullType::LimitedOffensive.shell_thickness());
        assert!(g.shell_thickness() < HullType::GeneralContactVehicle.shell_thickness());
        assert!(HullType::GeneralContactVehicle.shell_thickness() < HullType::GeneralOffensive.shell_thickness());

        // **The Systems row lands on the Band rungs**, which is the whole point
        // of the ratified ladder: the hold is the quantity that sits on a rung,
        // and its steps are the mass ladder's own factors.
        // The factors are the *cost* ladder's, raised to `3/2` — see
        // `the_hold_ladder_is_the_mass_ladder` for why that is no longer the
        // same thing as `MASS_LADDER` at the bottom rung.
        let step_lo = m.hold_volume(&cfg) / l.hold_volume(&cfg);
        let step_hi = g.hold_volume(&cfg) / m.hold_volume(&cfg);
        assert!(
            (step_lo - (cfg.limited_fleet_size / cfg.medium_fleet_size).powf(1.5)).abs() < 0.2,
            "Limited→Medium hold step {step_lo}"
        );
        assert!((step_hi - cfg.medium_fleet_size.powf(1.5)).abs() < 0.5, "Medium→General hold step {step_hi}");

        // **Capability, not competence** (roles §4, R-O44). A Limited Contact
        // or Offensive hull and a Rapid Offensive one reserve more than their
        // whole hold, so they carry exactly nothing — reached through
        // `max(0, ·)`, not clamped. A Limited *Systems* hull carries a scout's
        // sample locker, and a General Offensive one carries real tonnage.
        for hull in [HullType::LimitedContactVehicle, HullType::LimitedOffensive, HullType::RapidOffensive] {
            assert_eq!(hull.cargo_capacity(&cfg), Kilotons::ZERO, "{hull:?} must have no hold");
        }
        let lsv = l.cargo_capacity(&cfg).kilotons();
        assert!((0.001..0.1).contains(&lsv), "a Limited Systems hull carries a token load, got {lsv}");
        assert!(HullType::GeneralOffensive.cargo_capacity(&cfg) > Kilotons::new(1.0));

        // Design law #3, in every role: carry efficiency must rise with size,
        // so consolidation wins under geometry alone.
        let e = |h: HullType| h.cargo_capacity(&cfg).kilotons() / (h.cost_fraction(&cfg) * cfg.general_vehicle_cost);
        assert!(e(l) < e(m) && e(m) < e(g), "Systems: {} {} {}", e(l), e(m), e(g));
        assert!(e(HullType::LimitedContactVehicle) < e(HullType::GeneralContactVehicle));
        assert!(e(HullType::RapidOffensive) < e(HullType::GeneralOffensive));
    }

    /// **T-56 stage 4a: the hull that was ordered is the hull that is built.**
    ///
    /// R-O29 moved the hull choice into `BuildOrder::Hull`, but `apply_build`
    /// kept pricing and spawning from `role_hull_type(role)` — which agreed
    /// only because that map happened to invert `assign_role`. Nothing asserted
    /// the round trip, so the disagreement would have arrived silently the
    /// first time Doctrine ordered a hull the role map does not name.
    #[test]
    fn the_ordered_hull_is_the_hull_that_is_priced_and_flown() {
        let cfg = SimConfig::new(1);

        // The round trip that used to hold by luck, asserted: every hull the
        // baseline autopilot can order must come back to itself through the
        // role it is assigned.
        for (hull, role) in [
            (HullType::MediumSystems, Role::Colonizer),
            (HullType::LimitedSystems, Role::Miner),
            (HullType::LimitedContactVehicle, Role::Scout),
        ] {
            assert_eq!(role_hull_type(role), hull, "{role:?} still maps back to {hull:?}");
            assert!((role_cost(role, &cfg) - hull_cost(hull, &cfg)).abs() < Price::new(1e-12));
        }

        // And the thing the round trip was standing in for: a heavier hull
        // costs more, so pricing off the role rather than the order would have
        // bought a General hull at a Medium hull's price.
        assert!(hull_cost(HullType::GeneralSystems, &cfg) > hull_cost(HullType::MediumSystems, &cfg) * 5.0);
    }

    /// **T-56 stage 4: carry up to the target's carrying capacity, and no
    /// more — and the capacity now depends on the hull that founds it (R-O76).**
    ///
    /// Two ladders meet here. The **hold** ladder gives each hull a colonist
    /// capacity (`Band Empty` / `I` / `II`), and the **infrastructure** ladder
    /// converts the recycled hull into the colony's founding `K`. Before R-O76
    /// only the first varied and the second was a constant, which capped every
    /// new colony at one Band and made a heavier colony ship pointless by
    /// construction.
    #[test]
    fn a_colony_ship_carries_up_to_the_targets_capacity_and_no_more() {
        let mut sim = Simulation::with_baseline(Galaxy::generate(GalaxyConfig::new(3, 1)).unwrap(), test_cfg(1));
        // **A third cap joined the two this test is about** (R-O74): settlers
        // come out of the origin's population. Give this centre people to
        // spare so the hold-vs-world question stays readable;
        // `settlers_are_drawn_from_a_real_population` pins the origin case.
        let home = sim.world.player_info.get(sim.player_entity[0]).unwrap().home;
        sim.world.population.insert(home, Kilotons::at_tier(BandTier::IV));

        // Colonist capacity is the hold's rung, and the rungs are the mass
        // ladder's.
        let (m_cap, g_cap) = (
            HullType::MediumSystems.colony_seed_capacity(&sim.config),
            HullType::GeneralSystems.colony_seed_capacity(&sim.config),
        );
        assert!((m_cap.band().bands() - 1.0).abs() < 1e-9, "a Medium hold is Band I, got {m_cap}");
        assert!((g_cap.band().bands() - 2.0).abs() < 1e-6, "a General hold is Band II, got {g_cap}");
        assert!(
            HullType::LimitedSystems.colony_seed_capacity(&sim.config)
                < units::population_mass(sim.config.colony_seed_pop.band())
        );

        // **Founding infrastructure is the recycled hull's minerals, read on
        // the infrastructure ladder — which is the mineral ladder (R-O80).**
        // The rungs *are* the hull costs, so a recycled hull buys exactly the
        // infrastructure its minerals would have bought, and every anchor lands
        // on a whole rung with no rate and no subsidy in between.
        // (to a tolerance: `band_of` is a log round trip, so a whole rung comes
        // back as 2.0 minus a couple of ulps)
        for (hull, rung) in [
            (HullType::LimitedSystems, BandTier::Empty),
            (HullType::MediumSystems, BandTier::I),
            (HullType::GeneralSystems, BandTier::II),
        ] {
            let got = sim.founding_infra(hull);
            let got_band = sim.founding_infra_band(hull);
            assert!(
                (got_band.bands() - rung.band().bands()).abs() < 1e-9,
                "{hull:?} founds at {got_band} ({got}), want {rung}"
            );
        }
        let m_infra = sim.founding_infra(HullType::MediumSystems);
        let g_infra = sim.founding_infra(HullType::GeneralSystems);

        // And the prices those rungs correspond to: `Infra I costs minerals I`.
        for hull in [HullType::LimitedSystems, HullType::MediumSystems, HullType::GeneralSystems] {
            let rung = infra_rung_of(sim.founding_infra(hull), &sim.config);
            let priced = infra_rung_price(rung, &sim.config);
            assert!(
                (priced - hull_cost(hull, &sim.config)).abs() < Price::new(1e-12),
                "{hull:?}: rung {rung} prices at {priced} but the hull costs {}",
                hull_cost(hull, &sim.config)
            );
        }

        // A target far better than either hull can fill: `k_potential` of 4.
        let target = sim.planet_entity[11];
        sim.world.factors.insert(
            target,
            Factors::new(Band::new(4.0), Band::new(4.0).in_kilotons(), Band::new(4.0).in_kilotons(), Price::ZERO),
        );

        // **The founding capacity is the world's, and the hull has nothing to
        // do with it** (T-67). Infrastructure left `K`, so this is
        // `population_mass(k_potential)` — a `Band IV` ceiling on this target —
        // for every hull alike. `founding_infra` still lands as industrial
        // stock; it just no longer gates who may live there.
        assert_eq!(sim.founding_capacity(target), units::population_mass(Band::new(4.0)));
        let medium = sim.colony_seed_for(HullType::MediumSystems, home, target).expect("a Medium hull can found");
        let general = sim.colony_seed_for(HullType::GeneralSystems, home, target).expect("a General hull can found");
        let close = |a: Kilotons, b: Kilotons| (a.band().bands() - b.band().bands()).abs() < 1e-9;

        // **Both hulls are hold-limited now, not `K`-limited** — the exact
        // reverse of the pre-T-67 case. On a world this good neither hull can
        // fill the ceiling, so each lands precisely what it carried.
        assert!(close(medium, m_cap), "a Medium lands its hold: {m_cap} vs {medium}");
        assert!(close(general, g_cap), "and a General lands its: {g_cap} vs {general}");
        assert!(general > medium, "the hold is the only thing separating them now");

        // And the cap still binds the other way round: on a poor world the
        // *world* is the limit, and a General hull cannot force more people
        // onto it than it can hold. This is what stops founding from being able
        // to trigger the overshoot at all.
        let poor = sim.planet_entity[12];
        sim.world.factors.insert(
            poor,
            Factors::new(Band::new(1.0), Band::new(4.0).in_kilotons(), Band::new(4.0).in_kilotons(), Price::ZERO),
        );
        let capped = sim.colony_seed_for(HullType::GeneralSystems, home, poor).expect("a General hull can found");
        assert!(
            close(capped, units::population_mass(Band::new(1.0))),
            "a General hull on a Band I world lands Band I, not its hold: {capped}"
        );

        // **R-V9 is physics, through the hold** — and since T-67 it is enforced
        // there rather than falling out of the infrastructure coupling. A
        // Limited hull's hold is below `colony_seed_pop`, so it founds nothing
        // however good the world is.
        assert_eq!(sim.colony_seed_for(HullType::LimitedSystems, home, target), None);
        assert_eq!(sim.colony_seed_for(HullType::LimitedSystems, home, poor), None);
        let _ = (m_infra, g_infra);
    }

    /// **T-84: the books fill with real two-sided depth, and nothing clears.**
    ///
    /// The third and last inert stage. Two claims, and the first is what stops
    /// the second being vacuous — an Exchange with empty books would also be
    /// bit-identical.
    ///
    /// The interesting assertion is **two-sidedness**. §10.8 says the guard for
    /// Exchange work is a census rather than colony-years, and this is that
    /// census at its smallest: if every empire were short the same colours, the
    /// books would be all bids and no asks and there would be no trade to make.
    /// T-73 measured why they are not — 1,494 of 1,515 banks are
    /// single-coloured, so nearly every centre is simultaneously **long one
    /// colour and short the other two**, which is precisely the condition that
    /// makes a colour market worth having.
    #[test]
    fn the_exchange_books_fill_on_both_sides() {
        let run = |post: bool| {
            let mut gcfg = GalaxyConfig::new(3, 33);
            gcfg.planet_count = 400;
            let galaxy = Galaxy::generate(gcfg).unwrap();
            let mut cfg = test_cfg(33);
            cfg.horizon_years = 900.0; // two round barriers at the default cadence
            let mut sim = Simulation::with_baseline(galaxy, cfg);
            sim.exchange_posting = post;
            // Posting and *pricing* are what this stage is about; settlement is
            // T-77 and moves ore, which would make the inertness claim false.
            sim.exchange_settlement = false;
            let report = sim.run();
            let pops: Vec<u64> = report.players.iter().map(|x| x.total_population.kilotons().to_bits()).collect();
            (report.events_processed, report.planets_scanned_total, pops, sim.exchange_posted())
        };

        let (ev_off, scan_off, pop_off, depth_off) = run(false);
        let (ev_on, scan_on, pop_on, depth_on) = run(true);

        assert_eq!(depth_off, [(0, 0); 3], "posting disabled must post nothing");

        // **Both sides, on at least one colour.** A one-sided book is a market
        // with nothing to match.
        let (bids, asks): (u64, u64) = depth_on.iter().fold((0, 0), |(b, a), &(x, y)| (b + x, a + y));
        assert!(bids > 0, "no centre bid for anything it was short of: {depth_on:?}");
        assert!(asks > 0, "no centre offered anything it was long of: {depth_on:?}");
        assert!(
            depth_on.iter().any(|&(b, a)| b > 0 && a > 0),
            "no single colour has both sides, so nothing could ever match: {depth_on:?}"
        );

        // **Nothing clears.** Same seed, same run, to the bit.
        assert_eq!(ev_off, ev_on, "posting moved the event count");
        assert_eq!(scan_off, scan_on, "posting moved survey");
        assert_eq!(pop_off, pop_on, "posting moved population");
    }

    /// **T-85: contracts are struck and escrowed, and the world does not move.**
    ///
    /// **§10.7 predicted this stage would change the bed and it does not, which
    /// is the outpost amendment's doing** (§10.6). The build order was written
    /// when a cleared match was a cross-empire *delivery*, so clearing and
    /// moving goods were one step. Settlement now happens at a shared rock, so
    /// stage 4 strikes contracts and locks `$` — and `$` reaches nothing else —
    /// while every kilotonne stays exactly where it was until T-77.
    ///
    /// So the stage that was expected to be the risky one is inert, and the
    /// risk moved to T-77 with the goods. Worth having as a test rather than a
    /// note, because "this stage is neutral" is a claim about code and §6.7's
    /// version of it was wrong twice.
    #[test]
    fn clearing_strikes_escrowed_contracts_without_moving_the_world() {
        let run = |clear: bool| {
            let mut gcfg = GalaxyConfig::new(3, 51);
            gcfg.planet_count = 500;
            let galaxy = Galaxy::generate(gcfg).unwrap();
            let mut cfg = test_cfg(51);
            cfg.horizon_years = 1600.0; // several barriers, and time to open outposts
            let mut sim = Simulation::with_baseline(galaxy, cfg);
            sim.exchange_posting = clear;
            // **Stage 4 only.** T-77 schedules settlement, which does move ore;
            // holding it off is what keeps this a test of clearing alone.
            sim.exchange_settlement = false;
            let report = sim.run();
            let pops: Vec<u64> = report.players.iter().map(|x| x.total_population.kilotons().to_bits()).collect();
            let purses: Vec<f64> = (0..3).map(|p| sim.purse_of(PlayerId(p))).collect();
            (report.events_processed, report.planets_scanned_total, pops, sim.exchange_state(), purses)
        };

        let (ev_off, scan_off, pop_off, st_off, _) = run(false);
        let (ev_on, scan_on, pop_on, st_on, purses_on) = run(true);

        // **Trade happened.** Without this every assertion below is vacuous —
        // an Exchange that matches nothing is also bit-identical.
        assert_eq!(st_off.0, 0, "posting disabled must strike no contracts");
        assert!(
            st_on.0 > 0,
            "no contract was struck: either no colour had both sides, or no two empires shared an outpost"
        );
        assert_eq!(st_on.1, 0, "settlement was disabled, so nothing may have settled");

        // **Escrow was locked.** `$` left the buyers' purses and is owed rather
        // than spent — the first thing in the engine that is owed (§10.2).
        assert!(purses_on.iter().all(|&p| p.is_finite()), "a purse went non-finite: {purses_on:?}");

        // **And the world did not move.** Same seed, same run, to the bit.
        assert_eq!(ev_off, ev_on, "clearing moved the event count");
        assert_eq!(scan_off, scan_on, "clearing moved survey");
        assert_eq!(pop_off, pop_on, "clearing moved population");
    }

    /// **T-77: settlement moves ore between empires and conserves it.**
    ///
    /// The first Exchange stage that moves a kilotonne, so the thing to assert
    /// is **design law #11**: mass is conserved with no exclusions. The `$`
    /// burn is not a counterexample — `$` was never in the mass ledger (R-P1),
    /// which is the entire reason a faucet and a sink are legal at all.
    ///
    /// Also pins the destination, because it is the amendment's whole claim:
    /// the ore lands in the **buyer's pile at the shared rock**, not at the
    /// buyer's world. `outpost_stock` is already what a laden freighter loads
    /// from, so the collection leg needed no new code.
    #[test]
    fn settlement_moves_ore_to_the_buyers_pile_and_conserves_mass() {
        let mut sim = Simulation::with_baseline(Galaxy::generate(GalaxyConfig::new(2, 71)).unwrap(), test_cfg(71));
        let seller_centre = sim.world.player_info.get(sim.player_entity[1]).unwrap().home;
        let rock = sim.planet_entity[30];
        for p in 0..2u32 {
            sim.mine_crew.insert((p, rock.0), vec![sim.world.spawn()]);
        }
        sim.world.stockpile.get_mut(seller_centre).unwrap().yellow = 100.0;
        sim.credit(sim.player_entity[0], 500.0);

        let total_yellow = |s: &Simulation| -> f64 {
            let banked: f64 = s.planet_entity.iter().filter_map(|&e| s.world.stockpile.get(e)).map(|b| b.yellow).sum();
            let piled: f64 = s.outpost_stock.values().map(|m| m.yellow).sum();
            banked + piled
        };
        let before = total_yellow(&sim);

        let id = sim.exchange.next_id;
        sim.exchange.next_id += 1;
        sim.exchange.contracts.insert(
            id,
            Contract {
                buyer: PlayerId(0),
                seller: PlayerId(1),
                seller_centre,
                colour: Basic::Yellow,
                qty: 40.0,
                escrow: 80.0,
                seller_drop: rock,
                buyer_drop: None,
                struck: sim.clock,
            },
        );
        sim.credit(sim.player_entity[0], -80.0);
        let purse_before = sim.purse_of(PlayerId(1));

        sim.sys_contract_due(id);

        assert_eq!(sim.exchange_state().1, 1, "the contract must have settled");
        assert_eq!(sim.exchange_defaults(), 0);
        assert!(
            (sim.world.stockpile.get(seller_centre).unwrap().yellow - 60.0).abs() < 1e-9,
            "the seller's bank must be debited"
        );
        assert!(
            (sim.outpost_stock.get(&(0, rock.0)).map_or(0.0, |m| m.yellow) - 40.0).abs() < 1e-9,
            "the ore must land in the *buyer's* pile at the shared rock"
        );
        assert!((total_yellow(&sim) - before).abs() < 1e-9, "mass was not conserved across the trade");
        assert!(sim.purse_of(PlayerId(1)) > purse_before, "the seller must be paid");
    }

    /// **A seller that cannot deliver defaults, and the loss is shared**
    /// (§3.3, R-IND10, T-77).
    ///
    /// The buyer gets its escrow back **minus the burn** and the seller gets
    /// nothing — so default is not a free option: it forfeits the sale. That
    /// asymmetry is what makes escorting worth paying for, which is the reason
    /// §3.3 chose shared loss over returning the escrow whole.
    #[test]
    fn a_seller_that_cannot_deliver_defaults_and_both_sides_pay() {
        let mut sim = Simulation::with_baseline(Galaxy::generate(GalaxyConfig::new(2, 73)).unwrap(), test_cfg(73));
        let seller_centre = sim.world.player_info.get(sim.player_entity[1]).unwrap().home;
        let rock = sim.planet_entity[30];
        // The bank is short the colour it owes — rich in Cyan, owing Yellow.
        {
            let b = sim.world.stockpile.get_mut(seller_centre).unwrap();
            b.yellow = 1.0;
            b.cyan = 9_999.0;
        }
        let id = sim.exchange.next_id;
        sim.exchange.next_id += 1;
        sim.exchange.contracts.insert(
            id,
            Contract {
                buyer: PlayerId(0),
                seller: PlayerId(1),
                seller_centre,
                colour: Basic::Yellow,
                qty: 40.0,
                escrow: 80.0,
                seller_drop: rock,
                buyer_drop: None,
                struck: sim.clock - 50.0, // far enough back that the burn bites
            },
        );
        let (buyer_before, seller_before) = (sim.purse_of(PlayerId(0)), sim.purse_of(PlayerId(1)));

        sim.sys_contract_due(id);

        assert_eq!(sim.exchange_defaults(), 1, "a bank short the named colour must default");
        assert_eq!(sim.exchange_state().1, 0, "a default is not a settlement");
        let refunded = sim.purse_of(PlayerId(0)) - buyer_before;
        assert!(refunded > 0.0 && refunded < 80.0, "the buyer is refunded minus the burn, got {refunded}");
        assert_eq!(sim.purse_of(PlayerId(1)), seller_before, "the seller gains nothing by defaulting");
        assert!(sim.exchange_state().2 > 0.0, "the burn is the sink (§2.3)");
        // **A rich bank in the wrong colour does not help**, which is what the
        // colour axis is for.
        assert!(sim.world.stockpile.get(seller_centre).unwrap().cyan > 9_000.0, "the wrong colour was never touched");
    }

    /// **A trade needs a rock both parties work** (§10.6, T-85).
    ///
    /// Geography is the trade constraint, and this pins it as a *mechanic*
    /// rather than an implementation detail: two empires that share nothing
    /// cannot settle, and **each drop is chosen by the party shipping to it**
    /// (R-P17, as revised — a contract has two locations, and a shipper pays for
    /// its own leg).
    #[test]
    fn two_empires_can_only_trade_where_they_both_have_crew() {
        let mut sim = Simulation::with_baseline(Galaxy::generate(GalaxyConfig::new(2, 61)).unwrap(), test_cfg(61));
        let (a, b) = (PlayerId(0), PlayerId(1));
        let origin = Vec3::ZERO;

        assert_eq!(sim.shared_venue(a, b, origin), None, "no crews anywhere, no venue");

        // One rock each, different rocks: still nothing in common.
        let (r1, r2) = (sim.planet_entity[20], sim.planet_entity[21]);
        sim.mine_crew.insert((0, r1.0), vec![sim.world.spawn()]);
        sim.mine_crew.insert((1, r2.0), vec![sim.world.spawn()]);
        assert_eq!(sim.shared_venue(a, b, origin), None, "different rocks are not a venue");

        // Now share two, and each shipper gets the one nearest *itself*.
        let (r3, r4) = (sim.planet_entity[22], sim.planet_entity[23]);
        for &r in &[r3, r4] {
            sim.mine_crew.insert((0, r.0), vec![sim.world.spawn()]);
            sim.mine_crew.insert((1, r.0), vec![sim.world.spawn()]);
        }
        let (p3, p4) = (*sim.world.position.get(r3).unwrap(), *sim.world.position.get(r4).unwrap());
        assert_eq!(sim.shared_venue(a, b, p3), Some(r3), "a shipper standing on a shared rock drops there");
        assert_eq!(sim.shared_venue(a, b, p4), Some(r4));

        // **The venue set is symmetric even though the choice is not.** Which
        // rocks are *available* cannot depend on which party is named first;
        // which one is *picked* depends only on where the shipper is.
        assert_eq!(sim.shared_venue(a, b, p3), sim.shared_venue(b, a, p3));
        assert_ne!(
            sim.shared_venue(a, b, p3),
            sim.shared_venue(a, b, p4),
            "two shippers in different places must not be forced to one compromise rock"
        );
    }

    /// **T-82: the `$` ledger fills and nothing reads it.**
    ///
    /// §10.7 stages 1–3 land the Exchange **inert**, for the reason
    /// `Hyades_industry.md` §6.7's stages 3–5 did: a system that lands neutral
    /// can be verified against a bit-identical bed before anything switches on,
    /// so the first stage that *does* change behaviour is measured against a
    /// known baseline instead of against a moving one. That plan was not met at
    /// industry stage 5, and the stage that broke it was the one that changed
    /// what a purchase costs — which is stage 4 here.
    ///
    /// Two claims, and the second is the one that can rot silently:
    ///
    /// - **The faucet runs.** An empire that fabricates accrues `$`, so the
    ///   ledger is not merely present and empty.
    /// - **Nothing spends it.** The purse was write-only when this was written;
    ///   **T-85 made it readable**, so the test now runs with the Exchange
    ///   disabled and the claim narrows to what it always meant: the *faucet*
    ///   alone moves nothing. That is a retarget rather than a weakening — with
    ///   trade on, a zero rate stops trade and moves the world through that
    ///   instead, which is a different claim.
    #[test]
    fn the_dollar_ledger_fills_and_changes_nothing() {
        let run = |rate: f64| {
            let mut gcfg = GalaxyConfig::new(3, 21);
            gcfg.planet_count = 300;
            let galaxy = Galaxy::generate(gcfg).unwrap();
            let mut cfg = test_cfg(21);
            cfg.dollar_per_fabrication = rate;
            let mut sim = Simulation::with_baseline(galaxy, cfg);
            // **Isolate the faucet.** Since T-85 the purse *is* read — clearing
            // checks what a buyer can afford — so a zero rate would stop trade
            // and move the world through that, which is a different claim than
            // the one this test makes. The Exchange off, the faucet is again the
            // only variable.
            sim.set_exchange_enabled(false);
            let report = sim.run();
            let purses: Vec<u64> = (0..3).map(|p| sim.purse_of(PlayerId(p)).to_bits()).collect();
            let pops: Vec<u64> = report.players.iter().map(|x| x.total_population.kilotons().to_bits()).collect();
            (report.events_processed, report.planets_scanned_total, pops, purses)
        };

        let (ev_off, scan_off, pop_off, purse_off) = run(0.0);
        let (ev_on, scan_on, pop_on, purse_on) = run(1.0);

        // The faucet is doing something — otherwise everything below is
        // vacuously true and the test would still pass with `credit` deleted.
        assert!(purse_off.iter().all(|&b| f64::from_bits(b) == 0.0), "a zero rate must mint nothing");
        assert!(
            purse_on.iter().all(|&b| f64::from_bits(b) > 0.0),
            "every empire fabricates, so every purse must fill: {:?}",
            purse_on.iter().map(|&b| f64::from_bits(b)).collect::<Vec<_>>()
        );

        // And it is reading *production*, not merely counting ticks: an empire
        // that has built more infrastructure fabricates faster and must be
        // richer. Sorting the two orderings together is what makes this a claim
        // about the faucet's input rather than about a constant.
        assert!(
            purse_on.iter().map(|&b| f64::from_bits(b)).any(|v| v != f64::from_bits(purse_on[0])),
            "three empires with different industry must not hold identical purses"
        );

        // **Nothing reads it.** Same seed, same run, to the bit.
        assert_eq!(ev_off, ev_on, "the faucet moved the event count");
        assert_eq!(scan_off, scan_on, "the faucet moved survey");
        assert_eq!(pop_off, pop_on, "the faucet moved population");
    }

    /// **A non-finite credit is refused at the write** (T-82, design law #16).
    ///
    /// The purse is replicated state, and §6 H3 is explicit that core WASM picks
    /// NaN payloads nondeterministically — so a NaN reaching the hashed state is
    /// an intermittent desync with no reproducer to hand a bug report. Guarding
    /// at the single writer is cheaper than guarding at the digest and cannot be
    /// bypassed by a new call site.
    #[test]
    fn a_non_finite_credit_never_reaches_the_ledger() {
        let mut sim = Simulation::with_baseline(Galaxy::generate(GalaxyConfig::new(2, 9)).unwrap(), test_cfg(9));
        let pe = sim.player_entity[0];
        sim.credit(pe, 10.0);
        assert_eq!(sim.purse_of(PlayerId(0)), 10.0);

        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            sim.credit(pe, bad);
            assert!(sim.purse_of(PlayerId(0)).is_finite(), "{bad} reached the ledger");
            assert_eq!(sim.purse_of(PlayerId(0)), 10.0, "{bad} moved the balance");
        }

        // Debt is legal — `$` is a claim, and an obligation can be negative.
        // Only *not a number* is refused.
        sim.credit(pe, -25.0);
        assert_eq!(sim.purse_of(PlayerId(0)), -15.0, "a negative balance is a legal claim");
    }

    /// **Crew falls out of demand, and the sign is inverted from both retired
    /// policies** (T-87).
    ///
    /// The properties, not the numbers — the numbers are `ε`, `β` and
    /// `veins_per_band`, all placeholders (R-IND18). What has to hold whatever
    /// they become:
    ///
    /// - **No unmet demand, one hull.** A centre that can already afford what it
    ///   wants does not buy a mine, it buys a mine's minimum.
    /// - **A richer rock wants a *smaller* crew.** This is the inversion. Both
    ///   retired policies made crew rise with richness — a flat count was
    ///   richness-blind and a vein fraction rose with `N` — and §6.15 measured
    ///   both as losses. A rich body meets the same demand with fewer hands.
    /// - **More demand, more crew**, monotonically, up to the body's veins.
    #[test]
    fn a_mining_crew_is_derived_from_demand_and_shrinks_on_richer_rock() {
        let mut sim = Simulation::with_baseline(Galaxy::generate(GalaxyConfig::new(2, 3)).unwrap(), test_cfg(3));
        let center = sim.world.player_info.get(sim.player_entity[0]).unwrap().home;
        let rock = sim.planet_entity[10];

        let set_ore = |sim: &mut Simulation, band: f64| {
            let each = units::Kilotons::at_band(Band::new(band)).kilotons() / 3.0;
            let d = sim.world.density.get_mut(rock).unwrap();
            for b in Basic::ALL {
                d.set(b, units::Kilotons::new(each));
            }
        };

        // **A comfortable centre has no unmet demand.** `mineral_pressure` is
        // zero while the bank covers the next rung, so `D = 0` whatever the
        // yard could fabricate.
        {
            let bank = sim.world.stockpile.get_mut(center).unwrap();
            bank.cyan = 10_000.0;
            bank.magenta = 10_000.0;
            bank.yellow = 10_000.0;
        }
        set_ore(&mut sim, 3.0);
        assert_eq!(sim.mining_crew_for(center, rock), 1, "a centre that can afford its rung wants one hull");

        // Starve it: pressure goes to 1 and the crew is whatever meets the
        // yard's throughput.
        {
            let bank = sim.world.stockpile.get_mut(center).unwrap();
            bank.cyan = 0.0;
            bank.magenta = 0.0;
            bank.yellow = 0.0;
        }
        {
            let f = sim.world.factors.get_mut(center).unwrap();
            f.infra = infra_rung_price(1, &sim.config);
        }
        assert!(sim.mineral_pressure_of(center) > 0.99, "a broke centre must read full pressure");

        // **The crew never exceeds the body's veins**, at any richness. This is
        // the constraint that binds at the bottom of the ladder and it is why
        // the inversion below is stated over the range above it: a `Band I`
        // pebble has one vein, so it gets one hull because there is nowhere to
        // put a second — not because demand said so.
        for band in [0.0, 1.0, 2.0, 3.0, 4.0] {
            set_ore(&mut sim, band);
            let stock = sim.world.density.get(rock).unwrap().total_mass();
            let crew = sim.mining_crew_for(center, rock);
            assert!(crew >= 1, "a crew is at least one hull");
            assert!(
                crew as f64 <= veins(stock, &sim.config) + 0.5,
                "Band {band} has {} veins and asked for {crew}",
                veins(stock, &sim.config)
            );
        }

        // **The inversion**: same demand, richer rock, smaller crew — over the
        // range where *demand* is what binds. Both retired policies had this
        // sign backwards, and §6.15 measured both as losses.
        let mut last = usize::MAX;
        for band in [2.0, 3.0, 4.0] {
            set_ore(&mut sim, band);
            let crew = sim.mining_crew_for(center, rock);
            assert!(
                crew <= last,
                "a richer rock must not want a larger crew: Band {band} asked for {crew} after {last}"
            );
            last = crew;
        }
        assert!(last < 2, "a Band IV seam meets a rung-I yard's demand with a single hull, got {last}");

        // **Monotone in demand.** Raising what the yard can absorb cannot lower
        // the crew, on a body poor enough that the veins are not the binding
        // constraint.
        set_ore(&mut sim, 1.0);
        let lean = sim.mining_crew_for(center, rock);
        {
            let f = sim.world.factors.get_mut(center).unwrap();
            f.infra = infra_rung_price(4, &sim.config);
        }
        assert!(sim.mining_crew_for(center, rock) >= lean, "a hungrier yard must not want fewer miners: {lean}",);
    }

    /// **§4.2's vein table and §4.3's worked ratios, pinned** (T-71).
    ///
    /// The design's numbers, not the implementation's: a decade of veins per
    /// Band, and on `Band IV` a thousand miners lift **31.6x** what one does.
    /// That second figure is the one §4.3 uses to argue a large crew on a rich
    /// rock is rational, and it is the figure `crowding_factor`'s normalisation
    /// had to preserve (R-IND19) — so it is asserted through the normalised
    /// form, which is what the engine actually multiplies by.
    #[test]
    fn veins_are_a_decade_per_band_and_crowding_pays_at_scale() {
        let cfg = SimConfig::new(1);
        for (rung, want) in [(1.0, 1.0), (2.0, 10.0), (3.0, 100.0), (4.0, 1000.0)] {
            let mass = units::Kilotons::at_band(Band::new(rung));
            let got = veins(mass, &cfg);
            assert!(
                (got / want - 1.0).abs() < 1e-6,
                "Band {rung} holds {mass:?} and should have {want} veins, got {got}"
            );
        }

        // §4.3, verbatim: "on `Band IV`, one miner does `W = 31.6` and a
        // thousand do `W = 1000` — 31.6x the ore for 1000x the hulls".
        let rich = units::Kilotons::at_band(Band::new(4.0));
        let lone = crowding_factor(1.0, rich, &cfg);
        let full = crowding_factor(1000.0, rich, &cfg);
        assert!((full / lone - 31.6).abs() < 0.1, "1000 miners should lift 31.6x one, got {}", full / lone);

        // **Richness is worth going to.** A lone miner on `Band IV` works a
        // smaller *share* of the body than one on `Band I` — that is crowding —
        // and lifts vastly more ore, because the body is vastly larger. Both
        // halves are the design; only the second makes an outpost worth siting.
        let poor = units::Kilotons::at_band(Band::new(1.0));
        assert!(crowding_factor(1.0, rich, &cfg) < crowding_factor(1.0, poor, &cfg), "share must fall with richness");
        assert!(
            crowding_factor(1.0, rich, &cfg) * rich.kilotons() > crowding_factor(1.0, poor, &cfg) * poor.kilotons(),
            "absolute yield must rise with richness"
        );

        // A full crew takes the same share of any body, which is the
        // normalisation that stops output going as richness squared.
        for rung in [1.0, 2.0, 3.0, 4.0] {
            let mass = units::Kilotons::at_band(Band::new(rung));
            let n = veins(mass, &cfg);
            assert!((crowding_factor(n, mass, &cfg) - 1.0).abs() < 1e-9, "a full crew works the whole body at {rung}");
        }
    }

    /// **T-71: extraction is per miner and sublinear, and the deposit sets where
    /// the crowding bites.**
    ///
    /// Two prior laws, both superseded, and the shape of the correction is the
    /// interesting part. Before T-57 `outpost_mining_fraction` was the fraction
    /// a *rock* yielded per tick, with the hull standing on it contributing
    /// nothing but the schedule — so "how many miners per outpost" had no term
    /// to tune. T-57 made it linear in crew, which gave it one. T-71 makes it
    /// **sublinear and relative to the body**: `(n/N(S))^β`.
    ///
    /// **This test used to assert exact linearity and §4.5 says that does not
    /// survive** — T-57's ratified `miners_per_outpost = 3` was measured under a
    /// law with no deposit term at all, and the right crew is now a property of
    /// the rock rather than a constant. It asserts the new law's *properties*
    /// rather than three numbers: sublinear, monotone, saturating at `N`.
    #[test]
    fn a_mining_crew_extracts_in_proportion_to_its_size() {
        let extracted = |crew: usize| {
            let mut sim = Simulation::with_baseline(Galaxy::generate(GalaxyConfig::new(2, 3)).unwrap(), test_cfg(3));
            let outpost = sim.planet_entity[10];
            // **Pin the ore rather than inherit it.** This test is about the
            // *proportionality* of the crew, so which rock the generator put at
            // index 10 is a confound: T-62's log-normal field plus a widened
            // `Band Empty` floor made that particular world barren, and the test
            // failed on "a single miner must extract something" — a true
            // statement about a dead rock and nothing at all about crews.
            {
                let d = sim.world.density.get_mut(outpost).unwrap();
                for b in Basic::ALL {
                    d.set(b, Kilotons::new(1.0));
                }
            }
            let before = sim.world.density.get(outpost).unwrap().total_mass().kilotons();
            // A crew is just entities on the books; what they extract is what
            // this test is about, so they need no voyage.
            let hulls: Vec<Entity> = (0..crew).map(|_| sim.world.spawn()).collect();
            sim.mine_crew.insert((0, outpost.0), hulls);
            sim.sys_mining_tick(outpost);
            before - sim.world.density.get(outpost).unwrap().total_mass().kilotons()
        };

        let cfg = SimConfig::new(3);
        let one = extracted(1);
        assert!(one > 0.0, "a single miner must extract something");

        // **More crew is more ore, and each one is worth less than the last.**
        // Both halves matter: monotone is what makes a crew worth sending,
        // sublinear is what stops the answer being "always send more".
        let (two, three) = (extracted(2), extracted(3));
        assert!(two > one && three > two, "crew must be monotone: {one} {two} {three}");
        assert!(two < 2.0 * one, "two miners must be worth less than twice one: {}", two / one);
        assert!(three - two < two - one, "the marginal miner must be worth less than the last");

        // The exponent is exactly what β says, on a body with veins to spare.
        // 3 kt of ore reads well above `Band I`, so `N` is not the binding
        // constraint at these crew sizes and the pure `n^β` shows through.
        let want = 2f64.powf(cfg.crowding_beta);
        assert!((two / one - want).abs() < 1e-9, "two miners: {} want {want}", two / one);

        // **And a body saturates at its own vein count, not at a crew size.**
        // That is the whole of §4.2: a bare `n^β` would crowd every rock
        // identically and nobody would ever put a large crew anywhere.
        let deposit = Kilotons::new(3.0);
        let n_veins = veins(deposit, &cfg);
        let full = n_veins.ceil() as usize;
        assert!(
            (extracted(full) - extracted(full * 4)).abs() < 1e-9,
            "past `N` the rock binds, not the crew: {} vs {}",
            extracted(full),
            extracted(full * 4)
        );
        // **A degenerate config must produce an absurd number, not a divergent
        // one** (design law #16). `veins_per_band < 1` makes `powf` of a
        // negative exponent blow up, and the result is divided by — so an
        // infinity here is a NaN one subtraction later, in replicated state,
        // with no reproducer.
        let mut bad = SimConfig::new(3);
        bad.veins_per_band = 0.0;
        for rung in [0.0, 1.0, 4.0] {
            let mass = units::Kilotons::at_band(Band::new(rung));
            let n = veins(mass, &bad);
            assert!(n.is_finite() && n >= 1.0, "degenerate veins_per_band gave {n} at Band {rung}");
            assert!(crowding_factor(3.0, mass, &bad).is_finite(), "crowding diverged at Band {rung}");
        }

        // A full crew takes exactly `ε` of the body — the normalisation that
        // keeps a rich seam finite (`crowding_factor`, R-IND19).
        // Three colours at 1 kt each — the same body `deposit` names above.
        let before = deposit.kilotons();
        assert!(
            (extracted(full) / before - cfg.outpost_mining_fraction).abs() < 1e-9,
            "a full crew must take ε of the body, took {}",
            extracted(full) / before
        );
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
            let a = hull_base_thrust(hull, &cfg) / hull_dry_mass(hull, &cfg).kilotons();
            cfg.general_vehicle_cost = 17.0; // any scale at all
            let b = hull_base_thrust(hull, &cfg) / hull_dry_mass(hull, &cfg).kilotons();
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

    /// **T-74: the rate curve saturates, is monotone, and its two knobs mean
    /// what §6.3 says they mean.**
    ///
    /// `Hyades_industry.md` §6.7's test 4. Cheap, and it pins the two
    /// parameters to their stated roles so a later retune cannot quietly swap
    /// them — which matters because the tall/wide axis *is* those two knobs:
    /// Production raises `cap`, Growth and Expansion lower `half`.
    #[test]
    fn the_rate_curve_saturates_and_is_monotone() {
        let cfg = SimConfig::new(1);
        let w = cards::Works::default();
        let e = cards::Employment::Fabrication;
        let (cap, half) = (cfg.fab_cap, works_knee(&cfg));
        let rate = |infra_kt: f64| employment_rate(Price::new(infra_kt), &w, e, cap, half);

        // Zero stock, zero rate — the curve passes through the origin, so a
        // razed world fabricates nothing rather than falling back to a floor.
        assert_eq!(rate(0.0), 0.0);

        // Monotone increasing, and never past the ceiling.
        let mut prev = 0.0;
        for i in 1..400 {
            let r = rate(i as f64 * 0.01);
            assert!(r > prev, "rate must rise with the stock at {i}");
            assert!(r < cap, "rate must never reach the ceiling: {r} vs {cap}");
            prev = r;
        }

        // **`half` is the knee**: `u = half` gives exactly `cap/2`. `u` is the
        // employment's *share*, so at even allocation that is three knees of
        // total stock.
        let at_knee = rate(half * 3.0);
        assert!((at_knee - cap / 2.0).abs() < 1e-12, "u = half must give cap/2, got {at_knee}");

        // **Production raises the ceiling; Growth lowers the knee.** Both make a
        // planet faster, and they do it differently — which is the whole reason
        // one curve carries the tall/wide distinction.
        let tall = cards::Works { cap: [1.0, 3.0, 1.0], ..cards::Works::default() };
        let wide = cards::Works { half: [1.0, 0.25, 1.0], ..cards::Works::default() };
        let stock = Price::new(half * 3.0);
        let base = employment_rate(stock, &w, e, cap, half);
        let t = employment_rate(stock, &tall, e, cap, half);
        let d = employment_rate(stock, &wide, e, cap, half);
        assert!(t > base && d > base, "both routes must beat the base at the knee");

        // And they diverge where the design says: far up the stock the tall
        // route wins outright, because it moved the asymptote and the wide one
        // only got there sooner.
        let far = Price::new(half * 300.0);
        assert!(
            employment_rate(far, &tall, e, cap, half) > employment_rate(far, &wide, e, cap, half),
            "a raised ceiling must beat a lowered knee once the stock is large"
        );
    }

    /// **T-70: infrastructure is a stock of minerals; the rung is a reading.**
    ///
    /// It was a `Band` — a position on a ladder, stored — which is the thing
    /// `CLAUDE.md` §4 says never to do: *a Band is a reading, not a second thing
    /// to store*. `Hyades_industry.md` §1.3 states the same rule for this
    /// quantity specifically, because infrastructure is **built out of
    /// minerals** and minerals are masses (L6/R-O57).
    ///
    /// Three things are pinned, and the third is the one that would have been a
    /// silent disaster:
    ///
    /// 1. standing at rung `n` means holding exactly what rung `n` costs;
    /// 2. buying a rung moves the stock by exactly `infra_step_price`, so the
    ///    ladder is the single source of the number rather than an increment
    ///    that happens to agree with it;
    /// 3. **the reading is taken on the *Cost* ladder, not the mass ladder.**
    ///    `Price` is kilotons, so `in_bands()` compiles and returns a completely
    ///    different rung — the two ladders are `^1.5` apart (R-MC15). Reading
    ///    infrastructure on the wrong one would move every development gate at
    ///    once and typecheck while doing it, which is precisely the shape of the
    ///    `K = min(hab, bio, infra)` unit error this project already paid for.
    #[test]
    fn infrastructure_is_a_stock_and_the_rung_is_a_reading() {
        let mut sim = Simulation::with_baseline(Galaxy::generate(GalaxyConfig::new(2, 5)).unwrap(), test_cfg(5));
        let home = sim.world.player_info.get(sim.player_entity[0]).unwrap().home;

        for rung in 1..=4usize {
            let stock = infra_rung_price(rung, &sim.config);
            let f = sim.world.factors.get_mut(home).unwrap();
            f.infra = stock;
            assert_eq!(infra_rung_of(stock, &sim.config), rung, "standing at rung {rung} must read back as {rung}");

            // Buying the next rung moves the stock by exactly the step price.
            let step = infra_step_price(stock, &sim.config);
            let next = infra_rung_price(rung + 1, &sim.config);
            assert!(
                ((stock + step) - next).kilotons().abs() < 1e-12,
                "rung {rung} + step must land on rung {}: {} vs {next}",
                rung + 1,
                stock + step
            );
        }

        // **The two ladders are different, and the type is what keeps them
        // apart.** This is an assertion that the wrong reading is *available*
        // and wrong — the compile-time guard is the `Scale` marker, and this is
        // the runtime evidence that it is load-bearing rather than decorative.
        let stock = infra_rung_price(2, &sim.config);
        let on_cost = stock.band_from(cost_anchor(&sim.config));
        let on_mass = stock.on_scale::<units::Mass>().in_bands();
        assert!(
            (on_cost.bands() - on_mass.bands()).abs() > 0.1,
            "the same kilotons must read a different rung on each ladder: {on_cost} vs {on_mass}"
        );
    }

    /// **Why T-70 came out bit-identical, which was not the prediction.**
    ///
    /// `Hyades_industry.md` §6.7 predicted stage 3 would be neutral; §6.8 then
    /// argued it could not be, because storing the stock moves the rounding out
    /// of exact Band-space addition (`infra.up(1.0)`) and into a `ln` round trip
    /// (`band_from(rung_price(n))`), and those differ in the last bits. The
    /// arithmetic half of that is true. The conclusion was wrong, and the guard
    /// run said so: colony-years came back **identical to the decimal** on both
    /// seeds.
    ///
    /// The mechanism is that **infrastructure reaches every live decision
    /// through an integer**, so a difference of ~1e-12 in the Band reading is
    /// washed out before it can change anything:
    ///
    /// - `infra_rung_of` **rounds**, and it is what `infra_step_price` and
    ///   `mineral_pressure_of` are built on — every pricing path.
    /// - `BaselineAutopilot::rank` does not read infrastructure at all. It
    ///   scores `k_potential`, minerals and position.
    /// - The one continuous reader is `deepen_headroom = k_potential − infra`,
    ///   and **R-O68 measured that branch as dead at the shipped
    ///   `reinvest_bias = 0.5`** — it cannot fire while any candidate exists.
    ///
    /// So this test pins the actual invariant rather than the lucky number: the
    /// two representations disagree in the Band, agree in the rung, and
    /// therefore agree in the price. If a future change makes a *continuous*
    /// reader of infrastructure live — which is exactly what fixing R-O68 would
    /// do — this stops being true, and the failure will point at the reason.
    #[test]
    fn infrastructure_reaches_every_decision_through_an_integer() {
        let sim = Simulation::with_baseline(Galaxy::generate(GalaxyConfig::new(2, 5)).unwrap(), test_cfg(5));
        let cfg = &sim.config;

        for hull in [HullType::MediumSystems, HullType::GeneralSystems] {
            // What the old code stored after founding: the Band of the hull's
            // minerals. What the new code stores: those minerals.
            let old_band = hull_cost(hull, cfg).band_from(cost_anchor(cfg));
            let new_stock = sim.founding_infra(hull);
            let new_band = new_stock.band_from(cost_anchor(cfg));
            assert_eq!(old_band.bands().to_bits(), new_band.bands().to_bits(), "founding reads identically");

            // Now climb. The old code added 1.0 in Band space; the new code
            // buys the next rung. These are *not* the same f64 …
            let top = BandTier::MAX_PLAYABLE.band().bands() as usize;
            let mut old = old_band;
            let mut stock = new_stock;
            while infra_rung_of(stock, cfg) < top {
                old = old.up(1.0);
                stock = infra_rung_price(infra_rung_of(stock, cfg) + 1, cfg);
                let new = stock.band_from(cost_anchor(cfg));

                // … and the rung they round to is, which is the only thing any
                // live path reads.
                assert_eq!(
                    old.round().bands() as usize,
                    infra_rung_of(stock, cfg),
                    "{hull:?}: the rung must agree even when the Band does not (old {old}, new {new})"
                );
                assert_eq!(
                    infra_step_price(stock, cfg).kilotons().to_bits(),
                    (infra_rung_price(infra_rung_of(stock, cfg) + 1, cfg)
                        - infra_rung_price(infra_rung_of(stock, cfg), cfg))
                    .kilotons()
                    .to_bits(),
                    "and the price follows the rung, not the Band"
                );
            }

            // **The one place the two representations genuinely differ, stated
            // rather than glossed.** The old Band climbed without limit — an
            // `up(1.0)` on a position has no ceiling — while the stock
            // saturates at the top playable rung, because the ladder does. It
            // is invisible in play, and for a reason that is checked rather
            // than assumed: deepening is gated on `infra < k_potential`, and
            // `k_potential = min(hab, bio_max)` cannot exceed the top rung. So
            // nothing in a shipped run ever reaches the difference.
            //
            // This is the better behaviour of the two — an unbounded
            // infrastructure Band was a quantity with no meaning past `Band IV`
            // — but it is a change, and it is the reason to keep the gate.
            let over = infra_rung_price(top + 3, cfg);
            assert_eq!(infra_rung_of(over, cfg), top, "the stock saturates at the top playable rung");
            // The old Band had no such ceiling — `up(1.0)` on a position climbs
            // forever — which is the difference this comment exists to record.
            // It is not asserted, because an assertion about deleted code would
            // be an assertion that cannot fail.
        }
    }

    /// **T-73/§6.4: a mix card moves the colour mix and never lowers the total.**
    ///
    /// `eta_works` is multiplicative on the total; `mix_w` is a *share* of that
    /// total. So the two are orthogonal **by construction**, which is what makes
    /// §5.4's rule — *move the mix, never lower the total* — enforceable in a
    /// type rather than in review. Without it the card list becomes a discount
    /// race and "shift the mix away from pure Yellow" degrades into a worse way
    /// of saying "make it cheaper".
    #[test]
    fn a_mix_card_cannot_change_the_total() {
        let sim = Simulation::with_baseline(Galaxy::generate(GalaxyConfig::new(2, 5)).unwrap(), test_cfg(5));
        let step = infra_step_price(infra_rung_price(1, &sim.config), &sim.config);

        // Every mix a card could reach, including the sole-colour `1:0:0` that
        // §5.1 allows works and forbids card costs, and degenerate weights.
        let mixes: [[f64; 3]; 6] =
            [[1.0, 1.0, 1.0], [4.0, 2.0, 1.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0], [5.0, 4.0, 3.0], [1e-6, 1.0, 1e6]];
        for m in mixes {
            let w = cards::Works { mix_w: m, ..cards::Works::default() };
            let bill = works_bill(step, &w);
            let total: Price = bill.iter().fold(Price::ZERO, |a, &b| a + b);
            assert!(
                (total - step).kilotons().abs() < 1e-12,
                "mix {m:?} changed the bill: {total} against a step of {step}"
            );
            for b in bill {
                assert!(b >= Price::ZERO, "mix {m:?} produced a negative colour share");
            }
        }

        // And efficiency is the *other* axis: `eta_works` moves the total and
        // leaves the split alone.
        let base = works_bill(step, &cards::Works::default());
        let eff = cards::Works { eta_works: 2.0, ..cards::Works::default() };
        let bill = works_bill(step, &eff);
        let total: Price = bill.iter().fold(Price::ZERO, |a, &b| a + b);
        assert!((total - step * 0.5).kilotons().abs() < 1e-12, "eta_works must halve the bill, got {total}");
        // **The split is unchanged** — asserted as *shares*, not as three equal
        // numbers. The identity mix is `3:2:1` Y:C:M and never was even, so an
        // equality test here would have been checking the old placeholder
        // rather than the invariant.
        for i in 0..3 {
            let (a, b) = (bill[i] / total, base[i] / (base.iter().fold(Price::ZERO, |a, &b| a + b)));
            assert!((a - b).abs() < 1e-12, "an efficiency card must not move the mix: share {i} went {b} -> {a}");
        }
    }

    /// **T-73: a colour-poor centre cannot buy the rung, however rich it is.**
    ///
    /// This is the whole point of the stage, and the thing no total-based
    /// affordability test can express. `Minerals::try_spend_total` debits
    /// *proportional to holdings*, so before this a bank of pure Cyan could buy
    /// anything; now a bill naming Yellow needs Yellow. That is §5.1's *"the
    /// mechanism that makes the galaxy's mineral distribution bite on
    /// development"* — and T-62 made the field log-normal, so which colours a
    /// homeworld sits near is an enormous fact that until now went almost
    /// entirely unexpressed.
    ///
    /// Both halves are asserted, because only the pair is meaningful: the poor
    /// centre is refused, and a centre with the *same total* spread across the
    /// colours the bill names is not.
    #[test]
    fn a_colour_poor_centre_cannot_buy_the_rung() {
        let step = Price::new(9.0);
        let works = cards::Works::default(); // even thirds: 3.0 of each
        let bill = works_bill(step, &works);

        // A hundred times the bill, and all the wrong colour.
        let hoard = Minerals { cyan: 900.0, ..Default::default() };
        assert!(!can_pay_bill(&hoard, &bill), "a pure-Cyan hoard must not buy a bill that names Magenta and Yellow");

        // Exactly the bill, in the proportions the bill actually names — which
        // is `3:2:1` Y:C:M since the default mix was ratified, not even thirds.
        let mut spread = Minerals {
            cyan: bill[0].kilotons(),
            magenta: bill[1].kilotons(),
            yellow: bill[2].kilotons(),
            ..Default::default()
        };
        assert!(can_pay_bill(&spread, &bill), "exactly the bill, in the right colours, must pay");
        assert!(spread.basic_total() < hoard.basic_total(), "and it is the *poorer* bank that can afford it");

        // Paying takes each colour's share and nothing else.
        pay_bill(&mut spread, &bill);
        assert!(spread.basic_total().kilotons().abs() < 1e-9, "the bill should have emptied it exactly");

        // A sole-colour work — `1:0:0`, which §5.1 allows works and forbids card
        // costs — is payable only by an empire that has that colour.
        let yellow_only = cards::Works { mix_w: [0.0, 0.0, 1.0], ..cards::Works::default() };
        let y_bill = works_bill(step, &yellow_only);
        assert!(!can_pay_bill(&hoard, &y_bill), "Production's route is closed to a Yellow-poor empire");
        let yellow = Minerals { yellow: 9.0, ..Default::default() };
        assert!(can_pay_bill(&yellow, &y_bill), "and open to one that has Yellow");
    }

    /// **T-68: build time tracks mass, and the two ladders are one ladder.**
    ///
    /// `build_years` was flat at 10.0, so a Limited hull and a General hull took
    /// the same ten years across a **50x** mass ratio — which made the yard
    /// blind to what it was making and gave a General hull no temporal cost at
    /// all. `t_build = t_lead + m / F_slip` (`Hyades_industry.md` §3.2), and
    /// because dry mass *is* mineral cost (R-O57) there is no second ladder to
    /// keep in step.
    ///
    /// Pins the **approved starting schedule** of §3.3 — 2.2 / 3.0 / 12.0 yr —
    /// which is where the design wants it: a scout is a season's work, a
    /// coloniser is quick enough to spam, and a General hull is a twelve-year
    /// commitment an opponent has time to notice and answer. These are approved
    /// values, **not MC-ratified**; the name says placeholder and so does §3.3.
    ///
    /// **Since T-69 the schedule is a floor rather than a reading.** Slips
    /// divide a yard's throughput, so those three numbers are `t_lead +
    /// m/F_slip` — the limit an arbitrarily industrialised yard descends
    /// toward and never reaches. What T-74 pins here instead is the *anchor*:
    /// a rung-I centre with default doctrine fabricates at exactly the flat
    /// rate T-68 and the mining model shipped, so the rate curve pivots about
    /// a configuration that was already ratified.
    #[test]
    fn build_time_is_lead_plus_mass_over_throughput() {
        let mut sim = Simulation::with_baseline(Galaxy::generate(GalaxyConfig::new(2, 5)).unwrap(), test_cfg(5));
        // **T-74 made `t_build` a property of the yard, so the schedule needs a
        // yard to be read at — and the one it holds at is the anchor.** A centre
        // standing at rung I with default doctrine sits exactly on the knee of
        // the rate curve, where fabrication is `fab_cap/2 = slip_throughput`.
        // So §3.3's approved schedule is not merely preserved by T-74, it is
        // *what pins the calibration*: if this passes, the curve pivots about
        // the configuration that was already ratified.
        let yard = sim.world.player_info.get(sim.player_entity[0]).unwrap().home;
        {
            let f = sim.world.factors.get_mut(yard).unwrap();
            f.infra = infra_rung_price(1, &sim.config);
        }
        assert!(
            (sim.fabrication_rate(yard) - sim.config.slip_throughput).abs() < 1e-12,
            "a rung-I centre must fabricate at exactly the old flat rate, got {}",
            sim.fabrication_rate(yard)
        );
        // **§3.3's schedule is the limit, not a value any yard reaches** — T-69
        // is what made that distinction real. Before slips, `t_build` was
        // `t_lead + m/F` and a rung-I centre hit 2.2 / 3.0 / 12.0 on the nose.
        // Slips divide the throughput, so the table is now the **asymptote**
        // the curve descends toward and every real yard sits above it. Assert
        // the schedule as what it is — the two constants, composed — and assert
        // separately that a yard approaches it without arriving.
        for (hull, want) in
            [(HullType::LimitedSystems, 2.2), (HullType::MediumSystems, 3.0), (HullType::GeneralSystems, 12.0)]
        {
            let m = hull_cost(hull, &sim.config).on_scale::<units::Mass>().kilotons();
            let limit = sim.config.build_lead_years + m / sim.config.slip_throughput;
            assert!((limit - want).abs() < 1e-9, "{hull:?} floor is {limit} yr, schedule says {want}");
            let got = sim.build_time(yard, hull_cost(hull, &sim.config));
            assert!(got > want, "{hull:?} builds in {got} yr, under its own floor {want}");
        }

        // **The lead time is a floor and the mass term is strictly monotone.**
        // Asserted as properties rather than as three more numbers, so a
        // ratification of either constant cannot quietly invert the ordering the
        // observation model leans on — a big hull must stay a long, visible
        // commitment (§3.2).
        assert!((sim.build_time(yard, Price::ZERO) - sim.config.build_lead_years).abs() < 1e-12);
        let (l, m, g) = (
            sim.build_time(yard, hull_cost(HullType::LimitedSystems, &sim.config)),
            sim.build_time(yard, hull_cost(HullType::MediumSystems, &sim.config)),
            sim.build_time(yard, hull_cost(HullType::GeneralSystems, &sim.config)),
        );
        assert!(l < m && m < g, "time must order like mass: {l} {m} {g}");

        // And it is one expression over every order, not a hull table: an
        // infrastructure rung costs minerals, minerals are mass, so a rung has a
        // build time by the same rule. `Infra I` is priced as a Medium hull
        // (R-O80), so it takes a Medium hull's time.
        assert!(
            (sim.build_time(yard, infra_rung_price(1, &sim.config)) - m).abs() < 1e-9,
            "a rung priced like a Medium hull takes a Medium hull's time"
        );
    }

    /// **The soft floor is the whole point of §3.2, so pin the floor and not the
    /// formula** (T-69).
    ///
    /// `slips` exists so that industry buys *concurrency* and never a faster
    /// single hull. That claim is exactly `t_build ≥ t_lead + m / F_slip` for
    /// every yard, however rich — equivalently, per-berth throughput stays
    /// strictly *under* `F_slip` and climbs toward it. The reciprocal is what
    /// makes it easy to get backwards, and getting it backwards is not a
    /// cosmetic error: dropping the `1 +` from `slips` lets a rung-II centre
    /// build a Medium hull in 2.55 yr against a 3.0 yr floor, which deletes the
    /// design property while still passing any number-by-number schedule test.
    /// So this asserts the inequality, and the schedule test below asserts the
    /// limit it approaches.
    #[test]
    fn industry_buys_concurrency_and_never_undercuts_the_turnaround_floor() {
        let cfg = test_cfg(3);
        let per_slip = cfg.slip_throughput;
        let mass = Price::new(0.1);
        let floor = cfg.build_lead_years + mass.on_scale::<units::Mass>().kilotons() / per_slip;

        // Sweep across many multiples *and* straddle each one, because a
        // boundary is where an off-by-one in `floor` would hide.
        let mut last_slips = 0usize;
        for step in 0..400 {
            let f = per_slip * (step as f64) * 0.125;
            let n = slips(f, &cfg);
            assert!(n >= 1, "a centre always has a berth, F={f} gave {n}");
            assert!(n >= last_slips, "concurrency must not fall as throughput rises at F={f}");
            last_slips = n;
            let per_berth = f / n as f64;
            assert!(per_berth < per_slip, "per-berth throughput {per_berth} reaches the ceiling {per_slip} at F={f}");
        }
        // Concurrency itself is unbounded — that is the "build wide" axis.
        assert!(slips(100.0 * per_slip, &cfg) > slips(10.0 * per_slip, &cfg));

        // Read through `build_time`, since that is what the rest of the engine
        // sees: strictly above the floor at every rung, and monotonically
        // approaching it as the yard grows.
        let mut sim = Simulation::with_baseline(Galaxy::generate(GalaxyConfig::new(2, 3)).unwrap(), test_cfg(3));
        let yard = sim.world.player_info.get(sim.player_entity[0]).unwrap().home;
        let mut prev = f64::INFINITY;
        for rung in 1..=4 {
            {
                let f = sim.world.factors.get_mut(yard).unwrap();
                f.infra = infra_rung_price(rung, &sim.config);
            }
            let t = sim.build_time(yard, mass);
            assert!(t > floor, "rung {rung} builds in {t} yr, under the floor {floor}");
            assert!(t < prev, "a richer yard must not turn a hull around more slowly: {prev} -> {t}");
            prev = t;
        }
        assert!(prev < floor * 1.35, "four rungs should be well down the curve, got {prev} against {floor}");
    }

    /// **A yard with slips uses them** (T-69).
    ///
    /// Concurrency that is bought and not spent is worse than no concurrency at
    /// all: `slips` divides a yard's throughput among its berths, so a decision
    /// that commits one build and returns leaves the extra berths idle *and*
    /// the occupied one running at a fraction of the rate. That is all of the
    /// cost and none of the benefit, and it would have measured as a clean
    /// regression with a completely wrong mechanism attached.
    ///
    /// Asserted against `slips` rather than against the number 2, so it keeps
    /// meaning the same thing if `fab_cap` or `slip_throughput` is ratified.
    #[test]
    fn a_rich_yard_fills_every_berth_it_has_in_one_decision() {
        let galaxy = Galaxy::generate(GalaxyConfig::new(2, 13)).unwrap();
        let mut sim = Simulation::with_baseline(galaxy, test_cfg(13));
        let home = sim.world.player_info.get(sim.player_entity[0]).unwrap().home;
        {
            let f = sim.world.factors.get_mut(home).unwrap();
            f.infra = infra_rung_price(1, &sim.config);
        }
        let berths = slips(sim.fabrication_rate(home), &sim.config);
        assert!(berths >= 2, "the anchor must give a rung-I yard more than one berth, got {berths}");

        // Enough minerals that affordability cannot be what stops it.
        {
            let bank = sim.world.stockpile.get_mut(home).unwrap();
            bank.cyan = 5_000.0;
            bank.magenta = 5_000.0;
            bank.yellow = 5_000.0;
        }
        sim.sys_build_decision(home);
        let filled = sim.world.berths.get(home).map(|b| b.len()).unwrap_or(0);
        assert_eq!(filled, berths, "a yard with {berths} berths and money committed {filled} builds");

        // And it stops there rather than looping past its capacity.
        let t0 = sim.clock;
        sim.sys_build_decision(home);
        assert_eq!(sim.world.berths.get(home).map(|b| b.len()).unwrap_or(0), berths, "a full yard must commit nothing");
        assert_eq!(sim.clock, t0, "the decision must not advance the clock");
    }

    /// **The decision cadence is the build cadence, not the economy's (R-O69).**
    ///
    /// A center that commits a build occupies its yard for the **order's own**
    /// `t_build` and decides again the moment it clears — not at the next
    /// `cycle_years` economy tick. That was the largest single throttle on the
    /// expansion loop: measured beforehand, the median funded build fired at
    /// 5.5x the price of what it bought.
    ///
    /// Since T-68 the occupancy is `t_lead + m / F_slip` rather than a flat
    /// constant, so this asserts the **relation** — the yard is held for
    /// exactly as long as the mass that was committed takes — which is what
    /// keeps it honest when the two constants are eventually ratified.
    ///
    /// This pins the structural property rather than a number, so it survives
    /// retuning either constant: **a busy yard is skipped by the economy tick,
    /// and a decision is pending for when it clears.**
    #[test]
    fn a_committed_build_occupies_the_yard_and_schedules_its_own_next_decision() {
        let galaxy = Galaxy::generate(GalaxyConfig::new(2, 11)).unwrap();
        let mut sim = Simulation::with_baseline(galaxy, test_cfg(11));
        let home = sim.world.player_info.get(sim.player_entity[0]).unwrap().home;

        // A homeworld with minerals to burn will commit something. The log is
        // on so the assertion can read *what it spent* rather than assume which
        // order a rich homeworld picks (`logging_does_not_affect_outcomes`).
        sim.set_log_filter(LogFilter::none().with(crate::log::LogCategory::Production));
        sim.world.stockpile.get_mut(home).unwrap().cyan = 500.0;
        assert_eq!(sim.world.berths.get(home).map(|b| b.len()).unwrap_or(0), 0, "yard starts free");

        sim.sys_build_decision(home);
        let Some(&done) = sim.world.berths.get(home).and_then(|b| b.first()) else {
            panic!("a center with 500 minerals and a live frontier must commit something");
        };
        // Whatever it chose, the yard is held for that order's build time — and
        // the mass is readable from the log rather than assumed, so this does
        // not quietly re-encode which order a rich homeworld picks.
        let committed = sim
            .log()
            .iter()
            .filter_map(|r| match r.event {
                LogEvent::BuildApplied { cost, .. } => Some(Price::new(cost)),
                _ => None,
            })
            .last()
            .expect("a committed build logs what it spent");
        assert!(
            (done - (sim.clock + sim.build_time(home, committed))).abs() < 1e-9,
            "the yard is held for t_build({committed}), got {done} at clock {}",
            sim.clock
        );
        assert!(done > sim.clock + sim.config.build_lead_years, "and never less than the lead time");

        // The economy tick must not decide over a busy yard — that would be the
        // cadence sneaking back in through the other door.
        let before = sim.world.stockpile.get(home).unwrap().basic_total();
        sim.sys_production_tick(home);
        let after = sim.world.stockpile.get(home).unwrap().basic_total();
        assert!(after >= before, "an occupied yard must not have spent again: {before} -> {after}");
        assert!(
            sim.world.berths.get(home).map(|b| !b.is_empty()).unwrap_or(false),
            "the economy tick must not clear the yard"
        );

        // And the decision that clears it is scheduled, not waited for.
        assert!(
            sim.queue.iter().any(|e| matches!(e.0.kind, EventKind::BuildDecision { center } if center == home)),
            "no BuildDecision pending for the busy center"
        );
    }

    #[test]
    fn population_growth_is_paid_for_out_of_biosphere() {
        // L6, amended: population is no longer an exception to mass
        // conservation. Every kiloton of people is a kiloton of biosphere that
        // stopped being biosphere, so **population mass + biomass** is
        // invariant across a growth step once regrowth is switched off.
        //
        // Note what is conserved and what is not: the invariant is over
        // *masses*, `KT(pop) + biomass`. `pop + biomass` — the old assertion —
        // is a Band added to a mass and was only ever "conserved" because the
        // engine drew a Band increment out of a kiloton stock.
        let galaxy = Galaxy::generate(GalaxyConfig::new(2, 5)).unwrap();
        let mut cfg = test_cfg(5);
        cfg.biosphere_regen_rate = 0.0; // isolate the exchange from the regrowth
        let mut sim = Simulation::with_baseline(galaxy, cfg);
        let home = sim.world.player_info.get(sim.player_entity[0]).unwrap().home;
        let bio_max = Band::new(4.0).in_kilotons();
        let pop0 = Kilotons::at_band(Band::new(1.0));
        let biomass = bio_max;
        sim.world
            .factors
            .insert(home, Factors::new(Band::new(4.0), biomass, bio_max, infra_rung_price(4, &sim.config)));
        *sim.world.population.get_mut(home).unwrap() = pop0;

        let before = pop0 + sim.world.factors.get(home).unwrap().biomass;
        sim.sys_production_tick(home);
        let f = sim.world.factors.get(home).unwrap();
        let pop = *sim.world.population.get(home).unwrap();

        assert!(pop > pop0, "population should have grown, got {pop:?}");
        assert!(f.biomass < biomass, "biosphere should have been drawn down, got {}", f.biomass);
        let after = pop + f.biomass;
        assert!((after.kilotons() - before.kilotons()).abs() < 1e-9, "mass not conserved: {before:?} -> {after:?}");
    }

    /// **T-67: razing infrastructure must not move `K`.** This is what the
    /// amendment bought, so it is asserted rather than assumed — and it is the
    /// precondition for every card that attacks industry, because while
    /// infrastructure sat inside `K` an industrial strike drove the population
    /// below its ceiling and T-64's logistic answered with a crash.
    #[test]
    fn razing_infrastructure_does_not_move_the_ceiling() {
        let bio_max = Band::new(3.0).in_kilotons();
        let developed = Factors::new(Band::new(4.0), bio_max, bio_max, Price::new(10.0));
        let mut razed = developed;
        razed.infra = Price::ZERO;

        assert_eq!(developed.k(), razed.k(), "K must not depend on infrastructure");
        assert_eq!(developed.k(), Band::new(3.0), "and it is min(hab, bio_max) — here the biosphere");

        // The ceiling still moves for the two factors that *are* a world's
        // capacity to hold people, which is where the crash is wanted.
        let mut cratered = developed;
        cratered.set_bio_max(Band::new(1.0).in_kilotons());
        assert_eq!(cratered.k(), Band::new(1.0), "a biosphere strike still lowers K");
        let mut poisoned = developed;
        poisoned.hab = Band::new(0.5);
        assert_eq!(poisoned.k(), Band::new(0.5), "and so does a habitability strike");
    }

    /// The unit fix, stated as behaviour: eating the biosphere must not lower
    /// the world's ceiling. Under `K = min(hab, bio, infra)` a drawn-down
    /// standing stock cut `K` directly — a mass compared against two levels —
    /// and the population it could hold fell with it.
    #[test]
    fn drawing_the_biosphere_down_does_not_lower_the_ceiling() {
        let bio_max = Band::new(4.0).in_kilotons();
        let full = Factors::new(Band::new(4.0), bio_max, bio_max, Price::new(10.0));
        let mut razed = full;
        razed.biomass = bio_max * 0.01; // ecology in ruins, ceiling untouched

        assert_eq!(full.k(), razed.k(), "K must not depend on the standing stock");
        assert_eq!(full.k(), Band::new(4.0));

        // The *ceiling* moves only when the pristine biosphere does — which is
        // what makes an ecological strike durable rather than momentary.
        // Through the setter, which is the only sanctioned write: `k_potential`
        // reads a cached Band reading of `bio_max` (R-O70), and a card that
        // craters an ecology by assigning the field directly would leave the
        // ceiling reading the old world.
        let mut cratered = full;
        cratered.set_bio_max(Band::new(1.5).in_kilotons());
        assert_eq!(cratered.k(), Band::new(1.5));
    }

    #[test]
    fn biosphere_regrows_toward_its_pristine_ceiling_but_never_past_it() {
        let galaxy = Galaxy::generate(GalaxyConfig::new(2, 6)).unwrap();
        let mut sim = Simulation::with_baseline(galaxy, test_cfg(6));
        let home = sim.world.player_info.get(sim.player_entity[0]).unwrap().home;
        // A cratered ecology: standing mass far below the pristine ceiling.
        //
        // Infrastructure is held at zero so `K == 0` and nothing grows. That
        // isolates the question the test is asking — *does a razed biosphere
        // come back* — from the one it is not: with a live `K` the population
        // simply outruns the regrowth and eats the recovery as it happens,
        // which is correct behaviour and a different test.
        let bio_max = Band::new(4.0).in_kilotons();
        let start = bio_max * 0.125;
        sim.world.factors.insert(home, Factors::new(Band::new(4.0), start, bio_max, Price::ZERO));
        *sim.world.population.get_mut(home).unwrap() = units::POPULATION_SEED_FLOOR;

        let mut last = start;
        for _ in 0..40 {
            sim.sys_production_tick(home);
            let bio = sim.world.factors.get(home).unwrap().biomass;
            assert!(bio.kilotons() <= bio_max.kilotons() + 1e-9, "biosphere exceeded its ceiling: {bio:?}");
            last = bio;
        }
        assert!(last > start, "a razed biosphere should recover over time, got {last:?}");
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
        sim.world
            .factors
            .insert(home, Factors::new(Band::new(4.0), Kilotons::ZERO, Band::new(4.0).in_kilotons(), Price::new(10.0)));
        *sim.world.population.get_mut(home).unwrap() = units::POPULATION_SEED_FLOOR;

        for _ in 0..20 {
            sim.sys_production_tick(home);
        }
        assert_eq!(
            sim.world.factors.get(home).unwrap().biomass,
            Kilotons::ZERO,
            "regen_bonus=0 must leave the biosphere dead"
        );
    }

    /// A population with no biomass to eat stalls; it does not go negative, and
    /// it does not conjure people. This is the constraint that used to be
    /// expressed as `K` collapsing.
    #[test]
    fn growth_stalls_when_the_biosphere_cannot_pay_for_it() {
        let galaxy = Galaxy::generate(GalaxyConfig::new(2, 7)).unwrap();
        let mut cfg = test_cfg(7);
        cfg.biosphere_regen_rate = 0.0;
        let mut sim = Simulation::with_baseline(galaxy, cfg);
        let home = sim.world.player_info.get(sim.player_entity[0]).unwrap().home;
        sim.world
            .factors
            .insert(home, Factors::new(Band::new(4.0), Kilotons::ZERO, Band::new(4.0).in_kilotons(), Price::new(10.0)));
        *sim.world.population.get_mut(home).unwrap() = Kilotons::at_band(Band::new(2.0));

        for _ in 0..10 {
            sim.sys_production_tick(home);
        }
        let pop = *sim.world.population.get(home).unwrap();
        let f = sim.world.factors.get(home).unwrap();
        assert!(pop.band().bands() <= 2.0 + 1e-9, "grew on an empty biosphere: {pop:?}");
        assert!(f.biomass >= Kilotons::ZERO, "biomass went negative: {:?}", f.biomass);
        // `K` is untouched by the starvation — the ceiling is a Band, the
        // shortfall is a rate.
        assert_eq!(f.k(), Band::new(4.0));
    }

    /// **T-56 stage 4: a colony seeded above its carrying capacity crashes
    /// *below* it, and that is why a bigger colony ship does not pay.**
    ///
    /// Stage 4b made a colony ship's seed the Band its hold masses, so a
    /// General hull founds at `Band II` where a Medium founds at `Band I`. On
    /// the four-seed bed that is **−5.9%** colony-years with the colony *count*
    /// identical on every seed, and the ablation that isolates it — a `Band II`
    /// seed at a Medium hull's price — is **−2.6%**. Seed depth is not merely
    /// worthless; it is harmful even when nearly free, so the General hull's
    /// price is not what killed it.
    ///
    /// The mechanism is here, and it is not the `clamp` that first looked
    /// guilty. `sys_production_tick` grows population by the **discrete**
    /// logistic `s + r·s·(1 − s/K)`, whose growth term goes strongly negative
    /// above `K` — at `r = 0.873` a population at `2K` does not settle back to
    /// `K`, it overshoots to `0.25K` in one step. The `clamp` bounds the top
    /// only.
    ///
    /// And the `K` that matters is **1.0, not the mature 1.43**: a founding
    /// colony gets `infra = Band I`, and `K = min(hab, bio_max, infra)`. So
    /// every Band of seed above the first is not just wasted, it is a
    /// population crash on the colony's first tick.
    ///
    /// **R-O75 (new, open):** whether the discrete logistic should be replaced
    /// by one that cannot overshoot downward (a saturating step, or the
    /// closed-form solution over the interval). It is a real modelling artifact
    /// — nothing in the design says an overfull world should lose three
    /// quarters of its people in fifty years — but it is on the hottest path in
    /// the engine and every ratified growth number was measured with it, so it
    /// is recorded rather than changed here.
    #[test]
    fn a_colony_seeded_above_its_capacity_crashes_below_it() {
        let galaxy = Galaxy::generate(GalaxyConfig::new(2, 7)).unwrap();
        let mut sim = Simulation::with_baseline(galaxy, test_cfg(7));
        let home = sim.world.player_info.get(sim.player_entity[0]).unwrap().home;

        // A world whose *habitability* is `Band I` — so `K` is `Band I`
        // however much biomass it carries. **Set through habitability, not
        // infrastructure** (T-67): infrastructure left `K`, so the engine can
        // no longer put a population above its ceiling by founding, and the
        // overshoot is reachable only the way the design intends — by an attack
        // on habitability or biosphere. That is the whole point of the
        // amendment, and pinning the crash on a hab-limited world is what keeps
        // this test measuring the logistic rather than the old coupling.
        //
        // Plenty of biomass, so nothing here is about the mass budget.
        let founding = |sim: &mut Simulation, seed: Kilotons| {
            sim.world.factors.insert(
                home,
                Factors::new(
                    Band::new(1.0),
                    Band::new(4.0).in_kilotons(),
                    Band::new(4.0).in_kilotons(),
                    Price::new(10.0),
                ),
            );
            *sim.world.population.get_mut(home).unwrap() = seed;
            sim.sys_production_tick(home);
            *sim.world.population.get(home).unwrap()
        };
        let from_band_i = founding(&mut sim, Kilotons::at_tier(BandTier::I));
        let from_band_ii = founding(&mut sim, Kilotons::at_tier(BandTier::II));

        // A `Band I` seed sits exactly at `K` and stays there.
        assert!(
            (from_band_i.band().bands() - 1.0).abs() < 1e-9,
            "a Band I seed should rest at K = Band I, got {from_band_i}"
        );
        // A `Band II` seed does not settle back to `K` — it overshoots below.
        assert!(
            from_band_ii < from_band_i,
            "a Band II seed must end up *worse* than a Band I one: {from_band_ii} vs {from_band_i}"
        );
        assert!(
            from_band_ii < Kilotons::at_band(Band::new(0.5)),
            "the overshoot is severe, not marginal — expected well under half a Band, got {from_band_ii}"
        );
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
    fn an_exhausted_mining_pair_goes_to_reserve_and_is_re_tasked_not_stranded() {
        // The strategic content of R-AC19: a rock runs dry after a mean of 808
        // years (`examples/mining_probe -- census`) and the pair built for it
        // used to stop working for the rest of the match. Reserve is the roles
        // §4.6 state for a standing mission that ended, and a later mining
        // order takes the hull back rather than buying another.
        let galaxy = Galaxy::generate(GalaxyConfig::new(2, 5)).unwrap();
        let mut cfg = test_cfg(5);
        cfg.recycle_mining_pairs = true;
        // **This test is about recycling, not about how many miners open an
        // outpost**, and a large crew would make the mineral assertion below a
        // test of *that* instead — the same way three hull-ladder tests silently
        // became tests of `limited_fleet_size` at stage 3b.
        //
        // Since T-87 there is no crew knob to pin: the crew is derived from the
        // founding centre's unmet demand, and a fresh homeworld with a full
        // starting bank has none, so it opens an outpost with one hull. The
        // assertion below states that rather than assuming it.
        let doctrine = Doctrine::default();
        let autopilots: Vec<Box<dyn Autopilot>> =
            (0..2).map(|_| Box::new(BaselineAutopilot::new(doctrine)) as Box<_>).collect();
        let mut sim = Simulation::new(galaxy, cfg, autopilots);

        let outpost = sim.planet_entity[11];
        let next_rock = sim.planet_entity[12];
        let center = sim.world.player_info.get(sim.player_entity[0]).unwrap().home;

        // A miner on station, exactly as `sys_mining_arrive` leaves one.
        let miner = sim.world.spawn();
        sim.world.owner.insert(miner, PlayerId(0));
        sim.world.role.insert(miner, Role::Miner);
        sim.world.hull_type.insert(miner, HullType::LimitedSystems);
        sim.world.voyage.insert(miner, Voyage { target: outpost, heading_bias: None, hops: 0 });
        sim.world.cargo.insert(miner, Minerals::default());
        sim.world.home_center.insert(miner, center);
        sim.sys_mining_arrive(miner);
        assert_eq!(sim.mine_crew.get(&(0, outpost.0)), Some(&vec![miner]), "the crew must be recorded on station");

        // Mine it out, then tick: the rock is done.
        *sim.world.density.get_mut(outpost).unwrap() = MineralField::default();
        sim.sys_mining_tick(outpost);

        assert_eq!(
            sim.world.role.get(miner),
            Some(&Role::Reserve),
            "an exhausted miner stands down, it does not scrap"
        );
        assert_eq!(sim.reserve_miners[0], vec![miner], "and it joins its owner's pool");
        assert!(!sim.mine_crew.contains_key(&(0, outpost.0)), "the dead rock has no crew");

        // Now a center orders a mining pair. The reserved hull is taken back at
        // no mineral cost; only the un-recycled half is paid for.
        let stock_before = Price::new(1000.0);
        {
            let st = sim.world.stockpile.get_mut(center).unwrap();
            let each = stock_before.kilotons() / 3.0;
            st.cyan = each;
            st.magenta = each;
            st.yellow = each;
        }
        let pid = *sim.world.planet_id.get(next_rock).unwrap();
        let view = sim.view_of(next_rock);
        let candidates = vec![Candidate {
            view,
            ranked: Ranked { id: pid, score: 9.0, class: PlanetClass::MiningOutpost },
            settlers_by_hull: [Kilotons::ZERO; 2],
            mining_crew: 1,
        }];
        let center_pos = *sim.world.position.get(center).unwrap();
        sim.apply_build_with(
            0,
            center,
            center_pos,
            BuildOrder::Hull { hull_type: HullType::LimitedSystems, class: Class::Meadow },
            &candidates,
        );

        assert_eq!(sim.world.role.get(miner), Some(&Role::Miner), "the reserved hull is back on a mining mission");
        assert!(sim.reserve_miners[0].is_empty(), "and out of the pool");
        assert_eq!(
            sim.world.voyage.get(miner).map(|v| v.target),
            Some(next_rock),
            "re-tasked to the new rock, flying from where the old one left it"
        );
        let spent = stock_before - sim.world.stockpile.get(center).unwrap().basic_total();
        let freighter_only = role_cost(Role::Freighter, &sim.config);
        assert!(
            (spent - freighter_only).abs() < Price::new(1e-9),
            "only the freighter is bought: spent {spent}, freighter costs {freighter_only}"
        );
    }

    #[test]
    fn the_freighter_calls_a_rock_dead_at_the_same_point_the_miner_does() {
        // The two used to disagree: `sys_mining_tick` stops when the *yield*
        // hits the floor (metallicity 0.042 at the shipped values) while the
        // freighter waited for metallicity < 0.01. In that band the mine was
        // dead and the hauler was not told, so it flew empty round trips for
        // the rest of the match — measured on seed 1, not one freighter of
        // 2,655 ever reached its stand-down branch.
        let galaxy = Galaxy::generate(GalaxyConfig::new(2, 5)).unwrap();
        let mut cfg = test_cfg(5);
        cfg.recycle_mining_pairs = true;
        let autopilots: Vec<Box<dyn Autopilot>> =
            (0..2).map(|_| Box::new(BaselineAutopilot::default()) as Box<_>).collect();
        let mut sim = Simulation::new(galaxy, cfg, autopilots);

        let outpost = sim.planet_entity[11];
        let center = sim.world.player_info.get(sim.player_entity[0]).unwrap().home;

        // Metallicity inside the dead band: above `density_floor`, but its
        // yield is at or below it, which is exactly when mining stops.
        let dead_band = cfg.density_floor / cfg.outpost_mining_fraction * 0.9;
        assert!(dead_band > cfg.density_floor, "the test needs a value the old predicate would have called alive");
        {
            // The field's colours are Bands since T-62, so a target *mass* is
            // set by reading it back onto the ladder rather than assigned.
            let each = Kilotons::new(dead_band / 3.0);
            let d = sim.world.density.get_mut(outpost).unwrap();
            for b in Basic::ALL {
                d.set(b, each);
            }
        }
        *sim.world.stockpile.get_mut(outpost).unwrap() = Minerals::default();

        let freighter = sim.world.spawn();
        sim.world.owner.insert(freighter, PlayerId(0));
        sim.world.role.insert(freighter, Role::Freighter);
        sim.world.hull_type.insert(freighter, HullType::MediumSystems);
        sim.world.cargo.insert(freighter, Minerals::default());
        sim.world.home_center.insert(freighter, center);
        sim.world.shuttle.insert(freighter, Shuttle { outpost, destination: center, outbound: true });
        let here = *sim.world.position.get(outpost).unwrap();
        sim.park(freighter, here);

        sim.sys_freighter_arrive(freighter);

        assert_eq!(
            sim.world.role.get(freighter),
            Some(&Role::Reserve),
            "an empty hauler at a rock that has stopped producing stands down"
        );
        assert_eq!(sim.reserve_freighters[0], vec![freighter], "and becomes available to re-task");
    }

    #[test]
    fn recycling_off_leaves_the_pair_where_the_rock_died() {
        // The flag's default has to be honest about what it changes: with it
        // off, exhaustion strands the hull exactly as before.
        let galaxy = Galaxy::generate(GalaxyConfig::new(2, 5)).unwrap();
        let mut cfg = test_cfg(5);
        cfg.recycle_mining_pairs = false;
        let autopilots: Vec<Box<dyn Autopilot>> =
            (0..2).map(|_| Box::new(BaselineAutopilot::default()) as Box<_>).collect();
        let mut sim = Simulation::new(galaxy, cfg, autopilots);

        let outpost = sim.planet_entity[11];
        let center = sim.world.player_info.get(sim.player_entity[0]).unwrap().home;
        let miner = sim.world.spawn();
        sim.world.owner.insert(miner, PlayerId(0));
        sim.world.role.insert(miner, Role::Miner);
        sim.world.hull_type.insert(miner, HullType::LimitedSystems);
        sim.world.voyage.insert(miner, Voyage { target: outpost, heading_bias: None, hops: 0 });
        sim.world.cargo.insert(miner, Minerals::default());
        sim.world.home_center.insert(miner, center);
        sim.sys_mining_arrive(miner);

        *sim.world.density.get_mut(outpost).unwrap() = MineralField::default();
        sim.sys_mining_tick(outpost);

        assert_eq!(sim.world.role.get(miner), Some(&Role::Miner), "still nominally mining a dead rock");
        assert!(sim.reserve_miners[0].is_empty(), "nothing is pooled when recycling is off");
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
        sim.world.factors.insert(
            colony,
            Factors::new(Band::new(3.0), Band::new(3.0).in_kilotons(), Band::new(3.0).in_kilotons(), Price::new(1.0)),
        );
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

    /// **T-81: a hauler goes where its cargo is what is missing.**
    ///
    /// Routing scored need on a *total* — how broke a centre was overall — and
    /// carried no colour term at all. T-73 made that binding: a works bill is
    /// payable in named colours, T-62 made the field log-normal per colour, and
    /// together they left banks holding one colour and traces of the others,
    /// with deepening down 94.5%.
    ///
    /// Two centres, equally broke, needing opposite colours. A Yellow-laden
    /// hauler must pick the Yellow-short one — and the same hauler carrying
    /// Magenta must pick the other. Asserting *both* directions is the point: a
    /// routing rule that always picked the same centre would pass a one-sided
    /// test whatever it was keying on.
    #[test]
    fn a_hauler_routes_to_the_colour_that_is_missing() {
        let galaxy = Galaxy::generate(GalaxyConfig::new(2, 3)).unwrap();
        let mut sim = Simulation::with_baseline(galaxy, SimConfig::new(3));
        let home = sim.world.player_info.get(sim.player_entity[0]).unwrap().home;
        let here = *sim.world.position.get(home).unwrap();

        // Two colonies, co-located with home so the λ discount cannot decide
        // this — the colour term has to.
        let (a, b) = (sim.planet_entity[15], sim.planet_entity[16]);
        for &e in &[a, b] {
            sim.world.owner.insert(e, PlayerId(0));
            sim.world.factors.insert(
                e,
                Factors::new(
                    Band::new(3.0),
                    Band::new(3.0).in_kilotons(),
                    Band::new(3.0).in_kilotons(),
                    infra_rung_price(1, &sim.config),
                ),
            );
            sim.world.position.insert(e, here);
        }
        // Home can pay for everything, so it is never the needy one.
        sim.world.stockpile.insert(home, Minerals { cyan: 1e6, magenta: 1e6, yellow: 1e6, ..Default::default() });

        let step = infra_step_price(infra_rung_price(1, &sim.config), &sim.config);
        let bill = works_bill(step, &cards::Works::default());
        // `a` has everything except Yellow; `b` has everything except Magenta.
        sim.world.stockpile.insert(
            a,
            Minerals { cyan: bill[0].kilotons(), magenta: bill[1].kilotons(), yellow: 0.0, ..Default::default() },
        );
        sim.world.stockpile.insert(
            b,
            Minerals { cyan: bill[0].kilotons(), magenta: 0.0, yellow: bill[2].kilotons(), ..Default::default() },
        );

        let yellow = Minerals { yellow: 10.0, ..Default::default() };
        let magenta = Minerals { magenta: 10.0, ..Default::default() };
        assert_eq!(
            sim.best_delivery_center(PlayerId(0), here, &yellow),
            Some(a),
            "a Yellow-laden hauler must go to the Yellow-short centre"
        );
        assert_eq!(
            sim.best_delivery_center(PlayerId(0), here, &magenta),
            Some(b),
            "and the same route with Magenta aboard must go the other way"
        );

        // **And it concentrates** (R-IND17), which is the property T-81's relief
        // term did not have. A third centre needing *everything* must score
        // strictly lower on the same Yellow cargo than one needing only Yellow,
        // or ore scatters by colour and a three-colour bill is never assembled
        // anywhere.
        let empty = sim.planet_entity[17];
        sim.world.owner.insert(empty, PlayerId(0));
        sim.world.factors.insert(
            empty,
            Factors::new(
                Band::new(3.0),
                Band::new(3.0).in_kilotons(),
                Band::new(3.0).in_kilotons(),
                infra_rung_price(1, &sim.config),
            ),
        );
        sim.world.position.insert(empty, here);
        sim.world.stockpile.insert(empty, Minerals::default());

        let near = sim.bill_completion(a, PlayerId(0), &yellow);
        let far = sim.bill_completion(empty, PlayerId(0), &yellow);
        assert!((near - 1.0).abs() < 1e-12, "a centre missing only Yellow is completed by Yellow: {near}");
        assert!(far > 0.0 && far < near, "and one missing everything scores strictly less: {far} vs {near}");
        assert_eq!(sim.bill_completion(a, PlayerId(0), &magenta), 0.0, "the wrong colour completes nothing");
        assert_eq!(
            sim.best_delivery_center(PlayerId(0), here, &yellow),
            Some(a),
            "so the hauler goes to the centre it can finish, not the emptiest"
        );
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
        sim.world.factors.insert(
            colony,
            Factors::new(Band::new(3.0), Band::new(3.0).in_kilotons(), Band::new(3.0).in_kilotons(), Price::new(1.0)),
        );
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
            for b in Basic::ALL {
                field.set(b, Band::new(5.0).in_kilotons());
            }
            sim.world.density.insert(outpost, field);
        }
        sim.world.stockpile.insert(outpost, Minerals::default());

        // Build a freighter "paired" with the homeworld (its home_center),
        // as apply_build would, but the homeworld is the *less* needy side.
        let from = *sim.world.position.get(home).unwrap();
        sim.spawn_freighter(0, home, from, outpost, 0.0);
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

    /// **R-O74: settlers are people who were somewhere else first, and the
    /// rest of the hold is minerals that were in someone's bank.**
    ///
    /// This is design law #11 reaching the one quantity that was exempt from
    /// it. `spawn_courier` used to write `pop_cargo` and debit nothing, so
    /// every coloniser launched created mass — and it was not a rounding
    /// error: the policy that shipped the biggest seed measured **+13.97%
    /// colony-years** on an identical colony count, which was a measurement of
    /// the violation rather than of the policy (`Hyades_industry.md` §1.6).
    ///
    /// Asserted as a **balance**, not as two magnitudes. The founding centre
    /// loses exactly what the ship carries, the new world gains exactly what
    /// the ship carried, and the ship lands empty — which is the only form of
    /// the claim that a later change cannot half-satisfy.
    #[test]
    fn settlers_are_drawn_from_a_real_population() {
        let mut sim = Simulation::with_baseline(Galaxy::generate(GalaxyConfig::new(3, 1)).unwrap(), test_cfg(1));
        let home = sim.world.player_info.get(sim.player_entity[0]).unwrap().home;
        // **A modest world and a big hull**, which is the case the mixed hold
        // exists for: the ceiling caps the settlers well below the hold, so the
        // rest of the volume goes as minerals. A General hull to a `Band IV`
        // world would carry people the whole way and leave no room, which is
        // correct and would test only half the rule.
        let target = sim.planet_entity[11];
        sim.world.factors.insert(
            target,
            Factors::new(Band::new(1.0), Band::new(4.0).in_kilotons(), Band::new(4.0).in_kilotons(), Price::ZERO),
        );
        sim.world.population.insert(home, Kilotons::at_tier(BandTier::III));
        {
            let s = sim.world.stockpile.get_mut(home).unwrap();
            s.cyan = 30.0;
            s.magenta = 30.0;
            s.yellow = 30.0;
        }

        let pop_before = *sim.world.population.get(home).unwrap();
        let bank_before = sim.world.stockpile.get(home).unwrap().basic_total();
        let there_before = *sim.world.population.get(target).unwrap();

        sim.spawn_courier(0, Role::Colonizer, HullType::GeneralSystems, home, target, 0.0);
        let ship = Entity(sim.world.entity_count() as u64 - 1);

        let settlers = *sim.world.pop_cargo.get(ship).unwrap();
        let endowment = sim.world.cargo.get(ship).unwrap().basic_total();
        assert!(settlers > Kilotons::ZERO, "a General hull should have crewed this");
        assert!(endowment > Price::ZERO, "and filled the rest of its hold out of the bank");

        // **The hold is one budget.** Settlers and minerals mass the same
        // (R-O32), so what left the centre is exactly what the hull can hold —
        // no more, and not two independent allowances.
        let hold = HullType::GeneralSystems.colony_seed_capacity(&sim.config);
        let carried = settlers + endowment.on_scale::<units::Mass>();
        assert!(carried <= hold + Kilotons::new(1e-9), "carried {carried} in a {hold} hold");

        // The debit, both halves.
        let pop_after = *sim.world.population.get(home).unwrap();
        let bank_after = sim.world.stockpile.get(home).unwrap().basic_total();
        assert!(
            ((pop_before - pop_after) - settlers).kilotons().abs() < 1e-9,
            "the centre lost {} people for a seed of {settlers}",
            pop_before - pop_after
        );
        assert!(
            ((bank_before - bank_after) - endowment).kilotons().abs() < 1e-9,
            "the centre lost {} minerals for an endowment of {endowment}",
            bank_before - bank_after
        );

        // And the credit, on arrival — the ship lands empty.
        sim.sys_colony_arrive(ship);
        let there_after = *sim.world.population.get(target).unwrap();
        assert!(
            ((there_after - there_before) - settlers).kilotons().abs() < 1e-9,
            "the world gained {} people from a seed of {settlers}",
            there_after - there_before
        );
        assert!(
            (sim.world.stockpile.get(target).unwrap().basic_total() - endowment).kilotons().abs() < 1e-9,
            "the new colony should start on the endowment it was sent with"
        );
        assert_eq!(*sim.world.pop_cargo.get(ship).unwrap(), Kilotons::ZERO);
        assert_eq!(sim.world.cargo.get(ship).unwrap().basic_total(), Price::ZERO);
    }

    /// **R-IND12: the seed is priced in time — what it saves the child against
    /// what it costs the origin to replace.**
    ///
    /// Not a fraction of anything. `sim::settler_target` carries the objective,
    /// the symbol table, the two rival formulations that were measured and
    /// rejected, and why the optimum is gridded rather than solved. This pins
    /// the properties a retuning must not break, because the two rejected
    /// formulations each failed exactly one of them:
    ///
    /// - **something always ships** — the rate formulation sent nothing from an
    ///   origin below `K_p/2` to a distant world, i.e. nothing in the early game;
    /// - **distance reduces it** — that is the whole content of the discount;
    /// - **a small origin is not stripped** — the fill-time formulation took 99%;
    /// - **a poorer destination takes less**, since the seed is capped by what
    ///   that world can hold.
    #[test]
    fn the_seed_is_priced_in_time_not_as_a_share() {
        let mut sim = Simulation::with_baseline(Galaxy::generate(GalaxyConfig::new(3, 1)).unwrap(), test_cfg(1));
        let home = sim.world.player_info.get(sim.player_entity[0]).unwrap().home;
        let target = sim.planet_entity[11];
        let cap = Band::new(3.0);
        let factors = |b: Band| Factors::new(b, b.in_kilotons(), b.in_kilotons(), Price::ZERO);
        sim.world.factors.insert(home, factors(cap));
        sim.world.factors.insert(target, factors(cap));
        let here = *sim.world.position.get(home).unwrap();
        sim.world.position.insert(target, here);
        let k = units::population_mass(cap);
        let hold = Kilotons::new(f64::MAX / 4.0); // ask the policy, not the hull

        // **Something always ships**, at every fill level — including the one
        // that broke the marginal-rate formulation, an origin well below its own
        // growth peak.
        for fill in [0.05, 0.2, 0.5, 0.9, 1.0] {
            sim.world.population.insert(home, k * fill);
            let got = sim.settler_target(home, target, hold);
            assert!(got > Kilotons::ZERO, "an origin at {fill} of its ceiling must still colonise, got {got}");
            assert!(got <= k * fill * 0.5 + Kilotons::new(1e-9), "and must not be stripped: {got} of {}", k * fill);
        }

        // **Distance reduces it.** Same origin, same destination, further away.
        sim.world.population.insert(home, k);
        let near = sim.settler_target(home, target, hold);
        sim.world.position.insert(target, Vec3::new(here.x + 3000.0, here.y, here.z));
        let far = sim.settler_target(home, target, hold);
        assert!(sim.travel_discount(home, target) < 0.05, "3,000 ly should be heavily discounted");
        assert!(far < near, "a distant world is worth less to seed: {far} vs {near}");

        // **A poorer destination takes less**, because the seed cannot exceed
        // what that world can hold.
        sim.world.position.insert(target, here);
        sim.world.factors.insert(target, factors(Band::new(1.0)));
        let poor = sim.settler_target(home, target, hold);
        assert!(poor <= units::population_mass(Band::new(1.0)), "capped by the destination: {poor}");
        assert!(poor < near, "and it is less than a rich destination takes: {poor} vs {near}");
    }

    /// **The hull is chosen against what will actually be loaded.**
    ///
    /// The coloniser hull decision reads `Candidate::settlers_by_hull`; the
    /// launch reads `colony_seed_for`. Those are two call sites of one rule, and
    /// this asserts they agree for every hull and every candidate — because the
    /// failure mode if they drift is silent and expensive: a General hull bought
    /// on settlers that never board.
    ///
    /// This repo has the scar. `mining_pair_cost` carries a comment explaining
    /// that it must match what `apply_build_with` will spend, and it must,
    /// because nothing checks it. This one is checked.
    #[test]
    fn the_hull_choice_sees_what_the_launch_will_load() {
        let mut sim = Simulation::with_baseline(Galaxy::generate(GalaxyConfig::new(3, 1)).unwrap(), test_cfg(1));
        let home = sim.world.player_info.get(sim.player_entity[0]).unwrap().home;
        let here = *sim.world.position.get(home).unwrap();
        let factors = |b: Band| Factors::new(b, b.in_kilotons(), b.in_kilotons(), Price::ZERO);
        let hulls = [HullType::MediumSystems, HullType::GeneralSystems];

        for kp in [1.0, 2.5, 4.0] {
            for kc in [0.5, 1.0, 2.5, 4.0] {
                for fill in [0.05, 0.5, 1.0] {
                    for dist in [0.0, 20.0, 400.0] {
                        let target = sim.planet_entity[11];
                        sim.world.factors.insert(home, factors(Band::new(kp)));
                        sim.world.factors.insert(target, factors(Band::new(kc)));
                        sim.world.position.insert(target, Vec3::new(here.x + dist, here.y, here.z));
                        sim.world.population.insert(home, units::population_mass(Band::new(kp)) * fill);
                        for (i, hull) in hulls.iter().enumerate() {
                            let hold = hull.colony_seed_capacity(&sim.config);
                            let advertised = sim.settler_target(home, target, hold);
                            let loaded = sim.colony_seed_for(*hull, home, target).unwrap_or(Kilotons::ZERO);
                            assert!(
                                (advertised - loaded).kilotons().abs() < 1e-12,
                                "hull {i} K_p={kp} K_c={kc} fill={fill} d={dist}: \
                                 the decision saw {advertised}, the launch loaded {loaded}"
                            );
                        }
                    }
                }
            }
        }
    }

    /// **A centre cannot crew a hull it has no people for.**    /// **A centre cannot crew a hull it has no people for.**    /// **A centre cannot crew a hull it has no people for.**
    ///
    /// Whatever the policy says is worth sending, the hold is still a ceiling and
    /// the origin is still a floor — so the same General hull delivers a full
    /// hold from a big centre and a fraction of one from a small centre. This is
    /// what makes the hull choice a real question again: before conservation a
    /// hold was a promise, and "settlers per mineral" could always be paid.
    #[test]
    fn a_hold_is_an_upper_bound_not_a_promise() {
        let mut sim = Simulation::with_baseline(Galaxy::generate(GalaxyConfig::new(3, 1)).unwrap(), test_cfg(1));
        let home = sim.world.player_info.get(sim.player_entity[0]).unwrap().home;
        let target = sim.planet_entity[11];
        sim.world.factors.insert(
            target,
            Factors::new(Band::new(4.0), Band::new(4.0).in_kilotons(), Band::new(4.0).in_kilotons(), Price::ZERO),
        );
        let hold = HullType::GeneralSystems.colony_seed_capacity(&sim.config);

        // Rich enough that the policy would happily send more than fits: the
        // hold binds, and the hull lands full.
        sim.world.factors.insert(
            home,
            Factors::new(Band::new(4.0), Band::new(4.0).in_kilotons(), Band::new(4.0).in_kilotons(), Price::ZERO),
        );
        sim.world.population.insert(home, units::population_mass(Band::new(4.0)));
        let full = sim.colony_seed_for(HullType::GeneralSystems, home, target).unwrap();
        assert!((full - hold).kilotons().abs() < 1e-9, "a rich centre fills the hold: {full} vs {hold}");

        // Poor: the origin binds, and the hull flies part-laden.
        sim.world.population.insert(home, hold * 2.0);
        let got = sim.colony_seed_for(HullType::GeneralSystems, home, target).unwrap();
        assert!(got < hold, "a small centre cannot fill a General hold: {got} vs {hold}");

        // **The floor is absolute.** A centre with nothing to spare ships
        // nobody — a world emptied of people has no logistic left to regrow on.
        sim.world.population.insert(home, units::POPULATION_SEED_FLOOR);
        assert_eq!(sim.colony_seed_for(HullType::GeneralSystems, home, target), Some(Kilotons::ZERO));
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
            sim.world.pop_cargo.insert(v, units::population_mass(sim.config.colony_seed_pop.band()));
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
        sim.world.pop_cargo.insert(v, Kilotons::at_band(Band::new(1.0)));
        sim.world.home_center.insert(v, home0);

        sim.sys_colony_arrive(v);
        let pop = *sim.world.population.get(target).unwrap();
        assert!((pop.band().bands() - 1.0).abs() < 1e-9, "colony should be seeded with the carried 1.0 pop, got {pop}");
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
        sim.world.pop_cargo.insert(v, Kilotons::at_band(Band::new(1.0)));
        sim.world.home_center.insert(v, home0);

        sim.sys_colony_arrive(v);
        let stock = sim.world.stockpile.get(target).unwrap().basic_total();
        assert!(stock.abs() < Price::new(1e-9), "colony should start with zero minerals, got {stock}");
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
