//! **What is the limiter on reaching every planet in the test bed?**
//!
//! T-20 has coverage at ~49% of colonizable worlds inside the fixed 4,000-year
//! run and records that "the remaining headroom is not obviously in these
//! knobs". This driver answers *which* constraint the objective is actually
//! sitting against, by partitioning the target set rather than sweeping it — the
//! same interrogation-not-search method `coverage_trace` uses.
//!
//! Every coverage target falls in exactly one bucket, and each bucket names a
//! different limiter:
//!
//! | bucket | limiter it names |
//! |---|---|
//! | covered | — |
//! | `k_potential < k_high` | **classification**: no autopilot ever calls it a Colony |
//! | above the gate, never scanned by anyone | **survey** |
//! | above the gate, scanned, still unowned | **economy / transit inside the horizon** |
//!
//! The buckets are disjoint and exhaustive by construction, so the largest one
//! is the binding constraint, and a bucket that is nearly empty is a limiter
//! that is *not* binding however plausible it sounds.
//!
//! It also reports a second reach number — colonies **plus mining outposts** —
//! because "reaching a planet" and "colonizing a planet" are different
//! questions and the low-K half of the galaxy can only ever be reached the
//! second way (`rank` sends it to `MiningOutpost` or `Barren`).
//!
//! Run with: `cargo run --release --example reach_limit`

use std::collections::{HashMap, HashSet};

use hyades_engine::log::{LogCategory, LogEvent, LogFilter};
use hyades_engine::prelude::*;

const PLAYERS: usize = 3;
/// The **standard test bed** — the same four seeds `gradient_probe`,
/// `gradient_step` and `binding_check` score on, so these numbers sit directly
/// beside theirs. Four full 4,000-year runs, ~2 minutes; this is an offline
/// driver, not a CI step.
const SEEDS: [u64; 4] = [1, 7, 42, 31337];
/// One 500-year bucket per column in the founding-rate table.
const BUCKET: f64 = 500.0;
/// Thresholds for the ceiling curve — what a different `k_high` would make
/// colonizable at all, before any question of whether the economy gets there.
const K_HIGH_CURVE: [f64; 7] = [1.5, 2.0, 2.5, 3.0, 3.2, 3.5, 4.0];

/// Same definition `min_time_search`, `coverage_time` and `coverage_trace`
/// score against.
fn coverage_targets(galaxy: &Galaxy) -> HashSet<PlanetId> {
    galaxy.planets.iter().filter(|p| p.habitability.min(p.biosphere) > 0.01).map(|p| p.id).collect()
}

struct Reach {
    targets: usize,
    /// Targets at or above `k_high` — the ceiling any colonization policy with
    /// these rank weights can reach, whatever the economy does.
    above_gate: usize,
    covered: usize,
    covered_above_gate: usize,
    /// Below the gate, so never a Colony/ProductionCenter candidate.
    blocked_by_class: usize,
    /// Of those, the ones a miner could still be sent to (mineral_value high).
    blocked_but_minable: usize,
    /// Above the gate but nobody ever received a scan report for it.
    unscanned_above_gate: usize,
    /// Above the gate, scanned by someone, still unowned at the horizon.
    scanned_idle_above_gate: usize,
    /// Targets that were above the gate at generation but sit **below** it in
    /// the final snapshot, because population drew their biosphere down. The
    /// gate is therefore not quite a fixed set — measuring the denominator
    /// after a run gives a smaller number than measuring it before.
    gate_erosion: usize,
    /// Distinct planets reached by a colony **or** a mining outpost.
    reached_any: usize,
    outposts: usize,
    /// Colonies founded per `BUCKET`-year slice.
    founding_rate: Vec<usize>,
    last_colony: f64,
    contested: u64,
}

