//! Throughput probe: yr/s and ns/event on the standard bed.
use hyades_engine::prelude::*;
fn main() {
    let h: f64 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(800.0);
    for seed in [1u64, 7] {
        let mut cfg = SimConfig::new(seed);
        cfg.horizon_years = h;
        let mut sim = Simulation::with_baseline(Galaxy::generate(GalaxyConfig::new(3, seed)).unwrap(), cfg);
        let t0 = std::time::Instant::now();
        let r = sim.run();
        let s = t0.elapsed().as_secs_f64();
        let col: usize = r.players.iter().map(|p| p.colonies).sum();
        println!(
            "seed {seed}: {:>6.1} yr/s  {:>8} events  {:>7.0} ns/event  {col} colonies  pop {:.6}",
            h / s,
            r.events_processed,
            s * 1e9 / r.events_processed as f64,
            r.players.iter().map(|p| p.total_population.kilotons()).sum::<f64>()
        );
    }
}
