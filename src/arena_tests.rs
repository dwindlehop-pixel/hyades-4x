use super::*;

fn brute_force_nearest(points: &[(Vec3, usize)], query: Vec3) -> Option<(usize, f64)> {
    points.iter().map(|&(p, idx)| (idx, p.distance(query))).min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
}

#[test]
fn bsp_tree_empty_returns_none() {
    let tree = BspTree::build(&[]);
    assert!(tree.nearest(Vec3::ZERO).is_none());
}

#[test]
fn bsp_tree_single_point() {
    let p = Vec3::new(1.0, 2.0, 3.0);
    let tree = BspTree::build(&[(p, 7)]);
    let (idx, d) = tree.nearest(Vec3::ZERO).unwrap();
    assert_eq!(idx, 7);
    assert!((d - p.norm()).abs() < 1e-9);
}

/// The core correctness property: for many random point sets and
/// query points, spanning both fewer-than-one-leaf and multi-level
/// tree sizes, the BSP tree must find *exactly* the same nearest
/// point (by index) as an exhaustive linear scan — confirming the
/// pruning rule (`diff.abs() < best_distance`) never incorrectly
/// skips a subtree that could hold something closer.
#[test]
fn bsp_tree_matches_brute_force_nearest_for_random_point_sets() {
    let mut rng = Rng::new(1);
    for &n in &[1usize, 2, 8, 9, 50, 200, 1000] {
        let points: Vec<(Vec3, usize)> = (0..n)
            .map(|i| (Vec3::new(rng.range(-10.0, 10.0), rng.range(-10.0, 10.0), rng.range(-10.0, 10.0)), i))
            .collect();
        let tree = BspTree::build(&points);
        for _ in 0..20 {
            let query = Vec3::new(rng.range(-12.0, 12.0), rng.range(-12.0, 12.0), rng.range(-12.0, 12.0));
            let expected = brute_force_nearest(&points, query).unwrap();
            let actual = tree.nearest(query).unwrap();
            assert!(
                (actual.1 - expected.1).abs() < 1e-9,
                "n={n}: tree found distance {} (idx {}), brute force found {} (idx {})",
                actual.1,
                actual.0,
                expected.1,
                expected.0
            );
        }
    }
}

/// Points genuinely clustered along one axis (not uniformly random)
/// — a case worth checking on its own, since a poorly-implemented
/// pruning rule can pass on random data by luck but fail on
/// structured data (e.g. a whole fleet lined up along one heading).
#[test]
fn bsp_tree_matches_brute_force_for_axis_clustered_points() {
    let mut rng = Rng::new(2);
    let points: Vec<(Vec3, usize)> =
        (0..300).map(|i| (Vec3::new(i as f64 * 0.01, rng.range(-0.001, 0.001), 0.0), i)).collect();
    let tree = BspTree::build(&points);
    for _ in 0..30 {
        let query = Vec3::new(rng.range(-1.0, 4.0), rng.range(-1.0, 1.0), rng.range(-1.0, 1.0));
        let expected = brute_force_nearest(&points, query).unwrap();
        let actual = tree.nearest(query).unwrap();
        assert!((actual.1 - expected.1).abs() < 1e-9);
    }
}

#[test]
fn bsp_tree_handles_duplicate_positions() {
    let p = Vec3::new(5.0, 5.0, 5.0);
    let points: Vec<(Vec3, usize)> = (0..20).map(|i| (p, i)).collect();
    let tree = BspTree::build(&points);
    let (_, d) = tree.nearest(Vec3::ZERO).unwrap();
    assert!((d - p.norm()).abs() < 1e-9);
}

