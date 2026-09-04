//! **Take the gradient step, and verify it before believing it.**
//!
//! `gradient_probe` gives a direction. A direction is not a destination: the
//! elasticities are a *local* linear model, and `medium_fleet_size` at
//! +32.7 pts/ln would predict +13 points from a 0.4 step in log space, which is
//! certainly optimistic — the linearisation stops holding long before that.
//!
//! So this does a **line search along the normalised gradient**, evaluates each
//! candidate on the same CRN bed as the baseline, and reports the paired
//! per-seed improvement. Nothing is ratified that does not beat the baseline on
//! the *same seeds*, which is the only comparison that is not mostly noise.
//!
//! Only knobs whose elasticity cleared 2 SE are moved. The four that did not are
//! deliberately left alone: stepping along noise is how a search wanders.
//!
//! Run: `cargo run --release --example gradient_step`

use std::collections::HashSet;
use std::io::Write;

use hyades_engine::autopilot::{Autopilot, BaselineAutopilot, Doctrine};
use hyades_engine::prelude::*;

const SEEDS: &[u64] = &[1, 7, 42, 31337];
const PLAYERS: usize = 3;

/// Significant elasticities from `gradient_probe --screen` (points per ln of
/// the `colonies@2000` proxy). Only knobs outside 2 SE appear.
///
/// **⚠ These are CONSUMED — the step they pointed at has been taken and
/// ratified** (`growth_rate` 0.546 → 0.873). They are kept as the record of
/// that step, not as a direction to walk again. A gradient is local; reusing
/// one after the step that spent it is the "compare gradients across operating
/// points" error that produced this project's fifth artifact (R-AC18,
/// withdrawn) — and it had already claimed the *previous* set of constants in
/// this same file (`medium_fleet_size` 32.73, `biosphere` 19.70,
/// `outpost_mining` 14.48, `growth` 7.30, all measured before the step that
/// consumed them).
///
/// Twice is a pattern, so [`MEASURED_AT_GROWTH_RATE`] now makes the harness
/// refuse to run rather than trusting anyone to read this paragraph.
const G_GROWTH: f64 = 2.27;
const G_BIOSPHERE: f64 = 0.84;

/// The operating point the constants above were measured at.
///
/// **This is the guard.** Editing a default without re-running
/// `gradient_probe --screen` silently invalidates the direction, and the
/// failure is invisible: the line search still runs, still prints a tidy table,
/// and still nominates a winner — it is just walking a direction that belongs
/// to a point the engine has left. Comparing this against the live default
/// turns that into a refusal at startup.
const MEASURED_AT_GROWTH_RATE: f64 = 0.546;

/// **Deliberately excluded, with reasons** — stepping along noise, or along a
/// direction the objective itself cannot confirm, is how a search wanders:
///
/// - `medium_fleet_size`: the screen says `-0.58 ± 0.19` but the full
///   objective says `+1.93 ± 2.37` — it cannot even resolve the sign. Read
///   together with the ±25% levels (3.34 → 48.6%, **4.45 → 49.8%**, 5.56 →
///   42.2%, seed 1), all three measurements agree on one thing: **4.45 is on
///   the peak, where the gradient vanishes.** A ratified value that measures
///   flat is doing its job. Left alone.
/// - `center_mining_fraction` (+0.86 ± 0.48) and `outpost_mining_fraction`
///   (+0.72 ± 0.37): both inside 2 SE. Candidates for the ten-seed bed
///   (`hyades_todo.md` T-44), not for a step.
/// - `cargo_unit_size`, `trade_decay_lambda`, `survey_reserve`,
///   `rank.centrality_scale`: flat. The first two reading flat is a *proxy
///   validation* — `cargo_unit_size` is known-saturated and
///   `trade_decay_lambda` is known to sit at an interior optimum, so flat is
///   exactly what both should read.
const EXCLUDED: &[&str] = &[
    "medium_fleet_size (at peak)",
    "center/outpost_mining (noise)",
    "cargo_unit_size, lambda, reserve, centrality (flat)",
];

