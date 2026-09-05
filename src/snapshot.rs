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
use crate::units::{Band, Kilotons};

/// Which civilian role a ship is fulfilling (read-only mirror of the engine's
/// hull enum, kept here so presentation never depends on `sim`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VehicleKind {
    Scout,
    Colonizer,
    Miner,
    Freighter,
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
    pub infrastructure: Band,
    /// Liebig carrying capacity `K = min(hab, bio_max, infra)`, over Bands.
    pub k: Band,
    pub population: Band,
    pub pop_level: u8,
    /// In-ground mineral density (depletes as it is mined).
    pub density: MineralField,
    /// Mined minerals on hand at this planet (spent on builds).
    pub stockpile: Minerals,
    /// `Some(player_index)` if owned.
    pub owner: Option<u32>,
    pub is_homeworld: bool,
}

/// One ship's externally-visible state, including its exact position now.
#[derive(Clone, Copy, Debug)]
pub struct VehicleSnapshot {
    pub owner: u32,
    pub kind: VehicleKind,
    pub position: Vec3,
    /// Minerals carried (mining/freighter cargo).
    pub cargo: Minerals,
    /// `true` while in flight; `false` when on station / idle.
    pub in_flight: bool,
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
    pub stockpiled_total: f64,
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
