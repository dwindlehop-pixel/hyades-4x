//! 3-D vector math and the two delays the relativistic event scheduler needs.
//!
//! Units throughout the engine: **distance in light-years (ly), time in years**,
//! so `c = 1`. That choice makes the signal light-lag trivially equal to the
//! distance (`Hyades_card_contract.md` §2) and keeps every number human-legible.
//!
//! The theater "is fully deterministic … motion obeys special relativity:
//! velocities are bounded by *c*" (`Hyades_simulation_model.md` §1a). We do not
//! integrate ship trajectories frame-by-frame in this discrete-event core; we
//! only need *arrival times*, so we use the closed-form relativistic
//! constant-proper-acceleration (flip-and-burn) solution below.

/// Speed of light in engine units (ly / yr). Hard cap on every velocity.
pub const C: f64 = 1.0;

/// One standard gravity expressed in engine units (ly / yr²).
///
/// `g = 9.81 m/s²`, `1 yr = 3.1557e7 s`, `1 ly = 9.4607e15 m`
/// ⇒ `g = 9.81 · (3.1557e7)² / 9.4607e15 ≈ 1.0323 ly/yr²`.
/// A 1 g torchship thus reaches ≈ *c* in ≈ 1 year — the intuitive anchor.
pub const G: f64 = 1.0323;

/// A point or displacement in the continuous 3-D theater.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub const ZERO: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

    #[inline]
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Vec3 { x, y, z }
    }

    // Inherent `add`/`sub` rather than `impl std::ops::{Add, Sub}`: every call
    // site in the engine spells the operation out (`a.sub(b).scale(t)`), which
    // keeps the relativistic kinematics readable and matches the spec prose.
    // Operator impls would be a second, redundant surface, so the
    // should_implement_trait lint is silenced deliberately.
    #[inline]
    #[allow(clippy::should_implement_trait)]
    pub fn add(self, o: Vec3) -> Vec3 {
        Vec3::new(self.x + o.x, self.y + o.y, self.z + o.z)
    }

    #[inline]
    #[allow(clippy::should_implement_trait)]
    pub fn sub(self, o: Vec3) -> Vec3 {
        Vec3::new(self.x - o.x, self.y - o.y, self.z - o.z)
    }

    #[inline]
    pub fn scale(self, s: f64) -> Vec3 {
        Vec3::new(self.x * s, self.y * s, self.z * s)
    }

    #[inline]
    /// Right-handed cross product `self × o`. Added for the combat/arena
    /// station-keeping rotation (Rodrigues' formula in `combat.rs`).
    pub fn cross(self, o: Vec3) -> Vec3 {
        Vec3::new(self.y * o.z - self.z * o.y, self.z * o.x - self.x * o.z, self.x * o.y - self.y * o.x)
    }

    pub fn dot(self, o: Vec3) -> f64 {
        self.x * o.x + self.y * o.y + self.z * o.z
    }

    #[inline]
    pub fn norm(self) -> f64 {
        self.dot(self).sqrt()
    }

    #[inline]
    pub fn distance(self, o: Vec3) -> f64 {
        self.sub(o).norm()
    }

    /// Unit vector; returns `ZERO` for a zero-length input (no NaNs leak).
    #[inline]
    pub fn normalized(self) -> Vec3 {
        let n = self.norm();
        if n > 0.0 {
            self.scale(1.0 / n)
        } else {
            Vec3::ZERO
        }
    }

    /// The six cube-face headings (±X/±Y/±Z) the survey fan-out uses
    /// (`Hyades_autopilot_colonization_growth.md` §2).
    pub const CUBE_FACES: [Vec3; 6] = [
        Vec3 { x: 1.0, y: 0.0, z: 0.0 },
        Vec3 { x: -1.0, y: 0.0, z: 0.0 },
        Vec3 { x: 0.0, y: 1.0, z: 0.0 },
        Vec3 { x: 0.0, y: -1.0, z: 0.0 },
        Vec3 { x: 0.0, y: 0.0, z: 1.0 },
        Vec3 { x: 0.0, y: 0.0, z: -1.0 },
    ];
}

/// **Signal light-lag**: the years for information (a scan result, an order
/// reaching its actors) to cross `distance` ly. Equals the distance, since
/// `c = 1`. This is the delay on *every causal edge* of the schedule
/// (`Hyades_card_contract.md` §2).
#[inline]
pub fn signal_delay_years(distance_ly: f64) -> f64 {
    distance_ly / C
}

