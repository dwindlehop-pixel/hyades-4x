//! **Which mining-outpost strategy produces the most colonies?**
//!
//! The mining surface has never been measured as a *strategy*. `gradient_probe`
//! covers `outpost_mining_fraction` and `center_mining_fraction` — how hard a
//! world is worked — but not the three knobs that decide *which* worlds become
//! outposts and *when an empire prefers mining to settling*:
//!
//! | knob | the strategic question it answers |
//! |---|---|
//! | `rank.mineral_high` | how rich must a low-K world be to be worth mining |
//! | `rank.w_mineral` | how much mineral value counts in the score that picks the outward move |
//! | `rank.mineral_pressure_gain` | how sharply the empire pivots to mining when it is broke |
//! | `mining_tick_years` | how often an outpost yields |
//! | `density_floor` | when an outpost is called exhausted |
//!
//! Those five plus `outpost_mining_fraction` are the whole mining policy. The
//! objective is unchanged and is the one the user asked for: **colonies**.
//! `coverage` here is owned planets ∩ colonizable targets, and an outpost does
//! *not* take ownership (`sys_mining_arrive` writes `knowledge.exploited`, never
//! `world.owner`), so this counts colonies exactly.
//!
//! ## Method
//!
//! Identical to `gradient_probe`, and for the reasons documented there: common
//! random numbers, paired central differences, elasticity rather than slope, and
//! a standard error on every number. Nothing inside 2 SE of zero is a finding.
//!
//! **Why this is a separate driver rather than five more rows in
//! `gradient_probe`.** Cost. Each trial is a full 4,000-year snowball run, and
//! the container these run in is ephemeral — CLAUDE.md §2 records three sweeps
//! killed mid-flight. So this file takes the knobs to probe as arguments,
//! letting the work be split into chunks that each finish well inside ten
//! minutes, and flushes after every row so a killed run still yields the rows
//! that finished.
//!
//! ```text
//! cargo run --release --example mining_probe                      # every knob
//! cargo run --release --example mining_probe -- mineral_high      # one knob
//! cargo run --release --example mining_probe -- base w_mineral    # + the operating point
//! cargo run --release --example mining_probe -- step 0.5          # verify a joint move
//! ```
//!
//! `step` is the second half of the method: it applies a normalised move along
//! the measured gradient at fraction α and reports the paired improvement on the
//! same seeds, which is what turns a gradient into a ratifiable change.

use std::collections::HashSet;
use std::io::Write;

use hyades_engine::autopilot::{Autopilot, BaselineAutopilot, Doctrine};
use hyades_engine::log::{LogCategory, LogEvent, LogFilter};
use hyades_engine::prelude::*;

/// Common random numbers: the *same* seeds for every configuration — the
/// standard bed `gradient_probe`, `gradient_step` and `reach_limit` all use.
const SEEDS: &[u64] = &[1, 7, 42, 31337];
/// A wider bed, for settling a call the standard four cannot resolve. Not a
/// replacement for `SEEDS`: every elasticity in §5b is measured on the standard
/// bed so it sits beside `gradient_probe`'s numbers. This exists because the
/// recycling A/B landed at 1.86 SE — a real effect the four-seed bed could not
/// separate from noise, because seed 42 alone swings +3.0 points.
const SEEDS_WIDE: &[u64] = &[1, 7, 42, 31337, 2, 13, 99, 4242];
const PLAYERS: usize = 3;
/// Relative step for the central difference, matching `gradient_probe`.
const DELTA: f64 = 0.10;

fn coverage_targets(galaxy: &Galaxy) -> HashSet<PlanetId> {
    galaxy.planets.iter().filter(|p| p.habitability.min(p.biosphere) > 0.01).map(|p| p.id).collect()
}

