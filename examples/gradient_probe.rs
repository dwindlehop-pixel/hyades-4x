//! **How to tune, rather than a tuning.**
//!
//! Coordinate descent answers "which of these five values is best" and tells
//! you nothing else. It costs `values × params × seeds` evaluations, produces
//! no gradient, no error bars, and no way to compare one knob against another.
//! Three of its findings in this project turned out to be artifacts that error
//! bars would have caught immediately.
//!
//! This probe answers a different and more useful question — **which knobs move
//! the objective, in what direction, by how much, and is the effect bigger than
//! the noise** — for `2 × params × seeds` evaluations, roughly a third the cost.
//!
//! ## The four techniques, and why each one is here
//!
//! **1. Common random numbers (CRN).** Every configuration is evaluated on the
//! *same* seeds. The objective is a noisy function of the seed, and that noise
//! is enormous relative to the effects we are chasing — seed 42 runs ~40% below
//! seed 1. Comparing `mean(A)` against `mean(B)` from different seeds measures
//! mostly seed. Comparing them seed-by-seed cancels it. This is the single
//! highest-leverage thing in the file and it costs nothing.
//!
//! **2. Paired central differences.** `f(x(1+δ)) − f(x(1−δ))`, evaluated on
//! matched seeds. Central rather than forward because the error is `O(δ²)`
//! instead of `O(δ)` for the same two evaluations. Paired because under CRN the
//! *difference* has far lower variance than either level — the quantity we want
//! is exactly the quantity that is cheap to estimate.
//!
//! **3. Elasticity, not slope.** `∂f/∂ln x = x · ∂f/∂x`. Raw slopes are not
//! comparable across parameters measured in different units — a slope per
//! `cargo_unit_size` and a slope per `growth_rate` are different currencies.
//! The log-derivative is unit-free: *"a 1% change in this knob moves coverage
//! by this many points."* That is what makes a **ranking** meaningful, and the
//! ranking is the transferable knowledge.
//!
//! **4. A standard error on every number.** Per-seed differences give a paired
//! standard error directly. Anything inside `2·SE` of zero is not a finding, and
//! saying so is the whole difference between a measurement and a guess.
//!
//! ## Reading the output
//!
//! Knobs are ranked by `|elasticity|`. The top of the table is where tuning
//! effort belongs; anything flagged `~noise` should be left alone until the bed
//! is bigger, and anything flagged `flat` is genuinely inert at this operating
//! point and is a candidate for deletion rather than tuning.
//!
//! **A caution the method cannot remove:** a gradient is local. It says which
//! way is uphill *here*, not where the summit is, and it will happily point
//! along a direction that is an artifact if the model underneath is wrong. Use
//! it to decide what to investigate, not to decide what is true.
//!
//! Run: `cargo run --release --example gradient_probe`

use std::collections::HashSet;
use std::io::Write;

use hyades_engine::autopilot::{Autopilot, BaselineAutopilot, Doctrine};
use hyades_engine::prelude::*;

/// Common random numbers: the *same* seeds for every configuration.
const SEEDS: &[u64] = &[1, 7, 42, 31337];
const PLAYERS: usize = 3;
/// Relative step for the central difference. Large enough to clear simulation
/// granularity (colony counts are integers), small enough that the `O(δ²)`
/// truncation error stays below the sampling error.
const DELTA: f64 = 0.10;

fn coverage_targets(galaxy: &Galaxy) -> HashSet<PlanetId> {
    galaxy.planets.iter().filter(|p| p.habitability.min(p.biosphere) > 0.01).map(|p| p.id).collect()
}

/// Coverage fraction for one seed under one configuration.
fn trial(seed: u64, cfg: SimConfig, doctrine: Doctrine) -> f64 {
    let galaxy = Galaxy::generate(GalaxyConfig::new(PLAYERS, seed)).unwrap();
    let targets = coverage_targets(&galaxy);
    let total = targets.len().max(1);
    let autopilots: Vec<Box<dyn Autopilot>> =
        (0..PLAYERS).map(|_| Box::new(BaselineAutopilot::new(doctrine)) as Box<_>).collect();
    let mut sim = Simulation::new(galaxy, cfg, autopilots);
    sim.run();
    let snap = sim.snapshot();
    snap.planets.iter().filter(|p| p.owner.is_some() && targets.contains(&p.id)).count() as f64 / total as f64
}

/// Per-seed coverage vector — the CRN unit. Never collapse to a mean before
/// differencing; the whole variance reduction lives in keeping seeds aligned.
fn profile(cfg: SimConfig, doctrine: Doctrine) -> Vec<f64> {
    SEEDS.iter().map(|&s| trial(s, cfg, doctrine)).collect()
}

fn mean(v: &[f64]) -> f64 {
    v.iter().sum::<f64>() / v.len() as f64
}

/// Sample standard error of the mean. `n-1` in the denominator: with four seeds
/// the difference between `n` and `n-1` is 15% of the reported uncertainty.
fn stderr(v: &[f64]) -> f64 {
    let n = v.len() as f64;
    if n < 2.0 {
        return f64::INFINITY;
    }
    let m = mean(v);
    (v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (n - 1.0) / n).sqrt()
}

struct Knob {
    name: &'static str,
    /// Current value at the operating point.
    value: f64,
    /// Apply a value to a config/doctrine pair.
    set: fn(&mut SimConfig, &mut Doctrine, f64),
}

struct Finding {
    name: &'static str,
    value: f64,
    /// `∂coverage/∂ln x`, in percentage points.
    elasticity: f64,
    se: f64,
}

