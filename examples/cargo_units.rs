//! **T-53: is the cargo ladder on the Band ladder, or on the shell's?**
//!
//! `Hyades_mineral_cost_curve.md` §2.6 requires the *same* step factor
//! `F ∈ [4, 8]` to govern every Band-laddered quantity, and names **cargo
//! capacity** in that list. `HullType::cargo_capacity` instead derives capacity
//! from shell geometry, `(r − 1)³`. Those are different ladders; this prints
//! both so the disagreement is a number rather than an argument.
use hyades_engine::prelude::*;
use hyades_engine::units::{Band, Measure, MASS_LADDER};

fn main() {
    let cfg = SimConfig::new(1);
    let hulls = [
        ("Limited", HullType::LimitedSystems),
        ("Medium", HullType::MediumSystems),
        ("General", HullType::GeneralSystems),
    ];
    println!(
        "shipped defaults: medium_fleet_size={}, limited_fleet_size={}",
        cfg.medium_fleet_size, cfg.limited_fleet_size
    );
    println!("\n{:<9} {:>10} {:>10} {:>12}", "hull", "dry mass", "capacity", "cap/dry");
    let mut caps = Vec::new();
    for (name, h) in hulls {
        let dry = hyades_engine::sim::hull_dry_mass(h, &cfg).kilotons();
        let cap = h.cargo_capacity(&cfg).kilotons();
        caps.push((name, cap));
        println!("{name:<9} {dry:>10.4} {cap:>10.4} {:>12.2}", cap / dry);
    }
    println!("\nstep ratios (what §2.6 constrains to [4, 8]):");
    for w in caps.windows(2) {
        let (a, b) = (&w[0], &w[1]);
        let r = if a.1 > 0.0 { b.1 / a.1 } else { f64::INFINITY };
        let ok = (4.0..=8.0).contains(&r);
        println!("  {:>7} -> {:<8} = {:>10.2}x   {}", a.0, b.0, r, if ok { "in [4,8]" } else { "**OUTSIDE [4,8]**" });
    }
    println!("\nwhat the Band ladder would give at F(I→II) = {}:", MASS_LADDER[1]);
    for b in [0.0, 1.0, 2.0] {
        println!("  Band {b:.0} = {:>10.4} kt", Band::new(b).in_kilotons().kilotons());
    }
    println!(
        "\n  a Medium hull's hold is cargo_unit_size = {} kt, which reads as {}",
        cfg.cargo_unit_size,
        hyades_engine::units::Kilotons::new(cfg.cargo_unit_size).in_bands()
    );
}
