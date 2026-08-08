//! The Ship Testing Arena — the ONE piece that stays separate from the engine's
//! combat model (`combat.rs`). Its only job is to **spawn ships outside the
//! constraints of Hyades production** — no planets, no mining, no colonization,
//! no mineral budget — place them as two fleets, and hand them to the engine's
//! [`crate::combat::resolve_engagement`]. All actual fighting lives in `combat`;
//! nothing here resolves damage. This is the sandbox that sets ship *design*
//! (per-class volume/`r_eq`, slot layout) empirically — `Hyades_mineral_cost_curve.md`
//! §6, resolving R-MC13 — by pitting fabricated fleets against each other.
//!
//! Back-compat: the combat primitives this scenario layer surfaces are
//! re-exported below, so existing `arena::{solve_intercept, Missile, …}` paths
//! (tests, example drivers) keep resolving even though the definitions now live
//! in `combat`.

use crate::rng::Rng;
use crate::sim::{hull_thrust_multiplier_range, role_hull_type, HullType, Role, SimConfig};

pub use crate::combat::{
    laser_hit_check, laser_hit_check_missile, resolve_engagement, solve_intercept, BspTree, CombatConfig, Combatant,
    Combatant as ArenaShip, EngagementOutcome, FleetTrajectory, InterceptCriterion, InterceptSolution, Missile,
    StationKeeping, Winner,
};
use crate::math::Vec3;

/// Spawn one side's fleet: a trajectory plus `n` ships of the given
/// role/hull, station-keeping drawn from `rng` (fork the caller's own RNG
/// per ship for determinism, same discipline as the rest of the engine).
pub fn spawn_fleet(
    rng: &mut Rng,
    fleet_index: usize,
    n: usize,
    role: Role,
    hull: HullType,
    radius_range: (f64, f64),
    period_years_range: (f64, f64),
) -> Vec<Combatant> {
    (0..n)
        .map(|i| {
            let mut ship_rng = rng.fork(i as u64);
            let (lo, hi) = hull_thrust_multiplier_range(hull);
            let thrust_factor = if hull == HullType::RapidOffensive { hi } else { ship_rng.range(lo, hi) };
            Combatant {
                role,
                hull,
                thrust_factor,
                fleet: fleet_index,
                station: StationKeeping::draw(&mut ship_rng, radius_range, period_years_range),
                maneuver_velocity: Vec3::ZERO,
                maneuver_start: 0.0,
                maneuver_origin_offset: Vec3::ZERO,
            }
        })
        .collect()
}

/// Convenience: the hull a fresh combatant of `role` would fly, matching
/// `sim::role_hull_type`'s existing default lookup (Scout→LCV,
/// Colonizer/Freighter→MSV, Miner→LSV) — the arena mostly cares about
/// Offensive/Contact/Systems roles instead, so callers typically pick
/// `HullType` explicitly, but this is here for parity with the rest of
/// the engine's spawn conventions.
pub fn default_hull_for(role: Role) -> HullType {
    role_hull_type(role)
}

// --- Scenarios (the "spawn outside production, then fight" harness) ---------

/// Station-keeping ranges the ROU laser-vs-missile sweep was tuned at — kept
/// here (not in the example) so the tuned scenario setup lives in one place.
pub const ROU_STATION_RADIUS: (f64, f64) = (0.00005, 0.0002);
pub const ROU_STATION_PERIOD: (f64, f64) = (0.02, 0.08); // years

/// The canonical ROU laser-vs-missile trial: both sides fly `RapidOffensive`
/// (η=0.73 Gangster anchor), the laser fleet at rest and the missile fleet
/// closing/receding at `relative_velocity`, separated by `distance` ly. Spawns
/// both fleets outside production and resolves via the engine. This is the exact
/// setup the old `laser_vs_missile` example ran inline — now a one-call scenario.
#[allow(clippy::too_many_arguments)]
pub fn laser_vs_missile_trial(
    seed: u64,
    distance: f64,
    relative_velocity: f64,
    n_lasers: usize,
    n_missiles: usize,
    horizon: f64,
    dt: f64,
    volley_period: f64,
    combat_cfg: &CombatConfig,
) -> EngagementOutcome {
    let sim_cfg = SimConfig::new(seed);
    let mut rng = Rng::new(seed);
    let hull = HullType::RapidOffensive;

    let fleet_laser = FleetTrajectory { origin: Vec3::ZERO, velocity: Vec3::ZERO };
    let fleet_missile =
        FleetTrajectory { origin: Vec3::new(distance, 0.0, 0.0), velocity: Vec3::new(-relative_velocity, 0.0, 0.0) };
    let fleets = [fleet_laser, fleet_missile];

    let laser_ships = spawn_fleet(&mut rng, 0, n_lasers, Role::Freighter, hull, ROU_STATION_RADIUS, ROU_STATION_PERIOD);
    let missile_ships =
        spawn_fleet(&mut rng, 1, n_missiles, Role::Freighter, hull, ROU_STATION_RADIUS, ROU_STATION_PERIOD);

    resolve_engagement(
        &sim_cfg,
        combat_cfg,
        &mut rng,
        &fleets,
        &laser_ships,
        &missile_ships,
        horizon,
        dt,
        volley_period,
    )
}

#[cfg(test)]
#[path = "arena_tests.rs"]
mod tests;
