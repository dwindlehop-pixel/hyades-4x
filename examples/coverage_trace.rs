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
//! Two things it does that a sweep cannot:
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
//! Run with: `cargo run --release --example coverage_trace`

use std::collections::{HashMap, HashSet};

use hyades_engine::autopilot::{Autopilot, BaselineAutopilot, BuildOrder, Doctrine};
use hyades_engine::log::{LogCategory, LogEvent, LogFilter};
use hyades_engine::prelude::*;

const PLAYERS: usize = 3;
const SEED: u64 = 1;

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
                let can_deepen = infra + 1.0 <= k_potential + 1e-9;
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
                    BuildOrder::ColonyVehicle { .. } => fp.colonies += 1,
                    BuildOrder::MiningPair { .. } => fp.mining_pairs += 1,
                    BuildOrder::LightVehicle { .. } => {}
                }
            }
            LogEvent::ColonyFounded { planet, .. } => {
                founded.insert(planet);
                last_colony_time = last_colony_time.max(rec.time);
            }
            LogEvent::ScanReceived { player, planet } => {
                known.insert((player, planet));
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
        "{:>12}  {:>10}  {:>8}  {:>9}  {:>13}  {:>8}",
        "value", "decisions", "deepens", "colonies", "mining_pairs", "covered"
    );
    let mut first: Option<Fingerprint> = None;
    let mut all_same = true;
    for &v in values {
        let mut cfg = SimConfig::new(SEED);
        let mut doc = Doctrine::default();
        apply(&mut cfg, &mut doc, v);
        let o = run(SEED, cfg, doc);
        println!(
            "{:>12.2}  {:>10}  {:>8}  {:>9}  {:>13}  {:>8}",
            v, o.fp.decisions, o.fp.deepens, o.fp.colonies, o.fp.mining_pairs, o.fp.covered
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

    isolate("cargo_unit_size", &[2.5, 5.0, 7.5, 10.0, 15.0], |c, _, v| c.cargo_unit_size = v);
    isolate("outpost_mining_fraction", &[0.1, 0.2, 0.3, 0.4, 0.5], |c, _, v| c.outpost_mining_fraction = v);
    isolate("reinvest_bias (the range min_time_search swept)", &[0.0, 0.1, 0.25, 0.4, 0.5], |_, d, v| {
        d.reinvest_bias = v
    });
    // The deepen-vs-expand dial compares b*headroom against (1-b)*score.
    // `headroom` is bounded by k_potential (<= 1), while `score` is a weighted
    // sum that runs well above 1, so deepening cannot win anywhere in [0, 0.5]
    // — the swept range sits entirely on one side of the switch. Push past it.
    isolate("reinvest_bias (past the range it swept)", &[0.5, 0.9, 0.99, 0.999, 1.0], |_, d, v| d.reinvest_bias = v);
    isolate("medium_fleet_size (the one that DID move it)", &[3.0, 4.0, 6.0, 8.0, 12.0], |c, _, v| {
        c.medium_fleet_size = v
    });

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

    // Survey reach is a hard product of three integers, none of which any swept
    // knob touches: each seat launches `survey_vehicles` scouts, each of which
    // makes at most `max_survey_hops` hops. Nothing can be colonized that was
    // never scanned, so this is a ceiling on coverage independent of economy.
    let doctrine = Doctrine::default();
    let cfg = SimConfig::new(SEED);
    let reach = PLAYERS * doctrine.survey_vehicles * cfg.max_survey_hops;
    println!(
        "  survey reach ceiling = players({}) x survey_vehicles({}) x max_survey_hops({}) = {}",
        PLAYERS, doctrine.survey_vehicles, cfg.max_survey_hops, reach
    );
    println!(
        "  => at most {:.1}% of the {} coverage targets can EVER be known, so 100% coverage\n     \
         is unreachable by construction and 0/10 full-coverage seeds is structural, not tuning",
        reach as f64 / o.targets as f64 * 100.0,
        o.targets
    );

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
}
