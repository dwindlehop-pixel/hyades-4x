//! **What does a static, per-role capability rating cost, and can its combat
//! beds discriminate anything today?** (T-131, `Hyades_technology_tree.md` §4.)
//!
//! The Technology objective rates every Design by head-to-head competition at
//! equal mineral spend, one test bed per role. Before any of that is built, three
//! questions decide whether it is feasible and what it would measure:
//!
//! 1. **The pool.** Which Designs exist, what one costs, and how many of each an
//!    equal-spend budget buys — the fleet sizes the beds run at.
//! 2. **The cost of one combat match**, at the fleet sizes the budget implies.
//!    `resolve_beam_engagement` sorts every standing enemy for every shooter every
//!    tick, so it is `O(N² log N)` per tick and the budget sets `N`.
//! 3. **Whether starting distance changes a beam fight.** The long-range bed
//!    starts its fleets apart and the short-range bed at point blank; if beams
//!    hit identically at every range the two beds measure the same thing until a
//!    ranged family (missile, torpedo) or beam falloff is built.
//!
//! And one preview of the rating's failure mode: the short-range round robin
//! over every armed Design, printing the score share each side takes. A share of
//! exactly 0 or 1 is **complete separation**, where a Bradley–Terry maximum
//! likelihood rating diverges.
//!
//! `cargo run --release --example capability_probe`
use std::io::Write;
use std::time::Instant;

use hyades_engine::combat::{hull_hp_kj, resolve_beam_engagement, Armed, Combatant, FleetTrajectory, StationKeeping};
use hyades_engine::combat::{CombatConfig, STATION_PERIOD, STATION_RADIUS};
use hyades_engine::math::Vec3;
use hyades_engine::rng::Rng;
use hyades_engine::sim::{design_loadout, hull_dry_mass, Class, HullType, Role, SimConfig};

const HULLS: [(&str, HullType); 10] = [
    ("LSV", HullType::LimitedSystems),
    ("MSV", HullType::MediumSystems),
    ("GSV", HullType::GeneralSystems),
    ("LCV", HullType::LimitedContactVehicle),
    ("LCU", HullType::LimitedContactUnit),
    ("GCV", HullType::GeneralContactVehicle),
    ("GCU", HullType::GeneralContactUnit),
    ("LOU", HullType::LimitedOffensive),
    ("ROU", HullType::RapidOffensive),
    ("GOU", HullType::GeneralOffensive),
];

/// Equal-spend budget, in multiples of one General hull's price.
const BUDGET_GENERALS: f64 = 10.0;

fn fleet_size(hull: HullType, cfg: &SimConfig) -> usize {
    let budget = BUDGET_GENERALS * hull_dry_mass(HullType::GeneralSystems, cfg).kilotons();
    (budget / hull_dry_mass(hull, cfg).kilotons()).round().max(1.0) as usize
}

fn fleet(hull: HullType, n: usize, side: usize, seed: u64, cfg: &SimConfig, combat: &CombatConfig) -> Vec<Armed> {
    let mut rng = Rng::new(seed ^ (side as u64 + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15));
    let loadout = design_loadout(hull, Class::Unnamed, cfg, combat);
    let hp = hull_hp_kj(hull, cfg, combat);
    (0..n)
        .map(|_| Armed {
            ship: Combatant {
                role: Role::Picket,
                hull,
                thrust_factor: 1.0,
                fleet: side,
                station: StationKeeping::draw(&mut rng, STATION_RADIUS, STATION_PERIOD),
                maneuver_velocity: Vec3::ZERO,
                maneuver_start: 0.0,
                maneuver_origin_offset: Vec3::ZERO,
            },
            loadout,
            hp_kj: hp,
        })
        .collect()
}

