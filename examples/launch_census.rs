//! **Where do colony ships launch from?** (T-123) — pricing a port blockade
//! before building one.
//!
//! T-122 closed every route the first Warfare card had to `W` and found the
//! one fight the Warfare player can win — a picket defending held ground —
//! never happens: pickets stand on worlds rival colony ships are not going to,
//! and handing them the true destination produced 18 feasible intercepts a run
//! and no fights. A colony ship's *destination* is one of thousands of worlds;
//! its *origin* is one of a player's production centers. This census measures
//! how few origins there are, and how concentrated the launches are in them,
//! which is the whole question of whether a handful of armed hulls standing at
//! rival ports could meet a large share of rival colony ships.
//!
//! Baseline bed, no cards: 3 seats, 800 yr, the paired bed's 8 seeds. Grouped
//! by exact launch position — a system is a fixed point, so one center is one
//! position.

use hyades_engine::galaxy::{Galaxy, GalaxyConfig};
use hyades_engine::log::{LogCategory, LogEvent, LogFilter};
use hyades_engine::math::Vec3;
use hyades_engine::sim::{Role, SimConfig, Simulation};
use std::collections::BTreeMap;
use std::io::Write;

const SEATS: usize = 3;
const SEEDS: [u64; 8] = [1, 7, 42, 31337, 2, 3, 5, 11];
const HORIZON: f64 = 800.0;
/// The round-0 barrier, where a card played at earliest legal play lands.
const PLAY_AT: f64 = 200.0;

/// A position as a total-ordered key. Bit patterns are exact here: every launch
/// from one center reads the same stored `Vec3`.
fn key(v: Vec3) -> (u64, u64, u64) {
    (v.x.to_bits(), v.y.to_bits(), v.z.to_bits())
}

fn main() {
    println!("{SEATS} seats, {HORIZON} yr, {} seeds; colony-ship launches per seat", SEEDS.len());
    println!(
        "{:>6} {:>4} {:>8} {:>8} {:>9} {:>8} {:>8} {:>8} {:>10}",
        "seed", "seat", "launches", "after200", "origins", "home%", "top1%", "top3%", "top8%"
    );
    let _ = std::io::stdout().flush();
    let (mut home_share, mut top3_share, mut top8_share, mut origins_n) = (vec![], vec![], vec![], vec![]);
    for &seed in &SEEDS {
        let galaxy = Galaxy::generate(GalaxyConfig::new(SEATS, seed)).unwrap();
        let homes: Vec<Vec3> = galaxy.homeworlds.iter().map(|&h| galaxy.planet(h).position).collect();
        let mut cfg = SimConfig::new(seed);
        cfg.horizon_years = HORIZON;
        cfg.engagements_enabled = true;
        let mut sim = Simulation::with_baseline(galaxy, cfg);
        sim.set_log_filter(LogFilter::none().with(LogCategory::Vehicles));
        sim.run();

        // Per seat: launches after the round-0 barrier, keyed by origin.
        let mut by_seat: Vec<BTreeMap<(u64, u64, u64), u64>> = vec![BTreeMap::new(); SEATS];
        let mut all_launches = [0u64; SEATS];
        for r in sim.log().iter() {
            if let LogEvent::VehicleSpawned { player, role: Role::Colonizer, from, .. } = r.event {
                let p = player as usize;
                all_launches[p] += 1;
                if r.time >= PLAY_AT {
                    *by_seat[p].entry(key(from)).or_insert(0) += 1;
                }
            }
        }
        for p in 0..SEATS {
            let total: u64 = by_seat[p].values().sum();
            if total == 0 {
                continue;
            }
            let home = by_seat[p].get(&key(homes[p])).copied().unwrap_or(0);
            let mut counts: Vec<u64> = by_seat[p].values().copied().collect();
            counts.sort_unstable_by(|a, b| b.cmp(a));
            let top = |k: usize| counts.iter().take(k).sum::<u64>() as f64 / total as f64;
            let hs = home as f64 / total as f64;
            println!(
                "{seed:>6} {p:>4} {:>8} {total:>8} {:>9} {:>7.1}% {:>7.1}% {:>7.1}% {:>9.1}%",
                all_launches[p],
                counts.len(),
                100.0 * hs,
                100.0 * top(1),
                100.0 * top(3),
                100.0 * top(8)
            );
            home_share.push(hs);
            top3_share.push(top(3));
            top8_share.push(top(8));
            origins_n.push(counts.len() as f64);
        }
        let _ = std::io::stdout().flush();
    }
    let mean = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;
    println!(
        "\nmean over seat-seeds (launches after {PLAY_AT} yr): origins {:.1}, homeworld {:.1}%, top-3 {:.1}%, top-8 {:.1}%",
        mean(&origins_n),
        100.0 * mean(&home_share),
        100.0 * mean(&top3_share),
        100.0 * mean(&top8_share)
    );
}
