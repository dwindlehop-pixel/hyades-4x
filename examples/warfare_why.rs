//! **Where `W_0` goes** (T-116) — a decomposition of the Warfare card's
//! objective, because six arms of `denial_census` have now agreed on the sign
//! and none of them says *why*.
//!
//! `W_0 = C_0 − mean_j C_j` moves for two reasons that a single number cannot
//! separate: the card's own player founds fewer colonies, and its neighbors
//! found more. Those are not independent — the galaxy's colonizable worlds are
//! a shared pool, so a world seat 0 does not take is a world somebody else can
//! — and the ratio between them is the whole question. A card that costs its
//! player `k` colonies and hands `k` to the table loses `W_0` twice over; a
//! card that costs its player `k` and denies the table `m > k` wins.
//!
//! So this prints, per arm and per seed:
//!
//! - **colonizers built, by hull**, so "fewer colonies" can be traced to fewer
//!   *attempts* rather than to attempts that failed;
//! - **colonies founded, and colonizers that never founded** — diverted,
//!   contested, or destroyed — which is where an attempt goes when it is not a
//!   colony;
//! - **the transfer ratio** `Δneighbors / −Δown`: at 1.0 the card is a pure
//!   handover and every world it costs seat 0 is taken by somebody else, below
//!   1.0 some of the loss is worlds nobody takes (real denial, or real waste),
//!   above 1.0 the card is actively feeding the table.
//!
//! The last one is the number this harness exists for. `AGENTS.md` §2: an
//! aggregate over a set the treatment re-selects has to report its mix, and
//! `W_0` is a difference of two such aggregates.

use hyades_engine::autopilot::BuildOrder;
use hyades_engine::autopilot::{Autopilot, BaselineAutopilot, ColonizerPolicy, Doctrine};
use hyades_engine::galaxy::{Galaxy, GalaxyConfig};
use hyades_engine::log::{LogCategory, LogEvent, LogFilter};
use hyades_engine::sim::{HullType, Role, SimConfig, Simulation};
use std::io::Write;

const PLAYERS: usize = 3;
const HORIZON: f64 = 800.0;
const SEEDS: [u64; 4] = [1, 7, 42, 31337];

#[derive(Clone, Copy, PartialEq, Debug)]
enum Arm {
    /// Nobody plays anything.
    Peace,
    /// The Design write alone: the General colonizer is a Contact hull. No
    /// Doctrine, no pickets, nobody shoots. **Expected inert** — the shipped
    /// `CheapestViable` policy never names a General hull at all, so this
    /// substitutes into a slot nothing selects.
    GcvOnly,
    /// `SettlersPerMineral` alone, which is what makes the General slot live.
    /// Not part of the card; here so the arm below is attributable.
    PerMineral,
    /// …and the Design write on top of it. **This is the first arm in which the
    /// card's Design half does anything at all.**
    PerMineralGcv,
    /// The Doctrine write alone: colonizers picket after founding, neutrals are
    /// enemies. This is T-112's card.
    DoctrineOnly,
    /// Doctrine + Design, which is the card as §8.2 specifies it — and still
    /// with the shipped policy, so its Design half is still inert.
    WholeCard,
    /// Doctrine + Design + the policy that makes the Design half reachable.
    /// This is the card as it would have to ship to mean anything.
    WholeCardLive,
}

struct Row {
    own: f64,
    nbr: f64,
    built_medium: u64,
    built_general: u64,
    founded: u64,
    diverted: u64,
    contested: u64,
}

