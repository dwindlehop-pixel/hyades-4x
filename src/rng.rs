//! Deterministic, seedable PRNG — the engine's *only* source of randomness.
//!
//! Reproducibility is a hard requirement: the Monte-Carlo balancer reruns the
//! same `seed × matchup` and must get identical results on native and wasm32
//! (`Hyades_card_contract.md` §7). So we never touch the OS RNG; we use
//! `splitmix64` (Vigna), which is tiny, fast, well-distributed, and identical on
//! every platform because it is pure `u64` wrapping arithmetic.

/// A splitmix64 generator. Cheap to clone and to [`fork`](Rng::fork) into
/// independent sub-streams (e.g. one per player, per planet).
#[derive(Clone, Debug)]
pub struct Rng {
    state: u64,
}

impl Rng {
    /// Seed the generator. Any `u64` is a valid seed.
    pub fn new(seed: u64) -> Self {
        Rng { state: seed }
    }

    #[inline]
    fn next_u64(&mut self) -> u64 {
        // splitmix64
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform in `[0, 1)` with 53 bits of mantissa.
    #[inline]
    pub fn unit(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Uniform in `[lo, hi)`.
    #[inline]
    pub fn range(&mut self, lo: f64, hi: f64) -> f64 {
        lo + (hi - lo) * self.unit()
    }

    /// Uniform integer in `[0, n)`.
    #[inline]
    pub fn below(&mut self, n: usize) -> usize {
        if n == 0 {
            return 0;
        }
        (self.next_u64() % (n as u64)) as usize
    }

    /// Standard normal `N(0, 1)` via Box–Muller (one of the pair).
    #[inline]
    pub fn gaussian(&mut self) -> f64 {
        // Guard the log against u1 == 0.
        let u1 = (self.unit()).max(1e-18);
        let u2 = self.unit();
        (-2.0 * u1.ln()).sqrt() * (core::f64::consts::TAU * u2).cos()
    }

    /// Derive an independent sub-stream tagged by `label`. Mixing the label into
    /// the seed keeps per-entity streams reproducible *and* decorrelated.
    pub fn fork(&self, label: u64) -> Rng {
        let mut s = self.state ^ label.wrapping_mul(0x9E37_79B9_7F4A_7C15);
        // one round of avalanche so nearby labels diverge immediately
        s = (s ^ (s >> 33)).wrapping_mul(0xFF51_AFD7_ED55_8CCD);
        Rng::new(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_stream() {
        let mut a = Rng::new(42);
        let mut b = Rng::new(42);
        for _ in 0..1000 {
            assert_eq!(a.unit().to_bits(), b.unit().to_bits());
        }
    }

    #[test]
    fn unit_in_range() {
        let mut r = Rng::new(7);
        for _ in 0..100_000 {
            let u = r.unit();
            assert!((0.0..1.0).contains(&u));
        }
    }

    #[test]
    fn gaussian_mean_near_zero() {
        let mut r = Rng::new(99);
        let n = 200_000;
        let mean: f64 = (0..n).map(|_| r.gaussian()).sum::<f64>() / n as f64;
        assert!(mean.abs() < 0.02, "mean {mean}");
    }

    #[test]
    fn forks_decorrelate() {
        let base = Rng::new(1);
        let mut a = base.fork(1);
        let mut b = base.fork(2);
        let diffs = (0..1000).filter(|_| a.unit() != b.unit()).count();
        assert!(diffs > 990);
    }
}
