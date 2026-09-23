//! **Where does a card's effect go?** (T-122) — a paired, asymmetric probe for
//! finding the constraint between a card and its own tree's metric.
//!
//! `examples/card_table` answers *how much* a card is worth on the shared
//! twelve-seat table, and at three independent seeds it could not separate
//! either first card from zero (§8.14). This harness answers the prior
//! question — *what is throttling it* — and is built for that:
//!
//! - **Asymmetric** (R-TREE8/R-TREE12). Seat 0 plays the card; every other seat
//!   passes. The counterfactual is the same seed with seat 0 passing too, so
//!   each seed yields one **paired** difference and the seeds are independent
//!   replicates. On the twelve-seat table six seats shared one galaxy and the
//!   standard error counted them as six.
//! - **Arms are ablations.** Each arm is the card plus a change to the engine
//!   or the Doctrine; an arm that moves the metric names a constraint, and one
//!   that does not refutes it.
//! - **It prints the mechanism beside the metric.** Population, infrastructure,
//!   colony count and combat are read on every arm, so a flat objective can be
//!   read as "the lever moved and nothing downstream listened" or "the lever
//!   never moved" — which want opposite next steps.
//!
//! ```text
//! cargo run --release --example card_probe -- growth            # every Growth arm
//! cargo run --release --example card_probe -- warfare           # every Warfare arm
//! cargo run --release --example card_probe -- growth pop-output # arms whose label matches
//! ```
//!
//! **The metrics, and their units.**
//!
//! | name | meaning | unit |
//! |---|---|---|
//! | `ΔlnG` | `ln(G_card / G_pass)`, `G = ∫ infra dt` for seat 0 — Growth's work-years interim (R-TREE3) | dimensionless |
//! | `ΔW` | `W_card − W_pass`, `W = ∫ [C_0 − mean_{j≠0} C_j] dt` — Warfare's objective with `w_ij` uniform (R-WAR3 open) | colony-years |
//! | `Δln pop` | `ln(pop_card / pop_pass)` for seat 0 at the horizon | dimensionless |
//! | `t` | mean / standard error over independent seeds | — |
//!
//! "Substantial" in T-122 is `|t| ≥ 3`. The doubling-time criterion ("double
//! the rate") is reported too, as the ratio of fitted growth rates of seat 0's
//! stock, card over pass.

use hyades_engine::autopilot::{Autopilot, BaselineAutopilot, Doctrine};
use hyades_engine::cards::{self, CardId, Order, Target};
use hyades_engine::galaxy::{Galaxy, GalaxyConfig, PlayerId};
use hyades_engine::log::{LogCategory, LogEvent, LogFilter};
use hyades_engine::sim::{Role, SimConfig, Simulation};
use std::io::Write;

const SEATS: usize = 3;
/// Eight independent replicates — the standard four and the replication four
/// (`CLAUDE.md` §2: a set the candidate was not chosen against).
const SEEDS: [u64; 8] = [1, 7, 42, 31337, 2, 3, 5, 11];
/// 600 years of play after the round-0 barrier at 200 yr.
const HORIZON: f64 = 800.0;
const SAMPLE_YEARS: f64 = 10.0;

const GROWTH_CARD: u16 = 3;
const WARFARE_CARD: u16 = 15;

/// What an arm changes, applied to both halves of the pair except `card_only`,
/// which lands on the card half alone.
#[derive(Clone, Copy)]
struct Arm {
    label: &'static str,
    /// The card seat 0 plays on the card half, or `None` for an arm that is
    /// only a Doctrine write — how one write of a bundle is ablated alone.
    card: Option<u16>,
    /// Engine change applied to *both* halves — a different engine, same card
    /// question. This is how a constraint in the engine is removed.
    engine: fn(&mut SimConfig),
    /// Extra Doctrine applied to seat 0 on the *card* half only — how a card
    /// is widened to test whether a missing write is what binds.
    card_only: fn(&mut Doctrine),
}

fn no_engine(_: &mut SimConfig) {}
fn no_doctrine(_: &mut Doctrine) {}

