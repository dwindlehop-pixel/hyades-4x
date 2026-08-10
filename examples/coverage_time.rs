//! Coverage-time harness — *"the Monte Carlo trials are intended to find
//! the minimum time algorithm to cover the map in a standard test bed."*
//!
//! The requirement (this conversation), stated precisely: *"I do not care
//! about autopilot colonizing 100% of planets, but autopilot must colonize
//! 100% of K>0 planets given sufficient time."* This harness measures
//! exactly that — not "colonies at year 4000" (what `montecarlo.rs` and
//! `fleet_size_tuning.rs` measure), but **the game-year at which every
//! `K_potential > 0` planet in the galaxy has been colonized by someone**,
//! or an honest report that the horizon ran out first.
//!
//! `K_potential > 0` (not the current built `K`, which is `0` on every wild
//! world by definition until colonized) is `min(habitability, biosphere) >
//! 0` — "habitable in principle, for somebody" — under the current
//! single-scalar habitability model. Once `Hyades_habitability.md` lands in
//! code, only the *definition* of the target set changes; this harness's
//! methodology (a fixed standard test bed, completion time as the metric)
//! carries over unchanged.
//!
//! Run with:  `cargo run --release --example coverage_time`

use std::collections::{HashMap, HashSet};

use hyades_engine::log::{LogCategory, LogEvent, LogFilter};
use hyades_engine::prelude::*;

/// The standard test bed: a fixed seed set, fixed player count. Every
/// algorithm/parameter variant gets compared against the *same* galaxies, so
/// differences in outcome are the algorithm's doing, not sampling luck.
/// Two of the ten standard test-bed seeds. Each trial is now a full snowball
/// run (design law #9), so the original ten across two doctrines was 20 runs and
/// ~227 s — well past the 60 s budget CI targets (three seeds still cost 66 s).
/// Two preserves what this driver is actually for: the *doctrine comparison*
/// between the two beds below. Seed breadth is the offline search's job, and it
/// is not time-boxed. The full bed is kept here for it to reference.
const TEST_BED_SEEDS: &[u64] = &[1, 7];
#[allow(dead_code)]
const FULL_TEST_BED: &[u64] = &[1, 7, 42, 55, 99, 123, 2024, 31337, 8675309, 271828];
const PLAYERS: usize = 3;

/// `K_potential > 0` planets — the actual coverage target, not raw planet
/// count (`min(habitability, biosphere)`, not the current built `K`, which
/// is 0 everywhere wild by definition).
fn coverage_targets(galaxy: &Galaxy) -> HashSet<PlanetId> {
    galaxy.planets.iter().filter(|p| p.habitability.min(p.biosphere) > 0.01).map(|p| p.id).collect()
}

/// Run one trial. Returns (completion_time_years if fully covered within the
/// horizon else None, planets covered, total targets, horizon).
fn trial(seed: u64, cfg: SimConfig) -> (Option<f64>, usize, usize, f64) {
    let galaxy = Galaxy::generate(GalaxyConfig::new(PLAYERS, seed)).unwrap();
    let targets = coverage_targets(&galaxy);
    let total = targets.len();
    let horizon = cfg.horizon_years;

    let mut sim = Simulation::with_baseline(galaxy, cfg);
    sim.set_log_filter(LogFilter::none().with(LogCategory::Vehicles));
    sim.run();

    // Homeworlds are covered at t=0 (already owned at bootstrap); everything
    // else is covered at its ColonyFounded time, if any.
    let mut covered_at: HashMap<PlanetId, f64> = HashMap::new();
    for &hw in &sim_homeworld_ids(&sim) {
        if targets.contains(&hw) {
            covered_at.insert(hw, 0.0);
        }
    }
    for rec in sim.log().by_category(LogCategory::Vehicles) {
        if let LogEvent::ColonyFounded { planet, .. } = rec.event {
            if targets.contains(&planet) {
                covered_at.entry(planet).or_insert(rec.time);
            }
        }
    }

    let covered = covered_at.len();
    let completion = if covered == total {
        covered_at.values().cloned().fold(0.0_f64, f64::max)
    } else {
        f64::NAN // sentinel; caller checks covered == total instead of relying on this
    };
    (if covered == total { Some(completion) } else { None }, covered, total, horizon)
}

