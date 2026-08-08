//! The Ship Testing Arena's Monte Carlo harness — confirmed this
//! conversation: *"we will use Monte Carlo simulation to pit different
//! fleets against each other."* Resolves `Hyades_mineral_cost_curve.md`
//! §6's own worked scenario (§6.3): one General-class hull against N
//! equal-tree Limited-class hulls, sweeping starting distance and N.
//!
//! **What this measures, honestly stated:** combat resolution (weapons,
//! armor, the wreck roll) isn't implemented yet — building it is blocked
//! on `Hyades_loadout.md`'s R-L0/R-L1/R-L2, exactly as
//! `Hyades_mineral_cost_curve.md` §6.2 already flags. So "pitting fleets
//! against each other" here measures **kinematic outcomes only**: does
//! the chasing side ever close to within a placeholder "engagement range"
//! of a fleeing target within a bounded time horizon, and how long does
//! it take. This is the movement layer the eventual combat system sits on
//! top of, not a combat result.
//!
//! Run with: `cargo run --release --example combat_arena`

use hyades_engine::arena::{spawn_fleet, ArenaShip, FleetTrajectory, InterceptCriterion};
use hyades_engine::prelude::*;
use hyades_engine::rng::Rng;

/// Placeholder "weapon range" — closing to within this counts as
/// "engaged." R-ARENA5 (new, open): a real number once WPN slot stats
/// (Hyades_loadout.md §3.2/R-L0) exist; this is just small relative to
/// the swept starting distances.
const ENGAGEMENT_RANGE: f64 = 0.0002; // ly

/// Simulated ship state during stepping: explicit (position, velocity),
/// re-derived each trial from the analytic `ArenaShip` at t=0, then
/// integrated forward by simple Euler steps under whatever acceleration
/// its behavior (chase/flee) picks each tick. `Hyades_simulation_model.md`
/// §1a's relativistic requirement is honored where it matters (the engine's
/// existing interstellar-cruise math); this local, short-range, sub-c
/// tactical stepping uses flat kinematics — see `arena.rs`'s own
/// `solve_intercept` doc comment for why that's a stated, not silent,
/// simplification.
#[derive(Clone, Copy)]
struct ShipState {
    pos: Vec3,
    vel: Vec3,
    max_accel: f64,
}

fn initial_state(ship: &ArenaShip, fleets: &[FleetTrajectory], cfg: &SimConfig) -> ShipState {
    ShipState { pos: ship.position_at(fleets, 0.0), vel: ship.velocity_at(fleets, 0.0), max_accel: ship.max_accel(cfg) }
}

