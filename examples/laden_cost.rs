//! **What flying the colonization leg laden costs the bed** (R-WAR9).
//!
//! The fix is a correctness fix — R-O32 says a laden colony ship is slower and
//! the dispatcher was reading the empty rate — but it moves every colonization
//! transit, so the magnitude is the thing the next reader needs and `CLAUDE.md`
//! §2 wants `ns/event` beside `yr/s` for it.

use hyades_engine::autopilot::{Autopilot, BaselineAutopilot, Doctrine};
use hyades_engine::galaxy::{Galaxy, GalaxyConfig};
use hyades_engine::sim::{SimConfig, Simulation};
use std::io::Write;
use std::time::Instant;

const SEEDS: [u64; 4] = [1, 7, 42, 31337];
const HORIZON: f64 = 800.0;

fn main() {
    println!("{:>7}  {:>9}  {:>12}  {:>9}  {:>11}", "seed", "colonies", "pop kt", "yr/s", "ns/event");
    let _ = std::io::stdout().flush();
    for s in SEEDS {
        let galaxy = Galaxy::generate(GalaxyConfig::new(3, s)).unwrap();
        let aps: Vec<Box<dyn Autopilot>> =
            (0..3).map(|_| Box::new(BaselineAutopilot::new(Doctrine::default())) as Box<_>).collect();
        let mut cfg = SimConfig::new(s);
        cfg.horizon_years = HORIZON;
        let mut sim = Simulation::new(galaxy, cfg, aps);
        let t0 = Instant::now();
        let r = sim.run();
        let secs = t0.elapsed().as_secs_f64();
        let colonies: usize = r.players.iter().map(|p| p.colonies).sum();
        println!(
            "{s:>7}  {colonies:>9}  {:>12.0}  {:>9.1}  {:>11.0}",
            r.players.iter().map(|p| p.total_population.kilotons()).sum::<f64>(),
            HORIZON / secs,
            secs * 1e9 / r.events_processed as f64
        );
        let _ = std::io::stdout().flush();
    }
}
