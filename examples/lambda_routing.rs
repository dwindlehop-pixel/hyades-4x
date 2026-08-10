//! **R-P2 verification: is the Exchange's transit discount `λ` also the right
//! freighter-routing rule?**
//!
//! `Hyades_politics_trade_and_intelligence.md` §2.3 makes one constant do two
//! jobs — on the Exchange, `λ` is the travel-time discount *and* the `$` sink
//! (`seller receives E·exp(−λt)`, the remainder burns). R-P2 ratifies that
//! only on condition that the same constant demonstrably improves the engine's
//! existing problem: a laden freighter today routes to the highest-pressure
//! owned center **with no distance term at all**, so it will happily cross the
//! galaxy for a marginally needier destination.
//!
//! The claim under test: internal haulage is a trade you clear with yourself,
//! so the discount that prices a rival's delivery should price your own colony's.
//!
//! `λ = 0` reduces exactly to `most_needed_center`, so the first row is the
//! shipped baseline and every other row is the same run with one number moved.
//!
//! Run: `cargo run --release --example lambda_routing`

use std::collections::HashSet;
use std::io::Write;

use hyades_engine::prelude::*;

const SEEDS: &[u64] = &[1, 7, 42];
const PLAYERS: usize = 3;

fn coverage_targets(galaxy: &Galaxy) -> HashSet<PlanetId> {
    galaxy.planets.iter().filter(|p| p.habitability.min(p.biosphere) > 0.01).map(|p| p.id).collect()
}

fn run(seed: u64, lambda: f64) -> (usize, usize) {
    let galaxy = Galaxy::generate(GalaxyConfig::new(PLAYERS, seed)).unwrap();
    let targets = coverage_targets(&galaxy);
    let total = targets.len();
    let mut cfg = SimConfig::new(seed);
    cfg.trade_decay_lambda = lambda;
    let mut sim = Simulation::with_baseline(galaxy, cfg);
    sim.run();

    let snap = sim.snapshot();
    let covered = snap.planets.iter().filter(|p| p.owner.is_some() && targets.contains(&p.id)).count();
    (covered, total)
}

fn main() {
    println!("R-P2 — does the Exchange's transit discount also fix freighter routing?");
    println!("{PLAYERS} seats, {} seeds, 4,000 yr. lambda=0 IS `most_needed_center`.\n", SEEDS.len());
    println!("{:>10}  {:>28}  {:>10}", "lambda", "covered (per seed)", "mean %");

    let mut baseline = 0.0;
    // Half-life in years for reference: ln2/lambda. The interesting scale is
    // set by how long a haul takes, which at 1 g over tens of ly is decades.
    for (i, &lambda) in [0.0, 0.002, 0.005, 0.01, 0.02, 0.05].iter().enumerate() {
        let mut cov = Vec::new();
        let mut frac = 0.0;
        for &seed in SEEDS {
            let (c, t) = run(seed, lambda);
            cov.push(c);
            frac += c as f64 / t as f64;
        }
        frac /= SEEDS.len() as f64;
        if i == 0 {
            baseline = frac;
        }
        let delta = if i == 0 { String::new() } else { format!("  ({:+.1} pts)", (frac - baseline) * 100.0) };
        let half_life = if lambda > 0.0 { format!("{:.0} yr", 2f64.ln() / lambda) } else { "inf".into() };
        println!("{lambda:>10.3}  {:>28}  {:>9.2}%{delta}   half-life {half_life}", format!("{cov:?}"), frac * 100.0);
        // One row per expensive trial: flush so a partial run is readable
        // (CLAUDE.md §2 — a partial result you can read beats a complete one
        // you killed).
        std::io::stdout().flush().ok();
    }

    println!(
        "\nReading: lambda > 0 makes a freighter trade need against transit time\n\
         instead of chasing the neediest center at any distance. If no row beats\n\
         lambda=0, the unification R-P2 asks for is not supported by the engine\n\
         and the Exchange discount should be ratified on its own terms only."
    );
}
