//! **The standard twelve-seat card bed** (T-120, reshaped at T-125) — every
//! seat plays exactly one card at earliest legal play, and each card is scored
//! against the author's design target: **1.5x to 2.0x its own tree's metric at
//! the 92nd percentile** (`Hyades_trees_and_card_value.md` §4.2).
//!
//! Three arms, because a mixed table alone cannot separate the two cards:
//!
//! | arm | even seats | odd seats |
//! |---|---|---|
//! | `Pass` | — | — |
//! | `GrowthOnly` | — | Growth card |
//! | `Both` | Warfare card | Growth card |
//!
//! The Growth card's effect is `GrowthOnly` against `Pass` on odd seats; the
//! Warfare card's is `Both` against `GrowthOnly` on even seats. Landing both at
//! once and differencing against `Pass` would charge each card with the other's
//! effect on the same galaxy (AGENTS.md: *ablate them apart before you believe
//! either*).
//!
//! **One sample per card seat, as a ratio card / counterfactual:**
//!
//! | tree | metric | symbol |
//! |---|---|---|
//! | Growth | work-years, `∫₀ᵀ infra_i dt` (R-TREE3's interim stock) | `G_i` |
//! | Warfare | the integral of the seat's colonies over the rival mean, `∫₀ᵀ C_i / mean_{j≠i} C_j dt` — the author's reading of "1.5x" for a tree whose `W_i` is a difference | `S_i` |
//!
//! Six seats per tree per galaxy, eleven galaxies, so 66 samples a tree. **P92
//! is read over those samples, and its 90% interval is bootstrapped over
//! galaxies** — the twelve seats of one galaxy share it, so resampling seats
//! would count them as independent. Median and P98 are printed beside it as
//! the floor and blow-out reads §4.2 names.
//!
//! **Cards are played at earliest legal play**, the round-0 barrier at
//! `years_to_first_round` (200 yr). The harness checks `empire_can_afford` and
//! records when each card actually landed, because an unaffordable order
//! coerces to a *pass*.
//!
//! One `SEAT` line is printed per card seat as each galaxy finishes, so a run
//! killed by an ephemeral container resumes with `--seeds` and the lines
//! already printed are kept.

use hyades_engine::autopilot::{Autopilot, BaselineAutopilot, Doctrine};
use hyades_engine::cards::{self, CardId, Order, Target};
use hyades_engine::galaxy::{Galaxy, GalaxyConfig, PlayerId};
use hyades_engine::sim::{SimConfig, Simulation};
use hyades_engine::units::Price;
use std::io::Write;

const SEATS: usize = 12;
const WARFARE_CARD: u16 = 15;
const GROWTH_CARD: u16 = 3;
/// The standard eight (`examples/card_probe`) and three more, so the table
/// has eleven independent galaxies. `--seeds a,b,c` overrides it, which is how
/// a run killed by an ephemeral container is resumed: the seeds that finished
/// have already printed their `ROW` lines.
const SEEDS: [u64; 11] = [1, 7, 42, 31337, 2, 3, 5, 11, 13, 17, 19];
/// The compounding window, and the sampling stride inside it. The fit starts at
/// the round-0 barrier, where the cards land — before it both arms are the same
/// run to the last bit — and runs 600 years past it.
const WINDOW_START: f64 = 200.0;
const WINDOW_END: f64 = 800.0;
const SAMPLE_YEARS: f64 = 25.0;