fn run(seed: u64) -> Reach {
    let galaxy = Galaxy::generate(GalaxyConfig::new(PLAYERS, seed)).unwrap();
    let targets = coverage_targets(&galaxy);
    let cfg = SimConfig::new(seed);
    let doctrine = Doctrine::default();
    let w = doctrine.rank;

    // k_potential of an *unowned* world never moves: habitability is static and
    // biosphere is only drawn down by population, which only exists on an owned
    // one. So the pre-run value is the value the rank saw all run.
    let k_pot: HashMap<PlanetId, f64> =
        galaxy.planets.iter().map(|p| (p.id, p.habitability.min(p.biosphere))).collect();
    // Approximate: `mineral_value` also carries per-seat scarcity (1 or 2) and
    // live mineral pressure, both of which only *raise* it. Unit scarcity and
    // zero pressure therefore give a lower bound on the minable set.
    let minable: HashSet<PlanetId> = galaxy
        .planets
        .iter()
        .filter(|p| p.minerals.cyan + p.minerals.magenta + p.minerals.yellow >= w.mineral_high)
        .map(|p| p.id)
        .collect();

    let autopilots: Vec<Box<dyn Autopilot>> =
        (0..PLAYERS).map(|_| Box::new(BaselineAutopilot::new(doctrine)) as Box<_>).collect();
    let mut sim = Simulation::new(galaxy, cfg, autopilots);
    sim.set_log_filter(LogFilter::none().with(LogCategory::Vehicles).with(LogCategory::Scanning));
    sim.run();

    let mut founded: HashSet<PlanetId> = HashSet::new();
    let mut mined: HashSet<PlanetId> = HashSet::new();
    let mut scanned: HashSet<PlanetId> = HashSet::new();
    let buckets = (cfg.horizon_years / BUCKET).ceil() as usize;
    let mut founding_rate = vec![0usize; buckets];
    let mut last_colony = 0.0_f64;
    let mut contested = 0u64;

    for rec in sim.log().iter() {
        match rec.event {
            LogEvent::ColonyFounded { planet, .. } => {
                founded.insert(planet);
                last_colony = last_colony.max(rec.time);
                let b = ((rec.time / BUCKET) as usize).min(buckets - 1);
                founding_rate[b] += 1;
            }
            LogEvent::ColonyContested { .. } => contested += 1,
            LogEvent::VehicleParked { role: Role::Miner, at, .. } => {
                mined.insert(at);
            }
            LogEvent::ScanReceived { planet, .. } => {
                scanned.insert(planet);
            }
            _ => {}
        }
    }

    let snap = sim.snapshot();
    let mut covered: HashSet<PlanetId> = HashSet::new();
    for p in snap.planets.iter().filter(|p| p.is_homeworld) {
        if targets.contains(&p.id) {
            covered.insert(p.id);
        }
    }
    for pid in founded.iter().filter(|pid| targets.contains(pid)) {
        covered.insert(*pid);
    }

    let gate = |pid: &PlanetId| k_pot[pid] >= w.k_high;
    let above_gate = targets.iter().filter(|p| gate(p)).count();
    let uncovered: Vec<PlanetId> = targets.iter().copied().filter(|p| !covered.contains(p)).collect();
    let blocked: Vec<PlanetId> = uncovered.iter().copied().filter(|p| !gate(p)).collect();

    Reach {
        targets: targets.len(),
        above_gate,
        covered: covered.len(),
        covered_above_gate: covered.iter().filter(|p| gate(p)).count(),
        blocked_by_class: blocked.len(),
        blocked_but_minable: blocked.iter().filter(|p| minable.contains(p)).count(),
        unscanned_above_gate: uncovered.iter().filter(|p| gate(p) && !scanned.contains(p)).count(),
        scanned_idle_above_gate: uncovered.iter().filter(|p| gate(p) && scanned.contains(p)).count(),
        gate_erosion: snap
            .planets
            .iter()
            .filter(|p| targets.contains(&p.id) && k_pot[&p.id] >= w.k_high)
            .filter(|p| p.habitability.min(p.biosphere) < w.k_high)
            .count(),
        reached_any: covered.union(&mined.intersection(&targets).copied().collect()).count(),
        outposts: mined.len(),
        founding_rate,
        last_colony,
        contested,
    }
}

