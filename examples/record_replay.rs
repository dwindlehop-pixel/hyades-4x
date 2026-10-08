//! **Record the demo replays the viewer ships with** (`docs/Hyades_interface.md`
//! §2, §8).
//!
//! Writes each replay as `<out>/<name>.json` and an `<out>/index.json` listing
//! them, for the web viewer (`web/`) to load. Three scenarios:
//!
//! | name | what it shows |
//! |---|---|
//! | `expansion` | three seats founding, mining and hauling over 400 yr |
//! | `beams` | a Tor stack closing on a parked Cairn stack beside a homeworld, the fight frame by frame |
//! | `sentries` | missile sentries at a center firing on an armed raider, rounds and point defense |
//!
//! The fights are the determinism suite's seeded fleets
//! (`tests/determinism.rs`), placed where the mechanism must fire.
//!
//! Run: `cargo run --release --example record_replay -- <out dir> [--quick]`.
//! `--quick` records short, small versions — the fixture the viewer's tests
//! read in CI.
use hyades_engine::galaxy::{FleetSeeding, SeedFleet};
use hyades_engine::log::{LogCategory, LogFilter};
use hyades_engine::prelude::*;
use hyades_engine::replay::{record_run, ReplayConfig};
use hyades_engine::sim::Class;
use std::io::Write;

struct Scenario {
    name: &'static str,
    label: &'static str,
    seats: usize,
    seed: u64,
    planets: usize,
    horizon: f64,
    frame_years: f64,
    /// `(seat, hull, class, role, offset from seat 0's homeworld ly, velocity ly/yr)`.
    fleets: Vec<(usize, HullType, Class, Role, Vec3, Vec3)>,
    spend_kt: f64,
    filter: LogFilter,
}

const STILL: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

fn scenarios(quick: bool) -> Vec<Scenario> {
    let picket = |seat: usize, class: Class, offset: Vec3, velocity: Vec3| {
        (seat, HullType::LimitedContactVehicle, class, Role::Picket, offset, velocity)
    };
    let near = Vec3::new(0.5, 0.0, 0.0);
    let fights = LogFilter::none().with(LogCategory::Combat).with(LogCategory::Vehicles);
    vec![
        Scenario {
            name: "expansion",
            label: "Expansion — three seats founding, mining and hauling",
            seats: 3,
            seed: 7,
            planets: if quick { 150 } else { 300 },
            horizon: if quick { 60.0 } else { 400.0 },
            frame_years: 5.0,
            fleets: Vec::new(),
            spend_kt: 0.0,
            filter: LogFilter::none().with(LogCategory::Vehicles).with(LogCategory::Combat).with(LogCategory::Cards),
        },
        Scenario {
            name: "beams",
            label: "Beams — a Tor stack closing on a parked Cairn stack",
            seats: 2,
            seed: 32,
            planets: 200,
            horizon: if quick { 1.0 } else { 3.0 },
            frame_years: 0.02,
            fleets: vec![
                picket(0, Class::Cairn, near, STILL),
                picket(1, Class::Tor, near.add(Vec3::new(0.03, 0.0, 0.0)), Vec3::new(-0.3, 0.0, 0.0)),
            ],
            spend_kt: 0.4,
            filter: fights,
        },
        Scenario {
            name: "sentries",
            label: "Sentries — missiles from a center on an armed raider",
            seats: 3,
            seed: 42,
            planets: 200,
            horizon: if quick { 1.0 } else { 3.0 },
            frame_years: 0.02,
            fleets: vec![
                (0, HullType::LimitedOffensive, Class::Butte, Role::Sentry, STILL, STILL),
                picket(1, Class::Tor, Vec3::new(0.04, 0.0, 0.0), Vec3::new(-0.2, 0.0, 0.0)),
            ],
            spend_kt: 0.2,
            filter: fights,
        },
    ]
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let out = args.get(1).cloned().unwrap_or_else(|| "web/replays".into());
    let quick = args.iter().any(|a| a == "--quick");
    std::fs::create_dir_all(&out).expect("create the output directory");
    let mut index = String::from("[");
    for (i, sc) in scenarios(quick).iter().enumerate() {
        let started = std::time::Instant::now();
        let mut gcfg = GalaxyConfig::new(sc.seats, sc.seed);
        gcfg.planet_count = sc.planets;
        let bare = Galaxy::generate(gcfg).unwrap();
        let home = bare.planets[bare.homeworlds[0].0 as usize].position;
        let fleets = sc
            .fleets
            .iter()
            .map(|&(seat, hull, class, role, offset, velocity)| SeedFleet {
                seat,
                hull,
                class,
                role,
                position: home.add(offset),
                velocity,
            })
            .collect();
        let seeding = FleetSeeding { spend_kt: sc.spend_kt, known_radius_ly: 0.0, fleets, twin_bill: None };
        let galaxy = Galaxy::generate_with(gcfg, seeding).unwrap();
        let mut cfg = SimConfig::new(sc.seed);
        cfg.horizon_years = sc.horizon;
        let mut sim = Simulation::with_baseline(galaxy.clone(), cfg);
        sim.set_log_filter(sc.filter);
        // A fight opens on its fleets: the circle around their starting
        // places, with room for the closing stack's run in.
        let focus = sc.fleets.first().map(|_| {
            let (mut lo, mut hi) = ([f64::INFINITY; 2], [f64::NEG_INFINITY; 2]);
            for f in &sc.fleets {
                let p = home.add(f.4);
                (lo[0], lo[1], hi[0], hi[1]) = (lo[0].min(p.x), lo[1].min(p.y), hi[0].max(p.x), hi[1].max(p.y));
            }
            let r = 0.5 * (hi[0] - lo[0]).max(hi[1] - lo[1]) + 0.05;
            (0.5 * (lo[0] + hi[0]), 0.5 * (lo[1] + hi[1]), r)
        });
        let rc = ReplayConfig { frame_years: sc.frame_years, max_events: 50_000, label: sc.label.into(), focus };
        let json = record_run(&galaxy, sim, &rc);
        let file = format!("{}.json", sc.name);
        std::fs::write(format!("{out}/{file}"), &json).expect("write the replay");
        println!(
            "{:<10} {:>7.1} kB  {:>5} frames  {:.1} s",
            sc.name,
            json.len() as f64 / 1024.0,
            (sc.horizon / sc.frame_years).floor() as usize + 1,
            started.elapsed().as_secs_f64()
        );
        std::io::stdout().flush().ok();
        if i > 0 {
            index.push(',');
        }
        index.push_str(&format!(
            "{{\"file\":\"{file}\",\"name\":\"{}\",\"label\":\"{}\",\"seats\":{},\"horizon_years\":{},\"bytes\":{}}}",
            sc.name,
            sc.label,
            sc.seats,
            sc.horizon,
            json.len()
        ));
    }
    index.push(']');
    std::fs::write(format!("{out}/index.json"), index).expect("write the index");
}