/// `--smoke` runs one seed over a quarter of the window: a few seconds, enough
/// to see every column populated and no `NaN`, and not a measurement. Printed
/// as such so a smoke run is never mistaken for the bed.
fn smoke() -> bool {
    std::env::args().any(|a| a == "--smoke")
}
fn seeds() -> Vec<u64> {
    let args: Vec<String> = std::env::args().collect();
    if let Some(i) = args.iter().position(|a| a == "--seeds") {
        let list = args.get(i + 1).expect("--seeds takes a comma-separated list");
        return list.split(',').map(|x| x.trim().parse().expect("a seed is a u64")).collect();
    }
    if smoke() {
        SEEDS[..1].to_vec()
    } else {
        SEEDS.to_vec()
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
    // The same engine in every arm; only the cards differ (T-133: a bed
    // varies nothing but the galaxy and the protocol's own orders).
    let play_at = cfg.years_to_first_round;
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
            if slot.is_some() || sim.clock() < play_at {
                continue;
            }
            let Some(id) = card_for(i, arm) else { continue };
            let Some(c) = cards::card(id) else { continue };
            if !sim.empire_can_afford(i, Price::new(c.cost)) {
                continue;
            }
            sim.apply_orders(
                sim.current_round(),
                &[Order { seat: PlayerId(i as u32), card: Some(id), target: Target::None }],
            );
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

/// Trapezoidal `∫ x dt` over the sampled timeline.
fn integral(t: &[f64], x: &[f64]) -> f64 {
    t.windows(2).zip(x.windows(2)).map(|(tw, xw)| 0.5 * (xw[0] + xw[1]) * (tw[1] - tw[0])).sum()
}

/// `S_i(t) = C_i / mean_{j≠i} C_j` — this seat's colonies over the rival mean.
/// A rival field with no colonies yet (the first samples) reads as parity.
fn share(tr: &Trace, i: usize) -> Vec<f64> {
    (0..tr.t.len())
        .map(|k| {
            let others: f64 =
                (0..SEATS).filter(|&j| j != i).map(|j| tr.colonies[j][k]).sum::<f64>() / (SEATS - 1) as f64;
            if others > 0.0 {
                tr.colonies[i][k] / others
            } else {
                1.0
            }
        })
        .collect()
}

/// Linear-interpolated quantile of `v` at `q` in `[0, 1]`.
fn quantile(v: &[f64], q: f64) -> f64 {
    let mut s = v.to_vec();
    s.sort_by(f64::total_cmp);
    let pos = q * (s.len() - 1) as f64;
    let (lo, hi) = (pos.floor() as usize, pos.ceil() as usize);
    s[lo] + (s[hi] - s[lo]) * (pos - lo as f64)
}

/// **P92 with a 90% interval bootstrapped over galaxies.** Each replicate draws
/// galaxies with replacement and keeps every seat of each draw together.
/// Deterministic: a fixed splitmix stream, so a rerun prints the same interval.
fn p92_interval(by_galaxy: &[Vec<f64>]) -> (f64, f64) {
    let mut state: u64 = 0x5EED_0092;
    let mut next = || {
        state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    };
    let g = by_galaxy.len();
    let mut reps: Vec<f64> = (0..2000)
        .map(|_| {
            let pooled: Vec<f64> =
                (0..g).flat_map(|_| by_galaxy[(next() % g as u64) as usize].iter().copied()).collect();
            quantile(&pooled, 0.92)
        })
        .collect();
    reps.sort_by(f64::total_cmp);
    (quantile(&reps, 0.05), quantile(&reps, 0.95))
}

fn report(name: &str, by_galaxy: &[Vec<f64>]) {
    let all: Vec<f64> = by_galaxy.iter().flatten().copied().collect();
    if all.is_empty() {
        return;
    }
    let (lo, hi) = if by_galaxy.len() > 1 { p92_interval(by_galaxy) } else { (f64::NAN, f64::NAN) };
    let p92 = quantile(&all, 0.92);
    let verdict = if (1.5..=2.0).contains(&p92) {
        "in the 1.5-2.0x target"
    } else if p92 < 1.5 {
        "below target"
    } else {
        "above target"
    };
    println!(
        "  {name:<34} median {:.3}  P92 {p92:.3} [{lo:.3}, {hi:.3}]  P98 {:.3}  (n={} seats, {} galaxies) — {verdict}",
        quantile(&all, 0.5),
        quantile(&all, 0.98),
        all.len(),
        by_galaxy.len()
    );
}

fn main() {
    if smoke() {
        println!("*** --smoke: one seed, quarter window. A shape check, not a measurement. ***");
    }
    println!(
        "{SEATS} seats, three arms, {} galaxies, horizon {} yr, cards at the round-0 barrier",
        seeds().len(),
        window_end()
    );
    println!("  Warfare card {WARFARE_CARD}: {:?}", cards::card(CardId(WARFARE_CARD)).map(|c| c.effects));
    println!("  Growth   card {GROWTH_CARD}: {:?}", cards::card(CardId(GROWTH_CARD)).map(|c| c.effects));
    println!("  SEAT columns: seed, seat, tree, ratio card/counterfactual of the tree's metric");
    let _ = std::io::stdout().flush();

    let (mut growth, mut warfare) = (Vec::new(), Vec::new());
    let (mut late_plays, mut never_played) = (0usize, 0usize);
    for seed in seeds() {
        let t0 = std::time::Instant::now();
        let p = run(seed, Arm::Pass);
        let g = run(seed, Arm::GrowthOnly);
        let b = run(seed, Arm::Both);
        late_plays += b.played_at.iter().flatten().filter(|&&t| t > WINDOW_START + SAMPLE_YEARS).count();
        never_played += b.played_at.iter().filter(|p| p.is_none()).count();
        let (mut gs, mut ws) = (Vec::new(), Vec::new());
        for i in 0..SEATS {
            if i.is_multiple_of(2) {
                let r = integral(&b.t, &share(&b, i)) / integral(&g.t, &share(&g, i));
                println!("SEAT {seed} {i} warfare {r:.6}");
                ws.push(r);
            } else {
                let r = integral(&g.t, &g.infra[i]) / integral(&p.t, &p.infra[i]);
                println!("SEAT {seed} {i} growth {r:.6}");
                gs.push(r);
            }
        }
        growth.push(gs);
        warfare.push(ws);
        println!("  seed {seed} done in {:.0} s", t0.elapsed().as_secs_f64());
        let _ = std::io::stdout().flush();
    }
    println!(
        "\n  cards landing after the first sample past the barrier: {late_plays}; never landed: {never_played} (both 0 means every play was on time)"
    );
    report("Growth, G_card / G_pass", &growth);
    report("Warfare, S_card / S_pass", &warfare);
}
