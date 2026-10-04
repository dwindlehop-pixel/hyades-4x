//! **Is there demand for supers?** — the census for T-146 / R-M5: what the
//! refined books carry at each round barrier, against what forges make.
//!
//! Card-free, standard galaxy, 3 seats. Per seed:
//!
//! - **At each round barrier**, per refined book (Red, Green, Blue, apex) and
//!   summed over empires, kilotonnes: bid, asked from yards, asked from
//!   holdings away from a yard, and filled ([`Simulation::book_census`]).
//! - **Production over the run**, kilotonnes: basics mined; each super and
//!   apex made, and the precursors each drew (the `Synthesized` log).
//! - **Where the supers went**: drawn into apex, and held at the horizon.
//!
//! A refined bid is a center's declined order's refined shortfall, or a
//! forge's want for the supers that complete a balanced set (galaxy §4.5,
//! R-MX18). Only a Design write bills an order in refined material, so in a
//! card-free game the first is zero and every refined bid is a forge's.
//!
//! Run: `cargo run --release --example super_census -- [horizon]`;
//! `SC_SEEDS=2,3,5,11`; `SC_SITE_SPACING` and `SC_SITE_SIGMA` (ly) set the
//! color sites (§4.3) — the galaxy is the only thing a bed varies.
use hyades_engine::galaxy::{Galaxy, GalaxyConfig};
use hyades_engine::log::{LogCategory, LogEvent, LogFilter};
use hyades_engine::resources::Material;
use hyades_engine::sim::{SimConfig, Simulation, MATERIALS};
use std::io::Write;

const SEEDS: [u64; 4] = [1, 7, 42, 31337];
const SEATS: usize = 3;
/// `Material::ALL` indices of the refined books: Red, Green, Blue, apex.
const REFINED: [usize; 4] = [3, 4, 5, 6];
const NAMES: [&str; 4] = ["Red", "Green", "Blue", "apex"];

fn env_f64(name: &str) -> Option<f64> {
    std::env::var(name).ok().and_then(|v| v.trim().parse().ok())
}

fn seeds() -> Vec<u64> {
    match std::env::var("SC_SEEDS") {
        Ok(v) => v.split(',').filter_map(|s| s.trim().parse().ok()).collect(),
        Err(_) => SEEDS.to_vec(),
    }
}

fn galaxy_config(seed: u64) -> GalaxyConfig {
    let mut cfg = GalaxyConfig::new(SEATS, seed);
    if let Some(v) = env_f64("SC_SITE_SPACING") {
        cfg.color_site_spacing_ly = v;
    }
    if let Some(v) = env_f64("SC_SITE_SIGMA") {
        cfg.color_site_sigma_ly = v;
    }
    cfg
}

fn sum(rows: &[[f64; MATERIALS]], i: usize) -> f64 {
    rows.iter().map(|r| r[i]).sum()
}

fn main() {
    let horizon: f64 = std::env::args().nth(1).and_then(|a| a.parse().ok()).unwrap_or(1500.0);
    let probe = galaxy_config(1);
    println!(
        "super_census: {SEATS} seats, horizon {horizon} yr, seeds {:?}, color sites {} ly apart, width {} ly",
        seeds(),
        probe.color_site_spacing_ly,
        probe.color_site_sigma_ly
    );
    std::io::stdout().flush().ok();
    for seed in seeds() {
        let mut cfg = SimConfig::new(seed);
        cfg.horizon_years = horizon;
        let every = cfg.years_per_round;
        let mut next = cfg.years_to_first_round + 1e-6;
        let mut sim = Simulation::with_baseline(Galaxy::generate(galaxy_config(seed)).unwrap(), cfg);
        sim.set_log_filter(LogFilter::none().with(LogCategory::Production).with(LogCategory::Mining));
        println!("{seed:>5}  barrier  book    bid_kt  ask_yard_kt  ask_away_kt  filled_kt  bidding_empires");
        while sim.step() {
            if sim.clock() < next {
                continue;
            }
            next += every;
            let c = sim.book_census();
            for (k, &i) in REFINED.iter().enumerate() {
                let bidders = c.bid.iter().filter(|r| r[i] > 1e-9).count();
                println!(
                    "{seed:>5}  {:>7.0}  {:<5} {:>9.2} {:>12.2} {:>12.2} {:>10.2} {:>16}",
                    sim.clock(),
                    NAMES[k],
                    sum(&c.bid, i),
                    sum(&c.ask_yard, i),
                    sum(&c.ask_away, i),
                    sum(&c.filled, i),
                    bidders
                );
            }
        }
        // Production, from the log.
        let (mut mined, mut made, mut drew) = (0.0f64, [0.0f64; 4], [0.0f64; 4]);
        for r in sim.log().iter() {
            match r.event {
                LogEvent::MineralsExtracted { amount, .. } => mined += amount,
                LogEvent::Synthesized { material, made: m, used, .. } => {
                    if let Some(k) = Material::REFINED.iter().position(|&x| x == material) {
                        made[k] += m;
                        drew[k] += used;
                    }
                }
                _ => {}
            }
        }
        let c = sim.book_census();
        let held: Vec<f64> = REFINED.iter().map(|&i| sum(&c.banked, i) + sum(&c.held_away, i)).collect();
        let traded = sim.exchange_traded();
        println!(
            "{seed:>5}  production kt: basics mined {mined:.0}; Red {:.2}, Green {:.2}, Blue {:.2}, apex {:.2}",
            made[0], made[1], made[2], made[3]
        );
        println!(
            "{seed:>5}  precursors drawn kt: into supers {:.2} basics; into apex {:.2} supers",
            drew[0] + drew[1] + drew[2],
            drew[3]
        );
        println!(
            "{seed:>5}  held at the horizon kt: Red {:.2}, Green {:.2}, Blue {:.2}, apex {:.2} | delivered between empires kt: Red {:.2}, Green {:.2}, Blue {:.2}, apex {:.2}",
            held[0], held[1], held[2], held[3], traded[3], traded[4], traded[5], traded[6]
        );
        std::io::stdout().flush().ok();
    }
}
