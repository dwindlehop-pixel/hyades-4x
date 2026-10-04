//! **How far apart do empires finish, tree by tree?** — the census behind
//! T-147's second question: the variation across empires in each tree's stock
//! and in supers forged.
//!
//! Card-free. Per seat, integrated over the run on a fixed time grid (the
//! trapezoid `examples/tree_gradient` and `examples/work_years` use, so the
//! numbers compose):
//!
//! | metric | stock | unit |
//! |---|---|---|
//! | Expansion | `∫ C dt`, colonies owned | colony-years |
//! | Growth | `∫ V dt`, works standing | kt-years |
//! | Production | `∫ F dt`, fleet volume (`VehicleSnapshot::volume`) | hull-units³-years |
//!
//! and the supers and apex each seat has forged (`LogEvent::Synthesized`,
//! kt made). The spread between the seats of one galaxy is reported as the
//! **coefficient of variation** — standard deviation over mean — so metrics in
//! different units compare; then its mean over galaxies.
//!
//! Environment: `ES_SEEDS`, `ES_GROUND` (`random`, `identical`, `rotated`;
//! default `random`), `ES_HOMEWORLDS` (`trio`, `centered`; default `trio`),
//! `ES_TRACE=1` (each century, per seat: works in total, on its largest world,
//! and whether that world is its homeworld; and every infrastructure purchase
//! of whole Band IV, with the world's generated deposit and ceiling);
//! `ES_WATCH=<planet id>` prints that center's production decisions.
//!
//! Run: `cargo run --release --example empire_spread -- <seats> <horizon>`.
use hyades_engine::galaxy::{Galaxy, GalaxyConfig, Ground, Homeworlds};
use hyades_engine::log::{LogCategory, LogEvent, LogFilter};
use hyades_engine::resources::Material;
use hyades_engine::sim::{SimConfig, Simulation};
use std::io::Write;

/// Years between samples of the integrated stocks.
const SAMPLE_YEARS: f64 = 25.0;
/// Expansion, Growth, Production, supers forged, apex forged.
const METRICS: [&str; 5] = ["Expansion", "Growth", "Production", "supers", "apex"];

