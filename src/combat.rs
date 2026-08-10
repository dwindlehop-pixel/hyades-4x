//! Combat — the engine's real-space fighting model (`Hyades_simulation_model.md`
//! §4-5, `Hyades_loadout.md` §5). This is **engine-native**: the same code the
//! production game and the Monte-Carlo balancer both resolve fights with. The
//! Ship Testing Arena (`arena.rs`) is only a *scenario seeder* on top of this —
//! it fabricates combatants outside the production/economy constraints and
//! calls [`resolve_engagement`]; it owns no combat logic itself.
//!
//! Contents: the kinematic primitives (fleet trajectories, station-keeping,
//! Isaacs intercept, lasers with light-lag aim + point-defense, dodging
//! fuel-limited missiles, the BSP targeting index), the tuned weapon
//! parameters ([`CombatConfig`]), and the resolution loop ([`resolve_engagement`])
//! lifted out of what used to be the `laser_vs_missile` example so it lives in
//! the engine, not in a harness.
use crate::math::Vec3;
use crate::rng::Rng;
use crate::sim::{hull_base_thrust, hull_dry_mass, HullType, Role, SimConfig};
use std::f64::consts::{PI, TAU};

/// A fleet's reference trajectory — confirmed this conversation: *"the
/// first step is to define position of a fleet."* Deliberately the
/// simplest possible model, constant velocity, no acceleration: this is
/// the *formation center* individual ships hold station around, not a
/// ship itself, and it needs no engines of its own — the fleet's own
/// drift is just the aggregate of where its ships are headed.
#[derive(Clone, Copy, Debug)]
pub struct FleetTrajectory {
    pub origin: Vec3,
    pub velocity: Vec3,
}

impl FleetTrajectory {
    pub fn position_at(&self, t: f64) -> Vec3 {
        self.origin.add(self.velocity.scale(t))
    }
}

/// Confirmed this conversation, verbatim: *"Ships will maintain station
/// relative to a fleet's position & velocity, but they will orbit that
/// trajectory so their position and velocity are not predictable from
/// interstellar distances. We do not model the effects of gravity of
/// ships."*
///
/// **This is not an orbital-mechanics orbit** — there is no central mass
/// (confirmed: no ship-gravity), so nothing here integrates a real
/// two-body problem (no Keplerian elements, no Clohessy-Wiltshire
/// relative-motion equations — those both assume a gravitating primary).
/// It is a **designed periodic motion**, superimposed on the fleet's
/// linear drift: a circular path of fixed radius and angular rate in a
/// plane through the fleet center, seeded once per ship. The point is
/// exactly what's stated — a distant observer resolving only the fleet's
/// *aggregate* trajectory (the light-lag-delayed information every other
/// detection query in this engine already works from) cannot predict a
/// specific ship's *exact* instantaneous position or velocity without a
/// current, close-range scan, because the phase has evolved unobserved.
#[derive(Clone, Copy, Debug)]
pub struct StationKeeping {
    pub radius: f64,
    pub angular_velocity: f64, // radians / year
    pub phase: f64,
    pub(crate) axis: Vec3, // unit normal of the orbital plane (crate-visible for tests)
    reference: Vec3,       // unit vector in-plane; angle is measured from this
}

impl StationKeeping {
    /// Deterministic per-ship draw — same `Rng::fork(entity-keyed-label)`
    /// discipline `Simulation::draw_thrust_factor` already uses, so two
    /// runs with the same seed produce bit-identical station-keeping,
    /// independent of what else has drawn from the shared stream first.
    ///
    /// **R-ARENA2 (new, open):** the radius/period ranges below are
    /// placeholders — small enough that station-keeping is a local,
    /// sub-tactical jitter around the fleet reference point rather than a
    /// maneuver that competes with actual intercept burns, but the exact
    /// numbers are pending the arena's own results, same as every other
    /// placeholder this conversation introduces.
    pub fn draw(rng: &mut Rng, radius_range: (f64, f64), period_years_range: (f64, f64)) -> Self {
        let radius = rng.range(radius_range.0, radius_range.1);
        let period = rng.range(period_years_range.0, period_years_range.1).max(1e-9);
        let angular_velocity = TAU / period;
        let phase = rng.range(0.0, TAU);

        // A uniformly-random plane: pick axis via a random point on the
        // sphere, then any unit vector perpendicular to it as reference.
        let cos_theta = rng.range(-1.0, 1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).max(0.0).sqrt();
        let phi = rng.range(0.0, TAU);
        let axis = Vec3::new(sin_theta * phi.cos(), sin_theta * phi.sin(), cos_theta);
        let reference = arbitrary_perpendicular(axis);

        StationKeeping { radius, angular_velocity, phase, axis, reference }
    }

    fn angle_at(&self, t: f64) -> f64 {
        self.phase + self.angular_velocity * t
    }

    /// Offset from the fleet reference point at time `t`.
    pub fn offset_at(&self, t: f64) -> Vec3 {
        rotate_around_axis(self.reference, self.axis, self.angle_at(t)).scale(self.radius)
    }

    /// Rate of change of that offset — the ship's *own* velocity
    /// contribution on top of the fleet's drift.
    pub fn offset_velocity_at(&self, t: f64) -> Vec3 {
        let tangent = rotate_around_axis(self.reference, self.axis, self.angle_at(t) + PI / 2.0);
        tangent.scale(self.radius * self.angular_velocity)
    }
}

