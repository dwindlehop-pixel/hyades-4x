//! **Colony-years: the guard for performance work on the decision path.**
//!
//! Colony *count* at the horizon is the objective, but it is a weak invariant
//! for an optimization: a change that founds the same worlds a century later
//! scores identically. `∫ colonies dt` does not — it is the area under the
//! ramp, so it falls if anything slows down, and it is exactly what an
//! `O(galaxy)` fix must not touch.
//!
//! Computed as `Σ over foundings of (horizon − t_found)`, which is that
//! integral exactly while colonies are never lost — nothing in the shipped
//! engine takes one back.
//!
//! Run: `cargo run --release --example colony_years`
use hyades_engine::log::{LogCategory, LogEvent, LogFilter};
use hyades_engine::prelude::*;
use std::io::Write;

const SEEDS: &[u64] = &[1, 7];
const PLAYERS: usize = 3;

fn main() {
    for &seed in SEEDS {
        // Pass 1: clean, for throughput. The log allocates per event and would
        // otherwise be measured as engine cost.
        let t0 = std::time::Instant::now();
        let mut clean = build(seed, false);
        clean.run();
        let secs = t0.elapsed().as_secs_f64();
        let colonies = clean.snapshot().planets.iter().filter(|p| p.owner.is_some()).count();

        // Pass 2: logged, for the invariant.
        let mut logged = build(seed, true);
        logged.run();
        let horizon = logged.clock();
        let mut colony_years = 0.0_f64;
        let mut founded = 0usize;
        let mut first = f64::INFINITY;
        for r in logged.log().iter() {
            if matches!(r.event, LogEvent::ColonyFounded { .. }) {
                colony_years += horizon - r.time;
                founded += 1;
                first = first.min(r.time);
            }
        }
        println!(
            "seed {seed:>6}: colonies={colonies:>6} founded={founded:>6} colony_years={colony_years:>12.1} \
             first={first:>7.1} {secs:>5.1}s {:>4.0} yr/s",
            horizon / secs
        );
        std::io::stdout().flush().unwrap();
    }
}

fn build(seed: u64, log: bool) -> Simulation {
    let galaxy = Galaxy::generate(GalaxyConfig::new(PLAYERS, seed)).unwrap();
    let autopilots: Vec<Box<dyn Autopilot>> =
        (0..PLAYERS).map(|_| Box::new(BaselineAutopilot::new(Doctrine::default())) as Box<_>).collect();
    let mut sim = Simulation::new(galaxy, SimConfig::new(seed), autopilots);
    if log {
        sim.set_log_filter(LogFilter::none().with(LogCategory::Vehicles));
    }
    sim
}
