//! **Where the minerals actually go, once the founding subsidy is gone.**
//!
//! Removing the subsidy (T-60) cost half the bed: 1,813 colonies against 3,481.
//! The intended response is freight — minerals to low-infrastructure colonies,
//! then population to colonies with headroom. Before building either, this asks
//! the question `CLAUDE.md` §2 insists on: **is the thing being optimised what
//! is actually scarce?**
//!
//! `mineral_pressure_of` is `1 − stock/next_rung_price`, and a freighter picks
//! its destination by that, discounted by transit. A newly founded colony has
//! no stock, so its pressure is already **1.0 — the maximum**. Existing routing
//! therefore *already* prefers exactly the colonies the new freight is meant to
//! help, which means colony-to-colony hauling may not be the binding
//! constraint at all.
//!
//! Run: `cargo run --release --example supply_census`
use hyades_engine::prelude::*;
use hyades_engine::units::Measure;
use std::io::Write;

const SEEDS: &[u64] = &[1, 7];
const PLAYERS: usize = 3;

fn main() {
    for &seed in SEEDS {
        let galaxy = Galaxy::generate(GalaxyConfig::new(PLAYERS, seed)).unwrap();
        let autopilots: Vec<Box<dyn Autopilot>> =
            (0..PLAYERS).map(|_| Box::new(BaselineAutopilot::new(Doctrine::default())) as Box<_>).collect();
        let mut sim = Simulation::new(galaxy, SimConfig::new(seed), autopilots);
        sim.run();

        let snap = sim.snapshot();
        let owned: Vec<_> = snap.planets.iter().filter(|p| p.owner.is_some()).collect();

        // How far up the infrastructure ladder did colonies actually get?
        let mut buckets = [0usize; 5];
        let mut stalled_at_founding = 0usize;
        let mut sum_infra = 0.0;
        let mut sum_pop = 0.0;
        let mut headroom_kt = 0.0;
        for p in &owned {
            let infra = p.infrastructure.bands();
            sum_infra += infra;
            sum_pop += p.population.bands();
            buckets[(infra.floor() as usize).min(4)] += 1;
            // A colony still at what its own hull left behind has never been
            // supplied at all.
            if infra < 0.5 {
                stalled_at_founding += 1;
            }
            // Room to grow, as a mass: what `K` allows minus who is there.
            let k = p.habitability.min(p.biosphere).min(p.infrastructure);
            let gap = p.population.gap_to(k).kilotons();
            if gap > 0.0 {
                headroom_kt += gap;
            }
        }
        let n = owned.len().max(1) as f64;
        println!(
            "seed {seed:>6}: colonies={:<6} mean_infra={:.3} mean_pop={:.3}\n\
             {:>14}infra floor buckets [<1, 1-2, 2-3, 3-4, 4+] = {:?}\n\
             {:>14}still at founding infra (<0.5): {} ({:.1}%)\n\
             {:>14}unfilled population headroom: {:.0} kt across the empire",
            owned.len(),
            sum_infra / n,
            sum_pop / n,
            "",
            buckets,
            "",
            stalled_at_founding,
            100.0 * stalled_at_founding as f64 / n,
            "",
            headroom_kt,
        );
        std::io::stdout().flush().unwrap();
    }
}
