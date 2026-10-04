//! **What separates the seats on one ground** — the census behind T-147's
//! `Ground::Identical` arm.
//!
//! Card-free, standard galaxy on the ground `GC_GROUND` names (`random`,
//! `identical`, `rotated`; default `identical`). Per seed and seat: the
//! archetype's poor color, the wild deposit by color within `GC_REACH` ly of
//! the homeworld (default 40), and at each sample year the seat's colonies and
//! its stockpile by color, summed over the worlds it owns.
//!
//! Run: `cargo run --release --example ground_census -- <seats> <horizon> [sample years ...]`;
//! `GC_SEEDS=2,3,5,11`.
use hyades_engine::galaxy::{Galaxy, GalaxyConfig, Ground};
use hyades_engine::resources::Basic;
use hyades_engine::sim::{SimConfig, Simulation};
use std::io::Write;

const SEEDS: [u64; 4] = [1, 7, 42, 31337];

fn env_seeds() -> Vec<u64> {
    match std::env::var("GC_SEEDS") {
        Ok(v) => v.split(',').filter_map(|s| s.trim().parse().ok()).collect(),
        Err(_) => SEEDS.to_vec(),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let seats: usize = args.get(1).and_then(|a| a.parse().ok()).unwrap_or(3);
    let horizon: f64 = args.get(2).and_then(|a| a.parse().ok()).unwrap_or(800.0);
    let mut samples: Vec<f64> = args.iter().skip(3).filter_map(|a| a.parse().ok()).collect();
    if samples.is_empty() {
        samples = vec![50.0, 100.0, 200.0, 400.0, horizon];
    }
    samples.retain(|&t| t <= horizon);
    let ground = match std::env::var("GC_GROUND").as_deref().map(str::trim) {
        Ok("random") => Ground::Random,
        Ok("rotated") => Ground::ColorRotated,
        _ => Ground::Identical,
    };
    let reach: f64 = std::env::var("GC_REACH").ok().and_then(|v| v.trim().parse().ok()).unwrap_or(40.0);
    println!("ground_census: {seats} seats, {ground:?}, horizon {horizon} yr, reach {reach} ly");
    std::io::stdout().flush().ok();
    for seed in env_seeds() {
        let galaxy = Galaxy::generate(GalaxyConfig { ground, ..GalaxyConfig::new(seats, seed) }).unwrap();
        for (seat, &h) in galaxy.homeworlds.iter().enumerate() {
            let home = &galaxy.planets[h.0 as usize];
            let (_, _, poor) = home.archetype.unwrap().alignment();
            let mut kt = [0.0f64; 3];
            let mut rich = [0usize; 3];
            for p in &galaxy.planets {
                if p.is_homeworld || p.position.distance(home.position) > reach {
                    continue;
                }
                for (c, &b) in Basic::ALL.iter().enumerate() {
                    let m = p.minerals.get(b).kilotons();
                    kt[c] += m;
                    rich[c] += usize::from(m >= 1.0);
                }
            }
            println!(
                "{seed:>5} seat {seat}: poor {poor:?} | within {reach} ly C/M/Y {:.0}/{:.0}/{:.0} kt, worlds >= 1 kt {}/{}/{}",
                kt[0], kt[1], kt[2], rich[0], rich[1], rich[2]
            );
        }
        let mut cfg = SimConfig::new(seed);
        cfg.horizon_years = horizon;
        let mut sim = Simulation::with_baseline(galaxy, cfg);
        let mut k = 0;
        let take = |sim: &Simulation, t: f64| {
            let snap = sim.snapshot();
            let mut line = format!("{seed:>5} {t:>5.0} yr");
            for seat in 0..seats {
                let mut stock = [0.0f64; 3];
                for p in snap.planets.iter().filter(|p| p.owner == Some(seat as u32)) {
                    for (c, &b) in Basic::ALL.iter().enumerate() {
                        stock[c] += p.stockpile.get_basic(b);
                    }
                }
                line += &format!(
                    " | s{seat} col {:>4} stock C/M/Y {:>7.1}/{:>7.1}/{:>7.1}",
                    sim.tree_stock(seat).0,
                    stock[0],
                    stock[1],
                    stock[2]
                );
            }
            println!("{line}");
            std::io::stdout().flush().ok();
        };
        while sim.step() {
            while k < samples.len() && sim.clock() >= samples[k] {
                take(&sim, samples[k]);
                k += 1;
            }
        }
        while k < samples.len() {
            take(&sim, samples[k]);
            k += 1;
        }
    }
}
