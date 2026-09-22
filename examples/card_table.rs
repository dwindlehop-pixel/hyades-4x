//! **A twelve-seat table where every seat plays exactly one card** (T-120) —
//! the head-to-head `Hyades_warfare_tree.md` §8.4 asks for, costing the first
//! Warfare card against the first Growth card on one shared bed.
//!
//! Three arms, because a mixed table alone cannot separate the two cards:
//!
//! | arm | even seats | odd seats |
//! |---|---|---|
//! | `Pass` | — | — |
//! | `GrowthOnly` | — | Growth card |
//! | `Both` | Warfare card | Growth card |
//!
//! The Growth card's effect is `GrowthOnly − Pass` read on odd seats; the
//! Warfare card's is `Both − GrowthOnly` read on even seats. Landing both at
//! once and differencing against `Pass` would charge each card with the other's
//! effect on the same galaxy (CLAUDE.md: *ablate them apart before you believe
//! either*).
//!
//! **`g` is fitted by least squares on `ln X(t)` across the window**, not read
//! from two endpoints — `Hyades_trees_and_card_value.md` §2.4 asks for that
//! explicitly, because an endpoint pair is dominated by whichever end is
//! noisier and hides saturation instead of exposing it. One run per arm
//! carries the whole window, so this is also half the runs of an endpoint
//! design.
//!
//! ```text
//! ln X(t) ≈ a + g·t   over t in [WINDOW_START, WINDOW_END]
//! t½ = ln 2 / g
//! value = 1 − t½(card) / t½(counterfactual)
//! ```
//!
//! `R²` is printed beside every fit. A window in the exponential regime fits a
//! line; curvature is the signal that the window was chosen wrong, and it is
//! the reason to print it rather than to assume it.
//!
//! **Cards are played at earliest legal play** (§2.4, §4.4), which is round 0:
//! `bootstrap` seeds each homeworld above every tier-0 price. The harness
//! checks `empire_can_afford` anyway and records when each card actually
//! landed, because an order that cannot be paid for coerces to a *pass* and a
//! bed cannot otherwise tell that from a card that did nothing.
//!
//! **Warfare's stock is a difference and can be negative** (R-TREE9), so it has
//! no logarithm and no doubling time wherever `W_i <= 0`. The harness reports
//! raw `ΔW_i` in colonies there rather than inventing a value.
//!
//! `w_ij` is **uniform over the other eleven seats** — a stated placeholder,
//! because R-WAR3 leaves the weighting unset. It sums to 1 over `j != i`, which
//! §2.3.2 requires and an unweighted mean including self does not.

use hyades_engine::autopilot::{Autopilot, BaselineAutopilot, Doctrine};
use hyades_engine::cards::{self, CardId, Order, Target};
use hyades_engine::galaxy::{Galaxy, GalaxyConfig, PlayerId};
use hyades_engine::sim::{SimConfig, Simulation};
use hyades_engine::units::Price;
use std::io::Write;

const SEATS: usize = 12;
const WARFARE_CARD: u16 = 15;
const GROWTH_CARD: u16 = 3;
const SEEDS: [u64; 3] = [1, 7, 42];
/// The compounding window, and the sampling stride inside it. The run goes to
/// `WINDOW_END`; the fit starts at `WINDOW_START` so the opening survey
/// fan-out, which is not compounding, is outside it.
const WINDOW_START: f64 = 150.0;
const WINDOW_END: f64 = 600.0;
const SAMPLE_YEARS: f64 = 25.0;

