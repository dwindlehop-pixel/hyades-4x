//! Determinism stress tests.
//!
//! The engine must be bit-reproducible for the Monte-Carlo balancer
//! (`Hyades_card_contract.md` §7) and must expose deterministic continuous
//! `(x, y, z)` for every entity at every instant. These tests hammer both:
//! identical seeds must agree exactly on outcomes *and* on densely-sampled
//! in-flight positions across the entire timeline.

use hyades_engine::prelude::*;

/// Build a run with an explicit, short horizon — determinism holds at any
/// point in the run, so proving it does not need the full default horizon.
/// Every test here pins one explicitly: the shipped defaults snowball to
/// thousands of vehicles across 4,000 years (AGENTS.md design law #9), and a
/// full-length debug run costs a minute-plus each. Bit-identity is a property
/// of the arithmetic, not of how long you let it accumulate.
fn fresh_short(players: usize, seed: u64, horizon_years: f64) -> Simulation {
    let galaxy = Galaxy::generate(GalaxyConfig::new(players, seed)).unwrap();
    let mut cfg = SimConfig::new(seed);
    cfg.horizon_years = horizon_years;
    Simulation::with_baseline(galaxy, cfg)
}

/// **A galaxy with just enough in it to exercise one mechanism** — the
/// alternative to shortening a full bed.
///
/// Not every test is measuring a gradient, and only the ones that are need to
/// be comparable to each other. A test that asserts *"no entity moves faster
/// than c"* is asserting a property of `math::position_along`; it needs ships
/// in flight and nothing else. Running it on the standard bed makes it pay for
/// a colonization economy, a mineral field and thousands of planets it never
/// reads — and then the only lever left when the bed gets more expensive is to
/// cut the horizon, which eventually cuts the mechanism out too.
///
/// `planet_count` is a plain override (`GalaxyConfig`), so the cheaper move is
/// to shrink the *galaxy*: keep the full horizon, keep the mechanism, drop the
/// scenery. **Reduce complexity before duration.**
fn tiny_galaxy(players: usize, seed: u64, planets: usize, horizon_years: f64) -> Simulation {
    let mut gcfg = GalaxyConfig::new(players, seed);
    gcfg.planet_count = planets;
    let galaxy = Galaxy::generate(gcfg).unwrap();
    let mut cfg = SimConfig::new(seed);
    cfg.horizon_years = horizon_years;
    Simulation::with_baseline(galaxy, cfg)
}

