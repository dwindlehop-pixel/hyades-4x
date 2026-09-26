//! **The static rating of every named Design, per role** (`Hyades_technology_tree.md`
//! §4, T-131). `design_rating [role|all] [seeds]`.
//!
//! Every match is a galaxy generated with fleets and nothing else changed (the
//! author's rulings: a bed varies only the galaxy; fleets are generated with it
//! at equal mineral spend, with a position and a velocity). The simulation runs
//! its own event loop and its own role systems; the harness reads the log.
//!
//! **The pool** is the named Designs, deduplicated by every field the beds read
//! (R-TECH17): Meadow and Spur, Delta and Ford, Range and Strait are the same
//! object to the engine, so each pair is one candidate.
//!
//! | quantity | meaning | unit |
//! |---|---|---|
//! | `B` | each fleet's spend: ten General Systems hulls' price (R-TECH11) | kt |
//! | `x_A` | side A's task score, per bed below | per bed |
//! | `s_AB` | A's share, `x_A / (x_A + x_B)`, ½ when both are zero (§4.6) | [0, 1] |
//! | `γ` | Bradley–Terry maximum likelihood strength, one virtual draw per pair (R-TECH5/6), on the ratio scale with the role's default Design at 1 (R-TECH8): A takes `γ_A / (γ_A + γ_B)` of a match against B, and a rating is always positive | ratio |
//!
//! | bed | fleets | start | Doctrine | judged by `x` | horizon |
//! |---|---|---|---|---|---|
//! | short-range offensive | one per seat, Picket | point blank, off the disk | hostile (§4.4.6) | dry mass holding the field (R-TECH19) | 1 yr |
//! | long-range offensive | one per seat, Picket | `D_long` apart, closing | hostile | dry mass holding the field | 1 yr |
//! | picket | one per seat, Picket | on the rival's home port | default | rival colony ships wrecked or turned away | 150 yr |
//! | colonizer | one per seat, Colonizer | home port, surveyed start | default | colonies the fleet founds | 80 yr |
//! | miner | one per seat, Miner | home port, surveyed start | default | ore extracted at the fleet's rocks | 160 yr |
//! | freighter | reference Meadow miners + the candidate as Freighter | home port, surveyed start | default | kilotons the fleet delivers to its bank | 160 yr |
//! | scout | one per seat, Scout | home port | default | worlds it reaches first | 40 yr |
//!
//! Every pair is played on every seed with the seats swapped (R-TECH16); the
//! 90% interval on each rating is bootstrapped over seeds.
use hyades_engine::autopilot::{Autopilot, BaselineAutopilot, Doctrine};
use hyades_engine::galaxy::{FleetSeeding, Galaxy, GalaxyConfig, SeedFleet};
use hyades_engine::log::{CourseReason, FreighterLeg, LogCategory, LogEvent, LogFilter};
use hyades_engine::math::Vec3;
use hyades_engine::sim::{hull_dry_mass, Class, Entity, HullType, Role, SimConfig, Simulation};
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;

/// The distinct named Designs, with the names each stands for.
const POOL: [(HullType, Class, &str); 6] = [
    (HullType::LimitedSystems, Class::Meadow, "Meadow=Spur"),
    (HullType::LimitedContactVehicle, Class::Tor, "Tor"),
    (HullType::LimitedContactVehicle, Class::Cairn, "Cairn"),
    (HullType::MediumSystems, Class::Delta, "Delta=Ford"),
    (HullType::GeneralSystems, Class::Range, "Range=Strait"),
    (HullType::GeneralContactVehicle, Class::Scarp, "Scarp"),
];
const N: usize = POOL.len();
/// A per-pair quantity over the pool.
type Table = [[f64; N]; N];

#[derive(Clone, Copy, PartialEq, Debug)]
enum Bed {
    ShortRange,
    LongRange,
    Picket,
    Colonizer,
    Miner,
    Freighter,
    Scout,
}

