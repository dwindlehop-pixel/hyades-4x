//! **Finding a cheap stand-in for the coverage objective.**
//!
//! Every search in this tree optimizes *coverage at 4,000 years* — the
//! fraction of colonizable worlds colonized by the horizon. It is the right
//! objective and it is expensive: one evaluation is one full-length
//! simulation, ~25–30 s at the shipped snowball defaults, so a 9-knob
//! gradient probe costs 72 of them and an offline search costs hundreds.
//! Throughput here is not a comfort, it is balance coverage not bought
//! (CLAUDE.md §7).
//!
//! This file asks: **is there a metric that is quick to measure and ranks
//! configurations the same way coverage does?** If so, searches can run
//! against the proxy and confirm on the real thing, instead of paying full
//! price for every gradient step.
//!
//! ## Why time-to-10%-colonized was the wrong instinct
//!
//! `examples/time_to_10pct_probe.rs` used years-to-10%-colonized. It failed
//! *both* halves of the bar, and the failures are instructive:
//!
//! - **Not quick.** 10% is not reached until t≈2,800 of 4,000 (seed 1), so
//!   measuring it costs essentially a full run. A metric you can only read
//!   near the horizon is not a shortcut to the horizon.
//! - **Not aligned.** It ranked `medium_fleet_size` *backwards* relative to
//!   coverage (+161.5 ± 41.7 yr/ln against coverage's +32.7 ± 3.6 pts/ln).
//!   Read as a proxy rather than as a rival objective, that is not a design
//!   tension to adjudicate — it is a **disqualification**.
//!
//! ## The design
//!
//! A proxy is only useful if it **ranks configurations** the way the real
//! objective does, so that is what gets measured — not correlation across
//! seeds, which would only prove both metrics can tell an easy galaxy from a
//! hard one. For each seed the configurations are ranked by the proxy and by
//! true coverage, and Spearman's ρ is computed *within that seed*, then
//! averaged. Seed difficulty cancels exactly, the way common random numbers
//! cancel it elsewhere.
//!
//! Every candidate is computable from the same timestamped `ColonyFounded`
//! stream, so **one full-length run yields the ground truth and all
//! candidates simultaneously**. Calibration costs a single sweep; the payoff
//! is every sweep after it.
//!
//! Candidates, all oriented "higher is better" so a good proxy shows ρ ≈ +1:
//!
//! - `colonies@T` — colonized targets by year `T`, for a ladder of `T`.
//!   Measurable with `horizon_years = T`, which is where the speedup comes
//!   from (`horizon_years` is purely a stopping condition — a truncated run
//!   is a faithful prefix, not a different simulation).
//! - `-time_to_N` — years to reach `N` colonies, negated. Measurable by
//!   stopping as soon as `N` is hit.
//! - `log_slope` — the fitted exponential rate of the ramp over an early
//!   window. The ramp is clean compounding
//!   (`examples/colonization_ramp_trace.rs`: 3→7→14→…→3348), so its rate is
//!   the theoretically-motivated summary of "how fast is this economy
//!   doubling" and should predict where it lands.
//!
//! Reported alongside ρ: the **`medium_fleet_size` sign test**, the specific
//! trap that caught time-to-10%. A proxy that ranks that knob backwards is
//! rejected however good its average ρ looks.
//!
//! (`std::time::Instant` appears here for cost reporting. That is a harness
//! measurement, never an input to the simulation — the no-clock invariant is
//! a rule about `src/`, and nothing timed here feeds replicated state.)
//!
//! Run: `cargo run --release --example proxy_metric_calibration`

use std::collections::HashSet;
use std::io::Write;
use std::time::Instant;

use hyades_engine::autopilot::{Autopilot, BaselineAutopilot, Doctrine};
use hyades_engine::log::{LogCategory, LogEvent, LogFilter};
use hyades_engine::prelude::*;

const SEEDS: &[u64] = &[1, 7, 42];
const PLAYERS: usize = 3;

/// Checkpoints for the `colonies@T` family. The early ones are deliberately
/// included even though colony counts there are small integers with heavy
/// ties — showing *where* rank correlation breaks down is the point.
const CHECKPOINTS: &[f64] = &[250.0, 500.0, 750.0, 1000.0, 1500.0, 2000.0];
/// Thresholds for the `-time_to_N` family.
const THRESHOLDS: &[usize] = &[25, 50, 100, 200];
/// Window for the exponential-rate fit.
const SLOPE_WINDOW: (f64, f64) = (500.0, 1500.0);
/// True-coverage floor for the "healthy band" — configurations at or above it
/// are working economies rather than collapses, and telling *those* apart is
/// the job a search proxy has to do.
const HEALTHY_COVERAGE: f64 = 0.25;

