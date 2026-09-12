//! **Does deepening buy ships? Count what one centre actually founds.**
//!
//! The work-years sweep (R-O87) found `reinvest_bias` flat and explained it with
//! an identity: a mineral buys the same *works* whether it deepens or founds.
//! That identity is about **stock**, and the objection to it is about **flow** —
//! a colony is founded with a recycled hull and cannot keep improving that way,
//! so deepening and founding differ in what they buy *afterwards*, which is
//! exactly what an integral over 1,500 years should reward.
//!
//! The flow arithmetic, off the engine's own functions (`t_lead = 2.0`, a Medium
//! coloniser is 0.10 kt):
//!
//! | rung | stock | `F` | slips | `t_build` | hull/yr | next step |
//! |---|---|---|---|---|---|---|
//! | I | 0.10 kt | 0.1000 | 2 | 4.000 yr | 0.500 | 0.90 kt = **9 colonisers** |
//! | II | 1.00 kt | 0.1818 | 2 | 3.100 yr | **0.645 (+29.0%)** | 19 kt = 190 colonisers |
//! | III | 20.0 kt | 0.1990 | 2 | 3.005 yr | 0.666 (+3.2%) | 780 kt = 7,800 colonisers |
//! | IV | 800 kt | 0.2000 | 2 | 3.000 yr | 0.667 (+0.2%) | — |
//!
//! So the objection is right about rung I→II: **+29% build rate for the price of
//! nine colonisers pays itself back in 62 years** and should be worth ~200 extra
//! hulls over the horizon. It is wrong about slips, and that is the engine's
//! fault rather than the argument's — `slips(F) = 1 + ⌊F / slip_throughput⌋`
//! with `F < fab_cap = 0.2` and `slip_throughput = 0.1`, so **a yard has two
//! berths at every rung and no amount of infrastructure buys a third** (R-O85).
//! The whole ladder is worth +33% throughput, and 799 of its 800 kt buy +3.4%.
//!
//! Which leaves the question this harness exists to answer: if one rung is worth
//! +29% forever, why is the objective flat? So it counts the thing the argument
//! is actually about — **how many colonies a homeworld founds itself**, when its
//! first one lands, and how many hulls it laid down to get there — rather than
//! an empire-wide integral that averages the answer away.
//!
//! Run: `cargo run --release --example founding_tree -- [horizon] [bias...]`
use hyades_engine::autopilot::BuildOrder;
use hyades_engine::log::{LogCategory, LogEvent, LogFilter};
use hyades_engine::math::Vec3;
use hyades_engine::prelude::*;
use std::collections::HashMap;
use std::io::Write;

const SEEDS: [u64; 2] = [1, 7];
const PLAYERS: usize = 3;
const SAMPLE_YEARS: f64 = 50.0;

struct Arm {
    /// Colonies founded by a coloniser that launched **from a homeworld** —
    /// the homeworld's own children, not its descendants.
    home_foundings: f64,
    /// Year the first of those landed. The "delayed first ship" the deepening
    /// argument trades against.
    home_first: f64,
    /// Hulls of every kind the homeworld laid down.
    home_hulls: f64,
    /// The homeworld's infrastructure rung at the horizon.
    home_rung: f64,
    colonies: f64,
    /// Empire-wide, so the ablation arm's zero is visible.
    infra_builds: f64,
    work_years: f64,
    /// `∫ vehicles dt` — the objection's own suggested metric. Work-years
    /// measures the industrial stock; this measures the fleet the industry
    /// exists to produce, so a change that buys build *rate* shows up here
    /// where it might not there.
    fleet_years: f64,

    // --- why the yard is not the constraint -------------------------------
    /// Hulls the homeworld built as a fraction of what its rung could have
    /// built over the same span. **If this is small, build rate is not the
    /// binding constraint and no amount of deepening can matter.**
    utilisation: f64,
    /// Production decisions taken at the homeworld, and the share of them that
    /// chose `Idle`. A declined build leaves the yard free and the *next*
    /// retry is the economy tick, `cycle_years = 50` — sixteen builds' worth of
    /// yard time at rung II's 3.1-year `t_build`.
    decisions: f64,
    idle_share: f64,
    /// Mean years between consecutive decisions at the homeworld. Compare
    /// against `t_build`: if it is near `cycle_years` the centre is waiting on
    /// the tick, not on the yard.
    decision_gap: f64,
    /// Of the decisions that idled, the share that had **no candidate to build
    /// for** versus the share that could not **afford** what they wanted.
    idle_no_target: f64,
    #[allow(dead_code)]
    idle_no_money: f64,
    /// **Mean years from a decision to the centre's next one, split by what the
    /// decision chose.** This is the whole question. `commit_one_build`
    /// returning `None` leaves the yard free and schedules nothing — the next
    /// attempt is the *economy tick*, `cycle_years = 50`. A committed build
    /// schedules its own `BuildDecision` at `t_build`. So if the after-idle gap
    /// is an order of magnitude larger than the after-build gap, the centre's
    /// throughput is set by the retry cadence and not by the yard, and
    /// deepening — which only buys yard rate — cannot reach the objective.
    gap_after_idle: f64,
    gap_after_build: f64,
}

