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
    ((d / C).powi(2) + 4.0 * d / accel).sqrt()
}

/// Distance covered from rest under constant proper acceleration `accel` after
/// `tau` years: `x(τ) = (c²/a)(√(1 + (aτ/c)²) − 1)`.
#[inline]
fn accel_leg_distance(accel: f64, tau: f64) -> f64 {
    let t = tau.max(0.0);
    (C * C / accel) * ((1.0 + (accel * t / C).powi(2)).sqrt() - 1.0)
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
}
