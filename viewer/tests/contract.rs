//! **The seam's contract** (`docs/Hyades_interface.md` §2): what the engine
//! records, the viewer reads — every world, every hull of every frame, with
//! its Design, and the log — and the tactical mode plans every one of them.
//!
//! The only place the two crates meet, and only in a test: the viewer's
//! library does not link the engine.

use hyades_engine::log::{LogCategory, LogFilter};
use hyades_engine::prelude::*;
use hyades_engine::replay::{record_run, ReplayConfig};
use hyades_viewer::camera::Camera;
use hyades_viewer::palette::Palette;
use hyades_viewer::replay::Replay;
use hyades_viewer::tactical::{plan, Op, Scene};

fn recorded() -> (String, Simulation) {
    let mut g = GalaxyConfig::new(3, 7);
    g.planet_count = 150;
    let galaxy = Galaxy::generate(g).unwrap();
    let mut cfg = SimConfig::new(7);
    cfg.horizon_years = 80.0;
    let mut sim = Simulation::with_baseline(galaxy.clone(), cfg);
    sim.set_log_filter(LogFilter::none().with(LogCategory::Vehicles));
    let twin = {
        let mut t = Simulation::with_baseline(galaxy.clone(), cfg);
        t.set_log_filter(LogFilter::none().with(LogCategory::Vehicles));
        t
    };
    let rc = ReplayConfig { frame_years: 10.0, max_events: 100_000, label: "contract".into(), focus: None };
    (record_run(&galaxy, sim, &rc), twin)
}

#[test]
fn the_viewer_reads_every_world_hull_and_event_the_engine_recorded() {
    let (json, mut twin) = recorded();
    let r = Replay::from_json(&json).expect("the viewer reads the engine's replay");
    assert_eq!(r.frames.len(), 9);
    let mut hulls = 0;
    for (k, f) in r.frames.iter().enumerate() {
        let t = k as f64 * 10.0;
        while twin.next_event_time().is_some_and(|n| n <= t) {
            twin.step();
        }
        let snap = twin.snapshot_at(t);
        assert_eq!(f.t, t);
        assert_eq!(f.rows.len(), snap.vehicles.len(), "frame {k}: every hull");
        assert_eq!(r.planets.len(), snap.planets.len(), "every world");
        for v in &snap.vehicles {
            let row = f.rows.iter().find(|row| row.id == v.id).expect("each hull by id");
            let hull = r.hull_table[&v.id];
            assert_eq!(r.designs[hull.design], v.design, "its Design");
            assert_eq!(r.hulls[hull.hull], v.hull, "its hull");
            assert_eq!(hull.owner, v.owner as usize, "its seat");
            assert_eq!(r.kinds[row.kind], format!("{:?}", v.kind), "its role");
            assert!((row.pos[0] - v.position.x).abs() <= 0.005, "its position");
        }
        for (i, p) in snap.planets.iter().enumerate() {
            assert_eq!(f.owner[i], p.owner.map(|o| o as usize), "frame {k}: world {i}'s owner");
        }
        hulls += f.rows.len();
    }
    assert!(hulls > 50, "the run put hulls in the frames: {hulls}");
    assert!(!r.events.is_empty() && r.events.windows(2).all(|w| w[0].t <= w[1].t));
}

#[test]
fn the_tactical_mode_plans_every_recorded_entity() {
    let (json, _) = recorded();
    let r = Replay::from_json(&json).unwrap();
    let mut camera = Camera::new(960.0, 600.0);
    camera.fit(r.planets.iter().map(|p| p.pos), 20.0);
    let palette = Palette::default();
    for t in [0.0, 35.0, 80.0] {
        let view = r.view_at(t);
        let s = Scene { replay: &r, view: &view, camera: &camera, palette: &palette, selected: None };
        let ops = plan(&s);
        let worlds = ops.iter().filter(|o| matches!(o, Op::World { .. })).count();
        let drawn: u32 = ops.iter().map(|o| if let Op::Hull { count, .. } = o { *count } else { 0 }).sum();
        assert_eq!(worlds, r.planets.len(), "t {t}");
        assert_eq!(drawn as usize, view.hulls.len(), "t {t}: every hull drawn or counted in a stack");
    }
}
