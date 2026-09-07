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

// **Stage 3's legs are retired, not deleted — they measured a config that no
// longer exists.** `medium_fleet_size` and `cargo_unit_size` are now the
// ratified defaults, and since stage 3c `hull_radius` solves the shell model
// rather than square-rooting the cost ratio, so the "hold held fixed" ablation
// has nothing left to hold fixed: cost and capacity are separate functions of
// one body. The numbers those legs produced are recorded in `hyades_todo.md`
// T-56 stage 3b, which is where they belong.
//
// What is left to measure is stage 4: the ladder is only worth what the
// doctrine spending it can buy, and until now nothing in a run ever built a
// General hull.
const LEGS: &[Leg] = &[
    // The ratified ladder with the pre-T-56 doctrine: Colonizer pinned to the
    // Medium hull. This is what stage 3 shipped, and the baseline every other
    // leg is paired against seed by seed.
    Leg { name: "Medium colonizers (shipped)", apply: |_| {}, doctrine: |_| {} },
    // A General colonizer whenever the center can pay for one out of the
    // stockpile it has *this decision*, else a Medium. Expansion never stalls
    // waiting for a bigger ship.
    Leg {
        name: "General when affordable",
        apply: |_| {},
        doctrine: |d| d.colonizer_hull = ColonizerHull::GeneralWhenAffordable,
    },
    // Always General, even when it means banking instead of building — the
    // aggressive end. Fewer foundings, each starting a Band higher.
    Leg { name: "General always", apply: |_| {}, doctrine: |d| d.colonizer_hull = ColonizerHull::General },
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
