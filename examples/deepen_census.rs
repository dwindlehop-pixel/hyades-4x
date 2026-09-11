//! **Does the deepen branch fire, and what does it cost to make it fire?**
//!
//! R-O68's fix put both sides of `production_choice`'s deepen-vs-expand test in
//! one unit — `rank` score per kilotonne committed — which turns
//! `reinvest_bias` from a unit-conversion constant into an odds ratio on two
//! returns. This is the instrument for the question that fix raises: *at what
//! bias does depth actually start winning, and what does the objective pay for
//! it?*
//!
//! Reports, per bias: the build mix (infra rungs vs hulls), the infrastructure
//! the empires ended up standing at against the ceiling they were allowed, and
//! both objectives — colony count and colony-years.
//!
//! Run: `cargo run --release --example deepen_census -- [seed] [horizon] [biases...]`
use hyades_engine::autopilot::BuildOrder;
use hyades_engine::log::{LogCategory, LogEvent, LogFilter};
use hyades_engine::prelude::*;
use std::io::Write;

const PLAYERS: usize = 3;

fn main() {
    let mut args = std::env::args().skip(1);
    let seed: u64 = args.next().and_then(|s| s.parse().ok()).unwrap_or(1);
    let horizon: f64 = args.next().and_then(|s| s.parse().ok()).unwrap_or(1500.0);
    let biases: Vec<f64> = {
        let v: Vec<f64> = args.filter_map(|s| s.parse().ok()).collect();
        if v.is_empty() {
            vec![0.5]
        } else {
            v
        }
    };

    println!("deepen census — seed {seed}, {PLAYERS} seats, horizon {horizon:.0} yr");
    println!(
        "{:>6} {:>8} {:>8} {:>8} {:>7} {:>7} {:>7} {:>13} {:>7}",
        "bias", "infra", "hulls", "colonies", "infra_b", "ceil", "at_cap", "colony_yr", "secs"
    );
    std::io::stdout().flush().unwrap();

    for &b in &biases {
        let t0 = std::time::Instant::now();
        let galaxy = Galaxy::generate(GalaxyConfig::new(PLAYERS, seed)).unwrap();
        let doctrine = Doctrine { reinvest_bias: b, ..Doctrine::default() };
        let autopilots: Vec<Box<dyn Autopilot>> =
            (0..PLAYERS).map(|_| Box::new(BaselineAutopilot::new(doctrine)) as Box<_>).collect();
        let mut cfg = SimConfig::new(seed);
        cfg.horizon_years = horizon;
        let mut sim = Simulation::new(galaxy, cfg, autopilots);
        sim.set_log_filter(LogFilter::none().with(LogCategory::Production).with(LogCategory::Vehicles));
        sim.run();

        let (mut infra_builds, mut hull_builds) = (0u64, 0u64);
        let mut colony_years = 0.0_f64;
        let clock = sim.clock();
        for r in sim.log().iter() {
            match r.event {
                LogEvent::BuildApplied { order: BuildOrder::UpgradeInfrastructure, .. } => infra_builds += 1,
                LogEvent::BuildApplied { order: BuildOrder::Hull { .. }, .. } => hull_builds += 1,
                LogEvent::ColonyFounded { .. } => colony_years += clock - r.time,
                _ => {}
            }
        }

        // Where the empires ended up on the infra ladder, against the ceiling
        // `deepen_possible` allows them.
        let snap = sim.snapshot();
        let (mut n, mut sum_infra, mut sum_ceil, mut at_cap) = (0usize, 0.0_f64, 0.0_f64, 0usize);
        for p in snap.planets.iter().filter(|p| p.owner.is_some()) {
            n += 1;
            let infra = p.infrastructure.bands();
            let ceil = p.k.bands();
            sum_infra += infra;
            sum_ceil += ceil;
            if infra >= ceil - 1e-9 {
                at_cap += 1;
            }
        }
        let secs = t0.elapsed().as_secs_f64();
        println!(
            "{b:>6.3} {infra_builds:>8} {hull_builds:>8} {n:>8} {:>7.3} {:>7.3} {at_cap:>7} {colony_years:>13.1} \
             {secs:>7.1}",
            sum_infra / n.max(1) as f64,
            sum_ceil / n.max(1) as f64,
        );
        std::io::stdout().flush().unwrap();
    }
}
