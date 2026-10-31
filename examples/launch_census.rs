//! **The launch test** (`docs/Hyades_releases.md` §2): sampled games whose
//! census decides the next release number. The release workflow runs it on a
//! batch of `main` commits and compares its output with the census stored in
//! the previous release tag. Nothing here chooses a version.
//!
//! `launch_census [--smoke]` prints one line per observation:
//!
//! | line | meaning | decides |
//! |---|---|---|
//! | `card <id> <seed> <strength>` | a defined card landed on that seed; `strength` is defined below | a new card id, or a large swing in one card's strength: **major** |
//! | `kind <category> <name>` | an instance of an entity or a system appeared in at least one sampled game | a new kind: **minor** |
//!
//! **A defined card** is a card whose writes include no
//! `CardEffect::NotYetImplemented`, and which landed (was affordable at the
//! round-0 barrier and was played) on every even seat of that seed.
//!
//! **The games.** `SEATS` seats on a `PLANETS`-world galaxy, one per seed in
//! `SEEDS`, run to `HORIZON` years. One arm passes everywhere; one arm per
//! defined card plays it on every even seat at the round-0 barrier
//! (`years_to_first_round`) through `apply_orders`, the protocol's own entry
//! point. A card that needs a subject names the next seat. Every arm logs every
//! category, and every arm contributes to the kind census.
//!
//! **Strength.** For card `c` on seed `s`, the mean over even seats `i` of
//! `ln(M_i(card arm) / M_i(pass arm))`, where `M_i` is the integral from the
//! barrier to the horizon of the card's own tree's stock, sampled every
//! `SAMPLE_YEARS`:
//!
//! | tree | stock | source |
//! |---|---|---|
//! | Expansion, Politics | colonies | trees §2.1; politics is Expansion exactly (§2.3) |
//! | Growth | infrastructure in kilotons (work-years) | R-TREE3's interim stock |
//! | Production | enclosed hull volume `r³` | trees §2.3.4, R-PROD5 |
//! | Warfare | colonies over the rival mean | `examples/card_table` |
//! | Technology | mean of the Expansion and Growth log-ratios | no stock yet (T-131); a placeholder |
//!
//! **The kind census** reads three things: the name of every `LogEvent`
//! variant raised (a system acting), every `VehicleKind` standing at a sample
//! (an entity in a role), and the name of every `CardEffect` variant of every
//! card that landed (a write reaching the sim).
//!
//! Cost: about 40 s in release on one core; `--smoke` runs one seed and is
//! not a census.

use hyades_engine::cards::{self, CardEffect, CardId, Order, Target, Tree, TIER0};
use hyades_engine::galaxy::{Galaxy, GalaxyConfig, PlayerId};
use hyades_engine::log::LogFilter;
use hyades_engine::sim::{SimConfig, Simulation};
use hyades_engine::snapshot::VehicleKind;
use hyades_engine::units::Price;
use std::collections::BTreeSet;
use std::io::Write;

const SEATS: usize = 6;
const PLANETS: usize = 300;
const SEEDS: [u64; 8] = [1, 7, 42, 31337, 2, 3, 5, 11];
const HORIZON: f64 = 500.0;
const SAMPLE_YEARS: f64 = 25.0;

/// The name of an enum value: its `Debug` text up to the first field.
fn variant<T: std::fmt::Debug>(v: &T) -> String {
    let s = format!("{v:?}");
    s.split(|ch: char| !ch.is_alphanumeric() && ch != '_').next().unwrap_or("").to_string()
}

/// Per seat, the integral over `[barrier, horizon]` of each stock the strength
/// table reads.
struct Stocks {
    colonies: Vec<f64>,
    infra: Vec<f64>,
    volume: Vec<f64>,
    share: Vec<f64>,
}

struct Run {
    stocks: Stocks,
    landed: Vec<bool>,
}