impl Bed {
    const ALL: [Bed; 7] =
        [Bed::ShortRange, Bed::LongRange, Bed::Picket, Bed::Colonizer, Bed::Miner, Bed::Freighter, Bed::Scout];
    fn name(self) -> &'static str {
        match self {
            Bed::ShortRange => "short-range",
            Bed::LongRange => "long-range",
            Bed::Picket => "picket",
            Bed::Colonizer => "colonizer",
            Bed::Miner => "miner",
            Bed::Freighter => "freighter",
            Bed::Scout => "scout",
        }
    }
    /// Placeholders, each long enough that the bed's task has run its course.
    /// An outpost yields once per `mining_tick_years` (50) after its crew
    /// lands, so the two mining beds run for three ticks.
    fn horizon(self) -> f64 {
        match self {
            Bed::ShortRange | Bed::LongRange => 1.0,
            Bed::Picket => 150.0,
            Bed::Colonizer => 80.0,
            Bed::Miner | Bed::Freighter => 160.0,
            Bed::Scout => 40.0,
        }
    }
    /// The role's anchor: the Design the default standing layer assigns it, or
    /// the Warfare card's picket where the default is unarmed (R-TECH8).
    fn anchor(self) -> usize {
        match self {
            Bed::ShortRange | Bed::LongRange | Bed::Picket => 2,
            Bed::Colonizer | Bed::Freighter => 3,
            Bed::Miner | Bed::Scout => 0,
        }
    }
    fn hostile(self) -> bool {
        matches!(self, Bed::ShortRange | Bed::LongRange)
    }
    /// A surveyed start, ly, where the role needs known worlds at `t = 0`.
    fn known_radius(self) -> f64 {
        match self {
            Bed::Picket | Bed::Colonizer | Bed::Miner | Bed::Freighter => 20.0,
            _ => 0.0,
        }
    }
}

/// `D_long`, the long-range bed's starting separation, ly — beyond every beam
/// Design's reach (7.9e-3 ly). Placeholder (R-TECH12).
const D_LONG: f64 = 0.03;
/// Each long-range fleet's closing speed, `c` — enough that every hull passes
/// the midpoint before it can stop. Placeholder (R-TECH12).
const CLOSING: f64 = 0.45;

/// The standard two-seat field: role beds need worlds at the density a game has.
fn galaxy_config(seed: u64) -> GalaxyConfig {
    GalaxyConfig::new(2, seed)
}

/// One match: Design `a` on seat 0 against `b` on seat 1. Returns each side's
/// task score.
fn play(bed: Bed, seed: u64, a: usize, b: usize) -> [f64; 2] {
    let g = galaxy_config(seed);
    let cfg = SimConfig::new(seed);
    let spend = 10.0 * hull_dry_mass(HullType::GeneralSystems, &cfg).kilotons();
    let homes: Vec<Vec3> = {
        let probe = Galaxy::generate(g).unwrap();
        probe.homeworlds.iter().map(|&h| probe.planets[h.0 as usize].position).collect()
    };
    let design = [a, b];
    let fleet = |seat: usize, d: usize, role: Role, position: Vec3, velocity: Vec3| SeedFleet {
        seat,
        hull: POOL[d].0,
        class: POOL[d].1,
        role,
        position,
        velocity,
    };
    let off_disk = Vec3::new(0.0, 0.0, 60.0);
    let mut fleets = Vec::new();
    for (seat, &d) in design.iter().enumerate() {
        match bed {
            Bed::ShortRange => fleets.push(fleet(seat, d, Role::Picket, off_disk, Vec3::ZERO)),
            Bed::LongRange => {
                let side = if seat == 0 { -1.0 } else { 1.0 };
                let at = off_disk.add(Vec3::new(side * 0.5 * D_LONG, 0.0, 0.0));
                fleets.push(fleet(seat, d, Role::Picket, at, Vec3::new(-side * CLOSING, 0.0, 0.0)));
            }
            Bed::Picket => fleets.push(fleet(seat, d, Role::Picket, homes[1 - seat], Vec3::ZERO)),
            Bed::Colonizer => fleets.push(fleet(seat, d, Role::Colonizer, homes[seat], Vec3::ZERO)),
            Bed::Miner => fleets.push(fleet(seat, d, Role::Miner, homes[seat], Vec3::ZERO)),
            Bed::Freighter => {
                fleets.push(fleet(seat, 0, Role::Miner, homes[seat], Vec3::ZERO));
                fleets.push(fleet(seat, d, Role::Freighter, homes[seat], Vec3::ZERO));
            }
            Bed::Scout => fleets.push(fleet(seat, d, Role::Scout, homes[seat], Vec3::ZERO)),
        }
    }
    let seeding = FleetSeeding { spend_kt: spend, known_radius_ly: bed.known_radius(), fleets };
    let galaxy = Galaxy::generate_with(g, seeding).unwrap();
    let mut run_cfg = cfg;
    run_cfg.horizon_years = bed.horizon();
    let doctrine = Doctrine { engage_neutrals: bed.hostile(), ..Doctrine::default() };
    let aps: Vec<Box<dyn Autopilot>> =
        (0..2).map(|_| Box::new(BaselineAutopilot::new(doctrine)) as Box<dyn Autopilot>).collect();
    let filter = LogFilter::none().with(LogCategory::Vehicles).with(LogCategory::Mining).with(LogCategory::Combat);
    let mut sim = Simulation::new_logged(galaxy, run_cfg, aps, filter);
    sim.run();
    judge(bed, &sim, &cfg, spend, design)
}

