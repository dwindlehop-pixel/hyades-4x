//! **R-MC16 / R-O65: what a volume-proportional drive does to the hull ladder.**
//!
//! The question this answers, asked directly: *can a laden GSV be made
//! competitive with an equal-cost fleet of MSVs in both throughput and
//! turnaround?* Today it is not — it wins on throughput and loses on
//! turnaround, because civilian motion prices thrust as `civilian_accel_g ×
//! dry_mass` and dry mass is the **shell** (`r²`), while the load it has to
//! push is the **hold** (`r³`). So laden acceleration falls as `1/r` by
//! construction and the biggest hull is the most sluggish one.
//!
//! R-MC16 already ratified the fix and named it a *capacity*: thrust scales
//! with volume because ENG slots do, and realized thrust is a **Design**
//! quantity drawn from the hull's usable volume and paid for in minerals. This
//! harness prices that law before any of it is wired in.
//!
//! Three quantities, matching the three components the drive needs:
//!
//! 1. **Impulse density `J`** — thrust per unit of drive volume, in `kt·g` per
//!    hull-unit³. One constant, the same for every hull: *a unit of engine
//!    provides a proportionate unit of acceleration.*
//! 2. **Engine fraction `φ`** — the share of a hull's usable interior a Design
//!    gives to drive instead of cargo. This is "dedicate slots to engines," and
//!    it is a real trade: volume spent on drive is volume not spent on hold.
//! 3. **Shell thickness `τ`** — dry mass *is* the shell (R-O57), so "a Design
//!    variation that reduces dry mass" is a thinner skin, which is exactly the
//!    axis §2.3 already gives to Design level and the Supers gate.
//!
//! **Mass is conserved (design law #11).** An engine is built of minerals, so
//! drive volume carries drive mass at the same density cargo does — a kiloton is
//! a kiloton — and it is charged to the hull's price, because cost and dry mass
//! are one number. That is what makes `φ` a trade rather than a free upgrade:
//! it buys thrust and it costs cargo, mass and minerals at once.
//!
//! Run: `cargo run --release --example drive_probe`
use hyades_engine::math;
use hyades_engine::prelude::*;

/// Representative one-way freight distance, back-solved from the measured
/// 50.47-yr laden leg in `examples/freight_gap` (seed 1, 800 yr, one stop).
const LEG_LY: f64 = 35.79;
/// Measured hold fill on the same run: 0.813 kt into a 0.921 kt hold.
const FILL: f64 = 0.8828;

struct Ship {
    cost: f64,
    cargo: f64,
    a_laden: f64,
    a_empty: f64,
    round_trip: f64,
}

/// One Design, priced. `phi` is the share of usable interior given to drive,
/// and `k` is the **specific thrust** — `kt·g` of thrust per kilotonne of drive
/// mounted, which is the one constant the whole law needs.
///
/// **Volume is the ceiling, mass is the thrust** (R-MC16, verbatim: *"Volume
/// sets the ENG-slot ceiling a hull can mount; realized thrust is a Design
/// quantity drawn from `b_role · V` and paid for in minerals"*). The first cut
/// of this harness made thrust proportional to drive *volume* instead, which
/// reads the same until the mass is conserved: at bulk-cargo density a GSV's
/// drive massed **3.15 kt against a 1.00 kt shell**, so the ship was three
/// quarters engine and its throughput per mineral fell by 73%. Thrust per unit
/// of drive *mass* is the quantity an engineer would quote, and it is the one
/// that does not blow up.
fn design(h: HullType, cfg: &SimConfig, k: f64, phi: f64) -> Ship {
    let shell = hyades_engine::sim::hull_dry_mass(h, cfg).kilotons();
    // Usable interior, in hull-units³: the hold less the role's reserved core.
    let usable = h.cargo_capacity(cfg).kilotons() / cfg.cargo_unit_size;
    let (v_eng, v_hold) = (usable * phi, usable * (1.0 - phi));
    // A kiloton is a kiloton: drive occupies volume at the density everything
    // else does, and under R-O57 that mass is also its price. This is what
    // makes `phi` a trade — drive bought is hold not bought.
    let m_eng = v_eng * cfg.cargo_unit_size;
    let cargo = v_hold * cfg.cargo_unit_size * FILL;
    let dry = shell + m_eng;
    let thrust = k * m_eng;
    let (a_laden, a_empty) = (thrust / (dry + cargo), thrust / dry);
    let g = math::G;
    Ship {
        cost: dry,
        cargo,
        a_laden,
        a_empty,
        round_trip: math::ship_travel_years(LEG_LY, a_laden * g) + math::ship_travel_years(LEG_LY, a_empty * g),
    }
}

