//! **The short-range offensive rating of every named armed Design**
//! (`Hyades_technology_tree.md` §4, T-131; beds under the T-133 harness
//! ruling). `design_rating [seeds] [horizon_years]`.
//!
//! Each match is a galaxy generated with two fleets and nothing else changed
//! (the author's ruling: a bed varies only the galaxy, and fleets are generated
//! with it, at equal mineral spend). The two fleets are placed on one point far
//! off the disk — **point blank**, §4.4's start for the short-range role — and
//! the simulation runs its own event loop: detection, discharges, wreck points,
//! the fleets' decisions under the default Doctrine.
//!
//! | quantity | meaning | unit |
//! |---|---|---|
//! | `B` | each fleet's spend: ten General Systems hulls' price (R-TECH11's recommendation) | kt |
//! | `x_A` | side A's dry mass **still holding the field** at the horizon — neither wrecked nor withdrawn (R-TECH18, recommended reading of §4.4's "surviving dry mass") | kt |
//! | `s_AB` | A's score share, `x_A / (x_A + x_B)`, or ½ when both are zero (§4.6) | in [0, 1] |
//! | `R` | the Bradley–Terry maximum-likelihood rating on the Elo scale, one virtual draw per pair (R-TECH5, R-TECH6), Cairn anchored at 0 (R-TECH8) | Elo points |
//!
//! Every pair is played on every seed with the seats swapped (R-TECH16), and
//! the 90% interval on each rating is bootstrapped over seeds.
use hyades_engine::autopilot::{Autopilot, BaselineAutopilot, Doctrine};
use hyades_engine::galaxy::{FleetSeeding, Galaxy, GalaxyConfig, SeedFleet};
use hyades_engine::log::{CourseReason, LogCategory, LogEvent, LogFilter};
use hyades_engine::math::Vec3;
use hyades_engine::sim::{hull_dry_mass, Class, HullType, Role, SimConfig, Simulation};
use std::io::Write;

/// The named armed Designs — every class on a Contact hull.
const POOL: [(HullType, Class); 3] = [
    (HullType::LimitedContactVehicle, Class::Tor),
    (HullType::LimitedContactVehicle, Class::Cairn),
    (HullType::GeneralContactVehicle, Class::Scarp),
];
const ANCHOR: usize = 1;

/// A per-pair quantity over the pool: score summed, or games played.
type Table = [[f64; 3]; 3];

/// One match: `a` on seat 0 against `b` on seat 1. Returns each side's dry mass
/// holding the field, and what the fight did.
fn play(seed: u64, a: usize, b: usize, horizon: f64) -> ([f64; 2], [usize; 2], [usize; 2], f64) {
    let mut g = GalaxyConfig::new(2, seed);
    g.planet_count = 200;
    let cfg = SimConfig::new(seed);
    let mass_cfg = cfg;
    let spend = 10.0 * hull_dry_mass(HullType::GeneralSystems, &cfg).kilotons();
    let at = Vec3::new(0.0, 0.0, 60.0);
    let fleet = |seat: usize, d: usize| SeedFleet {
        seat,
        hull: POOL[d].0,
        class: POOL[d].1,
        role: Role::Picket,
        position: at,
        velocity: Vec3::ZERO,
    };
    let seeding = FleetSeeding { spend_kt: spend, fleets: vec![fleet(0, a), fleet(1, b)] };
    let galaxy = Galaxy::generate_with(g, seeding).unwrap();
    let mut cfg = cfg;
    cfg.horizon_years = horizon;
    // **Both seats hostile.** The author's ruling (warfare §8.19.2): two fleets
    // stand and fight only when both sides' Doctrine is to kill the other's
    // fleet, so a pitched-battle bed seats two such Doctrines. Under the
    // default a neutral that can outrun its attacker leaves on the first hit.
    let hostile = Doctrine { engage_neutrals: true, ..Doctrine::default() };
    let aps: Vec<Box<dyn Autopilot>> =
        (0..2).map(|_| Box::new(BaselineAutopilot::new(hostile)) as Box<dyn Autopilot>).collect();
    let mut sim = Simulation::new(galaxy, cfg, aps);
    sim.set_log_filter(LogFilter::none().with(LogCategory::Combat));
    let t0 = std::time::Instant::now();
    sim.run();
    let secs = t0.elapsed().as_secs_f64();
    let (mut wrecked, mut withdrew) = ([0usize; 2], [0usize; 2]);
    // **Held, read off the log.** Each fleet is `round(B / m)` hulls, and the
    // seeded fleets are the only Pickets on this bed (the default Doctrine
    // fields none), so a fleet has lost exactly the Picket hulls of its seat
    // that were wrecked or withdrew — each counted once.
    let mut gone = [std::collections::BTreeSet::new(), std::collections::BTreeSet::new()];
    for r in sim.log().iter() {
        match r.event {
            LogEvent::HullWrecked { player, vehicle, role: Role::Picket, .. } => {
                wrecked[player as usize] += 1;
                gone[player as usize].insert(vehicle);
            }
            LogEvent::HullWrecked { player, vehicle, role: Role::Reserve, .. } => {
                // A hull that had already withdrawn, wrecked on its way out.
                wrecked[player as usize] += 1;
                gone[player as usize].insert(vehicle);
            }
            LogEvent::CourseChanged { player, vehicle, role: Role::Picket, reason: CourseReason::Withdraw, .. } => {
                withdrew[player as usize] += 1;
                gone[player as usize].insert(vehicle);
            }
            _ => {}
        }
    }
    let mut held = [0.0; 2];
    for (seat, d) in [(0, a), (1, b)] {
        let m = hull_dry_mass(POOL[d].0, &mass_cfg).kilotons();
        let count = (spend / m).round() as usize;
        held[seat] = (count - gone[seat].len()) as f64 * m;
    }
    (held, wrecked, withdrew, secs)
}

