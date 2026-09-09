//! A tiny Monte-Carlo harness — the same engine the balancer uses
//! (`Hyades_card_contract.md` §7). Generates a galaxy, runs the event-driven ECS
//! to the horizon for several seeds, and prints per-seat outcomes.
//!
//! Run with:  `cargo run --example montecarlo`

use hyades_engine::prelude::*;

/// **This driver's job in CI is to prove the engine runs, not to measure it.**
///
/// The workflow calls it so "a panic or a NaN-poisoned sweep fails CI", and that
/// question needs seed *breadth* — a galaxy-specific panic hides on four seeds
/// and shows on five — but it does not need the full objective horizon.
///
/// **Pinned at 1,000 yr, explicitly** (design law #14's corollary). At the
/// shipped defaults that is ~2,500 colonies on seed 1: deep into the snowball,
/// with colonisation, mining, hauling and recycling all exercised. The default
/// 4,000 costs **35x** as much (`examples/horizon_cost`) because cost is
/// violently superlinear in duration, and it was 6 full runs — which is how this
/// job came to blow its 25-minute CI budget after T-68.
///
/// **What the trim costs:** nothing this driver was asked for. It prints
/// outcomes rather than asserting them, so no check is weakened; the numbers in
/// its table simply stop being comparable to 4,000-year figures quoted
/// elsewhere. Anything that wants the objective wants `colony_years` or the
/// offline search.
const HORIZON_YEARS: f64 = 1000.0;

fn cfg(seed: u64) -> SimConfig {
    let mut c = SimConfig::new(seed);
    c.horizon_years = HORIZON_YEARS;
    c
}

fn main() {
    let players = 3;
    let seeds = [1u64, 2, 7, 42, 2024];

    println!("Hyades engine — colonization/growth autopilot, {players} seats\n");
    println!("{:>5}  {:>7}  {:>9}  {:>10}  {:>10}  {:>9}", "seed", "events", "scanned", "colonies", "outposts", "pop");
    println!("{}", "-".repeat(60));

    let mut detail: Option<Simulation> = None;
    for &seed in &seeds {
        let galaxy = Galaxy::generate(GalaxyConfig::new(players, seed)).expect("fair count");
        let mut sim = Simulation::with_baseline(galaxy, cfg(seed));
        let report = sim.run();

        let colonies: usize = report.players.iter().map(|p| p.colonies).sum();
        let outposts: usize = report.players.iter().map(|p| p.mining_outposts).sum();
        let pop: f64 = report.players.iter().map(|p| p.total_population.kilotons()).sum();

        println!(
            "{:>5}  {:>7}  {:>9}  {:>10}  {:>10}  {:>9.2}",
            seed, report.events_processed, report.planets_scanned_total, colonies, outposts, pop
        );
        if seed == 42 {
            detail = Some(sim);
        }
    }

    // Show one detailed snapshot so the read-only presentation seam is visible.
    //
    // **Reusing the run rather than repeating it.** Seed 42 is already in the
    // loop above, so building a second identical galaxy and running it again was
    // a sixth full sim for a view of a state we had just computed — free before
    // T-68, and ~11 s of a 60 s budget after it.
    println!("\nDetailed final state (seed 42):");
    let snap = detail.expect("seed 42 is in the bed above").snapshot();
    println!("  t = {:.0} yr", snap.time_years);
    for (p, ps) in snap.players.iter().enumerate() {
        println!(
            "  seat {p}: {} planets, {} outposts, {} scanned, pop {:.2}",
            ps.planets_owned, ps.mining_outposts, ps.planets_scanned, ps.total_population
        );
    }
}
