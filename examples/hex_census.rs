//! **Hexes per player, and the color each empire holds** — the census behind
//! the hex width (galaxy §1, R-G1) and empire-scale color (§4.3).
//!
//! Card-free, standard galaxy. Each seed runs to the horizon; per seat it
//! prints the worlds the empire owns, the share of them nearest its own
//! homeworld, and the deposit its owned worlds were generated with, by color
//! (kilotonne share, and the count holding at least `Band I` = 1 kt). Then, for
//! each hex width given, the number of flat-top hexes — laid so every
//! homeworld stands in its own hex (`GalaxyConfig::hex_grid_origin`) — holding 90% of each
//! empire's owned worlds, against the author's target for the seat count.
//!
//! Run: `cargo run --release --example hex_census -- <seats> <horizon> [width ly ...]`;
//! `HC_SEEDS=2,3,5,11` for another seed set. Defaults: 3 seats, 1,500 yr, the
//! shipped width.
use hyades_engine::galaxy::{Galaxy, GalaxyConfig};
use hyades_engine::resources::Basic;
use hyades_engine::sim::{SimConfig, Simulation};
use std::collections::BTreeMap;
use std::io::Write;

const SEEDS: [u64; 4] = [1, 7, 42, 31337];
const SQRT_3: f64 = 1.732_050_807_568_877_2;

fn seeds() -> Vec<u64> {
    match std::env::var("HC_SEEDS") {
        Ok(v) => v.split(',').filter_map(|s| s.trim().parse().ok()).collect(),
        Err(_) => SEEDS.to_vec(),
    }
}

/// The author's target, hexes per player (R-G1).
fn target(seats: usize) -> (usize, usize) {
    match seats {
        6 | 12 => (6, 12),
        _ => (3, 6),
    }
}

/// The flat-top hex of side `side` holding `(x, y)`, as axial `(q, r)`.
fn hex_at(x: f64, y: f64, side: f64) -> (i64, i64) {
    let fq = (2.0 / 3.0) * x / side;
    let fr = (-x / 3.0 + (SQRT_3 / 3.0) * y) / side;
    let fs = -fq - fr;
    let (mut q, mut r, s) = (fq.round(), fr.round(), fs.round());
    let (dq, dr, ds) = ((q - fq).abs(), (r - fr).abs(), (s - fs).abs());
    if dq > dr && dq > ds {
        q = -r - s;
    } else if dr > ds {
        r = -q - s;
    }
    (q as i64, r as i64)
}

/// Hexes, fewest first, holding 90% of `points`, on the grid laid from
/// `origin` (a hex center).
fn hexes_for_90(points: &[(f64, f64)], width: f64, origin: (f64, f64)) -> usize {
    let side = width / SQRT_3;
    let mut counts: BTreeMap<(i64, i64), usize> = BTreeMap::new();
    for &(x, y) in points {
        *counts.entry(hex_at(x - origin.0, y - origin.1, side)).or_default() += 1;
    }
    let mut v: Vec<usize> = counts.into_values().collect();
    v.sort_unstable_by(|a, b| b.cmp(a));
    let (mut held, mut k) = (0usize, 0usize);
    for c in v {
        if held as f64 >= 0.9 * points.len() as f64 {
            break;
        }
        held += c;
        k += 1;
    }
    k
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let seats: usize = args.get(1).and_then(|a| a.parse().ok()).unwrap_or(3);
    let horizon: f64 = args.get(2).and_then(|a| a.parse().ok()).unwrap_or(1500.0);
    let mut widths: Vec<f64> = args.iter().skip(3).filter_map(|a| a.parse().ok()).collect();
    if widths.is_empty() {
        widths.push(GalaxyConfig::new(seats, 1).hex_across_flats_ly());
    }
    println!("hex_census: {seats} seats, horizon {horizon} yr, seeds {:?}", seeds());
    std::io::stdout().flush().ok();
    // Per empire, the positions of the worlds it owns at the horizon.
    let mut empires: Vec<Vec<(f64, f64)>> = Vec::new();
    for seed in seeds() {
        let galaxy = Galaxy::generate(GalaxyConfig::new(seats, seed)).unwrap();
        let homes: Vec<(f64, f64)> = galaxy
            .homeworlds
            .iter()
            .map(|h| {
                let p = galaxy.planets[h.0 as usize].position;
                (p.x, p.y)
            })
            .collect();
        let nearest = |x: f64, y: f64| {
            (0..seats)
                .min_by(|&a, &b| {
                    let da = (x - homes[a].0) * (x - homes[a].0) + (y - homes[a].1) * (y - homes[a].1);
                    let db = (x - homes[b].0) * (x - homes[b].0) + (y - homes[b].1) * (y - homes[b].1);
                    da.total_cmp(&db)
                })
                .unwrap()
        };
        let deposits: Vec<[f64; 3]> =
            galaxy.planets.iter().map(|p| Basic::ALL.map(|b| p.minerals.get(b).kilotons())).collect();
        let mut cfg = SimConfig::new(seed);
        cfg.horizon_years = horizon;
        let mut sim = Simulation::with_baseline(galaxy, cfg);
        let t0 = std::time::Instant::now();
        sim.run();
        let snap = sim.snapshot();
        let mut owned: Vec<Vec<usize>> = vec![Vec::new(); seats];
        for (i, p) in snap.planets.iter().enumerate() {
            if let Some(o) = p.owner {
                owned[o as usize].push(i);
            }
        }
        for (seat, worlds) in owned.iter().enumerate() {
            let pos = |i: usize| (snap.planets[i].position.x, snap.planets[i].position.y);
            let inside = worlds.iter().filter(|&&i| nearest(pos(i).0, pos(i).1) == seat).count();
            let (mut kt, mut rich) = ([0.0f64; 3], [0usize; 3]);
            for &i in worlds {
                for c in 0..3 {
                    kt[c] += deposits[i][c];
                    rich[c] += usize::from(deposits[i][c] >= 1.0);
                }
            }
            let total: f64 = kt.iter().sum::<f64>().max(1e-12);
            println!(
                "{seed:>5} seat {seat:>2}: owned {:>5}, nearest its homeworld {:>5.1}% | deposit C/M/Y {:>3.0}/{:>3.0}/{:>3.0}%, worlds >= 1 kt {}/{}/{}",
                worlds.len(),
                100.0 * inside as f64 / worlds.len().max(1) as f64,
                100.0 * kt[0] / total,
                100.0 * kt[1] / total,
                100.0 * kt[2] / total,
                rich[0],
                rich[1],
                rich[2],
            );
            empires.push(worlds.iter().map(|&i| pos(i)).collect());
        }
        println!("{seed:>5}: {:.1} s", t0.elapsed().as_secs_f64());
        std::io::stdout().flush().ok();
    }
    let (lo, hi) = target(seats);
    for w in widths {
        // The grid each width lays, anchored as the command view anchors it.
        let origin = GalaxyConfig { hex_side_ly: w / SQRT_3, ..GalaxyConfig::new(seats, 1) }.hex_grid_origin();
        let k: Vec<usize> = empires.iter().map(|e| hexes_for_90(e, w, origin)).collect();
        let mean = k.iter().sum::<usize>() as f64 / k.len().max(1) as f64;
        let inside = k.iter().filter(|&&x| (lo..=hi).contains(&x)).count();
        println!(
            "width {w:>5.1} ly: hexes holding 90% of owned worlds, mean {mean:.2}, range {}-{}, {inside}/{} in the target {lo}-{hi}",
            k.iter().min().unwrap_or(&0),
            k.iter().max().unwrap_or(&0),
            k.len()
        );
    }
}
