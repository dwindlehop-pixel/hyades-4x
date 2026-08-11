//! **Round 2 of the R-AC3 investigation** (`examples/survey_strategy_search.rs`
//! Round 1, `examples/colonization_ramp_trace.rs` for the diagnosis).
//!
//! Round 1 found survey strategy has no measurable effect on years-to-10%-
//! colonized. The trace explains why: a homeworld crosses the medium-hull
//! gate almost immediately (t≈200 yr, seed 1), and known candidates
//! (98 by t=200, 151 by t=400) always vastly outnumber what the treasury
//! can afford that cycle. Survey was never the binding constraint —
//! the ramp is a genuine compounding curve (3→7→14→...→3348 colonies over
//! 4,000 yr, seed 1) gated by **production capacity**: how fast minerals
//! come in and how many centers can spend them in parallel.
//!
//! This is the same gradient methodology as `examples/gradient_probe.rs`
//! (common random numbers, paired central differences, elasticity + standard
//! error — see CLAUDE.md §2 "How to search"), retargeted at **years to 10%
//! colonized** instead of coverage-at-horizon. Lower is better here, the
//! opposite sign convention from the coverage objective, so elasticity signs
//! read as "raise the knob, get there this many years sooner/later per ln."
//!
//! Knobs chosen from the diagnosis, not swept blind: `reinvest_bias` (the
//! expand-vs-deepen dial), `growth_rate` (how fast a center's own pop climbs
//! toward whatever `K` is currently affordable), `rank.k_high` (mining vs.
//! colony/center classification — shifts how much of the early economy is
//! mining income vs. deepening), `center_mining_fraction` /
//! `outpost_mining_fraction` (the income rate itself), and
//! `medium_fleet_size` (a Colonizer's cost, hence how many cycles of income
//! it takes to afford the next one).
//!
//! Run: `cargo run --release --example time_to_10pct_probe`

use std::collections::HashSet;
use std::io::Write;

use hyades_engine::autopilot::{Autopilot, BaselineAutopilot, Doctrine};
use hyades_engine::log::{LogCategory, LogEvent, LogFilter};
use hyades_engine::prelude::*;

const SEEDS: &[u64] = &[1, 7, 42, 31337];
const PLAYERS: usize = 3;
const DELTA: f64 = 0.10;
const TARGET_FRACTION: f64 = 0.10;

fn coverage_targets(galaxy: &Galaxy) -> HashSet<PlanetId> {
    galaxy.planets.iter().filter(|p| p.habitability.min(p.biosphere) > 0.01).map(|p| p.id).collect()
}

/// Years until `TARGET_FRACTION` of colonizable worlds are colonized, or
/// `horizon_years` (right-censored) if the horizon runs out first.
fn time_to_target(seed: u64, cfg: SimConfig, doctrine: Doctrine) -> f64 {
    let galaxy = Galaxy::generate(GalaxyConfig::new(PLAYERS, seed)).unwrap();
    let targets = coverage_targets(&galaxy);
    let need = ((targets.len() as f64) * TARGET_FRACTION).ceil() as usize;

    let autopilots: Vec<Box<dyn Autopilot>> =
        (0..PLAYERS).map(|_| Box::new(BaselineAutopilot::new(doctrine)) as Box<_>).collect();
    let mut sim = Simulation::new(galaxy, cfg, autopilots);
    sim.set_log_filter(LogFilter::none().with(LogCategory::Vehicles));
    sim.run();

    let mut times: Vec<f64> = sim
        .log()
        .by_category(LogCategory::Vehicles)
        .filter_map(|r| match r.event {
            LogEvent::ColonyFounded { planet, .. } if targets.contains(&planet) => Some(r.time),
            _ => None,
        })
        .collect();
    // Homeworlds count as founded at t=0 and are always targets (habitable by construction).
    let founded_before_horizon = times.len() + PLAYERS;
    if founded_before_horizon < need {
        return cfg.horizon_years;
    }
    times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    times[need - PLAYERS - 1]
}