/// One trial: Fleet A (1 General hull, the "consolidated" side) chases
/// Fleet B (`n` Limited hulls, the "fragmented" side); Fleet B ships each
/// flee the nearest chaser. Returns (engaged?, time_to_first_engagement).
fn run_trial(seed: u64, start_distance: f64, closing_speed: f64, n: usize, horizon: f64, dt: f64) -> (bool, f64) {
    let cfg = SimConfig::new(seed);
    let mut rng = Rng::new(seed);

    let fleet_a = FleetTrajectory { origin: Vec3::ZERO, velocity: Vec3::new(closing_speed, 0.0, 0.0) };
    let fleet_b = FleetTrajectory { origin: Vec3::new(start_distance, 0.0, 0.0), velocity: Vec3::ZERO };
    let fleets = [fleet_a, fleet_b];

    // Small station-keeping jitter relative to the starting distance —
    // confirmed this conversation: ships aren't exactly at their fleet's
    // reference point, so a chaser can't trivially aim at a fixed point.
    let jitter = (start_distance * 0.01, start_distance * 0.02);
    let period = (0.05, 0.2);

    let chasers = spawn_fleet(&mut rng, 0, 1, Role::Freighter, HullType::GeneralSystems, jitter, period);
    let fleeing = spawn_fleet(&mut rng, 1, n, Role::Freighter, HullType::LimitedSystems, jitter, period);

    let mut chaser_states: Vec<ShipState> = chasers.iter().map(|s| initial_state(s, &fleets, &cfg)).collect();
    let mut target_states: Vec<ShipState> = fleeing.iter().map(|s| initial_state(s, &fleets, &cfg)).collect();

    let mut t = 0.0;
    while t < horizon {
        // Each chaser: intercept the nearest still-distant target.
        let mut chaser_accel = vec![Vec3::ZERO; chaser_states.len()];
        for (ci, chaser) in chaser_states.iter().enumerate() {
            if let Some((ti, _)) = target_states
                .iter()
                .enumerate()
                .map(|(ti, tgt)| (ti, tgt.pos.distance(chaser.pos)))
                .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            {
                let rel_pos = target_states[ti].pos.sub(chaser.pos);
                let rel_vel = target_states[ti].vel.sub(chaser.vel);
                if let Some(sol) = hyades_engine::arena::solve_intercept(
                    rel_pos,
                    rel_vel,
                    chaser.max_accel,
                    InterceptCriterion::PositionWithin(ENGAGEMENT_RANGE),
                ) {
                    chaser_accel[ci] = sol.direction.scale(chaser.max_accel);
                }
            }
        }
        // Each target: flee directly away from the nearest chaser.
        let mut target_accel = vec![Vec3::ZERO; target_states.len()];
        for (ti, tgt) in target_states.iter().enumerate() {
            if let Some((_, nearest)) = chaser_states
                .iter()
                .enumerate()
                .min_by(|a, b| a.1.pos.distance(tgt.pos).partial_cmp(&b.1.pos.distance(tgt.pos)).unwrap())
            {
                let away = tgt.pos.sub(nearest.pos).normalized();
                target_accel[ti] = away.scale(tgt.max_accel);
            }
        }

        for (s, a) in chaser_states.iter_mut().zip(chaser_accel.iter()) {
            s.vel = s.vel.add(a.scale(dt));
            s.pos = s.pos.add(s.vel.scale(dt));
        }
        for (s, a) in target_states.iter_mut().zip(target_accel.iter()) {
            s.vel = s.vel.add(a.scale(dt));
            s.pos = s.pos.add(s.vel.scale(dt));
        }

        for chaser in &chaser_states {
            for tgt in &target_states {
                if chaser.pos.distance(tgt.pos) <= ENGAGEMENT_RANGE {
                    return (true, t);
                }
            }
        }
        t += dt;
    }
    (false, horizon)
}

fn main() {
    let cfg = SimConfig::new(0);
    println!("Ship Testing Arena — kinematic MC harness");
    println!(
        "Fleet A: 1 General Systems hull (cost {:.3}) vs Fleet B: N Limited Systems hulls (cost {:.3} each)\n",
        1.0_f64, /* General is the cost reference; general_fleet_size==1 */
        1.0 / cfg.limited_fleet_size
    );
    let equal_cost_n = cfg.limited_fleet_size /* / general_fleet_size(=1) */;
    println!("Equal-cost N (1 General's mineral cost ~= N Limiteds'): {equal_cost_n:.2}\n");

    const SEEDS: &[u64] = &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let horizon = 3.0; // years
    let dt = 0.005; // years

    println!("{:>12} {:>4} {:>14} {:>16}", "distance(ly)", "N", "engaged_frac", "mean_time(yr)");
    for &distance in &[0.001, 0.005, 0.02] {
        for &n in &[1usize, 2, 4, 8, 16] {
            let mut engaged = 0u32;
            let mut time_sum = 0.0;
            for &seed in SEEDS {
                let (hit, t) = run_trial(seed, distance, 0.05, n, horizon, dt);
                if hit {
                    engaged += 1;
                    time_sum += t;
                }
            }
            let frac = engaged as f64 / SEEDS.len() as f64;
            let mean_t = if engaged > 0 { time_sum / engaged as f64 } else { f64::NAN };
            println!("{distance:>12.4} {n:>4} {:>13.1}% {mean_t:>16.3}", frac * 100.0);
        }
    }
}
