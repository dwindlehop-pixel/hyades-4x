//! **What a colony is actually founded with** (T-113) — the distribution of the
//! founding infrastructure stock, and what set it.
//!
//! `Doctrine::founding_infra_share` erects part of a colonizer's hold as the new
//! colony's stock instead of banking it, and measured against the objective it
//! is flat (`examples/denial_census`: −8.33% against −8.30% own colonies, −119
//! against −120 on `W_0`). Flat against an objective is not a mechanism, and
//! `AGENTS.md` §2's rule for it is to **measure how much of the time the
//! resource the knob buys is even binding** — here, how often the erected share
//! is larger than the floor rung it has to beat to matter at all.
//!
//! Three arms on one seed, printing the distribution rather than a mean, because
//! a mean over a stock that is pinned at a floor most of the time says nothing
//! about either population.

use hyades_engine::autopilot::{Autopilot, BaselineAutopilot, Doctrine};
use hyades_engine::galaxy::{Galaxy, GalaxyConfig};
use hyades_engine::log::{LogCategory, LogEvent, LogFilter};
use hyades_engine::sim::{SimConfig, Simulation};
use std::io::Write;

const PLAYERS: usize = 3;
const HORIZON: f64 = 800.0;

fn run(seed: u64, share: f64) -> Vec<f64> {
    let galaxy = Galaxy::generate(GalaxyConfig::new(PLAYERS, seed)).unwrap();
    let autopilots: Vec<Box<dyn Autopilot>> = (0..PLAYERS)
        .map(|i| {
            let me = i == 0;
            let d = Doctrine {
                engage_neutrals: me,
                picket_after_founding: me,
                founding_infra_share: if me { share } else { 0.0 },
                ..Doctrine::default()
            };
            Box::new(BaselineAutopilot::new(d)) as Box<_>
        })
        .collect();
    let mut cfg = SimConfig::new(seed);
    cfg.horizon_years = HORIZON;
    let mut sim = Simulation::new(galaxy, cfg, autopilots);
    sim.set_log_filter(LogFilter::none().with(LogCategory::Vehicles));
    sim.run();
    sim.log()
        .iter()
        .filter_map(|r| match r.event {
            LogEvent::ColonyFounded { player: 0, infra, .. } => Some(infra),
            _ => None,
        })
        .collect()
}

fn main() {
    let cfg = SimConfig::new(1);
    let floor = hyades_engine::sim::infra_rung_price(0, &cfg).kilotons();
    println!("floor rung (Band Empty): {floor:.9} kt");
    println!(
        "{:>8}  {:>7}  {:>12}  {:>12}  {:>12}  {:>10}",
        "share", "founded", "median", "mean", "max", "above floor"
    );
    let _ = std::io::stdout().flush();
    for share in [0.0, 0.5, 1.0] {
        let mut v = run(1, share);
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        if v.is_empty() {
            println!("{share:>8.2}  {:>7}", 0);
            continue;
        }
        let median = v[v.len() / 2];
        let mean = v.iter().sum::<f64>() / v.len() as f64;
        let max = *v.last().unwrap();
        let above = v.iter().filter(|&&x| x > floor * 1.000_001).count();
        println!(
            "{share:>8.2}  {:>7}  {median:>12.6}  {mean:>12.6}  {max:>12.6}  {:>9.1}%",
            v.len(),
            100.0 * above as f64 / v.len() as f64
        );
        let _ = std::io::stdout().flush();
    }
}