fn run(seed: u64, card: Option<CardId>, kinds: &mut BTreeSet<(String, String)>) -> Run {
    let mut gcfg = GalaxyConfig::new(SEATS, seed);
    gcfg.planet_count = PLANETS;
    let galaxy = Galaxy::generate(gcfg).expect("a fair seat count generates");
    let mut cfg = SimConfig::new(seed);
    cfg.horizon_years = HORIZON;
    let barrier = cfg.years_to_first_round;
    let mut sim = Simulation::with_baseline(galaxy, cfg);
    sim.set_log_filter(LogFilter::all());

    let mut landed = vec![false; SEATS];
    let mut s = Stocks {
        colonies: vec![0.0; SEATS],
        infra: vec![0.0; SEATS],
        volume: vec![0.0; SEATS],
        share: vec![0.0; SEATS],
    };
    let mut next_sample = barrier;
    let mut last_t: Option<f64> = None;

    let mut sample = |sim: &mut Simulation, s: &mut Stocks, kinds: &mut BTreeSet<(String, String)>| {
        for r in sim.log().iter() {
            kinds.insert(("system".into(), variant(&r.event)));
        }
        sim.clear_log();
        let snap = sim.snapshot();
        let mut volume = [0.0; SEATS];
        for v in &snap.vehicles {
            if v.kind != VehicleKind::Scrapped {
                kinds.insert(("entity".into(), variant(&v.kind)));
            }
            if (v.owner as usize) < SEATS && v.kind != VehicleKind::Scrapped {
                volume[v.owner as usize] += v.volume.hull_units_cubed();
            }
        }
        let t = sim.clock();
        if t < barrier {
            return;
        }
        let dt = last_t.map_or(0.0, |l| t - l);
        last_t = Some(t);
        let stock: Vec<(usize, f64)> = (0..SEATS).map(|i| sim.tree_stock(i)).collect();
        let col: Vec<f64> = stock.iter().map(|&(c, _)| c as f64).collect();
        for i in 0..SEATS {
            let rivals = (0..SEATS).filter(|&j| j != i).map(|j| col[j]).sum::<f64>() / (SEATS - 1) as f64;
            let share = if rivals > 0.0 { col[i] / rivals } else { 1.0 };
            s.colonies[i] += col[i] * dt;
            s.infra[i] += stock[i].1 * dt;
            s.volume[i] += volume[i] * dt;
            s.share[i] += share * dt;
        }
    };

    while sim.step() {
        if let Some(id) = card {
            let c = cards::card(id).expect("a published card");
            for (i, done) in landed.iter_mut().enumerate() {
                if *done || i % 2 == 1 || sim.clock() < barrier || !sim.empire_can_afford(i, Price::new(c.cost)) {
                    continue;
                }
                let target =
                    if c.needs_subject { Target::Player(PlayerId(((i + 1) % SEATS) as u32)) } else { Target::None };
                sim.apply_orders(sim.current_round(), &[Order { seat: PlayerId(i as u32), card: Some(id), target }]);
                *done = true;
            }
        }
        if sim.clock() >= next_sample {
            next_sample = sim.clock() + SAMPLE_YEARS;
            sample(&mut sim, &mut s, kinds);
        }
    }
    sample(&mut sim, &mut s, kinds);
    Run { stocks: s, landed }
}

fn ln_ratio(card: &[f64], pass: &[f64], i: usize) -> f64 {
    (card[i].max(f64::MIN_POSITIVE) / pass[i].max(f64::MIN_POSITIVE)).ln()
}

fn strength(tree: Tree, card: &Stocks, pass: &Stocks, i: usize) -> f64 {
    match tree {
        Tree::Expansion | Tree::Politics => ln_ratio(&card.colonies, &pass.colonies, i),
        Tree::Growth => ln_ratio(&card.infra, &pass.infra, i),
        Tree::Production => ln_ratio(&card.volume, &pass.volume, i),
        Tree::Warfare => ln_ratio(&card.share, &pass.share, i),
        Tree::Technology => 0.5 * (ln_ratio(&card.colonies, &pass.colonies, i) + ln_ratio(&card.infra, &pass.infra, i)),
    }
}

fn main() {
    let smoke = std::env::args().any(|a| a == "--smoke");
    let seeds: &[u64] = if smoke { &SEEDS[..1] } else { &SEEDS };
    let mut out = std::io::stdout().lock();
    let mut kinds = BTreeSet::new();
    let defined: Vec<_> = TIER0.iter().filter(|c| !c.writes(CardEffect::NotYetImplemented)).collect();
    for &seed in seeds {
        let pass = run(seed, None, &mut kinds);
        for c in &defined {
            let arm = run(seed, Some(c.id), &mut kinds);
            let even: Vec<usize> = (0..SEATS).step_by(2).collect();
            if !even.iter().all(|&i| arm.landed[i]) {
                continue;
            }
            for e in c.effects {
                kinds.insert(("effect".into(), variant(e)));
            }
            let v =
                even.iter().map(|&i| strength(c.tree, &arm.stocks, &pass.stocks, i)).sum::<f64>() / even.len() as f64;
            writeln!(out, "card {} {seed} {v:.6}", c.id.0).unwrap();
            out.flush().unwrap();
        }
    }
    for (cat, name) in &kinds {
        writeln!(out, "kind {cat} {name}").unwrap();
    }
}
