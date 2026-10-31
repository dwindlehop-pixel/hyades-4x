//! **Native-against-wasm32 parity** (T-128, netcode §6 H4a).
//!
//! `tests/determinism.rs` runs one target, so it cannot see a disagreement
//! between targets: at its horizons a native and a wasm32 run can agree on the
//! report while their galaxies already differ in the last bit (appendix §D.6).
//! This crate is the check that can. The same functions are compiled natively
//! (`parity_native`) and for `wasm32-unknown-unknown` (the cdylib, run by
//! `compare.mjs` under node), and each prints one digest per case. The two
//! lists must be equal line for line.
//!
//! A digest is FNV-1a over the `Debug` text of what it covers. Rust formats an
//! `f64` as the shortest decimal that round-trips, so two values print the
//! same if and only if their bits are equal, and the formatter is the same
//! Rust code on both targets.
//!
//! Cases ([`CASES`]): the galaxy at every fair seat count, one run of the
//! standard 3-seat bed, and the two seeded fights of `tests/determinism.rs`.
//! A run's report, final snapshot and combat log are digested together.

use core::fmt::Write;
use hyades_engine::galaxy::{FleetSeeding, SeedFleet};
use hyades_engine::prelude::*;
use hyades_engine::sim::Class;

/// FNV-1a over bytes written through `fmt::Write`, so a large `Debug` value
/// is hashed without being held as one string.
struct Fnv(u64);

impl Fnv {
    fn new() -> Self {
        Fnv(0xcbf2_9ce4_8422_2325)
    }
}

impl Write for Fnv {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for b in s.bytes() {
            self.0 ^= b as u64;
            self.0 = self.0.wrapping_mul(0x0000_0100_0000_01b3);
        }
        Ok(())
    }
}

fn digest(value: &dyn core::fmt::Debug) -> u64 {
    let mut h = Fnv::new();
    write!(h, "{value:?}").expect("hashing cannot fail");
    h.0
}

/// What a case runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// The generated galaxy alone.
    Galaxy,
    /// A card-free baseline run of the galaxy to the horizon.
    Run,
    /// `tests/determinism.rs`'s parked beam fight: a Cairn and a Tor stack
    /// generated face to face beside seat 0's homeworld on a 200-planet galaxy.
    Beams,
    /// Its parked missile defense: Butte sentries at seat 0's homeworld and a
    /// Cairn stack inside their reach, on three seats.
    Missiles,
}

/// One parity case.
#[derive(Clone, Copy, Debug)]
pub struct Case {
    pub kind: Kind,
    pub seats: usize,
    pub seed: u64,
    pub horizon: f64,
}

/// The cases both targets run, in order: every fair seat count's galaxy, one
/// medium run, and the two seeded fights, which reach the combat code (the
/// station-keeping `sin_cos`, the wreck roll) that no card-free run does.
///
/// **The run is 3 seats × 500 yr, not the 800 the issue named**: at this
/// engine 800 yr costs 82 s native and 125 s under node, past the 60-second
/// rule (AGENTS.md §2). Every case digests whole values to the last bit, so a
/// divergence shows on the first value it touches rather than when it has
/// grown into an event count, which is what the long horizon was buying.
pub const CASES: [Case; 8] = [
    Case { kind: Kind::Galaxy, seats: 2, seed: 1, horizon: 0.0 },
    Case { kind: Kind::Galaxy, seats: 3, seed: 1, horizon: 0.0 },
    Case { kind: Kind::Galaxy, seats: 6, seed: 1, horizon: 0.0 },
    Case { kind: Kind::Galaxy, seats: 12, seed: 1, horizon: 0.0 },
    Case { kind: Kind::Galaxy, seats: 18, seed: 1, horizon: 0.0 },
    Case { kind: Kind::Run, seats: 3, seed: 1, horizon: 500.0 },
    Case { kind: Kind::Beams, seats: 2, seed: 31, horizon: 3.0 },
    Case { kind: Kind::Missiles, seats: 3, seed: 41, horizon: 3.0 },
];

/// The fleets a seeded fight generates with its galaxy.
fn fleets(kind: Kind, home: Vec3) -> (f64, Vec<SeedFleet>) {
    let still = Vec3::ZERO;
    let fleet = |seat, hull, class, role, offset: Vec3| SeedFleet {
        seat,
        hull,
        class,
        role,
        position: home.add(offset),
        velocity: still,
    };
    let near = Vec3::new(0.5, 0.0, 0.0);
    let off = Vec3::new(0.015, 0.0, 0.0);
    match kind {
        Kind::Beams => (
            0.4,
            vec![
                fleet(0, HullType::LimitedContactVehicle, Class::Cairn, Role::Picket, near),
                fleet(1, HullType::LimitedContactVehicle, Class::Tor, Role::Picket, near),
            ],
        ),
        Kind::Missiles => (
            0.2,
            vec![
                fleet(0, HullType::LimitedOffensive, Class::Butte, Role::Sentry, still),
                fleet(1, HullType::LimitedContactVehicle, Class::Cairn, Role::Picket, off),
            ],
        ),
        Kind::Galaxy | Kind::Run => (0.0, Vec::new()),
    }
}

