//! Why is coverage insensitive to `cargo_unit_size`, `outpost_mining_fraction`
//! and `reinvest_bias`?
//!
//! `min_time_search` reports 0/10 seeds fully covered and mean coverage flat to
//! one decimal across every value of those three knobs. A sweep that cannot move
//! its objective is telling you the objective is bound by something the sweep
//! does not touch — but it cannot say what. This driver answers that by
//! interrogating the log (`hyades_engine::log`, the diagnostic seam) instead of
//! adding another sweep.
//!
//! Three things it does that a sweep cannot:
//!
//! 1. **Isolate each knob** and compare a run *fingerprint* (decisions taken,
//!    colonies founded, coverage). A knob whose fingerprint is byte-identical
//!    across its whole range is not weakly coupled to the objective, it is
//!    disconnected from the run entirely — a much stronger and more useful
//!    statement than "the mean barely moved".
//!
//! 2. **Classify every `ProductionDecision` by why it went the way it did.**
//!    `production_choice` has one gate before expansion is even considered:
//!
//!    ```text
//!    if ctx.level < ctx.medium_min_level { deepen if affordable, else Idle }
//!    ```
//!
//!    so a center below level 3 can never build a colony vehicle however rich it
//!    is. Bucketing Idle on either side of that gate separates "starved of
//!    minerals" (which the swept knobs would fix) from "gated below the
//!    expansion level" and "permanently capped below it" (which they cannot).
//!
//! 3. **Account for the survey budget**, since nothing can be colonized that was
//!    never scanned. The scout fleet is fixed at bootstrap and each chain is
//!    capped at `max_survey_hops`, so total exploration is a product of three
//!    integers no swept knob touches.
//!
//! ## Status: the bugs it found are fixed; it is now a regression detector
//!
//! Everything above is why this driver was written, and the diagnosis held: the
//! binding constraint was `k_high` starving the Mining-outpost class, not any of
//! the swept economy knobs. With the snowball defaults ratified (R-AC16/R-AC17),
//! the same run reports the opposite of what it originally found — mining pairs
//! and freighter legs in the thousands, scouts built by production centers, and
//! coverage up from 41 worlds to over 1,500 at a 4,000-year horizon.
//!
//! It earns its keep from here as a **regression detector**: if the hauling path
//! or the scout order ever breaks again, the "INERT" verdicts and the zero
//! counters come straight back. Every conclusion it prints is derived from the
//! run rather than asserted in prose, precisely so it cannot go stale the way an
//! earlier version did — that one hardcoded "the fleet is never replenished"
//! and printed it directly beneath a line counting 125 replenishment orders.
//!
//! Run with: `cargo run --release --example coverage_trace`

use std::collections::{HashMap, HashSet};

use hyades_engine::autopilot::{Autopilot, BaselineAutopilot, BuildOrder, Doctrine};
use hyades_engine::log::{LogCategory, LogEvent, LogFilter};
use hyades_engine::prelude::*;
use hyades_engine::sim::HullType;

const PLAYERS: usize = 3;
const SEED: u64 = 1;

/// Part 1 asks a yes/no question — *does this knob move the run at all* — and a
/// shorter horizon answers it just as well as a full one, for a fraction of the
/// cost. Part 2/3 stay at the shipped horizon because they characterise the real
/// run rather than comparing two of them.
const ISOLATE_HORIZON: f64 = 2000.0;

