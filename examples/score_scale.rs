//! **Are `w_deepen` and `w_expand` on the same scale?**
//!
//! `production_choice` picks depth over expansion when
//! `b · deepen_headroom >= (1 − b) · score`. The left side is a **Band**
//! difference, `k_potential − infra`, bounded by 4. The right side is
//! `rank`'s weighted score, which is a sum over a Band, a mineral density and
//! a hub figure — no bound, and no shared unit with the left.
//!
//! If the two sides do not overlap in range, `reinvest_bias` is not a
//! preference dial: it is a unit-conversion constant with a preference hidden
//! inside it, and the "convex trade" the comment claims is a fiction. This
//! prints both distributions so the claim is measured rather than asserted.
use hyades_engine::autopilot::{Autopilot, BaselineAutopilot, PlanetView, RankContext};
use hyades_engine::galaxy::{Galaxy, GalaxyConfig, PlanetClass};
use hyades_engine::prelude::*;

const PLAYERS: usize = 3;

fn pct(v: &[f64], q: f64) -> f64 {
    if v.is_empty() {
        return f64::NAN;
    }
    let i = ((v.len() - 1) as f64 * q).round() as usize;
    v[i]
}

fn main() {
    let galaxy = Galaxy::generate(GalaxyConfig::new(PLAYERS, 1)).unwrap();
    let doctrine = Doctrine::default();
    let ap = BaselineAutopilot::new(doctrine);
    let home = &galaxy.planets[galaxy.homeworlds[0].0 as usize];

    for pressure in [0.0, 1.0] {
        let ctx =
            RankContext { scarcity: [1.0, 1.0, 1.0], holdings_centroid: home.position, mineral_pressure: pressure };
        let mut colony = Vec::new();
        let mut mining = Vec::new();
        for p in &galaxy.planets {
            let view = PlanetView {
                id: p.id,
                position: p.position,
                habitability: p.habitability,
                biosphere: p.biosphere,
                minerals: p.minerals,
                owner: None,
                pop_level: 0,
            };
            let r = ap.rank(&doctrine, &view, &ctx);
            match r.class {
                PlanetClass::Colony | PlanetClass::ProductionCenter => colony.push(r.score),
                PlanetClass::MiningOutpost => mining.push(r.score),
                PlanetClass::Barren => {}
            }
        }
        colony.sort_by(|a, b| a.partial_cmp(b).unwrap());
        mining.sort_by(|a, b| a.partial_cmp(b).unwrap());
        println!("\nmineral_pressure = {pressure}");
        for (name, v) in [("colony/center", &colony), ("mining", &mining)] {
            println!(
                "  {name:<14} n={:<6} p05={:>8.2} p50={:>8.2} p95={:>8.2} max={:>9.2}",
                v.len(),
                pct(v, 0.05),
                pct(v, 0.50),
                pct(v, 0.95),
                v.last().copied().unwrap_or(f64::NAN)
            );
        }
    }

    // The other side of the comparison, at its absolute maximum.
    let max_headroom = galaxy.planets.iter().map(|p| p.habitability.min(p.biosphere).bands()).fold(0.0_f64, f64::max);
    println!("\n  deepen_headroom is bounded by k_potential - infra <= {max_headroom:.2} (Bands)");
    let b = doctrine.reinvest_bias;
    println!("  at reinvest_bias = {b}, depth wins only when headroom >= score");
}
