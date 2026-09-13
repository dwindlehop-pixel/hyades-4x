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
use crate::units::{Band, BandTier, Kilotons, Price};

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
    /// Built infrastructure, read back onto the **Cost** ladder as a rung.
    pub infrastructure: Band,
    /// **The same infrastructure as the mass it actually is** — the minerals
    /// standing in it, in kilotons (T-70, `Hyades_industry.md` §1.3).
    ///
    /// Carried alongside the Band for the same reason `biomass` is carried
    /// alongside `biosphere`: a Band is a *reading* and the stock is the thing
    /// (`CLAUDE.md` §4). It is a `Price` because the infrastructure ladder is
    /// the mineral ladder (R-O80), so this is kilotons on the Cost scale — the
    /// two ladders are `^1.5` apart and reading it on the wrong one would move
    /// every threshold at once.
    ///
    /// This is the stock **Growth's work-years objective integrates**
    /// (`Hyades_trees_and_card_value.md` §2.3.3), and it is why that objective
    /// is measurable now rather than waiting on works.
    pub works: Price,
    /// Liebig carrying capacity `K = min(hab, bio_max, infra)`, over Bands.
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
#[derive(Clone, Copy, Debug)]
pub struct VehicleSnapshot {
    pub owner: u32,
    pub kind: VehicleKind,
    pub position: Vec3,
    /// Minerals carried (mining/freighter cargo).
    pub cargo: Minerals,
    /// **Dry mass of the hull, which since R-O57 *is* its mineral cost**
    /// (design law #11). Exposed because Production's objective is
    /// fleet-years **in mass** — `Hyades_trees_and_card_value.md` §2.3.4 is
    /// explicit that counting hulls rewards fragmentation and would put that
    /// tree in direct contradiction with design law #3. A consumer summing
    /// `vehicles.len()` is measuring the wrong thing and nothing in the old
    /// snapshot could tell it so.
    pub dry_mass: Kilotons,
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
