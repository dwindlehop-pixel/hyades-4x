//! **Are a centre's stockpiles even across the colours?**
//!
//! T-73's ablation (per-colour gate, proportional payment) reproduced the full
//! colour-billed run *bit-identically*, which says the payment scheme changes
//! nothing. That is only possible if every bank paying a works bill already has
//! the composition the bill asks for — so this checks the composition directly
//! rather than reasoning about it.
//!
//! **Extended at T-88 to answer the freight question** (`Hyades_industry.md`
//! §6.23). The decision census says 100% of idle decisions are "wants to deepen,
//! cannot pay the bill" with a mean bank of **561 kt** — the total is there and a
//! colour is not. This quantifies how much of a banked kilotonne is *dead* to a
//! three-colour bill, which is the number a routing change has to move.
//!
//! Two statistics, both per centre and then pooled:
//!
//! - **dominant share** — the largest colour's fraction of the bank. The
//!   galaxy's *sources* sit at 0.789 (T-81); if banks sit near that too, freight
//!   is inheriting the geology rather than mixing it.
//! - **payable fraction** — `3·min_c(bank_c) / total`. A works bill is a
//!   conjunction, so this is the share of the bank that could pay a balanced one
//!   and `1 − it` is ore that is banked and cannot be spent on a rung at any
//!   price. It is the direct measure of what R-O89's load-leg fix was reaching
//!   for and could not get all of.
//!
//! Run: `cargo run --release --example bank_mix`
use hyades_engine::autopilot::BuildOrder;
use hyades_engine::prelude::*;
use std::io::Write;