fn main() {
    let base_cfg = SimConfig::new(0);
    let base_doc = Doctrine::default();

    let knobs: Vec<Knob> = vec![
        Knob { name: "medium_fleet_size", value: base_cfg.medium_fleet_size, set: |c, _, v| c.medium_fleet_size = v },
        Knob { name: "cargo_unit_size", value: base_cfg.cargo_unit_size, set: |c, _, v| c.cargo_unit_size = v },
        Knob {
            name: "outpost_mining_fraction",
            value: base_cfg.outpost_mining_fraction,
            set: |c, _, v| c.outpost_mining_fraction = v,
        },
        Knob {
            name: "center_mining_fraction",
            value: base_cfg.center_mining_fraction,
            set: |c, _, v| c.center_mining_fraction = v,
        },
        Knob {
            name: "trade_decay_lambda",
            value: base_cfg.trade_decay_lambda,
            set: |c, _, v| c.trade_decay_lambda = v,
        },
        Knob {
            name: "biosphere_regen_rate",
            value: base_cfg.biosphere_regen_rate,
            set: |c, _, v| c.biosphere_regen_rate = v,
        },
        Knob { name: "growth_rate", value: base_doc.growth_rate, set: |_, d, v| d.growth_rate = v },
        Knob {
            name: "rank.centrality_scale",
            value: base_doc.rank.centrality_scale,
            set: |_, d, v| d.rank.centrality_scale = v,
        },
        Knob {
            name: "survey_reserve",
            value: base_doc.survey_reserve as f64,
            set: |_, d, v| d.survey_reserve = v as usize,
        },
    ];

    let evals = 2 * knobs.len() * SEEDS.len();
    println!(
        "Gradient probe — {PLAYERS} seats, {} seeds (common random numbers), delta = ±{:.0}%",
        SEEDS.len(),
        DELTA * 100.0
    );
    println!("Paired central differences. Budget: {evals} evaluations.");
    println!(
        "(Coordinate descent over 5 values would be {} for strictly less information.)\n",
        5 * knobs.len() * SEEDS.len()
    );

    let here = profile(base_cfg, base_doc);
    println!(
        "Operating point: {:.2}% ± {:.2} coverage   per-seed {:?}\n",
        mean(&here) * 100.0,
        stderr(&here) * 100.0,
        here.iter().map(|x| format!("{:.1}%", x * 100.0)).collect::<Vec<_>>()
    );
    std::io::stdout().flush().ok();

    let mut findings = Vec::new();
    for k in &knobs {
        let mut hi_cfg = base_cfg;
        let mut hi_doc = base_doc;
        (k.set)(&mut hi_cfg, &mut hi_doc, k.value * (1.0 + DELTA));
        let mut lo_cfg = base_cfg;
        let mut lo_doc = base_doc;
        (k.set)(&mut lo_cfg, &mut lo_doc, k.value * (1.0 - DELTA));

        if hi_cfg.hull_ladder_fault().is_some() || lo_cfg.hull_ladder_fault().is_some() {
            println!("  {:<24} skipped — step leaves the legal hull ladder", k.name);
            continue;
        }

        let hi = profile(hi_cfg, hi_doc);
        let lo = profile(lo_cfg, lo_doc);

        // The paired difference, seed by seed. This is where CRN pays.
        let diffs: Vec<f64> = hi.iter().zip(lo.iter()).map(|(a, b)| a - b).collect();
        // d(coverage)/d(ln x) = delta_f / (2*delta), since x(1±δ) is ±δ in log space to O(δ²).
        let elasticity = mean(&diffs) / (2.0 * DELTA) * 100.0;
        let se = stderr(&diffs) / (2.0 * DELTA) * 100.0;

        println!("  {:<24} value {:>9.4}   elasticity {:>+8.2} ± {:.2} pts/ln", k.name, k.value, elasticity, se);
        std::io::stdout().flush().ok();
        findings.push(Finding { name: k.name, value: k.value, elasticity, se });
    }

    findings.sort_by(|a, b| b.elasticity.abs().partial_cmp(&a.elasticity.abs()).unwrap());

    println!("\n=== Ranked by |elasticity| — where tuning effort belongs ===");
    println!("{:<24} {:>10} {:>10} {:>8}  verdict", "knob", "value", "d/dln x", "SE");
    for f in &findings {
        // Exactly-zero-with-zero-variance is the *most* informative outcome and
        // has to be caught before the significance test: `|e| < 2·se` is false
        // when both are zero, so it would otherwise fall through to a direction
        // and read as a weak recommendation instead of a structural finding.
        let verdict = if f.se == 0.0 && f.elasticity == 0.0 {
            "INERT — moved no seed at all; the knob does not reach the objective"
        } else if f.elasticity.abs() < 2.0 * f.se {
            if f.elasticity.abs() < 0.5 {
                "flat — inert here, consider deleting"
            } else {
                "~noise — needs a bigger bed"
            }
        } else if f.elasticity > 0.0 {
            "raise it"
        } else {
            "lower it"
        };
        println!("{:<24} {:>10.4} {:>+10.2} {:>8.2}  {verdict}", f.name, f.value, f.elasticity, f.se);
    }

    println!(
        "\nReading: elasticity is coverage percentage points per unit change in ln(knob),\n\
         so a value of +5 means a 10% increase in that knob buys ~0.5 points. Signs say\n\
         which way is uphill *from here* — a gradient is local and cannot see a summit,\n\
         a cliff, or a modelling artifact pointing the wrong way.\n\
         Anything within 2 SE of zero is not a result. Widen the seed bed before\n\
         believing it, or accept that the knob does not matter at this operating point."
    );
}
