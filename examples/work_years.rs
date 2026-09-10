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
//! Run: `cargo run --release --example work_years`
use hyades_engine::prelude::*;
use std::io::Write;

const SEEDS: [u64; 2] = [1, 7];
const PLAYERS: usize = 3;
const HORIZON: f64 = 4000.0;
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
}

fn run(seed: u64) -> Run {
    let galaxy = Galaxy::generate(GalaxyConfig::new(PLAYERS, seed)).unwrap();
    let mut cfg = SimConfig::new(seed);
    cfg.horizon_years = HORIZON;
    let mut sim = Simulation::with_baseline(galaxy, cfg);
    let t0 = std::time::Instant::now();

    let (mut work_years, mut colony_years) = (0.0f64, 0.0f64);
    let (mut prev_t, mut prev_works, mut prev_col) = (0.0f64, 0.0f64, 0.0f64);
    let mut done = false;
    let mut next = SAMPLE_YEARS;
    while !done && next <= HORIZON {
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
    let snap = sim.snapshot();
    let owned: Vec<_> = snap.planets.iter().filter(|p| p.owner.is_some()).collect();
    Run {
        work_years,
        colony_years,
        final_works: owned.iter().map(|p| p.works.kilotons()).sum(),
        final_colonies: owned.len(),
        yr_per_s: HORIZON / secs,
        vehicles: snap.vehicles.len(),
    }
}

fn main() {
    println!("work-years — Growth's objective, {PLAYERS} seats, {HORIZON:.0} yr, sampled every {SAMPLE_YEARS:.0} yr");
    println!("colony-years is carried alongside as a *side-effect* read, never the verdict\n");
    println!(
        "{:>6}{:>16}{:>16}{:>13}{:>11}{:>10}{:>10}",
        "seed", "work-years", "colony-years", "final works", "colonies", "vehicles", "yr/s"
    );
    std::io::stdout().flush().ok();

    let (mut w, mut c) = (0.0, 0.0);
    for seed in SEEDS {
        let r = run(seed);
        println!(
            "{seed:>6}{:>16.1}{:>16.1}{:>13.2}{:>11}{:>10}{:>10.1}",
            r.work_years, r.colony_years, r.final_works, r.final_colonies, r.vehicles, r.yr_per_s
        );
        std::io::stdout().flush().ok();
        w += r.work_years;
        c += r.colony_years;
    }
    let n = SEEDS.len() as f64;
    println!("\n{:>6}{:>16.1}{:>16.1}", "MEAN", w / n, c / n);
}
