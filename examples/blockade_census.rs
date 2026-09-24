//! **What limits the port strike's coverage?** (T-125) — per 100-year bucket
//! after the Warfare card lands: rival colony launches, how many were struck,
//! how many blockaders stood on station, and what the busiest ports carried in
//! hindsight. 3 seats, seat 0 plays `TIER0[15]` at the round-0 barrier, the
//! paired bed's eight seeds.
//!
//! `coverage` is struck / launched. `top-k` is the share of that bucket's rival
//! launches from the `k` busiest rival ports *of that bucket*, with `k` the mean
//! number of blockaders on station — an upper bound on what those hulls could
//! have met with perfect, lag-free placement.

use hyades_engine::autopilot::{Autopilot, BaselineAutopilot, Doctrine};
use hyades_engine::cards::{CardId, Order, Target};
use hyades_engine::galaxy::{Galaxy, GalaxyConfig, PlayerId};
use hyades_engine::log::{LogCategory, LogEvent, LogFilter};
use hyades_engine::sim::{Role, SimConfig, Simulation};
use std::collections::BTreeMap;
use std::io::Write;

const SEATS: usize = 3;
const SEEDS: [u64; 8] = [1, 7, 42, 31337, 2, 3, 5, 11];
const HORIZON: f64 = 800.0;
const BUCKET: f64 = 100.0;

fn main() {
    println!("bucket  launches  struck  coverage  on-station  in-flight  top-k(hindsight)");
    let _ = std::io::stdout().flush();
    let nb = (HORIZON / BUCKET) as usize;
    let (mut launches, mut struck) = (vec![0u64; nb], vec![0u64; nb]);
    let (mut station, mut flight, mut samples) = (vec![0.0; nb], vec![0.0; nb], vec![0u64; nb]);
    let mut per_port: Vec<BTreeMap<(u64, u64, u64), u64>> = vec![BTreeMap::new(); nb];
    for &seed in &SEEDS {
        let galaxy = Galaxy::generate(GalaxyConfig::new(SEATS, seed)).unwrap();
        let aps: Vec<Box<dyn Autopilot>> =
            (0..SEATS).map(|_| Box::new(BaselineAutopilot::new(Doctrine::default())) as Box<_>).collect();
        let mut cfg = SimConfig::new(seed);
        cfg.horizon_years = HORIZON;
        cfg.engagements_enabled = true;
        let play_at = cfg.years_to_first_round;
        let mut sim = Simulation::new(galaxy, cfg, aps);
        sim.set_log_filter(LogFilter::none().with(LogCategory::Vehicles).with(LogCategory::Combat));
        let (mut played, mut next) = (false, 0.0);
        while sim.step() {
            if !played && sim.clock() >= play_at {
                sim.apply_orders(
                    sim.current_round(),
                    &[Order { seat: PlayerId(0), card: Some(CardId(15)), target: Target::None }],
                );
                played = true;
            }
            if sim.clock() >= next {
                let b = ((sim.clock() / BUCKET) as usize).min(nb - 1);
                station[b] += sim.blockaders_on_station(0) as f64;
                flight[b] += sim.blockaders_in_flight(0) as f64;
                samples[b] += 1;
                next = sim.clock() + 5.0;
            }
        }
        for r in sim.log().iter() {
            let b = ((r.time / BUCKET) as usize).min(nb - 1);
            match r.event {
                LogEvent::VehicleSpawned { player, role: Role::Colonizer, from, .. } if player != 0 => {
                    launches[b] += 1;
                    *per_port[b].entry((player as u64, from.x.to_bits(), from.y.to_bits())).or_insert(0) += 1;
                }
                LogEvent::EngagementResolved { defender: 0, losses_attacker, .. } => {
                    struck[b] += losses_attacker as u64
                }
                _ => {}
            }
        }
        println!("  seed {seed} done");
        let _ = std::io::stdout().flush();
    }
    for b in 2..nb {
        let k = (station[b] / samples[b].max(1) as f64).round() as usize;
        let mut counts: Vec<u64> = per_port[b].values().copied().collect();
        counts.sort_unstable_by(|a, c| c.cmp(a));
        let topk: u64 = counts.iter().take(k * SEEDS.len()).sum();
        println!(
            "{:>4}-{:<4} {:>7} {:>7} {:>8.1}% {:>10.1} {:>10.1} {:>13.1}%",
            b as f64 * BUCKET,
            (b + 1) as f64 * BUCKET,
            launches[b],
            struck[b],
            100.0 * struck[b] as f64 / launches[b].max(1) as f64,
            station[b] / samples[b].max(1) as f64,
            flight[b] / samples[b].max(1) as f64,
            100.0 * topk as f64 / launches[b].max(1) as f64
        );
    }
}