/// Rotate unit vector `v` (assumed perpendicular to `axis`) about unit
/// vector `axis` by `angle` radians — Rodrigues' rotation formula
/// (right-handed convention, matching `Vec3::cross`'s):
/// `v_rot = v·cos(θ) + (axis × v)·sin(θ)` when `v ⊥ axis` (the `(axis·v)
/// axis (1−cos θ)` term of the general formula vanishes). See references.
fn rotate_around_axis(v: Vec3, axis: Vec3, angle: f64) -> Vec3 {
    v.scale(angle.cos()).add(axis.cross(v).scale(angle.sin()))
}

/// Any unit vector perpendicular to `axis` (assumed already unit length) —
/// picks whichever of the world X/Y axes is *less* parallel to `axis`
/// (avoiding the degenerate near-parallel case) and projects it off.
fn arbitrary_perpendicular(axis: Vec3) -> Vec3 {
    let seed = if axis.x.abs() < 0.9 { Vec3::new(1.0, 0.0, 0.0) } else { Vec3::new(0.0, 1.0, 0.0) };
    seed.sub(axis.scale(seed.dot(axis))).normalized()
}

/// A ship in the arena — hull/role (driving its max acceleration, per the
/// propulsion model confirmed this conversation), which fleet it belongs
/// to, and its station-keeping offset around that fleet's trajectory.
/// **`role` is presently a placeholder:** `sim::Role`'s catalog doesn't
/// yet include Offensive/Tribute/RKV-strike (`Hyades_vehicle_roles.md`
/// §4.7 — "role behavior only, hull types not yet designed"), so a combat
/// ship here uses whichever existing `Role` is convenient (its mission
/// dispatch isn't read by anything in this module); only `hull` actually
/// drives the physics.
#[derive(Clone, Copy, Debug)]
pub struct Combatant {
    pub role: Role,
    pub hull: HullType,
    pub thrust_factor: f64,
    pub fleet: usize,
    pub station: StationKeeping,
    /// This ship's own maneuvering velocity delta (accumulated from
    /// chase/avoid burns) — separate from the fleet-drift + station-
    /// keeping motion, so a ship that breaks formation to pursue/flee
    /// still has a well-defined position: `true_position = fleet.
    /// position_at(t) + station.offset_at(t) + maneuver_velocity·(t −
    /// t_since_maneuver_start) + …`. See `Combatant::position_at`.
    pub maneuver_velocity: Vec3,
    pub maneuver_start: f64,
    pub maneuver_origin_offset: Vec3,
}

impl Combatant {
    /// This ship's maximum acceleration magnitude — the propulsion model
    /// confirmed this conversation (`sim::hull_base_thrust`/`hull_dry_mass`),
    /// with zero cargo/pop mass (arena ships are combatants, not haulers —
    /// R-ARENA3, open: should a Systems Vehicle's cargo count against its
    /// combat mass here? Zero for now, the simpler assumption).
    pub fn max_accel(&self, cfg: &crate::sim::SimConfig) -> f64 {
        hull_base_thrust(self.hull, cfg) * self.thrust_factor / hull_dry_mass(self.hull, cfg) * crate::math::G
    }

    /// Position at time `t`: fleet drift + station-keeping orbit + any
    /// accumulated maneuver displacement since the last course change.
    pub fn position_at(&self, fleets: &[FleetTrajectory], t: f64) -> Vec3 {
        let base = fleets[self.fleet].position_at(t).add(self.station.offset_at(t));
        let dt = (t - self.maneuver_start).max(0.0);
        base.add(self.maneuver_origin_offset).add(self.maneuver_velocity.scale(dt))
    }

    /// Velocity at time `t`: fleet drift + station-keeping's own rate +
    /// accumulated maneuver velocity.
    pub fn velocity_at(&self, fleets: &[FleetTrajectory], t: f64) -> Vec3 {
        fleets[self.fleet].velocity.add(self.station.offset_velocity_at(t)).add(self.maneuver_velocity)
    }
}

/// What a chaser is solving for — confirmed this conversation: *"There may
/// be distinctions based on the load-out of the chaser and the avoider
/// whether delta position = 0, some value x_max, or delta velocity = 0,
/// or some value v_max."* Four variants, all solved by
/// [`solve_intercept`]: `PositionZero`/`PositionWithin` for a chaser
/// closing to rendezvous or to weapons range; `VelocityZero`/
/// `VelocityWithin` for matching velocity (boarding, precise formation-
/// keeping) or merely getting the closing speed under control.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InterceptCriterion {
    /// Minimum time to `relative_position = 0` exactly.
    PositionZero,
    /// Minimum time to `|relative_position| ≤ x_max` (e.g. a weapon's range).
    PositionWithin(f64),
    /// Minimum time to `relative_velocity = 0` exactly (rendezvous/boarding).
    VelocityZero,
    /// Minimum time to `|relative_velocity| ≤ v_max`.
    VelocityWithin(f64),
}

/// A solved intercept: how long it takes, and the constant acceleration
/// direction (unit vector) that achieves it at the pursuer's maximum
/// thrust. `None` if the criterion is already satisfied (time ≤ 0) or
/// (for `PositionZero`/`PositionWithin` with zero available acceleration)
/// unreachable.
#[derive(Clone, Copy, Debug)]
pub struct InterceptSolution {
    pub time: f64,
    pub direction: Vec3,
}

