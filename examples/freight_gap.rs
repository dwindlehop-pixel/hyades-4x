//! **Is the colour a centre needs anywhere its own empire can reach?**
//!
//! R-O85 closed with the constraint named but not located: 43.8–46.6% of every
//! production decision is a centre that holds the *total* of its next rung and
//! lacks a *colour* (§6.19c). That is either a **freight** problem — the colour
//! exists and is not moving — or a **trade** problem, the empire's ground simply
//! not holding it. Those want completely different fixes, and nothing has
//! separated them.
//!
//! So this puts the empire's colour deficit beside its own supply, per colour:
//!
//! - **deficit** — `Σ over owned centres` of the shortfall against the next rung.
//! - **at outposts** — mined, sitting in `outpost_stock`, waiting on a hull.
//! - **in banks** — already delivered to some centre, just not the one short of it.
//!
//! If supply covers deficit many times over, the ore exists and freight is the
//! constraint. If a colour is genuinely absent, that is §8.1's subject and the
//! Exchange's, not the hauler's.
//!
//! It also reports the freight fleet itself, because a routing fix is pointless
//! if the fleet is simply too small to move the tonnage: hold size against the
//! rung price is what says whether this is a *routing* or a *capacity* problem.
//!
//! And it reports the **work the fleet actually did**, decomposed, because a
//! freight change can move any one part independently and the total cannot tell
//! them apart. `total = population × rate`, so print both:
//!
//! - **trips** and **kt/trip** — what was moved.
//! - **laden / empty leg durations**, differenced off each hauler's own
//!   `FreighterTransfer` records, so a routing change shows up as the leg it
//!   actually lengthened rather than as a mean round trip.
//! - **active haulers** per window against **trips/hull** — the headcount and
//!   the rate. R-O89's rejected pickup-routing arm had a *shorter* round trip
//!   and an *unchanged* per-hull rate while the total halved, which is only
//!   legible with both columns present (`Hyades_industry.md` §6.20).
//! - **retired to Reserve** — whether hulls are leaving service.
//! - **extracted** — the check that the comparison is about hauling at all and
//!   not about the mine.
//!
//! Bucketed by time, because these are compounding runs and two arms can sit
//! inside noise for 500 years before they separate.
//!
//! Run: `cargo run --release --example freight_gap -- [seed] [horizon]`
use hyades_engine::log::{FreighterLeg, LogCategory, LogEvent, LogFilter};
use hyades_engine::prelude::*;
use hyades_engine::resources::Basic;
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;

const PLAYERS: usize = 3;