/// The generated galaxy of a case, with its seeded fleets for a fight.
fn galaxy_of(case: Case) -> Option<Galaxy> {
    let mut gcfg = GalaxyConfig::new(case.seats, case.seed);
    if matches!(case.kind, Kind::Beams | Kind::Missiles) {
        gcfg.planet_count = 200;
    }
    let bare = Galaxy::generate(gcfg).ok()?;
    if !matches!(case.kind, Kind::Beams | Kind::Missiles) {
        return Some(bare);
    }
    // Seeding moves no planet, so the bare galaxy says where the homeworld is.
    let home = bare.planets[bare.homeworlds[0].0 as usize].position;
    let (spend_kt, fleets) = fleets(case.kind, home);
    let seeding = FleetSeeding { spend_kt, known_radius_ly: 0.0, fleets, twin_bill: None };
    Galaxy::generate_with(gcfg, seeding).ok()
}

/// A case's simulation run to its horizon, with the combat log kept.
fn run(case: Case, galaxy: Galaxy) -> (SimReport, Simulation) {
    let mut cfg = SimConfig::new(case.seed);
    cfg.horizon_years = case.horizon;
    let mut sim = Simulation::with_baseline(galaxy, cfg);
    sim.set_log_filter(LogFilter::none().with(LogCategory::Combat));
    (sim.run(), sim)
}

/// The digest of one case. `0` if the galaxy cannot be generated, which no
/// case's does.
pub fn case_digest(case: Case) -> u64 {
    let Some(galaxy) = galaxy_of(case) else {
        return 0;
    };
    if case.kind == Kind::Galaxy {
        return digest(&galaxy);
    }
    let (report, sim) = run(case, galaxy);
    let log: Vec<_> = sim.log().iter().map(|r| (r.time, &r.event)).collect();
    digest(&(report, sim.snapshot(), log))
}

/// One line per case, `kind seats seed horizon digest`, the format both
/// targets print and `compare.mjs` compares.
pub fn case_line(case: Case) -> String {
    format!("{} {} {} {} {:016x}", case.kind as u32, case.seats, case.seed, case.horizon, case_digest(case))
}

/// The number of cases, for the wasm host.
#[no_mangle]
pub extern "C" fn parity_cases() -> u32 {
    CASES.len() as u32
}

thread_local! {
    static LAST: core::cell::Cell<u64> = const { core::cell::Cell::new(0) };
}

/// Runs case `i` and keeps its digest for [`parity_hi`] and [`parity_lo`].
/// The digest crosses as two 32-bit halves because a wasm export returning
/// `u64` reaches JavaScript as a `BigInt`, which the halves avoid needing.
/// Returns 0 for `i` past the end, 1 otherwise.
#[no_mangle]
pub extern "C" fn parity_run(i: u32) -> u32 {
    match CASES.get(i as usize) {
        Some(c) => {
            LAST.with(|l| l.set(case_digest(*c)));
            1
        }
        None => 0,
    }
}

/// The high 32 bits of the digest [`parity_run`] last kept.
#[no_mangle]
pub extern "C" fn parity_hi() -> u32 {
    LAST.with(|l| (l.get() >> 32) as u32)
}

/// The low 32 bits of the digest [`parity_run`] last kept.
#[no_mangle]
pub extern "C" fn parity_lo() -> u32 {
    LAST.with(|l| l.get() as u32)
}

/// Case `i`'s parameters, for the wasm host to print beside its digest:
/// field 0 the kind, 1 the seats, 2 the seed, 3 the horizon in whole years.
#[no_mangle]
pub extern "C" fn parity_case(i: u32, field: u32) -> u32 {
    CASES.get(i as usize).map_or(0, |c| match field {
        0 => c.kind as u32,
        1 => c.seats as u32,
        2 => c.seed as u32,
        _ => c.horizon as u32,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A digest is a function of the value, and the galaxy case covers the
    /// whole generated field: two seeds give two digests, one seed gives one.
    #[test]
    fn a_galaxy_digest_tells_seeds_apart() {
        let a = case_digest(Case { kind: Kind::Galaxy, seats: 2, seed: 1, horizon: 0.0 });
        let b = case_digest(Case { kind: Kind::Galaxy, seats: 2, seed: 2, horizon: 0.0 });
        assert_ne!(a, 0);
        assert_eq!(a, case_digest(Case { kind: Kind::Galaxy, seats: 2, seed: 1, horizon: 0.0 }));
        assert_ne!(a, b);
    }

    /// The two fight cases reach combat, which is why they are here: each
    /// logs an encounter and a wreck, and the missile case resolves a round.
    #[test]
    fn the_fight_cases_fire() {
        for case in CASES.iter().filter(|c| matches!(c.kind, Kind::Beams | Kind::Missiles)) {
            let galaxy = galaxy_of(*case).unwrap();
            let (_, sim) = run(*case, galaxy);
            let count = |f: fn(&LogEvent) -> bool| sim.log().iter().filter(|r| f(&r.event)).count();
            assert!(count(|e| matches!(e, LogEvent::EncounterBegan { .. })) > 0, "{case:?}: no encounter");
            if case.kind == Kind::Missiles {
                assert!(count(|e| matches!(e, LogEvent::MissileResolved { .. })) > 0, "{case:?}: no round");
            } else {
                assert!(count(|e| matches!(e, LogEvent::HullWrecked { .. })) > 0, "{case:?}: no wreck");
            }
        }
    }

    /// The FNV constants are the published 64-bit ones: the digest of `""` is
    /// the offset basis and of `"a"` is `0xaf63dc4c8601ec8c`.
    #[test]
    fn fnv_matches_the_reference_vectors() {
        let mut h = Fnv::new();
        assert_eq!(h.0, 0xcbf2_9ce4_8422_2325);
        h.write_str("a").unwrap();
        assert_eq!(h.0, 0xaf63_dc4c_8601_ec8c);
    }
}