/// **The lower end of [`exp_decay`]'s fitted range.** Arguments below this fall
/// back to [`crate::transcendental::exp`].
///
/// Measured, not assumed: the only caller is the ranking hot path, where the
/// argument is `−distance / centrality_scale`. Histogrammed over full runs it
/// spans **[−1.735, 0]** — 91.7% of calls in [−1, 0) and the rest in [−2, −1) —
/// and the most negative value seen across 2, 3, 12 and 18 seats is **−1.735**.
/// So `−2` is the measured extent plus 15%, and anything past it is somebody
/// else's galaxy, where correctness comes from the fallback rather than from
/// this bound being right.
pub const EXP_DECAY_MIN: f64 = -2.0;

/// Minimax coefficients for `exp` on `[EXP_DECAY_MIN, 0]`, degree 7, ascending.
///
/// Fitted by Remez exchange in the Chebyshev basis and converted to monomials;
/// the max error below is *measured* on a 200,001-point grid, not predicted.
/// They read as `1, 1, 1/2, 1/6, 1/24, …` because that is what they are — a
/// truncated exponential with the truncation error spread evenly across the
/// interval instead of piled up at the far end.
///
/// Written at the shortest decimal that round-trips to the fitted `f64`, which
/// is also what clippy's `excessive_precision` insists on: the extra digits the
/// fitter emitted named the same bits and read as accuracy that was not there.
const EXP_DECAY_C: [f64; 8] = [
    0.999_999_926_488_396_5,
    0.999_995_157_993_455_1,
    0.499_947_599_552_087_9,
    0.166_449_614_576_505_6,
    0.041_220_952_478_091_68,
    0.007_827_208_892_832_536,
    0.001_058_436_443_408_709_3,
    7.556_538_929_505_768e-5,
];

/// **`exp(x)` for the decay range, by polynomial** — 2.7x `f64::exp` at a
/// maximum relative error of **5.4e-7** (T-102).
///
/// The ranking hot path evaluates `exp(−distance / centrality_scale)` **45.4
/// million times** over an 800-year three-seat run, and the result is a
/// *classification weight*: it discounts a world's `k_potential` by distance and
/// the product is compared against `hub_high`. Seven significant figures is
/// several more than a weight in that position can spend.
///
/// **Estrin, not Horner, and that is the whole speed difference.** Both evaluate
/// the same degree-7 polynomial to the same bits-ish accuracy, but Horner is a
/// serial chain of seven dependent multiply-adds while Estrin splits it into
/// four independent pairs combined in two levels. Measured on this machine over
/// 65,536 inputs drawn from the real range:
///
/// | | ns/call | max relative error |
/// |---|---|---|
/// | `f64::exp` | 5.24 | — |
/// | Horner, degree 5 | 1.98 | 1.2e-4 |
/// | Horner, degree 7 | 2.98 | 5.4e-7 |
/// | Horner, degree 9 | 4.05 | 1.5e-9 |
/// | **Estrin, degree 7** | **1.97** | **5.4e-7** |
///
/// Estrin at degree 7 costs what Horner costs at degree 5 and is 220x more
/// accurate, so the dependency chain — not the multiply count — was the price.
///
/// **It was also the first host transcendental the engine removed** (T-102):
/// `f64::exp` is the platform libm natively and a Rust libm on wasm32, and the
/// two disagree in the last bit. T-127 removed the rest
/// ([`crate::transcendental`]), so determinism no longer depends on this
/// function and it is kept for speed. A polynomial of `+` and `*` is exactly
/// specified by IEEE 754 and identical everywhere by construction. **No
/// `mul_add`**: a fused multiply-add rounds once where a separate multiply and
/// add round twice, so mixing the two across targets reintroduces a
/// divergence.
///
/// Outside the fitted range — below [`EXP_DECAY_MIN`], above zero, or `NaN` —
/// it defers to [`crate::transcendental::exp`], so the function is correct on all of `f64` and only
/// *fast* where it was measured to matter. The upper guard is not decoration:
/// the polynomial is a minimax fit on a closed interval and says nothing at all
/// about `x > 0`, where it passes 1% relative error by `x = 1`. The caller that
/// motivated this passes `−distance / scale`, so neither branch is ever taken;
/// the next caller may not be so tidy, and a fast wrong answer is worse than the
/// two compares.
#[inline]
pub fn exp_decay(x: f64) -> f64 {
    // Written as a negated `in range` so a `NaN` argument falls through to
    // `transcendental::exp` and propagates rather than indexing into the polynomial.
    if !(EXP_DECAY_MIN..=0.0).contains(&x) {
        return crate::transcendental::exp(x);
    }
    let c = &EXP_DECAY_C;
    let x2 = x * x;
    let x4 = x2 * x2;
    // Four independent pairs, then two levels of combination: depth 4 against
    // Horner's 7, with the same operand count.
    let a = c[0] + c[1] * x;
    let b = c[2] + c[3] * x;
    let d = c[4] + c[5] * x;
    let e = c[6] + c[7] * x;
    (a + b * x2) + (d + e * x2) * x4
}