/// The bed's task score for each seat, read off the log.
fn judge(bed: Bed, sim: &Simulation, cfg: &SimConfig, spend: f64, design: [usize; 2]) -> [f64; 2] {
    // The generated fleet of each seat, by the role it was generated for.
    let mut fleet: BTreeMap<Entity, (usize, Role)> = BTreeMap::new();
    for r in sim.log().iter() {
        if let LogEvent::FleetGenerated { player, vehicle, role, .. } = r.event {
            fleet.insert(vehicle, (player as usize, role));
        }
    }
    let ours = |v: Entity, seat: usize, role: Role| fleet.get(&v).is_some_and(|&(s, r)| s == seat && r == role);
    // The rocks each seat's generated miners were sent to.
    let mut sites: [BTreeSet<u32>; 2] = [BTreeSet::new(), BTreeSet::new()];
    for r in sim.log().iter() {
        if let LogEvent::VehicleSpawned { player, vehicle, role: Role::Miner, to, .. } = r.event {
            if ours(vehicle, player as usize, Role::Miner) {
                sites[player as usize].insert(to.0);
            }
        }
    }
    let mut x = [0.0; 2];
    match bed {
        Bed::ShortRange | Bed::LongRange => {
            let mut gone = [BTreeSet::new(), BTreeSet::new()];
            for r in sim.log().iter() {
                match r.event {
                    LogEvent::HullWrecked { player, vehicle, .. } => {
                        gone[player as usize].insert(vehicle);
                    }
                    LogEvent::CourseChanged { player, vehicle, reason: CourseReason::Withdraw, .. } => {
                        gone[player as usize].insert(vehicle);
                    }
                    _ => {}
                }
            }
            for seat in 0..2 {
                let m = hull_dry_mass(POOL[design[seat]].0, cfg).kilotons();
                let count = (spend / m).round() as usize;
                let lost = gone[seat].iter().filter(|&&v| ours(v, seat, Role::Picket)).count();
                x[seat] = (count - lost) as f64 * m;
            }
        }
        Bed::Picket => {
            // A rival colony ship denied: wrecked, or turned away under fire or
            // its news. Only the fleets are armed, so every denial is theirs.
            let mut denied = [BTreeSet::new(), BTreeSet::new()];
            for r in sim.log().iter() {
                match r.event {
                    LogEvent::HullWrecked { player, vehicle, role: Role::Colonizer, .. }
                    | LogEvent::CourseChanged { player, vehicle, role: Role::Colonizer, .. } => {
                        denied[1 - player as usize].insert(vehicle);
                    }
                    _ => {}
                }
            }
            x = [denied[0].len() as f64, denied[1].len() as f64];
        }
        Bed::Colonizer => {
            for r in sim.log().iter() {
                if let LogEvent::ColonyFounded { player, vehicle, .. } = r.event {
                    if ours(vehicle, player as usize, Role::Colonizer) {
                        x[player as usize] += 1.0;
                    }
                }
            }
        }
        Bed::Miner => {
            for r in sim.log().iter() {
                if let LogEvent::MineralsExtracted { player, planet, amount, .. } = r.event {
                    if sites[player as usize].contains(&planet.0) {
                        x[player as usize] += amount;
                    }
                }
            }
        }
        Bed::Freighter => {
            for r in sim.log().iter() {
                if let LogEvent::FreighterTransfer { player, vehicle, leg: FreighterLeg::Deposited, amount, .. } =
                    r.event
                {
                    if ours(vehicle, player as usize, Role::Freighter) {
                        x[player as usize] += amount;
                    }
                }
            }
        }
        Bed::Scout => {
            let mut first: BTreeSet<u32> = BTreeSet::new();
            for r in sim.log().iter() {
                if let LogEvent::ContactArrived { player, vehicle, planet, .. } = r.event {
                    if first.insert(planet.0) && ours(vehicle, player as usize, Role::Scout) {
                        x[player as usize] += 1.0;
                    }
                }
            }
        }
    }
    x
}

