//! **What does a static, per-role capability rating cost, and can its combat
//! beds discriminate anything?** (T-131, `Hyades_technology_tree.md` §4; the
//! damage model it measures is T-132, `Hyades_warfare_tree.md` §8.18.)
//!
//! The Technology objective rates every Design by head-to-head competition at
//! equal mineral spend, one test bed per role. Five questions decide whether
//! the combat beds are feasible and what they would measure:
//!
//! 1. **The pool.** Which Designs exist, what one costs, how much structure it
//!    has, how long one beam mount takes to wreck it, and how many an equal-spend
//!    budget buys — the fleet sizes the beds run at.
//! 2. **The cost of one combat match**, at the fleet sizes the budget implies.
//!    `resolve_beam_engagement` sorts every standing enemy for every shooter every
//!    tick, so it is `O(N² log N)` per tick and the budget sets `N`.
//! 3. **Whether starting distance changes a beam fight.** The long-range bed
//!    starts its fleets apart and the short-range bed at point blank.
//! 4. **The short-range round robin** over every armed Design: the share of
//!    surviving mass each side takes, and how many ticks each fight lasts. A
//!    share of exactly 0 or 1 is complete separation, and a fight a few ticks
//!    long cannot resolve a small Design difference.
//! 5. **Design law #2 in the engine**: how many equal Designs of the smaller
//!    hull one larger hull beats (target 6–45 ROUs per GOU).
//!
//! `capability_probe [beam_mw] [engagement_horizon_years]` — both default to the
//! shipped values, so the probe can price a candidate before it ships.
use std::io::Write;
use std::time::Instant;

use hyades_engine::combat::StationKeeping;
use hyades_engine::combat::{hull_structure_kj, resolve_beam_engagement, Armed, Combatant, FleetTrajectory};
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
    let structure = hull_structure_kj(hull, cfg, combat);
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
            structure_kj: structure,
        })
        .collect()
}

