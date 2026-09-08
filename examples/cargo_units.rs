//! **T-53 / R-O71, closed: the hold ladder *is* the mass ladder.**
//!
//! `Hyades_mineral_cost_curve.md` §2.6 used to require one step factor
//! `F ∈ [4, 8]` to govern every Band-laddered quantity, cargo capacity named
//! among them, while the shell model derived capacity from `(r − 1)³` and
//! stepped Medium → General by ~106×. This harness existed to print that
//! disagreement as a number rather than an argument.
//!
//! R-MC15 resolved it. The `[4, 8]` window is withdrawn; there are two ladders,
//! **mass** and **mineral cost**, tied by `F_mass = F_cost^(3/2)` — which is
//! the shell model's own exponent, since cost tracks `r²` and the hold tracks
//! `r³`. So the harness now prints the *agreement*, and where it is inexact and
//! why.
//!
//! The distinction that does the work: **the Band rung is the hold**, and
//! `V_reserved` (the role's engine/crew core plus its scaling payload) is
//! deducted after it. So the hold walks the ladder and the usable cargo does
//! not — most visibly at the bottom, where a Limited hull's core eats ~88% of
//! its `Band Empty` hold.
//!
//! Run: `cargo run --release --example cargo_units`
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
        "ratified cost ladder: medium_fleet_size={}, limited_fleet_size={}  (F_cost = {} then {})",
        cfg.medium_fleet_size,
        cfg.limited_fleet_size,
        cfg.limited_fleet_size / cfg.medium_fleet_size,
        cfg.medium_fleet_size
    );

    println!(
        "\n{:<9} {:>9} {:>8} {:>9} {:>9} {:>9} {:>9}",
        "hull", "dry mass", "tau", "hold", "V_res", "cargo", "cargo/dry"
    );
    let mut holds = Vec::new();
    let mut cargoes = Vec::new();
    for (name, h) in hulls {
        let dry = hyades_engine::sim::hull_dry_mass(h, &cfg).kilotons();
        let hold = h.hold_volume(&cfg).hull_units_cubed();
        let res = h.reserved_volume(&cfg).hull_units_cubed();
        let cap = h.cargo_capacity(&cfg).kilotons();
        holds.push((name, hold));
        cargoes.push((name, cap));
        println!(
            "{name:<9} {dry:>9.4} {:>8.5} {hold:>9.4} {res:>9.4} {cap:>9.4} {:>9.2}",
            h.shell_thickness().hull_units(),
            cap / dry
        );
    }

    println!("\nthe hold walks the mass ladder — this is the ratified tie:");
    for (i, w) in holds.windows(2).enumerate() {
        let (a, b) = (&w[0], &w[1]);
        let step = b.1 / a.1;
        let want = MASS_LADDER[i];
        println!(
            "  {:>7} -> {:<8} = {step:>8.2}x   against F_mass[{i}] = {want:.2}   ({:+.2}%)",
            a.0,
            b.0,
            100.0 * (step / want - 1.0)
        );
    }

    println!("\nusable cargo does not, and V_reserved is why:");
    for (i, w) in cargoes.windows(2).enumerate() {
        let (a, b) = (&w[0], &w[1]);
        println!("  {:>7} -> {:<8} = {:>8.2}x   against F_mass[{i}] = {:.2}", a.0, b.0, b.1 / a.1, MASS_LADDER[i]);
    }

    println!("\nwhere the rungs sit, in kilotons:");
    for b in [0.0, 1.0, 2.0] {
        println!("  Band {b:.0} = {:>10.4} kt", Band::new(b).in_kilotons().kilotons());
    }
    println!(
        "\n  a Medium hull's hold is {:.4} kt at cargo_unit_size = {}, which reads as {}",
        holds[1].1 * cfg.cargo_unit_size,
        cfg.cargo_unit_size,
        hyades_engine::units::Kilotons::new(holds[1].1 * cfg.cargo_unit_size).in_bands()
    );
}
