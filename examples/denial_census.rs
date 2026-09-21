//! **T-112: does holding ground actually curb a neighbour's expansion?**
//!
//! The author's stated expectation, written down before the measurement so it
//! can refuse the change (`CLAUDE.md` §2): *"I expect to greatly decrease the
//! expansion of the Warfare player's neighbors. Some blood shed is expected,
//! but the goal is curbing the neighbour's growth."*
//!
//! So the acceptance criterion is **not** kills and **not** the player's own
//! colony count. It is the split:
//!
//! - **seat 0 plays the card**; seats 1..n are the neighbours at default doctrine;
//! - the number to watch is **neighbour colonies**, which must fall;
//! - **`W_0 = C_0 − mean(C_j)`** is Warfare's own objective at equal weights
//!   (`Hyades_warfare_tree.md` §0 with `w_ij` uniform — R-WAR3 is unset, and a
//!   uniform weight is the only defensible placeholder on a bed whose
//!   homeworlds are equilateral, trees §2.3.2);
//! - **the player's own colonies are expected to fall too**, because a
//!   coloniser that leaves is a colony founded without its `Band I` stock
//!   (roles §4.2). A card that cost its player nothing would be a different
//!   card.
//!
//! **This is an asymmetric bed** — one seat carrying the card, the rest at the
//! default — which is what R-TREE8 says Warfare requires and what every other
//! measurement in this project does not do.
//!
//! Run: `cargo run --release --example denial_census`
use hyades_engine::log::{LogCategory, LogEvent, LogFilter};
use hyades_engine::prelude::*;
use std::io::Write;

const SEEDS: &[u64] = &[1, 7, 42, 31337, 2, 3, 5, 11];
const PLAYERS: usize = 3;
const HORIZON: f64 = 800.0;

struct Arm {
    own: f64,
    neighbours: f64,
    kills: u64,
    diverts: u64,
    pickets: u64,
}

/// Which half of the card is switched on — `CLAUDE.md` §2's 2x2 rule, because
/// T-112 and T-113 address the same diagnosis and landing them together would
/// make either one unattributable.
#[derive(Clone, Copy, PartialEq)]
enum Card {
    /// Nobody plays anything.
    Peace,
    /// T-112 as measured: colonisers picket after founding, and nothing else.
    ColoniserPickets,
    /// …plus the hold erects part of its endowment as infrastructure (T-113).
    PlusInfraShare,
    /// …plus cheap Limited Offensive pickets built on purpose (T-113).
    PlusLouPickets,
    /// The LOU pickets *without* the coloniser ones — is the cheap hull the
    /// whole result, or does it only help the expensive one?
    LouOnly,
    /// `LouOnly` plus the claim branch: a centre saving for a world it has
    /// already named holds that world with a cheap hull while the bank fills.
    /// This is the *supply* arm — `LouOnly` builds ~12 pickets a run because
    /// its branch sits behind a survey test that is almost always true, so no
    /// placement or preference rule has a population to act on.
    ClaimsTarget,
}

fn run(seed: u64, arm: Card) -> Arm {
    let card = arm != Card::Peace;
    let galaxy = Galaxy::generate(GalaxyConfig::new(PLAYERS, seed)).unwrap();
    // Seat 0 alone plays the card.
    let autopilots: Vec<Box<dyn Autopilot>> = (0..PLAYERS)
        .map(|i| {
            let me = card && i == 0;
            let d = Doctrine {
                engage_neutrals: me,
                picket_after_founding: me && !matches!(arm, Card::LouOnly | Card::ClaimsTarget),
                // Half the hold is erected rather than banked — a placeholder,
                // probed below.
                founding_infra_share: if me && matches!(arm, Card::PlusInfraShare | Card::PlusLouPickets) {
                    0.5
                } else {
                    0.0
                },
                picket_reserve: if me && matches!(arm, Card::PlusLouPickets | Card::LouOnly | Card::ClaimsTarget) {
                    128
                } else {
                    0
                },
                picket_claims_target: me && arm == Card::ClaimsTarget,
                ..Doctrine::default()
            };
            Box::new(BaselineAutopilot::new(d)) as Box<_>
        })
        .collect();
    let mut cfg = SimConfig::new(seed);
    cfg.horizon_years = HORIZON;
    cfg.engagements_enabled = card;
    let mut sim = Simulation::new(galaxy, cfg, autopilots);
    sim.set_log_filter(LogFilter::none().with(LogCategory::Combat).with(LogCategory::Vehicles));
    let report = sim.run();

    let (mut kills, mut diverts, mut pickets) = (0u64, 0u64, 0u64);

    for r in sim.log().iter() {
        match r.event {
            LogEvent::EngagementResolved { losses_attacker, losses_defender, .. } => {
                kills += (losses_attacker + losses_defender) as u64;
            }
            // **Only picket diversions count.** `ColonyContested` also fires
            // for losing a race, which is a baseline behaviour and swamps this.
            LogEvent::ColonyDiverted { .. } => diverts += 1,
            LogEvent::VehicleParked { role: Role::Picket, .. } => pickets += 1,
            _ => {}
        }
    }
    let own = report.players[0].colonies as f64;
    let neighbours = report.players[1..].iter().map(|p| p.colonies as f64).sum::<f64>() / (PLAYERS - 1) as f64;
    Arm { own, neighbours, kills, diverts, pickets }
}

