//! Search for a configuration that minimizes time-to-100%-K>0-coverage
//! against the standard test bed (`coverage_time.rs`'s methodology).
//!
//! **Diagnosis first, sweep second** — this session's actual finding, not a
//! guess: tracing one seed with full logging (`Vehicles`/`Production`
//! categories) showed colonization *halting completely* at year ~1300 of a
//! 4000-year horizon, every subsequent production decision `Idle`. Checking
//! why: a colony's population is capped at `K = min(hab, bio, infra)`, infra
//! can only deepen while `infra+1 ≤ K_potential`, and **196/200 wild
//! planets don't hold enough *total* extractable local mineral density (5,
//! the cost of deepening 1→2→3) to ever afford reaching infra 3 from their
//! own mining alone**.
//!
//! First fix tried (a later turn, corrected): seed every colony with free
//! minerals at founding, mirroring `homeworld_start_minerals`. **Wrong** —
//! confirmed: *"No mineral seed for colonies. Mineral seed is only for
//! homeworld. Mining outposts are supposed to cover the mineral deficiency;
//! autopilot must haul minerals to where they are needed for infrastructure
//! upgrades and ship building."* The actual fix: `Simulation::
//! most_needed_center` — every Freighter, on loading, now routes to
//! whichever *owned production center currently has the highest mineral
//! pressure*, not back to whichever center happened to build it. This file
//! searches the parameters that make that hauling loop effective, now that
//! the underlying trap is closed by the right mechanism.
//!
//! Coordinate descent, not a full grid: with N parameters each swept over
//! ~5 values, a grid is `5^N` full test-bed runs; coordinate descent is
//! `~5×N` — vary one parameter at a time, keep the best value, move to the
//! next. Standard technique for expensive-to-evaluate objectives where the
//! full joint search isn't affordable — see e.g. [Wright, "Coordinate
//! Descent Algorithms" (2015)](https://arxiv.org/abs/1502.04759) for the
//! general method. It won't find a joint optimum if parameters interact
//! strongly, but at ~50ms/trial here a full grid is a future refinement,
//! not a requirement to report *a* result now.
//!
//! **Superseded as a method, kept as a record.** This harness has no error
//! bars, no common-random-numbers pairing, and no significance test, and it
//! produced this project's first two measurement artifacts — a phantom
//! optimum at `medium_fleet_size = 8` and a "cliff" at 12 that was a
//! normalisation bug, not economics (see Round 1's comment below). Every
//! knob it swept in isolation is now covered better, and *checkably*, by
//! `examples/gradient_probe.rs` (elasticity + standard error) and
//! `examples/gradient_step.rs` (a verified line search along the measured
//! gradient) — see CLAUDE.md §2 "How to search" and `hyades_todo.md` T-45.
//! Prefer those for any future doctrine work; this file stays for the
//! diagnosis narrative above, which is still the right account of the
//! freighter-routing fix.
//!
//! Run with:  `cargo run --release --example min_time_search`

use std::collections::{HashMap, HashSet};

use hyades_engine::autopilot::{Autopilot, BaselineAutopilot, Doctrine};
use hyades_engine::log::{LogCategory, LogEvent, LogFilter};
use hyades_engine::prelude::*;

/// **Four seeds, cut from ten — and this is a real reduction in what the search
/// proves, so it is recorded rather than hidden** (CLAUDE.md §2: cut samples,
/// not the question; say what the trim cost).
///
/// The trim is not about the 60-second rule — this job is offline and untimed.
/// It is about *finishing at all*. Ratifying `trade_decay_lambda` tripled
/// coverage and therefore entity count, which took the search from CLAUDE.md's
/// recorded ~40 minutes to ~110, and two consecutive runs were killed by
/// container restarts before they got past round two. A search that never
/// completes proves nothing; four seeds runs in ~45 minutes and, because
/// coordinate descent prints its best value per round as it goes, even a
/// truncated run yields ratified values for the rounds that finished.
///
/// **What it costs:** less variance coverage, so a value that wins by a point
/// or two is inside the noise. Treat the *ordering* within a round as the
/// result and the magnitude as indicative; re-confirm any close call on the
/// full ten before shipping it as a default. The full bed is preserved below
/// for exactly that.
///
/// Seeds chosen to span the difficulty range rather than to be the first four:
/// 42 is the hard one (it ran ~40% below 1 and 7 in the `lambda_routing`
/// sweep), so it is kept deliberately.
const TEST_BED_SEEDS: &[u64] = &[1, 7, 42, 31337];

