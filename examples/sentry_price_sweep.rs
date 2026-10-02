//! **R-WAR47: what a lost sentry should add to the price of the next one** —
//! the author's ruling that the answer is a Monte Carlo experiment.
//!
//! A center that has lost `L` sentries prices the next at `c_s · (1 + κ · L)`
//! (`Doctrine::sentry_loss_price`), so `κ = 0` replaces every loss and a large
//! `κ` stops replacing after the first few. This harness sweeps `κ`.
//!
//! **The bed.** Three seats on one galaxy. Seat 0 plays the missile card
//! (`TIER0[13]`), seat 1 the first Warfare card (`TIER0[15]`), seat 2 nothing —
//! the arm in which sentry losses were measured to run away (appendix §D.24).
//! Cards are played at the protocol's first round barrier through
//! `apply_orders`; the only thing varied between arms is seat 0's `κ`.
//!
//! **The objective is the defending seat's stock**, because the knob is a
//! decision about how much of that seat's yard goes to replacing losses:
//! seat 0's colony-years and work-years (`∫ colonies dt`, `∫ infra dt`,
//! `Hyades_trees_and_card_value.md` §2.3), each divided by the same seed's
//! `κ = 0` arm, and their geometric mean. Common random numbers: every `κ` runs
//! on the same seeds, and the ratio is taken seed by seed. Seat 1's ratio is
//! printed beside it, so a gain for seat 0 bought by seat 1 losing less is
//! visible as such.
//!
//! Run: `cargo run --release --example sentry_price_sweep -- [horizon] [κ...]`;
//! `SP_SEEDS=2,3,5,11` for the replication set, `SP_THREADS` to set
//! parallelism (default 4).
use hyades_engine::autopilot::{Autopilot, BaselineAutopilot, Doctrine};
use hyades_engine::cards::{CardId, Order, Target};
use hyades_engine::galaxy::{Galaxy, GalaxyConfig, PlayerId};
use hyades_engine::sim::{SimConfig, Simulation};
use std::io::Write;
use std::sync::{Arc, Mutex};

/// The standard four-seed CRN bed.
const SEEDS: [u64; 4] = [1, 7, 42, 31337];
const SEATS: usize = 3;
/// Missile card for seat 0, Warfare card for seat 1, none for seat 2.
const CARDS: [Option<u16>; SEATS] = [Some(13), Some(15), None];
const DEFAULT_HORIZON: f64 = 800.0;
const DEFAULT_KAPPAS: [f64; 7] = [0.0, 0.03, 0.1, 0.3, 1.0, 3.0, 10.0];
/// One sample per economy tick (`cycle_years = 5`).
const SAMPLE_YEARS: f64 = 5.0;

#[derive(Clone, Copy, Default)]
struct Run {
    /// `∫ colonies dt` and `∫ infra dt` per seat.
    colony_years: [f64; SEATS],
    work_years: [f64; SEATS],
    sentries_ordered: u64,
    sentries_lost: u64,
    rounds: u64,
    events: u64,
    secs: f64,
}

fn seeds() -> Vec<u64> {
    match std::env::var("SP_SEEDS") {
        Ok(v) => v.split(',').filter_map(|s| s.trim().parse().ok()).collect(),
        Err(_) => SEEDS.to_vec(),
    }
}

fn run(seed: u64, kappa: f64, horizon: f64) -> Run {
    let galaxy = Galaxy::generate(GalaxyConfig::new(SEATS, seed)).unwrap();
    let autopilots: Vec<Box<dyn Autopilot>> = (0..SEATS)
        .map(|i| {
            let d =
                if i == 0 { Doctrine { sentry_loss_price: kappa, ..Doctrine::default() } } else { Doctrine::default() };
            Box::new(BaselineAutopilot::new(d)) as Box<_>
        })
        .collect();
    let mut cfg = SimConfig::new(seed);
    cfg.horizon_years = horizon;
    let play_at = cfg.years_to_first_round;
    let mut sim = Simulation::new(galaxy, cfg, autopilots);
    let t0 = std::time::Instant::now();
    let mut out = Run::default();
    let mut played = false;
    let mut next = SAMPLE_YEARS;
    while sim.step() {
        if !played && sim.clock() >= play_at {
            let orders: Vec<Order> = CARDS
                .iter()
                .enumerate()
                .map(|(i, c)| Order { seat: PlayerId(i as u32), card: c.map(CardId), target: Target::None })
                .collect();
            sim.apply_orders(sim.current_round(), &orders);
            played = true;
        }
        while sim.clock() >= next && next <= horizon {
            for p in 0..SEATS {
                let (colonies, infra) = sim.tree_stock(p);
                out.colony_years[p] += colonies as f64 * SAMPLE_YEARS;
                out.work_years[p] += infra * SAMPLE_YEARS;
            }
            next += SAMPLE_YEARS;
        }
    }
    let m = sim.missile_stats();
    out.sentries_ordered = m.sentries_ordered;
    out.sentries_lost = m.sentries_lost;
    out.rounds = m.launched;
    out.events = sim.events_processed();
    out.secs = t0.elapsed().as_secs_f64();
    out
}

