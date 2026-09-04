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

use std::io::Write;

use hyades_engine::autopilot::{Autopilot, BaselineAutopilot, Doctrine};
use hyades_engine::prelude::*;

const SEEDS: &[u64] = &[1, 7, 42, 31337];
const PLAYERS: usize = 3;

/// Significant elasticities from `gradient_probe --screen`, in **colonies** per
/// ln of the `colonies@2000` proxy. Only knobs outside 2 SE appear.
///
/// **Re-measured against the corrected objective** (absolute colony count, not
/// a fraction — see `gradient_probe::trial`). That correction was not cosmetic:
/// it **reordered the top of the ranking**, because averaging fractions across
/// seeds weights each seed by `1/denominator` and the denominators differ.
///
/// | knob | under the fraction | under colony count |
/// |---|---|---|
/// | `biosphere_regen_rate` | +0.84 ± 0.24 — *third* | **+141.2 ± 18.1 — first, 7.8 SE** |
/// | `growth_rate` | +2.27 ± 0.50 — first | +73.8 ± 20.3 — second |
/// | `survey_reserve` | −0.13 ± 0.23 — **flat** | **−23.8 ± 10.1 — significant** |
///
/// A knob the old metric called inert is a real lever, and the lever the old
/// metric ranked first is second. Two prior sets of constants in this file went
/// stale by being *consumed*; this set was stale because the **objective** was
/// wrong, which is a failure mode the [`MEASURED_AT_GROWTH_RATE`] guard cannot
/// catch — it watches the operating point, not the metric.
const G_GROWTH: f64 = 73.8;
const G_BIOSPHERE: f64 = 141.2;

/// The operating point the constants above were measured at.
///
/// **This is the guard.** Editing a default without re-running
/// `gradient_probe --screen` silently invalidates the direction, and the
/// failure is invisible: the line search still runs, still prints a tidy table,
/// and still nominates a winner — it is just walking a direction that belongs
/// to a point the engine has left. Comparing this against the live default
/// turns that into a refusal at startup.
const MEASURED_AT_GROWTH_RATE: f64 = 0.873;