/// The full ten-seed bed. Use it to confirm a ratification, not to search.
#[allow(dead_code)]
const FULL_TEST_BED_SEEDS: &[u64] = &[1, 7, 42, 55, 99, 123, 2024, 31337, 8675309, 271828];
const PLAYERS: usize = 3;

fn coverage_targets(galaxy: &Galaxy) -> HashSet<PlanetId> {
    galaxy.planets.iter().filter(|p| p.habitability.min(p.biosphere) > 0.01).map(|p| p.id).collect()
}

/// One trial: (covered, total, completion_time_if_full).
fn trial(seed: u64, cfg: SimConfig, doctrine: Doctrine) -> (usize, usize, Option<f64>) {
    let galaxy = Galaxy::generate(GalaxyConfig::new(PLAYERS, seed)).unwrap();
    let targets = coverage_targets(&galaxy);
    let total = targets.len();

    let autopilots: Vec<Box<dyn Autopilot>> =
        (0..PLAYERS).map(|_| Box::new(BaselineAutopilot::new(doctrine)) as Box<_>).collect();
    let mut sim = Simulation::new(galaxy, cfg, autopilots);
    sim.set_log_filter(LogFilter::none().with(LogCategory::Vehicles));
    sim.run();

    let mut covered_at: HashMap<PlanetId, f64> = HashMap::new();
    for p in sim.snapshot().planets.iter().filter(|p| p.is_homeworld) {
        if targets.contains(&p.id) {
            covered_at.insert(p.id, 0.0);
        }
    }
    for rec in sim.log().by_category(LogCategory::Vehicles) {
        if let LogEvent::ColonyFounded { planet, .. } = rec.event {
            if targets.contains(&planet) {
                covered_at.entry(planet).or_insert(rec.time);
            }
        }
    }
    let covered = covered_at.len();
    let completion = if covered == total { Some(covered_at.values().cloned().fold(0.0_f64, f64::max)) } else { None };
    (covered, total, completion)
}

/// Score a configuration against the whole test bed: (seeds fully covered,
/// mean coverage fraction across all seeds, mean completion time among
/// seeds that did fully cover — `f64::INFINITY` if none did).
fn score(cfg: SimConfig, doctrine: Doctrine) -> (usize, f64, f64) {
    let mut full = 0;
    let mut frac_sum = 0.0;
    let mut completions = Vec::new();
    for &seed in TEST_BED_SEEDS {
        let (covered, total, completion) = trial(seed, cfg, doctrine);
        frac_sum += covered as f64 / total as f64;
        if let Some(t) = completion {
            full += 1;
            completions.push(t);
        }
    }
    let mean_frac = frac_sum / TEST_BED_SEEDS.len() as f64;
    let mean_completion =
        if completions.is_empty() { f64::INFINITY } else { completions.iter().sum::<f64>() / completions.len() as f64 };
    (full, mean_frac, mean_completion)
}

/// Better = more full-coverage seeds first, then higher mean fraction, then
/// (only among ties on those) lower mean completion time.
fn better(a: (usize, f64, f64), b: (usize, f64, f64)) -> bool {
    if a.0 != b.0 {
        return a.0 > b.0;
    }
    if (a.1 - b.1).abs() > 1e-9 {
        return a.1 > b.1;
    }
    a.2 < b.2
}

