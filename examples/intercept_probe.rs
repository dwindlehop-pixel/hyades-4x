//! **Can a picket ever win the race?** (T-115) — the feasibility of
//! `Doctrine::picket_intercepts`, priced before it is believed.
//!
//! The criterion the engine applies is
//! `depart + |origin − station| + flight(station → world) < colony_arrival`,
//! and the first term is light over the *same* ground the colony ship crosses.
//! So the question is whether the flight-time overhead a colony ship pays over
//! light is bigger than the hop a picket has to make — which is geometry, not
//! tuning, and one run answers it.
//!
//! **The first run of this harness answered it wrong, and the reason is worth
//! keeping.** It priced the colony ship at `civilian_accel_g · G` because that
//! is what `spawn_courier` flew it at — the R-WAR9 defect, found and written up
//! in the same session and then used anyway. An empty hull's overhead saturates
//! at **1.92 years** and a laden one's at **~8.1**, so the first answer was
//! *"fewer than a tenth of worlds have a neighbor in range"* and the true one
//! is about half. **A probe inherits every assumption of the code it measures**;
//! when you have just found one of them to be wrong, re-derive the probe before
//! trusting it.

use hyades_engine::autopilot::{Autopilot, BaselineAutopilot, Doctrine};
use hyades_engine::galaxy::{Galaxy, GalaxyConfig};
use hyades_engine::math::{self, G};
use hyades_engine::sim::{hull_dry_mass, HullType, SimConfig, Simulation};
use std::io::Write;

