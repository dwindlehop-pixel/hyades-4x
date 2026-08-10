//! **R-AC3, driven by data.** The autopilot doc has carried this as "deferred
//! to the Monte-Carlo phase" since it was written: *"whether the six
//! [opening-fan-out] vehicles own fixed sectors or pool on global
//! nearest-unscanned — revisit once the Monte-Carlo system exists and
//! prioritization can be measured"* (`Hyades_autopilot_colonization_growth.md`
//! §2). The MC system exists now. This is that revisit.
//!
//! ## The objective: time to 10% colonized, not coverage at a fixed horizon
//!
//! Every other search in this tree (`gradient_probe`, `gradient_step`,
//! `min_time_search`) optimizes *how much* of the galaxy gets colonized by a
//! fixed 4,000-year horizon. That is the right objective for the shipped
//! doctrine, but it is close to the wrong one for a *survey-targeting*
//! question specifically: coverage at 4,000 yr is dominated by the
//! late-game snowball (production capacity, hauling, infra), which a survey
//! strategy barely touches once the map is mostly known. Survey strategy
//! only has leverage while the map is still *thin* — the opening decades.
//! **Time to 10% colonized** is the objective that is actually sensitive to
//! this knob, because it lives entirely inside the window where "which
//! world does the next scout fly to" still matters.
//!
//! ## Three strategies, common random numbers
//!
//! [`SurveyStrategy`] (`src/autopilot.rs`) names three candidates:
//!
//! - `GlobalPool` — no heading bias anywhere, ever; every scout, opening
//!   fan-out included, always flies to the globally nearest unscanned world.
//! - `OpeningSectors` — **the shipped default, unchanged by adding this
//!   knob.** The six opening craft keep a soft heading bias (their cube-face
//!   hemisphere, falling back to global once it's exhausted) for their whole
//!   hop chain; every later, paid-for Scout pools globally from the start.
//! - `PersistentSectors` — like `OpeningSectors`, but every later Scout also
//!   inherits a heading, outward from home through the production center
//!   that built it, so sector discipline doesn't evaporate the moment the
//!   empire starts building replacement scouts.
//!
//! All three run on the *same* seeds (common random numbers, CLAUDE.md §2)
//! so the comparison is paired, not a comparison of noisy means.
//!
//! Run: `cargo run --release --example survey_strategy_search`

use std::collections::{HashMap, HashSet};
use std::io::Write;

use hyades_engine::autopilot::{Autopilot, BaselineAutopilot, Doctrine, SurveyStrategy};
use hyades_engine::log::{LogCategory, LogEvent, LogFilter};
use hyades_engine::prelude::*;

/// Common random numbers — the same seed bed `gradient_probe` uses, so a
/// finding here is directly comparable to the rest of the tree's search work.
const SEEDS: &[u64] = &[1, 7, 42, 31337];
const PLAYERS: usize = 3;
/// The objective threshold: simulated years until 10% of colonizable worlds
/// are colonized.
const TARGET_FRACTION: f64 = 0.10;

fn coverage_targets(galaxy: &Galaxy) -> HashSet<PlanetId> {
    galaxy.planets.iter().filter(|p| p.habitability.min(p.biosphere) > 0.01).map(|p| p.id).collect()
}