fn run(seed: u64, horizon: f64, bias: f64) -> Arm {
    let galaxy = Galaxy::generate(GalaxyConfig::new(PLAYERS, seed)).unwrap();
    let homes: Vec<Vec3> = galaxy.homeworlds.iter().map(|h| galaxy.planets[h.0 as usize].position).collect();
    let home_ids: Vec<PlanetId> = galaxy.homeworlds.clone();
    let doctrine = Doctrine { reinvest_bias: bias, ..Doctrine::default() };
    let autopilots: Vec<Box<dyn Autopilot>> =
        (0..PLAYERS).map(|_| Box::new(BaselineAutopilot::new(doctrine)) as Box<_>).collect();
    let mut cfg = SimConfig::new(seed);
    cfg.horizon_years = horizon;
    let mut sim = Simulation::new(galaxy, cfg, autopilots);
    sim.set_log_filter(LogFilter::none().with(LogCategory::Vehicles).with(LogCategory::Production));

    // Fleet-years and work-years both want a fixed time grid, not events —
    // event-driven sampling weights the series by activity, which is the thing
    // being measured.
    let (mut work_years, mut fleet_years) = (0.0f64, 0.0f64);
    let (mut prev_t, mut prev_w, mut prev_v) = (0.0f64, 0.0f64, 0.0f64);
    let mut next = SAMPLE_YEARS;
    let mut done = false;
    while !done && next <= horizon {
        while sim.clock() < next {
            if !sim.step() {
                done = true;
                break;
            }
        }
        let snap = sim.snapshot();
        let w: f64 = snap.planets.iter().filter(|p| p.owner.is_some()).map(|p| p.works.kilotons()).sum();
        let v = snap.vehicles.len() as f64;
        let dt = sim.clock().min(next) - prev_t;
        work_years += 0.5 * (w + prev_w) * dt;
        fleet_years += 0.5 * (v + prev_v) * dt;
        prev_t = sim.clock().min(next);
        prev_w = w;
        prev_v = v;
        next += SAMPLE_YEARS;
    }

    // Parentage by join: a spawn records where it launched from, a founding
    // records which vehicle did it. `from` is the centre's own position, copied
    // verbatim, so an exact match is the right test and not a tolerance.
    let is_home = |p: Vec3| homes.contains(&p);
    let mut launched_from_home: HashMap<u64, f64> = HashMap::new();
    let (mut decisions, mut idles, mut idle_no_target, mut idle_no_money) = (0.0f64, 0.0f64, 0.0f64, 0.0f64);
    let (mut first_decision, mut last_decision) = (f64::INFINITY, 0.0f64);
    // Per homeworld: the previous decision's time and whether it idled, so each
    // gap is attributed to what the *preceding* decision chose.
    let mut prev_dec: HashMap<u32, (f64, bool)> = HashMap::new();
    let (mut sum_idle_gap, mut n_idle_gap) = (0.0f64, 0.0f64);
    let (mut sum_build_gap, mut n_build_gap) = (0.0f64, 0.0f64);
    let (mut home_foundings, mut home_hulls) = (0.0f64, 0.0f64);
    let mut home_first = f64::INFINITY;
    let (mut colonies, mut infra_builds) = (0.0f64, 0.0f64);
    for r in sim.log().iter() {
        match r.event {
            LogEvent::VehicleSpawned { vehicle, from, .. } => {
                if is_home(from) {
                    launched_from_home.insert(vehicle.0, r.time);
                }
            }
            LogEvent::ColonyFounded { vehicle, .. } => {
                colonies += 1.0;
                if launched_from_home.contains_key(&vehicle.0) {
                    home_foundings += 1.0;
                    home_first = home_first.min(r.time);
                }
            }
            LogEvent::ProductionDecision { center, chosen, candidates_seen, stockpile, colonizer_cost, .. } => {
                if home_ids.contains(&center) {
                    decisions += 1.0;
                    first_decision = first_decision.min(r.time);
                    last_decision = last_decision.max(r.time);
                    if let Some((t, was_idle)) = prev_dec.insert(center.0, (r.time, chosen == BuildOrder::Idle)) {
                        let gap = r.time - t;
                        if was_idle {
                            sum_idle_gap += gap;
                            n_idle_gap += 1.0;
                        } else {
                            sum_build_gap += gap;
                            n_build_gap += 1.0;
                        }
                    }
                    if chosen == BuildOrder::Idle {
                        idles += 1.0;
                        if candidates_seen == 0 {
                            idle_no_target += 1.0;
                        } else if stockpile < colonizer_cost {
                            idle_no_money += 1.0;
                        }
                    }
                }
            }
            LogEvent::BuildApplied { center, order, .. } => match order {
                BuildOrder::UpgradeInfrastructure => infra_builds += 1.0,
                BuildOrder::Hull { .. } => {
                    if home_ids.contains(&center) {
                        home_hulls += 1.0;
                    }
                }
                BuildOrder::Idle => {}
            },
            _ => {}
        }
    }

    let snap = sim.snapshot();
    let home_rung: f64 =
        snap.planets.iter().filter(|p| home_ids.contains(&p.id)).map(|p| p.infrastructure.bands()).sum::<f64>()
            / PLAYERS as f64;

    // What rung II could have built over the span the homeworld was deciding.
    // `t_build` at rung II is 3.1 yr across 2 berths, i.e. 0.645 hull/yr.
    let span = (last_decision - first_decision).max(1e-9);
    let capacity = 0.6452 * span;
    let n = PLAYERS as f64;
    Arm {
        utilisation: (home_hulls / n) / capacity.max(1e-9),
        decisions: decisions / n,
        idle_share: if decisions > 0.0 { idles / decisions } else { 0.0 },
        decision_gap: if decisions > n { span / (decisions / n) } else { f64::NAN },
        idle_no_target: if idles > 0.0 { idle_no_target / idles } else { 0.0 },
        idle_no_money: if idles > 0.0 { idle_no_money / idles } else { 0.0 },
        gap_after_idle: if n_idle_gap > 0.0 { sum_idle_gap / n_idle_gap } else { f64::NAN },
        gap_after_build: if n_build_gap > 0.0 { sum_build_gap / n_build_gap } else { f64::NAN },
        home_foundings: home_foundings / n,
        home_first,
        home_hulls: home_hulls / n,
        home_rung,
        colonies,
        infra_builds,
        work_years,
        fleet_years,
    }
}

