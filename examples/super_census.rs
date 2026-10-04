//! **Is there demand for supers?** — the census for T-146 / R-M5: what the
//! refined books carry at each round barrier, against what forges make.
//!
//! Card-free, standard galaxy, 3 seats. Per seed:
//!
//! - **At each round barrier**, per refined book (Red, Green, Blue, apex) and
//!   summed over empires, kilotonnes: bid, asked from yards, asked from
//!   holdings away from a yard, and filled ([`Simulation::book_census`]).
//! - **Production over the run**, kilotonnes: basics mined; each super and
//!   apex made, and the precursors each drew (the `Synthesized` log).
//! - **Where the supers went**: drawn into apex, and held at the horizon.
//! - **Supply runs** (`Simulation::supply_stats`): runs from own forges and
//!   to rival forges, kilotonnes loaded and bought, `$` paid.
//! - **Freight**: hauler pickups (stops that loaded anything) at the worlds
//!   that are forges at the horizon and elsewhere, the refined kilotonnes
//!   loaded at each, and refined kilotonnes delivered to forges and to other
//!   worlds.
//!
//! A refined bid is a center's declined order's refined shortfall, or a
//! forge's want for the supers that complete a balanced set (galaxy §4.5,
//! R-MX18). Only a Design write bills an order in refined material, so in a
//! card-free game the first is zero and every refined bid is a forge's.
//!
//! `SC_TWINS=1` seeds every seat with twin Designs paid in supers, a third
//! each of Red, Green and Blue (`FleetSeeding::twin_bill`), so supers have a
//! final demand; the census then adds the hulls paid for in supers.
//!
//! `SC_CARD=<id>` plays that card on every seat at the first round barrier
//! that seat can afford it, through `apply_orders` (card 3 is the Inscrutable
//! Growth card, `growth_rate` × 1.6); the census then prints when each seat
//! played it and when its first forge forged.
//!
//! Run: `cargo run --release --example super_census -- [horizon]`;
//! `SC_SEEDS=2,3,5,11`; `SC_SITE_SPACING` and `SC_SITE_SIGMA` (ly) set the
//! color sites (§4.3) — the galaxy is the only thing a bed varies.
use hyades_engine::autopilot::BuildOrder;
use hyades_engine::cards::{self, CardId, Order, Target};

use hyades_engine::galaxy::{FleetSeeding, Galaxy, GalaxyConfig, PlayerId};
use hyades_engine::log::{FreighterLeg, LogCategory, LogEvent, LogFilter};
use hyades_engine::resources::Material;
use hyades_engine::sim::{DesignBill, SimConfig, Simulation, MATERIALS};
use hyades_engine::units::BandTier;
use hyades_engine::units::Price;
use std::io::Write;

const SEEDS: [u64; 4] = [1, 7, 42, 31337];
const SEATS: usize = 3;
/// `Material::ALL` indices of the refined books: Red, Green, Blue, apex.
const REFINED: [usize; 4] = [3, 4, 5, 6];
const NAMES: [&str; 4] = ["Red", "Green", "Blue", "apex"];

fn env_f64(name: &str) -> Option<f64> {
    std::env::var(name).ok().and_then(|v| v.trim().parse().ok())
}

fn seeds() -> Vec<u64> {
    match std::env::var("SC_SEEDS") {
        Ok(v) => v.split(',').filter_map(|s| s.trim().parse().ok()).collect(),
        Err(_) => SEEDS.to_vec(),
    }
}

fn galaxy_config(seed: u64) -> GalaxyConfig {
    let mut cfg = GalaxyConfig::new(SEATS, seed);
    if let Some(v) = env_f64("SC_SITE_SPACING") {
        cfg.color_site_spacing_ly = v;
    }
    if let Some(v) = env_f64("SC_SITE_SIGMA") {
        cfg.color_site_sigma_ly = v;
    }
    cfg
}

fn sum(rows: &[[f64; MATERIALS]], i: usize) -> f64 {
    rows.iter().map(|r| r[i]).sum()
}