/// Years until `TARGET_FRACTION` of colonizable worlds are colonized, or
/// `None` if the horizon runs out first (right-censored).
fn time_to_target(seed: u64, cfg: SimConfig, doctrine: Doctrine) -> Option<f64> {
    let galaxy = Galaxy::generate(GalaxyConfig::new(PLAYERS, seed)).unwrap();
    let targets = coverage_targets(&galaxy);
    let need = ((targets.len() as f64) * TARGET_FRACTION).ceil() as usize;

    let autopilots: Vec<Box<dyn Autopilot>> =
        (0..PLAYERS).map(|_| Box::new(BaselineAutopilot::new(doctrine)) as Box<_>).collect();
    let mut sim = Simulation::new(galaxy, cfg, autopilots);
    sim.set_log_filter(LogFilter::none().with(LogCategory::Vehicles));
    sim.run();

    let mut covered_at: HashMap<PlanetId, f64> = HashMap::new();
    for p in sim.snapshot().planets.iter().filter(|p| p.is_homeworld) {
        if targets.contains(&p.id) {
            covered_at.insert(p.id, 0.0);
        }
    }
    for rec in sim.log().by_category(LogCategory::Vehicles) {
        if let LogEvent::ColonyFounded { planet, .. } = rec.event {
            if targets.contains(&planet) {
                covered_at.entry(planet).or_insert(rec.time);
            }
        }
    }
    if covered_at.len() < need {
        return None;
    }
    let mut times: Vec<f64> = covered_at.into_values().collect();
    times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    Some(times[need - 1])
}

/// Per-seed time-to-target vector — the CRN unit. `horizon_years` stands in
/// for "never reached" (right-censored) so a strategy that fails outright is
/// heavily penalized rather than silently dropped from the pairing.
fn profile(cfg: SimConfig, doctrine: Doctrine) -> Vec<f64> {
    SEEDS.iter().map(|&s| time_to_target(s, cfg, doctrine).unwrap_or(cfg.horizon_years)).collect()
}

fn mean(v: &[f64]) -> f64 {
    v.iter().sum::<f64>() / v.len() as f64
}

fn stderr(v: &[f64]) -> f64 {
    let n = v.len() as f64;
    if n < 2.0 {
        return f64::INFINITY;
    }
    let m = mean(v);
    (v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (n - 1.0) / n).sqrt()
}

fn report(name: &str, times: &[f64], baseline: Option<&[f64]>) {
    print!(
        "  {name:<18} mean {:>7.1} yr   per-seed {:?}",
        mean(times),
        times.iter().map(|t| format!("{t:.0}")).collect::<Vec<_>>()
    );
    if let Some(b) = baseline {
        let diffs: Vec<f64> = times.iter().zip(b.iter()).map(|(a, c)| a - c).collect();
        let d = mean(&diffs);
        let se = stderr(&diffs);
        let verdict = if d.abs() < 2.0 * se {
            "~noise"
        } else if d < 0.0 {
            "FASTER"
        } else {
            "slower"
        };
        print!("   Δ vs baseline {:>+7.1} ± {:.1} yr  [{verdict}]", d, se);
    }
    println!();
    std::io::stdout().flush().ok();
}

fn main() {
    let cfg = SimConfig::new(0);
    let base_doctrine = Doctrine::default();

    println!(
        "R-AC3 survey-strategy search — {PLAYERS} seats, {} seeds (CRN), objective: years to {:.0}% colonized\n",
        SEEDS.len(),
        TARGET_FRACTION * 100.0
    );

    println!("=== Round 1: the three named strategies, shipped doctrine otherwise ===");
    let opening = profile(cfg, Doctrine { survey_strategy: SurveyStrategy::OpeningSectors, ..base_doctrine });
    report("OpeningSectors", &opening, None);
    report(
        "GlobalPool",
        &profile(cfg, Doctrine { survey_strategy: SurveyStrategy::GlobalPool, ..base_doctrine }),
        Some(&opening),
    );
    let persistent = profile(cfg, Doctrine { survey_strategy: SurveyStrategy::PersistentSectors, ..base_doctrine });
    report("PersistentSectors", &persistent, Some(&opening));

    println!(
        "\nReading: negative Δ means faster (better). ~noise means the difference is inside\n\
         2 standard errors of zero on this 4-seed bed and should not be trusted without a\n\
         wider bed (CLAUDE.md §2). This objective (years to {:.0}% colonized) is deliberately\n\
         different from the rest of the tree's 4,000-year coverage objective — it isolates\n\
         the opening decades, which is the only window a survey-targeting knob can reach.",
        TARGET_FRACTION * 100.0
    );
}
