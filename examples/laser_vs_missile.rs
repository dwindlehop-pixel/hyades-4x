//! Laser ROU vs Missile ROU — the ship-testing arena's headline sweep, now a
//! **thin driver**: all combat logic lives in `hyades_engine::combat`
//! ([`resolve_engagement`] + [`CombatConfig`]); the tuned ROU-vs-ROU scenario
//! setup lives in `hyades_engine::arena::laser_vs_missile_trial`. This file only
//! sweeps parameters and prints. Both sides fly `RapidOffensive` (η=0.73,
//! Gangster anchor), so the sole difference is the weapon system.
//!
//! Run with: `cargo run --release --example laser_vs_missile`

use hyades_engine::arena::laser_vs_missile_trial;
use hyades_engine::combat::{CombatConfig, Winner};

const N: usize = 100;
const SEEDS: &[u64] = &[1, 2, 3, 4, 5];
const DAYS_PER_YEAR: f64 = 365.25;

fn main() {
    let cfg = CombatConfig::default();
    // dt=0.0005yr (~0.18 days): at the old dt=0.006yr missile guidance could not
    // converge on a dodging target. Confirmed necessary, not just nicer.
    let dt = 0.0005;
    let period = 16.0 / DAYS_PER_YEAR;

    println!("=== Part 1: relative-velocity dependence at distance=0.01 ly ===");
    println!(
        "({N} lasers vs {N} missiles, burst={}, period=16d, shots/tick={}, tolerance={})\n",
        cfg.burst_count, cfg.laser_shots_per_tick, cfg.laser_hit_tolerance
    );
    println!("{:>10} {:>10} {:>12} {:>8}", "rel_v(c)", "laser_win%", "missile_win%", "draw%");
    for &rel_v in &[-0.002, -0.0005, 0.0, 0.0005, 0.002] {
        let (mut lw, mut mw, mut dr) = (0, 0, 0);
        for &seed in SEEDS {
            match laser_vs_missile_trial(seed, 0.01, rel_v, N, N, 0.5, dt, period, &cfg).winner {
                Winner::Laser => lw += 1,
                Winner::Missile => mw += 1,
                Winner::Draw => dr += 1,
            }
        }
        let n = SEEDS.len() as f64;
        println!(
            "{rel_v:>10.4} {:>9.1}% {:>12.1}% {:>7.1}%",
            lw as f64 / n * 100.0,
            mw as f64 / n * 100.0,
            dr as f64 / n * 100.0
        );
    }

    println!("\n=== Part 2: 200 Laser ROUs vs 100 Missile ROUs at distance=0.02 ly ===\n");
    let (mut lw, mut mw, mut dr) = (0, 0, 0);
    let mut laser_survivors_sum = 0usize;
    for &seed in SEEDS {
        let out = laser_vs_missile_trial(seed, 0.02, 0.0, 200, 100, 1.5, dt, period, &cfg);
        match out.winner {
            Winner::Laser => lw += 1,
            Winner::Missile => mw += 1,
            Winner::Draw => dr += 1,
        }
        laser_survivors_sum += out.laser_survivors;
        println!(
            "  seed={seed}: winner={:?}, laser survivors={}/200 ({} casualties)",
            out.winner,
            out.laser_survivors,
            200 - out.laser_survivors
        );
    }
    let n = SEEDS.len() as f64;
    println!(
        "\nlaser_win={:.1}%  missile_win={:.1}%  draw={:.1}%  mean laser survivors={:.1}/200",
        lw as f64 / n * 100.0,
        mw as f64 / n * 100.0,
        dr as f64 / n * 100.0,
        laser_survivors_sum as f64 / n
    );
}
