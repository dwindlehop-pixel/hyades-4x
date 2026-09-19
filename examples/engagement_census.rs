//! **T-111: what does wiring combat into the sim actually do?**
//!
//! `combat::resolve_engagement` is now called from `sim.rs` — the seam
//! `Hyades_warfare_tree.md` §3.4 records as missing, and the thing blocking
//! R-AC13, the belief wiring and every posture card. This prints the three
//! things a landing has to say about it:
//!
//! 1. **Does it fire**, and how often — against `examples/contact_census`,
//!    which measured the upstream side (how many sites are shared at all).
//! 2. **What it costs**, in `yr/s` *and* `ns/event` (`CLAUDE.md` §2: an
//!    aggregate rate cannot tell "each event got dearer" from "there are more
//!    events", and combat does both).
//! 3. **What it moves**, on the two objectives the project actually reads —
//!    colony count and colony-years — because a mechanic that halves expansion
//!    is a design decision and not a detail.
//!
//! **Neither arm is a balance measurement.** Nothing in `CombatConfig` was
//! tuned against a galaxy, the laser/missile side assignment is a placeholder
//! (R-WAR5), and Warfare's own objective is unreadable on a symmetric bed
//! (R-TREE8). This is a mechanism census.
//!
//! Run: `cargo run --release --example engagement_census`
use hyades_engine::log::{LogCategory, LogEvent, LogFilter};
use hyades_engine::prelude::*;
use std::io::Write;
use std::time::Instant;

/// The standard four plus four the mechanic was not developed against —
/// `CLAUDE.md` §2's replication rule, which is the last step before any claim
/// about an effect size.
const SEEDS: &[u64] = &[1, 7, 42, 31337, 2, 3, 5, 11];
const PLAYERS: usize = 3;
const HORIZON: f64 = 800.0;

struct Arm {
    colonies: u64,
    colony_years: f64,
    yr_per_s: f64,
    ns_per_event: f64,
    fights: u64,
    kills: u64,
    slag: f64,
    committed: u64,
}

fn run(seed: u64, war: bool) -> Arm {
    let galaxy = Galaxy::generate(GalaxyConfig::new(PLAYERS, seed)).unwrap();
    let doctrine = Doctrine { engage_neutrals: war, ..Doctrine::default() };
    let autopilots: Vec<Box<dyn Autopilot>> =
        (0..PLAYERS).map(|_| Box::new(BaselineAutopilot::new(doctrine)) as Box<_>).collect();
    let mut cfg = SimConfig::new(seed);
    cfg.horizon_years = HORIZON;
    cfg.engagements_enabled = war;
    let mut sim = Simulation::new(galaxy, cfg, autopilots);
    sim.set_log_filter(LogFilter::none().with(LogCategory::Combat).with(LogCategory::Vehicles));

    let t0 = Instant::now();
    let report = sim.run();
    let secs = t0.elapsed().as_secs_f64();

    let (mut fights, mut kills, mut slag, mut committed) = (0u64, 0u64, 0.0f64, 0u64);
    // Colony-years is the integral, accumulated the way `examples/colony_years`
    // does it: each founding contributes the time it stood before the horizon.
    let horizon = sim.clock();
    let (mut colony_years, mut colonies) = (0.0f64, 0u64);
    for r in sim.log().iter() {
        match r.event {
            LogEvent::EngagementResolved { losses_attacker, losses_defender, slag: s, committed: c, .. } => {
                fights += 1;
                kills += (losses_attacker + losses_defender) as u64;
                slag += s;
                if c {
                    committed += 1;
                }
            }
            LogEvent::ColonyFounded { .. } => {
                colony_years += horizon - r.time;
                colonies += 1;
            }
            _ => {}
        }
    }
    Arm {
        colonies,
        colony_years,
        yr_per_s: HORIZON / secs,
        ns_per_event: secs * 1e9 / report.events_processed.max(1) as f64,
        fights,
        kills,
        slag,
        committed,
    }
}

fn main() {
    println!(
        "{:>6}  {:>9}  {:>9}  {:>8}  {:>8}  {:>7}  {:>7}  {:>9}  {:>9}",
        "seed", "dCol%", "dColYr%", "yr/s p->w", "ns/ev p->w", "fights", "kills", "slag kt", "committed%"
    );
    let _ = std::io::stdout().flush();
    let (mut dc, mut dcy) = (Vec::new(), Vec::new());
    for &seed in SEEDS {
        let p = run(seed, false);
        let w = run(seed, true);
        let c = 100.0 * (w.colonies as f64 / p.colonies as f64 - 1.0);
        let y = 100.0 * (w.colony_years / p.colony_years - 1.0);
        dc.push(c);
        dcy.push(y);
        println!(
            "{seed:>6}  {c:>+8.2}%  {y:>+8.2}%  {:>4.0}->{:<4.0}  {:>5.0}->{:<5.0}  {:>7}  {:>7}  {:>9.2}  {:>8.1}%",
            p.yr_per_s,
            w.yr_per_s,
            p.ns_per_event,
            w.ns_per_event,
            w.fights,
            w.kills,
            w.slag,
            100.0 * w.committed as f64 / w.fights.max(1) as f64
        );
        let _ = std::io::stdout().flush();
    }
    for (label, v) in [("colonies", &dc), ("colony-years", &dcy)] {
        let n = v.len() as f64;
        let mean = v.iter().sum::<f64>() / n;
        // Standard error of the mean: the project reports every effect with one,
        // and anything inside 2 SE of zero is not a finding (`CLAUDE.md` §2).
        let var = v.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0);
        let se = (var / n).sqrt();
        let pos = v.iter().filter(|x| **x > 0.0).count();
        println!(
            "{label:>14}: {mean:+.2}% +/- {se:.2} ({:.1} SE), {pos}/{} seeds positive",
            (mean / se).abs(),
            v.len()
        );
    }
}
