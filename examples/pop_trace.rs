//! Scratch: homeworld population at a horizon, as a function of `cycle_years`.
//! Isolates the integrator from the expansion loop — one world, read directly.
use hyades_engine::prelude::*;

fn main() {
    let horizon: f64 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(300.0);
    println!("homeworld population at {horizon:.0} yr, seed 1, 3 seats");
    println!("{:>8} {:>14} {:>10}", "cycle", "population", "vs finest");
    let mut rows = Vec::new();
    for cycle in [50.0, 25.0, 10.0, 5.0, 1.0, 0.25] {
        let mut cfg = SimConfig::new(1);
        cfg.horizon_years = horizon;
        cfg.cycle_years = cycle;
        let mut sim = Simulation::with_baseline(Galaxy::generate(GalaxyConfig::new(3, 1)).unwrap(), cfg);
        sim.run();
        let pop: f64 = sim
            .snapshot()
            .planets
            .iter()
            .filter(|p| p.is_homeworld && p.owner == Some(0))
            .map(|p| p.population.kilotons())
            .sum();
        rows.push((cycle, pop));
    }
    let finest = rows.last().unwrap().1;
    for (c, p) in &rows {
        println!("{c:>8.2} {p:>14.2} {:>9.3}x", p / finest);
    }
}
