//! **Telemetry must not perturb what it measures.**
//!
//! Every census, ablation and throughput figure in this project is read off a
//! run with log categories enabled — `crew_census`, `haul_census`, `bank_mix`,
//! `colonizer_policy` and `cadence_throttle` all switch on a `LogFilter` and
//! then report numbers that get compared against runs that did not. Two
//! separate claims have to hold for that to be sound:
//!
//! - **Logging does not change the outcome.** An arithmetic identity, already
//!   pinned by `sim::tests::logging_does_not_affect_outcomes` — the log is a
//!   side channel, so the same seed gives the same run either way.
//! - **Logging does not change the *cost*.** This file. `CLAUDE.md` §2 now asks
//!   for the per-event budget alongside `yr/s`, and a throughput number taken
//!   from an instrumented run is only comparable to one taken from a bare run if
//!   the instrument is cheap. Otherwise every measurement is reading a system
//!   the measurement itself slowed down.
//!
//! **Timing tests are flaky by nature, so this one is built to be robust rather
//! than precise**, with three deliberate choices:
//!
//! - **Interleaved arms.** Runs alternate off/on/off/on rather than doing all of
//!   one arm then all of the other, so CPU frequency drift, a noisy neighbour
//!   and container throttling land on both arms equally. This is common random
//!   numbers applied to wall-clock: the *difference* is what is wanted, and
//!   pairing cancels the shared noise.
//! - **Minimum, not mean.** Interference is one-sided — nothing on this machine
//!   makes a run finish faster than its uncontended cost — so the minimum over
//!   repeats is the cleanest estimate of true cost, and a mean is mostly a
//!   measurement of what else was running.
//! - **A small galaxy, not a short horizon** (`CLAUDE.md` §2). The question is
//!   the *ratio* of two costs, which needs enough events to be stable and does
//!   not need the standard bed. Shrinking the field keeps the mechanism and the
//!   full horizon while making the test affordable.
//!
//! `Instant` here is fine: the no-clock invariant binds the engine, not a driver
//! measuring it — the same note `examples/mining_probe`'s bench carries.
use hyades_engine::log::LogFilter;
use hyades_engine::prelude::*;
use std::time::Instant;

const PLANETS: usize = 800;
const PLAYERS: usize = 3;
const HORIZON: f64 = 600.0;
const REPEATS: usize = 5;

/// The budget. Telemetry costing more than this makes every instrumented
/// measurement in the project a measurement of a different system.
///
/// **Measured at three scenario sizes, and the true cost is below this test's
/// own noise floor:**
///
/// | planets x horizon | events | ratio |
/// |---|---|---|
/// | 800 x 600 (shipped) | 15,602 | **0.995** |
/// | 1,500 x 800 | 59,197 | **0.974** |
/// | 2,500 x 1,000 | 148,622 | **1.020** |
///
/// Two of the three come out *under* 1.0, which is not a claim that recording
/// events makes the engine faster. It means `SimLog::push` returning early on a
/// disabled category costs less than the machine's run-to-run variance, so what
/// this test can honestly assert is a **bound**, not an estimate. 5% is about
/// twice the observed spread and roughly ten times any plausible true cost —
/// the right way round for a guard, which should fire on a regression that
/// matters and not on a slow afternoon.
///
/// The ratio is also stable across a 10x range of event counts, which is the
/// evidence that it is measuring a per-event property and not a fixed startup
/// cost being amortised differently.
const MAX_OVERHEAD: f64 = 1.05;

fn timed(seed: u64, filter: Option<LogFilter>) -> (f64, u64) {
    let mut gcfg = GalaxyConfig::new(PLAYERS, seed);
    gcfg.planet_count = PLANETS;
    let galaxy = Galaxy::generate(gcfg).unwrap();
    let mut cfg = SimConfig::new(seed);
    cfg.horizon_years = HORIZON;
    let mut sim = Simulation::with_baseline(galaxy, cfg);
    if let Some(f) = filter {
        sim.set_log_filter(f);
    }
    let t0 = Instant::now();
    let report = sim.run();
    (t0.elapsed().as_secs_f64(), report.events_processed)
}

#[test]
fn telemetry_does_not_perturb_throughput_by_more_than_five_percent() {
    let (mut bare, mut logged) = (f64::INFINITY, f64::INFINITY);
    let (mut bare_events, mut logged_events) = (0u64, 0u64);

    for _ in 0..REPEATS {
        // Interleaved, and in this order every time so neither arm gets a
        // systematically colder cache than the other.
        let (t_off, e_off) = timed(4242, None);
        let (t_on, e_on) = timed(4242, Some(LogFilter::all()));
        bare = bare.min(t_off);
        logged = logged.min(t_on);
        bare_events = e_off;
        logged_events = e_on;
    }

    // **The identity first.** If the two arms did different amounts of work the
    // ratio below is meaningless, and this is the cheap way to know — the same
    // seed must process the same events whether or not anything was recorded.
    assert_eq!(
        bare_events, logged_events,
        "the arms simulated different runs ({bare_events} vs {logged_events} events); \
         the timing comparison below would be meaningless"
    );
    // Enough work that the ratio is not dominated by process startup. The
    // first attempt at this scenario had 4,128 events and this guard caught it.
    assert!(bare_events > 10_000, "too few events to time stably ({bare_events})");

    let overhead = logged / bare;
    let yr_per_s_bare = HORIZON / bare;
    let yr_per_s_logged = HORIZON / logged;
    println!(
        "telemetry cost: {bare:.3}s bare ({yr_per_s_bare:.0} yr/s) vs {logged:.3}s logged \
         ({yr_per_s_logged:.0} yr/s) over {bare_events} events — ratio {overhead:.3}"
    );

    assert!(
        overhead <= MAX_OVERHEAD,
        "LogFilter::all() costs {:.1}% of throughput, budget is {:.0}% \
         ({bare:.3}s -> {logged:.3}s over {bare_events} events). Every instrumented \
         measurement is now reading a system the instrument slowed down.",
        100.0 * (overhead - 1.0),
        100.0 * (MAX_OVERHEAD - 1.0),
    );
}
