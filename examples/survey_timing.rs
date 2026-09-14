//! **Could an empire scout the same planets EARLIER by building more scouts?**
//!
//! Not "is every planet eventually scanned" — that question has a useless answer
//! (yes, 100%) and `Hyades_industry.md` §6.17 used it to wrongly rule survey out
//! as a sink for minerals. **Final coverage is a horizon artifact.** A survey
//! fleet that finishes the galaxy at year 3,000 and one that finishes at year
//! 800 both report 100%, and the second one is worth vastly more: a world
//! scanned earlier can be colonised earlier, and colony-years is an integral.
//!
//! So this measures **when**, against scout count:
//!
//! - **mean and median year of first scan**, per player — the thing more scouts
//!   would buy;
//! - **p90**, because the tail is where a starved survey shows up first;
//! - **colony-years**, to say whether earlier scanning is worth anything
//!   downstream or merely happens.
//!
//! `Doctrine::survey_reserve` is the knob: the frontier size a centre keeps
//! stocked, so it sets how many scouts exist. `CLAUDE.md` §2 records it as a
//! *false positive* in the gradient probe — a plateau at 1,024 with a cliff
//! below — but that was measured against **coverage**, which saturates. Against
//! *timing* it may not be flat at all, and that is the question.
//!
//! Run: `cargo run --release --example survey_timing`
use hyades_engine::log::{LogCategory, LogEvent, LogFilter};
use hyades_engine::prelude::*;
use std::collections::HashMap;
use std::io::Write;

const SEEDS: [u64; 2] = [1, 7];
const PLAYERS: usize = 3;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let planets: Option<usize> = args.first().and_then(|a| a.parse().ok());
    let horizon: f64 = args.get(1).and_then(|a| a.parse().ok()).unwrap_or(2000.0);

    println!("survey timing — when are planets scanned, not whether. {PLAYERS} seats, {horizon:.0} yr");
    println!(
        "{:<10}{:>8}{:>10}{:>10}{:>10}{:>10}{:>14}{:>9}{:>9}{:>9}",
        "scouts@t0", "seed", "scanned", "mean yr", "median", "p90", "colony-years", "scouts", "cand p50", "cand max"
    );
    std::io::stdout().flush().ok();

    // **Sweep the bootstrap scout count, not `survey_reserve`.** The reserve
    // turned out inert across a 64x range (256 -> 16,384: bit-identical scan
    // times, colony-years and scout counts), and the `cands` column below is the
    // mechanism rather than a guess: it prints where `candidate_count` — the
    // quantity `wants_survey = candidate_count < survey_reserve` compares —
    // actually sits. `survey_vehicles` is the bootstrap fan-out and reaches the
    // galaxy without passing through that predicate at all, so it is the clean
    // way to ask the counterfactual: **would more scouts scan earlier?**
    for reserve in [3usize, 6, 12, 24] {
        for seed in SEEDS {
            let mut gcfg = GalaxyConfig::new(PLAYERS, seed);
            if let Some(n) = planets {
                gcfg.planet_count = n;
            }
            let galaxy = Galaxy::generate(gcfg).unwrap();
            let doctrine = Doctrine { survey_vehicles: reserve, ..Default::default() };
            let autopilots: Vec<Box<dyn Autopilot>> =
                (0..PLAYERS).map(|_| Box::new(BaselineAutopilot::new(doctrine)) as Box<_>).collect();
            let mut cfg = SimConfig::new(seed);
            cfg.horizon_years = horizon;
            let mut sim = Simulation::new(galaxy, cfg, autopilots);
            sim.set_log_filter(
                LogFilter::none().with(LogCategory::Scanning).with(LogCategory::Vehicles).with(LogCategory::Production),
            );
            sim.run();

            // First scan of each (player, planet) pair — a re-scan is not news.
            let mut first: HashMap<(u32, u32), f64> = HashMap::new();
            let mut colony_years = 0.0f64;
            // **Scouts actually built**, against the 6 per seat handed out free
            // at bootstrap. If this is ~0 the survey fleet is the opening fan-out
            // and nothing else, and no amount of `survey_reserve` can matter.
            let mut scouts_built = 0u32;
            // **What `survey_reserve` is actually compared against.** A threshold
            // that sits above the whole range of the thing it thresholds is not a
            // knob, and this is the cheapest way to see that rather than infer it.
            let mut seen: Vec<u32> = Vec::new();
            for r in sim.log().iter() {
                match r.event {
                    LogEvent::ProductionDecision { candidates_seen, .. } => seen.push(candidates_seen),
                    LogEvent::ScanReceived { player, planet } => {
                        first.entry((player, planet.0)).or_insert(r.time);
                    }
                    LogEvent::ColonyFounded { .. } => colony_years += horizon - r.time,
                    LogEvent::VehicleSpawned { role: Role::Scout, .. } => scouts_built += 1,
                    _ => {}
                }
            }
            let mut times: Vec<f64> = first.values().copied().collect();
            times.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let n = times.len();
            let mean = if n > 0 { times.iter().sum::<f64>() / n as f64 } else { 0.0 };
            let pick = |q: f64| if n > 0 { times[((n as f64 - 1.0) * q) as usize] } else { 0.0 };
            seen.sort_unstable();
            let cand_p50 = seen.get(seen.len() / 2).copied().unwrap_or(0);
            let cand_max = seen.last().copied().unwrap_or(0);

            println!(
                "{reserve:<10}{seed:>8}{n:>10}{mean:>10.1}{:>10.1}{:>10.1}{colony_years:>14.0}{scouts_built:>9}\
                 {cand_p50:>9}{cand_max:>9}",
                pick(0.5),
                pick(0.9)
            );
            std::io::stdout().flush().ok();
        }
    }
}
