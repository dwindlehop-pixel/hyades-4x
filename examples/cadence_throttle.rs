//! **Is the 50-year production cadence throttling the expansion loop?**
//!
//! A production center decides once per `cycle_years`, and a decision buys at
//! most one thing. So a center that can afford several builds still makes one
//! and waits 50 years — the cadence is a hard cap of `horizon / cycle_years`
//! builds per center, whatever the economy does.
//!
//! Whether that cap *binds* is an empirical question, and this answers it by
//! reading the surplus at every decision: `stockpile / cost(chosen)`. A ratio
//! of 1 means the center spent what it had; 3 means it could have built three
//! times and the cadence stopped it. Idle decisions are reported separately,
//! since an idle center is throttled by poverty, not by cadence.
//!
//! Run: `cargo run --release --example cadence_throttle`
use hyades_engine::autopilot::{Autopilot, BaselineAutopilot, BuildOrder};
use hyades_engine::log::{LogCategory, LogEvent, LogFilter};
use hyades_engine::prelude::*;
use std::io::Write;

const PLAYERS: usize = 3;

fn main() {
    for seed in [1u64, 7] {
        let galaxy = Galaxy::generate(GalaxyConfig::new(PLAYERS, seed)).unwrap();
        let doctrine = Doctrine::default();
        let autopilots: Vec<Box<dyn Autopilot>> =
            (0..PLAYERS).map(|_| Box::new(BaselineAutopilot::new(doctrine)) as Box<_>).collect();
        let mut sim = Simulation::new(galaxy, SimConfig::new(seed), autopilots);
        sim.set_log_filter(LogFilter::none().with(LogCategory::Production));
        sim.run();

        let mut surplus: Vec<f64> = Vec::new();
        let (mut idle, mut total) = (0usize, 0usize);
        for r in sim.log().iter() {
            let LogEvent::ProductionDecision {
                stockpile, infra_cost, colonizer_cost, mining_pair_cost, chosen, ..
            } = r.event
            else {
                continue;
            };
            total += 1;
            let cost = match chosen {
                BuildOrder::Idle => {
                    idle += 1;
                    continue;
                }
                BuildOrder::UpgradeInfrastructure => infra_cost,
                BuildOrder::Hull { hull_type, .. } => match hull_type {
                    HullType::MediumSystems => colonizer_cost,
                    HullType::LimitedSystems => mining_pair_cost,
                    _ => continue, // survey craft: cheap by construction, not the question
                },
            };
            if cost > 0.0 {
                surplus.push(stockpile / cost);
            }
        }
        surplus.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let n = surplus.len();
        let q = |f: f64| if n == 0 { f64::NAN } else { surplus[(((n - 1) as f64) * f).round() as usize] };
        let could_have_built_again = surplus.iter().filter(|&&s| s >= 2.0).count();
        let could_have_built_5x = surplus.iter().filter(|&&s| s >= 5.0).count();

        println!("\nseed {seed}: {total} decisions, {idle} idle ({:.1}%)", 100.0 * idle as f64 / total as f64);
        println!(
            "  funded builds: n={n}  stockpile/cost  p50={:.2} p90={:.2} p99={:.2} max={:.1}",
            q(0.5),
            q(0.9),
            q(0.99),
            surplus.last().copied().unwrap_or(f64::NAN)
        );
        println!(
            "  could have built >=2x this cycle: {could_have_built_again} ({:.1}%)   >=5x: {could_have_built_5x} ({:.1}%)",
            100.0 * could_have_built_again as f64 / n.max(1) as f64,
            100.0 * could_have_built_5x as f64 / n.max(1) as f64
        );
        std::io::stdout().flush().unwrap();
    }
}