fn main() {
    let mut args = std::env::args().skip(1);
    let seed: u64 = args.next().and_then(|s| s.parse().ok()).unwrap_or(1);
    let horizon: f64 = args.next().and_then(|s| s.parse().ok()).unwrap_or(1500.0);

    println!("freight gap — seed {seed}, {PLAYERS} seats, {horizon:.0} yr");
    std::io::stdout().flush().ok();

    let galaxy = Galaxy::generate(GalaxyConfig::new(PLAYERS, seed)).unwrap();
    let mut cfg = SimConfig::new(seed);
    cfg.horizon_years = horizon;
    let mut sim = Simulation::with_baseline(galaxy, cfg);
    sim.set_log_filter(LogFilter::none().with(LogCategory::Mining).with(LogCategory::Vehicles));
    sim.run();

    let snap = sim.snapshot();
    println!(
        "\n{:>7} {:>9} {:>14} {:>14} {:>14} {:>12}",
        "player", "colour", "deficit kt", "at outposts", "in banks", "supply/need"
    );
    for p in 0..PLAYERS {
        let pid = PlayerId(p as u32);
        let deficit = sim.unmet_colour_demand(pid);
        let outposts = sim.outpost_holdings(pid);
        // Everything already delivered somewhere in this empire.
        let mut banks = [0.0f64; 3];
        for pl in snap.planets.iter().filter(|pl| pl.owner == Some(p as u32)) {
            for (i, &c) in Basic::ALL.iter().enumerate() {
                banks[i] += pl.stockpile.get_basic(c);
            }
        }
        for (i, &c) in Basic::ALL.iter().enumerate() {
            let need = deficit[i].kilotons();
            let out = outposts.get_basic(c);
            let ratio = if need > 1e-9 { (out + banks[i]) / need } else { f64::INFINITY };
            println!("{p:>7} {:>9?} {need:>14.1} {out:>14.1} {:>14.1} {ratio:>11.1}x", c, banks[i]);
        }
    }

    // Freight fleet, and the unit it has to move the tonnage in.
    let freighters = snap.vehicles.iter().filter(|v| v.kind == hyades_engine::snapshot::VehicleKind::Freighter).count();
    let miners = snap.vehicles.iter().filter(|v| v.kind == hyades_engine::snapshot::VehicleKind::Miner).count();
    let hold = HullType::MediumSystems.cargo_capacity(&SimConfig::new(seed)).kilotons();
    println!("\n  freighters {freighters}, miners {miners}, Medium hold {hold:.3} kt");
    println!(
        "  a General hull holds {:.3} kt for 10x the price — design law #3, unexploited by freight",
        HullType::GeneralSystems.cargo_capacity(&SimConfig::new(seed)).kilotons()
    );

    // **What the fleet actually moved**, off the log rather than inferred.
    let (mut trips, mut delivered, mut loads, mut loaded, mut extracted) = (0u64, 0.0f64, 0u64, 0.0f64, 0.0f64);
    for rec in sim.log().by_category(LogCategory::Mining) {
        match rec.event {
            LogEvent::FreighterTransfer { leg: FreighterLeg::Deposited, amount, .. } => {
                trips += 1;
                delivered += amount;
            }
            LogEvent::FreighterTransfer { leg: FreighterLeg::Loaded, amount, .. } => {
                loads += 1;
                loaded += amount;
            }
            LogEvent::MineralsExtracted { amount, .. } => extracted += amount,
            _ => {}
        }
    }
    let per_trip = if trips > 0 { delivered / trips as f64 } else { 0.0 };
    println!("\n  trips {trips}, delivered {delivered:.0} kt, {per_trip:.3} kt/trip (hold {hold:.3})");
    println!("  loads {loads}, loaded {loaded:.0} kt, extracted {extracted:.0} kt");

    // **Which leg got longer.** A round trip is `load → deposit → load`, and a
    // routing change can lengthen either half. Walk each hauler's own transfer
    // records in time order and difference them: `Loaded → Deposited` is the
    // laden delivery leg, `Deposited → Loaded` the empty pickup leg. Reading a
    // *mean round trip* alone would hide which one moved — the mix rule from
    // `CLAUDE.md` §2, applied to a duration.
    let mut legs: BTreeMap<Entity, (f64, FreighterLeg)> = BTreeMap::new();
    let (mut out_n, mut out_t, mut in_n, mut in_t) = (0u64, 0.0f64, 0u64, 0.0f64);
    for rec in sim.log().by_category(LogCategory::Mining) {
        let LogEvent::FreighterTransfer { vehicle, leg, .. } = rec.event else { continue };
        if let Some(&(t0, prev)) = legs.get(&vehicle) {
            match (prev, leg) {
                (FreighterLeg::Loaded, FreighterLeg::Deposited) => {
                    in_n += 1;
                    in_t += rec.time - t0;
                }
                (FreighterLeg::Deposited, FreighterLeg::Loaded) => {
                    out_n += 1;
                    out_t += rec.time - t0;
                }
                _ => {}
            }
        }
        legs.insert(vehicle, (rec.time, leg));
    }
    let mean = |n: u64, t: f64| if n > 0 { t / n as f64 } else { 0.0 };
    println!(
        "  laden leg {:.2} yr ({in_n}), empty pickup leg {:.2} yr ({out_n}), round trip {:.2} yr",
        mean(in_n, in_t),
        mean(out_n, out_t),
        mean(in_n, in_t) + mean(out_n, out_t)
    );

    // **When it diverged.** Trip count is a rate integrated over the run, and a
    // change that halves it either halved the rate throughout or throttled the
    // ramp early and compounded. Those are different defects. Bucketing says
    // which without a second instrument.
    let bucket = 250.0;
    let nb = (horizon / bucket).ceil() as usize;
    let mut per = vec![(0u64, 0.0f64, BTreeSet::new()); nb.max(1)];
    for rec in sim.log().by_category(LogCategory::Mining) {
        if let LogEvent::FreighterTransfer { leg: FreighterLeg::Deposited, amount, vehicle, .. } = rec.event {
            let b = ((rec.time / bucket) as usize).min(per.len() - 1);
            per[b].0 += 1;
            per[b].1 += amount;
            per[b].2.insert(vehicle);
        }
    }
    // `active` is the hauler count that *did work* in the window, so
    // `trips / active` is the per-hauler rate. Together they decompose a fall in
    // trips into "fewer hulls" and "each hull slower" — which want opposite
    // fixes, and which the total cannot tell apart.
    // **`Role::Reserve`, not `Role::Freighter`.** `release_to_reserve` re-roles
    // the hull *before* logging, so a filter on the old role counts zero and
    // reads as "nothing ever retires" — which is exactly what it said on the
    // first attempt. Miners and haulers both land here and the log cannot tell
    // them apart, so this is the pair count, not the hauler count.
    let mut retired = vec![0u64; nb.max(1)];
    for rec in sim.log().by_category(LogCategory::Vehicles) {
        if let LogEvent::VehicleParked { role: Role::Reserve, .. } = rec.event {
            retired[((rec.time / bucket) as usize).min(nb - 1)] += 1;
        }
    }
    println!("\n  {:>12} {:>8} {:>12} {:>8} {:>10} {:>8}", "years", "trips", "kt", "active", "trips/hull", "retired");
    for (i, (n, kt, act)) in per.iter().enumerate() {
        let a = act.len();
        let rate = if a > 0 { *n as f64 / a as f64 } else { 0.0 };
        println!(
            "  {:>5.0}-{:<6.0} {n:>8} {kt:>12.0} {a:>8} {rate:>10.2} {:>8}",
            i as f64 * bucket,
            (i + 1) as f64 * bucket,
            retired[i]
        );
    }
}