/// **The chase/avoid core** — confirmed this conversation: *"A chaser
/// should be calculating & memoizing each tick the angle of interception
/// such that the minimum time flightpath to delta position = 0."*
///
/// This is Isaacs' **"isotropic rocket"** pursuit model — a point mass
/// with acceleration bounded in magnitude but free in direction (R. Isaacs,
/// *Differential Games*, Wiley, 1965; see also Lewin & Olsder, "The
/// isotropic rocket — a surveillance evasion game," *Computers & Mathematics
/// with Applications* 18(1), 1989). For the two position-based criteria,
/// the minimum-time solution uses **maximum available acceleration
/// throughout**, directed to exactly cancel where the pursuer would
/// otherwise coast relative to the target — the standard result for this
/// model (confirmed numerically solvable "through the solution of a
/// quartic equation at each instant of time," Bakolas & Tsiotras-style
/// analyses of the same model, e.g. Buzikov & Mayer, "Time-optimal feedback
/// control for the game of two Isotropic Rockets," *Systems & Control
/// Letters*, 2024). Rather than the closed-form quartic (which has
/// numerically awkward degenerate cases), this solves the equivalent
/// scalar root-finding problem `f(T) = |r₀ + v₀T| − ½aT² = 0` by
/// bisection: `f(0) = |r₀| > 0` and `f(T) → −∞` as `T → ∞`, so a root
/// always exists for any finite `r₀, v₀, a > 0`, and `f` is well-behaved
/// enough that bisection converges reliably — simpler to get right than a
/// quartic's multiple branches, and "recompute once a tick" (this
/// conversation's own framing) has no need for a closed form's speed
/// advantage.
///
/// **Newtonian, not relativistic — a scoped simplification, flagged, not
/// silently assumed.** `Hyades_simulation_model.md` §1a requires motion to
/// obey special relativity; this solver uses flat/Newtonian kinematics to
/// pick the burn *direction and duration*, because the general problem of
/// continuously-redirected thrust under exact relativistic composition
/// (thrust not aligned with current velocity) has no equivalently clean
/// treatment and combat ranges/closing speeds are expected to be far
/// below `c` in practice (unlike interstellar cruises, which the engine's
/// existing `math::position_along` already treats exactly). **R-ARENA4
/// (new, open):** revisit if arena results show intercepts occurring at
/// relativistic closing speeds.
pub fn solve_intercept(
    relative_position: Vec3,
    relative_velocity: Vec3,
    max_accel: f64,
    criterion: InterceptCriterion,
) -> Option<InterceptSolution> {
    match criterion {
        InterceptCriterion::VelocityZero => {
            let speed = relative_velocity.norm();
            if speed < 1e-12 {
                return None; // already matched
            }
            if max_accel <= 1e-12 {
                return None; // can never close a velocity gap with no thrust
            }
            Some(InterceptSolution { time: speed / max_accel, direction: relative_velocity.scale(-1.0 / speed) })
        }
        InterceptCriterion::VelocityWithin(v_max) => {
            let speed = relative_velocity.norm();
            if speed <= v_max {
                return None; // already inside tolerance
            }
            if max_accel <= 1e-12 {
                return None;
            }
            Some(InterceptSolution {
                time: (speed - v_max) / max_accel,
                direction: relative_velocity.scale(-1.0 / speed),
            })
        }
        InterceptCriterion::PositionZero => {
            solve_position_intercept(relative_position, relative_velocity, max_accel, 0.0)
        }
        InterceptCriterion::PositionWithin(x_max) => {
            if relative_position.norm() <= x_max {
                return None; // already inside tolerance
            }
            solve_position_intercept(relative_position, relative_velocity, max_accel, x_max)
        }
    }
}

