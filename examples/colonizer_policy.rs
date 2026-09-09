//! **R-IND11: is a General coloniser ever worth it? — measured, not argued.**
//!
//! T-67 took infrastructure out of `K`, so both viable hulls now seed a colony
//! to the *world's own* ceiling and the hold is the only thing separating them.
//! That deleted the mechanism R-O76 measured ("seed depth does not pay" was
//! about a hold/ceiling mismatch that no longer exists) and reopened the
//! question with two credible answers:
//!
//! - **`CheapestViable`** — a deeper seed does not found another world. Ten
//!   Mediums make ten colonies, each with its own `K` and its own growth curve;
//!   one General makes one colony that starts further up a curve it would have
//!   climbed anyway.
//! - **`SettlersPerMineral`** — a General costs 10x and holds 31.6x, so where
//!   the world can absorb the load it lands three times the people per mineral.
//!
//! The objective is **absolute colony count** (T-20); colony-years is the guard
//! (`CLAUDE.md` §7). Common random numbers throughout — every policy is
//! evaluated on the same four seeds and compared seed by seed, because seed
//! noise here dwarfs the effect.
//!
//! ## The two rival mechanisms, and how this tells them apart
//!
//! A colony-years gain with a large negative shift in mean founding time has
//! two stories, and `CLAUDE.md` §2 forbids shipping the plausible one:
//!
//! - **Transit** — a deep seed becomes a *forward base* sooner, so later
//!   colonisers launch from closer to the frontier. Signature: `mean_flight`
//!   (coloniser spawn → founding) falls.
//! - **Gate-skipping** — a General hull seeds past the `PopBands` staircase the
//!   pre-`medium_min_level` deepening loop otherwise has to climb, so the new
//!   colony can build *at once*. Signature: `mean_dwell` (founding → that
//!   planet's own first build) collapses while flight time stays flat.
//!
//! They are not the same claim: transit shortens the voyage, gate-skipping
//! shortens the wait before the voyage is ever ordered.
//!
//! Run: `cargo run --release --example colonizer_policy`
use hyades_engine::autopilot::{BuildOrder, ColonizerPolicy};
use hyades_engine::log::{LogCategory, LogEvent, LogFilter};
use hyades_engine::prelude::*;
use std::collections::HashMap;
use std::io::Write;

const SEEDS: [u64; 4] = [1, 7, 42, 31337];
const PLAYERS: usize = 3;
const HORIZON: f64 = 4000.0;

struct Run {
    colonies: usize,
    colony_years: f64,
    first: f64,
    mean_founding: f64,
    /// Mean **flight** time of a colonising voyage, spawn to founding — the
    /// transit hypothesis made measurable.
    mean_flight: f64,
    /// Mean **dwell**: founding of a colony to that colony's own first applied
    /// build — the gate-skipping hypothesis made measurable.
    mean_dwell: f64,
    /// Share of coloniser builds that were General hulls. Mining pairs are
    /// `LimitedSystems`, so the coloniser population is exactly the
    /// Medium/General Systems builds.
    general_share: f64,
}