fn main() {
    let horizon: f64 = std::env::args().nth(1).and_then(|a| a.parse().ok()).unwrap_or(1500.0);
    let probe = galaxy_config(1);
    let twins = std::env::var("SC_TWINS").is_ok_and(|v| v.trim() == "1");
    let seeding = FleetSeeding {
        twin_bill: twins.then(|| DesignBill::coerced([1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0, 0.0])),
        ..Default::default()
    };
    println!(
        "super_census: {SEATS} seats, horizon {horizon} yr, seeds {:?}, color sites {} ly apart, width {} ly, twins {twins}",
        seeds(),
        probe.color_site_spacing_ly,
        probe.color_site_sigma_ly
    );
    std::io::stdout().flush().ok();
    for seed in seeds() {
        let mut cfg = SimConfig::new(seed);
        cfg.horizon_years = horizon;
        let every = cfg.years_per_round;
        let cfg_first_round = cfg.years_to_first_round;
        let mut next = cfg.years_to_first_round + 1e-6;
        let galaxy = Galaxy::generate_with(galaxy_config(seed), seeding.clone()).unwrap();
        let mut sim = Simulation::with_baseline(galaxy, cfg);
        sim.set_log_filter(LogFilter::none().with(LogCategory::Production).with(LogCategory::Mining));
        println!(
            "{seed:>5}  barrier  book    bid_kt  ask_yard_kt  ask_away_kt  filled_kt  bidding_empires  orders_want_kt  at_forges_kt"
        );
        let card: Option<CardId> = std::env::var("SC_CARD").ok().and_then(|v| v.trim().parse().ok()).map(CardId);
        let first_round = cfg_first_round;
        let mut played = [f64::NAN; SEATS];
        while sim.step() {
            if let Some(id) = card {
                for (i, slot) in played.iter_mut().enumerate() {
                    if !slot.is_nan() || sim.clock() < first_round {
                        continue;
                    }
                    let Some(c) = cards::card(id) else { continue };
                    if !sim.empire_can_afford(i, Price::new(c.cost)) {
                        continue;
                    }
                    sim.apply_orders(
                        sim.current_round(),
                        &[Order { seat: PlayerId(i as u32), card: Some(id), target: Target::None }],
                    );
                    *slot = sim.clock();
                }
            }
            if sim.clock() < next {
                continue;
            }
            next += every;
            let c = sim.book_census();
            for (k, &i) in REFINED.iter().enumerate() {
                let bidders = c.bid.iter().filter(|r| r[i] > 1e-9).count();
                println!(
                    "{seed:>5}  {:>7.0}  {:<5} {:>9.2} {:>12.2} {:>12.2} {:>10.2} {:>16} {:>15.2} {:>13.2}",
                    sim.clock(),
                    NAMES[k],
                    sum(&c.bid, i),
                    sum(&c.ask_yard, i),
                    sum(&c.ask_away, i),
                    sum(&c.filled, i),
                    bidders,
                    sum(&c.wanted, i),
                    sum(&c.at_forges, i),
                );
            }
        }
        // Production, from the log.
        let (mut mined, mut made, mut drew) = (0.0f64, [0.0f64; 4], [0.0f64; 4]);
        let (mut hulls, mut super_hulls, mut paid, mut hull_kt) = (0usize, 0usize, 0.0f64, 0.0f64);
        // Forges at the horizon: population Band IV, owned.
        let snap = sim.snapshot();
        let forge: Vec<bool> = snap.planets.iter().map(|p| p.owner.is_some() && p.pop_level == BandTier::IV).collect();
        // [at a forge, elsewhere]: pickups, refined loaded kt, refined delivered kt.
        let (mut stops, mut loaded, mut delivered) = ([0usize; 2], [0.0f64; 2], [0.0f64; 2]);
        let mut forge_haulers = std::collections::BTreeSet::new();
        // Hull orders per century: count, kt, kt paid in supers.
        let mut by_century = vec![(0usize, 0.0f64, 0.0f64); (horizon / 100.0).ceil() as usize + 1];
        for r in sim.log().iter() {
            match r.event {
                LogEvent::MineralsExtracted { amount, .. } => mined += amount,
                LogEvent::FreighterTransfer { leg, refined, at, vehicle, .. } => {
                    let k = usize::from(!forge[at.0 as usize]);
                    match leg {
                        FreighterLeg::Loaded => {
                            stops[k] += 1;
                            loaded[k] += refined;
                            if k == 0 {
                                forge_haulers.insert(vehicle.0);
                            }
                        }
                        FreighterLeg::Deposited => delivered[k] += refined,
                    }
                }
                LogEvent::BuildApplied { order: BuildOrder::Hull { .. }, cost, refined_paid, .. } => {
                    let b = &mut by_century[(r.time / 100.0) as usize];
                    b.0 += 1;
                    b.1 += cost;
                    b.2 += refined_paid;
                    hulls += 1;
                    hull_kt += cost;
                    paid += refined_paid;
                    super_hulls += usize::from(refined_paid > 0.0);
                }
                LogEvent::Synthesized { material, made: m, used, .. } => {
                    if let Some(k) = Material::REFINED.iter().position(|&x| x == material) {
                        made[k] += m;
                        drew[k] += used;
                    }
                }
                _ => {}
            }
        }
        let c = sim.book_census();
        let held: Vec<f64> = REFINED.iter().map(|&i| sum(&c.banked, i) + sum(&c.held_away, i)).collect();
        // Yards (owned, not forges) holding supers: how many of the three each holds.
        let mut colors_held = [0usize; 4];
        let mut kt_by_colors = [0.0f64; 4];
        for (i, p) in snap.planets.iter().enumerate() {
            if p.owner.is_none() || forge[i] {
                continue;
            }
            let s = p.stockpile;
            let n = [s.red, s.green, s.blue].iter().filter(|&&x| x > 1e-9).count();
            if n > 0 {
                colors_held[n] += 1;
                kt_by_colors[n] += s.red + s.green + s.blue;
            }
        }
        println!(
            "{seed:>5}  yards holding supers, by how many of the three: one {} ({:.2} kt), two {} ({:.2} kt), three {} ({:.2} kt)",
            colors_held[1], kt_by_colors[1], colors_held[2], kt_by_colors[2], colors_held[3], kt_by_colors[3]
        );
        let at_yards: Vec<f64> = REFINED.iter().map(|&i| sum(&c.banked, i) - sum(&c.at_forges, i)).collect();
        let want: Vec<f64> = REFINED.iter().map(|&i| sum(&c.wanted, i)).collect();
        println!(
            "{seed:>5}  at the horizon, at yards that are not forges kt: Red {:.2}, Green {:.2}, Blue {:.2}, apex {:.2}; orders want Red {:.2}, Green {:.2}, Blue {:.2}",
            at_yards[0], at_yards[1], at_yards[2], at_yards[3], want[0], want[1], want[2]
        );
        let traded = sim.exchange_traded();
        println!(
            "{seed:>5}  production kt: basics mined {mined:.0}; Red {:.2}, Green {:.2}, Blue {:.2}, apex {:.2}",
            made[0], made[1], made[2], made[3]
        );
        println!(
            "{seed:>5}  freight: forges {}; pickups at forges {} of {}, by {} haulers; refined loaded kt at forges {:.2}, elsewhere {:.2}; refined delivered kt to forges {:.2}, to other worlds {:.2}",
            forge.iter().filter(|&&f| f).count(),
            stops[0],
            stops[0] + stops[1],
            forge_haulers.len(),
            loaded[0],
            loaded[1],
            delivered[0],
            delivered[1]
        );
        let rows: Vec<String> = by_century
            .iter()
            .enumerate()
            .filter(|(_, b)| b.0 > 0)
            .map(|(c, b)| format!("{}: {} ({:.1} kt, {:.2} in supers)", c * 100, b.0, b.1, b.2))
            .collect();
        println!("{seed:>5}  hull orders by century: {}", rows.join("; "));
        let mut first_forge = [f64::NAN; SEATS];
        for r in sim.log().iter() {
            if let LogEvent::Synthesized { player, .. } = r.event {
                let p = player as usize;
                if first_forge[p].is_nan() {
                    first_forge[p] = r.time;
                }
            }
        }
        println!(
            "{seed:>5}  card played at {:?} yr; first forge at {:?} yr",
            played.map(|t| t.round()),
            first_forge.map(|t| t.round())
        );
        let st = sim.supply_stats();
        println!(
            "{seed:>5}  supply runs: from own forges {} ({:.2} kt loaded), to rival forges {} ({:.2} kt bought, {:.2} $ paid)",
            st.runs_own, st.loaded_own, st.runs_rival, st.bought_rival, st.paid
        );
        println!("{seed:>5}  hull orders built {hulls} ({hull_kt:.2} kt), paid in supers {super_hulls} ({paid:.2} kt)");
        println!(
            "{seed:>5}  precursors drawn kt: into supers {:.2} basics; into apex {:.2} supers",
            drew[0] + drew[1] + drew[2],
            drew[3]
        );
        println!(
            "{seed:>5}  held at the horizon kt: Red {:.2}, Green {:.2}, Blue {:.2}, apex {:.2} | delivered between empires kt: Red {:.2}, Green {:.2}, Blue {:.2}, apex {:.2}",
            held[0], held[1], held[2], held[3], traded[3], traded[4], traded[5], traded[6]
        );
        std::io::stdout().flush().ok();
    }
}
