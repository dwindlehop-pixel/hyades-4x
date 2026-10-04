//! **How far apart do empires finish?** — the census for the author's
//! target: card-free, the standard deviation of colony count between empires
//! should be about 20 colonies.
//!
//! Standard galaxy, card-free. Per seed and at each sample year, every seat's
//! colony count, and their standard deviation across the seats of that galaxy
//! (the population standard deviation, `sqrt(mean((c - mean)²))`). Then, per
//! sample year, the mean of that standard deviation over the seeds.
//!
//! `CS_GALAXY=<seed>` holds the galaxy fixed while `CS_SEEDS` varies only the
//! simulation's seed. `CS_SYMMETRIC=1` generates the galaxy with `GalaxyConfig::rotational_symmetry`.
//!
//! Run: `cargo run --release --example colony_spread -- <seats> <horizon> [sample years ...]`;
//! `CS_SEEDS=2,3,5,11`. Defaults: 3 seats, 1,500 yr, samples at 200, 400,
//! 800, 1,200 and the horizon.
use hyades_engine::galaxy::{Galaxy, GalaxyConfig};
use hyades_engine::sim::{SimConfig, Simulation};
use std::io::Write;

const SEEDS: [u64; 4] = [1, 7, 42, 31337];

fn seeds() -> Vec<u64> {
    match std::env::var("CS_SEEDS") {
        Ok(v) => v.split(',').filter_map(|s| s.trim().parse().ok()).collect(),
        Err(_) => SEEDS.to_vec(),
    }
}

fn sd(x: &[f64]) -> f64 {
    let m = x.iter().sum::<f64>() / x.len().max(1) as f64;
    (x.iter().map(|v| (v - m) * (v - m)).sum::<f64>() / x.len().max(1) as f64).sqrt()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let seats: usize = args.get(1).and_then(|a| a.parse().ok()).unwrap_or(3);
    let horizon: f64 = args.get(2).and_then(|a| a.parse().ok()).unwrap_or(1500.0);
    let mut samples: Vec<f64> = args.iter().skip(3).filter_map(|a| a.parse().ok()).collect();
    if samples.is_empty() {
        samples = vec![200.0, 400.0, 800.0, 1200.0, horizon];
    }
    samples.retain(|&t| t <= horizon);
    println!("colony_spread: {seats} seats, horizon {horizon} yr, seeds {:?}", seeds());
    std::io::stdout().flush().ok();
    let mut sds: Vec<Vec<f64>> = vec![Vec::new(); samples.len()];
    for seed in seeds() {
        // `CS_SIM_SEED` fixes the galaxy at `CS_GALAXY` and varies only the
        // simulation's own seed.
        let galaxy_seed: u64 = std::env::var("CS_GALAXY").ok().and_then(|v| v.trim().parse().ok()).unwrap_or(seed);
        let mut cfg = SimConfig::new(seed);
        cfg.horizon_years = horizon;
        let mut g = GalaxyConfig::new(seats, galaxy_seed);
        g.rotational_symmetry = std::env::var("CS_SYMMETRIC").is_ok_and(|v| v.trim() == "1");
        let mut sim = Simulation::with_baseline(Galaxy::generate(g).unwrap(), cfg);
        let mut k = 0;
        let take = |sim: &Simulation, k: usize| {
            let c: Vec<f64> = (0..seats).map(|p| sim.tree_stock(p).0 as f64).collect();
            let s = sd(&c);
            println!(
                "{seed:>5} {:>6.0} yr  colonies {:?}  sd {s:.1}",
                samples[k],
                c.iter().map(|v| *v as u64).collect::<Vec<_>>()
            );
            std::io::stdout().flush().ok();
            s
        };
        while sim.step() {
            while k < samples.len() && sim.clock() >= samples[k] {
                sds[k].push(take(&sim, k));
                k += 1;
            }
        }
        while k < samples.len() {
            sds[k].push(take(&sim, k));
            k += 1;
        }
    }
    for (k, t) in samples.iter().enumerate() {
        let m = sds[k].iter().sum::<f64>() / sds[k].len().max(1) as f64;
        println!(
            "{t:>6.0} yr: sd between empires, mean over seeds {m:.1} (per seed {:?})",
            sds[k].iter().map(|v| v.round()).collect::<Vec<_>>()
        );
    }
}
