//! **The demo games** the client ships (`docs/Hyades_interface.md` §2, §8):
//! each a recipe — a galaxy, its seeded fleets, the cards played at the round
//! barrier and how to frame the record — that [`Demo::recorder`] turns into a
//! replay. The engine is bit-deterministic, native and wasm32 alike (T-127), so
//! a recipe is the replay: the browser's engine module records it where it is
//! watched (T-180), and `examples/record_replay` writes the same bytes to
//! files.
//!
//! | name | what it shows |
//! |---|---|
//! | `expansion` | three seats founding, mining and hauling over 400 yr |
//! | `beams` | a Tor stack closing on a parked Cairn stack beside a homeworld, the fight frame by frame |
//! | `palette` | eighteen seats, a picket stack of each on a ring by a homeworld — half closing on the middle, to fight, brake and withdraw — then six centuries of every seat's traffic, a card played by each at the 200-yr barrier, and twin Designs billed in supers: the palette in play, for ratification |
//! | `forge` | six seats over eight centuries: a parade of every hull and named Design on its role at the start, a card played by each at the barrier, twin Designs billed in supers — forges, the supers and the apex held and carried, and `Band IV` works |
//! | `sentries` | missile sentries at a center firing on an armed raider, rounds and point defense |
//!
//! The fights are the determinism suite's seeded fleets
//! (`tests/determinism.rs`), placed where the mechanism must fire — except
//! that `beams`' Tors close at the speed their drive can shed in the gap, so
//! they can come to rest at the Cairns rather than overshoot them.
//!
//! **Nothing here is a code path the game does not run** (`AGENTS.md` §6): a
//! demo varies the galaxy and plays cards through
//! [`Simulation::apply_orders`] at the protocol's own barrier.

use crate::cards::{CardId, Order, Target};
use crate::galaxy::{FleetSeeding, Galaxy, GalaxyConfig, PlayerId, SeedFleet};
use crate::log::{LogCategory, LogFilter};
use crate::math::Vec3;
use crate::replay::{Recorder, ReplayConfig};
use crate::sim::{Class, DesignBill, HullType, Role, SimConfig, Simulation};
use crate::transcendental::sin_cos;

/// **A demo's recipe.**
#[derive(Clone, Debug)]
pub struct Demo {
    pub name: &'static str,
    pub label: &'static str,
    pub seats: usize,
    pub seed: u64,
    pub planets: usize,
    pub horizon: f64,
    pub frame_years: f64,
    /// A dense opening `(until, frame_years)` before `frame_years` takes over.
    pub dense: Option<(f64, f64)>,
    /// Cards played at the first round barrier, `(seat, card id)`, through
    /// `apply_orders` (T-180).
    pub cards: Vec<(usize, u16)>,
    /// Every seat's twin Designs billed in refined materials, if any
    /// (`FleetSeeding::twin_bill`): a demand for supers and the apex.
    pub twin_bill: Option<DesignBill>,
    pub fleets: Vec<Fleet>,
    /// **A surveyed start**: every seat begins having scanned the worlds
    /// within this distance of its homeworld, ly (`FleetSeeding`), so a
    /// seeded colonizer or miner has somewhere to go. `0` is an ordinary start.
    pub known_radius_ly: f64,
    /// A fleet, by index, whose speed is set so that braking at its drive's
    /// acceleration brings it to rest at fleet 0's place: its velocity above
    /// gives only the direction.
    pub closing: Option<usize>,
    pub spend_kt: f64,
    pub filter: LogFilter,
}

/// `(seat, hull, class, role, offset from seat 0's homeworld ly, velocity ly/yr)`.
pub type Fleet = (usize, HullType, Class, Role, Vec3, Vec3);

const STILL: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

/// The demo named `name`, full or `quick`.
pub fn demo(name: &str, quick: bool) -> Option<Demo> {
    demos(quick).into_iter().find(|d| d.name == name)
}