fn main() {
    let seats = 3;
    let galaxy = Galaxy::generate(GalaxyConfig::new(seats, 1)).unwrap();
    let autopilots: Vec<Box<dyn Autopilot>> =
        (0..seats).map(|_| Box::new(BaselineAutopilot::new(Doctrine::default())) as Box<_>).collect();
    let cfg = SimConfig::new(1);
    let pts: Vec<_> = galaxy.planets.iter().map(|p| p.position).collect();
    let _sim = Simulation::new(galaxy, cfg, autopilots);

    // The acceleration a colony ship actually flies at, and the one a picket
    // does. Both read off the hull rather than off a flat constant (R-WAR9).
    let dry = hull_dry_mass(HullType::MediumSystems, &cfg).kilotons();
    let drive = HullType::MediumSystems.drive_mass(&cfg).kilotons();
    let hold = HullType::MediumSystems.colony_seed_capacity(&cfg).kilotons();
    let a_laden = G * (cfg.drive_specific_thrust * drive) / (dry + hold);
    let p_dry = hull_dry_mass(HullType::LimitedOffensive, &cfg).kilotons();
    let p_drive = HullType::LimitedOffensive.drive_mass(&cfg).kilotons();
    let a_picket = G * (cfg.drive_specific_thrust * p_drive) / p_dry;
    let a_empty = G * (cfg.drive_specific_thrust * drive) / dry;
    println!("empty Medium colonizer = {a_empty:.4} ly/yr^2");
    println!("laden Medium colonizer = {a_laden:.4} ly/yr^2");
    println!("empty LOU picket       = {a_picket:.4} ly/yr^2");

    // How much longer than light does the colony ship take, as a function of
    // range? That overhead *is* the interceptor's window.
    println!("\n{:>8}  {:>10}  {:>10}  {:>10}  {:>12}", "range ly", "light yr", "empty yr", "laden yr", "window yr");
    for d in [1.0, 2.0, 5.0, 10.0, 25.0, 50.0, 100.0, 200.0] {
        let e = math::ship_travel_years(d, a_empty);
        let l = math::ship_travel_years(d, a_laden);
        println!("{d:>8.1}  {d:>10.3}  {e:>10.3}  {l:>10.3}  {:>12.3}", l - d);
    }
    let _ = std::io::stdout().flush();

    // And how far does a picket get inside a window of that size?
    println!("\n{:>10}  {:>14}", "window yr", "picket reach ly");
    for w in [2.0, 4.0, 6.0, 8.0, 8.14] {
        let (mut lo, mut hi) = (0.0f64, 10_000.0f64);
        for _ in 0..80 {
            let mid = 0.5 * (lo + hi);
            if math::ship_travel_years(mid, a_picket) < w {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        println!("{w:>10.2}  {lo:>14.3}");
    }
    let _ = std::io::stdout().flush();

    // **How much wider is meeting the ship than beating it to the world?**
    // (R-WAR10.) The shipped criterion races the colony ship to its
    // destination. Meeting it anywhere on its path is a strictly larger set,
    // and this prices the difference on a straight 25 ly voyage: a station
    // offset perpendicular to the track at its midpoint.
    println!("\n25 ly voyage, station offset perpendicular to the track:");
    let d_voyage = 25.0_f64;
    let t_laden = math::ship_travel_years(d_voyage, a_laden);
    let reach_by = |window: f64| {
        let (mut lo, mut hi) = (0.0f64, 10_000.0f64);
        for _ in 0..80 {
            let mid = 0.5 * (lo + hi);
            if math::ship_travel_years(mid, a_picket) < window {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        lo
    };
    // Racing to the destination: the station sits beside the *destination*, and
    // hears the launch a voyage-length away.
    let to_dest = reach_by(t_laden - d_voyage);
    // Meeting on the path: the station sits beside the *midpoint*, hears the
    // launch half a voyage away, and only has to reach the track.
    let mut best_path = 0.0f64;
    for k in 1..=200 {
        let frac = k as f64 / 200.0;
        let along = d_voyage * frac;
        // Where the ship is when it has covered `along`, and when.
        let (mut lo, mut hi) = (0.0f64, t_laden);
        for _ in 0..60 {
            let mid = 0.5 * (lo + hi);
            if math::flight_distance(a_laden, 0.0, t_laden, d_voyage).min(d_voyage) >= 0.0
                && math::position_along(
                    math::Vec3::new(0.0, 0.0, 0.0),
                    math::Vec3::new(d_voyage, 0.0, 0.0),
                    0.0,
                    t_laden,
                    a_laden,
                    mid,
                )
                .norm()
                    < along
            {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        let t_there = hi;
        // A station offset `x` from that point, which heard the launch at
        // `sqrt(along^2 + x^2)`… solved by bisection on `x`.
        let (mut xl, mut xh) = (0.0f64, 500.0f64);
        for _ in 0..60 {
            let x = 0.5 * (xl + xh);
            let t_see = (along * along + x * x).sqrt();
            let ok = t_see + math::ship_travel_years(x, a_picket) <= t_there;
            if ok {
                xl = x;
            } else {
                xh = x;
            }
        }
        if xl > best_path {
            best_path = xl;
        }
        if k % 25 == 0 {
            println!("  at {:>5.1}% of the track ({along:>5.2} ly): offset <= {xl:.3} ly", frac * 100.0);
        }
    }
    println!("  colony ship takes {t_laden:.3} yr over {d_voyage:.1} ly");
    println!("  best offset anywhere on the track: {best_path:.3} ly");
    println!("  (a picket flying {:.3} yr covers {:.3} ly)", t_laden - d_voyage, to_dest);
    let _ = std::io::stdout().flush();

    // What does the engine's own field look like? Nearest-neighbor spacing is
    // the hop a picket standing next door would have to make.
    let n = pts.len().min(400);
    let mut nn: Vec<f64> = Vec::new();
    for i in 0..n {
        let mut best = f64::INFINITY;
        for j in 0..pts.len() {
            if i == j {
                continue;
            }
            let d = pts[i].distance(pts[j]);
            if d < best {
                best = d;
            }
        }
        nn.push(best);
    }
    nn.sort_by(|a, b| a.partial_cmp(b).unwrap());
    println!("\n{} planets; nearest-neighbor spacing over {n} sampled:", pts.len());
    for (label, q) in [("min", 0.0), ("p10", 0.10), ("median", 0.50), ("p90", 0.90), ("max", 1.0)] {
        let i = ((nn.len() - 1) as f64 * q) as usize;
        println!("  {label:>6}: {:.3} ly  (hop costs {:.3} yr)", nn[i], math::ship_travel_years(nn[i], a_picket));
    }
    let _ = std::io::stdout().flush();
}