/// **One seat count per test** (T-127). The five arms were one test that ran
/// them in sequence, and it was the whole determinism target's critical path —
/// **64.9 s** on its own on the T-127 machine, against a 60-second budget, on
/// the old binary as on the new. As five tests the harness runs them in
/// parallel; every assertion and every galaxy is unchanged.
///
/// Every fair seat count, 18 included (R-NET14). `Hyades_netcode.md` §6 makes
/// bit-reproducibility a *network* property, not only an MC one: a divergence
/// at any seat count is a desync, and 18 is the count the protocol is now
/// specified for.
///
/// **Full-size galaxies, and the horizon is now per seat count.** This is the
/// one test `AGENTS.md` §2 says must *keep* the scenery: an ordering fault in
/// a large collection only shows at scale. So the lever is duration — and
/// bit-identity is an arithmetic identity, which needs no horizon at all
/// beyond enough of one that the mechanism has fired.
///
/// A *uniform* horizon was the wrong shape. Cost goes as events, which go as
/// seats × years, so 100 yr for everybody made the 18-seat arm pay for the
/// whole target while the 2-seat arm ran only **812 events** — the thinnest
/// coverage sat where the budget was not being spent. Opening the build-wide
/// axis (R-O88) tripled the hulls in the water by any given year and took
/// this target 30 s → 58 s, which is what forced the question.
///
/// Equalising instead — each seat count gets the horizon that buys it a
/// comparable number of events — is **cheaper and covers more**: the target
/// comes back under budget *and* the worst-covered arm goes from 812 events
/// to ~1,900. Measured events per arm at these horizons: 2,761 / 2,7xx /
/// 3,0xx / ~1,800 / ~1,900.
///
/// **Re-scaled again at T-98**, and this time event count went *down* for a
/// change that made the simulation do more: sizing a hauler's hull to its
/// rock means **fewer, bigger trips** for the same tonnage, so every arm lost
/// roughly a third of its events at a fixed horizon and three of the five
/// dropped under the floor. Re-measured: 2,381 / 1,853 / 1,775 / 1,930 /
/// 2,152 events at the horizons below. That is the `yr/s`-versus-`ns/event`
/// distinction (`AGENTS.md` §2) showing up in a test budget — the work per
/// event rose, the event count fell, and only one of those is visible here.
///
/// **Re-scaled at T-88**, and the first attempt overshot — which is the
/// point of the floor. `cycle_years` 50 → 5 makes the economy tick ten times
/// as often, so dividing every horizon by ten looked right and left the
/// 2-seat arm on **450 events**: event count is not linear in the horizon
/// here, because the early game has one center and the tick multiplier has
/// almost nothing to multiply. Measured instead, these horizons buy each arm
/// ~1,500 events — against the 1,800–3,000 the old ones
/// bought — and take the target from 267 s back under budget.
/// The rule `AGENTS.md` §2 states is the one being followed — when a change
/// raises event count, check the test horizons *in the same commit* — and
/// the floor below is what says the trim did not go too far.
///
/// The `events_processed` floor below is what stops any future trim going
/// vacuous — the same guard, and for the same reason, as `moving` in
/// `positions_never_exceed_lightspeed`. It fired on the first attempt here
/// too, at a uniform 50 yr.
fn full_run_reports_are_bit_identical(n: usize, seed: u64, horizon: f64) {
    let mut a = fresh_short(n, seed, horizon);
    let mut b = fresh_short(n, seed, horizon);
    let ra = a.run();
    let rb = b.run();
    assert!(
        ra.events_processed > 1_000,
        "n={n} seed={seed} processed only {} events — the horizon has been cut past the point where              this test asserts anything",
        ra.events_processed
    );
    assert_eq!(ra.events_processed, rb.events_processed, "events n={n} seed={seed}");
    assert_eq!(ra.planets_scanned_total, rb.planets_scanned_total);
    for (pa, pb) in ra.players.iter().zip(rb.players.iter()) {
        assert_eq!(pa.planets_owned, pb.planets_owned);
        assert_eq!(pa.colonies, pb.colonies);
        assert_eq!(pa.mining_outposts, pb.mining_outposts);
        assert_eq!(pa.total_population.kilotons().to_bits(), pb.total_population.kilotons().to_bits());
    }
}

#[test]
fn full_run_reports_are_bit_identical_2_seats() {
    full_run_reports_are_bit_identical(2, 1, 90.0);
}

#[test]
fn full_run_reports_are_bit_identical_3_seats() {
    full_run_reports_are_bit_identical(3, 7, 60.0);
}

#[test]
fn full_run_reports_are_bit_identical_6_seats() {
    full_run_reports_are_bit_identical(6, 13, 40.0);
}

#[test]
fn full_run_reports_are_bit_identical_12_seats() {
    full_run_reports_are_bit_identical(12, 99, 26.0);
}

#[test]
fn full_run_reports_are_bit_identical_18_seats() {
    full_run_reports_are_bit_identical(18, 4, 20.0);
}

#[test]
fn continuous_positions_are_bit_identical_across_the_timeline() {
    // Two runs of the same seed, stepped in lockstep; at a dense grid of sample
    // times we compare every entity's (x, y, z) bit-for-bit. In-flight ships make
    // this a real test (their positions are interpolated, not snapped to events).
    // 3 players / 1500 steps, not 6 / 6000: doesn't need the biggest player
    // count or the full step budget to prove the property, and the galaxy is
    // now thousands of planets at the default 10 ly hex (this conversation) —
    // `checks` below still clears its floor by a wide margin either way.
    let mut a = fresh_short(3, 4242, 800.0);
    let mut b = fresh_short(3, 4242, 800.0);

    let mut checks = 0u64;
    for _ in 0..1500 {
        let a_more = a.step();
        let b_more = b.step();
        assert_eq!(a_more, b_more);
        if !a_more {
            break;
        }
        let t = a.clock();
        // sample slightly before/at/after the current event time
        for &s in &[t - 3.0, t, t + 3.0, t + 11.0] {
            let pa = a.positions_at(s);
            let pb = b.positions_at(s);
            assert_eq!(pa.len(), pb.len());
            for (x, y) in pa.iter().zip(pb.iter()) {
                assert_eq!(x.x.to_bits(), y.x.to_bits(), "x mismatch at t={s}");
                assert_eq!(x.y.to_bits(), y.y.to_bits(), "y mismatch at t={s}");
                assert_eq!(x.z.to_bits(), y.z.to_bits(), "z mismatch at t={s}");
            }
            checks += pa.len() as u64;
        }
    }
    assert!(checks > 10_000, "stress test did not sample enough positions ({checks})");
}