/// **Every demo**, full or `quick` — short, small versions for the tests.
pub fn demos(quick: bool) -> Vec<Demo> {
    let picket = |seat: usize, class: Class, offset: Vec3, velocity: Vec3| {
        (seat, HullType::LimitedContactVehicle, class, Role::Picket, offset, velocity)
    };
    let near = Vec3::new(0.5, 0.0, 0.0);
    let fights = LogFilter::none().with(LogCategory::Combat).with(LogCategory::Vehicles);
    vec![
        Demo {
            name: "expansion",
            label: "Expansion — three seats founding, mining and hauling",
            seats: 3,
            seed: 7,
            planets: if quick { 150 } else { 300 },
            horizon: if quick { 60.0 } else { 400.0 },
            frame_years: 5.0,
            dense: None,
            cards: Vec::new(),
            twin_bill: None,
            fleets: Vec::new(),
            known_radius_ly: 0.0,
            closing: None,
            spend_kt: 0.0,
            filter: LogFilter::none().with(LogCategory::Vehicles).with(LogCategory::Combat).with(LogCategory::Cards),
        },
        Demo {
            name: "beams",
            label: "Beams — a Tor stack closing on a parked Cairn stack",
            seats: 2,
            seed: 32,
            planets: 200,
            horizon: if quick { 1.0 } else { 3.0 },
            frame_years: 0.02,
            dense: None,
            cards: Vec::new(),
            twin_bill: None,
            fleets: vec![
                picket(0, Class::Cairn, near, STILL),
                picket(1, Class::Tor, near.add(Vec3::new(0.03, 0.0, 0.0)), Vec3::new(-1.0, 0.0, 0.0)),
            ],
            known_radius_ly: 0.0,
            closing: Some(1),
            spend_kt: 0.4,
            filter: fights,
        },
        Demo {
            name: "palette",
            label: "Palette — eighteen seats' stacks meeting by a homeworld, every color in play",
            seats: 18,
            seed: 11,
            planets: 300,
            horizon: if quick { 2.0 } else { 600.0 },
            frame_years: if quick { 0.5 } else { 10.0 },
            dense: Some((1.0, 0.02)),
            cards: (0..18).map(|s| (s, [15, 13, 14, 3, 10, 5][s % 6])).collect(),
            twin_bill: Some(SUPER_BILL),
            fleets: palette_ring(),
            known_radius_ly: 0.0,
            closing: None,
            spend_kt: 0.2,
            filter: LogFilter::none().with(LogCategory::Combat).with(LogCategory::Vehicles).with(LogCategory::Cards),
        },
        Demo {
            name: "forge",
            label: "Forge — six seats over eight centuries: every Design, Band IV works, supers and the apex",
            seats: 6,
            seed: 11,
            planets: if quick { 150 } else { 300 },
            horizon: if quick { 4.0 } else { 800.0 },
            frame_years: if quick { 2.0 } else { 10.0 },
            dense: None,
            cards: (0..6).map(|s| (s, [15, 13, 14, 3, 10, 5][s])).collect(),
            twin_bill: Some(SUPER_BILL),
            fleets: if quick { Vec::new() } else { parade() },
            known_radius_ly: if quick { 0.0 } else { FORGE_KNOWN_LY },
            closing: None,
            spend_kt: PARADE_SPEND_KT,
            filter: LogFilter::none().with(LogCategory::Vehicles).with(LogCategory::Combat).with(LogCategory::Cards),
        },
        Demo {
            name: "sentries",
            label: "Sentries — missiles from a center on an armed raider",
            seats: 3,
            seed: 42,
            planets: 200,
            horizon: if quick { 1.0 } else { 3.0 },
            frame_years: 0.02,
            dense: None,
            cards: Vec::new(),
            twin_bill: None,
            fleets: vec![
                (0, HullType::LimitedOffensive, Class::Butte, Role::Sentry, STILL, STILL),
                picket(1, Class::Tor, Vec3::new(0.04, 0.0, 0.0), Vec3::new(-0.2, 0.0, 0.0)),
            ],
            known_radius_ly: 0.0,
            closing: None,
            spend_kt: 0.2,
            filter: fights,
        },
    ]
}

/// **The palette scenario's fleets**: one picket stack per seat on a ring of
/// `PALETTE_RING_LY` about a point beside seat 0's homeworld — Cairns and Tors
/// by turns, the odd seats closing on the middle at `PALETTE_CLOSING` ly/yr so
/// they brake, meet, fight and withdraw — and a sentry post of seat 0's at the
/// middle. Every seat's color, the drive's and the paths' shifts, hits,
/// damage, wrecks and retreat all come up within the first year.
const PALETTE_RING_LY: f64 = 0.03;
const PALETTE_CLOSING: f64 = 0.25;