/// Today's law, for the baseline row: thrust is `civilian_accel_g × dry_mass`,
/// so acceleration is `dry / (dry + cargo)` and the drive is free and massless.
fn today(h: HullType, cfg: &SimConfig) -> Ship {
    let dry = hyades_engine::sim::hull_dry_mass(h, cfg).kilotons();
    let cargo = h.cargo_capacity(cfg).kilotons() * FILL;
    let (a_laden, a_empty) = (cfg.civilian_accel_g * dry / (dry + cargo), cfg.civilian_accel_g);
    let g = math::G;
    Ship {
        cost: dry,
        cargo,
        a_laden,
        a_empty,
        round_trip: math::ship_travel_years(LEG_LY, a_laden * g) + math::ship_travel_years(LEG_LY, a_empty * g),
    }
}

fn row(name: &str, s: &Ship) {
    // Throughput per mineral is the quantity design law #3 is about: kilotons
    // delivered per year, per kilotonne of hull bought.
    let tput = s.cargo / s.round_trip;
    println!(
        "{name:<22} {:>8.4} {:>9.3} {:>9.4} {:>9.4} {:>9.2} {:>10.5} {:>10.5}",
        s.cost,
        s.cargo,
        s.a_laden,
        s.a_empty,
        s.round_trip,
        tput,
        tput / s.cost
    );
}

fn head(title: &str) {
    println!("\n{title}");
    println!(
        "{:<22} {:>8} {:>9} {:>9} {:>9} {:>9} {:>10} {:>10}",
        "design", "cost", "cargo", "a_laden", "a_empty", "rt (yr)", "kt/yr", "kt/yr/kt$"
    );
}

fn main() {
    let cfg = SimConfig::new(1);
    let hulls =
        [("LSV", HullType::LimitedSystems), ("MSV", HullType::MediumSystems), ("GSV", HullType::GeneralSystems)];

    head("today — thrust = civilian_accel_g x dry_mass (shell, r^2); the drive is free and massless");
    for (n, h) in hulls {
        row(n, &today(h, &cfg));
    }

    // **Calibrate the specific thrust against the MSV's turnaround, not against
    // a taste.** The Medium hull is what freight actually flies, so pinning its
    // laden acceleration makes every other row a comparison rather than a
    // rescaling — the R-O88 precedent: re-denominate against the operating
    // point, then read the change off what was not pinned.
    let base_msv = today(HullType::MediumSystems, &cfg);
    let base_gsv = today(HullType::GeneralSystems, &cfg);
    let solve_k = |phi: f64| {
        let (mut lo, mut hi) = (1e-6, 1e9);
        for _ in 0..300 {
            let mid = 0.5 * (lo + hi);
            if design(HullType::MediumSystems, &cfg, mid, phi).a_laden < base_msv.a_laden {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        0.5 * (lo + hi)
    };
    for phi in [0.02, 0.05, 0.10, 0.20] {
        let k = solve_k(phi);
        head(&format!("thrust = k x m_drive, phi = {phi:.2} of usable interior, k = {k:.3} (MSV turnaround pinned)"));
        for (n, h) in hulls {
            row(n, &design(h, &cfg, k, phi));
        }
    }

    // **The question as asked, as three ratios.** "Competitive in throughput and
    // turnaround" is a comparison between a General hull and the fleet of
    // Mediums the same minerals buy, so every column is GSV ÷ MSV and `1.00` is
    // parity. Today's row is the thing to beat on turnaround and to not lose on
    // throughput.
    println!("\nGSV against an equal-cost MSV fleet  (>1 favours the General hull)");
    println!("{:>7} {:>9} {:>12} {:>12} {:>12} {:>12}", "phi", "k", "a_laden", "round trip", "kt/yr/kt$", "cost x");
    println!(
        "{:>7} {:>9} {:>12.3} {:>12.3} {:>12.3} {:>12.3}",
        "today",
        "-",
        base_gsv.a_laden / base_msv.a_laden,
        base_gsv.round_trip / base_msv.round_trip,
        (base_gsv.cargo / base_gsv.round_trip / base_gsv.cost) / (base_msv.cargo / base_msv.round_trip / base_msv.cost),
        1.0
    );
    for i in 1..=20 {
        let phi = i as f64 * 0.01;
        let k = solve_k(phi);
        let (g, m) = (design(HullType::GeneralSystems, &cfg, k, phi), design(HullType::MediumSystems, &cfg, k, phi));
        println!(
            "{phi:>7.2} {k:>9.2} {:>12.3} {:>12.3} {:>12.3} {:>12.3}",
            g.a_laden / m.a_laden,
            g.round_trip / m.round_trip,
            (g.cargo / g.round_trip / g.cost) / (m.cargo / m.round_trip / m.cost),
            m.cost / base_msv.cost
        );
    }
}