/// **Deliberately excluded, with reasons** — stepping along noise is how a
/// search wanders. Verdicts re-derived against the corrected objective; two
/// of them changed, which is the point:
///
/// - `medium_fleet_size` (−62.5 ± 26.3, 2.4 SE): now *marginally* significant
///   where the fraction metric and the full objective disagreed. Three prior
///   measurements put 4.45 on its peak (±25% levels: 3.34 → 48.6%, **4.45 →
///   49.8%**, 5.56 → 42.2%, seed 1), and a 2.4-SE reading against that much
///   contrary evidence is not enough to move an MC-ratified value. **Re-check
///   it on the full objective before touching it** — it is the strongest
///   remaining candidate, not a settled exclusion.
/// - `survey_reserve` (−23.8 ± 10.1, 2.4 SE): **was flat under the fraction,
///   is a real lever under colony count.** Excluded from *this* step only
///   because it is an integer knob whose gradient the log-space line search
///   handles badly, not because it is inert. Owed its own sweep.
/// - `rank.centrality_scale` (+31.2 ± 26.7), `outpost_mining_fraction`
///   (+27.5 ± 18.9), `center_mining_fraction` (+11.2 ± 5.9): inside 2 SE.
///   Candidates for the wide bed (`SEEDS_WIDE`, `hyades_todo.md` T-44).
/// - `cargo_unit_size` (−1.2 ± 35.3) and `trade_decay_lambda` (+3.8 ± 22.3):
///   flat, and *correctly* so — the first is known-saturated and the second
///   sits at a ratified interior optimum, so flat is what both should read.
///   That they still read flat after the metric changed is a small validation
///   of the new objective.
const EXCLUDED: &[&str] = &[
    "medium_fleet_size (2.4 SE, but three prior measurements put it on the peak — re-check first)",
    "survey_reserve (2.4 SE and newly significant, but integer — owed its own sweep)",
    "centrality_scale, both mining fractions (inside 2 SE)",
    "cargo_unit_size, trade_decay_lambda (flat, as expected)",
];
/// Colonies held at the horizon — an **absolute count**, matching
/// `gradient_probe::trial`. See that function for why this is not a fraction:
/// a habitability-derived denominator would let a terraforming or bombardment
/// card move the score by changing what is counted rather than what is done.
fn profile(cfg: SimConfig, doctrine: Doctrine) -> Vec<f64> {
    SEEDS
        .iter()
        .map(|&seed| {
            let galaxy = Galaxy::generate(GalaxyConfig::new(PLAYERS, seed)).unwrap();
            let autopilots: Vec<Box<dyn Autopilot>> =
                (0..PLAYERS).map(|_| Box::new(BaselineAutopilot::new(doctrine)) as Box<_>).collect();
            let mut cfg = cfg;
            cfg.seed = seed;
            let mut sim = Simulation::new(galaxy, cfg, autopilots);
            sim.run();
            sim.snapshot().planets.iter().filter(|p| p.owner.is_some()).count() as f64
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
/// The direction is two-dimensional, and under the corrected objective
/// `biosphere_regen_rate` is now **86% of it** (`141.2 / √(141.2² + 73.8²)`) —
/// the reverse of the fraction-metric reading. That matters because
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
    println!("  baseline                     {:.0} colonies", mean(&base));
    std::io::stdout().flush().ok();

    let cases: [(&str, SimConfig, Doctrine); 3] = [
        ("growth_rate only", base_cfg, stepped_doc),
        ("biosphere_regen only", stepped_cfg, base_doc),
        ("both (the step)", stepped_cfg, stepped_doc),
    ];
    for (label, cfg, doc) in cases {
        let p = profile(cfg, doc);
        let gains: Vec<f64> = p.iter().zip(base.iter()).map(|(a, b)| a - b).collect();
        println!("  {label:<28} {:>8.0}   paired {:>+7.1} ± {:.1}", mean(&p), mean(&gains), stderr(&gains));
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
         'compare gradients across operating points' error (R-AC20, withdrawn) — and it\n\
         would not look like an error: the line search would print a tidy table and\n\
         nominate a winner regardless.\n\n\
         Fix: re-run `cargo run --release --example gradient_probe -- --screen` (~68 s),\n\
         then update G_* and MEASURED_AT_GROWTH_RATE from its output.\n\n\
         To inspect the consumed step anyway, pass --force."
    );
    false
}

/// **`survey_reserve` sweep** (`--sweep-reserve`). The corrected objective
/// promoted this knob from "flat" to a real lever (−23.8 ± 10.1), and unlike
/// the two in the gradient direction it carries **no design cost** — it is a
/// pure search-behaviour tunable, already documented as monotone by
/// construction (survey is a fallback, never a pre-emption). It is excluded
/// from the log-space line search only because it is an integer.
///
/// Negative elasticity means *lower* is better, so the sweep goes down.
fn sweep_reserve() {
    let cfg = SimConfig::new(0);
    let base_doc = Doctrine::default();
    println!("=== survey_reserve sweep (full objective, paired on the same seeds) ===");
    let base = profile(cfg, base_doc);
    println!("  {:>6} (shipped)  {:>8.0} colonies", base_doc.survey_reserve, mean(&base));
    std::io::stdout().flush().ok();
    for reserve in [64usize, 128, 256, 512, 2048] {
        let doc = Doctrine { survey_reserve: reserve, ..base_doc };
        let p = profile(cfg, doc);
        let gains: Vec<f64> = p.iter().zip(base.iter()).map(|(a, b)| a - b).collect();
        let (g, se) = (mean(&gains), stderr(&gains));
        let verdict = if g.abs() < 2.0 * se {
            "~noise"
        } else if g > 0.0 {
            "BETTER"
        } else {
            "worse"
        };
        println!("  {reserve:>6}            {:>8.0} colonies   paired {g:>+7.1} ± {se:.1}  [{verdict}]", mean(&p));
        std::io::stdout().flush().ok();
    }
}

fn main() {
    if std::env::args().any(|a| a == "--sweep-reserve") {
        sweep_reserve();
        return;
    }
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
    println!("alpha=0.00 (baseline)  {:.0} ± {:.0} colonies\n", mean(&base), stderr(&base));
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
            "{alpha:>6.2} {:>9.4} {:>9.3}   {:>8.0} {:>+11.1} ± {:.1}",
            cfg.biosphere_regen_rate,
            doc.growth_rate,
            mean(&p),
            mean(&gains),
            stderr(&gains)
        );
        std::io::stdout().flush().ok();
        if mean(&p) > best.1 {
            best = (alpha, mean(&p));
        }
    }

    println!("\nBest verified step: alpha = {:.2} at {:.0} colonies.", best.0, best.1);
    println!(
        "Ratify only this — the paired gain is the evidence, and a step that does not\n\
         beat the baseline on the same seeds is not an improvement no matter what the\n\
         gradient predicted."
    );
}
