//! **T-64: re-ratifying `growth_rate` after the logistic moved to mass.**
//!
//! The step used to be `s + r·s·(1 − s/K)` with `s` and `K` as *ladder
//! positions*. A Band difference is not an amount of anything, so `r` there was
//! a rate of change of an exponent: the same `growth_rate` meant a different
//! number of people at every point on the ladder, and compounded hardest where
//! the ladder is widest. On mass it means what its name says — the fraction a
//! small population adds per cycle — so the ratified 0.873 is a number tuned
//! against a different quantity and has to be measured again.
//!
//! Common random numbers throughout: every candidate is evaluated on the *same*
//! four seeds and compared seed by seed, because seed noise here dwarfs the
//! effect (CLAUDE.md §2). The objective is **colony-years**, not colony count —
//! the bed is saturated, so count has almost no room to move and the whole
//! question is *when* the worlds were taken.
//!
//! Run: `cargo run --release --example growth_ratify`
use hyades_engine::prelude::*;
use std::io::Write;

const SEEDS: [u64; 4] = [1, 7, 42, 31337];
const PLAYERS: usize = 3;

/// **Screen on a truncated horizon, confirm on the objective** (CLAUDE.md §2).
/// `horizon_years` is purely a stopping condition, so a short run is a faithful
/// *prefix*, and cost is violently superlinear in duration. Pass `--confirm` to
/// re-run the shortlist on the real 4,000-year objective.
const SCREEN: f64 = 2000.0;
const OBJECTIVE: f64 = 4000.0;

/// `∫ colonies dt` over the run — the guard metric, read off founding times.
fn colony_years(seed: u64, growth: f64, horizon: f64) -> (f64, usize) {
    let galaxy = Galaxy::generate(GalaxyConfig::new(PLAYERS, seed)).unwrap();
    let doctrine = Doctrine { growth_rate: growth, ..Default::default() };
    let autopilots: Vec<Box<dyn Autopilot>> =
        (0..PLAYERS).map(|_| Box::new(BaselineAutopilot::new(doctrine)) as Box<_>).collect();
    let mut cfg = SimConfig::new(seed);
    cfg.horizon_years = horizon;
    let mut sim = Simulation::new(galaxy, cfg, autopilots);
    sim.set_log_filter(LogFilter::none().with(LogCategory::Vehicles));
    sim.run();

    let mut years = 0.0;
    let mut founded = 0usize;
    for r in sim.log().iter() {
        if let LogEvent::ColonyFounded { .. } = r.event {
            years += horizon - r.time;
            founded += 1;
        }
    }
    (years, founded)
}

fn main() {
    // A geometric sweep: the logistic's time constant is `1/r`, so the
    // informative spacing is multiplicative, not additive.
    // **The list stops below 2.0 on purpose.** That is the period-doubling
    // bifurcation of the discrete logistic
    // (`the_population_logistic_is_a_rate_and_not_a_step`), past which a
    // population orbits `K` instead of settling at it. The engine clamps at
    // `K`, so a larger `r` would *score well here* while quietly turning growth
    // from a rate into a step function that fills a world in one cycle. A sweep
    // cannot see that, so it is ruled out before the sweep runs rather than by
    // it.
    let confirm = std::env::args().any(|a| a == "--confirm");
    let horizon = if confirm { OBJECTIVE } else { SCREEN };
    // **The objective is a step function of `r`, so map the plateaus.** Growth
    // reaches the objective only through *how many 50-year cycles* a centre
    // takes to cross a `PopBands` edge, and that is an integer — so `r` is a
    // selector over cycle counts, not a dial. The first screen made this
    // unmissable: 1.10 and 1.35 scored bit-identically, and so did 1.60 and
    // 1.90. Ratifying off a coarse grid would have picked a plateau *edge* by
    // accident, which is the same trap as `coverage_trace`'s degenerate sample
    // points (CLAUDE.md §2). Same root cause as R-O68: the gate downstream is
    // discrete.
    let fine = std::env::args().any(|a| a == "--fine");
    let sweep: Vec<f64> = (0..24).map(|i| 0.80 + 0.05 * i as f64).collect();
    let candidates: &[f64] = if confirm {
        &[0.873, 1.35, 1.9]
    } else if fine {
        &sweep
    } else {
        &[0.873, 1.1, 1.35, 1.6, 1.9]
    };
    println!("growth_rate sweep — colony-years, CRN over {SEEDS:?}, {PLAYERS} seats, {horizon:.0} yr\n");
    print!("{:>8}", "r");
    for s in SEEDS {
        print!("{:>14}", format!("seed {s}"));
    }
    println!("{:>14}{:>10}{:>12}", "mean", "colonies", "vs 0.873");
    std::io::stdout().flush().ok();

    let mut baseline = 0.0;
    for (i, &r) in candidates.iter().enumerate() {
        print!("{r:>8.3}");
        std::io::stdout().flush().ok();
        let mut total = 0.0;
        let mut cols = 0usize;
        for seed in SEEDS {
            let (y, c) = colony_years(seed, r, horizon);
            total += y;
            cols += c;
            print!("{y:>14.0}");
            std::io::stdout().flush().ok();
        }
        let mean = total / SEEDS.len() as f64;
        if i == 0 {
            baseline = mean;
        }
        println!("{:>14.0}{:>10}{:>11.2}%", mean, cols / SEEDS.len(), 100.0 * (mean / baseline - 1.0));
        std::io::stdout().flush().ok();
    }
}