/// Bradley–Terry by minorization–maximization (Hunter 2004) on fractional wins,
/// one virtual draw per pair (R-TECH6). Ratio scale, `anchor` at 1.
fn fit(wins: &Table, games: &Table, anchor: usize) -> [f64; N] {
    let (mut w, mut n) = (*wins, *games);
    for i in 0..N {
        for j in 0..N {
            if i != j {
                w[i][j] += 0.5;
                n[i][j] += 1.0;
            }
        }
    }
    let mut gamma = [1.0; N];
    for _ in 0..20_000 {
        let mut next = gamma;
        for i in 0..N {
            let won: f64 = (0..N).filter(|&j| j != i).map(|j| w[i][j]).sum();
            let den: f64 = (0..N).filter(|&j| j != i).map(|j| n[i][j] / (gamma[i] + gamma[j])).sum();
            next[i] = won / den;
        }
        let a = next[anchor];
        for g in next.iter_mut() {
            *g /= a;
        }
        gamma = next;
    }
    gamma
}

/// **The re-evaluation the author requires at every regeneration** (R-TECH7):
/// how far the one-number rating misses the matches, and where. A cyclic triad
/// is three Designs each beating the next on mean share — rock, paper,
/// scissors — which no rating can express; the residual is the largest gap
/// between an observed mean share and the share the ratings predict,
/// `γ_i / (γ_i + γ_j)`. Both come from the counter-graph, and both are
/// acceptable (the author's ruling) as long as they are read each time.
fn intransitivity(w: &Table, n: &Table, gamma: &[f64; N]) -> (Vec<[usize; 3]>, f64, (usize, usize)) {
    let share = |i: usize, j: usize| w[i][j] / n[i][j];
    let beats = |i: usize, j: usize| share(i, j) > 0.5;
    let mut cycles = Vec::new();
    for i in 0..N {
        for j in i + 1..N {
            for k in i + 1..N {
                if k != j && beats(i, j) && beats(j, k) && beats(k, i) {
                    cycles.push([i, j, k]);
                }
            }
        }
    }
    let (mut worst, mut at) = (0.0, (0, 0));
    for i in 0..N {
        for j in 0..N {
            if i != j {
                let r = (share(i, j) - gamma[i] / (gamma[i] + gamma[j])).abs();
                if r > worst {
                    (worst, at) = (r, (i, j));
                }
            }
        }
    }
    (cycles, worst, at)
}