/// Sweep one f64 field of `cfg` (via a getter/setter closure pair) over
/// `values`, holding everything else fixed; return the best value found and
/// its score.
fn sweep_config(
    label: &str,
    cfg: &mut SimConfig,
    doctrine: Doctrine,
    values: &[f64],
    set: impl Fn(&mut SimConfig, f64),
) -> (usize, f64, f64) {
    println!("\nSweeping {label}:");
    let mut best_val = values[0];
    let mut best_score = (0, 0.0, f64::INFINITY);
    for &v in values {
        let mut trial_cfg = *cfg;
        set(&mut trial_cfg, v);
        let s = score(trial_cfg, doctrine);
        println!(
            "  {label}={v:<8.2}  full_coverage={}/{}  mean_frac={:.1}%  mean_completion={}",
            s.0,
            TEST_BED_SEEDS.len(),
            s.1 * 100.0,
            if s.2.is_finite() { format!("{:.0} yr", s.2) } else { "n/a".to_string() }
        );
        if better(s, best_score) {
            best_score = s;
            best_val = v;
        }
    }
    set(cfg, best_val);
    println!("  -> best: {label}={best_val:.2}");
    best_score
}

fn sweep_doctrine(
    label: &str,
    cfg: SimConfig,
    doctrine: &mut Doctrine,
    values: &[f64],
    set: impl Fn(&mut Doctrine, f64),
) -> (usize, f64, f64) {
    println!("\nSweeping {label}:");
    let mut best_val = values[0];
    let mut best_score = (0, 0.0, f64::INFINITY);
    for &v in values {
        let mut trial_doc = *doctrine;
        set(&mut trial_doc, v);
        let s = score(cfg, trial_doc);
        println!(
            "  {label}={v:<8.2}  full_coverage={}/{}  mean_frac={:.1}%  mean_completion={}",
            s.0,
            TEST_BED_SEEDS.len(),
            s.1 * 100.0,
            if s.2.is_finite() { format!("{:.0} yr", s.2) } else { "n/a".to_string() }
        );
        if better(s, best_score) {
            best_score = s;
            best_val = v;
        }
    }
    set(doctrine, best_val);
    println!("  -> best: {label}={best_val:.2}");
    best_score
}

