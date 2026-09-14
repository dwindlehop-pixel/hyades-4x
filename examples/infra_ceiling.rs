//! **Are colonies anywhere near their infrastructure ceiling, and is the galaxy
//! scouted?**
//!
//! The two questions that decide whether R-IND21's "there is no demand for
//! minerals" is a fact about the *economy* or a fact about the *policy*.
//!
//! §6.17 measured that 91–98% of held ore sits idle while unmet demand is ~1/1000
//! of the pile, and concluded the mineral economy has no demand side. That
//! conclusion is only sound if the obvious sinks are actually full. They are the
//! two this prints:
//!
//! - **Infrastructure.** `k` is a world's ceiling and `infrastructure` is what it
//!   has built. If colonies sit far below `k` with ore in the bank, the demand is
//!   *there* and something is declining to spend it — which is a policy fault,
//!   not an economic one. R-O68 already names a candidate: `reinvest_bias`
//!   compares a Band difference against an unbounded rank score, so the deepen
//!   branch cannot fire at the shipped value while any candidate exists.
//! - **Survey.** If worlds go unscanned, scouts are a sink too.
//!
//! Run: `cargo run --release --example infra_ceiling`
use hyades_engine::prelude::*;
use std::io::Write;

const SEEDS: [u64; 2] = [1, 7];
const PLAYERS: usize = 3;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let planets: Option<usize> = args.first().and_then(|a| a.parse().ok());
    let horizon: f64 = args.get(1).and_then(|a| a.parse().ok()).unwrap_or(4000.0);

    println!("infrastructure ceiling and survey coverage — {PLAYERS} seats, {horizon:.0} yr\n");
    std::io::stdout().flush().ok();

    for seed in SEEDS {
        let mut gcfg = GalaxyConfig::new(PLAYERS, seed);
        if let Some(n) = planets {
            gcfg.planet_count = n;
        }
        let galaxy = Galaxy::generate(gcfg).unwrap();
        let total_planets = galaxy.planets.len();
        let mut cfg = SimConfig::new(seed);
        cfg.horizon_years = horizon;
        let mut sim = Simulation::with_baseline(galaxy, cfg);
        let report = sim.run();
        let snap = sim.snapshot();

        let owned: Vec<_> = snap.planets.iter().filter(|p| p.owner.is_some()).collect();
        let n = owned.len().max(1) as f64;
        let mean_infra = owned.iter().map(|p| p.infrastructure.bands()).sum::<f64>() / n;
        let mean_k = owned.iter().map(|p| p.k.bands()).sum::<f64>() / n;
        // Headroom is what a world could still build before hitting its own
        // ceiling — the latent sink, in Bands.
        let headroom = owned.iter().map(|p| (p.k.bands() - p.infrastructure.bands()).max(0.0)).sum::<f64>();
        let at_ceiling = owned.iter().filter(|p| p.infrastructure.bands() + 0.05 >= p.k.bands()).count();
        let banked = owned.iter().map(|p| p.stockpile.basic_total().kilotons()).sum::<f64>();

        println!("seed {seed}:  {} colonies of {total_planets} planets", owned.len());
        println!(
            "  infrastructure: mean Band {mean_infra:.3}  against mean ceiling k {mean_k:.3}   \
             at ceiling: {at_ceiling} ({:.1}%)",
            100.0 * at_ceiling as f64 / n
        );
        println!(
            "  latent headroom: {headroom:.0} Bands unbuilt across the empire; {banked:.0} kt banked to build it with"
        );
        println!(
            "  survey: {} scanned of {total_planets} ({:.1}%)",
            report.planets_scanned_total,
            100.0 * report.planets_scanned_total as f64 / total_planets as f64
        );
        std::io::stdout().flush().ok();
    }
}
