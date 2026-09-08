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
//!   The case is not the seed but what it *becomes*: a colony that reaches
//!   production levels sooner is a forward base sooner, which shortens every
//!   later colonising transit.
//!
//! The second is a mechanism claim about **transit**, so this prints founding
//! times and the founding-rate curve alongside the objective — a policy that
//! wins by shortening transits should show it as earlier founding, not merely
//! as a bigger total.
//!
//! **The objective is absolute colony count** (T-20); colony-years is the guard
//! (`CLAUDE.md` §7). Common random numbers throughout — every policy is
//! evaluated on the same four seeds and compared seed by seed, because seed
//! noise here dwarfs the effect.
//!
//! Run: `cargo run --release --example colonizer_policy`
use hyades_engine::autopilot::ColonizerPolicy;
use hyades_engine::log::{LogCategory, LogEvent, LogFilter};
use hyades_engine::prelude::*;
use std::io::Write;

const SEEDS: [u64; 4] = [1, 7, 42, 31337];
const PLAYERS: usize = 3;
const HORIZON: f64 = 4000.0;

struct Run {
    colonies: usize,
    colony_years: f64,
    first: f64,
    mean_founding: f64,
}

fn run(seed: u64, policy: ColonizerPolicy) -> Run {
    let galaxy = Galaxy::generate(GalaxyConfig::new(PLAYERS, seed)).unwrap();
    let doctrine = Doctrine { colonizer_policy: policy, ..Default::default() };
    let autopilots: Vec<Box<dyn Autopilot>> =
        (0..PLAYERS).map(|_| Box::new(BaselineAutopilot::new(doctrine)) as Box<_>).collect();
    let mut cfg = SimConfig::new(seed);
    cfg.horizon_years = HORIZON;
    let mut sim = Simulation::new(galaxy, cfg, autopilots);
    sim.set_log_filter(LogFilter::none().with(LogCategory::Vehicles));
    sim.run();

    let (mut colonies, mut colony_years, mut sum_t) = (0usize, 0.0, 0.0);
    let mut first = f64::INFINITY;
    for r in sim.log().iter() {
        if let LogEvent::ColonyFounded { .. } = r.event {
            colonies += 1;
            colony_years += HORIZON - r.time;
            sum_t += r.time;
            first = first.min(r.time);
        }
    }
    Run {
        colonies,
        colony_years,
        first: if first.is_finite() { first } else { 0.0 },
        mean_founding: if colonies > 0 { sum_t / colonies as f64 } else { 0.0 },
    }
}

fn main() {
    println!("R-IND11 — coloniser hull policy, CRN over {SEEDS:?}, {PLAYERS} seats, {HORIZON:.0} yr");
    println!("objective = colony COUNT; colony-years is the guard\n");
    println!("{:<20}{:>10}{:>14}{:>10}{:>14}", "policy / seed", "colonies", "colony-yr", "first", "mean found");
    std::io::stdout().flush().ok();

    let mut totals = Vec::new();
    for policy in [ColonizerPolicy::CheapestViable, ColonizerPolicy::SettlersPerMineral] {
        let (mut c, mut y, mut m) = (0usize, 0.0, 0.0);
        for seed in SEEDS {
            let r = run(seed, policy);
            println!(
                "{:<20}{:>10}{:>14.0}{:>10.1}{:>14.1}",
                format!("{policy:?} s{seed}"),
                r.colonies,
                r.colony_years,
                r.first,
                r.mean_founding
            );
            std::io::stdout().flush().ok();
            c += r.colonies;
            y += r.colony_years;
            m += r.mean_founding;
        }
        let n = SEEDS.len() as f64;
        println!("{:<20}{:>10.1}{:>14.0}{:>10}{:>14.1}\n", format!("{policy:?} MEAN"), c as f64 / n, y / n, "", m / n);
        std::io::stdout().flush().ok();
        totals.push((policy, c as f64 / n, y / n, m / n));
    }

    let (base_c, base_y, base_m) = (totals[0].1, totals[0].2, totals[0].3);
    let (alt_c, alt_y, alt_m) = (totals[1].1, totals[1].2, totals[1].3);
    println!("SettlersPerMineral vs CheapestViable:");
    println!("  colonies      {:+.1}  ({:+.2}%)", alt_c - base_c, 100.0 * (alt_c / base_c - 1.0));
    println!("  colony-years  {:+.0}  ({:+.2}%)", alt_y - base_y, 100.0 * (alt_y / base_y - 1.0));
    println!("  mean founding {:+.1} yr  (negative = colonies land earlier)", alt_m - base_m);
}