struct Sample {
    t: Vec<f64>,
    infra0: Vec<f64>,
    colonies: Vec<Vec<f64>>,
    pop0: f64,
    /// Stock-weighted staffing factor for seat 0, time-averaged over the run.
    u0: f64,
    /// Seat 0's idle stock at the horizon on worlds still growing, and on
    /// worlds at their population ceiling — as fractions of standing stock.
    idle_growing: f64,
    idle_capped: f64,
    kills_by_0: u64,
    losses_of_0: u64,
    engagements_0: u64,
    /// Denial, counted where it happens rather than inferred from colonies:
    /// pickets seat 0 parked, enemy colony ships its holdings turned back, and
    /// interceptions seat 0's pickets launched.
    pickets_0: u64,
    diverted_by_0: u64,
    intercepts_0: u64,
}

fn run(seed: u64, arm: &Arm, play: bool) -> Sample {
    let galaxy = Galaxy::generate(GalaxyConfig::new(SEATS, seed)).unwrap();
    // `card_only` widens the card by seeding seat 0's Doctrine at bootstrap,
    // on the card half alone — the same route every census here uses, so no
    // inbound channel beside `apply_orders` is opened (design law #15).
    let autopilots: Vec<Box<dyn Autopilot>> = (0..SEATS)
        .map(|i| {
            let mut d = Doctrine::default();
            if play && i == 0 {
                (arm.card_only)(&mut d);
            }
            Box::new(BaselineAutopilot::new(d)) as Box<_>
        })
        .collect();
    let mut cfg = SimConfig::new(seed);
    cfg.horizon_years = HORIZON;
    cfg.engagements_enabled = true;
    (arm.engine)(&mut cfg);
    let play_at = cfg.years_to_first_round;
    let mut sim = Simulation::new(galaxy, cfg, autopilots);
    sim.set_log_filter(LogFilter::none().with(LogCategory::Combat).with(LogCategory::Vehicles));

    // **Cards are played at the round-0 barrier, not at `t = 0`.** The
    // opening is card-free by protocol (`Hyades_netcode.md` §1,
    // `SimConfig::years_to_first_round`), so round 0 *is* earliest legal play
    // (trees §2.4). Playing at `t = 0` — which this harness and
    // `examples/card_table` both did before T-122 — spends the price out of
    // the bootstrap bank, a state no real game reaches, and on seed 31337
    // that alone cost ~550 colonies.
    let mut played = !play || arm.card.is_none();

    let mut s = Sample {
        t: Vec::new(),
        infra0: Vec::new(),
        colonies: vec![Vec::new(); SEATS],
        pop0: 0.0,
        u0: 0.0,
        idle_growing: 0.0,
        idle_capped: 0.0,
        kills_by_0: 0,
        losses_of_0: 0,
        engagements_0: 0,
        pickets_0: 0,
        diverted_by_0: 0,
        intercepts_0: 0,
    };
    let mut next = 0.0;
    let (mut u_sum, mut u_n) = (0.0, 0usize);
    let sample = |sim: &Simulation, s: &mut Sample| {
        s.t.push(sim.clock());
        s.infra0.push(sim.tree_stock(0).1);
        for i in 0..SEATS {
            s.colonies[i].push(sim.tree_stock(i).0 as f64);
        }
    };
    while sim.step() {
        if !played && sim.clock() >= play_at {
            let card = arm.card.expect("guarded by `played`");
            sim.apply_orders(
                sim.current_round(),
                &[Order { seat: PlayerId(0), card: Some(CardId(card)), target: Target::None }],
            );
            played = true;
        }
        if sim.clock() >= next {
            next = sim.clock() + SAMPLE_YEARS;
            sample(&sim, &mut s);
            let (w, g, c) = sim.staffing_split(0);
            if w + g + c > 0.0 {
                u_sum += w / (w + g + c);
                u_n += 1;
            }
        }
    }
    s.u0 = u_sum / u_n.max(1) as f64;
    let (w, g, c) = sim.staffing_split(0);
    let total = (w + g + c).max(1e-12);
    s.idle_growing = g / total;
    s.idle_capped = c / total;
    sample(&sim, &mut s);
    s.pop0 = sim.report().players[0].total_population.kilotons();
    for r in sim.log().iter() {
        match r.event {
            LogEvent::VehicleParked { player: 0, role: Role::Picket, .. } => s.pickets_0 += 1,
            LogEvent::ColonyDiverted { holder: 0, .. } => s.diverted_by_0 += 1,
            LogEvent::PicketIntercept { player: 0, .. } => s.intercepts_0 += 1,
            _ => {}
        }
        if let LogEvent::EngagementResolved { attacker, defender, losses_attacker, losses_defender, .. } = r.event {
            if attacker == 0 || defender == 0 {
                s.engagements_0 += 1;
            }
            if attacker == 0 {
                s.kills_by_0 += losses_defender as u64;
                s.losses_of_0 += losses_attacker as u64;
            } else if defender == 0 {
                s.kills_by_0 += losses_attacker as u64;
                s.losses_of_0 += losses_defender as u64;
            }
        }
    }
    s
}