#[test]
fn positions_never_exceed_lightspeed() {
    // Sample displacement over small windows the whole way to the horizon; no
    // entity may move faster than c (= 1 ly/yr).
    //
    // **The scenery came out, not the timeline.** This asserts a property of
    // `math::position_along` — no entity moves faster than c — which needs
    // *ships in flight* and reads nothing else: no economy, no mineral field,
    // no colonization. On the standard bed it paid for all three, and it was
    // **97 s of a 102 s target**, so the only lever left each time the bed got
    // more expensive was to cut the horizon. Cut it far enough and there is
    // nothing flying and the test passes vacuously.
    //
    // A 150-planet galaxy launches scouts from the first year and yields
    // **5,087 in-flight samples in 0.8 s** — five times the guard's floor, at
    // the *full* 800-year horizon rather than the 300 an earlier trim had cut
    // it to. So the horizon went back **up** and the test got 120x cheaper.
    // That is the trade: reduce complexity before duration. (Measured: 60
    // planets gives 1,541 samples, 400 gives 13,682 at 2.1 s — 150 is the
    // comfortable middle, not a guess.)
    const HORIZON: f64 = 800.0;
    let mut sim = tiny_galaxy(3, 808, 150, HORIZON);
    sim.run();
    let dt = 0.25;
    let mut t = 0.0;
    let mut moving = 0u64;
    while t < HORIZON {
        let p0 = sim.positions_at(t);
        let p1 = sim.positions_at(t + dt);
        for (u, v) in p0.iter().zip(p1.iter()) {
            let d = u.distance(*v);
            assert!(d <= dt + 1e-6, "superluminal motion near t={t}");
            if d > 1e-9 {
                moving += 1;
            }
        }
        t += 2.0;
    }
    // **The non-vacuity guard, and it earned its place** — it fired on the
    // first attempt at the earlier horizon trim, which is how that trim was
    // caught being too aggressive rather than shipping green and empty.
    assert!(moving > 1_000, "nothing was in flight: the bound was never exercised ({moving})");
}

#[test]
fn stepping_in_any_granularity_reaches_the_same_state() {
    // Driving the loop by hand must match run() exactly. Shorter horizon
    // (this conversation: galaxies are now thousands of planets at the
    // default 10 ly hex) — the property holds at any horizon length.
    let mut a = fresh_short(6, 555, 100.0);
    while a.step() {}
    let ra = a.report();

    let mut b = fresh_short(6, 555, 100.0);
    let rb = b.run();

    assert_eq!(ra.events_processed, rb.events_processed);
    assert_eq!(ra.planets_scanned_total, rb.planets_scanned_total);
    for (pa, pb) in ra.players.iter().zip(rb.players.iter()) {
        assert_eq!(pa.planets_owned, pb.planets_owned);
        assert_eq!(pa.total_population.kilotons().to_bits(), pb.total_population.kilotons().to_bits());
    }
}

