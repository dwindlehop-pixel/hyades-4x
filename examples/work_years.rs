//! **Growth's objective: work-years** — `∫ Σ_p infra_p dt`, per player
//! (`Hyades_trees_and_card_value.md` §2.3.3).
//!
//! **Why this exists, and why it is not colony-years.** Every economic
//! ratification in this project has been guarded on colony-years, and the works
//! branch measured that guard **inverted** for anything that changes how
//! minerals are spent:
//!
//! | | infra builds | colony-years, seed 1 |
//! |---|---|---|
//! | pre-works | 1,032 | 10,558,680 |
//! | T-73 | 57 | 10,606,309 |
//! | T-81 | 31 | **10,633,441** |
//! | R-IND17 | 66 | 10,582,211 |
//!
//! Colony-years *peaks* where development is most broken, because minerals
//! denied to infrastructure buy hulls and a `k_high`-bound bed takes worlds
//! earlier. Four changes were guarded on it and it rewarded the breakage every
//! time.
//!
//! **Work-years is the objective that would have caught all four**, and it is
//! measurable **now** rather than after works land: T-70 stores infrastructure
//! as the minerals standing in it, which is exactly the interim stock
//! §2.3.3 names.
//!
//! **Sampled on a fixed grid, not on events.** Event-driven sampling weights the
//! series by activity, which is precisely the thing being measured — so the sim
//! is stepped to a time grid and the snapshot read there. That is also why this
//! needs no engine surface: `step()` and `clock()` are already public, and the
//! metric stays in the harness where a metric belongs.
//!
//! **It is also the objective `reinvest_bias` is tuned against** (R-O87). That
//! knob decides deepen-versus-expand, so colony-years scores it with the sign
//! reversed for exactly the reason the table above shows: minerals denied to
//! infrastructure buy hulls, and a `k_high`-bound bed takes worlds earlier.
//! Sweep it here, where the metric is the stock the decision actually moves.
//!
//! Run: `cargo run --release --example work_years -- [horizon] [bias...]`
use hyades_engine::prelude::*;
use std::io::Write;

/// **The standard four-seed CRN bed** (`examples/gradient_probe`,
/// `examples/reach_limit`). Two seeds was enough while this harness only
/// reported one configuration; sweeping `reinvest_bias` compares configurations
/// against each other, and two seeds cannot separate a real +1% from a galaxy
/// that happened to like the policy — the fine sweep put seed 1's peak at 0.968
/// and seed 7's at 0.972 with ±3% swings between adjacent values.
const SEEDS: [u64; 4] = [1, 7, 42, 31337];

/// The bed, with an **independent replication set** available via `WY_SEEDS`.
///
/// A 2-SE reading on four seeds is a coin landing on its edge (`CLAUDE.md` §2 —
/// `survey_reserve` cleared that bar and was a false positive), and the fix for
/// that is more seeds rather than a softer bar. Replicating on seeds the
/// candidate was not chosen against is the stronger version: it cannot inherit
/// whatever made the original four agree.
fn seeds() -> Vec<u64> {
    match std::env::var("WY_SEEDS") {
        Ok(v) => v.split(',').filter_map(|s| s.trim().parse().ok()).collect(),
        Err(_) => SEEDS.to_vec(),
    }
}
const PLAYERS: usize = 3;
const DEFAULT_HORIZON: f64 = 4000.0;
/// One sample per production cycle. Finer buys nothing — infrastructure only
/// moves when a build applies, and builds are cadence-bounded.
const SAMPLE_YEARS: f64 = 50.0;

struct Run {
    work_years: f64,
    colony_years: f64,
    final_works: f64,
    final_colonies: usize,
    /// Simulated years per real second, against T-24's floor of 2.5. Free to
    /// collect here and the only way this harness notices that a change bought
    /// its work-years with entity count — which is the engine's first-order
    /// cost and the thing that has moved the throughput table every time.
    yr_per_s: f64,
    vehicles: usize,
    /// **Nanoseconds per event — the per-tick budget, not the aggregate.**
    ///
    /// `yr_per_s` is a *rate over a re-selected population*: a change that
    /// raises the number of events per simulated year lowers it without making
    /// any single event slower, and a change that makes every event slower
    /// lowers it too. Those are different problems with different fixes, and the
    /// aggregate cannot tell them apart. T-69 is the worked example —
    /// throughput fell 27% while vehicle count *fell*, because the cost was
    /// event count and not per-event work.
    ///
    /// This is `CLAUDE.md` §2's seventh artifact shape (an aggregate that moves
    /// against its parts) applied to performance, and the decomposition is the
    /// same prescription: report the mix beside the mean.
    ns_per_event: f64,
    events: u64,
}