fn main() {
    println!("Minimum-time search — {PLAYERS} seats, {} standard-test-bed seeds", TEST_BED_SEEDS.len());
    println!("Fix in place: need-based freighter routing (see module doc — this is what");
    println!("makes the search meaningful at all; without it, ~98% of colonies could never");
    println!("reach the medium-vehicle gate on their own local mining, permanently, regardless");
    println!("of any doctrine or fleet-cost setting swept below).\n");

    let mut cfg = SimConfig::new(0);
    let mut doctrine = Doctrine::default();

    let baseline = score(cfg, doctrine);
    println!(
        "Starting point (all defaults, with need-based hauling now in place): full={}/{} mean_frac={:.1}%",
        baseline.0,
        TEST_BED_SEEDS.len(),
        baseline.1 * 100.0
    );

    // Round 1: cheaper colonizers.
    //
    // **Superseded — kept as the historical record of how this harness's own
    // limits produced two artifacts, corrected in order.**
    //
    // (1) The `[3, 4, 6, 8, 12]` sweep reported 8.0 optimal at 25.1% (vs 15.2%
    // default) and read 12.0's collapse to 0.3% as a cliff. Coordinate descent
    // has no error bars, so nothing caught it. (2) The fix at the time —
    // `SimConfig::hull_ladder_fault` refusing `r_M < 1.25` — was *itself* an
    // artifact: capacity was normalised against the live Medium radius, so the
    // "explosion" was a denominator going to zero, not an economic finding.
    // Normalising against a fixed reference (see the doc comment on
    // `hull_ladder_fault`) removed the false bound; only a genuinely inverted
    // ladder (`medium_fleet_size >= limited_fleet_size`) is refused now.
    //
    // The real gradient (`examples/gradient_probe.rs`) found
    // `medium_fleet_size` the single largest lever at +32.7 ± 3.6 points/ln, and
    // a verified line search (`examples/gradient_step.rs`) moved it to **4.45**
    // jointly with three other knobs for a paired +10.99 ± 1.86 points. That is
    // the shipped default now — see `hyades_todo.md` T-45. This coordinate
    // sweep is retained to show what it cost to get the wrong answer twice, not
    // as a method to repeat: it has no CRN, no pairing, and no significance
    // test, and every one of this project's four measurement artifacts came
    // through exactly this kind of sweep.
    sweep_config("medium_fleet_size", &mut cfg, doctrine, &[2.0, 2.5, 3.0, 4.0, 5.0], |c, v| c.medium_fleet_size = v);

    // Round 2: how much cargo one freighter haul actually moves — directly
    // gates how fast hauling can relieve a starved center's pressure.
    // Range shifted downward: `coverage_trace` shows this saturates above ~5,
    // with 2.5 the best of the old points, so the old [2.5, 15] spent four of
    // five trials on a plateau. The interesting region is *below* 5.
    sweep_config("cargo_unit_size", &mut cfg, doctrine, &[1.0, 1.5, 2.5, 4.0, 6.0], |c, v| c.cargo_unit_size = v);

    // Round 3: how fast an outpost extracts in the first place — no amount
    // of hauling helps if there's nothing sitting at the outpost to load.
    sweep_config("outpost_mining_fraction", &mut cfg, doctrine, &[0.1, 0.2, 0.3, 0.4, 0.5], |c, v| {
        c.outpost_mining_fraction = v
    });

    // Round 4: reinvest_bias — how hard to favor expansion over deepening.
    // **Not a dial in practice, a cliff.** production_choice weighs
    // b*deepen_headroom against (1-b)*score; headroom is bounded by
    // k_potential <= ~4 while score runs well above it, so expansion wins
    // everywhere below roughly b = 0.95 and deepening wins above. The old
    // [0.0, 0.5] range sat entirely on one side and measured a flat line —
    // `coverage_trace` confirms an identical run fingerprint across all five
    // of those values, before *and* after the snowball ratification. Sample
    // across the switch instead, or this round is five wasted trials.
    sweep_doctrine("reinvest_bias", cfg, &mut doctrine, &[0.0, 0.5, 0.9, 0.97, 1.0], |d, v| d.reinvest_bias = v);

    // Round 5: growth_rate — how fast pop climbs toward whatever K is
    // currently achievable.
    sweep_doctrine("growth_rate", cfg, &mut doctrine, &[0.3, 0.5, 0.7, 1.0, 1.3], |d, v| d.growth_rate = v);

    // Round 6: centrality_scale (T-07). The ranking term is `exp(-dist /
    // centrality_scale)`, so the scale sets the distance at which a candidate
    // stops looking "near my holdings". It was tuned against a ~25 ly galaxy
    // and the galaxy is now hundreds of ly across, which is the standing
    // suspicion that this term is *saturated* — every candidate scoring the
    // same centrality means the term contributes nothing and the ranking has
    // silently lost a dimension. Swept across three orders of magnitude so the
    // answer is measured rather than asserted: if the best value sits at an
    // endpoint the range was still too narrow, and if the column is flat the
    // term is confirmed dead and should be retired rather than retuned.
    let final_score =
        sweep_doctrine("centrality_scale", cfg, &mut doctrine, &[25.0, 75.0, 150.0, 400.0, 1000.0], |d, v| {
            d.rank.centrality_scale = v
        });

    println!("\n=== Result ===");
    println!(
        "medium_fleet_size={:.1}  cargo_unit_size={:.1}  outpost_mining_fraction={:.2}  reinvest_bias={:.2}  growth_rate={:.2}  centrality_scale={:.0}",
        cfg.medium_fleet_size,
        cfg.cargo_unit_size,
        cfg.outpost_mining_fraction,
        doctrine.reinvest_bias,
        doctrine.growth_rate,
        doctrine.rank.centrality_scale
    );
    println!(
        "-> {}/{} seeds fully covered, mean coverage {:.1}%, mean completion time among finishers: {}",
        final_score.0,
        TEST_BED_SEEDS.len(),
        final_score.1 * 100.0,
        if final_score.2.is_finite() { format!("{:.0} yr", final_score.2) } else { "n/a".to_string() }
    );

    println!("\nPer-seed detail at the found configuration:");
    for &seed in TEST_BED_SEEDS {
        let (covered, total, completion) = trial(seed, cfg, doctrine);
        let status = completion.map(|t| format!("{t:.0} yr")).unwrap_or_else(|| "incomplete".into());
        println!("  seed {seed:>8}: {covered:>3}/{total} covered — {status}");
    }
}