/// A configuration to rank. `±25%` rather than the probe's `±10%`: this needs
/// *spread* in the ground truth to have something to correlate against, not a
/// local derivative.
struct Config {
    name: String,
    cfg: SimConfig,
    doctrine: Doctrine,
}

fn configs() -> Vec<Config> {
    let base_cfg = SimConfig::new(0);
    let base_doc = Doctrine::default();
    let mut out = vec![Config { name: "default".into(), cfg: base_cfg, doctrine: base_doc }];

    let mut push = |name: &str, cfg: SimConfig, doctrine: Doctrine| {
        if cfg.hull_ladder_fault().is_none() {
            out.push(Config { name: name.into(), cfg, doctrine });
        }
    };
    for (tag, mult) in [("lo", 0.75), ("hi", 1.25)] {
        let mut c = base_cfg;
        c.medium_fleet_size = base_cfg.medium_fleet_size * mult;
        push(&format!("medium_fleet_size.{tag}"), c, base_doc);

        let mut c = base_cfg;
        c.outpost_mining_fraction = base_cfg.outpost_mining_fraction * mult;
        push(&format!("outpost_mining_fraction.{tag}"), c, base_doc);

        let mut c = base_cfg;
        c.cargo_unit_size = base_cfg.cargo_unit_size * mult;
        push(&format!("cargo_unit_size.{tag}"), c, base_doc);

        let mut d = base_doc;
        d.growth_rate = base_doc.growth_rate * mult;
        push(&format!("growth_rate.{tag}"), base_cfg, d);

        let mut d = base_doc;
        d.rank.k_high = base_doc.rank.k_high * mult;
        push(&format!("rank.k_high.{tag}"), base_cfg, d);
    }
    out
}

fn coverage_targets(galaxy: &Galaxy) -> HashSet<PlanetId> {
    galaxy.planets.iter().filter(|p| p.habitability.min(p.biosphere) > 0.01).map(|p| p.id).collect()
}

/// A named proxy: how to read one candidate metric off a completed run.
type Candidate = (String, Box<dyn Fn(&RunResult) -> f64>);

/// Everything one full-length run yields: the ground truth plus every
/// candidate proxy, all off the same event stream.
struct RunResult {
    coverage: f64,
    colonies_at: Vec<f64>,
    neg_time_to: Vec<f64>,
    log_slope: f64,
}

fn run(seed: u64, cfg: SimConfig, doctrine: Doctrine) -> RunResult {
    let galaxy = Galaxy::generate(GalaxyConfig::new(PLAYERS, seed)).unwrap();
    let targets = coverage_targets(&galaxy);
    let total = targets.len().max(1);

    let autopilots: Vec<Box<dyn Autopilot>> =
        (0..PLAYERS).map(|_| Box::new(BaselineAutopilot::new(doctrine)) as Box<_>).collect();
    let mut sim = Simulation::new(galaxy, cfg, autopilots);
    sim.set_log_filter(LogFilter::none().with(LogCategory::Vehicles));
    sim.run();

    // Founding times, homeworlds included at t=0 so the curve starts where
    // the game does.
    let mut times: Vec<f64> = std::iter::repeat_n(0.0, PLAYERS)
        .chain(sim.log().by_category(LogCategory::Vehicles).filter_map(|r| match r.event {
            LogEvent::ColonyFounded { planet, .. } if targets.contains(&planet) => Some(r.time),
            _ => None,
        }))
        .collect();
    times.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let count_by = |t: f64| times.partition_point(|&x| x < t);
    let colonies_at: Vec<f64> = CHECKPOINTS.iter().map(|&t| count_by(t) as f64).collect();
    let neg_time_to: Vec<f64> =
        THRESHOLDS.iter().map(|&n| if times.len() >= n { -times[n - 1] } else { -cfg.horizon_years }).collect();

    // Exponential rate over the window: ln(count) is ~linear while compounding.
    let (t0, t1) = SLOPE_WINDOW;
    let (c0, c1) = (count_by(t0).max(1) as f64, count_by(t1).max(1) as f64);
    let log_slope = (c1.ln() - c0.ln()) / (t1 - t0);

    // Ground truth: coverage at the horizon, straight off the snapshot.
    let snap = sim.snapshot();
    let coverage =
        snap.planets.iter().filter(|p| p.owner.is_some() && targets.contains(&p.id)).count() as f64 / total as f64;

    RunResult { coverage, colonies_at, neg_time_to, log_slope }
}

