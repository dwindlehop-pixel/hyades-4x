//! **T-57: how many miners actually end up on one rock?**
//!
//! `mine_operator` was a `BTreeMap<u64, Entity>` and `insert` *overwrites*, so a
//! second miner reaching an already-worked outpost replaced the first: its
//! extraction stopped being counted and, on exhaustion, `remove()` returned only
//! the last hull — the earlier one was leaked, still `Role::Miner`, parked, never
//! re-tasked and never scrapped. `mine_crew` keeps both.
//!
//! That is the hypothesis for why `miners_per_outpost = 1` did not reproduce the
//! bed bit-for-bit. **It is a plausible mechanism attached to a real number,
//! which is the shape of every measurement artifact in `CLAUDE.md` §2**, so this
//! counts the crews instead of arguing about them. No engine change: the log
//! already records every miner that parks and where.
//!
//! Run: `cargo run --release --example crew_census`
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
        sim.set_log_filter(LogFilter::none().with(LogCategory::Vehicles));
        sim.run();

        // Distinct miners parked per planet, and by which player, so a
        // double-worked rock can be told from a re-tasked one.
        let mut per_planet: BTreeMap<u32, Vec<(u32, u64)>> = BTreeMap::new();
        for r in sim.log().iter() {
            if let LogEvent::VehicleParked { player, vehicle, role: Role::Miner, at } = r.event {
                let e = per_planet.entry(at.0).or_default();
                if !e.iter().any(|&(_, v)| v == vehicle.0) {
                    e.push((player, vehicle.0));
                }
            }
        }

        let outposts = per_planet.len();
        let multi: Vec<_> = per_planet.iter().filter(|(_, v)| v.len() > 1).collect();
        let max_crew = per_planet.values().map(|v| v.len()).max().unwrap_or(0);
        let cross_player = multi
            .iter()
            .filter(|(_, v)| {
                let first = v[0].0;
                v.iter().any(|&(p, _)| p != first)
            })
            .count();

        println!(
            "seed {seed:>6}: outposts worked = {outposts:>5}   with >1 miner = {:>5} ({:.1}%)   \
             max crew = {max_crew}   of those, cross-player = {cross_player}",
            multi.len(),
            100.0 * multi.len() as f64 / outposts.max(1) as f64,
        );
        std::io::stdout().flush().unwrap();
    }
}
