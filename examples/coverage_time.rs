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
//! **Offline only since T-68 — it is no longer in CI, and the reason is not
//! the clock.** Its second bed contrasts the default `medium_fleet_size`
//! against a "past the optimum" 6.0, and that comparison **went vacuous**:
//! T-56 stage 3c decoupled the cost ladder from the capacity ladder, so the
//! knob is now a *price* and not also a hold, and R-IND11's ablation B measured
//! that coloniser hull price is worth −0.08% — so both beds print identical
//! coverage (3,336 / 3,349 on seeds 1 / 7). It also grew to ~8 minutes. A check
//! that spends eight minutes printing a tautology is not a check.
//!
//! It is kept because the *first* bed still reports something real — coverage
//! against the K>0 target set — and because the completion column is an honest
//! statement of a structural fact (see the closing note). Give it a live second
//! bed, or a knob that still discriminates, before putting it back in CI.
//!
//! Run with:  `cargo run --release --example coverage_time`

use std::collections::{HashMap, HashSet};

use hyades_engine::log::{LogCategory, LogEvent, LogFilter};
use hyades_engine::prelude::*;
use hyades_engine::units::Band;

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
    galaxy.planets.iter().filter(|p| p.habitability.min(p.biosphere) > Band::new(0.01)).map(|p| p.id).collect()
}

/// **The horizon this driver runs at, and why it is not the objective's.**
///
/// 2,000 yr — `CLAUDE.md` §2's calibrated screen (rank agreement ρ = 0.923 with
/// the 4,000-year objective, ~31x cheaper). This harness exists for the
/// **doctrine comparison** between the two beds in `main`, and those two differ
/// by roughly a factor of two in coverage; a screen that ranks working
/// configurations at ρ = 0.923 settles a 2x gulf with room to spare.
///
/// **What the trim costs**, stated rather than left to look free: the absolute
/// coverage figures printed here are no longer comparable to the 4,000-year
/// numbers quoted in `CLAUDE.md` §7, and a *narrow* doctrine difference could
/// hide inside the screen's disagreement with the objective. Anything that
/// close belongs in the offline search, which is not time-boxed.
///
/// It moved because T-68 made `t_build` track hull mass and cost ~19x of
/// throughput, which took this driver from ~49 s to over 25 minutes — it was
/// **cancelled in CI**, which is how the cost was noticed.
const SCREEN_HORIZON_YEARS: f64 = 2000.0;

/// Run one trial. Returns (completion_time_years if fully covered within the
/// horizon else None, planets covered, total targets, horizon).
fn trial(seed: u64, mut cfg: SimConfig) -> (Option<f64>, usize, usize, f64) {
    cfg.horizon_years = SCREEN_HORIZON_YEARS;
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
        SCREEN_HORIZON_YEARS
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
    // directly aimed at minimizing time-to-coverage. **This now demonstrates
    // the non-monotonicity directly rather than assuming a direction.**
    //
    // The shipped default is 4.45 (T-45's verified gradient step, chasing this
    // exact knob). 6.0 is a further push the *same* way — nominally cheaper
    // colonizers still — and legal (`hull_ladder_fault` refuses only an
    // inverted ladder, `medium_fleet_size >= limited_fleet_size`; the earlier
    // narrower bound was a normalisation artifact, see that function's doc
    // comment). But it is worse, not better: coverage drops from 3,348/3,452 at
    // the default to roughly half that here. This is the cliff the gradient
    // step deliberately stopped short of (`gradient_step`'s α=1.0 row collapses
    // to 6.9%) — 6.0 is not past the cliff, but it is past the optimum, which
    // is the point: past some value a Medium hull's shrinking hold outweighs
    // its lower price, and "cheaper colonizers" stops being unambiguously good.
    let mut expand_cfg = SimConfig::new(0);
    expand_cfg.medium_fleet_size = 6.0;
    assert!(expand_cfg.hull_ladder_fault().is_none());
    run_test_bed("Past the optimum (medium_fleet_size=6, default is 4.45)", expand_cfg);

    println!(
        "\nReading: coverage is what fraction of the galaxy's K_potential>0 worlds\n\
         anyone has colonized by the {SCREEN_HORIZON_YEARS:.0} yr screen horizon, and the\n\
         comparison between the two beds above is what this driver is for.\n\
         \n\
         **The completion column will not fire, and that is structural.** The target\n\
         set is every world with min(hab,bio) > 0.01, but the autopilot's `k_high`\n\
         classifier admits only 51-53% of the galaxy (`examples/reach_limit`), so\n\
         '100% of K>0 colonized' is unreachable at any horizon — the bed already takes\n\
         ~99% of what `k_high` allows. The column is kept because it is the honest\n\
         report of that: it says >horizon, every time, on purpose. Widening the\n\
         classifier is the question (R-AC18), not lengthening the run.\n\
         \n\
         Run `min_time_search` offline for the coordinate-descent search."
    );
}
