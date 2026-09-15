//! Throughput + metric probe on the standard bed: `tput [horizon_years] [seed,seed,...]`.
use hyades_engine::prelude::*;
fn main() {
    let h: f64 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(800.0);
    let seeds: Vec<u64> = match std::env::args().nth(2) {
        Some(v) => v.split(',').filter_map(|s| s.trim().parse().ok()).collect(),
        None => vec![1, 7],
    };
    for seed in seeds {
        let mut cfg = SimConfig::new(seed);
        cfg.horizon_years = h;
        let mut sim = Simulation::with_baseline(Galaxy::generate(GalaxyConfig::new(3, seed)).unwrap(), cfg);
        let t0 = std::time::Instant::now();
        let r = sim.run();
        let s = t0.elapsed().as_secs_f64();
        let col: usize = r.players.iter().map(|p| p.colonies).sum();
        // `ns/event` beside `yr/s`, per CLAUDE.md: a rate over a population the
        // change re-selects cannot tell "more work" from "dearer work".
        println!(
            "seed {seed}: {:>6.1} yr/s  {:>8} events  {:>7.0} ns/event  {col} colonies  pop {:.6}",
            h / s,
            r.events_processed,
            s * 1e9 / r.events_processed as f64,
            r.players.iter().map(|p| p.total_population.kilotons()).sum::<f64>()
        );
    }
}