/// Colony fraction for one seed under one configuration. Ownership is the
/// colony test: outposts never set an owner.
fn trial(seed: u64, cfg: SimConfig, doctrine: Doctrine) -> f64 {
    let galaxy = Galaxy::generate(GalaxyConfig::new(PLAYERS, seed)).unwrap();
    let targets = coverage_targets(&galaxy);
    let total = targets.len().max(1);
    let autopilots: Vec<Box<dyn Autopilot>> =
        (0..PLAYERS).map(|_| Box::new(BaselineAutopilot::new(doctrine)) as Box<_>).collect();
    let mut sim = Simulation::new(galaxy, cfg, autopilots);
    sim.run();
    let snap = sim.snapshot();
    snap.planets.iter().filter(|p| p.owner.is_some() && targets.contains(&p.id)).count() as f64 / total as f64
}

/// Per-seed vector — the CRN unit. Never collapse to a mean before differencing.
fn profile(cfg: SimConfig, doctrine: Doctrine) -> Vec<f64> {
    profile_on(SEEDS, cfg, doctrine)
}

fn profile_on(seeds: &[u64], cfg: SimConfig, doctrine: Doctrine) -> Vec<f64> {
    seeds.iter().map(|&s| trial(s, cfg, doctrine)).collect()
}

fn mean(v: &[f64]) -> f64 {
    v.iter().sum::<f64>() / v.len() as f64
}

fn stderr(v: &[f64]) -> f64 {
    let n = v.len() as f64;
    if n < 2.0 {
        return f64::INFINITY;
    }
    let m = mean(v);
    (v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (n - 1.0) / n).sqrt()
}

struct Knob {
    name: &'static str,
    value: f64,
    set: fn(&mut SimConfig, &mut Doctrine, f64),
}

fn knobs() -> Vec<Knob> {
    let c = SimConfig::new(0);
    let d = Doctrine::default();
    vec![
        // --- which worlds become outposts ---
        Knob { name: "mineral_high", value: d.rank.mineral_high, set: |_, d, v| d.rank.mineral_high = v },
        Knob { name: "w_mineral", value: d.rank.w_mineral, set: |_, d, v| d.rank.w_mineral = v },
        Knob {
            name: "mineral_pressure_gain",
            value: d.rank.mineral_pressure_gain,
            set: |_, d, v| d.rank.mineral_pressure_gain = v,
        },
        // --- how hard they are worked ---
        Knob {
            name: "outpost_mining_fraction",
            value: c.outpost_mining_fraction,
            set: |c, _, v| c.outpost_mining_fraction = v,
        },
        Knob { name: "mining_tick_years", value: c.mining_tick_years, set: |c, _, v| c.mining_tick_years = v },
        Knob { name: "density_floor", value: c.density_floor, set: |c, _, v| c.density_floor = v },
    ]
}

fn print_operating_point() {
    let here = profile(SimConfig::new(0), Doctrine::default());
    println!(
        "Operating point: {:.2}% ± {:.2} colonies   per-seed {:?}",
        mean(&here) * 100.0,
        stderr(&here) * 100.0,
        here.iter().map(|x| format!("{:.1}%", x * 100.0)).collect::<Vec<_>>()
    );
    std::io::stdout().flush().ok();
}

