//! **Why is the sink unspent? Count the reason, per decision.**
//!
//! R-O85 said infrastructure was priced out of reach. R-O88 refuted the two
//! facts that claim rested on — `slips` is unbounded now and the saturation is
//! per berth — and re-pricing the ladder against what it *buys* leaves nothing
//! of the third:
//!
//! | rung | stock | berths | output | step cost | **payback** |
//! |---|---|---|---|---|---|
//! | I | 0.10 kt | 2 | 0.0500 kt/yr | 0.90 kt | — |
//! | II | 1.00 kt | 17 | 0.5484 kt/yr | 19.0 kt | **1.8 yr** |
//! | III | 20.0 kt | 334 | 11.11 kt/yr | 780 kt | **1.8 yr** |
//! | IV | 800 kt | 13,334 | 444.4 kt/yr | — | **1.8 yr** |
//!
//! Cost and output are both geometric in the stock, so the return is **constant
//! at every rung**: infrastructure pays for itself in 1.8 years, at any scale.
//! Meanwhile the bed banks **1,714,697 kt** and sits at mean Band **1.442**
//! against a ceiling of 3.612, with **none** at cap. The money is there, the
//! return is enormous, and it is not being spent.
//!
//! So the question is no longer "what does it cost" but **"which line declines
//! it"**. `production_choice` can refuse a rung in exactly three places, and
//! this counts them against every decision where deepening was *possible*:
//!
//! - **gated** — `infra >= k_potential`, so the world allows no more. Not a
//!   defect, a ceiling.
//! - **colour-short** — the bill is payable in *named colours* (T-73) and this
//!   centre lacks one. Read off `can_afford_infra`, the predicate the decision
//!   itself used, rather than reconstructed from totals: the galaxy's supply is
//!   single-coloured (mean dominant share 0.789), so a centre can hold a hundred
//!   times the bill and still be unable to pay it, and inferring affordability
//!   from a total counts that as a *choice* not to deepen.
//! - **ore-short** — short on the total as well, so colour is not the whole of
//!   it.
//! - **outbid** — it could afford it and chose something else. That is the
//!   deepen-vs-expand comparison (R-O68), and it is the only one of the three
//!   that is a policy decision rather than a fact about the world.
//!
//! Run: `cargo run --release --example deepen_blockers -- [seed] [horizon]`
use hyades_engine::autopilot::BuildOrder;
use hyades_engine::log::{LogCategory, LogEvent, LogFilter};
use hyades_engine::prelude::*;
use std::io::Write;

const PLAYERS: usize = 3;

fn main() {
    let mut args = std::env::args().skip(1);
    let seed: u64 = args.next().and_then(|s| s.parse().ok()).unwrap_or(1);
    let horizon: f64 = args.next().and_then(|s| s.parse().ok()).unwrap_or(1500.0);

    println!("deepen blockers — seed {seed}, {PLAYERS} seats, {horizon:.0} yr");
    std::io::stdout().flush().ok();

    let galaxy = Galaxy::generate(GalaxyConfig::new(PLAYERS, seed)).unwrap();
    let mut cfg = SimConfig::new(seed);
    cfg.horizon_years = horizon;
    let mut sim = Simulation::with_baseline(galaxy, cfg);
    sim.set_log_filter(LogFilter::none().with(LogCategory::Production));
    sim.run();

    let (mut total, mut gated, mut unaffordable, mut outbid, mut bought) = (0u64, 0u64, 0u64, 0u64, 0u64);
    // Of the unaffordable ones, how many held the *total* and lacked only a
    // colour. That is the split T-73 predicts and nothing has measured.
    let mut colour_only = 0u64;
    // Of the outbid ones, what did they buy instead?
    let (mut to_hull, mut to_idle) = (0u64, 0u64);
    // And how rich were they when they declined? A centre that is outbid while
    // holding many times the bill is the strongest form of the finding.
    let mut wealth: Vec<f64> = Vec::new();
    for r in sim.log().iter() {
        if let LogEvent::ProductionDecision {
            infra,
            k_potential,
            stockpile,
            infra_cost,
            can_afford_infra,
            chosen,
            ..
        } = r.event
        {
            total += 1;
            if chosen == BuildOrder::UpgradeInfrastructure {
                bought += 1;
                continue;
            }
            if infra >= k_potential - 1e-9 {
                gated += 1;
            } else if !can_afford_infra {
                unaffordable += 1;
                if stockpile >= infra_cost {
                    colour_only += 1;
                }
            } else {
                outbid += 1;
                if infra_cost > 0.0 {
                    wealth.push(stockpile / infra_cost);
                }
                match chosen {
                    BuildOrder::Hull { .. } => to_hull += 1,
                    BuildOrder::Idle => to_idle += 1,
                    BuildOrder::UpgradeInfrastructure => {}
                }
            }
        }
    }

    let pct = |n: u64| 100.0 * n as f64 / total.max(1) as f64;
    println!("\n{total} production decisions");
    println!("  bought a rung          {bought:>9}  {:>6.2}%", pct(bought));
    println!(
        "  gated at the ceiling   {gated:>9}  {:>6.2}%   (infra >= k_potential — a fact, not a defect)",
        pct(gated)
    );
    println!(
        "  could not pay the bill {unaffordable:>9}  {:>6.2}%   of which COLOUR-SHORT ONLY: {colour_only} ({:.2}% of all)",
        pct(unaffordable),
        pct(colour_only)
    );
    println!("  **outbid**             {outbid:>9}  {:>6.2}%   -> hull {to_hull}, idle {to_idle}", pct(outbid));

    wealth.sort_by(|a, b| a.partial_cmp(b).unwrap());
    if !wealth.is_empty() {
        let q = |f: f64| wealth[((wealth.len() - 1) as f64 * f) as usize];
        println!(
            "\n  bank / rung price, over the outbid decisions:  p50 {:.1}x  p90 {:.1}x  p99 {:.1}x  max {:.1}x",
            q(0.5),
            q(0.9),
            q(0.99),
            wealth.last().copied().unwrap_or(0.0)
        );
        println!("  (a centre declining a rung it could buy {:.0} times over is the finding)", q(0.9));
    }
}
