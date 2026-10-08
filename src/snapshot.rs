//! Read-only snapshot types — the **only** surface the presentation / command
//! layer is meant to touch.
//!
//! The decoupling contract (see crate docs) says the renderer and the hex
//! command view must never reach into engine internals. They consume a
//! [`Snapshot`] instead: plain owned data, no behavior, no back-references. The
//! command layer is where the continuous planet points get *bound* into the flat
//! hex tiling (`Hyades_galaxy_and_autopilot.md` §1) — that binding is not the
//! engine's job and is deliberately absent here.
//!
//! Minerals do **not** live on players: a planet carries a *stockpile* of mined
//! minerals (and an in-ground *density*), and a ship carries *cargo*. The
//! snapshot reflects that. Every spatial entity (planet and ship) reports an
//! exact `(x, y, z)` for the snapshot's instant.

use crate::galaxy::PlanetId;
use crate::math::Vec3;
use crate::resources::{MineralField, Minerals};
use crate::units::{Band, BandTier, Kilotons, Price, Volume};

/// Which civilian role a ship is fulfilling (read-only mirror of the engine's
/// hull enum, kept here so presentation never depends on `sim`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VehicleKind {
    Scout,
    Colonizer,
    Miner,
    Freighter,
    /// **Holding an unclaimed world against someone else's colonizer**
    /// (`Hyades_warfare_tree.md` §8, T-112) — a standing mission, so it never
    /// auto-scraps (roles §4.6).
    Picket,
    /// **Guarding an owned center with missiles** (T-139) — a standing
    /// mission.
    Sentry,
    Reserve,
    Scrapped,
}

/// One planet's externally-visible state.
#[derive(Clone, Copy, Debug)]
pub struct PlanetSnapshot {
    pub id: PlanetId,
    pub position: Vec3,
    pub habitability: Band,
    /// **Standing** biosphere, read back onto the Band ladder — what is
    /// growing there now, which falls as population is made out of it.
    pub biosphere: Band,
    /// **Pristine** biosphere ceiling, on the Band ladder. This is the term
    /// that enters `k`; [`Self::biosphere`] is the stock, not the ceiling.
    pub bio_max: Band,
    /// The standing biosphere as the mass it actually is. The same quantity as
    /// [`Self::biosphere`], in the unit conservation is stated in.
    pub biomass: Kilotons,
    /// **The pristine ceiling as the mass it actually is** — the same quantity
    /// as [`Self::bio_max`]. Carried because a Band is an approximate reading
    /// (T-129, within 3e-7 Band), so "the stock never exceeds its ceiling" is
    /// a comparison of two masses, not of a mass and a reconstruction.
    pub bio_max_mass: Kilotons,
    /// Built infrastructure, read back onto the **Cost** ladder as a whole Band.
    pub infrastructure: Band,
    /// **The same infrastructure as the mass it actually is** — the minerals
    /// standing in it, in kilotons (T-70, `Hyades_industry.md` §1.3).
    ///
    /// Carried alongside the Band for the same reason `biomass` is carried
    /// alongside `biosphere`: a Band is a *reading* and the stock is the thing
    /// (`AGENTS.md` §4). It is a `Price` because the infrastructure ladder is
    /// the mineral ladder (R-O80), so this is kilotons on the Cost scale — the
    /// two ladders are `^1.5` apart and reading it on the wrong one would move
    /// every threshold at once.
    ///
    /// This is the stock **Growth's work-years objective integrates**
    /// (`Hyades_trees_and_card_value.md` §2.3.3), and it is why that objective
    /// is measurable now rather than waiting on works.
    pub works: Price,
    /// Liebig carrying capacity `K = min(hab, bio_max)`, over Bands.
    /// **Infrastructure is not a term** — it left the minimum at
    /// `Hyades_industry.md` §1.1, because razing industry must not move
    /// population.
    pub k: Band,
    pub population: Kilotons,
    pub pop_level: BandTier,
    /// In-ground mineral density (depletes as it is mined).
    pub density: MineralField,
    /// Mined minerals on hand at this planet (spent on builds).
    pub stockpile: Minerals,
    /// `Some(player_index)` if owned.
    pub owner: Option<u32>,
    pub is_homeworld: bool,
}

