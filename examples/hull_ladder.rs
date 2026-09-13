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

/// One leg of the change, so each can be attributed rather than the whole move
/// being reported as a single number. A leg may touch the config, the doctrine,
/// or both — stage 4 is a doctrine change on top of stage 3's config one.
struct Leg {
    name: &'static str,
    apply: fn(&mut SimConfig),
    doctrine: fn(&mut Doctrine),
}

// **Only the baseline leg remains, and that is the finding.**
//
// Stage 3's legs measured a config that no longer exists — `medium_fleet_size`
// and `cargo_unit_size` are now the ratified defaults, and since stage 3c
// `hull_radius` solves the shell model rather than square-rooting the cost
// ratio, so the "hold held fixed" ablation has nothing left to hold fixed.
// Their numbers are recorded in `hyades_todo.md` T-56 stage 3b.
//
// Stage 4's legs measured a doctrine that no longer exists either. The
// colonizer's hull is not a preference any more: it is derived from the
// target's carrying capacity, because a seed above `K` crashes rather than
// settling. That derivation currently always answers "Medium", since a new
// colony's `K` is capped at one Band by the infra its recycled hull provides.
// Those numbers are in T-56 stage 4.
//
// What is left is the bed itself, which is still the instrument for the
// acceptance test and for anything that moves the expansion economy.
const LEGS: &[Leg] = &[
    Leg { name: "ratified ladder", apply: |_| {}, doctrine: |_| {} },
    // ~~2/3/5 miners per outpost~~, ~~vein fraction 0.1/0.3/0.6~~ — **there is no
    // crew leg any more (T-87).** Crew size stopped being a policy: it is
    // derived from the founding centre's unmet mineral demand
    // (`Hyades_industry.md` §4.5), so there is no value here to sweep.
    //
    // Both retired knobs were the same mistake, and the sweep is what found it:
    // a rock is a finite stock, so a crew only brings its yield *forward*, and
    // ore the empire cannot spend is a pure cost in hulls, freight and entity
    // count. Every crew size measured worse than the one below it.
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
        let is_baseline = base.len() < SEEDS.len();
        if let Some(f) = &filter {
            if !is_baseline && !leg.name.contains(f.as_str()) {
                continue;
            }
        }
        let mut sum_cy = 0.0;
        let mut sum_col = 0.0;
        let mut sum_dbl = 0.0;
        println!("== {} ==", leg.name);
        for (i, &seed) in SEEDS.iter().enumerate() {
            let r = run(seed, leg.apply, leg.doctrine);
            let (d_cy, d_col) = if is_baseline {
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
            if is_baseline {
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

fn run(seed: u64, apply: fn(&mut SimConfig), set_doctrine: fn(&mut Doctrine)) -> Run {
    let galaxy = Galaxy::generate(GalaxyConfig::new(PLAYERS, seed)).unwrap();
    let mut doctrine = Doctrine::default();
    set_doctrine(&mut doctrine);
    let autopilots: Vec<Box<dyn Autopilot>> =
        (0..PLAYERS).map(|_| Box::new(BaselineAutopilot::new(doctrine)) as Box<_>).collect();
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