fn run(seed: u64, horizon: f64, bias: f64) -> Run {
    let galaxy = Galaxy::generate(GalaxyConfig::new(PLAYERS, seed)).unwrap();
    let mut cfg = SimConfig::new(seed);
    cfg.horizon_years = horizon;
    // **Common random numbers**: the same seed drives the galaxy and the sim at
    // every bias, so the difference between two rows is the knob and not the
    // draw (`CLAUDE.md` §2).
    let doctrine = Doctrine { reinvest_bias: bias, ..Doctrine::default() };
    let autopilots: Vec<Box<dyn Autopilot>> =
        (0..PLAYERS).map(|_| Box::new(BaselineAutopilot::new(doctrine)) as Box<_>).collect();
    let mut sim = Simulation::new(galaxy, cfg, autopilots);
    let t0 = std::time::Instant::now();

    let (mut work_years, mut colony_years) = (0.0f64, 0.0f64);
    let (mut prev_t, mut prev_works, mut prev_col) = (0.0f64, 0.0f64, 0.0f64);
    let mut done = false;
    let mut next = SAMPLE_YEARS;
    while !done && next <= horizon {
        while sim.clock() < next {
            if !sim.step() {
                done = true;
                break;
            }
        }
        let snap = sim.snapshot();
        let (mut works, mut colonies) = (0.0f64, 0.0f64);
        for p in snap.planets.iter().filter(|p| p.owner.is_some()) {
            works += p.works.kilotons();
            colonies += 1.0;
        }
        // Trapezoid: the stock is piecewise constant between builds, so either
        // rule is defensible; trapezoid is stated so the number is reproducible.
        let dt = sim.clock().min(next) - prev_t;
        work_years += 0.5 * (works + prev_works) * dt;
        colony_years += 0.5 * (colonies + prev_col) * dt;
        prev_t = sim.clock().min(next);
        prev_works = works;
        prev_col = colonies;
        next += SAMPLE_YEARS;
    }

    let secs = t0.elapsed().as_secs_f64().max(1e-9);
    let events = sim.report().events_processed;
    let snap = sim.snapshot();
    let owned: Vec<_> = snap.planets.iter().filter(|p| p.owner.is_some()).collect();
    Run {
        work_years,
        colony_years,
        final_works: owned.iter().map(|p| p.works.kilotons()).sum(),
        final_colonies: owned.len(),
        yr_per_s: horizon / secs,
        vehicles: snap.vehicles.len(),
        ns_per_event: secs * 1.0e9 / (events.max(1) as f64),
        events,
    }
}

fn main() {
    let mut args = std::env::args().skip(1);
    let horizon: f64 = args.next().and_then(|a| a.parse().ok()).unwrap_or(DEFAULT_HORIZON);
    let biases: Vec<f64> = {
        let v: Vec<f64> = args.filter_map(|a| a.parse().ok()).collect();
        if v.is_empty() {
            vec![Doctrine::default().reinvest_bias]
        } else {
            v
        }
    };

    println!("work-years — Growth's objective, {PLAYERS} seats, {horizon:.0} yr, sampled every {SAMPLE_YEARS:.0} yr");
    println!("colony-years is carried alongside as a *side-effect* read, never the verdict\n");
    println!(
        "{:>7}{:>6}{:>16}{:>16}{:>13}{:>11}{:>10}{:>9}{:>12}{:>10}",
        "bias",
        "seed",
        "work-years",
        "colony-years",
        "final works",
        "colonies",
        "vehicles",
        "yr/s",
        "events",
        "ns/event"
    );
    std::io::stdout().flush().ok();

    // Per-seed work-years at the first bias listed, so every later row can be
    // reported as a **paired** difference against it. Under CRN the difference
    // has far lower variance than either level (`CLAUDE.md` §2), and the SE of
    // that difference is the only thing that says whether a row is a finding.
    let mut base: Option<Vec<f64>> = None;
    for &bias in &biases {
        let (mut w, mut c) = (0.0, 0.0);
        let mut per_seed: Vec<f64> = Vec::new();
        let bed = seeds();
        for seed in bed.iter().copied() {
            let r = run(seed, horizon, bias);
            println!(
                "{bias:>7.4}{seed:>6}{:>16.1}{:>16.1}{:>13.2}{:>11}{:>10}{:>9.1}{:>12}{:>10.0}",
                r.work_years,
                r.colony_years,
                r.final_works,
                r.final_colonies,
                r.vehicles,
                r.yr_per_s,
                r.events,
                r.ns_per_event
            );
            std::io::stdout().flush().ok();
            w += r.work_years;
            c += r.colony_years;
            per_seed.push(r.work_years);
        }
        let n = bed.len() as f64;
        print!("{bias:>7.4}{:>6}{:>16.1}{:>16.1}", "MEAN", w / n, c / n);
        match &base {
            None => base = Some(per_seed),
            Some(b) => {
                // Paired percentage differences, one per seed.
                let d: Vec<f64> =
                    b.iter().zip(&per_seed).map(|(&x, &y)| 100.0 * (y - x) / x.abs().max(1e-12)).collect();
                let mean = d.iter().sum::<f64>() / n;
                // Sample SD over seeds, then the SE of their mean. With n = 4
                // this is a coarse instrument; it is still the difference
                // between a finding and a coin landing on its edge.
                let var = if d.len() > 1 { d.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0) } else { 0.0 };
                let se = (var / n).sqrt();
                let verdict = if mean.abs() > 2.0 * se { "" } else { "  (inside 2 SE — not a finding)" };
                print!("   work-years {mean:+.2}% ± {se:.2} vs bias {:.4}{verdict}", biases[0]);
            }
        }
        println!();
        std::io::stdout().flush().ok();
    }
}
