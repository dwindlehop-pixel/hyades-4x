//! Determinism stress tests.
//!
//! The engine must be bit-reproducible for the Monte-Carlo balancer
//! (`Hyades_card_contract.md` §7) and must expose deterministic continuous
//! `(x, y, z)` for every entity at every instant. These tests hammer both:
//! identical seeds must agree exactly on outcomes *and* on densely-sampled
//! in-flight positions across the entire timeline.

use hyades_engine::prelude::*;

/// Build a run with an explicit, short horizon — determinism holds at any
/// point in the run, so proving it does not need the full default horizon.
/// Every test here pins one explicitly: the shipped defaults snowball to
/// thousands of vehicles across 4,000 years (CLAUDE.md design law #9), and a
/// full-length debug run costs a minute-plus each. Bit-identity is a property
/// of the arithmetic, not of how long you let it accumulate.
fn fresh_short(players: usize, seed: u64, horizon_years: f64) -> Simulation {
    let galaxy = Galaxy::generate(GalaxyConfig::new(players, seed)).unwrap();
    let mut cfg = SimConfig::new(seed);
    cfg.horizon_years = horizon_years;
    Simulation::with_baseline(galaxy, cfg)
}

#[test]
fn full_run_reports_are_bit_identical() {
    // Every fair seat count, 18 included (R-NET14). `Hyades_netcode.md` §6 makes
    // bit-reproducibility a *network* property, not only an MC one: a divergence
    // at any seat count is a desync, and 18 is the count the protocol is now
    // specified for. Cheap to cover — the horizon is pinned at 100 yr.
    for &(n, seed) in &[(2usize, 1u64), (3, 7), (6, 13), (12, 99), (18, 4)] {
        let mut a = fresh_short(n, seed, 100.0);
        let mut b = fresh_short(n, seed, 100.0);
        let ra = a.run();
        let rb = b.run();
        assert_eq!(ra.events_processed, rb.events_processed, "events n={n} seed={seed}");
        assert_eq!(ra.planets_scanned_total, rb.planets_scanned_total);
        for (pa, pb) in ra.players.iter().zip(rb.players.iter()) {
            assert_eq!(pa.planets_owned, pb.planets_owned);
            assert_eq!(pa.colonies, pb.colonies);
            assert_eq!(pa.mining_outposts, pb.mining_outposts);
            assert_eq!(pa.total_population.to_bits(), pb.total_population.to_bits());
        }
    }
}

#[test]
fn continuous_positions_are_bit_identical_across_the_timeline() {
    // Two runs of the same seed, stepped in lockstep; at a dense grid of sample
    // times we compare every entity's (x, y, z) bit-for-bit. In-flight ships make
    // this a real test (their positions are interpolated, not snapped to events).
    // 3 players / 1500 steps, not 6 / 6000: doesn't need the biggest player
    // count or the full step budget to prove the property, and the galaxy is
    // now thousands of planets at the default 10 ly hex (this conversation) —
    // `checks` below still clears its floor by a wide margin either way.
    let mut a = fresh_short(3, 4242, 800.0);
    let mut b = fresh_short(3, 4242, 800.0);

    let mut checks = 0u64;
    for _ in 0..1500 {
        let a_more = a.step();
        let b_more = b.step();
        assert_eq!(a_more, b_more);
        if !a_more {
            break;
        }
        let t = a.clock();
        // sample slightly before/at/after the current event time
        for &s in &[t - 3.0, t, t + 3.0, t + 11.0] {
            let pa = a.positions_at(s);
            let pb = b.positions_at(s);
            assert_eq!(pa.len(), pb.len());
            for (x, y) in pa.iter().zip(pb.iter()) {
                assert_eq!(x.x.to_bits(), y.x.to_bits(), "x mismatch at t={s}");
                assert_eq!(x.y.to_bits(), y.y.to_bits(), "y mismatch at t={s}");
                assert_eq!(x.z.to_bits(), y.z.to_bits(), "z mismatch at t={s}");
            }
            checks += pa.len() as u64;
        }
    }
    assert!(checks > 10_000, "stress test did not sample enough positions ({checks})");
}