/// Trapezoidal `∫ x dt` over the sampled timeline.
fn integral(t: &[f64], x: &[f64]) -> f64 {
    t.windows(2).zip(x.windows(2)).map(|(tw, xw)| 0.5 * (xw[0] + xw[1]) * (tw[1] - tw[0])).sum()
}

/// Least-squares growth rate of `ln x` over `[start, HORIZON]`.
fn growth_rate(t: &[f64], x: &[f64], start: f64) -> Option<f64> {
    let pts: Vec<(f64, f64)> =
        t.iter().zip(x).filter(|&(&ti, &xi)| ti >= start && xi > 0.0).map(|(&ti, &xi)| (ti, xi.ln())).collect();
    if pts.len() < 3 {
        return None;
    }
    let n = pts.len() as f64;
    let mt = pts.iter().map(|p| p.0).sum::<f64>() / n;
    let my = pts.iter().map(|p| p.1).sum::<f64>() / n;
    let sxx: f64 = pts.iter().map(|p| (p.0 - mt).powi(2)).sum();
    let sxy: f64 = pts.iter().map(|p| (p.0 - mt) * (p.1 - my)).sum();
    (sxx > 0.0).then(|| sxy / sxx)
}

/// Warfare's contrast for seat 0 at each sample, `w` uniform over the others.
fn contrast0(s: &Sample) -> Vec<f64> {
    (0..s.t.len())
        .map(|k| {
            let others: f64 = (1..SEATS).map(|j| s.colonies[j][k]).sum();
            s.colonies[0][k] - others / (SEATS - 1) as f64
        })
        .collect()
}

fn stat(v: &[f64]) -> (f64, f64) {
    let n = v.len() as f64;
    let m = v.iter().sum::<f64>() / n;
    let var = v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (n - 1.0).max(1.0);
    (m, (var / n).sqrt())
}

fn fmt(v: &[f64]) -> String {
    let (m, se) = stat(v);
    let t = if se > 0.0 { m / se } else { f64::INFINITY * m.signum() };
    format!("{m:+9.4} ± {se:7.4} (t {t:+6.2})")
}

fn scout_write(d: &mut Doctrine) {
    d.scout_hull_offensive = true;
}
fn colonizer_write(d: &mut Doctrine) {
    d.colonizer_general_contact = true;
}
fn hostility(d: &mut Doctrine) {
    d.engage_neutrals = true;
}

fn staffed(c: &mut SimConfig) {
    c.population_staffs_industry = true;
}
fn denial(d: &mut Doctrine) {
    d.engage_neutrals = true;
    d.picket_reserve = 8;
    d.picket_intercepts = true;
}
/// `TIER0[12]` is `UnlockDesign(MediumSystems)` at 0.5 kt — the Warfare card's
/// price exactly, and inert while `enforce_roster` is off. A pure-price control.
const PRICE_ONLY_CARD: u16 = 12;

fn no_conjunction(c: &mut SimConfig) {
    c.ablate_color_conjunction = true;
}
fn staffed_no_conjunction(c: &mut SimConfig) {
    c.population_staffs_industry = true;
    c.ablate_color_conjunction = true;
}
/// `TIER0[3]`'s write with no card played — the lever without its price.
fn growth_write(d: &mut Doctrine) {
    d.growth_rate *= 1.15;
}

/// Denial through the fight that already favors the holder — pickets on held
/// ground, racing colony ships there — with **no** hostility at rocks, whose
/// miner-suicide (R-WAR5) swamped the `+ denial` arm.
fn pickets_only(d: &mut Doctrine) {
    d.picket_reserve = 8;
    d.picket_intercepts = true;
}

/// The posture that already wins (T-113/T-115/T-120): hold ground, race colony
/// ships to it, and let a picket claim the world its center is saving for —
/// which is the one branch that may go ahead of survey (§8.8).
fn hold_ground(d: &mut Doctrine) {
    d.picket_reserve = 8;
    d.picket_intercepts = true;
    d.picket_claims_target = true;
}
fn hold_ground_wide(d: &mut Doctrine) {
    hold_ground(d);
    d.picket_reserve = 32;
}

