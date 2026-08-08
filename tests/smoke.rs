//! End-to-end smoke test of the public engine API (the ECS simulation seen only
//! through its `Simulation`/`SimReport`/`Snapshot` surface — no internals).

use hyades_engine::prelude::*;

/// Shorter horizon than the default — galaxies are now thousands of planets
/// at the default 10 ly hex (this conversation, benchmarked in
/// `examples/bench_hex_size.rs`); release-mode throughput comfortably
/// clears the confirmed 2.5-simulated-years/real-second target, but these
/// tests don't need the full default horizon to prove what they're checking.
fn run_short(players: usize, seed: u64, horizon_years: f64) -> (Simulation, SimReport) {
    let galaxy = Galaxy::generate(GalaxyConfig::new(players, seed)).unwrap();
    let mut cfg = SimConfig::new(seed);
    cfg.horizon_years = horizon_years;
    let mut sim = Simulation::with_baseline(galaxy, cfg);
    let report = sim.run();
    (sim, report)
}

#[test]
fn all_fair_counts_run_and_expand() {
    for &n in &[2usize, 3, 6, 12] {
        let (_sim, report) = run_short(n, 100 + n as u64, 500.0);
        assert_eq!(report.players.len(), n);
        assert!(report.planets_scanned_total >= n, "no scanning for {n} seats");
        let colonies: usize = report.players.iter().map(|p| p.colonies).sum();
        assert!(colonies > 0, "no expansion for {n} seats: {report:?}");
    }
}

#[test]
fn snapshot_is_consistent_with_report() {
    let (sim, report) = run_short(6, 314, 500.0);
    let snap = sim.snapshot();
    // Owned-planet totals computed two different ways must agree.
    let owned_from_report: usize = report.players.iter().map(|p| p.planets_owned).sum();
    let owned_from_snapshot = snap.planets.iter().filter(|p| p.owner.is_some()).count();
    assert_eq!(owned_from_report, owned_from_snapshot);
    // K = min(hab,bio,infra) must hold for every planet snapshot.
    for p in &snap.planets {
        let expected = p.habitability.min(p.biosphere).min(p.infrastructure);
        assert!((p.k - expected).abs() < 1e-9);
    }
}

#[test]
fn determinism_across_full_runs() {
    let (_a, ra) = run_short(6, 77, 500.0);
    let (_b, rb) = run_short(6, 77, 500.0);
    assert_eq!(ra.events_processed, rb.events_processed);
    assert_eq!(ra.planets_scanned_total, rb.planets_scanned_total);
}

#[test]
fn stepping_matches_running() {
    // Driving the event loop by hand must reach the same place as run().
    let galaxy = Galaxy::generate(GalaxyConfig::new(3, 9)).unwrap();
    let mut cfg = SimConfig::new(9);
    cfg.horizon_years = 500.0;
    let mut a = Simulation::with_baseline(galaxy, cfg);
    while a.step() {}
    let ra = a.report();

    let (_b, rb) = run_short(3, 9, 500.0);
    assert_eq!(ra.events_processed, rb.events_processed);
    assert_eq!(ra.planets_scanned_total, rb.planets_scanned_total);
}