/// One knob's paired central difference. Prints its own row and flushes, so a
/// killed run still leaves every row it completed.
fn probe(k: &Knob) {
    let (base_cfg, base_doc) = (SimConfig::new(0), Doctrine::default());
    let mut hi_cfg = base_cfg;
    let mut hi_doc = base_doc;
    (k.set)(&mut hi_cfg, &mut hi_doc, k.value * (1.0 + DELTA));
    let mut lo_cfg = base_cfg;
    let mut lo_doc = base_doc;
    (k.set)(&mut lo_cfg, &mut lo_doc, k.value * (1.0 - DELTA));

    if hi_cfg.hull_ladder_fault().is_some() || lo_cfg.hull_ladder_fault().is_some() {
        println!("  {:<24} skipped — step leaves the legal hull ladder", k.name);
        return;
    }

    let hi = profile(hi_cfg, hi_doc);
    let lo = profile(lo_cfg, lo_doc);
    let diffs: Vec<f64> = hi.iter().zip(lo.iter()).map(|(a, b)| a - b).collect();
    let elasticity = mean(&diffs) / (2.0 * DELTA) * 100.0;
    let se = stderr(&diffs) / (2.0 * DELTA) * 100.0;
    let verdict = if se == 0.0 && elasticity == 0.0 {
        "INERT — moved no seed at all"
    } else if elasticity.abs() < 2.0 * se {
        if elasticity.abs() < 0.5 {
            "flat — inert here"
        } else {
            "~noise — needs a bigger bed"
        }
    } else if elasticity > 0.0 {
        "raise it"
    } else {
        "lower it"
    };
    println!("  {:<24} value {:>9.4}   d/dln x {:>+8.2} ± {:.2}   {verdict}", k.name, k.value, elasticity, se);
    std::io::stdout().flush().ok();
}

/// The measured gradient, filled in from a completed probe. Elasticities are
/// percentage points of colony coverage per unit of `ln(knob)`; a knob left out
/// was inside 2 SE of zero and is deliberately not moved — stepping along noise
/// is how a bed this size produces an artifact.
const GRADIENT: &[(&str, f64)] = &[
    ("mineral_high", MINERAL_HIGH_E),
    ("w_mineral", W_MINERAL_E),
    ("mineral_pressure_gain", PRESSURE_GAIN_E),
    ("outpost_mining_fraction", OUTPOST_FRACTION_E),
    ("mining_tick_years", TICK_YEARS_E),
];
const MINERAL_HIGH_E: f64 = 0.0;
const W_MINERAL_E: f64 = 0.0;
const PRESSURE_GAIN_E: f64 = 0.0;
const OUTPOST_FRACTION_E: f64 = 0.0;
const TICK_YEARS_E: f64 = 0.0;

/// Apply the normalised gradient move at fraction `alpha` in log space:
/// `x ← x · exp(α · e / max|e|)`. Normalising by the largest elasticity keeps
/// the step scale-free and bounded, which is what makes α comparable between
/// this move and `gradient_step`'s.
fn stepped(alpha: f64) -> (SimConfig, Doctrine) {
    let mut cfg = SimConfig::new(0);
    let mut doc = Doctrine::default();
    let norm = GRADIENT.iter().map(|(_, e)| e.abs()).fold(0.0_f64, f64::max).max(1e-12);
    for k in knobs() {
        if let Some((_, e)) = GRADIENT.iter().find(|(n, _)| *n == k.name) {
            (k.set)(&mut cfg, &mut doc, k.value * (alpha * e / norm).exp());
        }
    }
    (cfg, doc)
}

fn verify_step(alpha: f64) {
    let base = profile(SimConfig::new(0), Doctrine::default());
    let (cfg, doc) = stepped(alpha);
    println!("Stepped configuration (alpha = {alpha}):");
    let b = SimConfig::new(0);
    let bd = Doctrine::default();
    println!("  mineral_high            {:.4} -> {:.4}", bd.rank.mineral_high, doc.rank.mineral_high);
    println!("  w_mineral               {:.4} -> {:.4}", bd.rank.w_mineral, doc.rank.w_mineral);
    println!("  mineral_pressure_gain   {:.4} -> {:.4}", bd.rank.mineral_pressure_gain, doc.rank.mineral_pressure_gain);
    println!("  outpost_mining_fraction {:.4} -> {:.4}", b.outpost_mining_fraction, cfg.outpost_mining_fraction);
    println!("  mining_tick_years       {:.4} -> {:.4}", b.mining_tick_years, cfg.mining_tick_years);
    std::io::stdout().flush().ok();

    let moved = profile(cfg, doc);
    let diffs: Vec<f64> = moved.iter().zip(base.iter()).map(|(a, b)| (a - b) * 100.0).collect();
    println!(
        "  baseline {:.2}%  ->  stepped {:.2}%   paired {:+.2} ± {:.2} points   per-seed {:?}",
        mean(&base) * 100.0,
        mean(&moved) * 100.0,
        mean(&diffs),
        stderr(&diffs),
        diffs.iter().map(|d| format!("{d:+.1}")).collect::<Vec<_>>()
    );
    std::io::stdout().flush().ok();
}