/// **H3 / R-NET11 — no NaN reaches replicated state.**
///
/// `Hyades_netcode.md` §6 H3: core WASM picks arithmetic-NaN payloads
/// *nondeterministically*, so NaN bits differ across browser engines even with
/// relaxed SIMD disabled. A NaN that reaches the hashed state is therefore a
/// latent, intermittent, unreproducible desync — the worst failure class the
/// protocol has, because it presents as one client being "wrong" with no
/// reproducer.
///
/// The state digest (§8.1) does not exist yet, so this guards the two surfaces
/// that will feed it: the report and the full snapshot. It is the seed of the
/// H3 discipline rather than the whole of it — when the digest lands, the check
/// belongs *inside* it, as a fatal error rather than a test.
///
/// Infinities are checked too. They are deterministic in WASM, so they are not
/// a desync, but they reach NaN in one subtraction and there is no legitimate
/// infinite quantity in this model.
#[test]
fn no_nan_or_infinity_reaches_replicated_state() {
    // **150 yr, trimmed at R-MX8** (200 at R-O88, 400 before). The assertion
    // is an invariant — no non-finite value reaches replicated state — so it
    // needs the mechanism to have fired, not a long accumulation. R-MX8 moved
    // this bed from 37 s to 44 s at 200 yr; probed at 100 / 130 / 150 / 170 /
    // 200 yr: 9.0k / 14.7k / 19.3k / 24.6k / 33.5k events in 17 / 24 / 30 / 35
    // / 44 s, and the first center-to-center load on this bed is at 32.4 yr, so
    // every one of those horizons walks the new path. The full galaxy stays:
    // this walks every planet's snapshot fields, so breadth is what it is
    // actually reading.
    let mut sim = fresh_short(6, 31337, 150.0);
    let report = sim.run();
    assert!(
        report.events_processed > 1_000,
        "only {} events — the trim has gone past where this asserts anything",
        report.events_processed
    );

    let finite = |v: f64, what: &str| assert!(v.is_finite(), "{what} is not finite: {v}");

    for (i, p) in report.players.iter().enumerate() {
        finite(p.total_population.kilotons(), &format!("report.players[{i}].total_population"));
    }

    let snap = sim.snapshot();
    finite(snap.time_years, "snapshot.time_years");
    for (i, p) in snap.players.iter().enumerate() {
        finite(p.total_population.kilotons(), &format!("players[{i}].total_population"));
        finite(p.stockpile_total, &format!("players[{i}].stockpile_total"));
    }
    for pl in &snap.planets {
        let id = pl.id.0;
        for (v, what) in [
            (pl.position.x, "position.x"),
            (pl.position.y, "position.y"),
            (pl.position.z, "position.z"),
            (pl.habitability.bands(), "habitability"),
            (pl.biosphere.bands(), "biosphere"),
            (pl.bio_max.bands(), "bio_max"),
            (pl.biomass.kilotons(), "biomass"),
            (pl.infrastructure.bands(), "infrastructure"),
            (pl.k.bands(), "k"),
            (pl.population.kilotons(), "population"),
            (pl.stockpile.basic_total().kilotons(), "stockpile"),
        ] {
            finite(v, &format!("planet {id} {what}"));
        }
    }
    for (i, v) in snap.vehicles.iter().enumerate() {
        for (val, what) in [
            (v.position.x, "position.x"),
            (v.position.y, "position.y"),
            (v.position.z, "position.z"),
            (v.cargo.basic_total().kilotons(), "cargo"),
        ] {
            finite(val, &format!("vehicle {i} {what}"));
        }
    }
}

// ---------------------------------------------------------------------------
// **Bit-identity with shots fired** (T-133). The tests above play no card, so
// no Design is armed and no fire code runs in them. These do, each from its own
// initial conditions, so that no one galaxy decides whether a mechanism fires
// inside the gate: a single card-play bed once carried the whole of combat,
// and every change to galaxy generation moved which seeds launched a missile
// at all. Fleets are generated with the galaxy (`Galaxy::generate_with`, the
// author's ruling: the only thing a bed varies is the galaxy) and placed where
// the mechanism must fire. Two runs must agree on every combat record, to the
// last bit of its time and slag, and on the report; the floors say the
// mechanism fired.
// ---------------------------------------------------------------------------

use hyades_engine::galaxy::{FleetSeeding, SeedFleet};
use hyades_engine::log::MissileOutcome;
use hyades_engine::sim::Class;

/// What one combat run did: encounters begun, hulls wrecked, course changes,
/// missile rounds by outcome, and magazines refilled at a center.
#[derive(Debug, Default, Clone, Copy)]
struct Fired {
    encounters: usize,
    wrecks: usize,
    turns: usize,
    hits: usize,
    intercepted: usize,
    missed: usize,
    /// Magazines refilled at a center.
    rearmed: u64,
}

impl Fired {
    fn rounds(&self) -> usize {
        self.hits + self.intercepted + self.missed
    }
}

/// One set of initial conditions: a galaxy, the fleets generated with it, the
/// cards each seat plays at the first round barrier (none for a seeded fight),
/// and a horizon.
#[derive(Clone)]
struct Scenario {
    seats: usize,
    seed: u64,
    planets: usize,
    spend_kt: f64,
    /// Fleets, placed relative to a homeworld: `(seat, hull, class, role,
    /// homeworld whose position the offset is from, offset ly, velocity ly/yr)`.
    fleets: Vec<(usize, HullType, Class, Role, usize, Vec3, Vec3)>,
    cards: Option<Vec<u16>>,
    horizon: f64,
}

