//! Regression guard on the tuned laser-vs-missile balance.
//!
//! The `laser_vs_missile` example sweeps and prints; it asserts nothing, so
//! before this file a retuning that inverted the whole matchup still exited 0
//! and CI stayed green. These tests pin the outcome.
//!
//! **Golden values, deliberately.** The engine is deterministic — seeded RNG,
//! ordered event queue, no clock and no OS entropy — and the numbers below were
//! observed bit-identical on two unrelated machines (a local x86-64 box and a
//! GitHub `ubuntu-latest` runner). So the table is pinned exactly rather than
//! bounded loosely: any drift at all is a real behavioural change and should be
//! looked at, not absorbed by a tolerance. If you retune combat on purpose,
//! update the constants here in the same commit — that diff is the record of
//! what your change did to the balance.
//!
//! Alongside the golden table, [`tuned_matchup_is_unchanged`] also asserts the
//! qualitative properties `MIGRATION.md` states as the design intent, so a
//! failure distinguishes "the numbers moved" from "the intent broke".
//!
//! Slow — ~52 s in release at three seeds, far worse in debug — so every test
//! here is `#[ignore]`d and excluded from a plain `cargo test`. CI runs them in
//! the `balance` job:
//!
//! ```text
//! cargo test --release --test balance -- --ignored --nocapture
//! ```

use hyades_engine::arena::laser_vs_missile_trial;
use hyades_engine::combat::{CombatConfig, Winner};

/// Three seeds, not five. These are full missile-vs-laser sims and the pair of
/// tests ran 86 s at five, over the 60 s budget CI targets. The physics is
/// unchanged and the golden tables still catch any drift; what is lost is two
/// samples of seed variation, which the offline sweeps cover far better than a
/// regression test ever could.
const SEEDS: &[u64] = &[1, 2, 3];
const DAYS_PER_YEAR: f64 = 365.25;

/// ~0.18 days. At the old dt=0.006yr missile guidance could not converge on a
/// dodging target, so this is load-bearing, not a comfort setting — the example
/// carries the same note.
const DT: f64 = 0.0005;

/// Win counts over [`SEEDS`] at one relative velocity: (laser, missile, draw).
type Tally = (u32, u32, u32);

fn volley_period() -> f64 {
    16.0 / DAYS_PER_YEAR
}

/// Part 1 of the sweep: 100 v 100 at 0.01 ly, one tally per relative velocity.
fn relative_velocity_table(cfg: &CombatConfig) -> Vec<(f64, Tally)> {
    [-0.002, -0.0005, 0.0, 0.0005, 0.002]
        .iter()
        .map(|&rel_v| {
            let mut tally: Tally = (0, 0, 0);
            for &seed in SEEDS {
                let out = laser_vs_missile_trial(seed, 0.01, rel_v, 100, 100, 0.5, DT, volley_period(), cfg);
                match out.winner {
                    Winner::Laser => tally.0 += 1,
                    Winner::Missile => tally.1 += 1,
                    Winner::Draw => tally.2 += 1,
                }
            }
            (rel_v, tally)
        })
        .collect()
}

#[test]
#[ignore = "slow sweep; run with --release --ignored (CI does this in the balance job)"]
fn tuned_matchup_is_unchanged() {
    // (relative velocity in c, (laser wins, missile wins, draws)) over SEEDS.
    // Negative rel_v is the missile fleet closing; positive is receding.
    const EXPECTED: &[(f64, Tally)] =
        &[(-0.002, (0, 3, 0)), (-0.0005, (2, 1, 0)), (0.0, (2, 1, 0)), (0.0005, (2, 1, 0)), (0.002, (3, 0, 0))];

    let table = relative_velocity_table(&CombatConfig::default());

    println!("  rel_v(c)   laser  missile  draw");
    for (rel_v, (lw, mw, dr)) in &table {
        println!("{rel_v:>10.4} {lw:>7} {mw:>8} {dr:>5}");
    }

    assert_eq!(
        table.len(),
        EXPECTED.len(),
        "the sweep changed shape; update EXPECTED alongside relative_velocity_table"
    );
    for (&(rel_v, got), &(want_rel_v, want)) in table.iter().zip(EXPECTED) {
        assert_eq!(rel_v, want_rel_v, "relative-velocity grid changed");
        assert_eq!(
            got, want,
            "balance moved at rel_v={rel_v}: got (laser, missile, draw) = {got:?}, expected {want:?}. \
             If you retuned combat deliberately, update EXPECTED in this file in the same commit."
        );
    }

    // The design intent behind those numbers (MIGRATION.md): missiles are
    // favoured when the fleets are closing, lasers when at rest or receding.
    // Asserted separately from the golden table so a failure says which of the
    // two broke.
    let tally_at = |v: f64| table.iter().find(|(rel_v, _)| *rel_v == v).expect("velocity present in table").1;

    let (closing_laser, closing_missile, _) = tally_at(-0.002);
    assert!(
        closing_missile > closing_laser,
        "design intent broken: missiles should be favoured closing at -0.002c, got laser={closing_laser} missile={closing_missile}"
    );

    let (rest_laser, rest_missile, _) = tally_at(0.0);
    assert!(
        rest_laser > rest_missile,
        "design intent broken: lasers should be favoured at rest, got laser={rest_laser} missile={rest_missile}"
    );

    let (receding_laser, receding_missile, _) = tally_at(0.002);
    assert!(
        receding_laser > receding_missile,
        "design intent broken: lasers should be favoured receding at +0.002c, got laser={receding_laser} missile={receding_missile}"
    );
}

#[test]
#[ignore = "slow sweep; run with --release --ignored (CI does this in the balance job)"]
fn two_to_one_lasers_win_with_light_casualties() {
    // Per-seed (laser survivors of 200, missile survivors of 100), in SEEDS
    // order. Note the missile fleet is NOT wiped out: within the 1.5 yr horizon
    // the lasers never finish it off, and `resolve_engagement` awards the win on
    // the larger surviving count. "Lasers win" here means outnumbering at the
    // horizon, not annihilation — worth stating, because assuming otherwise is
    // an easy mistake to bake into a test.
    const EXPECTED_SURVIVORS: &[(usize, usize)] = &[(192, 100), (192, 100), (190, 100)];

    let cfg = CombatConfig::default();
    let mut survivors = Vec::new();
    for &seed in SEEDS {
        let out = laser_vs_missile_trial(seed, 0.02, 0.0, 200, 100, 1.5, DT, volley_period(), &cfg);
        println!(
            "  seed={seed}: winner={:?}, laser survivors={}/200, missile survivors={}/100",
            out.winner, out.laser_survivors, out.missile_survivors
        );
        assert_eq!(out.winner, Winner::Laser, "seed={seed}: 200 lasers should beat 100 missiles");
        survivors.push((out.laser_survivors, out.missile_survivors));
    }

    assert_eq!(
        survivors, EXPECTED_SURVIVORS,
        "survivor counts moved. If you retuned combat deliberately, update \
         EXPECTED_SURVIVORS in this file in the same commit."
    );

    // "Light casualties" as MIGRATION.md puts it — the property the exact
    // counts above are one instance of.
    let mean = survivors.iter().map(|&(l, _)| l).sum::<usize>() as f64 / survivors.len() as f64;
    println!("  mean laser survivors = {mean:.1}/200");
    assert!(
        mean >= 185.0,
        "design intent broken: 2:1 lasers should win with light casualties, mean survivors {mean:.1}/200"
    );
}
