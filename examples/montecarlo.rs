//! A tiny Monte-Carlo harness — the same engine the balancer uses
//! (`Hyades_card_contract.md` §7). Generates a galaxy, runs the event-driven ECS
//! to the horizon for several seeds, and prints per-seat outcomes.
//!
//! Run with:  `cargo run --example montecarlo`

use hyades_engine::prelude::*;

fn main() {
    let players = 3;
    let seeds = [1u64, 2, 7, 42, 2024];

    println!("Hyades engine — colonization/growth autopilot, {players} seats\n");
    println!("{:>5}  {:>7}  {:>9}  {:>10}  {:>10}  {:>9}", "seed", "events", "scanned", "colonies", "outposts", "pop");
    println!("{}", "-".repeat(60));

    for &seed in &seeds {
        let galaxy = Galaxy::generate(GalaxyConfig::new(players, seed)).expect("fair count");
        let mut sim = Simulation::with_baseline(galaxy, SimConfig::new(seed));
        let report = sim.run();

        let colonies: usize = report.players.iter().map(|p| p.colonies).sum();
        let outposts: usize = report.players.iter().map(|p| p.mining_outposts).sum();
        let pop: f64 = report.players.iter().map(|p| p.total_population).sum();

        println!(
            "{:>5}  {:>7}  {:>9}  {:>10}  {:>10}  {:>9.2}",
            seed, report.events_processed, report.planets_scanned_total, colonies, outposts, pop
        );
    }

    // Show one detailed snapshot so the read-only presentation seam is visible.
    println!("\nDetailed final state (seed 42):");
    let galaxy = Galaxy::generate(GalaxyConfig::new(players, 42)).unwrap();
    let mut sim = Simulation::with_baseline(galaxy, SimConfig::new(42));
    sim.run();
    let snap = sim.snapshot();
    println!("  t = {:.0} yr", snap.time_years);
    for (p, ps) in snap.players.iter().enumerate() {
        println!(
            "  seat {p}: {} planets, {} outposts, {} scanned, pop {:.2}",
            ps.planets_owned, ps.mining_outposts, ps.planets_scanned, ps.total_population
        );
    }
}