fn main() {
    let mut args = std::env::args().skip(1);
    let horizon: f64 = args.next().and_then(|a| a.parse().ok()).unwrap_or(1500.0);
    let biases: Vec<f64> = {
        let v: Vec<f64> = args.filter_map(|a| a.parse().ok()).collect();
        if v.is_empty() {
            vec![0.5]
        } else {
            v
        }
    };

    println!("founding tree — what a homeworld founds itself, {PLAYERS} seats, {horizon:.0} yr");
    println!(
        "{:>7}{:>6}{:>10}{:>9}{:>7}{:>9}{:>9}{:>13}{:>13}{:>7}{:>8}{:>8}{:>9}{:>9}{:>10}{:>9}{:>8}",
        "bias",
        "seed",
        "home kid",
        "first yr",
        "hulls",
        "rung",
        "colonies",
        "work-years",
        "fleet-years",
        "util",
        "decis",
        "idle%",
        "gap yr",
        "noTarget",
        "gap|idle",
        "gap|bld",
        "infra"
    );
    std::io::stdout().flush().ok();
    for &b in &biases {
        for seed in SEEDS {
            let a = run(seed, horizon, b);
            println!(
                "{b:>7.3}{seed:>6}{:>10.1}{:>9.1}{:>7.0}{:>9.3}{:>9.0}{:>13.0}{:>13.0}{:>7.1}%{:>8.0}{:>7.0}%{:>9.1}{:>8.0}%{:>10.1}{:>9.1}{:>8.0}",
                a.home_foundings,
                a.home_first,
                a.home_hulls,
                a.home_rung,
                a.colonies,
                a.work_years,
                a.fleet_years,
                100.0 * a.utilisation,
                a.decisions,
                100.0 * a.idle_share,
                a.decision_gap,
                100.0 * a.idle_no_target,
                a.gap_after_idle,
                a.gap_after_build,
                a.infra_builds
            );
            std::io::stdout().flush().ok();
        }
    }
}
