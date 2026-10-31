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
//! **Strength** is capability (`Hyades_technology_tree.md` §4.7), for every
//! card whatever its tree — the author's ruling: *"Strength is a measure of
//! fleet (hull, role, number) capability … defined over the current active
//! fleet, and changes as roles change. All cards will alter the measure of
//! strength."* For card `c` on seed `s` it is the mean over even seats `i` of
//! `ln(T_i(card arm) / T_i(pass arm))`, where
//!
//! | symbol | name | unit |
//! |---|---|---|
//! | `T_i` | capability-years: `∫ Q_i dt` from the barrier to the horizon, sampled every `SAMPLE_YEARS` | kt·yr |
//! | `Q_i` | capability stock: `Σ_h m_h · γ(d_h, bed(role_h))` over seat `i`'s hulls standing at the sample | kt of anchor-equivalent |
//! | `m_h` | the hull's dry mass, which is its mineral cost (R-O57; R-TECH9's weight) | kt |
//! | `γ(d, b)` | Design `d`'s strength in bed `b` on the ratio scale, the bed's anchor 1, read from `data/design_ratings.tsv` | ratio |
//! | `bed(role)` | the bed that rates a role (`BED_OF_ROLE`); `Reserve` has none and counts zero (R-TECH10) | — |
//!
//! A hull whose Design the table does not rate in its role's bed counts zero
//! and is reported on an `unrated <design> <bed> <hull-years>` line, which the
//! release rule ignores. The table's stamp is printed on a `table` line.
//!
//! **The kind census** reads three things: the name of every `LogEvent`
//! variant raised (a system acting), every `VehicleKind` standing at a sample
//! (an entity in a role), and the name of every `CardEffect` variant of every
//! card that landed (a write reaching the sim).
//!
//! Cost: about 70 s in release on one core; `--smoke` runs one seed and is
//! not a census.

use hyades_engine::cards::{self, CardEffect, CardId, Order, Target, TIER0};
use hyades_engine::galaxy::{Galaxy, GalaxyConfig, PlayerId};
use hyades_engine::log::LogFilter;
use hyades_engine::sim::{Class, SimConfig, Simulation};
use hyades_engine::snapshot::{VehicleKind, VehicleSnapshot};
use hyades_engine::units::Price;
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;

const SEATS: usize = 6;
const PLANETS: usize = 300;
const SEEDS: [u64; 8] = [1, 7, 42, 31337, 2, 3, 5, 11];
const HORIZON: f64 = 500.0;
const SAMPLE_YEARS: f64 = 1.0;
const TABLE: &str = include_str!("../data/design_ratings.tsv");

/// The bed that rates each role (`Hyades_technology_tree.md` §4.4). `Sentry`
/// has no bed of its own; it reads the long-range bed, a placeholder (its
/// missiles are the long-range family, §9). `Reserve` and `Scrapped` are not
/// listed and count zero.
const BED_OF_ROLE: [(VehicleKind, &str); 6] = [
    (VehicleKind::Scout, "scout"),
    (VehicleKind::Colonizer, "colonizer"),
    (VehicleKind::Miner, "miner"),
    (VehicleKind::Freighter, "freighter"),
    (VehicleKind::Picket, "picket"),
    (VehicleKind::Sentry, "long-range"),
];

/// **The rating table**, `(bed, design) → γ`. A label such as `Meadow=Spur`
/// names Designs the rating bed could not tell apart, and each of them gets
/// its `γ`. An `Unnamed` hull (a seed hull, built to no Design) reads the
/// label of the one table Design on its hull, when exactly one is.
struct Ratings {
    gamma: BTreeMap<(String, String), f64>,
    /// `hull code → label` for hulls exactly one table label sits on.
    unnamed: BTreeMap<String, String>,
    stamp: String,
}

/// The hull a named Design is on, by the docs' abbreviation (`Class::hull`).
fn hull_of(design: &str) -> &'static str {
    Class::ALL.iter().find(|c| c.name() == design).and_then(|c| c.hull()).map_or("", |h| h.code())
}

impl Ratings {
    fn parse(text: &str) -> Ratings {
        let mut gamma = BTreeMap::new();
        let mut labels_on: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        let stamp = text.lines().next().unwrap_or("").trim_start_matches("# ").to_string();
        for line in text.lines().filter(|l| !l.starts_with('#') && !l.starts_with("bed\t")) {
            let f: Vec<&str> = line.split('\t').collect();
            let [bed, label, g, ..] = f[..] else { continue };
            let g: f64 = g.parse().expect("gamma is a number");
            for d in label.split('=') {
                gamma.insert((bed.to_string(), d.to_string()), g);
                labels_on.entry(hull_of(d).to_string()).or_default().insert(label.to_string());
            }
            gamma.insert((bed.to_string(), label.to_string()), g);
        }
        let unnamed = labels_on
            .into_iter()
            .filter(|(_, l)| l.len() == 1)
            .map(|(h, l)| (h, l.into_iter().next().unwrap()))
            .collect();
        Ratings { gamma, unnamed, stamp }
    }