/// **The census the knobs cannot see.**
///
/// A mining pair is built for one rock: `Shuttle { outpost, .. }` fixes the
/// pickup leg at spawn and only the *delivery* leg is need-routed, so without
/// recycling a rock hitting `density_floor` ends both hulls' working lives.
///
/// Note what is and is not a hull statistic here. **Rock lifetime is a property
/// of the rock** — recycling cannot change how long a world holds ore, and the
/// first block below is identical with the flag either way. What recycling
/// changes is what happens to the *hulls* afterwards, so the second block
/// counts missions against distinct hulls, and the third counts the years a
/// hull spends parked with nothing to do.
fn census(seed: u64, recycle: bool) {
    let mut cfg = SimConfig::new(seed);
    cfg.recycle_mining_pairs = recycle;
    let doctrine = Doctrine::default();
    let galaxy = Galaxy::generate(GalaxyConfig::new(PLAYERS, seed)).unwrap();
    let autopilots: Vec<Box<dyn Autopilot>> =
        (0..PLAYERS).map(|_| Box::new(BaselineAutopilot::new(doctrine)) as Box<_>).collect();
    let mut sim = Simulation::new(galaxy, cfg, autopilots);
    sim.set_log_filter(LogFilter::none().with(LogCategory::Mining).with(LogCategory::Vehicles));
    sim.run();

    use std::collections::HashMap;
    let horizon = cfg.horizon_years;
    let mut opened: HashMap<PlanetId, f64> = HashMap::new();
    let mut exhausted: Vec<(PlanetId, f64)> = Vec::new();
    let mut extracted_total = 0.0_f64;
    // Missions launched per hull. A re-tasked hull logs a second `VehicleSpawned`
    // under the same entity, so `missions - hulls` is exactly the recycling count.
    let mut miner_missions: HashMap<hyades_engine::sim::Entity, u32> = HashMap::new();
    let mut freighter_missions: HashMap<hyades_engine::sim::Entity, u32> = HashMap::new();
    // When a hull last went idle, and whether anything picked it up again.
    let mut idle_since: HashMap<hyades_engine::sim::Entity, f64> = HashMap::new();
    let mut last_launch: HashMap<hyades_engine::sim::Entity, f64> = HashMap::new();
    // Which miner is on which rock — last arrival wins, since a recycled hull
    // can work several in a run.
    let mut operator_of: HashMap<PlanetId, hyades_engine::sim::Entity> = HashMap::new();

    for rec in sim.log().iter() {
        match rec.event {
            LogEvent::VehicleParked { vehicle, role: Role::Miner, at, .. } => {
                opened.entry(at).or_insert(rec.time);
                operator_of.insert(at, vehicle);
            }
            LogEvent::MiningExhausted { planet } => exhausted.push((planet, rec.time)),
            LogEvent::MineralsExtracted { amount, .. } => extracted_total += amount,
            LogEvent::VehicleSpawned { vehicle, role: Role::Miner, .. } => {
                *miner_missions.entry(vehicle).or_insert(0) += 1;
                last_launch.insert(vehicle, rec.time);
                idle_since.remove(&vehicle);
            }
            LogEvent::VehicleSpawned { vehicle, role: Role::Freighter, .. } => {
                *freighter_missions.entry(vehicle).or_insert(0) += 1;
                last_launch.insert(vehicle, rec.time);
                idle_since.remove(&vehicle);
            }
            // Freighters: the park-on-a-dead-rock (flag off) and the Reserve
            // stand-down (flag on) are the same branch of `sys_freighter_arrive`
            // under two labels, so counting both keeps the comparison about
            // behaviour rather than about bookkeeping.
            LogEvent::VehicleParked { vehicle, role: Role::Reserve, .. }
            | LogEvent::VehicleParked { vehicle, role: Role::Freighter, .. } => {
                idle_since.insert(vehicle, rec.time);
            }
            _ => {}
        }
    }
    // Miners are the asymmetric case and have to be handled by hand: with the
    // flag off a stranded miner logs *nothing at all* when its rock dies, so
    // reading idleness off the log would credit the old behaviour with zero
    // idle years. Derive it from the rock's death instead, which is symmetric —
    // and re-check afterwards whether anything picked the hull up again.
    let exhausted_at: HashMap<PlanetId, f64> = exhausted.iter().copied().collect();
    for (rock, t_dead) in &exhausted_at {
        if let Some(&miner) = operator_of.get(rock) {
            idle_since.entry(miner).and_modify(|t| *t = t.min(*t_dead)).or_insert(*t_dead);
        }
    }
    // A hull re-tasked after its last idle moment is not idle at the horizon.
    for (e, t_last_launch) in &last_launch {
        if let Some(t_idle) = idle_since.get(e).copied() {
            if *t_last_launch > t_idle {
                idle_since.remove(e);
            }
        }
    }

    let lifetimes: Vec<f64> =
        exhausted.iter().filter_map(|(pid, t_end)| opened.get(pid).map(|t_open| t_end - t_open)).collect();
    let miner_hulls = miner_missions.len();
    let miner_flights: u32 = miner_missions.values().sum();
    let freighter_hulls = freighter_missions.len();
    let freighter_flights: u32 = freighter_missions.values().sum();
    let idle_years: f64 = idle_since.values().map(|t| horizon - t).sum();

    println!(
        "\nMining census — seed {seed}, {PLAYERS} seats, {horizon:.0} yr, recycling {}",
        if recycle { "ON" } else { "off" }
    );
    println!("  -- the rocks (recycling cannot change these) --");
    println!("  outposts opened                 : {:>8}", opened.len());
    println!(
        "  outposts mined out              : {:>8}  ({:.0}%)",
        exhausted.len(),
        pct(exhausted.len(), opened.len())
    );
    if !lifetimes.is_empty() {
        println!(
            "  mean productive life of a rock  : {:>8.0} yr",
            lifetimes.iter().sum::<f64>() / lifetimes.len() as f64
        );
    }
    println!("  dead rocks at the horizon       : {:>8}", exhausted_at.len());
    println!("  total minerals extracted        : {extracted_total:>8.0}");
    println!("  -- the hulls (what recycling is for) --");
    println!("  miner hulls built               : {miner_hulls:>8}   mining missions flown {miner_flights:>8}");
    println!(
        "  freighter hulls built           : {freighter_hulls:>8}   hauling missions flown {freighter_flights:>8}"
    );
    println!(
        "  missions taken by a re-used hull: {:>8}",
        (miner_flights as usize - miner_hulls) + (freighter_flights as usize - freighter_hulls)
    );
    println!("  idle hull-years at the horizon  : {idle_years:>8.0}");
    std::io::stdout().flush().ok();
}

