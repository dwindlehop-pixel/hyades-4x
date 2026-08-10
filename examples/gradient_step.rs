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

/// Significant elasticities from `gradient_probe` (outside 2 SE), points per ln.
const G_MEDIUM_FLEET: f64 = 32.73;
const G_BIOSPHERE: f64 = 19.70;
const G_OUTPOST_MINING: f64 = 14.48;
const G_GROWTH: f64 = 7.30;

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
fn at(alpha: f64) -> (SimConfig, Doctrine) {
    let norm = (G_MEDIUM_FLEET.powi(2) + G_BIOSPHERE.powi(2) + G_OUTPOST_MINING.powi(2) + G_GROWTH.powi(2)).sqrt();
    let step = |g: f64| (alpha * g / norm).exp();
    let mut cfg = SimConfig::new(0);
    let mut doc = Doctrine::default();
    cfg.medium_fleet_size *= step(G_MEDIUM_FLEET);
    cfg.biosphere_regen_rate *= step(G_BIOSPHERE);
    // A fraction of local density; clamping is a constraint, not a preference.
    cfg.outpost_mining_fraction = (cfg.outpost_mining_fraction * step(G_OUTPOST_MINING)).min(0.95);
    doc.growth_rate *= step(G_GROWTH);
    (cfg, doc)
}

fn main() {
    println!("Gradient step — line search along the normalised gradient, CRN bed of {} seeds.", SEEDS.len());
    println!("Only the four knobs that cleared 2 SE are moved.\n");

    let (base_cfg, base_doc) = at(0.0);
    let base = profile(base_cfg, base_doc);
    println!("alpha=0.00 (baseline)  {:.2}% ± {:.2}\n", mean(&base) * 100.0, stderr(&base) * 100.0);
    std::io::stdout().flush().ok();

    println!(
        "{:>6} {:>7} {:>7} {:>7} {:>7}   {:>8} {:>18}",
        "alpha", "mfs", "bio", "outp", "growth", "mean", "paired gain"
    );
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
            "{alpha:>6.2} {:>7.3} {:>7.4} {:>7.3} {:>7.3}   {:>7.2}% {:>+11.2} ± {:.2}",
            cfg.medium_fleet_size,
            cfg.biosphere_regen_rate,
            cfg.outpost_mining_fraction,
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