/// **Ship travel time**: years for a torchship to cross `distance` ly under a
/// symmetric flip-and-burn at constant proper acceleration `accel` (ly/yr²),
/// starting and ending at rest.
///
/// Closed form for one constant-proper-acceleration leg of length `x` from rest
/// is `t(x) = sqrt((x/c)² + 2x/a)`; flip-and-burn does two legs of `d/2`:
///
/// ```text
/// t(d) = 2 · sqrt( (d/2c)² + 2·(d/2)/a ) = sqrt( (d/c)² + 4d/a ).
/// ```
///
/// As `d → ∞` this tends to `d/c` *from above* — never beating light, exactly
/// the `c`-cap the sim mandates (`Hyades_simulation_model.md` §1a). For small
/// `d` it is dominated by the `4d/a` term (the slow-boat regime).
#[inline]
pub fn ship_travel_years(distance_ly: f64, accel: f64) -> f64 {
    debug_assert!(accel > 0.0, "acceleration must be positive");
    let d = distance_ly.max(0.0);
    let q = d / C;
    (q * q + 4.0 * d / accel).sqrt()
}

/// Distance covered from rest under constant proper acceleration `accel` after
/// `tau` years: `x(τ) = (c²/a)(√(1 + (aτ/c)²) − 1)`.
#[inline]
fn accel_leg_distance(accel: f64, tau: f64) -> f64 {
    let t = tau.max(0.0);
    let q = accel * t / C;
    (C * C / accel) * ((1.0 + q * q).sqrt() - 1.0)
}

/// Exact along-track distance covered at elapsed time `tau` into a symmetric
/// **flip-and-burn** of total span `total_time` over total `distance_ly`
/// (accelerate to the midpoint, decelerate after). Deterministic and monotone in
/// `tau`; clamps to `[0, distance]`. This is what gives every in-flight entity a
/// precise position at any moment.
pub fn flight_distance(accel: f64, tau: f64, total_time: f64, distance_ly: f64) -> f64 {
    if total_time <= 0.0 || distance_ly <= 0.0 {
        return 0.0;
    }
    let t = tau.clamp(0.0, total_time);
    let half = total_time / 2.0;
    if t <= half {
        accel_leg_distance(accel, t).min(distance_ly)
    } else {
        (distance_ly - accel_leg_distance(accel, total_time - t)).clamp(0.0, distance_ly)
    }
}

