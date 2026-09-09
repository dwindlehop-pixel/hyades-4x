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

/// **Which of the three caps actually binds** — the mechanism behind the result.
///
/// `settler_target` is the least of the hull's hold, the destination's capacity
/// and what the origin will part with. Bit-identical colony-years against the
/// model this replaced is a *symptom*; the mechanism is which cap was binding,
/// and if it is the hold on essentially every launch then "fill the hold" is the
/// answer both models give and the identity is arithmetic rather than luck
/// (`CLAUDE.md` §2 — never leave a symptom without a proven mechanism).
///
/// Run on a truncated horizon on purpose: this is a question about the *shape*
/// of launches, not about the objective, and the shape is established long
/// before 4,000 yr.
fn census(seed: u64, horizon: f64) {
    let galaxy = Galaxy::generate(GalaxyConfig::new(PLAYERS, seed)).unwrap();
    let autopilots: Vec<Box<dyn Autopilot>> =
        (0..PLAYERS).map(|_| Box::new(BaselineAutopilot::new(Doctrine::default())) as Box<_>).collect();
    let mut cfg = SimConfig::new(seed);
    cfg.horizon_years = horizon;
    let medium = HullType::MediumSystems.colony_seed_capacity(&cfg);
    let general = HullType::GeneralSystems.colony_seed_capacity(&cfg);
    let mut sim = Simulation::new(galaxy, cfg, autopilots);
    sim.set_log_filter(LogFilter::none().with(LogCategory::Vehicles));
    sim.run();

    let (mut n, mut full_hold, mut all_settlers, mut carried_minerals) = (0usize, 0usize, 0usize, 0usize);
    let (mut sum_s, mut sum_e) = (0.0, 0.0);
    for r in sim.log().iter() {
        if let LogEvent::VehicleSpawned { role: Role::Colonizer, settlers, endowment, .. } = r.event {
            n += 1;
            sum_s += settlers;
            sum_e += endowment;
            // Which hull flew is not in the record, so test against both holds.
            let hold = if (settlers + endowment) > medium.kilotons() * 1.5 { general } else { medium };
            if (settlers + endowment - hold.kilotons()).abs() < 1e-9 {
                full_hold += 1;
            }
            if (settlers - hold.kilotons()).abs() < 1e-9 {
                all_settlers += 1;
            }
            if endowment > 1e-12 {
                carried_minerals += 1;
            }
        }
    }
    let pct = |k: usize| if n > 0 { 100.0 * k as f64 / n as f64 } else { 0.0 };
    println!("census — seed {seed}, {horizon:.0} yr, {n} colonisers launched");
    println!("  hold filled to capacity : {full_hold:>7}  ({:.1}%)", pct(full_hold));
    println!("  hold all settlers, no ore: {all_settlers:>7}  ({:.1}%)", pct(all_settlers));
    println!("  carried any minerals     : {carried_minerals:>7}  ({:.1}%)", pct(carried_minerals));
    println!("  mean settlers {:.4} kt, mean endowment {:.4} kt", sum_s / n.max(1) as f64, sum_e / n.max(1) as f64);
    println!("  Medium hold {:.4} kt, General hold {:.4} kt", medium.kilotons(), general.kilotons());
}

fn main() {
    if std::env::args().any(|a| a == "--census") {
        for seed in SEEDS {
            census(seed, 2000.0);
        }
        return;
    }
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