/// Bradley–Terry by minorization–maximization (Hunter 2004) on fractional
/// wins, with one virtual draw per pair. Returns Elo points, anchor at 0.
fn fit(wins: &Table, games: &Table) -> [f64; 3] {
    let mut w = *wins;
    let mut n = *games;
    for i in 0..3 {
        for j in 0..3 {
            if i != j {
                w[i][j] += 0.5;
                n[i][j] += 1.0;
            }
        }
    }
    let mut gamma = [1.0; 3];
    for _ in 0..10_000 {
        let mut next = gamma;
        for i in 0..3 {
            let won: f64 = (0..3).filter(|&j| j != i).map(|j| w[i][j]).sum();
            let den: f64 = (0..3).filter(|&j| j != i).map(|j| n[i][j] / (gamma[i] + gamma[j])).sum();
            next[i] = won / den;
        }
        let a = next[ANCHOR];
        for g in next.iter_mut() {
            *g /= a;
        }
        gamma = next;
    }
    gamma.map(|g| 400.0 * g.log10())
}

fn main() {
    let seeds: Vec<u64> = std::env::args()
        .nth(1)
        .map(|v| v.split(',').filter_map(|s| s.trim().parse().ok()).collect())
        .unwrap_or_else(|| vec![1, 7, 42, 31337]);
    let horizon: f64 = std::env::args().nth(2).and_then(|s| s.parse().ok()).unwrap_or(1.0);
    println!("short-range bed: {} seeds, horizon {horizon} yr, pool {:?}", seeds.len(), POOL.map(|p| p.1));
    let _ = std::io::stdout().flush();
    // per seed: shares[seed][i][j] summed over the two seatings, and games
    let mut per_seed: Vec<(Table, Table)> = Vec::new();
    for &seed in &seeds {
        let (mut wins, mut games) = ([[0.0; 3]; 3], [[0.0; 3]; 3]);
        for a in 0..3 {
            for b in 0..3 {
                if a == b {
                    continue;
                }
                let (held, wrecked, withdrew, secs) = play(seed, a, b, horizon);
                let tot = held[0] + held[1];
                let s = if tot > 0.0 { held[0] / tot } else { 0.5 };
                wins[a][b] += s;
                wins[b][a] += 1.0 - s;
                games[a][b] += 1.0;
                games[b][a] += 1.0;
                println!(
                    "MATCH seed {seed} {:?} v {:?}: held {:.3} / {:.3} kt  share {s:.3}  wrecked {wrecked:?}  withdrew {withdrew:?}  {secs:.1} s",
                    POOL[a].1, POOL[b].1, held[0], held[1]
                );
                let _ = std::io::stdout().flush();
            }
        }
        per_seed.push((wins, games));
    }
    let sum = |set: &[usize]| {
        let (mut w, mut n) = ([[0.0; 3]; 3], [[0.0; 3]; 3]);
        for &k in set {
            for i in 0..3 {
                for j in 0..3 {
                    w[i][j] += per_seed[k].0[i][j];
                    n[i][j] += per_seed[k].1[i][j];
                }
            }
        }
        (w, n)
    };
    let all: Vec<usize> = (0..seeds.len()).collect();
    let (w, n) = sum(&all);
    let r = fit(&w, &n);
    // Bootstrap over seeds, a fixed splitmix stream so a rerun prints the same.
    let mut state: u64 = 0x05EE_DE10;
    let mut next = || {
        state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    };
    let mut boots: Vec<[f64; 3]> = Vec::new();
    for _ in 0..2000 {
        let pick: Vec<usize> = (0..seeds.len()).map(|_| (next() % seeds.len() as u64) as usize).collect();
        let (bw, bn) = sum(&pick);
        boots.push(fit(&bw, &bn));
    }
    println!("\nmean share, row against column (both seatings, all seeds):");
    for i in 0..3 {
        let row: Vec<String> =
            (0..3).map(|j| if i == j { "   -  ".into() } else { format!("{:6.3}", w[i][j] / n[i][j]) }).collect();
        println!("  {:>6?} {}", POOL[i].1, row.join(" "));
    }
    println!("\nrating (Elo points, {:?} = 0), 90% interval bootstrapped over seeds:", POOL[ANCHOR].1);
    for i in 0..3 {
        let mut v: Vec<f64> = boots.iter().map(|b| b[i]).collect();
        v.sort_by(f64::total_cmp);
        let q = |p: f64| v[((v.len() - 1) as f64 * p).round() as usize];
        println!("  {:>6?}: {:+8.1}  [{:+8.1}, {:+8.1}]", POOL[i].1, r[i], q(0.05), q(0.95));
    }
}