/// Position of a body flip-and-burning from `origin` to `dest`, departing at
/// `depart` and arriving at `arrive`, evaluated at absolute time `t`.
pub fn position_along(origin: Vec3, dest: Vec3, depart: f64, arrive: f64, accel: f64, t: f64) -> Vec3 {
    let total_time = arrive - depart;
    let d = origin.distance(dest);
    if total_time <= 0.0 || d <= 0.0 {
        return dest;
    }
    let x = flight_distance(accel, t - depart, total_time, d);
    origin.add(dest.sub(origin).normalized().scale(x))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn travel_never_beats_light() {
        for &d in &[0.1, 1.0, 10.0, 100.0, 1000.0] {
            let t = ship_travel_years(d, G);
            assert!(t > signal_delay_years(d), "ship beat light at d={d}");
        }
    }

    #[test]
    fn travel_approaches_light_for_large_distance() {
        // At 10_000 ly the flip-and-burn is within ~1% of the light time.
        let d = 10_000.0;
        let ratio = ship_travel_years(d, G) / signal_delay_years(d);
        assert!(ratio < 1.01, "ratio {ratio} not near 1");
    }

    #[test]
    fn one_ly_takes_about_two_years_at_1g() {
        let t = ship_travel_years(1.0, G);
        assert!((t - 2.0).abs() < 0.25, "1 ly at 1g was {t} yr");
    }

    #[test]
    fn flight_interpolation_is_monotone_and_bracketed() {
        let o = Vec3::new(0.0, 0.0, 0.0);
        let d = Vec3::new(30.0, 40.0, 0.0); // 50 ly
        let dist = o.distance(d);
        let arrive = ship_travel_years(dist, G);
        let mut prev = -1.0;
        let mut t = 0.0;
        while t <= arrive {
            let p = position_along(o, d, 0.0, arrive, G, t);
            let along = o.distance(p);
            assert!(along >= prev - 1e-9, "not monotone at t={t}");
            assert!(along <= dist + 1e-6, "overshoot at t={t}");
            prev = along;
            t += arrive / 50.0;
        }
        // endpoints land exactly.
        assert!(position_along(o, d, 0.0, arrive, G, 0.0).distance(o) < 1e-6);
        assert!(position_along(o, d, 0.0, arrive, G, arrive).distance(d) < 1e-6);
    }

    /// The accuracy claim in [`exp_decay`]'s doc comment, asserted rather than
    /// asserted-in-prose. 1e-6 is the *bound* the caller was accepted against;
    /// the fit measures 5.4e-7, so there is 1.9x of headroom and a refit that
    /// silently loses a digit fails here instead of in a balance measurement.
    #[test]
    fn exp_decay_holds_its_error_bound_over_the_fitted_range() {
        const STEPS: u32 = 200_000;
        let mut worst = 0.0f64;
        let mut worst_at = 0.0f64;
        for i in 0..=STEPS {
            let x = EXP_DECAY_MIN * (f64::from(STEPS - i) / f64::from(STEPS));
            let want = crate::transcendental::exp(x);
            let rel = ((exp_decay(x) - want) / want).abs();
            if rel > worst {
                worst = rel;
                worst_at = x;
            }
        }
        assert!(worst <= 1e-6, "max relative error {worst:e} at x={worst_at}");
    }

    /// Outside the fitted range the answer must be the library's, **bit for
    /// bit** — not merely close. A fallback that is approximately right is a
    /// second approximation nobody measured, and the guard is the only thing
    /// keeping the polynomial's domain honest.
    #[test]
    fn exp_decay_defers_to_the_library_outside_the_fitted_range() {
        for &x in &[
            // Just past the guard: `EPSILON` is an ulp at 1.0, and the ulp
            // at 2.0 is twice that, so a bare `MIN - EPSILON` rounds back onto
            // the boundary and tests nothing.
            EXP_DECAY_MIN - 4.0 * f64::EPSILON,
            -2.5,
            -10.0,
            -745.0,
            f64::NEG_INFINITY,
            f64::MIN,
            f64::EPSILON,
            0.5,
            1.0,
            700.0,
            f64::INFINITY,
        ] {
            assert_eq!(exp_decay(x).to_bits(), crate::transcendental::exp(x).to_bits(), "x={x}");
        }
        assert!(exp_decay(f64::NAN).is_nan());
    }

    /// The fit is monotone on the range and brackets the values the caller
    /// treats as anchors — a weight that ran backwards in distance would be a
    /// ranking inversion, which no error bound on its own rules out.
    #[test]
    fn exp_decay_is_monotone_and_anchored() {
        // The fit does not interpolate its endpoints — a minimax polynomial
        // spreads the error evenly rather than pinning it to zero anywhere — so
        // `exp_decay(0.0)` is 1.0 to within the bound, not 1.0.
        assert!((exp_decay(0.0) - 1.0).abs() <= 1e-6);
        let mut prev = f64::INFINITY;
        for i in 0..=2_000 {
            let x = EXP_DECAY_MIN * (f64::from(i) / 2_000.0);
            let y = exp_decay(x);
            assert!(y <= prev, "not monotone decreasing at x={x}");
            assert!(y > 0.0, "non-positive weight at x={x}");
            prev = y;
        }
    }
}
