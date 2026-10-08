//! **What each seat's ground holds at generation** — galaxy-only, no
//! simulation (T-147). The census behind the question of what separates the
//! seats of one galaxy on `Ground::Random`.
//!
//! Per seed and seat, over the seat's **cell** (the worlds nearer its
//! homeworld than any other homeworld) and within `SG_REACH` ly of the
//! homeworld (default 40):
//!
//! | column | meaning |
//! |---|---|
//! | `adm` | worlds `k_high` admits (`k_potential ≥ k_high`), not homeworlds or their companions |
//! | `near` | `Σ exp(−d / SG_SCALE)` over admitted worlds of the cell, `d` the distance to the homeworld in ly (default scale 20) |
//! | `C M Y` | wild deposit by color in the cell, Mt |
//! | `rC rM rY` | wild deposit by color within the reach, Mt |
//!
//! `SG_GROUND` (`random`, `identical`, `rotated`; default `random`),
//! `SG_SEEDS` (default 1, 7, 42, 31337, 2, 3, 5, 11).
//!
//! Run: `cargo run --release --example seat_ground -- <seats>`.
use hyades_engine::autopilot::Doctrine;
use hyades_engine::galaxy::{Galaxy, GalaxyConfig, Ground};
use hyades_engine::resources::Basic;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let seats: usize = args.get(1).and_then(|a| a.parse().ok()).unwrap_or(3);
    let seeds: Vec<u64> = std::env::var("SG_SEEDS")
        .map(|v| v.split(',').filter_map(|s| s.trim().parse().ok()).collect())
        .unwrap_or_else(|_| vec![1, 7, 42, 31337, 2, 3, 5, 11]);
    let ground = match std::env::var("SG_GROUND").as_deref().map(str::trim) {
        Ok("identical") => Ground::Identical,
        Ok("rotated") => Ground::ColorRotated,
        _ => Ground::Random,
    };
    let env = |k: &str, d: f64| std::env::var(k).ok().and_then(|v| v.trim().parse().ok()).unwrap_or(d);
    let reach = env("SG_REACH", 40.0);
    let scale = env("SG_SCALE", 20.0);
    let k_high = Doctrine::default().rank.k_high;
    println!("seat_ground: {seats} seats, {ground:?}, reach {reach} ly, scale {scale} ly, k_high {k_high}");
    for seed in seeds {
        let g = Galaxy::generate(GalaxyConfig {
            ground,
            fair_start_ly: std::env::var("SG_FAIR").ok().and_then(|v| v.trim().parse().ok()).unwrap_or(0.0),
            color_site_clearance_ly: std::env::var("SG_CLEAR").ok().and_then(|v| v.trim().parse().ok()).unwrap_or(0.0),
            homeworlds: if std::env::var("SG_HOMEWORLDS").is_ok_and(|v| v.trim() == "centered") {
                hyades_engine::galaxy::Homeworlds::ColorCentered
            } else {
                hyades_engine::galaxy::Homeworlds::Trio
            },
            ..GalaxyConfig::new(seats, seed)
        })
        .unwrap();
        let homes: Vec<_> = g.planets.iter().filter(|p| p.is_homeworld).collect();
        println!(
            "{seed:>6} worlds {} homeworld spacing {:.1} ly",
            g.planets.len(),
            homes[0].position.distance(homes[1 % homes.len()].position)
        );
        let mut adm = vec![0usize; homes.len()];
        let mut near = vec![0.0; homes.len()];
        let mut cell = vec![[0.0; 3]; homes.len()];
        let mut within = vec![[0.0; 3]; homes.len()];
        for p in g.planets.iter().filter(|p| !p.is_homeworld) {
            let (s, d) = homes
                .iter()
                .enumerate()
                .map(|(i, h)| (i, p.position.distance(h.position)))
                .fold((0, f64::INFINITY), |a, b| if b.1 < a.1 { b } else { a });
            // Companions are owned at generation; they are not ground to race for.
            if p.owner.is_none() && p.k_potential() >= k_high {
                adm[s] += 1;
                near[s] += hyades_engine::transcendental::exp(-d / scale);
            }
            for b in Basic::ALL {
                let kt = p.minerals.get(b).kilotons() / 1000.0;
                cell[s][b as usize] += kt;
                if d <= reach {
                    within[s][b as usize] += kt;
                }
            }
        }
        for (i, h) in homes.iter().enumerate() {
            let c = cell[i];
            let r = within[i];
            println!(
                "{seed:>6} s{} {:?} adm {:>5} near {:>7.2} C {:>9.1} M {:>9.1} Y {:>9.1} rC {:>9.1} rM {:>9.1} rY {:>9.1}",
                h.owner.map_or(9, |o| o.0),
                h.archetype.map(|a| a.native_super()),
                adm[i],
                near[i],
                c[0],
                c[1],
                c[2],
                r[0],
                r[1],
                r[2]
            );
        }
    }
}