fn main() {
    let all = [
        ("T-112 coloniser pickets", Card::ColoniserPickets),
        ("+ hold erects infra    ", Card::PlusInfraShare),
        ("+ cheap LOU pickets    ", Card::PlusLouPickets),
        ("LOU pickets alone      ", Card::LouOnly),
        ("+ claims its target    ", Card::ClaimsTarget),
    ];
    // **Re-run one arm without re-running the bed.** A full pass is 48 runs and
    // the peace baseline is a sixth of it, so an arm that has to be re-measured
    // after an engine fix costs a whole afternoon otherwise. The baseline is
    // always run, because every figure below is a ratio against it on the same
    // seed (CRN).
    let only: Vec<String> = std::env::args().skip(1).collect();
    let arms: Vec<_> = if only.is_empty() {
        all.to_vec()
    } else {
        all.iter().filter(|(l, _)| only.iter().any(|o| l.trim().contains(o.as_str()))).copied().collect()
    };
    assert!(!arms.is_empty(), "no arm matched {only:?}");
    // **Print before the work, not after it.** `CLAUDE.md` §2: a harness that
    // says nothing until its first expensive stage finishes is indistinguishable
    // from a hung one, and this one sat silent for four minutes.
    println!("peace baseline: {} seeds, {PLAYERS} seats, {HORIZON} yr…", SEEDS.len());
    let _ = std::io::stdout().flush();
    let peace: Vec<Arm> = SEEDS
        .iter()
        .map(|&s| {
            let a = run(s, Card::Peace);
            print!(".");
            let _ = std::io::stdout().flush();
            a
        })
        .collect();
    println!();
    println!(
        "{:>25}  {:>10}  {:>10}  {:>9}  {:>8}  {:>8}  {:>8}",
        "arm", "own", "neighbours", "W_0", "diverts", "pickets", "kills"
    );
    let _ = std::io::stdout().flush();

    for (label, arm) in arms {
        let (mut d_own, mut d_nbr, mut w_gain) = (Vec::new(), Vec::new(), Vec::new());
        let (mut diverts, mut pickets, mut kills) = (0u64, 0u64, 0u64);
        for (i, &seed) in SEEDS.iter().enumerate() {
            let p = &peace[i];
            let c = run(seed, arm);
            d_own.push(100.0 * (c.own / p.own - 1.0));
            d_nbr.push(100.0 * (c.neighbours / p.neighbours - 1.0));
            w_gain.push((c.own - c.neighbours) - (p.own - p.neighbours));
            diverts += c.diverts;
            pickets += c.pickets;
            kills += c.kills;
        }
        let m = |v: &[f64]| {
            let n = v.len() as f64;
            let mean = v.iter().sum::<f64>() / n;
            let var = v.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0);
            (mean, (var / n).sqrt())
        };
        let (own_m, own_se) = m(&d_own);
        let (nbr_m, nbr_se) = m(&d_nbr);
        let (w_m, w_se) = m(&w_gain);
        println!(
            "{label}  {own_m:>+6.2}±{own_se:<4.2}  {nbr_m:>+6.2}±{nbr_se:<4.2}  {w_m:>+6.0}±{w_se:<3.0}  {diverts:>8}  {pickets:>8}  {kills:>8}",
        );
        let _ = std::io::stdout().flush();
    }
    println!("\n  own/neighbours are % against peace; W_0 = (C_0 - mean C_j) change, in colonies.");
    println!("  Neighbours should FALL and W_0 should RISE. 8 seeds, 3 seats, 800 yr.");
}