    /// `(bed, γ)` for one hull, or `Err((design, bed))` when the table does not
    /// rate it; `Ok(None)` for a role no bed rates.
    fn of(&self, v: &VehicleSnapshot) -> Result<Option<f64>, (String, &'static str)> {
        let Some(&(_, bed)) = BED_OF_ROLE.iter().find(|(k, _)| *k == v.kind) else { return Ok(None) };
        let design =
            if v.design == "Unnamed" { self.unnamed.get(v.hull).map_or(v.design, |l| l.as_str()) } else { v.design };
        match self.gamma.get(&(bed.to_string(), design.to_string())) {
            Some(&g) => Ok(Some(g)),
            None => Err((format!("{}/{}", v.design, v.hull), bed)),
        }
    }
}

/// The name of an enum value: its `Debug` text up to the first field.
fn variant<T: std::fmt::Debug>(v: &T) -> String {
    let s = format!("{v:?}");
    s.split(|ch: char| !ch.is_alphanumeric() && ch != '_').next().unwrap_or("").to_string()
}

struct Run {
    /// Capability-years per seat.
    capability: [f64; SEATS],
    landed: [bool; SEATS],
}

/// What every arm adds to the census beside its own strengths.
#[derive(Default)]
struct Census {
    kinds: BTreeSet<(String, String)>,
    /// Hull-years of hulls the table does not rate, by `(design/hull, bed)`.
    unrated: BTreeMap<(String, &'static str), f64>,
}

fn run(seed: u64, card: Option<CardId>, ratings: &Ratings, census: &mut Census) -> Run {
    let mut gcfg = GalaxyConfig::new(SEATS, seed);
    gcfg.planet_count = PLANETS;
    let galaxy = Galaxy::generate(gcfg).expect("a fair seat count generates");
    let mut cfg = SimConfig::new(seed);
    cfg.horizon_years = HORIZON;
    let barrier = cfg.years_to_first_round;
    let mut sim = Simulation::with_baseline(galaxy, cfg);
    sim.set_log_filter(LogFilter::all());

    let mut out = Run { capability: [0.0; SEATS], landed: [false; SEATS] };
    let mut next_sample = barrier;
    let mut last_t: Option<f64> = None;

    let mut sample = |sim: &mut Simulation, out: &mut Run, census: &mut Census| {
        for r in sim.log().iter() {
            census.kinds.insert(("system".into(), variant(&r.event)));
        }
        sim.clear_log();
        let snap = sim.snapshot();
        let t = sim.clock();
        let dt = match last_t {
            Some(l) if t >= barrier => t - l,
            _ => 0.0,
        };
        if t >= barrier {
            last_t = Some(t);
        }
        for v in snap.vehicles.iter().filter(|v| v.kind != VehicleKind::Scrapped) {
            census.kinds.insert(("entity".into(), variant(&v.kind)));
            if (v.owner as usize) >= SEATS {
                continue;
            }
            match ratings.of(v) {
                Ok(Some(g)) => out.capability[v.owner as usize] += v.dry_mass.kilotons() * g * dt,
                Ok(None) => {}
                Err(key) => *census.unrated.entry(key).or_default() += dt,
            }
        }
    };

    while sim.step() {
        if let Some(id) = card {
            let c = cards::card(id).expect("a published card");
            for (i, done) in out.landed.iter_mut().enumerate() {
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
            sample(&mut sim, &mut out, census);
        }
    }
    sample(&mut sim, &mut out, census);
    out
}

fn main() {
    let smoke = std::env::args().any(|a| a == "--smoke");
    let seeds: &[u64] = if smoke { &SEEDS[..1] } else { &SEEDS };
    let ratings = Ratings::parse(TABLE);
    let mut out = std::io::stdout().lock();
    writeln!(out, "table {}", ratings.stamp).unwrap();
    let mut census = Census::default();
    let defined: Vec<_> = TIER0.iter().filter(|c| !c.writes(CardEffect::NotYetImplemented)).collect();
    let even: Vec<usize> = (0..SEATS).step_by(2).collect();
    for &seed in seeds {
        let pass = run(seed, None, &ratings, &mut census);
        for c in &defined {
            let arm = run(seed, Some(c.id), &ratings, &mut census);
            if !even.iter().all(|&i| arm.landed[i]) {
                continue;
            }
            for e in c.effects {
                census.kinds.insert(("effect".into(), variant(e)));
            }
            let ln =
                |i: usize| (arm.capability[i].max(f64::MIN_POSITIVE) / pass.capability[i].max(f64::MIN_POSITIVE)).ln();
            let v = even.iter().map(|&i| ln(i)).sum::<f64>() / even.len() as f64;
            writeln!(out, "card {} {seed} {v:.6}", c.id.0).unwrap();
            out.flush().unwrap();
        }
    }
    for ((design, bed), years) in &census.unrated {
        writeln!(out, "unrated {design} {bed} {years:.0}").unwrap();
    }
    for (cat, name) in &census.kinds {
        writeln!(out, "kind {cat} {name}").unwrap();
    }
}
