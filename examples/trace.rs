//! Interrogation demo — uses [`hyades_engine::log`] to answer a concrete
//! question raised by the `experiments` sweep: at `reinvest_bias = 1.0`,
//! `mean colonies` collapsed to 0 across every seed. Is that a starved-economy
//! soft-lock (a center endlessly saving for infrastructure it can never quite
//! afford) or something else?
//!
//! Run with:  `cargo run --example trace`

use hyades_engine::autopilot::{BaselineAutopilot, BuildOrder};
use hyades_engine::log::{LogCategory, LogEvent, LogFilter};
use hyades_engine::prelude::*;

fn main() {
    let players = 3;
    let seed = 1;
    let galaxy = Galaxy::generate(GalaxyConfig::new(players, seed)).unwrap();

    // "always deepen first" — the setting under suspicion
    let doctrine = Doctrine { reinvest_bias: 1.0, ..Doctrine::default() };
    let autopilots: Vec<Box<dyn hyades_engine::autopilot::Autopilot>> =
        (0..players).map(|_| Box::new(BaselineAutopilot::new(doctrine)) as Box<_>).collect();

    let mut sim = Simulation::new(galaxy, SimConfig::new(seed), autopilots);
    sim.set_log_filter(LogFilter::none().with(LogCategory::Production).with(LogCategory::Mining));
    let report = sim.run();

    println!("reinvest_bias = 1.0, seed = {seed}, {players} seats, horizon = {:.0} yr\n", sim.clock());
    for (p, r) in report.players.iter().enumerate() {
        println!(
            "  P{p}: planets={} colonies={} outposts={} pop={:.2}",
            r.planets_owned, r.colonies, r.mining_outposts, r.total_population
        );
    }

    // Focus on player 0's homeworld production history.
    let decisions: Vec<_> =
        sim.log().by_player(0).filter(|r| matches!(r.event, LogEvent::ProductionDecision { .. })).collect();
    let idle = decisions
        .iter()
        .filter(|r| matches!(r.event, LogEvent::ProductionDecision { chosen: BuildOrder::Idle, .. }))
        .count();
    println!(
        "\nP0 production cycles: {} total, {idle} idle ({:.0}%)",
        decisions.len(),
        100.0 * idle as f64 / decisions.len().max(1) as f64
    );

    println!("\nFirst 5 production decisions:");
    for r in decisions.iter().take(5) {
        println!("  {r}");
    }
    println!("\nLast 5 production decisions:");
    for r in decisions.iter().rev().take(5).collect::<Vec<_>>().into_iter().rev() {
        println!("  {r}");
    }

    // Total lifetime mineable endowment vs. what full deepening would cost, to
    // check the "finite resource, insufficient to ever fully deepen" theory.
    if let Some(LogEvent::ProductionDecision { infra_cost, k_potential, infra, .. }) = decisions.last().map(|r| r.event)
    {
        println!("\nAt end: infra={infra:.2} / k_potential={k_potential:.2}, next upgrade would cost {infra_cost:.2}");
    }
}