fn combat_run(sc: &Scenario) -> (SimReport, Vec<String>, Fired) {
    let mut gcfg = GalaxyConfig::new(sc.seats, sc.seed);
    gcfg.planet_count = sc.planets;
    // Seeding moves no planet, so the bare galaxy says where the homeworlds are.
    let bare = Galaxy::generate(gcfg).unwrap();
    let home = |p: usize| bare.planets[bare.homeworlds[p].0 as usize].position;
    let fleets = sc
        .fleets
        .iter()
        .map(|&(seat, hull, class, role, near, offset, velocity)| SeedFleet {
            seat,
            hull,
            class,
            role,
            position: home(near).add(offset),
            velocity,
        })
        .collect();
    let seeding = FleetSeeding { spend_kt: sc.spend_kt, known_radius_ly: 0.0, fleets, twin_bill: None };
    let galaxy = Galaxy::generate_with(gcfg, seeding).unwrap();
    let mut cfg = SimConfig::new(sc.seed);
    cfg.horizon_years = sc.horizon;
    let play_at = cfg.years_to_first_round;
    let mut sim = Simulation::with_baseline(galaxy, cfg);
    sim.set_log_filter(LogFilter::none().with(LogCategory::Combat));
    let mut played = sc.cards.is_none();
    while sim.step() {
        if !played && sim.clock() >= play_at {
            let cards = sc.cards.as_ref().unwrap();
            let orders: Vec<Order> = (0..sc.seats)
                .map(|i| Order {
                    seat: PlayerId(i as u32),
                    card: Some(CardId(cards[i % cards.len()])),
                    target: Target::None,
                })
                .collect();
            sim.apply_orders(sim.current_round(), &orders);
            played = true;
        }
    }
    // `{:?}` on an `f64` prints its shortest round-trip form, so equal strings
    // are equal bits.
    let log = sim.log().iter().map(|r| format!("{:?} {:?}", r.time.to_bits(), r.event)).collect();
    let mut f = Fired { rearmed: sim.missile_stats().rearmed_at_center, ..Fired::default() };
    for r in sim.log().iter() {
        match r.event {
            LogEvent::EncounterBegan { .. } => f.encounters += 1,
            LogEvent::HullWrecked { .. } => f.wrecks += 1,
            LogEvent::CourseChanged { .. } => f.turns += 1,
            LogEvent::MissileResolved { outcome: MissileOutcome::Hit, .. } => f.hits += 1,
            LogEvent::MissileResolved { outcome: MissileOutcome::Intercepted, .. } => f.intercepted += 1,
            LogEvent::MissileResolved { outcome: MissileOutcome::Missed, .. } => f.missed += 1,
            _ => {}
        }
    }
    (sim.report(), log, f)
}

/// Run a scenario twice and assert the two runs are one run; return what it did.
fn bit_identical(name: &str, sc: &Scenario) -> Fired {
    let (ra, la, fired) = combat_run(sc);
    let (rb, lb, _) = combat_run(sc);
    eprintln!("{name}: {fired:?}");
    assert_eq!(la.len(), lb.len(), "{name}: combat record count");
    for (i, (a, b)) in la.iter().zip(&lb).enumerate() {
        assert_eq!(a, b, "{name}: combat record {i}");
    }
    assert_eq!(ra.events_processed, rb.events_processed, "{name}: events");
    for (pa, pb) in ra.players.iter().zip(rb.players.iter()) {
        assert_eq!(pa.colonies, pb.colonies, "{name}: colonies");
        assert_eq!(pa.mining_outposts, pb.mining_outposts, "{name}: outposts");
        assert_eq!(pa.total_population.kilotons().to_bits(), pb.total_population.kilotons().to_bits(), "{name}");
    }
    fired
}

const STILL: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