#[test]
fn positions_never_exceed_lightspeed() {
    // Sample displacement over small windows the whole way to the horizon; no
    // entity may move faster than c (= 1 ly/yr).
    let mut sim = fresh_short(3, 808, 800.0);
    sim.run();
    let dt = 0.25;
    let mut t = 0.0;
    while t < 800.0 {
        let p0 = sim.positions_at(t);
        let p1 = sim.positions_at(t + dt);
        for (u, v) in p0.iter().zip(p1.iter()) {
            assert!(u.distance(*v) <= dt + 1e-6, "superluminal motion near t={t}");
        }
        t += 7.0; // stride across the timeline
    }
}

#[test]
fn stepping_in_any_granularity_reaches_the_same_state() {
    // Driving the loop by hand must match run() exactly. Shorter horizon
    // (this conversation: galaxies are now thousands of planets at the
    // default 10 ly hex) — the property holds at any horizon length.
    let mut a = fresh_short(6, 555, 100.0);
    while a.step() {}
    let ra = a.report();

    let mut b = fresh_short(6, 555, 100.0);
    let rb = b.run();

    assert_eq!(ra.events_processed, rb.events_processed);
    assert_eq!(ra.planets_scanned_total, rb.planets_scanned_total);
    for (pa, pb) in ra.players.iter().zip(rb.players.iter()) {
        assert_eq!(pa.planets_owned, pb.planets_owned);
        assert_eq!(pa.total_population.to_bits(), pb.total_population.to_bits());
    }
}

/// **H3 / R-NET11 — no NaN reaches replicated state.**
///
/// `Hyades_netcode.md` §6 H3: core WASM picks arithmetic-NaN payloads
/// *nondeterministically*, so NaN bits differ across browser engines even with
/// relaxed SIMD disabled. A NaN that reaches the hashed state is therefore a
/// latent, intermittent, unreproducible desync — the worst failure class the
/// protocol has, because it presents as one client being "wrong" with no
/// reproducer.
///
/// The state digest (§8.1) does not exist yet, so this guards the two surfaces
/// that will feed it: the report and the full snapshot. It is the seed of the
/// H3 discipline rather than the whole of it — when the digest lands, the check
/// belongs *inside* it, as a fatal error rather than a test.
///
/// Infinities are checked too. They are deterministic in WASM, so they are not
/// a desync, but they reach NaN in one subtraction and there is no legitimate
/// infinite quantity in this model.
#[test]
fn no_nan_or_infinity_reaches_replicated_state() {
    let mut sim = fresh_short(6, 31337, 400.0);
    let report = sim.run();

    let finite = |v: f64, what: &str| assert!(v.is_finite(), "{what} is not finite: {v}");

    for (i, p) in report.players.iter().enumerate() {
        finite(p.total_population, &format!("report.players[{i}].total_population"));
    }

    let snap = sim.snapshot();
    finite(snap.time_years, "snapshot.time_years");
    for (i, p) in snap.players.iter().enumerate() {
        finite(p.total_population, &format!("players[{i}].total_population"));
        finite(p.stockpiled_total, &format!("players[{i}].stockpiled_total"));
    }
    for pl in &snap.planets {
        let id = pl.id.0;
        for (v, what) in [
            (pl.position.x, "position.x"),
            (pl.position.y, "position.y"),
            (pl.position.z, "position.z"),
            (pl.habitability, "habitability"),
            (pl.biosphere, "biosphere"),
            (pl.infrastructure, "infrastructure"),
            (pl.k, "k"),
            (pl.population, "population"),
            (pl.stockpile.basic_total(), "stockpile"),
        ] {
            finite(v, &format!("planet {id} {what}"));
        }
    }
    for (i, v) in snap.vehicles.iter().enumerate() {
        for (val, what) in [
            (v.position.x, "position.x"),
            (v.position.y, "position.y"),
            (v.position.z, "position.z"),
            (v.cargo.basic_total(), "cargo"),
        ] {
            finite(val, &format!("vehicle {i} {what}"));
        }
    }
}