fn run(seed: u64, policy: ColonizerPolicy) -> Run {
    let galaxy = Galaxy::generate(GalaxyConfig::new(PLAYERS, seed)).unwrap();
    let doctrine = Doctrine { colonizer_policy: policy, ..Default::default() };
    let autopilots: Vec<Box<dyn Autopilot>> =
        (0..PLAYERS).map(|_| Box::new(BaselineAutopilot::new(doctrine)) as Box<_>).collect();
    let mut cfg = SimConfig::new(seed);
    cfg.horizon_years = HORIZON;
    let mut sim = Simulation::new(galaxy, cfg, autopilots);
    sim.set_log_filter(LogFilter::none().with(LogCategory::Vehicles).with(LogCategory::Production));
    sim.run();

    // Pass 1 — voyages, foundings, and each centre's first build.
    let mut spawn: HashMap<u64, f64> = HashMap::new();
    let mut founded_at: HashMap<u32, f64> = HashMap::new();
    let mut first_build: HashMap<u32, f64> = HashMap::new();

    let (mut colonies, mut colony_years, mut sum_t, mut first) = (0usize, 0.0, 0.0, f64::INFINITY);
    let (mut sum_flight, mut n_flight) = (0.0, 0usize);
    let (mut general, mut colonisers) = (0usize, 0usize);

    for r in sim.log().iter() {
        match r.event {
            LogEvent::VehicleSpawned { vehicle, role: Role::Colonizer, .. } => {
                spawn.insert(vehicle.0, r.time);
            }
            LogEvent::ColonyFounded { vehicle, planet, .. } => {
                colonies += 1;
                colony_years += HORIZON - r.time;
                sum_t += r.time;
                first = first.min(r.time);
                founded_at.entry(planet.0).or_insert(r.time);
                if let Some(&t0) = spawn.get(&vehicle.0) {
                    sum_flight += r.time - t0;
                    n_flight += 1;
                }
            }
            LogEvent::BuildApplied { center, order: BuildOrder::Hull { hull_type, .. }, .. } => {
                first_build.entry(center.0).or_insert(r.time);
                match hull_type {
                    HullType::MediumSystems => colonisers += 1,
                    HullType::GeneralSystems => {
                        colonisers += 1;
                        general += 1;
                    }
                    _ => {}
                }
            }
            LogEvent::BuildApplied { center, .. } => {
                first_build.entry(center.0).or_insert(r.time);
            }
            _ => {}
        }
    }

    // Pass 2 — dwell, over the colonies that ever built anything at all.
    let (mut sum_dwell, mut n_dwell) = (0.0, 0usize);
    for (planet, t_found) in &founded_at {
        if let Some(t_build) = first_build.get(planet) {
            if t_build >= t_found {
                sum_dwell += t_build - t_found;
                n_dwell += 1;
            }
        }
    }

    let mean = |s: f64, n: usize| if n > 0 { s / n as f64 } else { 0.0 };
    Run {
        colonies,
        colony_years,
        first: if first.is_finite() { first } else { 0.0 },
        mean_founding: mean(sum_t, colonies),
        mean_flight: mean(sum_flight, n_flight),
        mean_dwell: mean(sum_dwell, n_dwell),
        general_share: if colonisers > 0 { general as f64 / colonisers as f64 } else { 0.0 },
    }
}

fn main() {
    println!("R-IND11 — coloniser hull policy, CRN over {SEEDS:?}, {PLAYERS} seats, {HORIZON:.0} yr");
    println!("objective = colony COUNT; colony-years is the guard\n");
    println!(
        "{:<26}{:>10}{:>14}{:>9}{:>12}{:>10}{:>9}{:>9}",
        "policy / seed", "colonies", "colony-yr", "first", "mean found", "flight", "dwell", "gen%"
    );
    std::io::stdout().flush().ok();

    let mut totals = Vec::new();
    for policy in [ColonizerPolicy::CheapestViable, ColonizerPolicy::SettlersPerMineral] {
        let (mut c, mut y, mut m, mut fl, mut dw, mut gs) = (0usize, 0.0, 0.0, 0.0, 0.0, 0.0);
        for seed in SEEDS {
            let r = run(seed, policy);
            println!(
                "{:<26}{:>10}{:>14.0}{:>9.1}{:>12.1}{:>10.1}{:>9.1}{:>8.1}%",
                format!("{policy:?} s{seed}"),
                r.colonies,
                r.colony_years,
                r.first,
                r.mean_founding,
                r.mean_flight,
                r.mean_dwell,
                100.0 * r.general_share
            );
            std::io::stdout().flush().ok();
            c += r.colonies;
            y += r.colony_years;
            m += r.mean_founding;
            fl += r.mean_flight;
            dw += r.mean_dwell;
            gs += r.general_share;
        }
        let n = SEEDS.len() as f64;
        println!(
            "{:<26}{:>10.1}{:>14.0}{:>9}{:>12.1}{:>10.1}{:>9.1}{:>8.1}%\n",
            format!("{policy:?} MEAN"),
            c as f64 / n,
            y / n,
            "",
            m / n,
            fl / n,
            dw / n,
            100.0 * gs / n
        );
        std::io::stdout().flush().ok();
        totals.push((c as f64 / n, y / n, m / n, fl / n, dw / n));
    }

    let (b, a) = (&totals[0], &totals[1]);
    println!("SettlersPerMineral vs CheapestViable:");
    println!("  colonies      {:+.1}  ({:+.2}%)", a.0 - b.0, 100.0 * (a.0 / b.0 - 1.0));
    println!("  colony-years  {:+.0}  ({:+.2}%)", a.1 - b.1, 100.0 * (a.1 / b.1 - 1.0));
    println!("  mean founding {:+.1} yr  (negative = colonies land earlier)", a.2 - b.2);
    println!("  MECHANISM — transit:       mean flight {:+.1} yr ({:+.1}%)", a.3 - b.3, 100.0 * (a.3 / b.3 - 1.0));
    println!("  MECHANISM — gate-skipping: mean dwell  {:+.1} yr ({:+.1}%)", a.4 - b.4, 100.0 * (a.4 / b.4 - 1.0));
}
