//! **T-112: does holding ground actually curb a neighbour's expansion?**
//!
//! The author's stated expectation, written down before the measurement so it
//! can refuse the change (`CLAUDE.md` §2): *"I expect to greatly decrease the
//! expansion of the Warfare player's neighbors. Some blood shed is expected,
//! but the goal is curbing the neighbour's growth."*
//!
//! So the acceptance criterion is **not** kills and **not** the player's own
//! colony count. It is the split:
//!
//! - **seat 0 plays the card**; seats 1..n are the neighbours at default doctrine;
//! - the number to watch is **neighbour colonies**, which must fall;
//! - **`W_0 = C_0 − mean(C_j)`** is Warfare's own objective at equal weights
//!   (`Hyades_warfare_tree.md` §0 with `w_ij` uniform — R-WAR3 is unset, and a
//!   uniform weight is the only defensible placeholder on a bed whose
//!   homeworlds are equilateral, trees §2.3.2);
//! - **the player's own colonies are expected to fall too**, because a
//!   coloniser that leaves is a colony founded without its `Band I` stock
//!   (roles §4.2). A card that cost its player nothing would be a different
//!   card.
//!
//! **This is an asymmetric bed** — one seat carrying the card, the rest at the
//! default — which is what R-TREE8 says Warfare requires and what every other
//! measurement in this project does not do.
//!
//! Run: `cargo run --release --example denial_census`
use hyades_engine::log::{LogCategory, LogEvent, LogFilter};
use hyades_engine::prelude::*;
use std::io::Write;

const SEEDS: &[u64] = &[1, 7, 42, 31337, 2, 3, 5, 11];
const PLAYERS: usize = 3;
const HORIZON: f64 = 800.0;

struct Arm {
    own: f64,
    neighbours: f64,
    kills: u64,
    diverts: u64,
    pickets: u64,
}

fn run(seed: u64, card: bool) -> Arm {
    let galaxy = Galaxy::generate(GalaxyConfig::new(PLAYERS, seed)).unwrap();
    // Seat 0 alone plays the card.
    let autopilots: Vec<Box<dyn Autopilot>> = (0..PLAYERS)
        .map(|i| {
            let d = Doctrine {
                engage_neutrals: card && i == 0,
                picket_after_founding: card && i == 0,
                ..Doctrine::default()
            };
            Box::new(BaselineAutopilot::new(d)) as Box<_>
        })
        .collect();
    let mut cfg = SimConfig::new(seed);
    cfg.horizon_years = HORIZON;
    cfg.engagements_enabled = card;
    cfg.picket_keeps_founding_infra = std::env::var("KEEP_INFRA").is_ok();
    let mut sim = Simulation::new(galaxy, cfg, autopilots);
    sim.set_log_filter(LogFilter::none().with(LogCategory::Combat).with(LogCategory::Vehicles));
    let report = sim.run();

    let (mut kills, mut diverts, mut pickets) = (0u64, 0u64, 0u64);

    for r in sim.log().iter() {
        match r.event {
            LogEvent::EngagementResolved { losses_attacker, losses_defender, .. } => {
                kills += (losses_attacker + losses_defender) as u64;
            }
            // **Only picket diversions count.** `ColonyContested` also fires
            // for losing a race, which is a baseline behaviour and swamps this.
            LogEvent::ColonyDiverted { .. } => diverts += 1,
            LogEvent::VehicleParked { role: Role::Picket, .. } => pickets += 1,
            _ => {}
        }
    }
    let own = report.players[0].colonies as f64;
    let neighbours = report.players[1..].iter().map(|p| p.colonies as f64).sum::<f64>() / (PLAYERS - 1) as f64;
    Arm { own, neighbours, kills, diverts, pickets }
}

fn stat(label: &str, v: &[f64]) {
    let n = v.len() as f64;
    let mean = v.iter().sum::<f64>() / n;
    let var = v.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0);
    let se = (var / n).sqrt();
    let neg = v.iter().filter(|x| **x < 0.0).count();
    println!(
        "{label:>22}: {mean:+.2}% +/- {se:.2} ({:.1} SE), {neg}/{} seeds negative",
        (mean / se.max(1e-12)).abs(),
        v.len()
    );
}

fn main() {
    println!(
        "{:>6}  {:>9}  {:>9}  {:>9}  {:>9}  {:>7}  {:>8}  {:>8}",
        "seed", "own p->c", "nbr p->c", "d-own%", "d-nbr%", "kills", "diverts", "pickets"
    );
    let _ = std::io::stdout().flush();
    let (mut d_own, mut d_nbr, mut w_gain) = (Vec::new(), Vec::new(), Vec::new());
    let (mut spent, mut denied) = (Vec::new(), Vec::new());
    for &seed in SEEDS {
        let p = run(seed, false);
        let c = run(seed, true);
        let do_ = 100.0 * (c.own / p.own - 1.0);
        let dn = 100.0 * (c.neighbours / p.neighbours - 1.0);
        d_own.push(do_);
        d_nbr.push(dn);
        // **The trade the card is actually making**, in colonies: what seat 0
        // gave up against what the neighbours lost. This is the ratio the whole
        // design turns on — see the write-up.
        spent.push(p.own - c.own);
        denied.push((p.neighbours - c.neighbours) * (PLAYERS - 1) as f64);
        // Warfare's objective at uniform w_ij, as a change in colonies.
        w_gain.push((c.own - c.neighbours) - (p.own - p.neighbours));
        println!(
            "{seed:>6}  {:>4.0}->{:<4.0}  {:>4.0}->{:<4.0}  {do_:>+8.2}%  {dn:>+8.2}%  {:>7}  {:>8}  {:>8}",
            p.own, c.own, p.neighbours, c.neighbours, c.kills, c.diverts, c.pickets
        );
        let _ = std::io::stdout().flush();
    }
    stat("own colonies", &d_own);
    stat("neighbour colonies", &d_nbr);
    let tot_spent: f64 = spent.iter().sum();
    let tot_denied: f64 = denied.iter().sum();
    println!(
        "{:>22}: seat 0 gave up {tot_spent:.0} colonies to deny {tot_denied:.0} — ratio {:.3} denied per spent",
        "the trade",
        tot_denied / tot_spent.abs().max(1e-9)
    );
    let n = w_gain.len() as f64;
    let m = w_gain.iter().sum::<f64>() / n;
    let var = w_gain.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (n - 1.0);
    let se = (var / n).sqrt();
    println!(
        "{:>22}: {m:+.1} colonies +/- {se:.1} ({:.1} SE), {}/{} seeds positive",
        "W_0 (uniform w_ij)",
        (m / se.max(1e-12)).abs(),
        w_gain.iter().filter(|x| **x > 0.0).count(),
        w_gain.len()
    );
}
