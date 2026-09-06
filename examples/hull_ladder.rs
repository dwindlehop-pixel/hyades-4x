//! **T-56 stage 3: what the ratified hull ladder costs, one leg at a time.**
//!
//! The acceptance test for the ladder is stated in `hyades_todo.md` T-56: if
//! making General hulls more expensive degrades the colony doubling rate or
//! total colony-years, the ladder is bad. This is the instrument for it.
//!
//! Colony *count* is a weak invariant — a change that founds the same worlds a
//! century later scores identically — so the reported quantity is
//! `∫ colonies dt = Σ (horizon − t_found)`, which falls the moment anything
//! slows down, plus the doubling rate read straight off the founding times.
//!
//! Run: `cargo run --release --example hull_ladder` for every leg, or
//! `... -- <substring>` to run only the legs whose name matches. The baseline
//! leg always runs, because every delta is against it on the same seeds (common
//! random numbers, `CLAUDE.md` §"How to search").
use hyades_engine::log::{LogCategory, LogEvent, LogFilter};
use hyades_engine::prelude::*;
use std::io::Write;

const SEEDS: &[u64] = &[1, 7, 42, 31337];
const PLAYERS: usize = 3;

/// One leg of the ladder change, so each can be attributed rather than the
/// whole move being reported as a single number.
struct Leg {
    name: &'static str,
    apply: fn(&mut SimConfig),
}

const LEGS: &[Leg] = &[
    Leg { name: "shipped", apply: |_| {} },
    Leg {
        name: "cost ladder",
        apply: |c| {
            c.medium_fleet_size = 10.0;
            c.limited_fleet_size = 50.0;
        },
    },
    Leg { name: "hold = KT(I)", apply: |c| c.cargo_unit_size = 1.0 },
    // **The ablation that separates price from hold.** `hull_radius` is
    // `sqrt(cost ratio)`, so `medium_fleet_size` is not a price knob — it moves
    // the Medium radius √2.02 → √5 and its hold 0.19 → 4.81 units, 25× bigger,
    // at the same time as it raises the General hull's relative price. CLAUDE.md
    // §"the artifact pattern" says exactly this: since R-O58 the cost ladder
    // *is* the capacity ladder, so sweeping one leg of it measures two things
    // and reports one.
    //
    // This leg applies the cost ladder and then rescales `cargo_unit_size` so
    // the Medium hold is **unchanged** at its shipped 0.959 kt. Whatever
    // survives here is the price; whatever disappears was the hold.
    Leg {
        name: "cost ladder, hold held fixed",
        apply: |c| {
            c.medium_fleet_size = 10.0;
            c.limited_fleet_size = 50.0;
            c.cargo_unit_size = 0.199_25;
        },
    },
    Leg {
        name: "both (ratified)",
        apply: |c| {
            c.medium_fleet_size = 10.0;
            c.limited_fleet_size = 50.0;
            c.cargo_unit_size = 1.0;
        },
    },
];

struct Run {
    colonies: usize,
    colony_years: f64,
    doubling: f64,
}

fn main() {
    let filter = std::env::args().nth(1);
    let mut base: Vec<Run> = Vec::new();
    for leg in LEGS {
        // The baseline is never skipped: the deltas are paired against it
        // seed by seed, which is what cancels the enormous seed noise.
        if let Some(f) = &filter {
            if leg.name != "shipped" && !leg.name.contains(f.as_str()) {
                continue;
            }
        }
        let mut sum_cy = 0.0;
        let mut sum_col = 0.0;
        let mut sum_dbl = 0.0;
        println!("== {} ==", leg.name);
        for (i, &seed) in SEEDS.iter().enumerate() {
            let r = run(seed, leg.apply);
            let (d_cy, d_col) = if leg.name == "shipped" {
                (0.0, 0.0)
            } else {
                (
                    100.0 * (r.colony_years - base[i].colony_years) / base[i].colony_years,
                    100.0 * (r.colonies as f64 - base[i].colonies as f64) / base[i].colonies as f64,
                )
            };
            println!(
                "  seed {seed:>6}: colonies={:>6} ({d_col:>+6.1}%)  colony_years={:>12.1} ({d_cy:>+6.1}%)  \
                 doubling={:>6.1} yr",
                r.colonies, r.colony_years, r.doubling
            );
            std::io::stdout().flush().unwrap();
            sum_cy += r.colony_years;
            sum_col += r.colonies as f64;
            sum_dbl += r.doubling;
            if leg.name == "shipped" {
                base.push(r);
            }
        }
        let n = SEEDS.len() as f64;
        println!(
            "  mean: colonies={:.1}  colony_years={:.1}  doubling={:.1} yr\n",
            sum_col / n,
            sum_cy / n,
            sum_dbl / n
        );
        std::io::stdout().flush().unwrap();
    }
}

fn run(seed: u64, apply: fn(&mut SimConfig)) -> Run {
    let galaxy = Galaxy::generate(GalaxyConfig::new(PLAYERS, seed)).unwrap();
    let autopilots: Vec<Box<dyn Autopilot>> =
        (0..PLAYERS).map(|_| Box::new(BaselineAutopilot::new(Doctrine::default())) as Box<_>).collect();
    let mut cfg = SimConfig::new(seed);
    apply(&mut cfg);
    let mut sim = Simulation::new(galaxy, cfg, autopilots);
    sim.set_log_filter(LogFilter::none().with(LogCategory::Vehicles));
    sim.run();

    let horizon = sim.clock();
    let mut times: Vec<f64> = Vec::new();
    for r in sim.log().iter() {
        if matches!(r.event, LogEvent::ColonyFounded { .. }) {
            times.push(r.time);
        }
    }
    times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let colony_years = times.iter().map(|t| horizon - t).sum();
    let colonies = sim.snapshot().planets.iter().filter(|p| p.owner.is_some()).count();

    // Doubling time over the compounding phase: the founding count doubles
    // repeatedly, so the mean interval between successive doublings is the
    // loop's time constant. Measured from the 8th founding (past the seats'
    // own homeworlds and the first scouted world) to the last.
    let doubling = if times.len() >= 16 {
        let start = 8usize;
        let steps = ((times.len() as f64 / start as f64).log2()).max(1.0);
        (times[times.len() - 1] - times[start - 1]) / steps
    } else {
        f64::NAN
    };
    Run { colonies, colony_years, doubling }
}
