//! **When does each seat's first forge stand?** — the census behind T-147's
//! forge target (the author: a mean of 400 yr with a 5-yr standard deviation).
//!
//! A forge stands where population reaches `Band IV` (`Simulation::is_forge`),
//! so the census reads each homeworld's population once per simulated year and
//! records the first year it reads `Band IV`. Card-free, standard galaxy.
//!
//! Environment:
//!
//! | variable | meaning | default |
//! |---|---|---|
//! | `FT_SEEDS` | galaxy seeds | 1,7,42,31337 |
//! | `FT_GROUND` | `random`, `identical`, `rotated` | `identical` |
//! | `FT_HOMEWORLDS` | `trio`, `centered` | `centered` |
//! | `FT_START` | `GalaxyConfig::homeworld_start_population`, as a Band reading (`2.785` is `Band II .785`) | the config's |
//! | `FT_FLEETS` | `1` seeds every seat with a miner, a freighter and a colonizer fleet | off |
//! | `FT_SPEND` | kt each seeded fleet is built from (`FleetSeeding::spend_kt`) | 0.3 |
//! | `FT_KNOWN` | surveyed radius at the start, ly (`FleetSeeding::known_radius_ly`) | 15 with fleets, else 0 |
//! | `FT_SPREAD` | `1` runs on to the horizon and reports each seat's colonies and their standard deviation | off |
//!
//! Run: `cargo run --release --example forge_time -- <seats> <horizon>`.
use hyades_engine::autopilot::class_ordered_for;
use hyades_engine::galaxy::{FleetSeeding, Galaxy, GalaxyConfig, Ground, Homeworlds, SeedFleet};
use hyades_engine::math::Vec3;
use hyades_engine::sim::{role_hull_type, HullType, Role, SimConfig, Simulation};
use hyades_engine::units::{Band, BandTier};
use std::io::Write;

fn env<T: std::str::FromStr>(name: &str) -> Option<T> {
    std::env::var(name).ok().and_then(|v| v.trim().parse().ok())
}

fn mean_sd(x: &[f64]) -> (f64, f64) {
    let n = x.len().max(1) as f64;
    let m = x.iter().sum::<f64>() / n;
    (m, (x.iter().map(|v| (v - m) * (v - m)).sum::<f64>() / n).sqrt())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let seats: usize = args.get(1).and_then(|a| a.parse().ok()).unwrap_or(3);
    let horizon: f64 = args.get(2).and_then(|a| a.parse().ok()).unwrap_or(800.0);
    let seeds: Vec<u64> = std::env::var("FT_SEEDS")
        .map(|v| v.split(',').filter_map(|s| s.trim().parse().ok()).collect())
        .unwrap_or_else(|_| vec![1, 7, 42, 31337]);
    let ground = match std::env::var("FT_GROUND").as_deref().map(str::trim) {
        Ok("random") => Ground::Random,
        Ok("rotated") => Ground::ColorRotated,
        _ => Ground::Identical,
    };
    let homeworlds = match std::env::var("FT_HOMEWORLDS").as_deref().map(str::trim) {
        Ok("trio") => Homeworlds::Trio,
        _ => Homeworlds::ColorCentered,
    };
    let fleets_on = std::env::var("FT_FLEETS").is_ok_and(|v| v.trim() == "1");
    let spend: f64 = env("FT_SPEND").unwrap_or(0.3);
    let known: f64 = env("FT_KNOWN").unwrap_or(if fleets_on { 15.0 } else { 0.0 });
    let start: Option<f64> = env("FT_START");
    let spread_on = std::env::var("FT_SPREAD").is_ok_and(|v| v.trim() == "1");
    let mut spreads = Vec::new();
    println!(
        "forge_time: {seats} seats, horizon {horizon} yr, {ground:?}, {homeworlds:?}, start band {start:?}, fleets {fleets_on} \
         (spend {spend} kt each, known {known} ly), seeds {seeds:?}"
    );
    std::io::stdout().flush().ok();
    let mut all = Vec::new();
    let mut within = Vec::new();
    for &seed in &seeds {
        let mut g = GalaxyConfig { ground, homeworlds, ..GalaxyConfig::new(seats, seed) };
        if let Some(b) = start {
            let tier = BandTier::containing(Band::new(b));
            g.homeworld_start_population = (tier, b - tier.band().bands());
        }
        let mut seeding = FleetSeeding { spend_kt: spend, known_radius_ly: known, ..FleetSeeding::default() };
        if fleets_on {
            for seat in 0..seats {
                for (role, hull) in [
                    (Role::Miner, role_hull_type(Role::Miner)),
                    (Role::Freighter, role_hull_type(Role::Freighter)),
                    (Role::Colonizer, HullType::MediumSystems),
                ] {
                    seeding.fleets.push(SeedFleet {
                        seat,
                        hull,
                        class: class_ordered_for(hull),
                        role,
                        position: Vec3::ZERO,
                        velocity: Vec3::ZERO,
                    });
                }
            }
        }
        let galaxy = match Galaxy::generate_with(g, seeding) {
            Ok(g) => g,
            Err(e) => {
                println!("{seed:>5}: {e}");
                continue;
            }
        };
        let homes: Vec<usize> = galaxy.homeworlds.iter().map(|h| h.0 as usize).collect();
        let mut cfg = SimConfig::new(seed);
        cfg.horizon_years = horizon;
        let mut sim = Simulation::with_baseline(galaxy, cfg);
        let mut forged = vec![f64::NAN; seats];
        let mut next = 1.0;
        while sim.step() {
            if sim.clock() < next {
                continue;
            }
            next = sim.clock().floor() + 1.0;
            let snap = sim.snapshot();
            for (p, &h) in homes.iter().enumerate() {
                if forged[p].is_nan() && snap.planets[h].pop_level >= BandTier::IV {
                    forged[p] = sim.clock();
                }
            }
            if !spread_on && forged.iter().all(|t| !t.is_nan()) {
                break;
            }
        }
        let got: Vec<f64> = forged.iter().copied().filter(|t| !t.is_nan()).collect();
        let (_, sd) = mean_sd(&got);
        println!(
            "{seed:>5}: first forge {:?} yr, sd {sd:.1}",
            forged.iter().map(|t| if t.is_nan() { -1.0 } else { t.round() }).collect::<Vec<_>>()
        );
        std::io::stdout().flush().ok();
        if got.len() == seats {
            within.push(sd);
        }
        if spread_on {
            let c: Vec<f64> = (0..seats).map(|p| sim.tree_stock(p).0 as f64).collect();
            let (_, csd) = mean_sd(&c);
            println!(
                "{seed:>5}: colonies at {horizon} yr {:?}, sd {csd:.1}",
                c.iter().map(|v| *v as u64).collect::<Vec<_>>()
            );
            spreads.push(csd);
        }
        all.extend(got);
    }
    let (m, sd) = mean_sd(&all);
    let (w, _) = mean_sd(&within);
    if spread_on {
        let (cm, _) = mean_sd(&spreads);
        println!("colony sd between empires at {horizon} yr, mean over galaxies {cm:.1}");
    }
    println!("first forge over {} seats: mean {m:.1} yr, sd {sd:.1} yr; mean sd within a galaxy {w:.1} yr", all.len());
}
