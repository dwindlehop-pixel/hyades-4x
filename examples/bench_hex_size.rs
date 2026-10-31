//! Throughput against **entity count** (T-02): `bench_hex_size [seats] [horizon] [segment] [seed] [hex_side_ly]`.
//!
//! The file keeps the name `galaxy.rs`, `tests/smoke.rs` and
//! `Hyades_matching.md` cite, but the sweep is no longer over hex size. Since
//! the snowball is the shipped behavior (design law #14), the first-order cost
//! of a run is how many entities it carries, and that moves through a run by
//! orders of magnitude while the hex stays fixed. So one run is cut into
//! segments of `segment` simulated years, and each segment prints:
//!
//! - the vehicle count at its end (`Snapshot::vehicles`),
//! - its own `yr/s` and `ns/event` (AGENTS.md §2: report the per-event cost
//!   beside the rate, so "more work" and "dearer work" can be told apart),
//! - the margin against T-24's floor of 2.5 simulated years per real second.
//!
//! Defaults: seats `3,12`, horizon 800 yr, segment 100 yr, seed 1, the shipped
//! hex side. `seats` is a comma list of fair counts (`Galaxy::FAIR_COUNTS`).
//! `hex_side_ly`, when given, overrides `GalaxyConfig::hex_side_ly`, which is
//! the old sweep's axis kept as an option.
//!
//! Each row is flushed as it is printed, so a run that is cut short still
//! yields every segment that finished (AGENTS.md §2, "always run sweeps
//! unbuffered").
use hyades_engine::prelude::*;
use std::io::Write;
use std::time::Instant;

/// T-24's throughput floor, in simulated years per real second.
const FLOOR_YR_PER_S: f64 = 2.5;

struct Args {
    seats: Vec<usize>,
    horizon: f64,
    segment: f64,
    seed: u64,
    hex_side_ly: Option<f64>,
}

fn parse_args() -> Args {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let seats =
        a.first().map(|s| s.split(',').filter_map(|x| x.trim().parse().ok()).collect()).unwrap_or_else(|| vec![3, 12]);
    Args {
        seats,
        horizon: a.get(1).and_then(|s| s.parse().ok()).unwrap_or(800.0),
        segment: a.get(2).and_then(|s| s.parse().ok()).unwrap_or(100.0),
        seed: a.get(3).and_then(|s| s.parse().ok()).unwrap_or(1),
        hex_side_ly: a.get(4).and_then(|s| s.parse().ok()),
    }
}

/// Runs the simulation forward to `until` (or to its horizon) and returns how
/// many events that took.
fn advance_to(sim: &mut Simulation, until: f64) -> u64 {
    let mut n = 0;
    while sim.next_event_time().is_some_and(|t| t <= until) {
        sim.step();
        n += 1;
    }
    n
}

fn bench(seats: usize, args: &Args) {
    let mut gcfg = GalaxyConfig::new(seats, args.seed);
    if let Some(h) = args.hex_side_ly {
        gcfg.hex_side_ly = h;
    }
    let planets = gcfg.planet_count;
    let galaxy = match Galaxy::generate(gcfg) {
        Ok(g) => g,
        Err(e) => {
            println!("{seats} seats: galaxy generation failed: {e:?}");
            return;
        }
    };
    let mut cfg = SimConfig::new(args.seed);
    cfg.horizon_years = args.horizon;
    let mut sim = Simulation::with_baseline(galaxy, cfg);
    println!("--- {seats} seats, seed {}, {planets} planets, horizon {} yr ---", args.seed, args.horizon);
    println!("{:>8} {:>9} {:>10} {:>9} {:>11} {:>9}", "t (yr)", "vehicles", "events", "yr/s", "ns/event", "x floor");
    let _ = std::io::stdout().flush();

    let (mut t, mut events_total, mut wall_total) = (0.0_f64, 0_u64, 0.0_f64);
    while t < args.horizon {
        let until = (t + args.segment).min(args.horizon);
        let t0 = Instant::now();
        let events = advance_to(&mut sim, until);
        let wall = t0.elapsed().as_secs_f64().max(1e-9);
        let vehicles = sim.snapshot().vehicles.len();
        let rate = (until - t) / wall;
        let ns = if events > 0 { wall * 1e9 / events as f64 } else { 0.0 };
        println!("{until:>8.0} {vehicles:>9} {events:>10} {rate:>9.1} {ns:>11.0} {:>9.1}", rate / FLOOR_YR_PER_S);
        let _ = std::io::stdout().flush();
        events_total += events;
        wall_total += wall;
        t = until;
    }
    let rate = args.horizon / wall_total.max(1e-9);
    println!(
        "   whole run: {rate:.1} yr/s, {events_total} events, {:.0} ns/event, {:.1}x the floor\n",
        wall_total * 1e9 / events_total.max(1) as f64,
        rate / FLOOR_YR_PER_S
    );
}

fn main() {
    let args = parse_args();
    for &seats in &args.seats {
        bench(seats, &args);
    }
}
