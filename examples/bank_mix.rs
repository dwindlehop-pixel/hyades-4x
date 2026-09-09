//! **Are a centre's stockpiles even across the colours?**
//!
//! T-73's ablation (per-colour gate, proportional payment) reproduced the full
//! colour-billed run *bit-identically*, which says the payment scheme changes
//! nothing. That is only possible if every bank paying a works bill already has
//! the composition the bill asks for — so this checks the composition directly
//! rather than reasoning about it.
//!
//! Run: `cargo run --release --example bank_mix`
use hyades_engine::autopilot::BuildOrder;
use hyades_engine::prelude::*;
use std::io::Write;

fn main() {
    let seed = 1u64;
    let galaxy = Galaxy::generate(GalaxyConfig::new(3, seed)).unwrap();
    let mut cfg = SimConfig::new(seed);
    cfg.horizon_years = 800.0;
    let mut sim = Simulation::with_baseline(galaxy, cfg);
    sim.set_log_filter(LogFilter::none().with(LogCategory::Production));
    sim.run();

    // **How often does a works bill actually get paid?** If the per-colour gate
    // refuses nearly all of them, `pay_bill` barely runs — and that alone would
    // explain why swapping the payment scheme changed nothing.
    let (mut upgrades, mut hulls, mut idle_decisions, mut decisions) = (0usize, 0usize, 0usize, 0usize);
    for r in sim.log().iter() {
        match r.event {
            LogEvent::BuildApplied { order: BuildOrder::UpgradeInfrastructure, .. } => upgrades += 1,
            LogEvent::BuildApplied { order: BuildOrder::Hull { .. }, .. } => hulls += 1,
            LogEvent::ProductionDecision { chosen, .. } => {
                decisions += 1;
                if chosen == BuildOrder::Idle {
                    idle_decisions += 1;
                }
            }
            _ => {}
        }
    }
    println!("builds applied: {upgrades} infrastructure, {hulls} hulls");
    println!("decisions: {decisions} total, {idle_decisions} Idle");
    std::io::stdout().flush().ok();
    println!("bank composition at t=800, seed {seed}");
    std::io::stdout().flush().ok();

    let snap = sim.snapshot();
    let (mut n, mut even, mut skewed) = (0usize, 0usize, 0usize);
    let mut worst = (0.0f64, 0usize);
    for p in snap.planets.iter().filter(|p| p.owner.is_some()) {
        let s = p.stockpile;
        let t = s.cyan + s.magenta + s.yellow;
        if t <= 1e-9 {
            continue;
        }
        n += 1;
        let (c, m, y) = (s.cyan / t, s.magenta / t, s.yellow / t);
        // Max deviation from an even third.
        let dev =
            [(c - 1.0 / 3.0).abs(), (m - 1.0 / 3.0).abs(), (y - 1.0 / 3.0).abs()].into_iter().fold(0.0f64, f64::max);
        if dev < 1e-12 {
            even += 1;
        } else {
            skewed += 1;
        }
        if dev > worst.0 {
            worst = (dev, p.id.0 as usize);
        }
    }
    println!("  {n} non-empty banks: {even} exactly even, {skewed} skewed");
    println!("  worst deviation from an even third: {:.6} (planet {})", worst.0, worst.1);
    // Show a few so the shape is visible rather than summarised away.
    for p in snap.planets.iter().filter(|p| p.owner.is_some()).take(60) {
        let s = p.stockpile;
        let t = s.cyan + s.magenta + s.yellow;
        if t > 1e-6 {
            println!("  planet {:>5}: C {:>10.4}  M {:>10.4}  Y {:>10.4}", p.id.0, s.cyan, s.magenta, s.yellow);
        }
    }
}