/// Shared solver for `PositionZero`/`PositionWithin`: minimum `T` with
/// `|r₀ + v₀T| = tolerance + ½·a·T²`, found by bisection on
/// `f(T) = |r₀ + v₀T| − tolerance − ½aT²`.
fn solve_position_intercept(r0: Vec3, v0: Vec3, a: f64, tolerance: f64) -> Option<InterceptSolution> {
    if a <= 1e-12 {
        return None; // no thrust, no reachable intercept (unless already there, handled by callers)
    }
    let f = |t: f64| r0.add(v0.scale(t)).norm() - tolerance - 0.5 * a * t * t;

    // f(0) = |r0| - tolerance >= 0 by the caller's precondition; f -> -inf
    // as t grows (the -1/2 a t^2 term dominates), so bracket by doubling.
    let mut lo = 0.0;
    let mut hi = 1.0;
    while f(hi) > 0.0 {
        hi *= 2.0;
        if hi > 1e18 {
            return None; // shouldn't happen for a > 0, but never spin forever
        }
    }
    for _ in 0..100 {
        let mid = 0.5 * (lo + hi);
        if f(mid) > 0.0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let t = 0.5 * (lo + hi);
    if t <= 1e-12 {
        return None;
    }
    // The burn direction closes the coast-to-t position: with relative
    // position defined as target-minus-pursuer, r(t) = r0 + v0 t -
    // (1/2) u t^2 (the pursuer's own acceleration u *reduces* the
    // separation), so intercept requires u = +2(r0+v0 t)/t^2 — the SAME
    // sign as the coast vector, not its negation. (Verified numerically
    // before fixing: the negated version doubles the separation instead
    // of closing it — a real bug caught this conversation, not a
    // hypothetical one; `combat_arena.rs`'s and `Missile`'s existing
    // "apply sol.direction directly, no negation" usage is what's
    // correct, and is exactly what this fix makes true.)
    let coast = r0.add(v0.scale(t));
    let direction = if coast.norm() > 1e-12 { coast.scale(1.0 / coast.norm()) } else { Vec3::ZERO };
    Some(InterceptSolution { time: t, direction })
}

/// **Laser hit check** — confirmed this conversation: *"Lasers travel in a
/// straight line at the speed of light; effective at short distances but
/// easy to dodge at interstellar distances."* In this engine's own units
/// (distance in ly, time in years, so `c = 1`), a beam fired "at" a target
/// is really fired at that target's **predicted** position — the shooter
/// can only aim where the target *will be* after the beam's travel time,
/// by linearly extrapolating the target's velocity at the moment of
/// firing. The beam hits only if the target's *actual* position at
/// arrival (real orbital station-keeping included, not just linear drift)
/// falls within `hit_tolerance` of that prediction. Distance therefore
/// hurts accuracy two ways at once: a longer light-travel-time both gives
/// the target more real time to deviate from any linear prediction *and*
/// (for a fixed station-keeping period) covers more of its orbital cycle,
/// so the predicted and actual points can end up on opposite sides of the
/// same small loop.
///
/// **Simplified relative to a fully faithful light-lag model, flagged, not
/// silent:** the shooter is treated as having instantaneous knowledge of
/// the target's *current* position/velocity at the moment of firing (real
/// detection would add a second, matching light-lag for that observation
/// itself, doubling the effective prediction window) — this is the
/// *generous* direction for lasers, so if anything it understates how bad
/// long-range beam accuracy really is. `hit_tolerance` is an abstracted
/// fire-control precision stat (not literal hull cross-section, which at
/// real ROU dimensions — `Hulls_classes_the_qualitative_counter-graph.md`'s
/// own Gangster-class anchor, 200m — would make literal-geometry hits
/// practically impossible at any tactical range at all; **R-ARENA6, new,
/// open:** the right tolerance value, same as every other combat stat
/// here, is for the arena itself to help calibrate, not something derived
/// from first principles today).
pub fn laser_hit_check(
    shooter_pos: Vec3,
    target: &Combatant,
    fleets: &[FleetTrajectory],
    t_now: f64,
    hit_tolerance: f64,
) -> bool {
    let target_pos_now = target.position_at(fleets, t_now);
    let target_vel_now = target.velocity_at(fleets, t_now);
    let transit_time = shooter_pos.distance(target_pos_now); // c = 1
    let predicted = target_pos_now.add(target_vel_now.scale(transit_time));
    let actual = target.position_at(fleets, t_now + transit_time);
    predicted.distance(actual) <= hit_tolerance
}

/// **A missile** — confirmed this conversation: *"Missiles start at some
/// delta_v compared to their source and accelerate continuously with
/// relatively low mass using chase algorithm, but have limited fuel."*
/// Reuses [`solve_intercept`] every tick (re-solved fresh each time, same
/// "memoize each tick" discipline as a ship-scale chaser), but only while
/// `fuel_remaining > 0`; once fuel is spent it coasts on whatever velocity
/// it had, no further correction — a target that's still actively
/// station-keeping (let alone maneuvering) is very unlikely to wander back
/// into a ballistic missile's path, so fuel-out is treated as a practical
/// miss here rather than modeling an extended coast phase.
///
/// **Direction convention:** [`solve_intercept`]'s returned `direction` is
/// the acceleration direction that closes `relative_position` to zero when
/// `relative_position = target − pursuer` (confirmed by `combat_arena.rs`'s
/// existing, tested usage: `chaser_accel = sol.direction.scale(max_accel)`
/// with `rel_pos = target.pos − chaser.pos`) — applied *directly*, not
/// negated, here and in [`Missile::step`].
///
/// **Dodge — confirmed this conversation:** *"add pursuit with dodging as
/// the new behavior of missiles... could perhaps reuse the station
/// keeping behavior for the missile trajectory rather than a point."*
/// `pos`/`vel` track the missile's *core guided trajectory* (its own
/// guidance is precise about this — hitting its own target is unaffected
/// by the jink, the same way a real missile's IMU can correct for a
/// deliberate evasive weave while still tracking a target). `dodge` is a
/// small [`StationKeeping`] loop layered on top, representing the
/// missile's *true physical position* for the purpose of a defensive
/// laser trying to hit *it* — reusing the exact same "a distant shooter
/// can only linearly extrapolate observed velocity, not a hidden orbital
/// phase" mechanic that already makes ships hard to snipe at range
/// ([`laser_hit_check`]), applied one level down. See
/// [`Missile::true_position_at`] and [`laser_hit_check_missile`].
#[derive(Clone, Copy, Debug)]
pub struct Missile {
    pub pos: Vec3,
    pub vel: Vec3,
    pub max_accel: f64,
    pub fuel_remaining: f64,
    pub target: usize,
    pub shooter: usize,
    pub dodge: StationKeeping,
    pub launch_time: f64,
}

impl Missile {
    /// Launch, from `shooter_pos`/`shooter_vel` toward `target`, with an
    /// initial boost of `launch_delta_v` — confirmed: "start at some
    /// delta_v compared to their source" — aimed along that first
    /// instant's intercept solution. `dodge` is drawn fresh per missile
    /// (`Rng::fork`-seeded by the caller, same determinism discipline as
    /// every other per-entity draw in this engine) and its offset is
    /// normalized to zero at `t = launch_time`, so the missile's true
    /// position starts exactly at `shooter_pos` rather than jumping.
    #[allow(clippy::too_many_arguments)]
    pub fn launch(
        shooter_pos: Vec3,
        shooter_vel: Vec3,
        target_pos: Vec3,
        target_vel: Vec3,
        launch_delta_v: f64,
        max_accel: f64,
        fuel_duration: f64,
        target_idx: usize,
        shooter_idx: usize,
        dodge: StationKeeping,
        launch_time: f64,
    ) -> Self {
        let rel_pos = target_pos.sub(shooter_pos);
        let rel_vel = target_vel.sub(shooter_vel);
        let boost_dir = solve_intercept(rel_pos, rel_vel, max_accel, InterceptCriterion::PositionZero)
            .map(|s| s.direction)
            .unwrap_or_else(|| rel_pos.normalized());
        Missile {
            pos: shooter_pos,
            vel: shooter_vel.add(boost_dir.scale(launch_delta_v)),
            max_accel,
            fuel_remaining: fuel_duration,
            target: target_idx,
            shooter: shooter_idx,
            dodge,
            launch_time,
        }
    }

    /// One tick of flight: re-solve the intercept (if fuel remains),
    /// apply acceleration, integrate position — then report whether it
    /// has reached `target` within `hit_radius`. Uses the *core* position
    /// (unaffected by dodge) — the missile's own guidance sees through
    /// its own jink.
    pub fn step(&mut self, target_pos: Vec3, target_vel: Vec3, dt: f64, hit_radius: f64) -> bool {
        if self.fuel_remaining > 0.0 {
            let rel_pos = target_pos.sub(self.pos);
            let rel_vel = target_vel.sub(self.vel);
            if let Some(sol) = solve_intercept(rel_pos, rel_vel, self.max_accel, InterceptCriterion::PositionZero) {
                self.vel = self.vel.add(sol.direction.scale(self.max_accel * dt));
            }
            self.fuel_remaining -= dt;
        }
        self.pos = self.pos.add(self.vel.scale(dt));
        self.pos.distance(target_pos) <= hit_radius
    }

    /// The missile's *true* physical position at (near-future) time `t` —
    /// core guided position, locally linearly extrapolated from the
    /// current tick's `pos`/`vel` (valid over the short transit times a
    /// defensive laser's light-travel-time represents, since guidance
    /// re-solves every tick anyway), plus its dodge offset relative to
    /// where that offset sat at launch.
    pub fn true_position_at(&self, t: f64, t_now: f64) -> Vec3 {
        let core = self.pos.add(self.vel.scale(t - t_now));
        core.add(self.dodge.offset_at(t)).sub(self.dodge.offset_at(self.launch_time))
    }
}

/// **A defensive laser shot at an in-flight missile** — confirmed this
/// conversation: *"lasers can target and destroy missiles."* Same
/// light-lag prediction-vs-actual mechanic as [`laser_hit_check`], adapted
/// for a `Missile` instead of an `Combatant`: the shooter can observe the
/// missile's current true position and velocity (dodge included, since
/// that's where it actually *is* right now) but can only extrapolate
/// linearly — it doesn't know the dodge's hidden phase/period any more
/// than it would for a full ship, so it can't predict the jink's future
/// arc. This mirrors real point-defense laser doctrine: a beam's
/// speed-of-light travel time means it "eliminat[es] the need to
/// calculate an intercept course, as interceptor missiles must do," and
/// can "follow and maintain [its] beam on radically maneuvering missiles"
/// (U.S. Congressional Research Service, *Department of Defense Directed
/// Energy Weapons: Background and Issues for Congress*, R46925,
/// <https://www.congress.gov/crs-product/R46925>) — the beam only has to
/// win the *prediction* problem, not out-accelerate anything, which is
/// exactly the asymmetry this function checks.
pub fn laser_hit_check_missile(shooter_pos: Vec3, missile: &Missile, t_now: f64, hit_tolerance: f64) -> bool {
    let true_pos_now = missile.true_position_at(t_now, t_now);
    let true_vel_now = missile.vel.add(missile.dodge.offset_velocity_at(t_now));
    let transit_time = shooter_pos.distance(true_pos_now);
    let predicted = true_pos_now.add(true_vel_now.scale(transit_time));
    let actual = missile.true_position_at(t_now + transit_time, t_now);
    predicted.distance(actual) <= hit_tolerance
}

/// **Spatial partitioning for nearest-neighbor targeting** — confirmed
/// this conversation: *"spatial partitioning should be fixed... a BSP
/// tree is the canonical algorithm."* Every laser choosing a target, and
/// every missile-carrier choosing one, was a linear scan over its entire
/// candidate set each tick — `O(shooters × candidates)`, which is exactly
/// what made point-defense targeting the dominant cost at realistic
/// missile counts (measured this conversation: 1.5-2.8ms/tick at
/// 700-1000 concurrent missiles, almost entirely this scan). A
/// [`BspTree`] answers "nearest surviving point to here" in `O(log n)`
/// instead.
///
/// An axis-aligned BSP tree (a k-d tree — the standard specialization of
/// BSP trees for point-cloud data, not a meaningfully different
/// structure). The splitting axis cycles by tree depth rather than being
/// chosen by variance/spread, and the split value is the exact median
/// (found by sorting each sub-range in place — `O(n log n)` to build).
/// **Rebuilt fresh every tick** from whichever points are currently valid
/// (alive ships, in-flight missiles) — no incremental updates, no
/// persistence across ticks. Angular/directional pruning (culling by
/// heading, not just distance) is a real further optimization once
/// profiling asks for it, not attempted here.
///
/// **Slab-allocated — confirmed this conversation:** *"Treebuild
/// constraint requires a slab memory allocation so you don't have to
/// make the tree on the heap [node by node]."* Profiling the first
/// (`Box`-per-node, recursive) version found ~89% of the remaining
/// per-tick cost was the *build* step, not the `O(log n)` queries —
/// exactly the overhead a slab avoids. Both the tree's internal nodes
/// (`nodes`) and its points (`points`) live in one contiguous `Vec`
/// each, sized once via `with_capacity` up front; a leaf stores a
/// `[start, end)` range into the shared `points` slab instead of owning
/// its own `Vec`, and a split node stores its children as plain `usize`
/// indices into `nodes` instead of `Box` pointers. Building the tree is
/// now two allocations total (one per slab), not one per node — the
/// points slab is partitioned in place (in-place sorting per sub-range,
/// no extra allocation per split) rather than copied into new `Vec`s at
/// every level the way the `Box` version's `split_off` did.
pub(crate) enum BspNodeSlab {
    // crate-visible so the pub(crate) `nodes` field is nameable
    Leaf { start: usize, end: usize },
    Split { axis: usize, value: f64, left: usize, right: usize },
}

#[inline]
fn coord(v: Vec3, axis: usize) -> f64 {
    match axis {
        0 => v.x,
        1 => v.y,
        _ => v.z,
    }
}

pub struct BspTree {
    pub(crate) nodes: Vec<BspNodeSlab>, // crate-visible for the slab-bound test
    points: Vec<(Vec3, usize)>,
    root: Option<usize>,
}

impl BspTree {
    /// Leaves hold up to this many points directly rather than splitting
    /// further — small enough to keep tree depth (and so query cost)
    /// low, large enough that leaf scans stay cheap; not tuned beyond "a
    /// reasonable round number."
    const LEAF_CAPACITY: usize = 8;

    /// Build a tree over `points` — each a position paired with whatever
    /// index/id the caller wants back from a query (a ship index, a
    /// `(shooter, local index)` pair encoded as a single `usize`, etc.).
    pub fn build(points: &[(Vec3, usize)]) -> Self {
        let mut points = points.to_vec(); // the one allocation for the points slab
        let n = points.len();
        if n == 0 {
            return BspTree { nodes: Vec::new(), points, root: None };
        }
        // Worst case: every leaf holds a single point (n leaves), and a
        // binary tree with n leaves has n-1 internal nodes -- 2n-1 nodes
        // total. A simple, safe upper bound; the slab's whole point is to
        // size this once and never reallocate mid-build.
        let mut nodes = Vec::with_capacity(2 * n);
        let root = Self::build_node(&mut points, 0, n, 0, &mut nodes);
        BspTree { nodes, points, root: Some(root) }
    }

    fn build_node(
        points: &mut [(Vec3, usize)],
        start: usize,
        end: usize,
        depth: usize,
        nodes: &mut Vec<BspNodeSlab>,
    ) -> usize {
        let len = end - start;
        if len <= Self::LEAF_CAPACITY {
            nodes.push(BspNodeSlab::Leaf { start, end });
            return nodes.len() - 1;
        }
        let axis = depth % 3;
        points[start..end].sort_by(|a, b| coord(a.0, axis).partial_cmp(&coord(b.0, axis)).unwrap());
        let mid = start + len / 2;
        let value = coord(points[mid].0, axis);
        let left = Self::build_node(points, start, mid, depth + 1, nodes);
        let right = Self::build_node(points, mid, end, depth + 1, nodes);
        nodes.push(BspNodeSlab::Split { axis, value, left, right });
        nodes.len() - 1
    }

    /// The nearest point to `query`, and its paired index — `None` if the
    /// tree is empty.
    pub fn nearest(&self, query: Vec3) -> Option<(usize, f64)> {
        let mut best: Option<(usize, f64)> = None;
        if let Some(root) = self.root {
            self.nearest_in(root, query, &mut best);
        }
        best
    }

    fn nearest_in(&self, node_idx: usize, query: Vec3, best: &mut Option<(usize, f64)>) {
        match &self.nodes[node_idx] {
            BspNodeSlab::Leaf { start, end } => {
                for &(p, idx) in &self.points[*start..*end] {
                    let d = p.distance(query);
                    if best.map(|(_, bd)| d < bd).unwrap_or(true) {
                        *best = Some((idx, d));
                    }
                }
            }
            BspNodeSlab::Split { axis, value, left, right } => {
                let diff = coord(query, *axis) - value;
                let (near, far) = if diff <= 0.0 { (*left, *right) } else { (*right, *left) };
                self.nearest_in(near, query, best);
                // The far side can only hold something closer than the
                // current best if the query's distance to the splitting
                // plane itself is less than that best — the standard k-d
                // tree pruning rule.
                if best.map(|(_, bd)| diff.abs() < bd).unwrap_or(true) {
                    self.nearest_in(far, query, best);
                }
            }
        }
    }
}

// ===========================================================================
// Weapon parameters + engagement resolution — moved verbatim out of the old
// `laser_vs_missile` example so the fight lives in the engine. Every constant
// below is the exact value that example was tuned at; changing them is what
// re-balances laser-vs-missile, so they are gathered here as one config object
// rather than scattered `const`s a harness could quietly diverge on.
// ===========================================================================

/// The tuned weapon/engagement parameters (previously top-level `const`s in the
/// `laser_vs_missile` harness). Defaults reproduce that harness bit-for-bit.
/// R-ARENA7 (open): these remain placeholders the arena exists to calibrate.
#[derive(Clone, Copy, Debug)]
pub struct CombatConfig {
    /// Missiles are "relatively low mass" → much higher accel than their carrier.
    pub missile_accel_multiplier: f64,
    pub missile_fuel_years: f64,
    pub missile_launch_delta_v: f64,
    /// ly — a missile counts as a hit within this of its target.
    pub missile_hit_radius: f64,
    /// ly — abstracted fire-control precision for a beam's predicted-vs-actual
    /// aim (see [`laser_hit_check`]).
    pub laser_hit_tolerance: f64,
    pub dodge_radius: (f64, f64),
    pub dodge_period: (f64, f64),
    /// Guidance-channel cap on a shooter's simultaneously in-flight missiles.
    pub max_missiles_per_shooter: usize,
    /// A laser resolves several shots per tick rather than a hard one-kill
    /// ceiling (the "kills/tick" fix); each shot re-targets the once-per-tick
    /// BSP query.
    pub laser_shots_per_tick: usize,
    /// Missiles per burst; bursts are desynchronized and released one per tick.
    pub burst_count: usize,
}

impl Default for CombatConfig {
    fn default() -> Self {
        CombatConfig {
            missile_accel_multiplier: 8.0,
            missile_fuel_years: 0.08,
            missile_launch_delta_v: 0.02,
            missile_hit_radius: 0.00005,
            // modest 2x over the original 0.00003, from the dimensional-grounding
            // analysis; with `laser_shots_per_tick` gives real relative-velocity
            // dependence at 0.01 ly.
            laser_hit_tolerance: 0.00006,
            dodge_radius: (0.00001, 0.00004),
            dodge_period: (0.005, 0.02),
            max_missiles_per_shooter: 30,
            laser_shots_per_tick: 40,
            burst_count: 4,
        }
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Winner {
    Laser,
    Missile,
    Draw,
}

/// Result of one resolved engagement.
#[derive(Clone, Copy, Debug)]
pub struct EngagementOutcome {
    pub winner: Winner,
    pub laser_survivors: usize,
    pub missile_survivors: usize,
}

/// Who a laser targets this shot — a ship, or an in-flight missile
/// (point-defense, indexed by `(shooter, local index)`).
enum LaserTarget {
    Ship(usize),
    Missile(usize, usize),
}

/// Resolve one laser-side-vs-missile-side engagement between two already-placed
/// fleets of [`Combatant`]s. **This is the engine combat entry point** the arena
/// scenarios and (eventually) the production game's `sys_engagement` both call.
/// Spawning is the caller's job (the arena spawns outside production); this only
/// fights. Deterministic given `rng`; iterates in index order.
#[allow(clippy::too_many_arguments)]
pub fn resolve_engagement(
    sim_cfg: &SimConfig,
    cfg: &CombatConfig,
    rng: &mut Rng,
    fleets: &[FleetTrajectory; 2],
    laser_ships: &[Combatant],
    missile_ships: &[Combatant],
    horizon: f64,
    dt: f64,
    volley_period: f64,
) -> EngagementOutcome {
    let n_lasers = laser_ships.len();
    let n_missiles = missile_ships.len();
    let carrier_accel = laser_ships[0].max_accel(sim_cfg); // same hull both sides
    let missile_accel = carrier_accel * cfg.missile_accel_multiplier;

    let mut laser_alive = vec![true; n_lasers];
    let mut missile_alive = vec![true; n_missiles];
    let mut in_flight: Vec<Vec<Missile>> = vec![Vec::new(); n_missiles];
    let mut burst_remaining: Vec<usize> = vec![0; n_missiles];
    let mut next_launch: Vec<f64> =
        (0..n_missiles).map(|mi| rng.fork(mi as u64 + 500_000).range(0.0, volley_period.max(1e-9))).collect();
    let mut launch_counter: u64 = 0;

    let mut t = 0.0;
    while t < horizon {
        let laser_positions: Vec<(Vec3, usize)> =
            (0..n_lasers).filter(|&i| laser_alive[i]).map(|i| (laser_ships[i].position_at(fleets, t), i)).collect();
        let laser_tree = BspTree::build(&laser_positions);
        let missile_ship_positions: Vec<(Vec3, usize)> = (0..n_missiles)
            .filter(|&i| missile_alive[i])
            .map(|i| (missile_ships[i].position_at(fleets, t), i))
            .collect();
        let missile_ship_tree = BspTree::build(&missile_ship_positions);

        // 1. Step all in-flight missiles.
        for salvo in in_flight.iter_mut().take(n_missiles) {
            let laser_alive_ref = &mut laser_alive;
            let ships_ref = laser_ships;
            let fleets_ref = fleets;
            salvo.retain_mut(|missile| {
                if !laser_alive_ref[missile.target] {
                    return false;
                }
                let target_pos = ships_ref[missile.target].position_at(fleets_ref, t);
                let target_vel = ships_ref[missile.target].velocity_at(fleets_ref, t);
                let hit = missile.step(target_pos, target_vel, dt, cfg.missile_hit_radius);
                if hit {
                    laser_alive_ref[missile.target] = false;
                    false
                } else {
                    missile.fuel_remaining > 0.0
                }
            });
        }
        // 2. Burst launches.
        for mi in 0..n_missiles {
            if !missile_alive[mi] {
                continue;
            }
            if burst_remaining[mi] == 0 && t >= next_launch[mi] {
                burst_remaining[mi] = cfg.burst_count;
                next_launch[mi] += volley_period;
            }
            if burst_remaining[mi] > 0 && in_flight[mi].len() < cfg.max_missiles_per_shooter {
                let shooter_pos = missile_ships[mi].position_at(fleets, t);
                let shooter_vel = missile_ships[mi].velocity_at(fleets, t);
                if let Some((ti, _)) = laser_tree.nearest(shooter_pos) {
                    let target_pos = laser_ships[ti].position_at(fleets, t);
                    let target_vel = laser_ships[ti].velocity_at(fleets, t);
                    let dodge = StationKeeping::draw(&mut rng.fork(launch_counter), cfg.dodge_radius, cfg.dodge_period);
                    launch_counter += 1;
                    in_flight[mi].push(Missile::launch(
                        shooter_pos,
                        shooter_vel,
                        target_pos,
                        target_vel,
                        cfg.missile_launch_delta_v,
                        missile_accel,
                        cfg.missile_fuel_years,
                        ti,
                        mi,
                        dodge,
                        t,
                    ));
                }
                burst_remaining[mi] -= 1;
            }
        }
        // 3. Lasers fire at nearest ship or in-flight missile (point-defense).
        let mut missile_lookup: Vec<(usize, usize)> = Vec::new();
        let mut missile_positions: Vec<(Vec3, usize)> = Vec::new();
        for (mi, salvo) in in_flight.iter().enumerate().take(n_missiles) {
            for (li, m) in salvo.iter().enumerate() {
                let flat = missile_lookup.len();
                missile_lookup.push((mi, li));
                missile_positions.push((m.pos, flat));
            }
        }
        let missile_tree = BspTree::build(&missile_positions);
        for li in 0..n_lasers {
            if !laser_alive[li] {
                continue;
            }
            let shooter_pos = laser_ships[li].position_at(fleets, t);
            for _ in 0..cfg.laser_shots_per_tick {
                let nearest_ship = missile_ship_tree.nearest(shooter_pos).filter(|&(si, _)| missile_alive[si]);
                let nearest_missile = missile_tree.nearest(shooter_pos).filter(|&(flat, _)| {
                    in_flight[missile_lookup[flat].0][missile_lookup[flat].1].fuel_remaining != f64::NEG_INFINITY
                });

                let target = match (nearest_ship, nearest_missile) {
                    (Some((si, d_ship)), Some((flat, d_missile))) => {
                        if d_ship <= d_missile {
                            Some(LaserTarget::Ship(si))
                        } else {
                            Some(LaserTarget::Missile(missile_lookup[flat].0, missile_lookup[flat].1))
                        }
                    }
                    (Some((si, _)), None) => Some(LaserTarget::Ship(si)),
                    (None, Some((flat, _))) => {
                        Some(LaserTarget::Missile(missile_lookup[flat].0, missile_lookup[flat].1))
                    }
                    (None, None) => None,
                };

                match target {
                    Some(LaserTarget::Ship(si)) => {
                        if laser_hit_check(shooter_pos, &missile_ships[si], fleets, t, cfg.laser_hit_tolerance) {
                            missile_alive[si] = false;
                        }
                    }
                    Some(LaserTarget::Missile(mi, idx)) => {
                        if laser_hit_check_missile(shooter_pos, &in_flight[mi][idx], t, cfg.laser_hit_tolerance) {
                            in_flight[mi][idx].fuel_remaining = f64::NEG_INFINITY;
                        }
                    }
                    None => break,
                }
            }
        }
        for v in in_flight.iter_mut() {
            v.retain(|m| m.fuel_remaining != f64::NEG_INFINITY);
        }

        let laser_count = laser_alive.iter().filter(|&&a| a).count();
        let missile_count = missile_alive.iter().filter(|&&a| a).count();
        if laser_count == 0 && missile_count == 0 {
            return EngagementOutcome { winner: Winner::Draw, laser_survivors: 0, missile_survivors: 0 };
        } else if laser_count == 0 {
            return EngagementOutcome { winner: Winner::Missile, laser_survivors: 0, missile_survivors: missile_count };
        } else if missile_count == 0 {
            return EngagementOutcome { winner: Winner::Laser, laser_survivors: laser_count, missile_survivors: 0 };
        }
        t += dt;
    }
    let laser_survivors = laser_alive.iter().filter(|&&a| a).count();
    let missile_survivors = missile_alive.iter().filter(|&&a| a).count();
    let winner = match laser_survivors.cmp(&missile_survivors) {
        std::cmp::Ordering::Greater => Winner::Laser,
        std::cmp::Ordering::Less => Winner::Missile,
        std::cmp::Ordering::Equal => Winner::Draw,
    };
    EngagementOutcome { winner, laser_survivors, missile_survivors }
}
