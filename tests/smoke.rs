//! End-to-end smoke test of the public engine API (the ECS simulation seen only
//! through its `Simulation`/`SimReport`/`Snapshot` surface — no internals).

use hyades_engine::prelude::*;
use hyades_engine::units::Measure;

/// Shorter horizon than the default — galaxies are now thousands of planets
/// at the default 10 ly hex (this conversation, benchmarked in
/// `examples/bench_hex_size.rs`); release-mode throughput comfortably
/// clears the confirmed 2.5-simulated-years/real-second target, but these
/// tests don't need the full default horizon to prove what they're checking.
///
/// **500 → 300 yr at T-68**, for the reason `CLAUDE.md` §2 gives for watching
/// test *targets* rather than test *asks*: `t_build` now tracks hull mass, so a
/// Medium hull takes 3.0 yr instead of 10, centres decide three times as often,
/// and the entity count follows. Nothing here asks a long-run question — every
/// assertion is an invariant that holds at any horizon where expansion has
/// started — so the horizon is the part that was safe to cut.
fn run_short(players: usize, seed: u64, horizon_years: f64) -> (Simulation, SimReport) {
    let galaxy = Galaxy::generate(GalaxyConfig::new(players, seed)).unwrap();
    let mut cfg = SimConfig::new(seed);
    cfg.horizon_years = horizon_years;
    let mut sim = Simulation::with_baseline(galaxy, cfg);
    let report = sim.run();
    (sim, report)
}

/// **The horizon is 150 yr, and every assertion here is an invariant** — the
/// seat count comes back, scanning happened, expansion happened. `CLAUDE.md`
/// §2: an invariant needs the mechanism to have fired *once*, not a long run to
/// accumulate in. 300 yr was inherited from when a simulated year was cheap;
/// T-69 and T-71/T-72 have each roughly doubled entity count since, and this
/// test alone was **65.8 s of a 68 s target**.
///
/// `colonies > 0` is also the non-vacuity guard: shorten it too far and the
/// assertion fails rather than passing on an empty galaxy, which is the property
/// a trimmed horizon most needs to keep.
#[test]
fn all_fair_counts_run_and_expand() {
    for &n in &[2usize, 3, 6, 12] {
        let (_sim, report) = run_short(n, 100 + n as u64, 150.0);
        assert_eq!(report.players.len(), n);
        assert!(report.planets_scanned_total >= n, "no scanning for {n} seats");
        let colonies: usize = report.players.iter().map(|p| p.colonies).sum();
        assert!(colonies > 0, "no expansion for {n} seats: {report:?}");
    }
}

#[test]
fn snapshot_is_consistent_with_report() {
    let (sim, report) = run_short(6, 314, 150.0);
    let snap = sim.snapshot();
    // Owned-planet totals computed two different ways must agree.
    let owned_from_report: usize = report.players.iter().map(|p| p.planets_owned).sum();
    let owned_from_snapshot = snap.planets.iter().filter(|p| p.owner.is_some()).count();
    assert_eq!(owned_from_report, owned_from_snapshot);
    // `K = min(hab, bio_max)` must hold for every planet snapshot — a minimum
    // over **Bands**, and since T-67 (`Hyades_industry.md` §1.1) **without an
    // infrastructure term**. Infrastructure is the industrial stock: it mines
    // and fabricates and can be razed, and razing it must not move population.
    // The standing biosphere is a mass and is deliberately not a term either;
    // see `hyades_engine::units`.
    for p in &snap.planets {
        let expected = p.habitability.min(p.bio_max);
        assert!((p.k.bands() - expected.bands()).abs() < 1e-9);
        // The standing biosphere never exceeds its own pristine ceiling.
        //
        // Note what is *not* asserted: `biomass + KT(pop) <= bio_max`. People
        // are drawn out of the biosphere but the biosphere regrows toward
        // `bio_max` regardless of them (design law #11, taken literally), so a
        // settled world legitimately carries more living mass than its pristine
        // stock alone. `K` is what caps population; this caps the stock.
        assert!(
            p.biomass.kilotons() <= p.bio_max.in_kilotons().kilotons() + 1e-9,
            "planet {} holds {} of biomass against a {} ceiling",
            p.id.0,
            p.biomass,
            p.bio_max.in_kilotons()
        );
        // And population never exceeds the Liebig ceiling it grows toward.
        assert!(
            p.population.band().bands() <= p.k.bands().max(SimConfig::new(6).colony_seed_pop.band().bands()) + 1e-9,
            "planet {} holds {} against K = {}",
            p.id.0,
            p.population,
            p.k
        );
    }
}

/// **A paired-run identity, so it pays double for a horizon that buys it
/// nothing** — the same argument `sim::paired_cfg` and `tests/determinism.rs`
/// already make. Determinism is a property of the arithmetic, not of how long
/// you accumulate it, so 150 yr proves it exactly as well as 300 and costs half.
#[test]
fn determinism_across_full_runs() {
    let (_a, ra) = run_short(6, 77, 150.0);
    let (_b, rb) = run_short(6, 77, 150.0);
    assert_eq!(ra.events_processed, rb.events_processed);
    assert_eq!(ra.planets_scanned_total, rb.planets_scanned_total);
}

#[test]
fn stepping_matches_running() {
    // Driving the event loop by hand must reach the same place as run().
    let galaxy = Galaxy::generate(GalaxyConfig::new(3, 9)).unwrap();
    let mut cfg = SimConfig::new(9);
    cfg.horizon_years = 150.0;
    let mut a = Simulation::with_baseline(galaxy, cfg);
    while a.step() {}
    let ra = a.report();

    let (_b, rb) = run_short(3, 9, 150.0);
    assert_eq!(ra.events_processed, rb.events_processed);
    assert_eq!(ra.planets_scanned_total, rb.planets_scanned_total);
}