/// `--smoke` runs one seed over a quarter of the window: a few seconds, enough
/// to see every column populated and no `NaN`, and not a measurement. Printed
/// as such so a smoke run is never mistaken for the bed.
fn smoke() -> bool {
    std::env::args().any(|a| a == "--smoke")
}
fn seeds() -> &'static [u64] {
    if smoke() {
        &SEEDS[..1]
    } else {
        &SEEDS
    }
}
fn window_end() -> f64 {
    if smoke() {
        WINDOW_START + (WINDOW_END - WINDOW_START) / 4.0
    } else {
        WINDOW_END
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Arm {
    Pass,
    GrowthOnly,
    Both,
}

/// Which card seat `i` holds in `arm`, or `None` for a pass. Seats alternate so
/// neither tree is systematically placed nearer the middle of the galaxy.
fn card_for(i: usize, arm: Arm) -> Option<CardId> {
    match (arm, i.is_multiple_of(2)) {
        (Arm::Pass, _) => None,
        (Arm::GrowthOnly, true) => None,
        (Arm::Both, true) => Some(CardId(WARFARE_CARD)),
        (_, false) => Some(CardId(GROWTH_CARD)),
    }
}

/// One arm's whole timeline: the sample times, and per seat the colony count
/// and work stock at each of them.
struct Trace {
    t: Vec<f64>,
    colonies: Vec<Vec<f64>>,
    infra: Vec<Vec<f64>>,
    played_at: Vec<Option<f64>>,
}

fn run(seed: u64, arm: Arm) -> Trace {
    let galaxy = Galaxy::generate(GalaxyConfig::new(SEATS, seed)).unwrap();
    let autopilots: Vec<Box<dyn Autopilot>> =
        (0..SEATS).map(|_| Box::new(BaselineAutopilot::new(Doctrine::default())) as Box<_>).collect();
    let mut cfg = SimConfig::new(seed);
    cfg.horizon_years = window_end();
    // The same bed in every arm. Gating the engagement layer on the arm would
    // put its own effect inside the Warfare card's measured value.
    cfg.engagements_enabled = true;
    let mut sim = Simulation::new(galaxy, cfg, autopilots);

    let mut tr = Trace {
        t: Vec::new(),
        colonies: vec![Vec::new(); SEATS],
        infra: vec![Vec::new(); SEATS],
        played_at: vec![None; SEATS],
    };
    let mut next_sample = 0.0_f64;
    let sample = |sim: &Simulation, tr: &mut Trace| {
        tr.t.push(sim.clock());
        for i in 0..SEATS {
            let (c, f) = sim.tree_stock(i);
            tr.colonies[i].push(c as f64);
            tr.infra[i].push(f);
        }
    };

    while sim.step() {
        for (i, slot) in tr.played_at.iter_mut().enumerate() {
            if slot.is_some() {
                continue;
            }
            let Some(id) = card_for(i, arm) else { continue };
            let Some(c) = cards::card(id) else { continue };
            if !sim.empire_can_afford(i, Price::new(c.cost)) {
                continue;
            }
            sim.apply_orders(0, &[Order { seat: PlayerId(i as u32), card: Some(id), target: Target::None }]);
            *slot = Some(sim.clock());
        }
        if sim.clock() >= next_sample {
            next_sample = sim.clock() + SAMPLE_YEARS;
            sample(&sim, &mut tr);
        }
    }
    sample(&sim, &mut tr);
    tr
}

/// Least-squares slope of `ln x` on `t` over the window, with `R²`. `None`
/// where fewer than three samples are positive — a stock that is zero, or a
/// difference that is negative, has no logarithm (R-TREE9).
fn fit(t: &[f64], x: &[f64]) -> Option<(f64, f64)> {
    let pts: Vec<(f64, f64)> = t
        .iter()
        .zip(x)
        .filter(|&(&ti, &xi)| (WINDOW_START..=window_end()).contains(&ti) && xi > 0.0)
        .map(|(&ti, &xi)| (ti, xi.ln()))
        .collect();
    if pts.len() < 3 {
        return None;
    }
    let n = pts.len() as f64;
    let mt = pts.iter().map(|p| p.0).sum::<f64>() / n;
    let my = pts.iter().map(|p| p.1).sum::<f64>() / n;
    let sxx = pts.iter().map(|p| (p.0 - mt).powi(2)).sum::<f64>();
    let sxy = pts.iter().map(|p| (p.0 - mt) * (p.1 - my)).sum::<f64>();
    let syy = pts.iter().map(|p| (p.1 - my).powi(2)).sum::<f64>();
    if sxx <= 0.0 || syy <= 0.0 {
        return None;
    }
    Some((sxy / sxx, sxy * sxy / (sxx * syy)))
}

/// Doubling time from a fitted rate, or `None` where the stock is not growing
/// and `ln 2 / g` is not a time.
fn half_life(t: &[f64], x: &[f64]) -> Option<(f64, f64)> {
    let (g, r2) = fit(t, x)?;
    (g > 0.0).then(|| (std::f64::consts::LN_2 / g, r2))
}

/// `W_i(t) = C_i − Σ_{j≠i} w_ij C_j` with `w_ij` uniform over the other seats.
fn contrast(tr: &Trace, i: usize) -> Vec<f64> {
    (0..tr.t.len())
        .map(|k| {
            let others: f64 = (0..SEATS).filter(|&j| j != i).map(|j| tr.colonies[j][k]).sum();
            tr.colonies[i][k] - others / (SEATS - 1) as f64
        })
        .collect()
}

fn stat(v: &[f64]) -> (f64, f64, usize) {
    if v.is_empty() {
        return (f64::NAN, f64::NAN, 0);
    }
    let n = v.len() as f64;
    let m = v.iter().sum::<f64>() / n;
    let var = v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (n - 1.0).max(1.0);
    (m, (var / n).sqrt(), v.len())
}

fn main() {
    if smoke() {
        println!("*** --smoke: one seed, quarter window. A shape check, not a measurement. ***");
    }
    println!(
        "{SEATS} seats, three arms, fit on ln X over [{WINDOW_START}, {}] yr every {SAMPLE_YEARS} yr, {} seeds",
        window_end(),
        seeds().len()
    );
    println!("  Warfare card {WARFARE_CARD}: {:?}", cards::card(CardId(WARFARE_CARD)).map(|c| c.effect));
    println!("  Growth   card {GROWTH_CARD}: {:?}", cards::card(CardId(GROWTH_CARD)).map(|c| c.effect));
    let _ = std::io::stdout().flush();

    let mut growth_val = Vec::new();
    let mut growth_raw = Vec::new();
    let mut growth_r2 = Vec::new();
    let mut warfare_val = Vec::new();
    let mut warfare_raw = Vec::new();
    let mut warfare_r2 = Vec::new();
    let mut w_defined = 0usize;
    let mut w_total = 0usize;
    let mut play_times = Vec::new();
    let mut identical = true;

    for &seed in seeds() {
        let t0 = std::time::Instant::now();
        let p = run(seed, Arm::Pass);
        let g = run(seed, Arm::GrowthOnly);
        let b = run(seed, Arm::Both);
        if b.colonies != g.colonies || b.infra != g.infra {
            identical = false;
        }
        for t in b.played_at.iter().flatten() {
            play_times.push(*t);
        }

        for i in 0..SEATS {
            if i.is_multiple_of(2) {
                // Warfare seat: `Both` against the same seat under
                // `GrowthOnly`, which is that seat passing.
                let wb = contrast(&b, i);
                let wg = contrast(&g, i);
                w_total += 1;
                warfare_raw.push(wb[wb.len() - 1] - wg[wg.len() - 1]);
                if let (Some((a, r2)), Some((c, _))) = (half_life(&b.t, &wb), half_life(&g.t, &wg)) {
                    warfare_val.push(1.0 - a / c);
                    warfare_r2.push(r2);
                    w_defined += 1;
                }
            } else {
                // Growth seat: work stock under `GrowthOnly` against `Pass`.
                let last = g.infra[i].len() - 1;
                growth_raw.push(g.infra[i][last] / p.infra[i][last] - 1.0);
                if let (Some((a, r2)), Some((c, _))) = (half_life(&g.t, &g.infra[i]), half_life(&p.t, &p.infra[i])) {
                    growth_val.push(1.0 - a / c);
                    growth_r2.push(r2);
                }
            }
        }
        println!("  seed {seed} done in {:.0} s", t0.elapsed().as_secs_f64());
        let _ = std::io::stdout().flush();
    }

    let (gm, gse, gn) = stat(&growth_val);
    let (grm, grse, _) = stat(&growth_raw);
    let (gr2, ..) = stat(&growth_r2);
    let (wm, wse, wn) = stat(&warfare_val);
    let (wrm, wrse, wrn) = stat(&warfare_raw);
    let (wr2, ..) = stat(&warfare_r2);
    let (pm, _, pn) = stat(&play_times);

    println!("\n  earliest legal play: {pm:.1} yr, mean over seats and seeds (n={pn})");
    println!("\n  Growth  card value (1 - t½ ratio):  {gm:+.4} ± {gse:.4}  (n={gn}, mean R² {gr2:.4})");
    println!("  Growth  work stock at {} yr:  {grm:+.4} ± {grse:.4}", window_end());
    println!("\n  Warfare card value (1 - t½ ratio):  {wm:+.4} ± {wse:.4}  (n={wn}, mean R² {wr2:.4})");
    println!("  Warfare W_i defined on {w_defined} of {w_total} seat-seeds");
    println!("  Warfare ΔW_i at {} yr, colonies: {wrm:+.3} ± {wrse:.3}  (n={wrn})", window_end());
    println!("\n  Warfare arm reproduces the Growth-only arm exactly: {}", if identical { "YES" } else { "no" });
}