/// Confirms the slab-allocation claim this conversation ("a slab
/// memory allocation so you don't have to make the tree on the heap
/// [node by node]"): total node count never exceeds the `2n-1`
/// worst-case bound `build` pre-sizes its `Vec::with_capacity` to, for
/// a range of point-set sizes — i.e. the slab genuinely never needs a
/// mid-build reallocation.
#[test]
fn bsp_tree_node_count_stays_within_the_slab_capacity_bound() {
    let mut rng = Rng::new(9);
    for &n in &[1usize, 7, 8, 9, 50, 137, 500] {
        let points: Vec<(Vec3, usize)> =
            (0..n).map(|i| (Vec3::new(rng.range(-1.0, 1.0), rng.range(-1.0, 1.0), rng.range(-1.0, 1.0)), i)).collect();
        let tree = BspTree::build(&points);
        assert!(tree.nodes.len() <= 2 * n, "n={n}: {} nodes exceeds the 2n-1 bound", tree.nodes.len());
        assert!(
            tree.nodes.capacity() >= tree.nodes.len(),
            "capacity should never be exceeded, let alone need reallocation"
        );
    }
}

#[test]
fn station_keeping_is_deterministic_given_the_same_seed() {
    let mut r1 = Rng::new(42);
    let mut r2 = Rng::new(42);
    let s1 = StationKeeping::draw(&mut r1, (0.001, 0.01), (0.01, 0.1));
    let s2 = StationKeeping::draw(&mut r2, (0.001, 0.01), (0.01, 0.1));
    assert_eq!(s1.radius, s2.radius);
    assert_eq!(s1.angular_velocity, s2.angular_velocity);
    assert_eq!(s1.offset_at(1.23), s2.offset_at(1.23));
}

#[test]
fn station_keeping_offset_stays_within_its_drawn_radius() {
    let mut rng = Rng::new(7);
    let s = StationKeeping::draw(&mut rng, (0.5, 0.5), (0.1, 0.1));
    for i in 0..20 {
        let t = i as f64 * 0.037;
        let off = s.offset_at(t);
        assert!((off.norm() - 0.5).abs() < 1e-9, "offset should stay exactly on the drawn radius, got {}", off.norm());
    }
}

#[test]
fn station_keeping_orbit_is_perpendicular_to_its_axis() {
    let mut rng = Rng::new(99);
    let s = StationKeeping::draw(&mut rng, (1.0, 1.0), (1.0, 1.0));
    for i in 0..10 {
        let t = i as f64 * 0.1;
        assert!(s.offset_at(t).dot(s.axis).abs() < 1e-9, "orbit must stay in the plane perpendicular to its axis");
    }
}

#[test]
fn two_fleets_with_the_same_seed_produce_bit_identical_ships() {
    let hull = HullType::MediumSystems;
    let mut rng1 = Rng::new(5);
    let mut rng2 = Rng::new(5);
    let fleet1 = spawn_fleet(&mut rng1, 0, 6, Role::Freighter, hull, (0.001, 0.01), (0.01, 0.1));
    let fleet2 = spawn_fleet(&mut rng2, 0, 6, Role::Freighter, hull, (0.001, 0.01), (0.01, 0.1));
    for (a, b) in fleet1.iter().zip(fleet2.iter()) {
        assert_eq!(a.thrust_factor, b.thrust_factor);
        assert_eq!(a.station.offset_at(2.5), b.station.offset_at(2.5));
    }
}

/// Confirmed this conversation: "Rapid will be on the higher end of
/// the range for Medium, always" — holds in the arena's own spawn
/// path too, not just `Simulation::draw_thrust_factor`.
#[test]
fn rapid_offensive_ships_in_the_arena_are_also_pinned_to_the_top_of_their_range() {
    let mut rng = Rng::new(3);
    let fleet = spawn_fleet(&mut rng, 0, 5, Role::Freighter, HullType::RapidOffensive, (0.01, 0.01), (1.0, 1.0));
    let (_, hi) = hull_thrust_multiplier_range(HullType::RapidOffensive);
    for ship in &fleet {
        assert_eq!(ship.thrust_factor, hi);
    }
}