/// Galaxy-only: how the *ceiling* moves with `k_high`, and what it costs.
///
/// `k_high` does double duty in `rank` — above it a world is a Colony or a
/// Production Center, below it a mineral-rich world is a Mining outpost. So
/// lowering the gate to make more of the galaxy colonizable also **removes**
/// worlds from the class that funds colonization (R-AC17). This prints both
/// sides of that trade, and needs no simulation to do it.
fn ceiling_curve(seed: u64) {
    let galaxy = Galaxy::generate(GalaxyConfig::new(PLAYERS, seed)).unwrap();
    let targets: Vec<&Planet> = galaxy.planets.iter().filter(|p| p.habitability.min(p.biosphere) > 0.01).collect();
    let mineral_high = Doctrine::default().rank.mineral_high;
    println!("\n  ceiling curve (galaxy alone, no run):");
    println!("  {:>8}  {:>12}  {:>8}  {:>12}", "k_high", "colonizable", "share", "minable below");
    for k in K_HIGH_CURVE {
        let above = targets.iter().filter(|p| p.habitability.min(p.biosphere) >= k).count();
        let minable_below = targets
            .iter()
            .filter(|p| p.habitability.min(p.biosphere) < k)
            .filter(|p| p.minerals.cyan + p.minerals.magenta + p.minerals.yellow >= mineral_high)
            .count();
        println!("  {:>8.1}  {:>12}  {:>7.1}%  {:>12}", k, above, pct(above, targets.len()), minable_below);
    }
}

fn pct(n: usize, d: usize) -> f64 {
    if d == 0 {
        0.0
    } else {
        n as f64 / d as f64 * 100.0
    }
}

fn main() {
    println!("Reach limiter, {PLAYERS} seats, {:.0} yr, k_high = {}", SimConfig::new(1).horizon_years, {
        Doctrine::default().rank.k_high
    });

    for seed in SEEDS {
        let r = run(seed);
        println!("\n=== seed {seed} ===");
        println!("coverage targets (min(hab,bio) > 0.01) : {:>6}", r.targets);
        println!(
            "  of which k_potential >= k_high        : {:>6}  ({:.1}% — the classification ceiling)",
            r.above_gate,
            pct(r.above_gate, r.targets)
        );
        println!(
            "colonized                              : {:>6}  ({:.1}% of targets)",
            r.covered,
            pct(r.covered, r.targets)
        );
        println!(
            "  as a share of the reachable set       :         {:.1}% of the above-gate worlds",
            pct(r.covered_above_gate, r.above_gate)
        );
        println!("\nuncovered targets, by limiter:");
        println!(
            "  below k_high (never a colony candidate): {:>6}  ({:.1}% of all targets)",
            r.blocked_by_class,
            pct(r.blocked_by_class, r.targets)
        );
        println!("      of those, mineral-rich enough to mine: {:>6}", r.blocked_but_minable);
        println!("  above k_high, never scanned by anyone  : {:>6}", r.unscanned_above_gate);
        println!("  above k_high, scanned, still unowned   : {:>6}", r.scanned_idle_above_gate);
        println!(
            "\nreach counting mining outposts too     : {:>6}  ({:.1}%)",
            r.reached_any,
            pct(r.reached_any, r.targets)
        );
        println!("  distinct outposts worked              : {:>6}", r.outposts);
        println!("contested colonizations                 : {:>6}", r.contested);
        println!("above-gate at generation, below it now  : {:>6}  (biosphere drawn down by pop)", r.gate_erosion);
        print!("colonies founded per {BUCKET:.0} yr           : ");
        for (i, n) in r.founding_rate.iter().enumerate() {
            print!("{}{}", if i == 0 { "" } else { " " }, n);
        }
        println!("\nlast colony founded at                 : {:.0} yr", r.last_colony);
        ceiling_curve(seed);
    }
}