/// Ranks with ties averaged.
fn ranks(v: &[f64]) -> Vec<f64> {
    let n = v.len();
    let mut idx: Vec<usize> = (0..n).collect();
    idx.sort_by(|&a, &b| v[a].partial_cmp(&v[b]).unwrap());
    let mut out = vec![0.0; n];
    let mut i = 0;
    while i < n {
        let mut j = i;
        while j + 1 < n && v[idx[j + 1]] == v[idx[i]] {
            j += 1;
        }
        let avg = ((i + j) as f64) / 2.0 + 1.0;
        for &k in &idx[i..=j] {
            out[k] = avg;
        }
        i = j + 1;
    }
    out
}

fn pearson(a: &[f64], b: &[f64]) -> f64 {
    let n = a.len() as f64;
    let (ma, mb) = (a.iter().sum::<f64>() / n, b.iter().sum::<f64>() / n);
    let cov: f64 = a.iter().zip(b).map(|(x, y)| (x - ma) * (y - mb)).sum();
    let va: f64 = a.iter().map(|x| (x - ma).powi(2)).sum::<f64>().sqrt();
    let vb: f64 = b.iter().map(|y| (y - mb).powi(2)).sum::<f64>().sqrt();
    if va == 0.0 || vb == 0.0 {
        return 0.0;
    }
    cov / (va * vb)
}

fn spearman(a: &[f64], b: &[f64]) -> f64 {
    pearson(&ranks(a), &ranks(b))
}

fn main() {
    // The two phases are independently runnable: the correlation sweep is the
    // expensive one (full-length runs by construction — it needs the ground
    // truth), the cost table is cheap. `--cost-only` skips straight to the
    // second so the price of a truncated run can be re-measured on a new
    // machine without re-paying for the first.
    let cost_only = std::env::args().any(|a| a == "--cost-only");
    if !cost_only {
        correlation_phase();
    }
    cost_phase();
}