/// One engagement: `na` hulls of `a` against `nb` of `b`, the fleets
/// `separation_ly` apart and not closing. Returns the survivors per side, the
/// fight's length in ticks, and the wall time of the resolver call alone.
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
) -> ([usize; 2], f64, f64) {
    let fleets = [
        FleetTrajectory { origin: Vec3::ZERO, velocity: Vec3::ZERO },
        FleetTrajectory { origin: Vec3::new(separation_ly, 0.0, 0.0), velocity: Vec3::ZERO },
    ];
    let sa = fleet(a, na, 0, seed, cfg, combat);
    let sb = fleet(b, nb, 1, seed, cfg, combat);
    let start = Instant::now();
    let out = resolve_beam_engagement(&fleets, [&sa, &sb], cfg.engagement_horizon_years, cfg.engagement_dt_years);
    let secs = start.elapsed().as_secs_f64();
    let live = |v: &[bool]| v.iter().filter(|&&x| x).count();
    ([live(&out.alive[0]), live(&out.alive[1])], out.duration_years / cfg.engagement_dt_years, secs)
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
    let mut cfg = SimConfig::new(1);
    let mut combat = CombatConfig::default();
    if let Some(mw) = std::env::args().nth(1).and_then(|v| v.parse().ok()) {
        combat.beam_power_mw = mw;
    }
    if let Some(h) = std::env::args().nth(2).and_then(|v| v.parse().ok()) {
        cfg.engagement_horizon_years = h;
    }
    let combat = combat;
    let mut out = std::io::stdout();
    let dt = cfg.engagement_dt_years;
    let days = |yr: f64| yr * 365.25;
    let power = combat.beam_power_kj_per_year();
    println!(
        "beam {} MW, structure {:e} kJ per hull unit^3, engagement {} yr = {:.0} ticks of {:.2} days",
        combat.beam_power_mw,
        combat.structure_kj_per_hull_unit3,
        cfg.engagement_horizon_years,
        cfg.engagement_horizon_years / dt,
        days(dt)
    );

    println!(
        "\n== 1. the pool: every Design the engine can build, at an equal spend of {BUDGET_GENERALS} General hulls"
    );
    println!(
        "{:<5} {:>8} {:>8} {:>6} {:>12} {:>16} {:>6}",
        "hull", "dry kt", "r^3", "beams", "structure TJ", "1 mount kills in", "N"
    );
    for (name, h) in HULLS {
        let l = design_loadout(h, Class::Unnamed, &cfg, &combat);
        let structure = hull_structure_kj(h, &cfg, &combat);
        println!(
            "{name:<5} {:>8.4} {:>8.4} {:>6} {:>12.1} {:>11.1} days {:>6}",
            hull_dry_mass(h, &cfg).kilotons(),
            h.hull_volume(&cfg).hull_units_cubed(),
            l.beams,
            structure / 1e9,
            days(structure / power),
            fleet_size(h, &cfg)
        );
    }

    let lcv = HullType::LimitedContactVehicle;
    println!("\n== 2. cost of one point-blank match, same Design both sides (LCV), by fleet size");
    for n in [10usize, 50, 100, 250, 500] {
        let ([la, lb], ticks, secs) = engage(lcv, n, lcv, n, 0.0, 1, &cfg, &combat);
        println!("N = {n:>4}: {secs:>8.3} s   {ticks:>6.0} ticks   survivors {la} / {lb}");
        out.flush().unwrap();
    }

    println!("\n== 3. does starting distance change a beam fight? LCV x10 against LCV x10, 3 seeds");
    println!("{:>12} {:>14} {:>14} {:>12}", "separation ly", "mean survivors", "share side 0", "mean ticks");
    for sep in [0.0, 1e-4, 3e-4, 1e-3, 3e-3, 1e-2, 1e-1] {
        let (mut surv, mut sh, mut tk) = (0.0, 0.0, 0.0);
        for seed in 1..=3u64 {
            let ([la, lb], ticks, _) = engage(lcv, 10, lcv, 10, sep, seed, &cfg, &combat);
            surv += (la + lb) as f64 / 3.0;
            sh += share([la as f64, lb as f64]) / 3.0;
            tk += ticks / 3.0;
        }
        println!("{sep:>12.0e} {surv:>14.2} {sh:>14.3} {tk:>12.0}");
        out.flush().unwrap();
    }

    println!("\n== 4. short-range round robin over the armed Designs, equal spend, point blank, seed 1");
    println!("   entry = row's share of surviving mass against column / fight length in ticks");
    let armed: Vec<(&str, HullType)> =
        HULLS.iter().copied().filter(|&(_, h)| design_loadout(h, Class::Unnamed, &cfg, &combat).is_armed()).collect();
    print!("{:<5}", "");
    for (name, _) in &armed {
        print!(" {name:>12}");
    }
    println!();
    let (mut total_secs, mut survivors_kt) = (0.0, 0.0);
    let (mut shortest, mut longest) = (f64::INFINITY, 0.0f64);
    for &(rn, rh) in &armed {
        print!("{rn:<5}");
        for &(_, ch) in &armed {
            let ([la, lb], ticks, secs) =
                engage(rh, fleet_size(rh, &cfg), ch, fleet_size(ch, &cfg), 0.0, 1, &cfg, &combat);
            let kt = |h: HullType| hull_dry_mass(h, &cfg).kilotons();
            let surv = [la as f64 * kt(rh), lb as f64 * kt(ch)];
            total_secs += secs;
            survivors_kt += surv[0] + surv[1];
            shortest = shortest.min(ticks);
            longest = longest.max(ticks);
            print!(" {:>6.3}/{ticks:>5.0}", share(surv));
            out.flush().unwrap();
        }
        println!();
    }
    println!("round robin wall time: {total_secs:.1} s for {} matches", armed.len() * armed.len());
    println!("surviving dry mass summed over every match, both sides: {survivors_kt:.4} kt");
    println!("fight length: {shortest:.0} to {longest:.0} ticks");

    println!("\n== 5. design law #2: one large hull against N of a smaller one, point blank, seeds 1-3");
    for (big, small, ns) in [
        (HullType::GeneralOffensive, HullType::RapidOffensive, [5usize, 10, 20, 30, 40, 50, 70]),
        (HullType::RapidOffensive, HullType::LimitedOffensive, [2usize, 4, 6, 8, 10, 14, 20]),
    ] {
        print!("{big:?} vs N {small:?}:");
        for n in ns {
            let wins = (1..=3u64).filter(|&seed| engage(big, 1, small, n, 0.0, seed, &cfg, &combat).0[0] == 1).count();
            print!("  N={n} {wins}/3");
            out.flush().unwrap();
        }
        println!();
    }
}