/// The homeworld planet ids for a simulation — read via a snapshot rather
/// than needing engine internals; homeworld planets are exactly the ones
/// each player owns at time 0 with `is_homeworld` set, but the simplest
/// stable way from outside the engine is: every player's very first owned
/// planet. Snapshot gives us owner + is_homeworld directly.
fn sim_homeworld_ids(sim: &Simulation) -> Vec<PlanetId> {
    let snap = sim.snapshot();
    snap.planets.iter().filter(|p| p.is_homeworld).map(|p| p.id).collect()
}

fn run_test_bed(label: &str, cfg: SimConfig) {
    println!("\n{label}");
    println!("{:>10}  {:>10}  {:>10}  {:>14}  {:>10}", "seed", "covered", "targets", "completion_yr", "status");
    println!("{}", "-".repeat(62));

    let mut completions = Vec::new();
    let mut full_coverage_count = 0;
    for &seed in TEST_BED_SEEDS {
        let (completion, covered, total, horizon) = trial(seed, cfg);
        let status = match completion {
            Some(_) => {
                full_coverage_count += 1;
                "FULL"
            }
            None => "incomplete",
        };
        let completion_str = completion.map(|t| format!("{t:.0}")).unwrap_or_else(|| format!(">{horizon:.0}"));
        println!("{seed:>10}  {covered:>10}  {total:>10}  {completion_str:>14}  {status:>10}");
        if let Some(t) = completion {
            completions.push(t);
        }
    }

    println!(
        "\n{full_coverage_count}/{} seeds reached 100% K>0 coverage within the {} yr horizon.",
        TEST_BED_SEEDS.len(),
        cfg.horizon_years
    );
    if !completions.is_empty() {
        let mean = completions.iter().sum::<f64>() / completions.len() as f64;
        println!("Mean completion time (seeds that finished): {mean:.0} yr.");
    }
}

fn main() {
    println!("Coverage-time — standard test bed, {PLAYERS} seats, {} seeds", TEST_BED_SEEDS.len());
    println!("Target: 100% of K_potential>0 planets colonized by someone.");

    run_test_bed("Baseline doctrine (current defaults)", SimConfig::new(0));

    // A doctrine biased hard toward expansion, since that's the lever most
    // directly aimed at minimizing time-to-coverage.
    //
    // **4.0, not the 6.0 this used to run.** Since R-O58 the cost ladder is also
    // the *capacity* ladder — hull radius is `sqrt(cost / cost_Limited)` — so
    // 6.0 against a `limited_fleet_size` of 9 leaves the Medium hull barely
    // larger than the Limited one and a General hull holding ~700x its load.
    // `SimConfig::hull_ladder_fault` refuses it, and rightly: the comparison
    // this makes is "cheaper colonizers", not "break the contents ladder".
    let mut expand_cfg = SimConfig::new(0);
    expand_cfg.medium_fleet_size = 4.0; // cheaper Colonizers than the 3.0 default
    assert!(expand_cfg.hull_ladder_fault().is_none());
    run_test_bed("Cheaper Colonizers (medium_fleet_size=4)", expand_cfg);

    println!(
        "\nReading: coverage is what fraction of the galaxy's K_potential>0 worlds\n\
         anyone has colonized by the horizon. Under the ratified snowball defaults\n\
         (R-AC16/R-AC17) expansion compounds instead of stalling, so the interesting\n\
         question is no longer *whether* it moves but how far it gets in 4,000 years —\n\
         full coverage currently needs closer to 8,000. Run `min_time_search` offline\n\
         for the coordinate-descent search over the remaining parameters."
    );
}
