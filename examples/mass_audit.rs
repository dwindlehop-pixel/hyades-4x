//! **Where the engine leaks mass** (T-118, design law #11).
//!
//! `Simulation::mass_ledger` weighs every store. Run the same seed to
//! increasing horizons — the run is deterministic, so drift is a function of
//! time — and the first horizon at which a store stops balancing localizes the
//! defect without any guessing about which path it is.

use hyades_engine::autopilot::{Autopilot, BaselineAutopilot, Doctrine};
use hyades_engine::galaxy::{Galaxy, GalaxyConfig};
use hyades_engine::sim::{SimConfig, Simulation};
use std::io::Write;

fn main() {
    let seats = 3;
    let seed = 11;
    println!(
        "{:>8}  {:>14}  {:>10}  {:>10}  {:>10}  {:>10}",
        "horizon", "drift kt", "hulls", "infra", "slag", "banked"
    );
    let _ = std::io::stdout().flush();
    for h in [20.0f64, 24.0, 26.0, 28.0, 30.0, 32.0, 34.0, 36.0, 38.0, 40.0] {
        let galaxy = Galaxy::generate({
            let mut g = GalaxyConfig::new(seats, seed);
            g.planet_count = 200;
            g
        })
        .unwrap();
        let aps: Vec<Box<dyn Autopilot>> =
            (0..seats).map(|_| Box::new(BaselineAutopilot::new(Doctrine::default())) as Box<_>).collect();
        let mut cfg = SimConfig::new(seed);
        cfg.horizon_years = h;
        cfg.biosphere_regen_rate = 0.0;
        let mut sim = Simulation::new(galaxy, cfg, aps);
        let before = sim.mass_ledger();
        sim.run();
        let after = sim.mass_ledger();
        let d = before.delta(&after);
        println!(
            "{h:>8.0}  {:>14.6}  {:>10.4}  {:>10.4}  {:>10.4}  {:>10.2}",
            after.total() - before.total(),
            d.hulls,
            d.infrastructure,
            d.slag,
            d.banked
        );
        let _ = std::io::stdout().flush();
    }
}
