//! **R-IND12: how much of itself does a production centre send with a colony?**
//!
//! `Doctrine::endowment_fraction` exists because settlers stopped being free.
//! Until R-O74 was closed the founding population was conjured — nothing was
//! debited anywhere — so "how many settlers" was bounded only by the hull's
//! hold and the target's ceiling, and any policy that shipped a bigger seed
//! scored better by exactly the mass it invented (`Hyades_industry.md` §1.6).
//! With the seed drawn from the origin there has to be a rule for how much a
//! parent spends on a child, and the rule is policy.
//!
//! The knob governs **both halves of the hold**: settlers up to this share of
//! the centre's people, then minerals up to this share of its bank. So it trades
//! two things against each other that are not obviously comparable —
//!
//! - **low** keeps the parent growing on its own logistic and founds thin,
//!   slow-starting colonies;
//! - **high** founds colonies that can build immediately but sets the parent
//!   back down its own growth curve, and `x + r·x·(1 − x/K)` is fastest at
//!   `K/2`, so a centre stripped toward the floor regrows *slowly*.
//!
//! Neither direction is obviously right, which is why this is measured rather
//! than argued. Objective is absolute colony **count** (T-20); colony-years is
//! the guard. Common random numbers throughout — the same four seeds for every
//! value, compared seed by seed.
//!
//! Run: `cargo run --release --example endowment`
use hyades_engine::prelude::*;
use std::io::Write;

const SEEDS: [u64; 4] = [1, 7, 42, 31337];
const PLAYERS: usize = 3;
const HORIZON: f64 = 4000.0;
const VALUES: [f64; 5] = [0.05, 0.15, 0.25, 0.50, 0.90];

fn run(seed: u64, fraction: f64) -> (usize, f64) {
    let galaxy = Galaxy::generate(GalaxyConfig::new(PLAYERS, seed)).unwrap();
    let doctrine = Doctrine { endowment_fraction: fraction, ..Default::default() };
    let autopilots: Vec<Box<dyn Autopilot>> =
        (0..PLAYERS).map(|_| Box::new(BaselineAutopilot::new(doctrine)) as Box<_>).collect();
    let mut cfg = SimConfig::new(seed);
    cfg.horizon_years = HORIZON;
    let mut sim = Simulation::new(galaxy, cfg, autopilots);
    sim.set_log_filter(LogFilter::none().with(LogCategory::Vehicles));
    sim.run();

    let (mut colonies, mut colony_years) = (0usize, 0.0);
    for r in sim.log().iter() {
        if let LogEvent::ColonyFounded { .. } = r.event {
            colonies += 1;
            colony_years += HORIZON - r.time;
        }
    }
    (colonies, colony_years)
}

fn main() {
    println!("R-IND12 — endowment_fraction, CRN over {SEEDS:?}, {PLAYERS} seats, {HORIZON:.0} yr");
    println!("objective = colony COUNT; colony-years is the guard\n");
    print!("{:>10}", "fraction");
    for s in SEEDS {
        print!("{:>12}", format!("s{s}"));
    }
    println!("{:>12}{:>16}", "colonies", "colony-yr");
    std::io::stdout().flush().ok();

    // Printed per row and kept, so a run cut short by the container still says
    // something (`CLAUDE.md` §2 — a partial result you can read beats a
    // complete one you killed).
    let mut best: Option<(f64, f64)> = None;
    for f in VALUES {
        let (mut c, mut y) = (0usize, 0.0);
        print!("{f:>10.2}");
        for seed in SEEDS {
            let (ci, yi) = run(seed, f);
            print!("{ci:>12}");
            std::io::stdout().flush().ok();
            c += ci;
            y += yi;
        }
        let n = SEEDS.len() as f64;
        println!("{:>12.1}{:>16.0}", c as f64 / n, y / n);
        std::io::stdout().flush().ok();
        if best.is_none_or(|(_, bc)| c as f64 / n > bc) {
            best = Some((f, c as f64 / n));
        }
        if let Some((bf, bc)) = best {
            println!("{:>10}  best so far: {bf:.2} at {bc:.1} colonies", "");
            std::io::stdout().flush().ok();
        }
    }
}