fn main() {
    let seed = 1u64;
    let galaxy = Galaxy::generate(GalaxyConfig::new(3, seed)).unwrap();
    let mut cfg = SimConfig::new(seed);
    cfg.horizon_years = 800.0;
    let mut sim = Simulation::with_baseline(galaxy, cfg);
    sim.set_log_filter(LogFilter::none().with(LogCategory::Production));
    sim.run();

    // **How often does a works bill actually get paid?** If the per-colour gate
    // refuses nearly all of them, `pay_bill` barely runs — and that alone would
    // explain why swapping the payment scheme changed nothing.
    let (mut upgrades, mut hulls, mut idle_decisions, mut decisions) = (0usize, 0usize, 0usize, 0usize);
    for r in sim.log().iter() {
        match r.event {
            LogEvent::BuildApplied { order: BuildOrder::UpgradeInfrastructure, .. } => upgrades += 1,
            LogEvent::BuildApplied { order: BuildOrder::Hull { .. }, .. } => hulls += 1,
            LogEvent::ProductionDecision { chosen, .. } => {
                decisions += 1;
                if chosen == BuildOrder::Idle {
                    idle_decisions += 1;
                }
            }
            _ => {}
        }
    }
    println!("builds applied: {upgrades} infrastructure, {hulls} hulls");
    println!("decisions: {decisions} total, {idle_decisions} Idle");
    std::io::stdout().flush().ok();
    println!("bank composition at t=800, seed {seed}");
    std::io::stdout().flush().ok();

    let snap = sim.snapshot();
    let (mut n, mut even, mut skewed) = (0usize, 0usize, 0usize);
    let mut worst = (0.0f64, 0usize);
    for p in snap.planets.iter().filter(|p| p.owner.is_some()) {
        let s = p.stockpile;
        let t = s.cyan + s.magenta + s.yellow;
        if t <= 1e-9 {
            continue;
        }
        n += 1;
        let (c, m, y) = (s.cyan / t, s.magenta / t, s.yellow / t);
        // Max deviation from an even third.
        let dev =
            [(c - 1.0 / 3.0).abs(), (m - 1.0 / 3.0).abs(), (y - 1.0 / 3.0).abs()].into_iter().fold(0.0f64, f64::max);
        if dev < 1e-12 {
            even += 1;
        } else {
            skewed += 1;
        }
        if dev > worst.0 {
            worst = (dev, p.id.0 as usize);
        }
    }
    println!("  {n} non-empty banks: {even} exactly even, {skewed} skewed");

    // **Where the ore comes from.** A freighter carries one outpost's ore, so
    // if a *source* is single-coloured then no routing rule over single-coloured
    // cargoes can assemble a three-coloured bank. Measure the sources.
    let (mut src, mut src_dom) = (0usize, 0.0f64);
    let mut dom_hist = [0usize; 5]; // <40, <60, <80, <95, >=95 % dominant
    for p in snap.planets.iter() {
        let d = &p.density;
        let t: f64 = Basic::ALL.iter().map(|&c| d.get(c).kilotons()).sum();
        if t <= 1e-9 {
            continue;
        }
        src += 1;
        let dom = Basic::ALL.iter().map(|&c| d.get(c).kilotons() / t).fold(0.0f64, f64::max);
        src_dom += dom;
        let b = if dom < 0.40 {
            0
        } else if dom < 0.60 {
            1
        } else if dom < 0.80 {
            2
        } else if dom < 0.95 {
            3
        } else {
            4
        };
        dom_hist[b] += 1;
    }
    println!("  {src} mineral sources, mean dominant-colour share {:.3}", src_dom / src.max(1) as f64);
    println!(
        "    <40%: {}  40-60%: {}  60-80%: {}  80-95%: {}  >=95%: {}",
        dom_hist[0], dom_hist[1], dom_hist[2], dom_hist[3], dom_hist[4]
    );
    std::io::stdout().flush().ok();
    println!("  worst deviation from an even third: {:.6} (planet {})", worst.0, worst.1);
    // Show a few so the shape is visible rather than summarised away.
    for p in snap.planets.iter().filter(|p| p.owner.is_some()).take(60) {
        let s = p.stockpile;
        let t = s.cyan + s.magenta + s.yellow;
        if t > 1e-6 {
            println!("  planet {:>5}: C {:>10.4}  M {:>10.4}  Y {:>10.4}", p.id.0, s.cyan, s.magenta, s.yellow);
        }
    }

    // --- the two summary statistics ---------------------------------------
    let snap = sim.snapshot();
    let mut dom = Vec::new();
    let mut pay = Vec::new();
    let (mut banked, mut dead) = (0.0f64, 0.0f64);
    for pl in snap.planets.iter().filter(|p| p.owner.is_some()) {
        let c = [pl.stockpile.cyan, pl.stockpile.magenta, pl.stockpile.yellow];
        let total: f64 = c.iter().sum();
        if total <= 1e-9 {
            continue;
        }
        let hi = c.iter().cloned().fold(f64::MIN, f64::max);
        let lo = c.iter().cloned().fold(f64::MAX, f64::min);
        dom.push(hi / total);
        let payable = 3.0 * lo / total;
        pay.push(payable);
        banked += total;
        dead += total * (1.0 - payable);
    }
    dom.sort_by(|a, b| a.partial_cmp(b).unwrap());
    pay.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let pct = |v: &Vec<f64>, q: f64| v[((v.len() - 1) as f64 * q) as usize];
    println!("\n=== composition of {} non-empty banks ===", dom.len());
    println!(
        "  dominant colour share   p10 {:.3}  median {:.3}  p90 {:.3}   (sources sit at 0.789, T-81)",
        pct(&dom, 0.1),
        pct(&dom, 0.5),
        pct(&dom, 0.9)
    );
    println!(
        "  payable fraction        p10 {:.3}  median {:.3}  p90 {:.3}   (3*min/total)",
        pct(&pay, 0.1),
        pct(&pay, 0.5),
        pct(&pay, 0.9)
    );
    println!(
        "  banked {banked:.0} kt, of which {dead:.0} kt ({:.1}%) cannot pay a balanced rung at any price",
        100.0 * dead / banked.max(1e-9)
    );
}
