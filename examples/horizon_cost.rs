//! **What a horizon costs, and what it buys — re-measured after T-68.**
//!
//! `CLAUDE.md` §2 carries a horizon/cost table that the screen-vs-objective
//! rule leans on. It was measured at an operating point three landings ago and
//! is now wrong by two orders of magnitude, which is exactly the failure mode
//! that file warns about in its own §7 ("the 456 yr/s row is stale in a way
//! worth naming"). A stale cost table is worse than none: it makes a 40-minute
//! job look like a 30-second one, and the schedule is planned off it.
//!
//! Two things this reports, and the second is the one that changed:
//!
//! - **Cost is violently superlinear in duration**, because entity count
//!   compounds and every event is paid for at the count that is standing when
//!   it fires. Doubling the horizon does far more than double the bill.
//! - **Colony count saturates long before the horizon does.** T-68 accelerated
//!   the expansion loop enough that the bed is essentially finished by ~1,500
//!   years, so the back half of a 4,000-year run is simulating a full galaxy at
//!   peak entity count to add a handful of colonies. Colony-*years* keep
//!   accruing — the guard still needs the full run — but colony **count**, the
//!   T-20 objective, stops discriminating.
//!
//! Not in CI: it is a few minutes by construction, and it is a calibration tool
//! rather than a check. Re-run it whenever a landing moves the expansion loop,
//! and update the table it feeds.
//!
//! Run: `cargo run --release --example horizon_cost`
use hyades_engine::prelude::*;
use std::io::Write;
use std::time::Instant;

const HORIZONS: [f64; 5] = [500.0, 1000.0, 1500.0, 2000.0, 4000.0];
const SEED: u64 = 1;
const PLAYERS: usize = 3;

fn main() {
    println!("horizon cost — seed {SEED}, {PLAYERS} seats");
    println!("{:>8}{:>10}{:>12}{:>14}", "horizon", "seconds", "colonies", "% of final");
    std::io::stdout().flush().ok();

    let mut rows = Vec::new();
    for h in HORIZONS {
        let t = Instant::now();
        let galaxy = Galaxy::generate(GalaxyConfig::new(PLAYERS, SEED)).unwrap();
        let mut cfg = SimConfig::new(SEED);
        cfg.horizon_years = h;
        let mut sim = Simulation::with_baseline(galaxy, cfg);
        let report = sim.run();
        let colonies: usize = report.players.iter().map(|p| p.colonies).sum();
        rows.push((h, t.elapsed().as_secs_f64(), colonies));
        // Flushed per row: a run cut short still yields the rows that finished.
        let (h, s, c) = *rows.last().unwrap();
        println!("{h:>8.0}{s:>10.1}{c:>12}{:>14}", "");
        std::io::stdout().flush().ok();
    }

    let final_colonies = rows.last().map(|r| r.2).unwrap_or(1).max(1);
    println!("\n{:>8}{:>10}{:>12}{:>14}", "horizon", "seconds", "colonies", "% of final");
    for (h, s, c) in &rows {
        println!("{h:>8.0}{s:>10.1}{c:>12}{:>13.1}%", 100.0 * *c as f64 / final_colonies as f64);
    }
    println!("\nRead the last column before choosing a horizon: past the point where it\nreaches ~100%, a longer run buys colony-years and nothing else.");
}