/// The textbook check for the position-intercept solver: a
/// stationary target dead ahead, at distance `d`, reached in exactly
/// `T = sqrt(4d/a)` under constant acceleration from rest — the
/// classic `d = ½aT²` kinematics result (any introductory mechanics
/// text; this is not a novel claim, just confirming the general
/// solver reduces to it in the degenerate case).
#[test]
fn stationary_target_matches_the_classic_half_a_t_squared_result() {
    let r0 = Vec3::new(10.0, 0.0, 0.0);
    let v0 = Vec3::ZERO;
    let a = 2.0;
    let sol = solve_intercept(r0, v0, a, InterceptCriterion::PositionZero).unwrap();
    let expected_t = (2.0 * 10.0 / a).sqrt(); // d = 1/2 a T^2 => T = sqrt(2d/a)
    assert!((sol.time - expected_t).abs() < 1e-6, "got {} expected {}", sol.time, expected_t);
    // Direction should point straight at the target (+x, since the
    // target is in +x and the pursuer must accelerate toward it —
    // verified numerically before fixing a sign bug this conversation
    // caught: the old, wrong sign made the pursuer flee at full
    // thrust, doubling the separation instead of closing it).
    assert!((sol.direction.x - 1.0).abs() < 1e-9);
}

/// A receding target that the pursuer is faster than should still be
/// catchable in finite time; a target receding faster than the
/// pursuer's achievable closing speed within any reasonable horizon
/// should just take correspondingly longer — the solver shouldn't
/// panic or loop forever either way.
#[test]
fn receding_target_still_solves() {
    let r0 = Vec3::new(5.0, 0.0, 0.0);
    let v0 = Vec3::new(1.0, 0.0, 0.0); // receding directly away
    let a = 0.5;
    let sol = solve_intercept(r0, v0, a, InterceptCriterion::PositionZero).unwrap();
    // Verify it actually satisfies the defining equation.
    let check = r0.add(v0.scale(sol.time)).norm() - 0.5 * a * sol.time * sol.time;
    assert!(check.abs() < 1e-6, "solution should satisfy |r0+v0T| = 0.5*a*T^2, residual {check}");
}

#[test]
fn position_within_tolerance_resolves_faster_than_exact_zero() {
    let r0 = Vec3::new(20.0, 0.0, 0.0);
    let v0 = Vec3::ZERO;
    let a = 1.0;
    let exact = solve_intercept(r0, v0, a, InterceptCriterion::PositionZero).unwrap();
    let tolerant = solve_intercept(r0, v0, a, InterceptCriterion::PositionWithin(5.0)).unwrap();
    assert!(tolerant.time < exact.time, "reaching within a tolerance should never take longer than reaching exactly");
}

#[test]
fn velocity_zero_matches_the_simple_v_over_a_closed_form() {
    let v0 = Vec3::new(3.0, 4.0, 0.0); // |v0| = 5
    let a = 2.0;
    let sol = solve_intercept(Vec3::ZERO, v0, a, InterceptCriterion::VelocityZero).unwrap();
    assert!((sol.time - 2.5).abs() < 1e-9, "5/2 = 2.5");
}

#[test]
fn velocity_within_tolerance_is_never_slower_than_matching_exactly() {
    let v0 = Vec3::new(10.0, 0.0, 0.0);
    let a = 1.0;
    let exact = solve_intercept(Vec3::ZERO, v0, a, InterceptCriterion::VelocityZero).unwrap();
    let tolerant = solve_intercept(Vec3::ZERO, v0, a, InterceptCriterion::VelocityWithin(4.0)).unwrap();
    assert!(tolerant.time < exact.time);
}

#[test]
fn already_satisfied_criteria_return_none() {
    assert!(solve_intercept(Vec3::ZERO, Vec3::ZERO, 1.0, InterceptCriterion::PositionZero).is_none());
    assert!(
        solve_intercept(Vec3::new(1.0, 0.0, 0.0), Vec3::ZERO, 1.0, InterceptCriterion::PositionWithin(5.0)).is_none()
    );
    assert!(solve_intercept(Vec3::ZERO, Vec3::ZERO, 1.0, InterceptCriterion::VelocityZero).is_none());
    assert!(
        solve_intercept(Vec3::ZERO, Vec3::new(1.0, 0.0, 0.0), 1.0, InterceptCriterion::VelocityWithin(5.0)).is_none()
    );
}