fn profile(cfg: SimConfig, doctrine: Doctrine) -> Vec<f64> {
    SEEDS
        .iter()
        .map(|&seed| {
            let galaxy = Galaxy::generate(GalaxyConfig::new(PLAYERS, seed)).unwrap();
            let targets: HashSet<PlanetId> =
                galaxy.planets.iter().filter(|p| p.habitability.min(p.biosphere) > 0.01).map(|p| p.id).collect();
            let total = targets.len().max(1);
            let autopilots: Vec<Box<dyn Autopilot>> =
                (0..PLAYERS).map(|_| Box::new(BaselineAutopilot::new(doctrine)) as Box<_>).collect();
            let mut cfg = cfg;
            cfg.seed = seed;
            let mut sim = Simulation::new(galaxy, cfg, autopilots);
            sim.run();
            let snap = sim.snapshot();
            snap.planets.iter().filter(|p| p.owner.is_some() && targets.contains(&p.id)).count() as f64 / total as f64
        })
        .collect()
}

fn mean(v: &[f64]) -> f64 {
    v.iter().sum::<f64>() / v.len() as f64
}

fn stderr(v: &[f64]) -> f64 {
    let n = v.len() as f64;
    let m = mean(v);
    (v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (n - 1.0) / n).sqrt()
}

/// Configuration `alpha` steps along the normalised gradient, in log space.
///
/// The direction is now only two-dimensional, and `growth_rate` is **94% of
/// it** (`2.27 / √(2.27² + 0.84²)`). That matters because
/// `biosphere_regen_rate` is not a free economic knob: it is the dial that
/// decides whether biological warfare is a real strategy or a rounding error
/// (R-O63, standing-layer §9.1). If design pins it, stepping `growth_rate`
/// alone still captures almost all of the available gain.
fn at(alpha: f64) -> (SimConfig, Doctrine) {
    let norm = (G_BIOSPHERE.powi(2) + G_GROWTH.powi(2)).sqrt();
    let step = |g: f64| (alpha * g / norm).exp();
    let mut cfg = SimConfig::new(0);
    let mut doc = Doctrine::default();
    cfg.biosphere_regen_rate *= step(G_BIOSPHERE);
    doc.growth_rate *= step(G_GROWTH);
    (cfg, doc)
}

/// **Attribution at the chosen step** (`--attribution`): which component of a
/// multi-knob step actually earned the gain?
///
/// This is not academic here. `biosphere_regen_rate` is a *design* dial, not a
/// free economic one — it sets whether a razed ecology is a durable wound or a
/// rounding error (R-O63, standing-layer §9.1). If `growth_rate` earns the
/// whole gain by itself, the right default change is the one that leaves the
/// design surface untouched. Same CRN seeds, full objective, paired.
fn attribution(alpha: f64) {
    let (stepped_cfg, stepped_doc) = at(alpha);
    let (base_cfg, base_doc) = at(0.0);

    println!("\n=== Attribution at alpha = {alpha:.2} (full objective, paired on the same seeds) ===");
    let base = profile(base_cfg, base_doc);
    println!("  baseline                     {:.2}%", mean(&base) * 100.0);
    std::io::stdout().flush().ok();

    let cases: [(&str, SimConfig, Doctrine); 3] = [
        ("growth_rate only", base_cfg, stepped_doc),
        ("biosphere_regen only", stepped_cfg, base_doc),
        ("both (the step)", stepped_cfg, stepped_doc),
    ];
    for (label, cfg, doc) in cases {
        let p = profile(cfg, doc);
        let gains: Vec<f64> = p.iter().zip(base.iter()).map(|(a, b)| a - b).collect();
        println!(
            "  {label:<28} {:.2}%   paired {:>+6.2} ± {:.2}",
            mean(&p) * 100.0,
            mean(&gains) * 100.0,
            stderr(&gains) * 100.0
        );
        std::io::stdout().flush().ok();
    }
    println!(
        "\nIf `growth_rate only` matches `both` within noise, ratify growth_rate alone and\n\
         leave biosphere_regen_rate at its R-O63 placeholder for the design owner."
    );
}

