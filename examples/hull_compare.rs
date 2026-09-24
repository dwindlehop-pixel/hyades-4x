//! **What a GCV is, against the GSV it would replace** (T-116) — the ladder
//! read out before a card is written on top of it.
use hyades_engine::math::G;
use hyades_engine::sim::{hull_dry_mass, HullType, SimConfig};

fn main() {
    let cfg = SimConfig::new(1);
    let dsp = cfg.drive_specific_thrust;
    println!(
        "{:>24}  {:>9}  {:>9}  {:>9}  {:>9}  {:>9}  {:>8}",
        "hull", "cost kt", "hold kt", "seed kt", "a_empty", "a_laden", "kt/kt"
    );
    for h in
        [HullType::MediumSystems, HullType::GeneralSystems, HullType::GeneralContactVehicle, HullType::LimitedOffensive]
    {
        let dry = hull_dry_mass(h, &cfg).kilotons();
        let drive = h.drive_mass(&cfg).kilotons();
        let hold = h.cargo_capacity(&cfg).kilotons();
        let seed = h.colony_seed_capacity(&cfg).kilotons();
        let ae = cfg.civilian_accel_g * G * (dsp * drive) / dry;
        let al = cfg.civilian_accel_g * G * (dsp * drive) / (dry + seed);
        println!("{h:>24?}  {dry:>9.4}  {hold:>9.4}  {seed:>9.4}  {ae:>9.4}  {al:>9.5}  {:>8.3}", seed / dry);
    }
}
