//! Diagnosis for the survey-strategy search: why does "10% colonized" take
//! ~2,800 of 4,000 years (`examples/survey_strategy_search.rs` Round 1),
//! when the same doctrine reaches ~49% by the horizon? If the ramp is that
//! back-loaded, a survey-targeting knob may have no seed left to grow in —
//! "diagnose first, sweep second" (CLAUDE.md §2, the `min_time_search`
//! precedent).
//!
//! Traces one seed's full colonization timeline (bucketed `ColonyFounded`
//! times) and player 0's homeworld production history, to find what's
//! actually gating the opening decades before concluding survey strategy
//! doesn't matter there.
//!
//! Run with:  `cargo run --release --example colonization_ramp_trace`

use std::collections::HashSet;

use hyades_engine::autopilot::{BuildOrder, Doctrine};
use hyades_engine::log::{LogCategory, LogEvent, LogFilter};
use hyades_engine::prelude::*;

fn coverage_targets(galaxy: &Galaxy) -> HashSet<PlanetId> {
    galaxy.planets.iter().filter(|p| p.habitability.min(p.biosphere) > 0.01).map(|p| p.id).collect()
}

fn main() {
    let players = 3;
    let seed = 1;
    let galaxy = Galaxy::generate(GalaxyConfig::new(players, seed)).unwrap();
    let targets = coverage_targets(&galaxy);
    let total = targets.len();

    let doctrine = Doctrine::default();
    let autopilots: Vec<Box<dyn hyades_engine::autopilot::Autopilot>> =
        (0..players).map(|_| Box::new(hyades_engine::autopilot::BaselineAutopilot::new(doctrine)) as Box<_>).collect();

    let cfg = SimConfig::new(seed);
    let mut sim = Simulation::new(galaxy, cfg, autopilots);
    sim.set_log_filter(LogFilter::none().with(LogCategory::Production).with(LogCategory::Vehicles));
    sim.run();

    println!("seed={seed} {players} seats, horizon={:.0} yr, colonizable targets={total}\n", cfg.horizon_years);

    // --- 1. The full colonization timeline, bucketed every 200 yr ---
    let mut founded_times: Vec<f64> = sim
        .log()
        .by_category(LogCategory::Vehicles)
        .filter_map(|r| match r.event {
            LogEvent::ColonyFounded { planet, .. } if targets.contains(&planet) => Some(r.time),
            _ => None,
        })
        .collect();
    founded_times.sort_by(|a, b| a.partial_cmp(b).unwrap());

    println!("=== Colonization timeline (bucketed every 200 yr) ===");
    let bucket = 200.0;
    let mut cumulative = players; // each homeworld counts as founded at t=0
    let mut next_report = bucket;
    let mut idx = 0;
    while next_report <= cfg.horizon_years {
        while idx < founded_times.len() && founded_times[idx] < next_report {
            cumulative += 1;
            idx += 1;
        }
        println!(
            "  t<{next_report:>5.0}: {cumulative:>4}/{total} colonized ({:>5.1}%)",
            100.0 * cumulative as f64 / total as f64
        );
        next_report += bucket;
    }
    let need10 = ((total as f64) * 0.10).ceil() as usize;
    if need10 > players {
        let t10 = founded_times[need10 - players - 1];
        println!("\n  -> crosses 10% ({need10}/{total}) at t={t10:.0} yr");
    }

    // --- 2. Player 0's homeworld: when does it cross the medium_min_level gate? ---
    println!("\n=== Player 0 homeworld production history (first 15 non-idle decisions) ===");
    let home_events: Vec<_> = sim
        .log()
        .by_player(0)
        .filter(|r| matches!(r.event, LogEvent::BuildApplied { .. } | LogEvent::ProductionDecision { .. }))
        .collect();

    let mut shown = 0;
    let mut first_colonizer_build: Option<f64> = None;
    for r in &home_events {
        if let LogEvent::BuildApplied { order, .. } = r.event {
            if shown < 15 {
                println!("  t={:>6.1}  {order:?}", r.time);
                shown += 1;
            }
            if first_colonizer_build.is_none()
                && matches!(order, BuildOrder::Hull { hull_type, .. } if hull_type == HullType::MediumSystems)
            {
                first_colonizer_build = Some(r.time);
            }
        }
    }
    if let Some(t) = first_colonizer_build {
        println!("\n  -> P0 homeworld's first Colonizer (MediumSystems) build applied at t={t:.0} yr");
    } else {
        println!("\n  -> P0 homeworld never built a Colonizer (still below medium_min_level, or never funded)");
    }

    // --- 3. Idle-decision fraction across the whole run, split early vs late ---
    let decisions: Vec<_> =
        sim.log().by_player(0).filter(|r| matches!(r.event, LogEvent::ProductionDecision { .. })).collect();
    let early: Vec<_> = decisions.iter().filter(|r| r.time < 1000.0).collect();
    let idle_early = early
        .iter()
        .filter(|r| matches!(r.event, LogEvent::ProductionDecision { chosen: BuildOrder::Idle, .. }))
        .count();
    println!(
        "\n=== P0 production cycles, t<1000 yr ===\n  {} total, {idle_early} idle ({:.0}%)",
        early.len(),
        100.0 * idle_early as f64 / early.len().max(1) as f64
    );

    println!("\nFirst 10 production decisions (P0 homeworld):");
    for r in decisions.iter().take(10) {
        println!("  {r}");
    }
}