fn palette_ring() -> Vec<Fleet> {
    let middle = Vec3::new(0.5, 0.0, 0.0);
    let mut fleets: Vec<Fleet> = (0..18)
        .map(|seat| {
            let (sin, cos) = sin_cos(core::f64::consts::TAU * seat as f64 / 18.0);
            let out = Vec3::new(cos, sin, 0.0);
            let class = if seat % 2 == 0 { Class::Cairn } else { Class::Tor };
            let velocity = if seat % 2 == 1 { out.scale(-PALETTE_CLOSING) } else { STILL };
            (
                seat,
                HullType::LimitedContactVehicle,
                class,
                Role::Picket,
                middle.add(out.scale(PALETTE_RING_LY)),
                velocity,
            )
        })
        .collect();
    fleets.push((0, HullType::LimitedOffensive, Class::Butte, Role::Sentry, middle, STILL));
    fleets
}

/// **Every seat's twin Designs billed partly in refined materials**: a
/// 0.15 of the price in each super and 0.05 in the apex
/// (`FleetSeeding::twin_bill`), so the supers and the apex have a buyer and
/// are carried to the yards that build the twins.
const SUPER_BILL: DesignBill = DesignBill { refined: [0.15, 0.15, 0.15, 0.05] };

/// **The parade's spend**, kt: enough to buy one General hull (1.0–1.3 kt),
/// so each seeded fleet below has at least one hull — about ten of a Medium
/// one and fifty of a Limited one.
const PARADE_SPEND_KT: f64 = 1.0;
/// How far each `forge` seat has surveyed at the start, ly.
const FORGE_KNOWN_LY: f64 = 10.0;

/// **The parade**: every named Design on its role, and the four hulls no
/// Design names yet, spread over `forge`'s six seats — so every Design and
/// role is in play from the start, built by nobody's yard. A seat's miners
/// come before its freighters, which serve their outposts; the pickets stand
/// near seat 0's homeworld.
fn parade() -> Vec<Fleet> {
    use HullType::*;
    let near = |x: f64, y: f64| Vec3::new(x, y, 0.0);
    vec![
        (0, LimitedSystems, Class::Spur, Role::Scout, STILL, STILL),
        (0, GeneralSystems, Class::Range, Role::Colonizer, STILL, STILL),
        (0, GeneralOffensive, Class::Unnamed, Role::Picket, near(0.6, 0.0), STILL),
        (1, LimitedContactVehicle, Class::Tor, Role::Scout, STILL, STILL),
        (1, MediumSystems, Class::Delta, Role::Colonizer, STILL, STILL),
        (1, LimitedContactUnit, Class::Unnamed, Role::Picket, near(0.6, 0.1), STILL),
        (2, LimitedSystems, Class::Meadow, Role::Miner, STILL, STILL),
        (2, MediumSystems, Class::Ford, Role::Freighter, STILL, STILL),
        (2, GeneralSystems, Class::Strait, Role::Freighter, STILL, STILL),
        (3, GeneralContactVehicle, Class::Scarp, Role::Colonizer, STILL, STILL),
        (3, GeneralContactUnit, Class::Unnamed, Role::Picket, near(0.6, -0.1), STILL),
        (4, LimitedContactVehicle, Class::Cairn, Role::Picket, near(0.5, 0.1), STILL),
        (4, RapidOffensive, Class::Unnamed, Role::Picket, near(0.5, -0.1), STILL),
        (5, LimitedOffensive, Class::Butte, Role::Sentry, STILL, STILL),
        (5, LimitedOffensive, Class::Mesa, Role::Picket, near(0.5, 0.0), STILL),
    ]
}

