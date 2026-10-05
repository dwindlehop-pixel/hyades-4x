//! **The forge's price, swept on the twin bed** — T-147's Monte Carlo for
//! [`Doctrine::forge_premium`], [`Doctrine::forge_price_floor`] and
//! [`Doctrine::forge_holding_scale`].
//!
//! A forge can only be judged where its output is wanted, so the bed is the
//! standard galaxy with every seat seeded twin Designs paid a third each in
//! Red, Green and Blue (`FleetSeeding::twin_bill`, galaxy §3.1): supers have a
//! final demand in hulls. Card-free, 3 seats.
//!
//! Per seed, one line per seat and one for the galaxy: the tree stocks
//! `examples/empire_spread` integrates on a 25-year grid — Expansion `∫ C dt`
//! (colony-years), Growth `∫ V dt` (works, kt-years), Production `∫ F dt`
//! (fleet volume, hull-units³-years) — and supers and apex forged and hull
//! kilotonnes paid in refined material.
//!
//! Environment: `FS_SEEDS` (default 1,7,42,31337), `FS_PREMIUM`, `FS_FLOOR`,
//! `FS_SCALE`, `FS_COMPLETION` (the forge's three Doctrine fields and
//! [`Doctrine::completion_exponent`]; defaults are the shipped ones),
//! `FS_GROUND` (`random`, `identical`, `rotated`; default `random`),
//! `FS_LAMBDA` and `FS_STOPS` ([`SimConfig::trade_decay_lambda`] and
//! [`SimConfig::max_pickup_stops`]; defaults are the shipped ones).
//!
//! Each seat's line also counts its **centers built to `Band IV` works** at
//! the horizon (`band4`): owned worlds whose infrastructure stands at the top
//! whole Band. A line per seed gives the run's event count.
//!
//! Run: `cargo run --release --example forge_sweep -- [horizon]`.
use hyades_engine::autopilot::{Autopilot, BaselineAutopilot, Doctrine};
use hyades_engine::galaxy::{FleetSeeding, Galaxy, GalaxyConfig, Ground};
use hyades_engine::log::{LogCategory, LogEvent, LogFilter};
use hyades_engine::resources::Material;
use hyades_engine::sim::{DesignBill, SimConfig, Simulation};
use hyades_engine::units::BandTier;
use std::io::Write;

const SEATS: usize = 3;
const SAMPLE_YEARS: f64 = 25.0;

fn env<T: std::str::FromStr>(name: &str) -> Option<T> {
    std::env::var(name).ok().and_then(|v| v.trim().parse().ok())
}

fn main() {
    let horizon: f64 = std::env::args().nth(1).and_then(|a| a.parse().ok()).unwrap_or(1500.0);
    let seeds: Vec<u64> = std::env::var("FS_SEEDS")
        .map(|v| v.split(',').filter_map(|s| s.trim().parse().ok()).collect())
        .unwrap_or_else(|_| vec![1, 7, 42, 31337]);
    let ground = match std::env::var("FS_GROUND").as_deref().map(str::trim) {
        Ok("identical") => Ground::Identical,
        Ok("rotated") => Ground::ColorRotated,
        _ => Ground::Random,
    };
    let base = Doctrine::default();
    let doctrine = Doctrine {
        forge_premium: env("FS_PREMIUM").unwrap_or(base.forge_premium),
        forge_price_floor: env("FS_FLOOR").unwrap_or(base.forge_price_floor),
        forge_holding_scale: env("FS_SCALE").unwrap_or(base.forge_holding_scale),
        completion_exponent: env("FS_COMPLETION").unwrap_or(base.completion_exponent),
        ..base
    };
    let tag = format!(
        "premium {} floor {} scale {} completion {}",
        doctrine.forge_premium, doctrine.forge_price_floor, doctrine.forge_holding_scale, doctrine.completion_exponent
    );
    println!("forge_sweep: {SEATS} seats, horizon {horizon} yr, {ground:?}, twin bed, {tag}");
    std::io::stdout().flush().ok();
    for seed in seeds {
        let seeding = FleetSeeding {
            twin_bill: Some(DesignBill::coerced([1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0, 0.0])),
            ..Default::default()
        };
        let galaxy = Galaxy::generate_with(GalaxyConfig { ground, ..GalaxyConfig::new(SEATS, seed) }, seeding).unwrap();
        let mut cfg = SimConfig::new(seed);
        cfg.horizon_years = horizon;
        cfg.trade_decay_lambda = env("FS_LAMBDA").unwrap_or(cfg.trade_decay_lambda);
        cfg.max_pickup_stops = env("FS_STOPS").unwrap_or(cfg.max_pickup_stops);
        let autopilots: Vec<Box<dyn Autopilot>> =
            (0..SEATS).map(|_| Box::new(BaselineAutopilot::new(doctrine)) as Box<dyn Autopilot>).collect();
        let top = BandTier::MAX_PLAYABLE.band().bands() - 1e-6;
        let mut sim = Simulation::new(galaxy, cfg, autopilots);
        sim.set_log_filter(LogFilter::none().with(LogCategory::Production));
        let mut acc = [[0.0f64; 3]; SEATS];
        let mut prev = [[0.0f64; 3]; SEATS];
        let (mut prev_t, mut next) = (0.0, SAMPLE_YEARS);
        loop {
            let mut running = true;
            while sim.clock() < next {
                if !sim.step() {
                    running = false;
                    break;
                }
            }
            let snap = sim.snapshot();
            let mut now = [[0.0f64; 3]; SEATS];
            for pl in &snap.planets {
                if let Some(o) = pl.owner {
                    now[o as usize][0] += 1.0;
                    now[o as usize][1] += pl.works.kilotons();
                }
            }
            for v in &snap.vehicles {
                now[v.owner as usize][2] += v.volume.hull_units_cubed();
            }
            let t = sim.clock().min(next).min(horizon);
            for p in 0..SEATS {
                for m in 0..3 {
                    acc[p][m] += 0.5 * (now[p][m] + prev[p][m]) * (t - prev_t);
                }
            }
            prev = now;
            prev_t = t;
            next += SAMPLE_YEARS;
            if !running || t >= horizon {
                break;
            }
        }
        // Supers and apex forged, and hull kilotonnes paid in refined material.
        let mut forged = [[0.0f64; 3]; SEATS];
        for r in sim.log().iter() {
            match r.event {
                LogEvent::Synthesized { player, material, made, .. } => {
                    forged[player as usize][if material == Material::Apex { 1 } else { 0 }] += made;
                }
                LogEvent::BuildApplied { player, refined_paid, .. } => forged[player as usize][2] += refined_paid,
                _ => {}
            }
        }
        let mut band4 = [0usize; SEATS];
        for pl in &sim.snapshot().planets {
            if let (Some(o), true) = (pl.owner, pl.infrastructure.bands() >= top) {
                band4[o as usize] += 1;
            }
        }
        println!("{seed:>5} events {}", sim.events_processed());
        for p in 0..SEATS {
            println!(
                "{seed:>5} s{p} expansion {:.1} growth {:.1} production {:.1} supers {:.1} apex {:.1} refined_paid {:.3} band4 {}",
                acc[p][0], acc[p][1], acc[p][2], forged[p][0], forged[p][1], forged[p][2], band4[p]
            );
        }
        std::io::stdout().flush().ok();
    }
}
