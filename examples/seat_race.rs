//! **Where the seats of one galaxy part** — the race for territory, per seat,
//! over time (T-147). Card-free, the twin bed of `examples/forge_sweep`.
//!
//! A seat's **cell** is the worlds nearer its homeworld than any other
//! homeworld. Every `SR_STEP` years (default 100), per seat: worlds owned,
//! of them how many lie in its own cell (`home`) and in a rival's (`away`),
//! and how many worlds of its own cell a rival owns (`lost`); and colony-years
//! so far (`∫ C dt`, sampled every 25 yr).
//!
//! `SR_FAIR` (`GalaxyConfig::fair_start_ly`), `SR_HOMEWORLDS=centered`,
//! `SR_SEEDS` (default 1, 7, 42, 31337), `SR_GROUND` (`random`, `identical`,
//! `rotated`), `SR_LAMBDA` and `SR_STOPS` as `forge_sweep`'s.
//!
//! Run: `cargo run --release --example seat_race -- <horizon>`.
use hyades_engine::autopilot::{Autopilot, BaselineAutopilot, Doctrine};
use hyades_engine::galaxy::{FleetSeeding, Galaxy, GalaxyConfig, Ground};
use hyades_engine::log::{LogCategory, LogEvent, LogFilter};
use hyades_engine::sim::{DesignBill, SimConfig, Simulation};
use std::io::Write;

const SEATS: usize = 3;
const SAMPLE_YEARS: f64 = 25.0;

fn env<T: std::str::FromStr>(name: &str) -> Option<T> {
    std::env::var(name).ok().and_then(|v| v.trim().parse().ok())
}

fn main() {
    let horizon: f64 = std::env::args().nth(1).and_then(|a| a.parse().ok()).unwrap_or(1500.0);
    let seeds: Vec<u64> = std::env::var("SR_SEEDS")
        .map(|v| v.split(',').filter_map(|s| s.trim().parse().ok()).collect())
        .unwrap_or_else(|_| vec![1, 7, 42, 31337]);
    let ground = match std::env::var("SR_GROUND").as_deref().map(str::trim) {
        Ok("identical") => Ground::Identical,
        Ok("rotated") => Ground::ColorRotated,
        _ => Ground::Random,
    };
    let step: f64 = env("SR_STEP").unwrap_or(100.0);
    println!("seat_race: {SEATS} seats, horizon {horizon} yr, {ground:?}, twin bed");
    std::io::stdout().flush().ok();
    for seed in seeds {
        let seeding = FleetSeeding {
            twin_bill: Some(DesignBill::coerced([1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0, 0.0])),
            ..Default::default()
        };
        let galaxy = Galaxy::generate_with(
            GalaxyConfig {
                ground,
                fair_start_ly: std::env::var("SR_FAIR").ok().and_then(|v| v.trim().parse().ok()).unwrap_or(0.0),
                homeworlds: if std::env::var("SR_HOMEWORLDS").is_ok_and(|v| v.trim() == "centered") {
                    hyades_engine::galaxy::Homeworlds::ColorCentered
                } else {
                    hyades_engine::galaxy::Homeworlds::Trio
                },
                ..GalaxyConfig::new(SEATS, seed)
            },
            seeding,
        )
        .unwrap();
        let homes: Vec<_> = galaxy.planets.iter().filter(|p| p.is_homeworld).map(|p| p.position).collect();
        let cell: Vec<usize> = galaxy
            .planets
            .iter()
            .map(|p| {
                (0..homes.len())
                    .map(|i| (i, p.position.distance(homes[i])))
                    .fold((0, f64::INFINITY), |a, b| if b.1 < a.1 { b } else { a })
                    .0
            })
            .collect();
        let mut cfg = SimConfig::new(seed);
        cfg.horizon_years = horizon;
        cfg.trade_decay_lambda = env("SR_LAMBDA").unwrap_or(cfg.trade_decay_lambda);
        cfg.max_pickup_stops = env("SR_STOPS").unwrap_or(cfg.max_pickup_stops);
        let autopilots: Vec<Box<dyn Autopilot>> =
            (0..SEATS).map(|_| Box::new(BaselineAutopilot::new(Doctrine::default())) as Box<dyn Autopilot>).collect();
        let early = std::env::var("SR_EARLY").is_ok_and(|v| v.trim() == "1");
        let positions: Vec<_> = galaxy.planets.iter().map(|p| p.position).collect();
        let mut sim = Simulation::new(galaxy, cfg, autopilots);
        if early {
            sim.set_log_filter(LogFilter::none().with(LogCategory::Vehicles).with(LogCategory::Mining));
        }
        let mut acc = [0.0f64; SEATS];
        let mut prev = [0.0f64; SEATS];
        let (mut prev_t, mut next, mut report) = (0.0, SAMPLE_YEARS, step);
        loop {
            let mut running = true;
            while sim.clock() < next {
                if !sim.step() {
                    running = false;
                    break;
                }
            }
            let snap = sim.snapshot();
            let mut now = [0.0f64; SEATS];
            let mut home = [0usize; SEATS];
            let mut away = [0usize; SEATS];
            let mut lost = [0usize; SEATS];
            for pl in &snap.planets {
                if let Some(o) = pl.owner {
                    let o = o as usize;
                    now[o] += 1.0;
                    let c = cell[pl.id.0 as usize];
                    if c == o {
                        home[o] += 1;
                    } else {
                        away[o] += 1;
                        lost[c] += 1;
                    }
                }
            }
            let t = sim.clock().min(next).min(horizon);
            for p in 0..SEATS {
                acc[p] += 0.5 * (now[p] + prev[p]) * (t - prev_t);
            }
            prev = now;
            prev_t = t;
            if t + 1e-9 >= report || !running || t >= horizon {
                let line: Vec<String> = (0..SEATS)
                    .map(|p| {
                        format!(
                            "s{p} C {:>5} home {:>5} away {:>4} lost {:>4} cy {:>9.0}",
                            now[p], home[p], away[p], lost[p], acc[p]
                        )
                    })
                    .collect();
                println!("{seed:>6} {t:>6.0} yr  {}", line.join("  |  "));
                std::io::stdout().flush().ok();
                report += step;
            }
            next += SAMPLE_YEARS;
            if !running || t >= horizon {
                break;
            }
        }
        if early {
            // Each seat's foundings (year, distance from its homeworld in ly)
            // and the ore its outposts and centers mined, per 25 yr.
            let mut found: Vec<Vec<(f64, f64)>> = vec![Vec::new(); SEATS];
            let buckets = (horizon / SAMPLE_YEARS).ceil() as usize + 1;
            let mut mined = vec![vec![0.0f64; buckets]; SEATS];
            for r in sim.log().iter() {
                match r.event {
                    LogEvent::ColonyFounded { player, planet, .. } => {
                        let p = player as usize;
                        found[p].push((r.time, positions[planet.0 as usize].distance(homes[p])));
                    }
                    LogEvent::MineralsExtracted { player, amount, .. } => {
                        mined[player as usize][(r.time / SAMPLE_YEARS) as usize] += amount;
                    }
                    _ => {}
                }
            }
            for p in 0..SEATS {
                let f: Vec<String> = found[p].iter().take(12).map(|(t, d)| format!("{t:.0}@{d:.1}")).collect();
                let m: Vec<String> = mined[p].iter().map(|v| format!("{v:.0}")).collect();
                println!("{seed:>6} s{p} founded {}", f.join(" "));
                println!("{seed:>6} s{p} mined/25yr {}", m.join(" "));
            }
        }
    }
}