impl Demo {
    /// **The recipe, ready to record**: the galaxy with its fleets, the
    /// simulation, and the record's framing — its focus on the fleets, and
    /// its card plays at the first round barrier.
    pub fn recorder(&self) -> Recorder {
        let mut gcfg = GalaxyConfig::new(self.seats, self.seed);
        gcfg.planet_count = self.planets;
        let bare = Galaxy::generate(gcfg).expect("a demo's galaxy generates");
        let home = bare.planets[bare.homeworlds[0].0 as usize].position;
        let mut fleets = self.fleets.clone();
        if let Some(i) = self.closing {
            let gap = fleets[i].4.distance(fleets[0].4);
            let dir = fleets[i].5.normalized();
            // The drive's braking acceleration, read off the hull itself at
            // `t = 0` while it sheds a probe speed.
            fleets[i].5 = dir.scale(0.1);
            let probe = Simulation::with_baseline(self.galaxy(gcfg, home, &fleets), SimConfig::new(self.seed));
            let accel = probe
                .snapshot_at(0.0)
                .vehicles
                .iter()
                .find(|v| v.owner == fleets[i].0 as u32 && v.burn < 0)
                .expect("the closing fleet is braking")
                .accel;
            fleets[i].5 = dir.scale(stopping_speed(accel, gap));
        }
        let galaxy = self.galaxy(gcfg, home, &fleets);
        let mut cfg = SimConfig::new(self.seed);
        cfg.horizon_years = self.horizon;
        let barrier = cfg.years_to_first_round;
        let mut sim = Simulation::with_baseline(galaxy.clone(), cfg);
        sim.set_log_filter(self.filter);
        // A fight opens on its fleets: the circle around their starting
        // places, with room for the closing stack's run in.
        let focus = self.fleets.first().map(|_| {
            let (mut lo, mut hi) = ([f64::INFINITY; 2], [f64::NEG_INFINITY; 2]);
            for f in &self.fleets {
                let p = home.add(f.4);
                (lo[0], lo[1], hi[0], hi[1]) = (lo[0].min(p.x), lo[1].min(p.y), hi[0].max(p.x), hi[1].max(p.y));
            }
            let r = 0.5 * (hi[0] - lo[0]).max(hi[1] - lo[1]) + 0.05;
            (0.5 * (lo[0] + hi[0]), 0.5 * (lo[1] + hi[1]), r)
        });
        let orders: Vec<Order> = self
            .cards
            .iter()
            .map(|&(seat, card)| Order { seat: PlayerId(seat as u32), card: Some(CardId(card)), target: Target::None })
            .collect();
        let plays = if orders.is_empty() { Vec::new() } else { vec![(barrier, orders)] };
        let rc = ReplayConfig {
            frame_years: self.frame_years,
            dense: self.dense,
            plays,
            max_events: 50_000,
            label: self.label.into(),
            focus,
        };
        Recorder::new(&galaxy, sim, rc)
    }

    /// The galaxy with `fleets` seeded at their offsets from `home`.
    fn galaxy(&self, gcfg: GalaxyConfig, home: Vec3, fleets: &[Fleet]) -> Galaxy {
        let fleets = fleets
            .iter()
            .map(|&(seat, hull, class, role, offset, velocity)| SeedFleet {
                seat,
                hull,
                class,
                role,
                position: home.add(offset),
                velocity,
            })
            .collect();
        let seeding = FleetSeeding {
            spend_kt: self.spend_kt,
            known_radius_ly: self.known_radius_ly,
            fleets,
            twin_bill: self.twin_bill,
        };
        Galaxy::generate_with(gcfg, seeding).expect("a demo's galaxy generates")
    }
}

/// **The speed a hull sheds in exactly `length` ly** braking at proper
/// acceleration `accel` (ly/yr², `c = 1`): the braking length is
/// `(γ − 1) / accel`, so `γ = 1 + accel · length` and `v = √(γ² − 1) / γ`.
fn stopping_speed(accel: f64, length: f64) -> f64 {
    let gamma = 1.0 + accel * length;
    (gamma * gamma - 1.0).sqrt() / gamma
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **`forge`'s parade puts every hull and every named Design in play**
    /// (T-181): the ten hull types, and each Design the engine names.
    #[test]
    fn the_parade_has_every_hull_and_every_named_design() {
        use HullType::*;
        let p = parade();
        for hull in [
            LimitedSystems,
            MediumSystems,
            GeneralSystems,
            LimitedContactVehicle,
            LimitedContactUnit,
            GeneralContactVehicle,
            GeneralContactUnit,
            LimitedOffensive,
            RapidOffensive,
            GeneralOffensive,
        ] {
            assert!(p.iter().any(|f| f.1 == hull), "{hull:?}");
        }
        use Class::*;
        for class in [Meadow, Spur, Tor, Cairn, Delta, Range, Scarp, Ford, Strait, Butte, Mesa] {
            assert!(p.iter().any(|f| f.2 == class), "{class:?}");
        }
        let forge = demo("forge", false).unwrap();
        assert_eq!(forge.fleets.len(), p.len());
        assert!(forge.cards.len() == forge.seats && forge.twin_bill.is_some());
    }

    /// Every demo is named once, and the quick list names the same demos.
    #[test]
    fn every_demo_has_one_name_in_both_lists() {
        let names = |q| demos(q).iter().map(|d| d.name).collect::<Vec<_>>();
        let full = names(false);
        let mut unique = full.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), full.len());
        assert_eq!(names(true), full);
    }
}