fn correlation_phase() {
    let cfgs = configs();
    println!("Proxy-metric calibration — {PLAYERS} seats, {} seeds, {} configurations (±25%)", SEEDS.len(), cfgs.len());
    println!(
        "Ground truth: coverage at {:.0} yr. One full run yields every candidate.\n",
        SimConfig::new(0).horizon_years
    );
    std::io::stdout().flush().ok();

    // results[seed][config]
    let mut results: Vec<Vec<RunResult>> = Vec::new();
    let started = Instant::now();
    for &seed in SEEDS {
        let mut per_seed = Vec::new();
        for c in &cfgs {
            let t = Instant::now();
            let r = run(seed, c.cfg, c.doctrine);
            println!(
                "  seed {seed:>5}  {:<28} coverage {:>5.1}%   colonies@1000 {:>5.0}   log_slope {:.5}   [{:.1}s]",
                c.name,
                r.coverage * 100.0,
                r.colonies_at[3],
                r.log_slope,
                t.elapsed().as_secs_f64()
            );
            std::io::stdout().flush().ok();
            per_seed.push(r);
        }
        results.push(per_seed);
    }
    println!("\nFull-length sweep took {:.1} min.\n", started.elapsed().as_secs_f64() / 60.0);

    // Candidate proxies, all oriented higher-is-better.
    let mut candidates: Vec<Candidate> = Vec::new();
    for (i, &t) in CHECKPOINTS.iter().enumerate() {
        candidates.push((format!("colonies@{t:.0}"), Box::new(move |r: &RunResult| r.colonies_at[i])));
    }
    for (i, &n) in THRESHOLDS.iter().enumerate() {
        candidates.push((format!("-time_to_{n}"), Box::new(move |r: &RunResult| r.neg_time_to[i])));
    }
    candidates.push((
        format!("log_slope[{:.0},{:.0}]", SLOPE_WINDOW.0, SLOPE_WINDOW.1),
        Box::new(|r: &RunResult| r.log_slope),
    ));

    // The medium_fleet_size sign test — the trap that disqualified time-to-10%.
    let mfs_lo = cfgs.iter().position(|c| c.name == "medium_fleet_size.lo");
    let mfs_hi = cfgs.iter().position(|c| c.name == "medium_fleet_size.hi");

    println!("=== Rank agreement with coverage, within seed (Spearman ρ, averaged) ===");
    println!(
        "{:<24} {:>8} {:>8} {:>9}  {:<18}  measurable by",
        "proxy", "mean ρ", "min ρ", "ρ healthy", "mfs sign test"
    );
    let mut table = Vec::new();
    for (name, f) in &candidates {
        let mut rhos = Vec::new();
        let mut healthy_rhos = Vec::new();
        for per_seed in &results {
            let truth: Vec<f64> = per_seed.iter().map(|r| r.coverage).collect();
            let proxy: Vec<f64> = per_seed.iter().map(f).collect();
            rhos.push(spearman(&proxy, &truth));

            // **The mirage check.** The config set includes `k_high`
            // catastrophes (sub-15% coverage against ~50% for everything
            // else), and merely detecting "this economy collapsed" would earn
            // a high ρ without the proxy being able to tell two *working*
            // configurations apart — which is the discrimination a search
            // actually needs. So ρ is recomputed over the healthy band alone.
            let healthy: Vec<usize> = (0..truth.len()).filter(|&i| truth[i] >= HEALTHY_COVERAGE).collect();
            if healthy.len() >= 3 {
                let t: Vec<f64> = healthy.iter().map(|&i| truth[i]).collect();
                let p: Vec<f64> = healthy.iter().map(|&i| proxy[i]).collect();
                healthy_rhos.push(spearman(&p, &t));
            }
        }
        let mean_rho = rhos.iter().sum::<f64>() / rhos.len() as f64;
        let min_rho = rhos.iter().cloned().fold(f64::INFINITY, f64::min);
        let healthy_rho = if healthy_rhos.is_empty() {
            f64::NAN
        } else {
            healthy_rhos.iter().sum::<f64>() / healthy_rhos.len() as f64
        };

        // Does the proxy order medium_fleet_size lo-vs-hi the same way coverage does?
        let sign = match (mfs_lo, mfs_hi) {
            (Some(lo), Some(hi)) => {
                let agree = results.iter().filter(|s| {
                    let truth_dir = s[hi].coverage - s[lo].coverage;
                    let proxy_dir = f(&s[hi]) - f(&s[lo]);
                    truth_dir * proxy_dir > 0.0
                });
                format!("{}/{} seeds agree", agree.count(), results.len())
            }
            _ => "n/a".into(),
        };

        // What horizon does reading this proxy actually require?
        let need = if name.starts_with("colonies@") {
            name.trim_start_matches("colonies@").to_string()
        } else if name.starts_with("log_slope") {
            format!("{:.0}", SLOPE_WINDOW.1)
        } else {
            "when N is hit".into()
        };

        println!("{name:<24} {mean_rho:>8.3} {min_rho:>8.3} {healthy_rho:>9.3}  {sign:<18}  horizon {need}");
        std::io::stdout().flush().ok();
        table.push((name.clone(), mean_rho, min_rho, healthy_rho, sign));
    }

    table.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    println!("\n=== Ranked by mean ρ ===");
    for (name, mean_rho, min_rho, healthy_rho, sign) in &table {
        println!(
            "  {name:<24} ρ={mean_rho:>6.3}  (worst seed {min_rho:>6.3}, healthy-only {healthy_rho:>6.3})  {sign}"
        );
    }
    println!(
        "\n`ρ healthy` is the column that decides adoption: ρ over configurations that all\n\
         actually work (true coverage ≥ {:.0}%). A proxy can score well on the full set just by\n\
         spotting collapses; only the healthy-band number says whether it can rank two viable\n\
         configurations, which is what a search spends its time doing.",
        HEALTHY_COVERAGE * 100.0
    );

    println!(
        "\nReading: ρ is rank agreement *within a seed*, so seed difficulty cannot inflate it.\n\
         A proxy worth adopting needs a high mean ρ, a worst-seed ρ that is still high (one\n\
         bad seed means the search can be led astray), and it must pass the medium_fleet_size\n\
         sign test — that is the knob whose backwards ranking disqualified time-to-10%.\n\
         The cheapest proxy clearing all three is the one to search against, confirming\n\
         finalists on the real objective."
    );
}

/// The other half of "quick to measure": what a truncated run costs.
///
/// `horizon_years` is purely a stopping condition, so this is the real price
/// of every `colonies@T` proxy. Cost is superlinear in duration
/// (CLAUDE.md §7) because entity count compounds, so the saving is much
/// larger than the ratio of horizons.
fn cost_phase() {
    println!("\n=== Cost of a truncated run (default config, mean over seeds) ===");
    let base_cfg = SimConfig::new(0);
    let base_doc = Doctrine::default();
    let horizons = [500.0, 1000.0, 1500.0, 2000.0, base_cfg.horizon_years];
    let mut costs = Vec::new();
    for &h in &horizons {
        let mut cfg = base_cfg;
        cfg.horizon_years = h;
        let t = Instant::now();
        for &seed in SEEDS {
            run(seed, cfg, base_doc);
        }
        costs.push(t.elapsed().as_secs_f64() / SEEDS.len() as f64);
    }
    let full_cost = *costs.last().unwrap();
    println!("{:<12} {:>10} {:>10}  proxy it buys", "horizon", "mean sec", "speedup");
    for (&h, &secs) in horizons.iter().zip(&costs) {
        let buys = if h == base_cfg.horizon_years { "the objective itself" } else { "colonies@this horizon" };
        println!("{h:<12.0} {secs:>10.2} {:>9.1}x  {buys}", full_cost / secs);
    }
    std::io::stdout().flush().ok();
}