/// One ship's externally-visible state, including its exact position now.
///
/// Every hull the engine holds is here, a wreck included: the presentation's
/// tactical mode draws every entity at its position (`Hyades_interface.md`
/// §6), and a wreck still coasts through the theater.
#[derive(Clone, Copy, Debug)]
pub struct VehicleSnapshot {
    /// **The hull's entity id**, stable for its whole life and never reused —
    /// what a replay matches one frame's hull to the next's by.
    pub id: u64,
    pub owner: u32,
    pub kind: VehicleKind,
    pub position: Vec3,
    /// Minerals carried (mining/freighter cargo).
    pub cargo: Minerals,
    /// **Dry mass of the hull, which since R-O57 *is* its mineral cost plus its
    /// mounted drive** (design law #11, T-96). This is what the hull *masses*
    /// and what it *cost*; it is **not** Production's objective — see
    /// [`Self::volume`].
    pub dry_mass: Kilotons,
    /// **Enclosed volume `r³`, and the stock Production's objective integrates**
    /// (`Hyades_trees_and_card_value.md` §2.3.4, R-PROD5).
    ///
    /// Counting *hulls* rewards fragmentation outright. Counting *mass* is
    /// subtler and was wrong for longer: since dry mass is mineral cost, mass
    /// -years is "minerals committed to hulls, integrated", which scores one
    /// General hull and the ten Mediums its minerals would buy **exactly
    /// equally** — so it is silent on design law #3, a law about that ratio.
    /// Volume is the law's own value basis, and on the shipped ladder an
    /// equal-cost General fleet encloses 2.47x a Medium one.
    ///
    /// Both fields ship because they answer different questions: `dry_mass` is
    /// conservation and acceleration, `volume` is capability. A consumer summing
    /// `vehicles.len()` is measuring the wrong thing, and nothing in the
    /// snapshot could tell it so until these existed.
    pub volume: Volume,
    /// `true` while in flight; `false` when on station / idle.
    pub in_flight: bool,
    /// **The hull type**, by the docs' abbreviation (`Hyades_vehicle_roles.md`
    /// §3): `LSV`, `MSV`, `GSV`, `LCV`, `LCU`, `GCV`, `GCU`, `LOU`, `ROU`, `GOU`.
    pub hull: &'static str,
    /// **The Design class it was built to** (R-O42b): `Meadow`, `Spur`, `Tor`,
    /// `Cairn`, `Delta`, `Range`, `Scarp`, `Ford`, `Strait`, `Butte`, `Mesa`,
    /// or `Unnamed`.
    pub design: &'static str,
    /// Beam mounts its Design carries (T-125). Zero is unarmed.
    pub beams: u32,
    /// Missile tubes its Design carries (T-139).
    pub tubes: u32,
    /// **Damage absorbed, as a share of the hull's structure** (T-133): `0` is
    /// undamaged; a hull is wrecked at a point drawn past its structure, so a
    /// standing hull can read above `1`. A wreck reads exactly `1`.
    pub damage: f64,
    /// **Wrecked, and coasting on the course it had** (T-133).
    pub wrecked: bool,
    /// Coordinate velocity now, ly/yr (`c = 1`).
    pub velocity: Vec3,
    /// **Proper acceleration the drive is flying now**, ly/yr²; `0` at rest
    /// and for a wreck. The observable of design law #10.
    pub accel: f64,
    /// **The drive's sense**: `1` gaining speed, `-1` shedding it, `0` at rest.
    pub burn: i8,
    /// The world its voyage is bound for, when it has one.
    pub destination: Option<PlanetId>,
    /// People aboard, kt (a colonizer's settlers).
    pub settlers: Kilotons,
}

/// One empire's aggregate state. (No `minerals` field — empires do not hold
/// minerals; planets and ships do.)
#[derive(Clone, Copy, Debug, Default)]
pub struct PlayerSnapshot {
    pub planets_owned: u32,
    pub mining_outposts: u32,
    pub planets_scanned: u32,
    pub ships: u32,
    /// The empire's people, **as a mass**. Population is a Band level and
    /// Bands are magnitude tiers, so adding them across planets sums
    /// logarithms and means nothing; adding the masses means what it says.
    pub total_population: Kilotons,
    /// Convenience roll-up: total minerals stockpiled across this empire's
    /// planets (the empire does not hold these centrally; this is a sum).
    pub stockpile_total: f64,
}

/// A full read-only picture of the simulation at one instant. Every entity's
/// `(x, y, z)` for `time_years` is recoverable from `planets` + `vehicles`.
#[derive(Clone, Debug)]
pub struct Snapshot {
    pub time_years: f64,
    pub players: Vec<PlayerSnapshot>,
    pub planets: Vec<PlanetSnapshot>,
    pub vehicles: Vec<VehicleSnapshot>,
}