fn pct(n: usize, d: usize) -> f64 {
    if d == 0 {
        0.0
    } else {
        n as f64 / d as f64 * 100.0
    }
}

/// **A/B the structural change**: recycling an exhausted mining pair against
/// leaving it parked, on the same seeds. This is not a gradient — it is a term
/// that is either in the model or not — so the honest test is a paired
/// difference at the operating point, not an elasticity.
fn compare_recycling(seeds: &[u64]) {
    // **Set both sides explicitly.** Reading either arm off the default is how
    // this driver once reported +0.00 ± 0.00 on all four seeds: the default had
    // just been flipped to `true`, so the "off" arm was running the same
    // configuration as the "on" arm. A bit-identical A/B is a harness bug
    // before it is a finding.
    let mut off_cfg = SimConfig::new(0);
    off_cfg.recycle_mining_pairs = false;
    let off = profile_on(seeds, off_cfg, Doctrine::default());
    let mut on_cfg = SimConfig::new(0);
    on_cfg.recycle_mining_pairs = true;
    let on = profile_on(seeds, on_cfg, Doctrine::default());
    let diffs: Vec<f64> = on.iter().zip(off.iter()).map(|(a, b)| (a - b) * 100.0).collect();
    println!(
        "  recycle_mining_pairs  {} seeds   off {:.2}%  ->  on {:.2}%   paired {:+.2} ± {:.2} points   per-seed {:?}",
        seeds.len(),
        mean(&off) * 100.0,
        mean(&on) * 100.0,
        mean(&diffs),
        stderr(&diffs),
        diffs.iter().map(|d| format!("{d:+.1}")).collect::<Vec<_>>()
    );
    std::io::stdout().flush().ok();
}

