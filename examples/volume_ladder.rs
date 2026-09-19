//! **Does fleet-years in *volume* express design law #3 where fleet-years in
//! mass does not?** (R-PROD5.)
//!
//! Law #3: cost is surface area, value is volume, so consolidation wins under
//! geometry alone. Since R-O57 dry mass **is** mineral cost, so fleet-years in
//! mass is "minerals committed to hulls, integrated" — which is *indifferent*
//! between one General hull and ten Mediums at the same spend. A metric that
//! cannot tell those apart cannot express a law about their ratio.
//!
//! This prints, per hull, the enclosed volume `r³` and the **volume bought per
//! kilotonne spent**. If that column rises with size, volume-years rewards
//! consolidation and mass-years does not.
use hyades_engine::prelude::*;
use hyades_engine::sim::{hull_dry_mass, HullType};

fn main() {
    let cfg = SimConfig::new(1);
    let hulls = [
        ("LSV", HullType::LimitedSystems),
        ("MSV", HullType::MediumSystems),
        ("GSV", HullType::GeneralSystems),
        ("LCV", HullType::LimitedContactVehicle),
        ("GCV", HullType::GeneralContactVehicle),
        ("LOU", HullType::LimitedOffensive),
        ("ROU", HullType::RapidOffensive),
        ("GOU", HullType::GeneralOffensive),
    ];
    println!(
        "{:<5} {:>10} {:>10} {:>12} {:>12} {:>12} {:>10}",
        "hull", "dry kt", "radius", "vol r^3", "hold", "shell", "vol per kt"
    );
    for (name, h) in hulls {
        let dry = hull_dry_mass(h, &cfg).kilotons();
        let r = h.hull_radius(&cfg).hull_units();
        let vol = h.hull_radius(&cfg).cubed().hull_units_cubed();
        let hold = h.hold_volume(&cfg).hull_units_cubed();
        let shell = h.shell_volume(&cfg).hull_units_cubed();
        println!("{name:<5} {dry:>10.5} {r:>10.4} {vol:>12.5} {hold:>12.5} {shell:>12.5} {:>10.4}", vol / dry);
    }

    // The law-#3 test: one General against an equal-*cost* fleet of Mediums.
    println!();
    for (big, small, label) in [
        (HullType::GeneralSystems, HullType::MediumSystems, "GSV vs MSV"),
        (HullType::GeneralOffensive, HullType::RapidOffensive, "GOU vs ROU"),
        (HullType::MediumSystems, HullType::LimitedSystems, "MSV vs LSV"),
    ] {
        let cb = hull_dry_mass(big, &cfg).kilotons();
        let cs = hull_dry_mass(small, &cfg).kilotons();
        let n = cb / cs; // how many small hulls the big one's minerals buy
        let vb = big.hull_radius(&cfg).cubed().hull_units_cubed();
        let vs = small.hull_radius(&cfg).cubed().hull_units_cubed();
        let hb = big.hold_volume(&cfg).hull_units_cubed();
        let hs = small.hold_volume(&cfg).hull_units_cubed();
        println!(
            "{label}: {n:.2} small per big | r^3 {vb:.4} vs {:.4} -> x{:.3} | hold {hb:.4} vs {:.4} -> x{:.3} | mass x1.000 by construction",
            n * vs,
            vb / (n * vs),
            n * hs,
            hb / (n * hs)
        );
    }
}
