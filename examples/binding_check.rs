//! **Is freighter capacity ever the binding constraint?**
//!
//! `gradient_probe` reports `cargo_unit_size` as INERT — a ±10% change moved
//! *no seed at all*. Two explanations, and they call for opposite responses:
//!
//! - The knob is disconnected (a wiring bug) — fix it.
//! - `load = cap.min(avail)` and `avail < cap` always, so capacity never binds
//!   — the knob is real but the economy never reaches it, and the interesting
//!   question moves to why outposts never accumulate a full hold.
//!
//! A local gradient cannot tell these apart, which is the point: it says a knob
//! does not matter *here*, never why. Distinguishing them needs a large step,
//! not a small one, so this sweeps `cargo_unit_size` across two orders of
//! magnitude. If coverage moves at the small end, capacity binds there and the
//! knob is wired; if it never moves, the knob is not connected.
//!
//! Run: `cargo run --release --example binding_check`

use std::collections::HashSet;
use std::io::Write;

use hyades_engine::autopilot::{Autopilot, BaselineAutopilot, Doctrine};
use hyades_engine::prelude::*;

const SEEDS: &[u64] = &[1, 7, 42, 31337];
const PLAYERS: usize = 3;

fn main() {
    println!("Is freighter capacity ever binding? Sweeping cargo_unit_size wide.");
    println!("Default is 5.0; the shell model makes a Medium hull's hold exactly that.\n");
    println!("{:>16}  {:>28}  {:>8}", "cargo_unit_size", "covered (per seed)", "mean");

    for &cus in &[0.05, 0.2, 1.0, 5.0, 25.0, 100.0] {
        let mut covered = Vec::new();
        let mut frac = 0.0;
        for &seed in SEEDS {
            let galaxy = Galaxy::generate(GalaxyConfig::new(PLAYERS, seed)).unwrap();
            let targets: HashSet<PlanetId> =
                galaxy.planets.iter().filter(|p| p.habitability.min(p.biosphere) > 0.01).map(|p| p.id).collect();
            let mut cfg = SimConfig::new(seed);
            cfg.cargo_unit_size = cus;
            let autopilots: Vec<Box<dyn Autopilot>> =
                (0..PLAYERS).map(|_| Box::new(BaselineAutopilot::new(Doctrine::default())) as Box<_>).collect();
            let mut sim = Simulation::new(galaxy, cfg, autopilots);
            sim.run();
            let snap = sim.snapshot();
            let c = snap.planets.iter().filter(|p| p.owner.is_some() && targets.contains(&p.id)).count();
            covered.push(c);
            frac += c as f64;
        }
        frac /= SEEDS.len() as f64;
        println!("{cus:>16.2}  {:>28}  {:>7.2}%", format!("{covered:?}"), frac * 100.0);
        std::io::stdout().flush().ok();
    }

    println!(
        "\nReading: if the small end moves and the large end does not, capacity binds\n\
         only below some threshold and the shipped default sits far above it — the\n\
         knob is wired but slack. If nothing moves anywhere, it is not wired."
    );
}