/// Throughput both ways, against the confirmed floor of 2.5 simulated-years
/// per real second (T-24). Recycling trades *builds* for *active mines*, and
/// entity count is the first-order cost in this engine, so the term has to be
/// checked against the floor rather than assumed cheap. `Instant` is fine here:
/// the no-clock invariant binds the engine, not a driver measuring it.
fn bench() {
    for recycle in [false, true] {
        let mut cfg = SimConfig::new(1);
        cfg.recycle_mining_pairs = recycle;
        let galaxy = Galaxy::generate(GalaxyConfig::new(PLAYERS, 1)).unwrap();
        let autopilots: Vec<Box<dyn Autopilot>> =
            (0..PLAYERS).map(|_| Box::new(BaselineAutopilot::default()) as Box<_>).collect();
        let mut sim = Simulation::new(galaxy, cfg, autopilots);
        let t0 = std::time::Instant::now();
        sim.run();
        let secs = t0.elapsed().as_secs_f64();
        let snap = sim.snapshot();
        println!(
            "  recycling {:<3}  {:>7.1} yr/s  ({:.1}x the 2.5 floor)   vehicles {:>6}  colonies {:>6}",
            if recycle { "ON" } else { "off" },
            cfg.horizon_years / secs,
            cfg.horizon_years / secs / 2.5,
            snap.vehicles.len(),
            snap.planets.iter().filter(|p| p.owner.is_some()).count()
        );
        std::io::stdout().flush().ok();
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    println!(
        "Mining-outpost probe — {PLAYERS} seats, {} seeds (CRN), delta = ±{:.0}%, objective = colonies",
        SEEDS.len(),
        DELTA * 100.0
    );
    std::io::stdout().flush().ok();

    if let Some(i) = args.iter().position(|a| a == "step") {
        let alpha: f64 = args.get(i + 1).and_then(|s| s.parse().ok()).unwrap_or(0.5);
        verify_step(alpha);
        return;
    }
    if args.iter().any(|a| a == "bench") {
        bench();
        return;
    }
    if args.iter().any(|a| a == "recycle") {
        let seeds = if args.iter().any(|a| a == "wide") { SEEDS_WIDE } else { SEEDS };
        compare_recycling(seeds);
        return;
    }
    if args.iter().any(|a| a == "census") {
        census(1, false);
        census(1, true);
        return;
    }
    if args.iter().any(|a| a == "base") {
        print_operating_point();
    }
    let wanted: Vec<&String> = args.iter().filter(|a| *a != "base").collect();
    for k in knobs() {
        if wanted.is_empty() || wanted.iter().any(|w| *w == k.name) {
            probe(&k);
        }
    }
}
