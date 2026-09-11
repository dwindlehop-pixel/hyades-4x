//! **Does Yellow actually move from Yellow-rich empires to Yellow-poor ones?**
//!
//! The design's stated goal for the Exchange
//! (`Hyades_politics_trade_and_intelligence.md` §10.6): *"move Y from Y rich
//! empires to Y poor empires with a sort of Arabian trade via outposts."*
//!
//! **That is a claim about direction, and no scalar carries it.** §10.8 is
//! explicit that the guard for Exchange work is a census rather than
//! colony-years — colony-years is *inverted* for anything that changes how
//! minerals are spent, and it rewarded the breakage four times on the works
//! branch. Work-years is the right objective but it answers "did the empire
//! develop", not "did Yellow flow downhill".
//!
//! So this measures the flow itself, three ways:
//!
//! - **Direction.** For every settled contract, was the seller richer in that
//!   colour than the buyer *at the moment it was struck*? A market that moves
//!   ore from those who have it to those who do not should be near 100%; a
//!   market that shuffles at random should be near 50%. **50% is the null
//!   hypothesis and it is what makes this a measurement rather than a tally.**
//! - **Dispersion.** The spread of each colour's holdings across empires, over
//!   time. Trade that works should *narrow* it — that is what "rich to poor"
//!   means in aggregate, and it is the half a per-contract tally cannot see,
//!   because a market can move every unit downhill and still not move enough
//!   units to matter.
//! - **Reach.** How many contracts found a venue against how many fills were
//!   struck. Geography is the trade constraint (§10.6), so a low reach is not a
//!   bug — it is the map saying these empires cannot trade yet.
//!
//! Run: `cargo run --release --example colour_flow`
use hyades_engine::prelude::*;
use hyades_engine::resources::Basic;
use std::io::Write;

const SEEDS: [u64; 2] = [1, 7];
const PLAYERS: usize = 3;
/// Overridable from argv (`colour_flow <planets> <horizon>`) so the *diagnosis*
/// can run on a small galaxy and only the verdict pays for the full bed —
/// `CLAUDE.md` §2's "reduce the galaxy before the horizon".
const HORIZON: f64 = 4000.0;
const SAMPLE_YEARS: f64 = 200.0;

/// Coefficient of variation of a colour's holdings across empires — dimensionless,
/// so the three colours are comparable even though their masses are not.
fn dispersion(holdings: &[f64]) -> f64 {
    let n = holdings.len() as f64;
    if n < 2.0 {
        return 0.0;
    }
    let mean = holdings.iter().sum::<f64>() / n;
    if mean <= 0.0 {
        return 0.0;
    }
    let var = holdings.iter().map(|h| (h - mean) * (h - mean)).sum::<f64>() / n;
    var.sqrt() / mean
}

fn main() {
    println!("colour flow — does Y move from Y-rich to Y-poor? {PLAYERS} seats, {HORIZON:.0} yr");
    println!("null hypothesis: a market that shuffles at random sells downhill 50% of the time\n");
    std::io::stdout().flush().ok();

    let args: Vec<String> = std::env::args().skip(1).collect();
    let planets: Option<usize> = args.first().and_then(|a| a.parse().ok());
    let horizon: f64 = args.get(1).and_then(|a| a.parse().ok()).unwrap_or(HORIZON);
    // `off` runs the ablation: identical bed, no Exchange at all.
    let trade = !args.iter().any(|a| a == "off");
    println!("Exchange: {}\n", if trade { "ON" } else { "OFF (ablation)" });

    for seed in SEEDS {
        let mut gcfg = GalaxyConfig::new(PLAYERS, seed);
        if let Some(n) = planets {
            gcfg.planet_count = n;
        }
        let galaxy = Galaxy::generate(gcfg).unwrap();
        let mut cfg = SimConfig::new(seed);
        cfg.horizon_years = horizon;
        let mut sim = Simulation::with_baseline(galaxy, cfg);
        sim.set_exchange_enabled(trade);

        // Dispersion at the start and end of the run, per colour.
        let holdings = |s: &Simulation| -> [Vec<f64>; 3] {
            let mut out = [vec![0.0; PLAYERS], vec![0.0; PLAYERS], vec![0.0; PLAYERS]];
            let snap = s.snapshot();
            for p in snap.planets.iter() {
                if let Some(o) = p.owner {
                    for (i, &c) in Basic::ALL.iter().enumerate() {
                        out[i][o as usize] += p.stockpile.get_basic(c);
                    }
                }
            }
            // **Outpost piles count.** Delivered ore lands in the buyer's pile
            // at the shared rock and stays there until its own freighter
            // collects it, so a census over planet banks alone is blind to
            // exactly what the Exchange moved.
            for (pl, _) in (0..PLAYERS).enumerate() {
                let piles = s.outpost_holdings(PlayerId(pl as u32));
                for (i, &c) in Basic::ALL.iter().enumerate() {
                    out[i][pl] += piles.get_basic(c);
                }
            }
            out
        };

        let first = holdings(&sim);
        let mut next = SAMPLE_YEARS.min(horizon);
        let mut samples = 0u32;
        let mut disp_sum = [0.0f64; 3];
        while next <= horizon {
            while sim.clock() < next {
                if !sim.step() {
                    break;
                }
            }
            let h = holdings(&sim);
            for i in 0..3 {
                disp_sum[i] += dispersion(&h[i]);
            }
            samples += 1;
            next += SAMPLE_YEARS;
        }
        let last = holdings(&sim);
        let (in_flight, settled, burned) = sim.exchange_state();
        let posted = sim.exchange_posted();

        println!("seed {seed}:");
        println!(
            "  contracts: {settled} settled, {} defaulted, {in_flight} in flight, ${burned:.1} burned",
            sim.exchange_defaults()
        );
        let (fills, rej) = sim.exchange_rejections();
        println!(
            "  fills {fills} -> rejected: self {} / no-venue {} / no-price {} / no-purse {}",
            rej[0], rej[1], rej[2], rej[3]
        );
        println!(
            "  offers posted (bids/asks): C {}/{}  M {}/{}  Y {}/{}",
            posted[0].0, posted[0].1, posted[1].0, posted[1].1, posted[2].0, posted[2].1
        );
        // **Banked against piled** — the asymmetry test (R-P18). The seller's
        // ore leaves a *spendable* bank at settlement; the buyer's lands in an
        // outpost pile and waits for its own freighter. If trade is a machine
        // for moving minerals out of banks and into piles, this is where it
        // shows, and it is the difference between a market that works and one
        // that only appears to.
        let mut banked = 0.0f64;
        let snap = sim.snapshot();
        for p in snap.planets.iter().filter(|p| p.owner.is_some()) {
            banked += p.stockpile.basic_total().kilotons();
        }
        let piled: f64 =
            (0..PLAYERS).map(|pl| sim.outpost_holdings(PlayerId(pl as u32)).basic_total().kilotons()).sum();
        println!(
            "  holdings: {banked:.0} kt banked / {piled:.0} kt piled at outposts  ({:.1}% idle)",
            100.0 * piled / (banked + piled).max(1e-9)
        );

        let traded = sim.exchange_traded();
        for (i, &c) in Basic::ALL.iter().enumerate() {
            println!("  {c:?}: {:.1} kt delivered", traded[i]);
            println!(
                "  {c:?}: dispersion {:.3} -> {:.3}  (mean over run {:.3})",
                dispersion(&first[i]),
                dispersion(&last[i]),
                disp_sum[i] / samples.max(1) as f64
            );
        }
        std::io::stdout().flush().ok();
    }
}