fn run(seed: u64, arm: Arm) -> Row {
    let galaxy = Galaxy::generate(GalaxyConfig::new(PLAYERS, seed)).unwrap();
    let doctrine_on = matches!(arm, Arm::DoctrineOnly | Arm::WholeCard | Arm::WholeCardLive);
    let design_on = matches!(arm, Arm::GcvOnly | Arm::PerMineralGcv | Arm::WholeCard | Arm::WholeCardLive);
    let per_mineral = matches!(arm, Arm::PerMineral | Arm::PerMineralGcv | Arm::WholeCardLive);
    let autopilots: Vec<Box<dyn Autopilot>> = (0..PLAYERS)
        .map(|i| {
            let me = i == 0;
            let d = Doctrine {
                engage_neutrals: me && doctrine_on,
                picket_after_founding: me && doctrine_on,
                colonizer_general_contact: me && design_on,
                colonizer_policy: if me && per_mineral {
                    ColonizerPolicy::SettlersPerMineral
                } else {
                    Doctrine::default().colonizer_policy
                },
                ..Doctrine::default()
            };
            Box::new(BaselineAutopilot::new(d)) as Box<_>
        })
        .collect();
    let mut cfg = SimConfig::new(seed);
    cfg.horizon_years = HORIZON;
    let mut sim = Simulation::new(galaxy, cfg, autopilots);
    sim.set_log_filter(
        LogFilter::none().with(LogCategory::Production).with(LogCategory::Vehicles).with(LogCategory::Combat),
    );
    let report = sim.run();

    let (mut bm, mut bg, mut founded, mut diverted, mut contested) = (0u64, 0u64, 0u64, 0u64, 0u64);
    for r in sim.log().iter() {
        match r.event {
            LogEvent::BuildApplied { player: 0, order: BuildOrder::Hull { hull_type, .. }, .. } => match hull_type {
                HullType::MediumSystems => bm += 1,
                HullType::GeneralSystems | HullType::GeneralContactVehicle => bg += 1,
                _ => {}
            },
            LogEvent::ColonyFounded { player: 0, .. } => founded += 1,
            LogEvent::CourseChanged { player: 0, role: Role::Colonizer, .. } => diverted += 1,
            LogEvent::ColonyContested { player: 0, .. } => contested += 1,
            _ => {}
        }
    }
    // A Medium build is a colonizer only when it was tasked as one; the same
    // hull hauls freight. Counted off the spawn, which carries the role.
    let mut colonizers = 0u64;
    for r in sim.log().iter() {
        if let LogEvent::VehicleSpawned { player: 0, role: Role::Colonizer, .. } = r.event {
            colonizers += 1;
        }
    }
    let _ = colonizers;
    Row {
        own: report.players[0].colonies as f64,
        nbr: report.players[1..].iter().map(|p| p.colonies as f64).sum::<f64>() / (PLAYERS - 1) as f64,
        built_medium: bm,
        built_general: bg,
        founded,
        diverted,
        contested,
    }
}

fn main() {
    println!("peace baseline: {} seeds, {PLAYERS} seats, {HORIZON} yr…", SEEDS.len());
    let _ = std::io::stdout().flush();
    let peace: Vec<Row> = SEEDS
        .iter()
        .map(|&s| {
            let r = run(s, Arm::Peace);
            print!(".");
            let _ = std::io::stdout().flush();
            r
        })
        .collect();
    println!();
    println!(
        "{:>22}  {:>8}  {:>8}  {:>9}  {:>8}  {:>8}  {:>8}  {:>8}  {:>9}",
        "arm", "own", "nbr", "W_0", "MSV", "Gen", "founded", "lost", "transfer"
    );
    let _ = std::io::stdout().flush();

    for arm in
        [Arm::GcvOnly, Arm::PerMineral, Arm::PerMineralGcv, Arm::DoctrineOnly, Arm::WholeCard, Arm::WholeCardLive]
    {
        let (mut own, mut nbr, mut w) = (0.0, 0.0, 0.0);
        let (mut bm, mut bg, mut fo, mut lost) = (0u64, 0u64, 0u64, 0u64);
        for (i, &seed) in SEEDS.iter().enumerate() {
            let p = &peace[i];
            let c = run(seed, arm);
            own += c.own - p.own;
            nbr += c.nbr - p.nbr;
            w += (c.own - c.nbr) - (p.own - p.nbr);
            bm += c.built_medium;
            bg += c.built_general;
            fo += c.founded;
            lost += c.diverted + c.contested;
        }
        let n = SEEDS.len() as f64;
        // `transfer` is Δneighbors / −Δown: 1.0 means every colony the card
        // costs its own player is taken by somebody else.
        let transfer = if own < 0.0 { nbr / -own } else { f64::NAN };
        println!(
            "{:>22?}  {:>+8.1}  {:>+8.1}  {:>+9.1}  {bm:>8}  {bg:>8}  {fo:>8}  {lost:>8}  {transfer:>9.2}",
            arm,
            own / n,
            nbr / n,
            w / n
        );
        let _ = std::io::stdout().flush();
    }
    println!("\n  own/nbr/W_0 are per-seed means against peace, in colonies.");
    println!("  MSV/Gen are seat 0's hull builds; founded/lost are its colony ships.");
    println!("  transfer = neighbor gain / own loss. >=1.0 means the card hands worlds over.");
}
