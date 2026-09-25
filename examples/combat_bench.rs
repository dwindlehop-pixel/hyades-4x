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
use hyades_engine::log::{CourseReason, LogCategory, LogEvent, LogFilter};
use hyades_engine::sim::{Role, SimConfig, Simulation};
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
        // Encounters begun, hulls wrecked by role, and course changes by
        // reason (T-133): what the event-loop fight model did.
        let (mut encounters, mut wrecked, mut colony_wrecks, mut retargets, mut withdrawals) =
            (0usize, 0usize, 0usize, 0usize, 0usize);
        let mut by_role: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
        for r in sim.log().iter() {
            match r.event {
                LogEvent::EncounterBegan { .. } => encounters += 1,
                LogEvent::HullWrecked { role, .. } => {
                    wrecked += 1;
                    *by_role.entry(format!("{role:?}")).or_insert(0usize) += 1;
                    if role == Role::Colonizer {
                        colony_wrecks += 1;
                    }
                }
                LogEvent::CourseChanged { reason: CourseReason::Retarget, .. } => retargets += 1,
                LogEvent::CourseChanged { reason: CourseReason::Withdraw, .. } => withdrawals += 1,
                _ => {}
            }
        }
        let colonies: usize = sim.report().players.iter().map(|p| p.colonies).sum();
        println!(
            "seed {seed}: {:>7.2} yr/s  {:>9} events  {:>7.0} ns/event  {encounters} encounters  \
             {wrecked} wrecked ({colony_wrecks} colony ships)  {retargets} retargets  {withdrawals} withdrawals  \
             {colonies} colonies",
            h / s,
            events,
            s * 1e9 / events as f64
        );
        println!("  wrecked by role: {by_role:?}");
        let _ = std::io::stdout().flush();
    }
}