/// Mean and standard error of a sample.
fn mean_se(v: &[f64]) -> (f64, f64) {
    let n = v.len() as f64;
    let m = v.iter().sum::<f64>() / n;
    if v.len() < 2 {
        return (m, f64::NAN);
    }
    let var = v.iter().map(|x| (x - m) * (x - m)).sum::<f64>() / (n - 1.0);
    (m, (var / n).sqrt())
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let horizon: f64 = args.first().and_then(|a| a.parse().ok()).unwrap_or(DEFAULT_HORIZON);
    let mut kappas: Vec<f64> = args.iter().skip(1).filter_map(|a| a.parse().ok()).collect();
    if kappas.is_empty() {
        kappas = DEFAULT_KAPPAS.to_vec();
    }
    // κ = 0 is every seed's reference arm.
    if !kappas.contains(&0.0) {
        kappas.insert(0, 0.0);
    }
    let seeds = seeds();
    let threads: usize = std::env::var("SP_THREADS").ok().and_then(|v| v.parse().ok()).unwrap_or(4);
    println!("sentry_price_sweep: horizon {horizon} yr, seeds {seeds:?}, kappas {kappas:?}, {threads} threads");
    std::io::stdout().flush().ok();

    let jobs: Vec<(usize, usize)> = (0..seeds.len()).flat_map(|s| (0..kappas.len()).map(move |k| (s, k))).collect();
    let jobs = Arc::new(Mutex::new(jobs.into_iter()));
    let results = Arc::new(Mutex::new(vec![vec![Run::default(); kappas.len()]; seeds.len()]));
    let handles: Vec<_> = (0..threads)
        .map(|_| {
            let (jobs, results, seeds, kappas) = (jobs.clone(), results.clone(), seeds.clone(), kappas.clone());
            std::thread::spawn(move || loop {
                let Some((s, k)) = jobs.lock().unwrap().next() else { break };
                let r = run(seeds[s], kappas[k], horizon);
                println!(
                    "  seed {:>5} κ {:>6}: seat0 cy {:.0} wy {:.1} | seat1 cy {:.0} | sentries {} lost {} rounds {} | {} events {:.1}s",
                    seeds[s],
                    kappas[k],
                    r.colony_years[0],
                    r.work_years[0],
                    r.colony_years[1],
                    r.sentries_ordered,
                    r.sentries_lost,
                    r.rounds,
                    r.events,
                    r.secs
                );
                std::io::stdout().flush().ok();
                results.lock().unwrap()[s][k] = r;
            })
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }
    let results = results.lock().unwrap();

    println!();
    println!(
        "{:>7} | {:>18} {:>18} {:>18} {:>5} | {:>18} | {:>9} {:>9} {:>9}",
        "κ", "seat0 colony-yr", "seat0 work-yr", "seat0 geomean", "+/n", "seat1 geomean", "sentries", "lost", "secs"
    );
    for (k, &kappa) in kappas.iter().enumerate() {
        let ratio = |f: &dyn Fn(&Run) -> f64| -> Vec<f64> {
            (0..seeds.len()).map(|s| f(&results[s][k]) / f(&results[s][0]) - 1.0).collect()
        };
        let cy = ratio(&|r| r.colony_years[0]);
        let wy = ratio(&|r| r.work_years[0]);
        let gm: Vec<f64> = cy.iter().zip(&wy).map(|(a, b)| ((1.0 + a) * (1.0 + b)).sqrt() - 1.0).collect();
        let gm1: Vec<f64> = {
            let c = ratio(&|r| r.colony_years[1]);
            let w = ratio(&|r| r.work_years[1]);
            c.iter().zip(&w).map(|(a, b)| ((1.0 + a) * (1.0 + b)).sqrt() - 1.0).collect()
        };
        let pos = gm.iter().filter(|&&x| x > 0.0).count();
        let mean =
            |f: &dyn Fn(&Run) -> f64| (0..seeds.len()).map(|s| f(&results[s][k])).sum::<f64>() / seeds.len() as f64;
        let pct = |v: &[f64]| {
            let (m, se) = mean_se(v);
            format!("{:+.2}% ± {:.2}", 100.0 * m, 100.0 * se)
        };
        println!(
            "{:>7} | {:>18} {:>18} {:>18} {:>2}/{:<2} | {:>18} | {:>9.0} {:>9.0} {:>9.1}",
            kappa,
            pct(&cy),
            pct(&wy),
            pct(&gm),
            pos,
            seeds.len(),
            pct(&gm1),
            mean(&|r| r.sentries_ordered as f64),
            mean(&|r| r.sentries_lost as f64),
            mean(&|r| r.secs)
        );
    }
}