fn cv(x: &[f64]) -> f64 {
    let n = x.len().max(1) as f64;
    let m = x.iter().sum::<f64>() / n;
    if m <= 0.0 {
        return 0.0;
    }
    (x.iter().map(|v| (v - m) * (v - m)).sum::<f64>() / n).sqrt() / m
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let seats: usize = args.get(1).and_then(|a| a.parse().ok()).unwrap_or(3);
    let horizon: f64 = args.get(2).and_then(|a| a.parse().ok()).unwrap_or(1500.0);
    let seeds: Vec<u64> = std::env::var("ES_SEEDS")
        .map(|v| v.split(',').filter_map(|s| s.trim().parse().ok()).collect())
        .unwrap_or_else(|_| vec![1, 7, 42, 31337]);
    let ground = match std::env::var("ES_GROUND").as_deref().map(str::trim) {
        Ok("identical") => Ground::Identical,
        Ok("rotated") => Ground::ColorRotated,
        _ => Ground::Random,
    };
    let homeworlds = match std::env::var("ES_HOMEWORLDS").as_deref().map(str::trim) {
        Ok("centered") => Homeworlds::ColorCentered,
        _ => Homeworlds::Trio,
    };
    let trace = std::env::var("ES_TRACE").is_ok_and(|v| v.trim() == "1");
    println!("empire_spread: {seats} seats, horizon {horizon} yr, {ground:?}, {homeworlds:?}, seeds {seeds:?}");
    std::io::stdout().flush().ok();
    let mut cvs: Vec<[f64; 5]> = Vec::new();
    for &seed in &seeds {
        let g = GalaxyConfig { ground, homeworlds, ..GalaxyConfig::new(seats, seed) };
        let mut cfg = SimConfig::new(seed);
        cfg.horizon_years = horizon;
        let galaxy = Galaxy::generate(g).unwrap();
        // Per planet: is it a homeworld, its generated ceiling `K`, its deposit.
        let galaxy_planets: Vec<(bool, f64, [f64; 3])> = galaxy
            .planets
            .iter()
            .map(|p| {
                (
                    p.is_homeworld,
                    p.habitability.min(p.biosphere).bands(),
                    hyades_engine::resources::Basic::ALL.map(|b| p.minerals.get(b).kilotons()),
                )
            })
            .collect();
        let mut sim = Simulation::with_baseline(galaxy, cfg);
        let filter = LogFilter::none().with(LogCategory::Production);
        sim.set_log_filter(if trace { filter.with(LogCategory::Mining) } else { filter });
        let mut acc = vec![[0.0f64; 3]; seats];
        let mut prev = vec![[0.0f64; 3]; seats];
        let mut prev_t = 0.0;
        let mut next = SAMPLE_YEARS;
        let mut running = true;
        while running {
            while sim.clock() < next {
                if !sim.step() {
                    running = false;
                    break;
                }
            }
            let snap = sim.snapshot();
            let mut now = vec![[0.0f64; 3]; seats];
            for pl in &snap.planets {
                if let Some(o) = pl.owner {
                    now[o as usize][0] += 1.0;
                    now[o as usize][1] += pl.works.kilotons();
                }
            }
            for v in &snap.vehicles {
                now[v.owner as usize][2] += v.volume.hull_units_cubed();
            }
            let t = sim.clock().min(next).min(horizon);
            if trace && (next % 100.0).abs() < 1e-9 {
                let mut line = format!("{seed:>5} {next:>5.0} yr");
                for (p, stock) in now.iter().enumerate() {
                    let top = snap
                        .planets
                        .iter()
                        .filter(|pl| pl.owner == Some(p as u32))
                        .max_by(|a, b| a.works.kilotons().total_cmp(&b.works.kilotons()));
                    let (w, home) = top.map_or((0.0, false), |pl| (pl.works.kilotons(), pl.is_homeworld));
                    line +=
                        &format!(" | s{p} works {:>9.1} top {:>9.1}{}", stock[1], w, if home { " (home)" } else { "" });
                }
                println!("{line}");
            }
            for p in 0..seats {
                for m in 0..3 {
                    acc[p][m] += 0.5 * (now[p][m] + prev[p][m]) * (t - prev_t);
                }
            }
            prev = now;
            prev_t = t;
            next += SAMPLE_YEARS;
            if t >= horizon {
                break;
            }
        }
        if trace {
            for r in sim.log().iter() {
                if let LogEvent::BuildApplied { player, center, order, cost, .. } = r.event {
                    if order == hyades_engine::autopilot::BuildOrder::UpgradeInfrastructure && cost > 100.0 {
                        let pl = &galaxy_planets[center.0 as usize];
                        println!(
                            "{seed:>5} band IV bought: t {:.0} seat {player} cost {cost:.0} kt, world {} home {} K {:.2}, deposit C/M/Y {:.0}/{:.0}/{:.0} kt",
                            r.time,
                            center.0,
                            pl.0,
                            pl.1,
                            pl.2[0],
                            pl.2[1],
                            pl.2[2]
                        );
                    }
                }
            }
        }
        if let Some(w) = std::env::var("ES_WATCH").ok().and_then(|v| v.trim().parse::<u32>().ok()) {
            // One center's own decisions: when it decided, what it held and
            // what it chose.
            let mut last = -100.0;
            for r in sim.log().iter() {
                if let LogEvent::ProductionDecision {
                    center,
                    infra,
                    k_potential,
                    stockpile,
                    infra_cost,
                    can_afford_infra,
                    chosen,
                    ..
                } = r.event
                {
                    if center.0 == w && r.time - last >= 10.0 {
                        last = r.time;
                        println!(
                            "{seed:>5} watch {w}: t {:.0} infra {infra:.2} K {k_potential:.2} bank {stockpile:.0} kt, \
                             next whole Band {infra_cost:.0} kt, affordable {can_afford_infra}, chose {chosen:?}",
                            r.time
                        );
                    }
                }
            }
        }
        if trace {
            // Freight deposited per center, by century: the top three centers
            // of each seat and their share of that seat's deposits.
            let mut by: std::collections::BTreeMap<(u32, u64, u32), f64> = Default::default();
            for r in sim.log().iter() {
                if let LogEvent::FreighterTransfer {
                    player,
                    leg: hyades_engine::log::FreighterLeg::Deposited,
                    amount,
                    at,
                    ..
                } = r.event
                {
                    *by.entry((player, (r.time / 100.0) as u64, at.0)).or_default() += amount;
                }
            }
            for p in 0..seats as u32 {
                for c in 1..=5u64 {
                    let mut v: Vec<(u32, f64)> = by
                        .iter()
                        .filter(|((q, cc, _), _)| *q == p && *cc == c)
                        .map(|((_, _, w), a)| (*w, *a))
                        .collect();
                    let total: f64 = v.iter().map(|x| x.1).sum();
                    v.sort_by(|a, b| b.1.total_cmp(&a.1));
                    let top: Vec<String> = v
                        .iter()
                        .take(3)
                        .map(|(w, a)| format!("{w}: {a:.0} ({:.0}%)", 100.0 * a / total.max(1e-9)))
                        .collect();
                    println!(
                        "{seed:>5} freight s{p} {}00s: {total:.0} kt to {} centers; top {}",
                        c,
                        v.len(),
                        top.join(", ")
                    );
                }
            }
        }
        let mut forged = vec![[0.0f64; 2]; seats];
        for r in sim.log().iter() {
            if let LogEvent::Synthesized { player, material, made, .. } = r.event {
                let k = if material == Material::Apex { 1 } else { 0 };
                forged[player as usize][k] += made;
            }
        }
        let column =
            |m: usize| -> Vec<f64> { (0..seats).map(|p| if m < 3 { acc[p][m] } else { forged[p][m - 3] }).collect() };
        let row: [f64; 5] = core::array::from_fn(|m| cv(&column(m)));
        println!(
            "{seed:>5}: cv {}",
            METRICS.iter().zip(row.iter()).map(|(n, c)| format!("{n} {c:.3}")).collect::<Vec<_>>().join(", ")
        );
        for (m, name) in METRICS.iter().enumerate() {
            println!("{seed:>5}   {name:<10} {:?}", column(m).iter().map(|v| v.round()).collect::<Vec<_>>());
        }
        std::io::stdout().flush().ok();
        cvs.push(row);
    }
    let n = cvs.len().max(1) as f64;
    println!(
        "mean cv over {} galaxies: {}",
        cvs.len(),
        METRICS
            .iter()
            .enumerate()
            .map(|(m, name)| format!("{name} {:.3}", cvs.iter().map(|r| r[m]).sum::<f64>() / n))
            .collect::<Vec<_>>()
            .join(", ")
    );
}
