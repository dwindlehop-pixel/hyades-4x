//! **What the engine spends its events on** — a per-kind census, so a throughput
//! change can be attributed to *which* work got more expensive rather than to
//! the aggregate.
//!
//! `CLAUDE.md` §2 says to read `yr/s` and `ns/event` together, and the row that
//! matters here is "`yr/s` down, `ns/event` up": events got dearer. But that
//! diagnosis stops one step short of actionable — it does not say *which*
//! events. T-88 moved a saving centre's build decision off the economy tick and
//! onto the freight arrival that changes its situation; the scheduled-event
//! count barely moved (−1.8%) while throughput fell 17%, because the decision
//! is now done **inside** `FreighterArrive` and is charged to it.
//!
//! So this counts the expensive work directly:
//!
//! - **decisions** — `ProductionDecision` records, the thing T-88 moved.
//! - **candidate scans** — the same count, since every decision runs one; this
//!   is T-52's `O(scanned)` loop and the largest loop left in the engine.
//! - **builds applied**, **ticks**, **freight deposits** — the populations a
//!   decision count has to be read against.
//!
//! The ratio that matters is **decisions per simulated year**, because that is
//! what `cycle_years` used to set and what T-88 severed.
//!
//! Run: `cargo run --release --example decision_census -- [seed] [horizon] [cycle_years]`
use hyades_engine::log::{LogCategory, LogEvent, LogFilter};
use hyades_engine::prelude::*;
use std::io::Write;

const PLAYERS: usize = 3;

fn main() {
    let mut a = std::env::args().skip(1);
    let seed: u64 = a.next().and_then(|s| s.parse().ok()).unwrap_or(1);
    let horizon: f64 = a.next().and_then(|s| s.parse().ok()).unwrap_or(1500.0);
    let cycle: Option<f64> = a.next().and_then(|s| s.parse().ok());

    let galaxy = Galaxy::generate(GalaxyConfig::new(PLAYERS, seed)).unwrap();
    let mut cfg = SimConfig::new(seed);
    cfg.horizon_years = horizon;
    if let Some(c) = cycle {
        cfg.cycle_years = c;
    }
    println!("decision census — seed {seed}, {PLAYERS} seats, {horizon:.0} yr, cycle {:.1}", cfg.cycle_years);
    std::io::stdout().flush().ok();

    let mut sim = Simulation::with_baseline(galaxy, cfg);
    sim.set_log_filter(LogFilter::none().with(LogCategory::Production).with(LogCategory::Population));
    let t0 = std::time::Instant::now();
    sim.run();
    let secs = t0.elapsed().as_secs_f64().max(1e-9);

    let (mut decisions, mut idle, mut builds, mut pop_steps) = (0u64, 0u64, 0u64, 0u64);
    let mut idle_why = [0u64; 5];
    let mut idle_stock = [0.0f64; 5];
    let mut scanned_seen = 0u64;
    for rec in sim.log().iter() {
        match rec.event {
            LogEvent::ProductionDecision {
                chosen,
                candidates_seen,
                pop_level,
                infra,
                k_potential,
                can_afford_infra,
                stockpile,
                ..
            } => {
                decisions += 1;
                scanned_seen += candidates_seen as u64;
                if matches!(chosen, hyades_engine::autopilot::BuildOrder::Idle) {
                    idle += 1;
                    // **Why it idled, off the decision's own logged inputs**
                    // rather than a reconstruction. The first three are the
                    // gates `production_choice` tests in order, so a decision is
                    // attributed to the first one that would have stopped it —
                    // which is what makes the buckets add up.
                    let deepen_possible = infra < k_potential - 1e-9;
                    let below_tier = pop_level < hyades_engine::units::BandTier::II;
                    let b = if !deepen_possible && below_tier {
                        0 // capped and below the build tier: money cannot help
                    } else if below_tier && !can_afford_infra {
                        1 // below the tier, wants to deepen, cannot pay the bill
                    } else if !deepen_possible {
                        2 // capped, above the tier: nothing outward was chosen
                    } else if !can_afford_infra {
                        3 // wants to deepen, colour-short or broke
                    } else {
                        4 // could afford to deepen and still chose nothing
                    };
                    idle_why[b] += 1;
                    idle_stock[b] += stockpile;
                }
            }
            LogEvent::BuildApplied { .. } => builds += 1,
            LogEvent::PopulationStep { .. } => pop_steps += 1,
            _ => {}
        }
    }
    let rep = sim.report();
    let snap = sim.snapshot();
    let colonies = snap.planets.iter().filter(|p| p.owner.is_some()).count();
    println!("  {:>12} {:>12} {:>12} {:>12} {:>12}", "decisions", "idle", "builds", "econ ticks", "colonies");
    println!("  {decisions:>12} {idle:>12} {builds:>12} {pop_steps:>12} {colonies:>12}");
    println!(
        "\n  decisions/yr {:.1}   decisions/colony {:.2}   idle share {:.1}%",
        decisions as f64 / horizon,
        decisions as f64 / colonies.max(1) as f64,
        100.0 * idle as f64 / decisions.max(1) as f64
    );
    // **The scan, not the decision, is the cost.** Every decision walks the
    // player's scanned set; `candidates_seen` is that walk's length, so this
    // product is the engine's largest loop measured rather than estimated.
    println!(
        "  candidate-scan steps {scanned_seen}  ({:.0} per decision)",
        scanned_seen as f64 / decisions.max(1) as f64
    );
    const WHY: [&str; 5] = [
        "capped AND below build tier (money cannot help)",
        "below tier, wants to deepen, cannot pay the bill",
        "capped, above tier, chose nothing outward",
        "wants to deepen, cannot pay the bill",
        "could afford to deepen and chose nothing anyway",
    ];
    println!("\n  why {idle} decisions idled:");
    for (i, w) in WHY.iter().enumerate() {
        if idle_why[i] > 0 {
            println!(
                "    {:>7} ({:>4.1}%)  mean bank {:>9.2} kt   {w}",
                idle_why[i],
                100.0 * idle_why[i] as f64 / idle.max(1) as f64,
                idle_stock[i] / idle_why[i] as f64
            );
        }
    }
    println!(
        "\n  events {}  {:.1} yr/s  {:.0} ns/event  wall {:.1}s",
        rep.events_processed,
        horizon / secs,
        secs * 1.0e9 / rep.events_processed.max(1) as f64,
        secs
    );
}