/// Same definition `min_time_search` and `coverage_time` score against.
fn coverage_targets(galaxy: &Galaxy) -> HashSet<PlanetId> {
    galaxy.planets.iter().filter(|p| p.habitability.min(p.biosphere) > 0.01).map(|p| p.id).collect()
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
struct Fingerprint {
    decisions: u64,
    deepens: u64,
    colonies: u64,
    mining_pairs: u64,
    lights: u64,
    covered: usize,
}

#[derive(Default, Debug, Clone, Copy)]
struct Idle {
    /// Below the gate, cannot afford the next infra level.
    gated_broke: u64,
    /// Below the gate, and infra is already at `k_potential`: this center can
    /// never deepen again, so it can never reach the gate. A dead end.
    gated_capped: u64,
    gated_other: u64,
    /// At/above the gate with nothing worth expanding to.
    open_no_candidates: u64,
    /// At/above the gate, candidates available, too poor to buy any.
    open_broke: u64,
    open_other: u64,
}

struct Outcome {
    fp: Fingerprint,
    idle: Idle,
    idle_total: u64,
    at_or_above_gate: u64,
    /// Of the at/above-gate decisions, how many could still deepen at all.
    gate_deepen_possible: u64,
    centers_seen: usize,
    centers_reaching_gate: usize,
    targets: usize,
    known_planets: usize,
    last_colony_time: f64,
    /// Hauling-economy activity: outposts placed, freighter legs, extractions.
    freighter_legs: u64,
    extractions: u64,
    gated_stockpile_sum: f64,
    gated_cost_sum: f64,

    // --- survey ---
    /// Distinct scout entities that ever flew.
    scouts: usize,
    /// Hops completed (one `ContactArrived` per world reached).
    hops_total: u64,
    /// Scout chains that ran to termination (`next: None`).
    chains_exhausted: u64,
    /// Largest hop count any single scout chain reached.
    max_hops_seen: u64,
    /// Total planets in the galaxy, and distinct planets scanned by ANYONE.
    galaxy_planets: usize,
    distinct_scanned: usize,
    /// Clock at the last scan report received.
    last_scan_time: f64,
    /// Total planets scanned that are also coverage targets.
    scanned_targets: usize,
}

fn run(seed: u64, cfg: SimConfig, doctrine: Doctrine) -> Outcome {
    let galaxy = Galaxy::generate(GalaxyConfig::new(PLAYERS, seed)).unwrap();
    let targets = coverage_targets(&galaxy);

    let autopilots: Vec<Box<dyn Autopilot>> =
        (0..PLAYERS).map(|_| Box::new(BaselineAutopilot::new(doctrine)) as Box<_>).collect();
    let mut sim = Simulation::new(galaxy, cfg, autopilots);
    // Everything except Population: we want decisions, vehicle lifecycle, the
    // mineral/hauling economy, and what knowledge the empires ever acquired.
    sim.set_log_filter(
        LogFilter::none()
            .with(LogCategory::Production)
            .with(LogCategory::Vehicles)
            .with(LogCategory::Mining)
            .with(LogCategory::Scanning),
    );
    sim.run();

    let mut fp = Fingerprint::default();
    let mut idle = Idle::default();
    let (mut idle_total, mut at_gate, mut gate_deepen_possible) = (0u64, 0u64, 0u64);
    let (mut freighter_legs, mut extractions) = (0u64, 0u64);
    let (mut gated_stockpile_sum, mut gated_cost_sum) = (0.0f64, 0.0f64);
    let mut centers: HashMap<(u32, PlanetId), u8> = HashMap::new();
    let mut known: HashSet<(u32, PlanetId)> = HashSet::new();
    let mut founded: HashSet<PlanetId> = HashSet::new();
    let mut last_colony_time = 0.0_f64;
    let mut distinct_scanned: HashSet<PlanetId> = HashSet::new();
    let mut scout_hops: HashMap<hyades_engine::sim::Entity, u64> = HashMap::new();
    let (mut hops_total, mut chains_exhausted, mut max_hops_seen) = (0u64, 0u64, 0u64);
    let mut last_scan_time = 0.0_f64;
    let galaxy_planets = sim.snapshot().planets.len();

    for rec in sim.log().iter() {
        match rec.event {
            LogEvent::ProductionDecision {
                player,
                center,
                pop_level,
                infra,
                k_potential,
                stockpile,
                infra_cost,
                colonizer_cost,
                mining_pair_cost,
                candidates_seen,
                chosen,
                ..
            } => {
                fp.decisions += 1;
                let e = centers.entry((player, center)).or_insert(0);
                *e = (*e).max(pop_level);

                let gated = pop_level < cfg.medium_min_level;
                // Must mirror `production_choice`'s guard exactly, or the
                // buckets below describe a decision the engine never made.
                let can_deepen = infra < k_potential - 1e-9;
                if !gated {
                    at_gate += 1;
                    if can_deepen {
                        gate_deepen_possible += 1;
                    }
                }

                match chosen {
                    BuildOrder::Idle => {
                        idle_total += 1;
                        if gated {
                            gated_stockpile_sum += stockpile;
                            gated_cost_sum += infra_cost;
                            if !can_deepen {
                                idle.gated_capped += 1;
                            } else if stockpile + 1e-9 < infra_cost {
                                idle.gated_broke += 1;
                            } else {
                                idle.gated_other += 1;
                            }
                        } else if candidates_seen == 0 {
                            idle.open_no_candidates += 1;
                        } else if stockpile + 1e-9 < colonizer_cost.min(mining_pair_cost).min(infra_cost) {
                            idle.open_broke += 1;
                        } else {
                            idle.open_other += 1;
                        }
                    }
                    BuildOrder::UpgradeInfrastructure => fp.deepens += 1,
                    // Post-R-O29 the order names the hull, not the errand, so
                    // these buckets read the hull type. Medium = a settler,
                    // Limited Systems = a miner-and-freighter pair, LCV = a
                    // scout — the same three things, now inferred rather than
                    // announced, which is the point of the split.
                    BuildOrder::Hull { hull_type: HullType::MediumSystems, .. } => fp.colonies += 1,
                    BuildOrder::Hull { hull_type: HullType::LimitedSystems, .. } => fp.mining_pairs += 1,
                    BuildOrder::Hull { hull_type: HullType::LimitedContactVehicle, .. } => fp.lights += 1,
                    BuildOrder::Hull { .. } => {}
                }
            }
            LogEvent::ColonyFounded { planet, .. } => {
                founded.insert(planet);
                last_colony_time = last_colony_time.max(rec.time);
            }
            LogEvent::ScanReceived { player, planet } => {
                known.insert((player, planet));
                distinct_scanned.insert(planet);
                last_scan_time = last_scan_time.max(rec.time);
            }
            LogEvent::ContactArrived { vehicle, next, .. } => {
                hops_total += 1;
                let h = scout_hops.entry(vehicle).or_insert(0u64);
                *h += 1;
                max_hops_seen = max_hops_seen.max(*h);
                if next.is_none() {
                    chains_exhausted += 1;
                }
            }
            LogEvent::FreighterTransfer { .. } => freighter_legs += 1,
            LogEvent::MineralsExtracted { .. } => extractions += 1,
            _ => {}
        }
    }

    let mut covered: HashSet<PlanetId> = HashSet::new();
    for p in sim.snapshot().planets.iter().filter(|p| p.is_homeworld) {
        if targets.contains(&p.id) {
            covered.insert(p.id);
        }
    }
    for pid in founded.iter().filter(|pid| targets.contains(pid)) {
        covered.insert(*pid);
    }
    fp.covered = covered.len();

    Outcome {
        fp,
        idle,
        idle_total,
        at_or_above_gate: at_gate,
        gate_deepen_possible,
        centers_seen: centers.len(),
        centers_reaching_gate: centers.values().filter(|&&l| l >= cfg.medium_min_level).count(),
        targets: targets.len(),
        known_planets: known.len(),
        last_colony_time,
        freighter_legs,
        extractions,
        gated_stockpile_sum,
        gated_cost_sum,
        scouts: scout_hops.len(),
        hops_total,
        chains_exhausted,
        max_hops_seen,
        galaxy_planets,
        scanned_targets: distinct_scanned.iter().filter(|p| targets.contains(p)).count(),
        distinct_scanned: distinct_scanned.len(),
        last_scan_time,
    }
}

fn pct(n: u64, d: u64) -> f64 {
    if d == 0 {
        0.0
    } else {
        n as f64 / d as f64 * 100.0
    }
}

/// Sweep one knob in isolation and report whether the run fingerprint moves.
fn isolate(label: &str, values: &[f64], apply: impl Fn(&mut SimConfig, &mut Doctrine, f64)) {
    println!("\n-- {label} --");
    println!(
        "{:>12}  {:>10}  {:>8}  {:>9}  {:>8}  {:>8}  {:>8}",
        "value", "decisions", "deepens", "colonies", "mining", "scouts", "covered"
    );
    let mut first: Option<Fingerprint> = None;
    let mut all_same = true;
    for &v in values {
        let mut cfg = SimConfig::new(SEED);
        cfg.horizon_years = ISOLATE_HORIZON;
        let mut doc = Doctrine::default();
        apply(&mut cfg, &mut doc, v);
        let o = run(SEED, cfg, doc);
        println!(
            "{:>12.2}  {:>10}  {:>8}  {:>9}  {:>8}  {:>8}  {:>8}",
            v, o.fp.decisions, o.fp.deepens, o.fp.colonies, o.fp.mining_pairs, o.fp.lights, o.fp.covered
        );
        match first {
            None => first = Some(o.fp),
            Some(f) => {
                if f != o.fp {
                    all_same = false;
                }
            }
        }
    }
    if all_same {
        println!("  => INERT: identical fingerprint across the whole range");
    } else {
        println!("  => couples to the objective");
    }
}

fn main() {
    println!("Coverage trace — seed {SEED}, {PLAYERS} seats, interrogating the production log");
    println!("Question: why do cargo_unit_size / outpost_mining_fraction / reinvest_bias");
    println!("          fail to move coverage in min_time_search?\n");

    println!("=== Part 1: isolate each knob, compare run fingerprints ===");
    println!("(min_time_search swept these jointly by coordinate descent, which cannot");
    println!(" distinguish 'weakly coupled' from 'not connected to the run at all')");

    isolate("cargo_unit_size", &[1.0, 2.5, 6.0], |c, _, v| c.cargo_unit_size = v);
    isolate("outpost_mining_fraction", &[0.1, 0.3, 0.5], |c, _, v| c.outpost_mining_fraction = v);
    isolate("reinvest_bias (the range min_time_search swept)", &[0.0, 0.25, 0.5], |_, d, v| d.reinvest_bias = v);
    // The deepen-vs-expand dial compares b*headroom against (1-b)*score.
    // `headroom` is bounded by k_potential (<= 1), while `score` is a weighted
    // sum that runs well above 1, so deepening cannot win anywhere in [0, 0.5]
    // — the swept range sits entirely on one side of the switch. Push past it.
    isolate("reinvest_bias (past the range it swept)", &[0.9, 0.97, 1.0], |_, d, v| d.reinvest_bias = v);
    isolate("medium_fleet_size (the one that DID move it)", &[3.0, 8.0, 12.0], |c, _, v| c.medium_fleet_size = v);

    println!("\n\n=== Part 2: where the run actually spends its decisions (baseline) ===");
    let o = run(SEED, SimConfig::new(SEED), Doctrine::default());

    println!(
        "\ncoverage {}/{} ({:.2}%)   last colony founded t={:.0} yr of {:.0}",
        o.fp.covered,
        o.targets,
        o.fp.covered as f64 / o.targets as f64 * 100.0,
        o.last_colony_time,
        SimConfig::new(SEED).horizon_years
    );
    println!("distinct (player, center) pairs: {}", o.centers_seen);
    println!(
        "  ever reached the medium gate (level >= 3): {} ({:.1}%)",
        o.centers_reaching_gate,
        pct(o.centers_reaching_gate as u64, o.centers_seen as u64)
    );
    println!("planets ever scanned into knowledge, (player, planet) pairs: {}", o.known_planets);

    // The *bootstrap* fan-out is a hard product of three integers. It used to be
    // the whole survey budget, and therefore a hard ceiling on coverage; now that
    // production centers build scouts too it is only the opening allowance, and
    // the run routinely exceeds it. Reported as a ratio so the line stays true
    // either way instead of asserting a ceiling that no longer holds.
    let doctrine = Doctrine::default();
    let cfg = SimConfig::new(SEED);
    let bootstrap_reach = PLAYERS * doctrine.survey_vehicles * cfg.max_survey_hops;
    println!(
        "  bootstrap survey allowance = players({}) x survey_vehicles({}) x max_survey_hops({}) = {}",
        PLAYERS, doctrine.survey_vehicles, cfg.max_survey_hops, bootstrap_reach
    );
    let ratio = o.hops_total as f64 / bootstrap_reach as f64;
    if ratio > 1.0 {
        println!(
            "  => {} hops actually flown = {ratio:.1}x that allowance, so survey is NOT budget-capped:\n     \
             centers are replenishing the fleet and {} of {} worlds ({:.1}%) were reached",
            o.hops_total,
            o.distinct_scanned,
            o.galaxy_planets,
            o.distinct_scanned as f64 / o.galaxy_planets as f64 * 100.0
        );
    } else {
        println!(
            "  => {} hops flown, within the allowance: survey is budget-capped, and at most\n     \
             {:.1}% of the {} coverage targets can ever be known",
            o.hops_total,
            bootstrap_reach as f64 / o.targets as f64 * 100.0,
            o.targets
        );
    }

    println!("\nthe hauling economy the swept knobs parameterize:");
    println!("  MiningPair orders ever issued: {}", o.fp.mining_pairs);
    println!("  freighter load/deposit legs:   {}", o.freighter_legs);
    println!("  mineral extraction events:     {}", o.extractions);

    println!(
        "\nproduction decisions: {} ({} Idle, {:.1}%)",
        o.fp.decisions,
        o.idle_total,
        pct(o.idle_total, o.fp.decisions)
    );
    println!(
        "  taken at/above the medium gate: {} ({:.2}%), of which {} could still deepen ({:.1}%)",
        o.at_or_above_gate,
        pct(o.at_or_above_gate, o.fp.decisions),
        o.gate_deepen_possible,
        pct(o.gate_deepen_possible, o.at_or_above_gate)
    );

    println!("\nwhy Idle ({} total):", o.idle_total);
    let rows = [
        ("below gate, cannot afford next infra level", o.idle.gated_broke, ""),
        ("below gate, infra already at k_potential", o.idle.gated_capped, "  <- can NEVER reach the gate"),
        ("below gate, other", o.idle.gated_other, ""),
        ("at/above gate, no candidates", o.idle.open_no_candidates, ""),
        ("at/above gate, candidates but too poor", o.idle.open_broke, ""),
        ("at/above gate, other", o.idle.open_other, ""),
    ];
    for (what, n, note) in rows {
        println!("  {what:<44} {n:>7}  ({:.1}%){note}", pct(n, o.idle_total));
    }

    let gated_idle = o.idle.gated_broke + o.idle.gated_capped + o.idle.gated_other;
    if gated_idle > 0 {
        println!(
            "\nmean stockpile among below-gate Idle decisions: {:.3} (mean infra cost saved toward: {:.3})",
            o.gated_stockpile_sum / gated_idle as f64,
            o.gated_cost_sum / gated_idle as f64
        );
    }

    println!("\n\n=== Part 3: why wasn't every planet scanned? ===");
    println!("galaxy planets: {}   coverage targets: {}", o.galaxy_planets, o.targets);
    println!("distinct planets ever scanned by anyone: {}", o.distinct_scanned);
    println!("  of those, coverage targets: {}", o.scanned_targets);
    println!("last scan report received: t={:.0} yr of {:.0}", o.last_scan_time, cfg.horizon_years);

    println!("\nsurvey fleet:");
    println!("  scout entities that ever flew: {}", o.scouts);
    println!(
        "  of which free at bootstrap = players({}) x survey_vehicles({}) = {}",
        PLAYERS,
        doctrine.survey_vehicles,
        PLAYERS * doctrine.survey_vehicles
    );
    println!("  LightVehicle build orders issued after bootstrap: {}", o.fp.lights);
    println!("  hops completed: {}   chains run to termination: {}", o.hops_total, o.chains_exhausted);
    println!("  longest chain: {} hops (max_survey_hops = {})", o.max_hops_seen, cfg.max_survey_hops);

    // Derived, not asserted. An earlier version hardcoded "the fleet is never
    // replenished, because BuildOrder::LightVehicle is constructed nowhere" — true
    // when written, and printed directly underneath a line reporting 125 such
    // orders once the scout fix landed. A diagnostic that states conclusions its
    // own numbers contradict is worse than one that states none.
    let replenished = o.fp.lights > 0;
    let time_bound = o.last_scan_time >= cfg.horizon_years * 0.95;
    let unexplored = 100.0 - o.distinct_scanned as f64 / o.galaxy_planets as f64 * 100.0;
    println!("\nverdict:");
    if replenished {
        println!(
            "  survey scales with the empire — {} of the {} scouts were built by production\n  \
             centers after bootstrap, so exploration is no longer a fixed budget.",
            o.fp.lights, o.scouts
        );
    } else {
        println!(
            "  survey is capped at the bootstrap fan-out: {} scouts, none replenished, so\n  \
             exploration is a fixed product no economy knob can move.",
            o.scouts
        );
    }
    if time_bound {
        println!(
            "  The horizon binds: scanning was still running at t={:.0} of {:.0} yr, with {unexplored:.1}%\n  \
             of the galaxy still dark. A longer horizon would explore further.",
            o.last_scan_time, cfg.horizon_years
        );
    } else {
        println!(
            "  The horizon does NOT bind: the last scan landed at t={:.0} of {:.0} yr, leaving\n  \
             {unexplored:.1}% of the galaxy dark with time to spare.",
            o.last_scan_time, cfg.horizon_years
        );
    }
}