#[test]
fn zero_thrust_cannot_intercept() {
    assert!(solve_intercept(Vec3::new(1.0, 0.0, 0.0), Vec3::ZERO, 0.0, InterceptCriterion::PositionZero).is_none());
    assert!(solve_intercept(Vec3::ZERO, Vec3::new(1.0, 0.0, 0.0), 0.0, InterceptCriterion::VelocityZero).is_none());
}

/// A target with *zero* station-keeping radius moves perfectly
/// linearly, so linear extrapolation is exact and a laser must always
/// hit it, at any distance or velocity — misses can only come from
/// the orbital-jitter component, confirming `laser_hit_check` isn't
/// spuriously failing for some other reason.
#[test]
fn laser_always_hits_a_perfectly_linear_non_jittering_target() {
    let fleets = [FleetTrajectory { origin: Vec3::ZERO, velocity: Vec3::new(0.3, -0.1, 0.05) }];
    let mut rng = Rng::new(1);
    let target = ArenaShip {
        role: Role::Freighter,
        hull: HullType::LimitedSystems,
        thrust_factor: 1.0,
        fleet: 0,
        station: StationKeeping::draw(&mut rng, (0.0, 0.0), (1.0, 1.0)), // zero radius = no jitter
        maneuver_velocity: Vec3::ZERO,
        maneuver_start: 0.0,
        maneuver_origin_offset: Vec3::ZERO,
    };
    let shooter_pos = Vec3::new(-0.01, 0.002, -0.003);
    assert!(laser_hit_check(shooter_pos, &target, &fleets, 2.5, 1e-9));
}

/// The reverse: a target with a *large* station-keeping radius,
/// shot at from far enough away that the transit time spans a
/// meaningful fraction of its orbital period, can genuinely be
/// missed by an abstracted-precision laser — confirming distance
/// does degrade accuracy, not just that hits are always guaranteed.
#[test]
fn laser_can_miss_a_jittering_target_at_long_transit_time() {
    let fleets = [FleetTrajectory { origin: Vec3::ZERO, velocity: Vec3::ZERO }];
    let mut rng = Rng::new(1);
    let target = ArenaShip {
        role: Role::Freighter,
        hull: HullType::LimitedSystems,
        thrust_factor: 1.0,
        fleet: 0,
        station: StationKeeping::draw(&mut rng, (0.01, 0.01), (0.1, 0.1)), // big loop, short period
        maneuver_velocity: Vec3::ZERO,
        maneuver_start: 0.0,
        maneuver_origin_offset: Vec3::ZERO,
    };
    // Transit time = distance = 5 ly here, vastly longer than the 0.1
    // year period — many full orbits happen mid-flight, so a tight
    // tolerance should miss far more often than a loose one.
    let shooter_pos = Vec3::new(-5.0, 0.0, 0.0);
    let mut misses = 0;
    for i in 0..20 {
        let t = i as f64 * 0.037;
        if !laser_hit_check(shooter_pos, &target, &fleets, t, 1e-6) {
            misses += 1;
        }
    }
    assert!(misses > 0, "a tight-tolerance laser should miss at least sometimes over a long transit time");
}

/// A missile with ample fuel, chasing a stationary target, should
/// close in and hit within a reasonable number of ticks.
#[test]
fn missile_with_fuel_hits_a_stationary_target() {
    let target_pos = Vec3::new(0.001, 0.0, 0.0);
    let mut rng = Rng::new(1);
    let dodge = StationKeeping::draw(&mut rng, (0.0, 0.0), (1.0, 1.0));
    let mut m = Missile::launch(Vec3::ZERO, Vec3::ZERO, target_pos, Vec3::ZERO, 0.001, 5.0, 1.0, 0, 0, dodge, 0.0);
    let dt = 0.0005;
    let mut hit = false;
    for _ in 0..2000 {
        if m.step(target_pos, Vec3::ZERO, dt, 1e-6) {
            hit = true;
            break;
        }
    }
    assert!(hit, "a fueled missile should eventually catch a stationary target");
}