/// Refuse to walk a direction that belongs to an operating point the engine has
/// left. Returns `true` if the constants still match the live defaults.
fn direction_is_still_live() -> bool {
    let live = Doctrine::default().growth_rate;
    if (live - MEASURED_AT_GROWTH_RATE).abs() < 1e-9 {
        return true;
    }
    eprintln!(
        "REFUSING TO RUN — the measured direction has been consumed.\n\n\
         The elasticities in this file were measured at growth_rate = {MEASURED_AT_GROWTH_RATE},\n\
         but the shipped default is now {live}. That step was taken and ratified, so this\n\
         direction belongs to a point the engine has left. Walking it anyway is the\n\
         'compare gradients across operating points' error (R-AC18, withdrawn) — and it\n\
         would not look like an error: the line search would print a tidy table and\n\
         nominate a winner regardless.\n\n\
         Fix: re-run `cargo run --release --example gradient_probe -- --screen` (~68 s),\n\
         then update G_* and MEASURED_AT_GROWTH_RATE from its output.\n\n\
         To inspect the consumed step anyway, pass --force."
    );
    false
}

fn main() {
    let forced = std::env::args().any(|a| a == "--force");
    if !forced && !direction_is_still_live() {
        std::process::exit(1);
    }
    if let Some(pos) = std::env::args().position(|a| a == "--attribution") {
        let alpha = std::env::args().nth(pos + 1).and_then(|s| s.parse().ok()).unwrap_or(0.5);
        attribution(alpha);
        return;
    }
    println!("Gradient step — line search along the normalised gradient, CRN bed of {} seeds.", SEEDS.len());
    println!("Direction re-measured at the *current* defaults; only knobs outside 2 SE move.");
    for why in EXCLUDED {
        println!("  excluded: {why}");
    }
    println!("Verified on the FULL objective (coverage at 4000 yr) — the screen picks the\ndirection, the objective ratifies the step.\n");

    let (base_cfg, base_doc) = at(0.0);
    let base = profile(base_cfg, base_doc);
    println!("alpha=0.00 (baseline)  {:.2}% ± {:.2}\n", mean(&base) * 100.0, stderr(&base) * 100.0);
    std::io::stdout().flush().ok();

    println!("{:>6} {:>9} {:>9}   {:>8} {:>18}", "alpha", "bio_regen", "growth", "mean", "paired gain");
    let mut best = (0.0f64, mean(&base));
    for &alpha in &[0.25, 0.5, 1.0, 1.5, 2.0] {
        let (cfg, doc) = at(alpha);
        if let Some(why) = cfg.hull_ladder_fault() {
            println!("{alpha:>6.2}  stopped — {why}");
            break;
        }
        let p = profile(cfg, doc);
        // Paired, seed by seed: the only comparison that is not mostly seed.
        let gains: Vec<f64> = p.iter().zip(base.iter()).map(|(a, b)| a - b).collect();
        println!(
            "{alpha:>6.2} {:>9.4} {:>9.3}   {:>7.2}% {:>+11.2} ± {:.2}",
            cfg.biosphere_regen_rate,
            doc.growth_rate,
            mean(&p) * 100.0,
            mean(&gains) * 100.0,
            stderr(&gains) * 100.0
        );
        std::io::stdout().flush().ok();
        if mean(&p) > best.1 {
            best = (alpha, mean(&p));
        }
    }

    println!("\nBest verified step: alpha = {:.2} at {:.2}% coverage.", best.0, best.1 * 100.0);
    println!(
        "Ratify only this — the paired gain is the evidence, and a step that does not\n\
         beat the baseline on the same seeds is not an improvement no matter what the\n\
         gradient predicted."
    );
}
