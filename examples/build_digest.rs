//! **An exact cross-binary identity check** — `build_digest [horizon] [seeds]`.
//!
//! `examples/tput` prints population at six decimals, which is a fine *report*
//! and a bad *identity*: two builds can agree to six places and differ in the
//! last bit, and the last bit is what `tests/determinism.rs` is about. This
//! prints raw `f64` bits, so two runs are identical or they are not.
//!
//! The question it exists to answer: does changing `-C target-cpu` change the
//! simulation? Rust sets `fp-contract=off` and does not reassociate float
//! reductions without fast-math, so the answer *should* be no even with `+fma`
//! and AVX-512 available — but `AGENTS.md` §4 makes bit-identity a hard
//! requirement and §2 says to measure rather than to reason about it.
use hyades_engine::prelude::*;

fn main() {
    let h: f64 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(400.0);
    let seeds: Vec<u64> = match std::env::args().nth(2) {
        Some(v) => v.split(',').filter_map(|s| s.trim().parse().ok()).collect(),
        None => vec![1, 7, 42],
    };
    for seed in seeds {
        let mut cfg = SimConfig::new(seed);
        cfg.horizon_years = h;
        let mut sim = Simulation::with_baseline(Galaxy::generate(GalaxyConfig::new(3, seed)).unwrap(), cfg);
        let t0 = std::time::Instant::now();
        let r = sim.run();
        let secs = t0.elapsed().as_secs_f64();
        let col: usize = r.players.iter().map(|p| p.colonies).sum();
        let pop: f64 = r.players.iter().map(|p| p.total_population.kilotons()).sum();
        // Fold every player's population into one order-independent-by-construction
        // digest: the sum is taken in player order, which is fixed.
        println!(
            "seed {seed}: events {:>8}  colonies {col:>5}  popbits {:016x}  | {:>6.1} yr/s",
            r.events_processed,
            pop.to_bits(),
            h / secs
        );
    }
}