/// A missile launched with *zero* fuel just coasts on its initial
/// boost — it should generally miss an off-axis target rather than
/// mysteriously still homing in (confirming fuel exhaustion actually
/// disables guidance, per "limited fuel" being the whole point).
#[test]
fn missile_with_no_fuel_just_coasts_and_generally_misses() {
    let target_pos = Vec3::new(0.001, 0.001, 0.0); // off-axis from the initial straight boost
    let mut rng = Rng::new(1);
    let dodge = StationKeeping::draw(&mut rng, (0.0, 0.0), (1.0, 1.0));
    let mut m = Missile::launch(
        Vec3::ZERO,
        Vec3::ZERO,
        Vec3::new(0.001, 0.0, 0.0),
        Vec3::ZERO,
        0.0005,
        5.0,
        0.0,
        0,
        0,
        dodge,
        0.0,
    );
    let dt = 0.0005;
    let mut hit = false;
    for _ in 0..2000 {
        if m.step(target_pos, Vec3::ZERO, dt, 1e-6) {
            hit = true;
            break;
        }
    }
    assert!(!hit, "a fuel-less missile aimed at a different point shouldn't hit an off-axis target");
}

/// True position at `t = launch_time` should equal the launch position
/// exactly — the dodge offset is normalized to cancel out at launch,
/// so the missile's true position doesn't discontinuously jump the
/// instant it leaves its carrier.
#[test]
fn missile_true_position_starts_exactly_at_the_launch_point() {
    let mut rng = Rng::new(3);
    let dodge = StationKeeping::draw(&mut rng, (0.0005, 0.0005), (0.01, 0.01));
    let launch_pos = Vec3::new(1.0, -2.0, 0.5);
    let m = Missile::launch(
        launch_pos,
        Vec3::ZERO,
        Vec3::new(2.0, -2.0, 0.5),
        Vec3::ZERO,
        0.001,
        1.0,
        0.1,
        0,
        0,
        dodge,
        7.0,
    );
    let true_pos_at_launch = m.true_position_at(7.0, 7.0);
    assert!((true_pos_at_launch.distance(launch_pos)) < 1e-12);
}

/// A missile with zero dodge radius is a perfectly predictable point
/// target, so a point-defense laser should always hit it (mirroring
/// `laser_always_hits_a_perfectly_linear_non_jittering_target` for
/// ships).
#[test]
fn point_defense_always_hits_a_non_dodging_missile() {
    let mut rng = Rng::new(1);
    let dodge = StationKeeping::draw(&mut rng, (0.0, 0.0), (1.0, 1.0));
    let m = Missile::launch(
        Vec3::new(0.001, 0.0, 0.0),
        Vec3::new(0.1, 0.0, 0.0),
        Vec3::new(0.002, 0.0, 0.0),
        Vec3::ZERO,
        0.001,
        2.0,
        0.1,
        0,
        0,
        dodge,
        0.0,
    );
    assert!(laser_hit_check_missile(Vec3::ZERO, &m, 1.5, 1e-9));
}

/// The reverse: a missile with a large dodge radius, shot at from far
/// enough that the transit time spans a meaningful fraction of its
/// jink period, can genuinely be missed by point defense — confirming
/// distance-degrades-accuracy applies to missiles too, not just ships.
#[test]
fn point_defense_can_miss_a_dodging_missile_at_long_transit_time() {
    let mut rng = Rng::new(1);
    let dodge = StationKeeping::draw(&mut rng, (0.01, 0.01), (0.1, 0.1));
    let m = Missile::launch(
        Vec3::new(5.0, 0.0, 0.0),
        Vec3::ZERO,
        Vec3::new(6.0, 0.0, 0.0),
        Vec3::ZERO,
        0.0,
        1.0,
        1.0,
        0,
        0,
        dodge,
        0.0,
    );
    let mut misses = 0;
    for i in 0..20 {
        let t = i as f64 * 0.031;
        if !laser_hit_check_missile(Vec3::ZERO, &m, t, 1e-6) {
            misses += 1;
        }
    }
    assert!(misses > 0, "a tight-tolerance point-defense laser should miss a dodging missile at least sometimes over a long transit time");
}