/// **Beam fleets meet** — two seats, two sets of initial conditions: a Cairn
/// stack and a Tor stack parked face to face beside a homeworld, and a Tor
/// stack closing at 0.3 c on a parked Cairn stack from 0.03 ly, braking into
/// its reach. Wrecks and
/// course changes must happen in both.
#[test]
fn beam_fights_are_bit_identical() {
    let picket = |seat: usize, class: Class, offset: Vec3, velocity: Vec3| {
        (seat, HullType::LimitedContactVehicle, class, Role::Picket, 0usize, offset, velocity)
    };
    let near = Vec3::new(0.5, 0.0, 0.0);
    let parked = Scenario {
        seats: 2,
        seed: 31,
        planets: 200,
        spend_kt: 0.4,
        fleets: vec![picket(0, Class::Cairn, near, STILL), picket(1, Class::Tor, near, STILL)],
        cards: None,
        horizon: BEAM_HORIZON,
    };
    let closing = Scenario {
        seed: 32,
        fleets: vec![
            picket(0, Class::Cairn, near, STILL),
            picket(1, Class::Tor, near.add(Vec3::new(0.03, 0.0, 0.0)), Vec3::new(-0.3, 0.0, 0.0)),
        ],
        ..parked.clone()
    };
    for (name, sc) in [("beam, parked", parked), ("beam, closing", closing)] {
        let f = bit_identical(name, &sc);
        assert!(f.encounters > 0 && f.wrecks > 0, "{name}: the beams must fight: {f:?}");
        assert!(f.turns > 0, "{name}: a fleet must change course, so the belief path runs: {f:?}");
    }
}

/// **Sentries defend a center and rearm there** — three seats: missile
/// sentries generated at seat 0's homeworld, and seat 1's armed beam stack
/// inside their reach and outside a beam's, parked in one set of conditions and
/// closing in the other. Rounds must fly, some must be stopped by the stack's
/// point defense or hit, and an emptied sentry must refill from the center's
/// bank. (A sentry fires only on armed hulls, so the raider is armed.)
///
/// The supply line's other two paths — an ammo run and a flight home — need a
/// post or a voyage to return to, and a fleet generated with the galaxy has
/// neither, so no seeded bed reaches them; `a_dry_missile_picket_is_resupplied_by_ammo_run_or_by_return`
/// covers them in the unit target.
#[test]
fn missile_defense_is_bit_identical() {
    let sentries = (0, HullType::LimitedOffensive, Class::Butte, Role::Sentry, 0usize, STILL, STILL);
    let off = Vec3::new(0.015, 0.0, 0.0);
    let parked = Scenario {
        seats: 3,
        seed: 41,
        planets: 200,
        spend_kt: 0.2,
        fleets: vec![sentries, (1, HullType::LimitedContactVehicle, Class::Cairn, Role::Picket, 0, off, STILL)],
        cards: None,
        horizon: MISSILE_HORIZON,
    };
    let closing = Scenario {
        seed: 42,
        fleets: vec![
            sentries,
            (
                1,
                HullType::LimitedContactVehicle,
                Class::Tor,
                Role::Picket,
                0,
                off.scale(3.0),
                Vec3::new(-0.2, 0.0, 0.0),
            ),
        ],
        ..parked.clone()
    };
    for (name, sc) in [("sentries, parked raider", parked), ("sentries, closing raider", closing)] {
        let f = bit_identical(name, &sc);
        assert!(f.rounds() > 0, "{name}: no round resolved, so the missile path never ran: {f:?}");
        assert!(f.hits + f.intercepted > 0, "{name}: no round hit or was stopped: {f:?}");
        assert!(f.rearmed > 0, "{name}: no sentry refilled its magazine at the center: {f:?}");
    }
}

/// **A played game** — the card bed's protocol on a small galaxy: six seats,
/// 600 planets, the Warfare, Growth and missile cards in rotation at the first
/// round barrier, as a game plays them. Whatever fights the cards produce must
/// reproduce; the floors are on beams and the belief path, which every seed
/// reaches, and not on missiles, which the seeded tests above carry.
#[test]
fn a_card_play_game_is_bit_identical() {
    for seed in [1u64, 7] {
        let sc = Scenario {
            seats: 6,
            seed,
            planets: 600,
            spend_kt: 0.0,
            fleets: Vec::new(),
            cards: Some(vec![15, 3, 13]),
            horizon: CARD_HORIZON,
        };
        let f = bit_identical(&format!("cards, seed {seed}"), &sc);
        assert!(f.encounters >= 50, "seed {seed}: only {} encounters — the bed no longer reaches combat", f.encounters);
        assert!(f.wrecks > 0 && f.turns > 0, "seed {seed}: the wreck and belief paths must run: {f:?}");
    }
}

const BEAM_HORIZON: f64 = 3.0;
const MISSILE_HORIZON: f64 = 3.0;
const CARD_HORIZON: f64 = 350.0;
