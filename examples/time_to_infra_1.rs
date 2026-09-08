//! **How long does a colony take to reach `Band I` infrastructure?**
//!
//! Since T-60 removed the founding subsidy a colony is founded with only the
//! mass of the hull that founded it — `Band 0.046` for a Medium hull — and must
//! buy its first real rung out of its own ground. This measures how long that
//! takes, per colony, from the log:
//!
//! - `ColonyFounded` gives the founding time.
//! - The first `BuildApplied { order: UpgradeInfrastructure }` at that centre is
//!   the upgrade that lifts it to `Band I`, because `infra_step_price` rounds
//!   `0.046` to `0` and charges `1`.
//!
//! Run: `cargo run --release --example time_to_infra_1`
use hyades_engine::autopilot::BuildOrder;
use hyades_engine::log::{LogCategory, LogEvent, LogFilter};
use hyades_engine::prelude::*;
use std::collections::BTreeMap;
use std::io::Write;

const SEEDS: &[u64] = &[1, 7];
const PLAYERS: usize = 3;

fn main() {
    for &seed in SEEDS {
        let galaxy = Galaxy::generate(GalaxyConfig::new(PLAYERS, seed)).unwrap();
        let autopilots: Vec<Box<dyn Autopilot>> =
            (0..PLAYERS).map(|_| Box::new(BaselineAutopilot::new(Doctrine::default())) as Box<_>).collect();
        let mut sim = Simulation::new(galaxy, SimConfig::new(seed), autopilots);
        sim.set_log_filter(LogFilter::none().with(LogCategory::Vehicles).with(LogCategory::Production));
        sim.run();
        let horizon = sim.clock();

        let mut founded: BTreeMap<u32, f64> = BTreeMap::new();
        let mut first_upgrade: BTreeMap<u32, f64> = BTreeMap::new();
        // **Which hull actually founds colonies** — the question that decides
        // what the wait below even measures.
        let mut hulls: BTreeMap<String, usize> = BTreeMap::new();
        for r in sim.log().iter() {
            match r.event {
                LogEvent::ColonyFounded { planet, .. } => {
                    founded.entry(planet.0).or_insert(r.time);
                }
                LogEvent::BuildApplied { center, order: BuildOrder::UpgradeInfrastructure, .. } => {
                    first_upgrade.entry(center.0).or_insert(r.time);
                }
                LogEvent::BuildApplied { order: BuildOrder::Hull { hull_type, .. }, .. } => {
                    *hulls.entry(format!("{hull_type:?}")).or_default() += 1;
                }
                _ => {}
            }
        }

        let mut waits: Vec<f64> = Vec::new();
        let mut never = 0usize;
        for (pid, t0) in &founded {
            match first_upgrade.get(pid) {
                Some(t1) if t1 >= t0 => waits.push(t1 - t0),
                _ => never += 1,
            }
        }
        waits.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let pct = |q: f64| if waits.is_empty() { f64::NAN } else { waits[((waits.len() - 1) as f64 * q) as usize] };
        let mean = if waits.is_empty() { f64::NAN } else { waits.iter().sum::<f64>() / waits.len() as f64 };

        println!("seed {seed:>6}: hulls built {hulls:?}");
        println!(
            "{:>14}colonies founded={:<6} first upgrade seen={:<6} none={:<6} ({:.1}%)\n\
             {:>14}wait to that upgrade: mean {:.0} yr   p10 {:.0}   median {:.0}   p90 {:.0}   max {:.0}\n\
             {:>14}(horizon {horizon:.0} yr, so a colony founded late may simply have run out of time)",
            "",
            founded.len(),
            waits.len(),
            never,
            100.0 * never as f64 / founded.len().max(1) as f64,
            "",
            mean,
            pct(0.10),
            pct(0.50),
            pct(0.90),
            waits.last().copied().unwrap_or(f64::NAN),
            "",
        );
        std::io::stdout().flush().unwrap();
    }
}
