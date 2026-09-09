//! **R-IND12: what a coloniser is worth carrying — measured on the objective.**
//!
//! The endowment used to be a *share of the parent* (`Doctrine::endowment_fraction`),
//! and that model was retired by the author's ruling: a fraction is irrelevant.
//! What decides the amount is the destination's carrying capacity against the
//! hull's hold, the build-out the site will actually pay for, and the growth of
//! the **combined** origin-plus-colony system under a travel discount. The
//! derivation, the symbol table and the three limits live on
//! `sim::settler_target`; the limits are pinned by
//! `the_endowment_ships_from_above_the_peak_to_below_it`.
//!
//! What a harness can add that a unit test cannot is the **objective**: whether
//! shipping people from above a logistic peak to below one is worth anything
//! across a whole galaxy, once transit, hull prices and the expansion loop are
//! all in the way. Absolute colony **count** is the objective (T-20);
//! colony-years is the guard.
//!
//! Common random numbers: the same four seeds, compared seed by seed.
//!
//! Run: `cargo run --release --example endowment`
use hyades_engine::prelude::*;
use std::io::Write;

const SEEDS: [u64; 4] = [1, 7, 42, 31337];
const PLAYERS: usize = 3;
const HORIZON: f64 = 4000.0;

fn run(seed: u64) -> (usize, f64, f64) {
    let galaxy = Galaxy::generate(GalaxyConfig::new(PLAYERS, seed)).unwrap();
    let autopilots: Vec<Box<dyn Autopilot>> =
        (0..PLAYERS).map(|_| Box::new(BaselineAutopilot::new(Doctrine::default())) as Box<_>).collect();
    let mut cfg = SimConfig::new(seed);
    cfg.horizon_years = HORIZON;
    let mut sim = Simulation::new(galaxy, cfg, autopilots);
    sim.set_log_filter(LogFilter::none().with(LogCategory::Vehicles));
    sim.run();

    let (mut colonies, mut colony_years, mut sum_t) = (0usize, 0.0, 0.0);
    for r in sim.log().iter() {
        if let LogEvent::ColonyFounded { .. } = r.event {
            colonies += 1;
            colony_years += HORIZON - r.time;
            sum_t += r.time;
        }
    }
    (colonies, colony_years, if colonies > 0 { sum_t / colonies as f64 } else { 0.0 })
}

fn main() {
    println!("R-IND12 — demand-driven endowment, CRN over {SEEDS:?}, {PLAYERS} seats, {HORIZON:.0} yr");
    println!("objective = colony COUNT; colony-years is the guard");
    // **No baseline is hardcoded here on purpose.** A constant copied into a
    // harness is a number that rots silently the next time anything upstream
    // lands — `CLAUDE.md` §7 keeps a stale 456 yr/s row precisely as the
    // reminder. Absolutes go here; the comparison lives in the doc, next to the
    // landing it belongs to.
    println!("absolute figures — compare against the run recorded in Hyades_industry.md §1.7\n");
    println!("{:>8}{:>11}{:>15}{:>13}", "seed", "colonies", "colony-yr", "mean found");
    std::io::stdout().flush().ok();

    let (mut c, mut y, mut m) = (0usize, 0.0, 0.0);
    for seed in SEEDS {
        let (ci, yi, mi) = run(seed);
        println!("{seed:>8}{ci:>11}{yi:>15.0}{mi:>13.1}");
        std::io::stdout().flush().ok();
        c += ci;
        y += yi;
        m += mi;
    }
    let n = SEEDS.len() as f64;
    println!("\n{:>8}{:>11.1}{:>15.0}{:>13.1}", "MEAN", c as f64 / n, y / n, m / n);
}