/// **Strike colony ships at the rival's port** (T-123, R-WAR16): pickets go to
/// the rival center this empire has seen launch the most, and meet what leaves
/// it at range zero. Supply is the hold-ground posture's.
fn blockade(d: &mut Doctrine) {
    hold_ground(d);
    d.picket_intercepts = false;
    d.picket_blockades = true;
}
/// The same, on the fallback supply alone — no claim-the-target branch — which
/// T-113 measured as almost never firing.
fn blockade_fallback(d: &mut Doctrine) {
    blockade(d);
    d.picket_claims_target = false;
}
fn blockade_wide(d: &mut Doctrine) {
    blockade(d);
    d.picket_reserve = 32;
}

fn oracle(c: &mut SimConfig) {
    c.ablate_oracle_intercept = true;
}

fn arms() -> Vec<Arm> {
    vec![
        Arm { label: "warfare / blockade", card: Some(WARFARE_CARD), engine: no_engine, card_only: blockade },
        Arm {
            label: "warfare / blockade, fallback supply",
            card: Some(WARFARE_CARD),
            engine: no_engine,
            card_only: blockade_fallback,
        },
        Arm {
            label: "warfare / blockade, wide",
            card: Some(WARFARE_CARD),
            engine: no_engine,
            card_only: blockade_wide,
        },
        Arm {
            label: "warfare / hold ground, oracle",
            card: Some(WARFARE_CARD),
            engine: oracle,
            card_only: hold_ground,
        },
        Arm { label: "warfare / hold ground", card: Some(WARFARE_CARD), engine: no_engine, card_only: hold_ground },
        Arm {
            label: "warfare / hold ground, wide",
            card: Some(WARFARE_CARD),
            engine: no_engine,
            card_only: hold_ground_wide,
        },
        Arm {
            label: "warfare / pickets, no hostility",
            card: Some(WARFARE_CARD),
            engine: no_engine,
            card_only: pickets_only,
        },
        Arm { label: "growth / write alone, staffed", card: None, engine: staffed, card_only: growth_write },
        Arm {
            label: "growth / staffed, no conjunction",
            card: Some(GROWTH_CARD),
            engine: staffed_no_conjunction,
            card_only: no_doctrine,
        },
        Arm {
            label: "growth / write alone, staffed, no conjunction",
            card: None,
            engine: staffed_no_conjunction,
            card_only: growth_write,
        },
        Arm {
            label: "growth / no conjunction",
            card: Some(GROWTH_CARD),
            engine: no_conjunction,
            card_only: no_doctrine,
        },
        // T-107: the engine changes on both halves, the card question does not.
        Arm { label: "growth / staffed", card: Some(GROWTH_CARD), engine: staffed, card_only: no_doctrine },
        Arm { label: "warfare / price alone", card: Some(PRICE_ONLY_CARD), engine: no_engine, card_only: no_doctrine },
        Arm { label: "warfare / + denial", card: Some(WARFARE_CARD), engine: no_engine, card_only: denial },
        Arm { label: "growth / as shipped", card: Some(GROWTH_CARD), engine: no_engine, card_only: no_doctrine },
        Arm { label: "warfare / as shipped", card: Some(WARFARE_CARD), engine: no_engine, card_only: no_doctrine },
        // The card's Doctrine write has two halves; each alone, with no card
        // played and so no price paid and no Design unlocked.
        Arm { label: "warfare / scout write alone", card: None, engine: no_engine, card_only: scout_write },
        Arm { label: "warfare / colonizer write alone", card: None, engine: no_engine, card_only: colonizer_write },
        // §8.2's "a Neutral empire is an Enemy empire" — specified, and not in
        // the shipped card.
        Arm { label: "warfare / + hostility", card: Some(WARFARE_CARD), engine: no_engine, card_only: hostility },
    ]
}