fn profile(cfg: SimConfig, doctrine: Doctrine) -> Vec<f64> {
    SEEDS.iter().map(|&s| time_to_target(s, cfg, doctrine)).collect()
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

struct Knob {
    name: &'static str,
    value: f64,
    set: fn(&mut SimConfig, &mut Doctrine, f64),
}

struct Finding {
    name: &'static str,
    value: f64,
    elasticity: f64,
    se: f64,
}

fn main() {
    let base_cfg = SimConfig::new(0);
    let base_doc = Doctrine::default();

    let knobs: Vec<Knob> = vec![
        Knob { name: "reinvest_bias", value: base_doc.reinvest_bias, set: |_, d, v| d.reinvest_bias = v },
        Knob { name: "growth_rate", value: base_doc.growth_rate, set: |_, d, v| d.growth_rate = v },
        Knob { name: "rank.k_high", value: base_doc.rank.k_high, set: |_, d, v| d.rank.k_high = v },
        Knob {
            name: "center_mining_fraction",
            value: base_cfg.center_mining_fraction,
            set: |c, _, v| c.center_mining_fraction = v,
        },
        Knob {
            name: "outpost_mining_fraction",
            value: base_cfg.outpost_mining_fraction,
            set: |c, _, v| c.outpost_mining_fraction = v,
        },
        Knob { name: "medium_fleet_size", value: base_cfg.medium_fleet_size, set: |c, _, v| c.medium_fleet_size = v },
    ];

    let evals = 2 * knobs.len() * SEEDS.len();
    println!(
        "Time-to-{:.0}%-colonized probe — {PLAYERS} seats, {} seeds (CRN), delta = ±{:.0}%",
        TARGET_FRACTION * 100.0,
        SEEDS.len(),
        DELTA * 100.0
    );
    println!("Paired central differences. Budget: {evals} evaluations.\n");

    let here = profile(base_cfg, base_doc);
    println!(
        "Operating point: {:.1} ± {:.1} yr   per-seed {:?}\n",
        mean(&here),
        stderr(&here),
        here.iter().map(|x| format!("{x:.0}")).collect::<Vec<_>>()
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

        let diffs: Vec<f64> = hi.iter().zip(lo.iter()).map(|(a, b)| a - b).collect();
        // d(years)/d(ln x) — negative means raising the knob gets to 10% *sooner*.
        let elasticity = mean(&diffs) / (2.0 * DELTA);
        let se = stderr(&diffs) / (2.0 * DELTA);

        println!("  {:<24} value {:>9.4}   elasticity {:>+9.1} ± {:.1} yr/ln", k.name, k.value, elasticity, se);
        std::io::stdout().flush().ok();
        findings.push(Finding { name: k.name, value: k.value, elasticity, se });
    }

    findings.sort_by(|a, b| b.elasticity.abs().partial_cmp(&a.elasticity.abs()).unwrap());

    println!("\n=== Ranked by |elasticity| — where the leverage on early speed actually is ===");
    println!("{:<24} {:>10} {:>10} {:>8}  verdict", "knob", "value", "yr/ln", "SE");
    for f in &findings {
        let verdict = if f.se == 0.0 && f.elasticity == 0.0 {
            "INERT — moved no seed at all"
        } else if f.elasticity.abs() < 2.0 * f.se {
            "~noise — needs a bigger bed"
        } else if f.elasticity < 0.0 {
            "raise it — gets to 10% sooner"
        } else {
            "lower it — gets to 10% sooner"
        };
        println!("{:<24} {:>10.4} {:>+10.1} {:>8.1}  {verdict}", f.name, f.value, f.elasticity, f.se);
    }

    println!(
        "\nReading: negative elasticity means raising the knob *shortens* time-to-{:.0}%.\n\
         This is the opposite sign convention from gradient_probe's coverage objective —\n\
         there, positive is good; here, negative is good. Anything within 2 SE of zero is\n\
         not a finding.",
        TARGET_FRACTION * 100.0
    );
}