/// One engagement: `na` hulls of `a` against `nb` of `b`, the fleets
/// `separation_ly` apart and not closing. Returns the survivors per side and the
/// wall time of the resolver call alone.
#[allow(clippy::too_many_arguments)]
fn engage(
    a: HullType,
    na: usize,
    b: HullType,
    nb: usize,
    separation_ly: f64,
    seed: u64,
    cfg: &SimConfig,
    combat: &CombatConfig,
) -> ([usize; 2], f64) {
    let fleets = [
        FleetTrajectory { origin: Vec3::ZERO, velocity: Vec3::ZERO },
        FleetTrajectory { origin: Vec3::new(separation_ly, 0.0, 0.0), velocity: Vec3::ZERO },
    ];
    let sa = fleet(a, na, 0, seed, cfg, combat);
    let sb = fleet(b, nb, 1, seed, cfg, combat);
    let start = Instant::now();
    let alive = resolve_beam_engagement(&fleets, [&sa, &sb], cfg.engagement_horizon_years, cfg.engagement_dt_years);
    let secs = start.elapsed().as_secs_f64();
    let live = |v: &[bool]| v.iter().filter(|&&x| x).count();
    ([live(&alive[0]), live(&alive[1])], secs)
}

/// Side 0's share of the surviving mass; 0.5 when nothing survives on either side.
fn share(surv: [f64; 2]) -> f64 {
    let total = surv[0] + surv[1];
    if total > 0.0 {
        surv[0] / total
    } else {
        0.5
    }
}

fn main() {
    let cfg = SimConfig::new(1);
    let combat = CombatConfig::default();
    let mut out = std::io::stdout();

    println!("== 1. the pool: every Design the engine can build, at an equal spend of {BUDGET_GENERALS} General hulls");
    println!("{:<5} {:>9} {:>6} {:>9} {:>7}", "hull", "dry kt", "beams", "hp kJ", "N");
    for (name, h) in HULLS {
        let l = design_loadout(h, Class::Unnamed, &cfg, &combat);
        println!(
            "{name:<5} {:>9.4} {:>6} {:>9.1} {:>7}",
            hull_dry_mass(h, &cfg).kilotons(),
            l.beams,
            hull_hp_kj(h, &cfg, &combat),
            fleet_size(h, &cfg)
        );
    }

    let lcv = HullType::LimitedContactVehicle;
    println!("\n== 2. cost of one point-blank match, same Design both sides (LCV), by fleet size");
    for n in [10usize, 50, 100, 250, 500] {
        let ([la, lb], secs) = engage(lcv, n, lcv, n, 0.0, 1, &cfg, &combat);
        println!("N = {n:>4}: {secs:>8.3} s   survivors {la} / {lb}");
        out.flush().unwrap();
    }

    println!("\n== 3. does starting distance change a beam fight? LCV x10 against LCV x10, 3 seeds");
    println!("{:>12} {:>14} {:>14}", "separation ly", "mean survivors", "share side 0");
    for sep in [0.0, 1e-4, 3e-4, 1e-3, 3e-3, 1e-2, 1e-1] {
        let mut surv = 0.0;
        let mut sh = 0.0;
        for seed in 1..=3u64 {
            let ([la, lb], _) = engage(lcv, 10, lcv, 10, sep, seed, &cfg, &combat);
            surv += (la + lb) as f64 / 3.0;
            sh += share([la as f64, lb as f64]) / 3.0;
        }
        println!("{sep:>12.0e} {surv:>14.2} {sh:>14.3}");
        out.flush().unwrap();
    }

    println!("\n== 4. short-range round robin over the armed Designs, equal spend, point blank, seed 1");
    println!("   entry = row's share of surviving mass against column; 0 or 1 is complete separation");
    let armed: Vec<(&str, HullType)> =
        HULLS.iter().copied().filter(|&(_, h)| design_loadout(h, Class::Unnamed, &cfg, &combat).is_armed()).collect();
    print!("{:<5}", "");
    for (name, _) in &armed {
        print!(" {name:>6}");
    }
    println!();
    let mut total_secs = 0.0;
    let mut survivors_kt = 0.0;
    for &(rn, rh) in &armed {
        print!("{rn:<5}");
        for &(_, ch) in &armed {
            let ([la, lb], secs) = engage(rh, fleet_size(rh, &cfg), ch, fleet_size(ch, &cfg), 0.0, 1, &cfg, &combat);
            let kt = |h: HullType| hull_dry_mass(h, &cfg).kilotons();
            let surv = [la as f64 * kt(rh), lb as f64 * kt(ch)];
            total_secs += secs;
            survivors_kt += surv[0] + surv[1];
            print!(" {:>6.3}", share(surv));
            out.flush().unwrap();
        }
        println!();
    }
    println!("round robin wall time: {total_secs:.1} s for {} matches", armed.len() * armed.len());
    println!("surviving dry mass summed over every match, both sides: {survivors_kt:.4} kt");
}
