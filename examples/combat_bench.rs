//! **Release-binary throughput on the standard twelve-seat bed, with shots
//! fired** (T-126). `combat_bench [horizon] [seed,seed,...]`.
//!
//! The bed `examples/card_table` measures cards on: twelve seats, the Warfare
//! card on even seats and the Growth card on odd seats at the round-0 barrier,
//! engagements on — so the run includes blockades striking colony ships. It is
//! not simulated to completion: the default horizon, 450 yr, is 250 years past
//! the barrier, which covers the first strikes and the rivals' expansion peak.
//!
//! Prints `yr/s` and `ns/event` side by side (`CLAUDE.md` §2: a rate over a
//! population the change re-selects cannot tell "more work" from "dearer work"),
//! and the fight count, so a knob that changed the simulation rather than its
//! speed shows up as a different count rather than as a speedup.

use hyades_engine::autopilot::{Autopilot, BaselineAutopilot, Doctrine};
use hyades_engine::cards::{CardId, Order, Target};
use hyades_engine::galaxy::{Galaxy, GalaxyConfig, PlayerId};
use hyades_engine::log::{LogCategory, LogEvent, LogFilter};
use hyades_engine::sim::{SimConfig, Simulation};
use std::io::Write;

const SEATS: usize = 12;

fn main() {
    let h: f64 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(450.0);
    let seeds: Vec<u64> = match std::env::args().nth(2) {
        Some(v) => v.split(',').filter_map(|s| s.trim().parse().ok()).collect(),
        None => vec![1, 7],
    };
    for seed in seeds {
        let galaxy = Galaxy::generate(GalaxyConfig::new(SEATS, seed)).unwrap();
        let aps: Vec<Box<dyn Autopilot>> =
            (0..SEATS).map(|_| Box::new(BaselineAutopilot::new(Doctrine::default())) as Box<_>).collect();
        let mut cfg = SimConfig::new(seed);
        cfg.horizon_years = h;
        cfg.engagements_enabled = true;
        let play_at = cfg.years_to_first_round;
        let mut sim = Simulation::new(galaxy, cfg, aps);
        // Combat only: the count is the check that shots were fired, and one
        // category keeps the log's cost out of the timing (tests/telemetry.rs).
        sim.set_log_filter(LogFilter::none().with(LogCategory::Combat));
        let t0 = std::time::Instant::now();
        let mut played = false;
        while sim.step() {
            if !played && sim.clock() >= play_at {
                let orders: Vec<Order> = (0..SEATS)
                    .map(|i| Order {
                        seat: PlayerId(i as u32),
                        card: Some(CardId(if i % 2 == 0 { 15 } else { 3 })),
                        target: Target::None,
                    })
                    .collect();
                sim.apply_orders(sim.current_round(), &orders);
                played = true;
            }
        }
        let s = t0.elapsed().as_secs_f64();
        let events = sim.report().events_processed;
        let fights = sim.log().iter().filter(|r| matches!(r.event, LogEvent::EngagementResolved { .. })).count();
        println!(
            "seed {seed}: {:>7.2} yr/s  {:>9} events  {:>7.0} ns/event  {fights} fights",
            h / s,
            events,
            s * 1e9 / events as f64
        );
        let _ = std::io::stdout().flush();
    }
}