fn main() {
    let filters: Vec<String> = std::env::args().skip(1).collect();
    let chosen: Vec<Arm> =
        arms().into_iter().filter(|a| filters.iter().all(|f| a.label.contains(f.as_str()))).collect();
    assert!(!chosen.is_empty(), "no arm matched {filters:?}");
    println!(
        "{SEATS} seats, seat 0 plays at the round-0 barrier ({} yr), {} independent seeds, {HORIZON} yr",
        SimConfig::new(0).years_to_first_round,
        SEEDS.len()
    );
    let _ = std::io::stdout().flush();

    for arm in &chosen {
        println!("\n== {} — card {:?}", arm.label, arm.card.and_then(|c| cards::card(CardId(c))).map(|c| c.effects));
        let _ = std::io::stdout().flush();
        let (mut dlng, mut dw, mut dpop, mut rate_ratio_g, mut dcol) = (vec![], vec![], vec![], vec![], vec![]);
        let (mut kills, mut losses, mut engs) = (vec![], vec![], vec![]);
        let (mut u_pass, mut u_card) = (vec![], vec![]);
        let (mut pk, mut dv, mut ic) = (vec![], vec![], vec![]);
        let (mut idle_g, mut idle_c) = (vec![], vec![]);
        let mut drival = vec![];
        for &seed in &SEEDS {
            let p = run(seed, arm, false);
            let c = run(seed, arm, true);
            let (gp, gc) = (integral(&p.t, &p.infra0), integral(&c.t, &c.infra0));
            dlng.push((gc / gp).ln());
            dw.push(integral(&c.t, &contrast0(&c)) - integral(&p.t, &contrast0(&p)));
            dpop.push((c.pop0 / p.pop0).ln());
            dcol.push(c.colonies[0].last().unwrap() - p.colonies[0].last().unwrap());
            let rivals = |s: &Sample| (1..SEATS).map(|i| *s.colonies[i].last().unwrap()).sum::<f64>();
            drival.push(rivals(&c) - rivals(&p));
            let start = SimConfig::new(seed).years_to_first_round;
            if let (Some(a), Some(b)) = (growth_rate(&c.t, &c.infra0, start), growth_rate(&p.t, &p.infra0, start)) {
                rate_ratio_g.push(a / b - 1.0);
            }
            pk.push(c.pickets_0 as f64);
            dv.push(c.diverted_by_0 as f64);
            ic.push(c.intercepts_0 as f64);
            u_pass.push(p.u0);
            idle_g.push(p.idle_growing);
            idle_c.push(p.idle_capped);
            u_card.push(c.u0);
            kills.push(c.kills_by_0 as f64 - p.kills_by_0 as f64);
            losses.push(c.losses_of_0 as f64 - p.losses_of_0 as f64);
            engs.push(c.engagements_0 as f64 - p.engagements_0 as f64);
            println!(
                "  seed {seed:>5}: ΔlnG {:+.4}  ΔW {:+9.1}  Δln pop {:+.4}  Δcolonies {:+5.0}  Δkills {:+4.0}  Δlosses {:+4.0}",
                dlng.last().unwrap(),
                dw.last().unwrap(),
                dpop.last().unwrap(),
                dcol.last().unwrap(),
                kills.last().unwrap(),
                losses.last().unwrap(),
            );
            let _ = std::io::stdout().flush();
        }
        println!("  ΔlnG (work-years)      {}", fmt(&dlng));
        println!("  ΔW   (colony-years)    {}", fmt(&dw));
        println!("  Δln pop                {}", fmt(&dpop));
        println!("  Δcolonies at horizon   {}", fmt(&dcol));
        println!("  Δrival colonies        {}  (seats 1.., summed, at horizon)", fmt(&drival));
        if !rate_ratio_g.is_empty() {
            println!("  g_card / g_pass − 1     {}  n={}", fmt(&rate_ratio_g), rate_ratio_g.len());
        }
        println!("  staffing u, pass       {}", fmt(&u_pass));
        println!("  staffing u, card       {}", fmt(&u_card));
        println!("  idle stock, growing    {}  (share of standing, pass arm, at horizon)", fmt(&idle_g));
        println!("  idle stock, at K       {}  (share of standing, pass arm, at horizon)", fmt(&idle_c));
        println!("  pickets parked, card   {}", fmt(&pk));
        println!("  ships diverted, card   {}", fmt(&dv));
        println!("  intercepts, card       {}", fmt(&ic));
        println!("  Δkills by seat 0       {}", fmt(&kills));
        println!("  Δlosses of seat 0      {}", fmt(&losses));
        println!("  Δengagements seat 0    {}", fmt(&engs));
        let _ = std::io::stdout().flush();
    }
}
