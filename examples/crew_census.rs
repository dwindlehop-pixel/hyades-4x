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
//! **Since T-71 it is also the concentration instrument** (`Hyades_industry.md`
//! §4.5). The crowding law predicts **fewer, larger, closer outposts**: a rich
//! body genuinely rewards a big crew, so the fleet concentrates rather than
//! spreading, and freight cost scales with *sites and distance* rather than with
//! crew. `CLAUDE.md` §2 is explicit that a claim about site count has to be read
//! from a census and must not be inferred from the objective — an objective that
//! moved the right way would be consistent with the prediction and would not be
//! evidence for it. So all three halves of "fewer, larger, closer" are printed
//! side by side: site count, mean crew and mean ore per site, and mean distance
//! from the working player's homeworld.
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
        // Homeworld positions before the sim consumes the galaxy, so "closer"
        // has something to be closer *to*.
        let homes: Vec<Vec3> = (0..PLAYERS).map(|p| galaxy.planet(galaxy.homeworlds[p]).position).collect();
        let planet_pos: Vec<Vec3> = galaxy.planets.iter().map(|p| p.position).collect();
        let mut sim = Simulation::new(galaxy, SimConfig::new(seed), autopilots);
        sim.set_log_filter(LogFilter::none().with(LogCategory::Vehicles).with(LogCategory::Mining));
        sim.run();

        // Distinct miners parked per planet, and by which player, so a
        // double-worked rock can be told from a re-tasked one.
        let mut per_planet: BTreeMap<u32, Vec<(u32, u64)>> = BTreeMap::new();
        // Ore lifted per body — "larger" in the sense that matters to freight.
        let mut ore: BTreeMap<u32, f64> = BTreeMap::new();
        for r in sim.log().iter() {
            match r.event {
                LogEvent::VehicleParked { player, vehicle, role: Role::Miner, at } => {
                    let e = per_planet.entry(at.0).or_default();
                    if !e.iter().any(|&(_, v)| v == vehicle.0) {
                        e.push((player, vehicle.0));
                    }
                }
                LogEvent::MineralsExtracted { planet, amount, .. } => {
                    *ore.entry(planet.0).or_default() += amount;
                }
                _ => {}
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

        // "fewer, larger, closer" — the three halves of §4.5's prediction, each
        // over the *worked* sites only, which is the population the prediction
        // is about.
        let crews: usize = per_planet.values().map(|v| v.len()).sum();
        let mean_crew = crews as f64 / outposts.max(1) as f64;
        let worked_ore: f64 = per_planet.keys().filter_map(|k| ore.get(k)).sum();
        let mean_ore = worked_ore / outposts.max(1) as f64;
        let mut dist_sum = 0.0;
        let mut dist_n = 0usize;
        for (&pid, crew) in &per_planet {
            let Some(&pos) = planet_pos.get(pid as usize) else { continue };
            for &(player, _) in crew {
                if let Some(&home) = homes.get(player as usize) {
                    dist_sum += home.distance(pos);
                    dist_n += 1;
                }
            }
        }
        let mean_dist = if dist_n > 0 { dist_sum / dist_n as f64 } else { 0.0 };

        println!(
            "seed {seed:>6}: outposts worked = {outposts:>5}   with >1 miner = {:>5} ({:.1}%)   \
             max crew = {max_crew}   of those, cross-player = {cross_player}",
            multi.len(),
            100.0 * multi.len() as f64 / outposts.max(1) as f64,
        );
        println!(
            "        §4.5 concentration: mean crew = {mean_crew:.2}                mean ore/site = {mean_ore:.1} kt   mean distance = {mean_dist:.2} ly"
        );
        std::io::stdout().flush().unwrap();
    }
}