fn rate(bed: Bed, seeds: &[u64]) {
    println!("\n== {} bed: {} seeds, horizon {} yr", bed.name(), seeds.len(), bed.horizon());
    let _ = std::io::stdout().flush();
    let t0 = std::time::Instant::now();
    let mut per_seed: Vec<(Table, Table)> = Vec::new();
    let (mut decided, mut matches) = (0usize, 0usize);
    for &seed in seeds {
        let (mut wins, mut games) = ([[0.0; N]; N], [[0.0; N]; N]);
        for a in 0..N {
            for b in 0..N {
                if a == b {
                    continue;
                }
                let x = play(bed, seed, a, b);
                let tot = x[0] + x[1];
                let s = if tot > 0.0 { x[0] / tot } else { 0.5 };
                wins[a][b] += s;
                wins[b][a] += 1.0 - s;
                games[a][b] += 1.0;
                games[b][a] += 1.0;
                matches += 1;
                if tot > 0.0 && (s == 0.0 || s == 1.0) {
                    decided += 1;
                }
                println!(
                    "MATCH\t{}\t{seed}\t{}\t{}\t{:.4}\t{:.4}\t{s:.3}",
                    bed.name(),
                    POOL[a].2,
                    POOL[b].2,
                    x[0],
                    x[1]
                );
                let _ = std::io::stdout().flush();
            }
        }
        per_seed.push((wins, games));
    }
    let sum = |set: &[usize]| {
        let (mut w, mut n) = ([[0.0; N]; N], [[0.0; N]; N]);
        for &k in set {
            for i in 0..N {
                for j in 0..N {
                    w[i][j] += per_seed[k].0[i][j];
                    n[i][j] += per_seed[k].1[i][j];
                }
            }
        }
        (w, n)
    };
    let all: Vec<usize> = (0..seeds.len()).collect();
    let (w, n) = sum(&all);
    let r = fit(&w, &n, bed.anchor());
    let mut state: u64 = 0x05EE_DE10;
    let mut next = || {
        state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    };
    let mut boots: Vec<[f64; N]> = Vec::new();
    for _ in 0..1000 {
        let pick: Vec<usize> = (0..seeds.len()).map(|_| (next() % seeds.len() as u64) as usize).collect();
        let (bw, bn) = sum(&pick);
        boots.push(fit(&bw, &bn, bed.anchor()));
    }
    println!("mean share, row against column:");
    for i in 0..N {
        let row: Vec<String> =
            (0..N).map(|j| if i == j { "   -  ".into() } else { format!("{:6.3}", w[i][j] / n[i][j]) }).collect();
        println!("  {:>13} {}", POOL[i].2, row.join(" "));
    }
    println!(
        "decided outright: {decided} of {matches} matches; {:.1} s. Rating on the ratio scale, {} = 1, 90% over seeds:",
        t0.elapsed().as_secs_f64(),
        POOL[bed.anchor()].2
    );
    for i in 0..N {
        let mut v: Vec<f64> = boots.iter().map(|b| b[i]).collect();
        v.sort_by(f64::total_cmp);
        let q = |p: f64| v[((v.len() - 1) as f64 * p).round() as usize];
        println!("RATING\t{}\t{}\t{:.4}\t{:.4}\t{:.4}", bed.name(), POOL[i].2, r[i], q(0.05), q(0.95));
    }
    let (cycles, worst, (i, j)) = intransitivity(&w, &n, &r);
    let names: Vec<String> = cycles
        .iter()
        .map(|c| format!("{} > {} > {} > {}", POOL[c[0]].2, POOL[c[1]].2, POOL[c[2]].2, POOL[c[0]].2))
        .collect();
    println!(
        "INTRANSITIVITY\t{}\tcyclic triads {}{}\tlargest residual {:.3} ({} v {}: observed {:.3}, rated {:.3})",
        bed.name(),
        cycles.len(),
        if names.is_empty() { String::new() } else { format!(" [{}]", names.join("; ")) },
        worst,
        POOL[i].2,
        POOL[j].2,
        w[i][j] / n[i][j],
        r[i] / (r[i] + r[j])
    );
    let _ = std::io::stdout().flush();
}

fn main() {
    let which = std::env::args().nth(1).unwrap_or_else(|| "all".into());
    let seeds: Vec<u64> = std::env::args()
        .nth(2)
        .map(|v| v.split(',').filter_map(|s| s.trim().parse().ok()).collect())
        .unwrap_or_else(|| vec![1, 7, 42, 31337]);
    for bed in Bed::ALL {
        if which == "all" || which == bed.name() {
            rate(bed, &seeds);
        }
    }
}
