//! **Does the simulation ever put two empires in the same place?**
//!
//! The precondition for wiring `combat::resolve_engagement` into the sim
//! (T-30/T-52, `Hyades_warfare_tree.md` §3.4). `CLAUDE.md` §2: *check whether the
//! thing upstream was ever short* — an engagement trigger keyed on co-location is
//! worth nothing if co-location never happens, and one census answers that before
//! any code is written.
//!
//! Colonization is **exclusive** by default (R-V3), so colony worlds cannot host
//! two owners. Mining is **non-exclusive** (roles §4.3), so a mining outpost is
//! the one site in the shipped engine where two empires' hulls can legitimately
//! stand together. This counts them.
//!
//! Run: `cargo run --release --example contact_census`
use hyades_engine::log::{LogCategory, LogEvent, LogFilter};
use hyades_engine::prelude::*;
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;

const SEEDS: &[u64] = &[1, 7];
const PLAYERS: usize = 3;
const HORIZON: f64 = 1500.0;

fn main() {
    println!("{:>5} {:>8} {:>10} {:>10} {:>9} {:>10}", "seed", "sites", "shared", "share%", "maxplrs", "contacts");
    let _ = std::io::stdout().flush();
    for &seed in SEEDS {
        let galaxy = Galaxy::generate(GalaxyConfig::new(PLAYERS, seed)).unwrap();
        let autopilots: Vec<Box<dyn Autopilot>> =
            (0..PLAYERS).map(|_| Box::new(BaselineAutopilot::new(Doctrine::default())) as Box<_>).collect();
        let mut cfg = SimConfig::new(seed);
        cfg.horizon_years = HORIZON;
        let mut sim = Simulation::new(galaxy, cfg, autopilots);
        sim.set_log_filter(LogFilter::none().with(LogCategory::Vehicles));
        sim.run();

        // Which players ever parked a hull at each site, in arrival order, so a
        // "contact" is a park that finds someone else's hull already there.
        let mut occupants: BTreeMap<u32, BTreeSet<u32>> = BTreeMap::new();
        let mut contacts = 0u64;
        for r in sim.log().iter() {
            if let LogEvent::VehicleParked { player, at, .. } = r.event {
                let e = occupants.entry(at.0).or_default();
                if !e.is_empty() && !e.contains(&player) {
                    contacts += 1;
                }
                e.insert(player);
            }
        }
        let sites = occupants.len();
        let shared = occupants.values().filter(|s| s.len() > 1).count();
        let maxp = occupants.values().map(|s| s.len()).max().unwrap_or(0);
        println!(
            "{seed:>5} {sites:>8} {shared:>10} {:>9.2}% {maxp:>9} {contacts:>10}",
            100.0 * shared as f64 / sites.max(1) as f64
        );
        let _ = std::io::stdout().flush();
    }
}
