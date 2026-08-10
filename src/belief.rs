//! **Believed kinematics — decisions run on what you have seen, not on what is
//! true** (R-O41, `Hyades_standing_layer_and_observation.md` §6.4).
//!
//! Acceleration is the long-range observable (design law #10), and it is a
//! **one-sided** one: a ship may fly below peak and it may never fly above it.
//! So an observation is a *lower bound* on capability, and the best estimate
//! available from a history of observations is their **maximum** — the fastest
//! burn a target has ever been caught making.
//!
//! Everything interesting follows from that asymmetry:
//!
//! - A target that has been under-burning has a true reachable set **larger**
//!   than the one an observer computes, so its actual destination can lie
//!   outside the cone. **Surprise attack falls out of the physics** instead of
//!   being bolted on.
//! - An intercept solved against a believed `a_max` can simply *fail* against a
//!   target that was masking. [`solve_intercept`](crate::combat::solve_intercept)
//!   already takes acceleration as a parameter; what this module supplies is the
//!   discipline about *which* acceleration a decision is allowed to use.
//! - A disengagement judged winnable on belief can be unwinnable in fact, which
//!   is the accept/decline case sim §4 cares about.
//!
//! **Belief only ever moves one way.** A capability once displayed cannot be
//! un-displayed, so [`BeliefAMax`] is monotone non-decreasing. Masking is
//! therefore a *spend-once* resource: the first time you burn hard in view of
//! someone, you have told them, permanently. That is the physical form of the
//! standing layer's asymmetric-leak rule (§5) — Design never goes stale.
//!
//! ## Scope, stated plainly
//!
//! This module is the estimator and the decision predicate. **It is not yet
//! wired into a sim-level engagement**, because the engine has no accept/decline
//! site to wire it to: `combat::resolve_engagement` is a pure tactical resolver
//! that takes two fully-specified fleets, and there is no round/command layer
//! for a "do I take this fight" decision to live in (`hyades_todo.md` T-30).
//! When that lands, the rule is that the decision reads a [`BeliefAMax`] and
//! **never** a [`Combatant::max_accel`](crate::combat::Combatant::max_accel) of
//! the other side.
//!
//! Missile terminal guidance is deliberately exempt and stays on true
//! kinematics: §6.2 says close range is exactly where the degeneracy breaks, so
//! a missile a few thousand km out is *not* working from a lower bound.

/// One piece of observational evidence about a target's acceleration.
///
/// `range_ly` is the distance the evidence has to cross, and with `c = 1` it is
/// also the delay — an observation made at `made_at` is not admissible until
/// `made_at + range_ly`. Carrying the range rather than a precomputed arrival
/// time keeps the light-lag visible at the call site, which is where it is
/// easy to forget.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Observation {
    /// Acceleration the target was seen sustaining, in g. A lower bound on its
    /// peak by construction — see the module docs.
    pub a_observed: f64,
    /// Sim time (years) at which the light left the target.
    pub made_at: f64,
    /// Distance the light must cross, in light-years. `c = 1`.
    pub range_ly: f64,
}

impl Observation {
    /// Sim time at which this evidence reaches the observer.
    pub fn arrives_at(&self) -> f64 {
        self.made_at + self.range_ly
    }
}

/// A one-sided estimate of some target's peak acceleration, built from
/// [`Observation`]s that have actually arrived.
///
/// Default is "never observed": [`a_max`](Self::a_max) returns `None`, which
/// callers must handle explicitly rather than defaulting to zero. Treating an
/// unobserved contact as immobile is the exact mistake this type exists to make
/// impossible — see [`Unobserved`].
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BeliefAMax {
    best: f64,
    as_of: f64,
    samples: u32,
}

impl BeliefAMax {
    /// Admit an observation if its light has arrived by `now`.
    ///
    /// Returns `true` if the belief *changed* — i.e. this observation showed the
    /// target doing something it had not been seen doing before. That is the
    /// event worth logging: it is the moment a concealment stopped working.
    ///
    /// A stale observation (one recording a slower burn than something already
    /// seen) is not an error and not discarded information — it simply cannot
    /// lower a bound, so it moves nothing but the sample count.
    pub fn observe(&mut self, obs: Observation, now: f64) -> bool {
        if now < obs.arrives_at() {
            return false; // light has not got here yet
        }
        if !obs.a_observed.is_finite() || obs.a_observed < 0.0 {
            return false; // design law #16: no NaN or infinity into state
        }
        self.samples += 1;
        self.as_of = self.as_of.max(obs.arrives_at());
        if self.samples == 1 || obs.a_observed > self.best {
            let changed = self.samples > 1;
            self.best = obs.a_observed;
            return changed || self.samples == 1;
        }
        false
    }

    /// The believed peak acceleration, or `None` if this target has never been
    /// observed at all.
    pub fn a_max(&self) -> Option<f64> {
        (self.samples > 0).then_some(self.best)
    }

    /// Sim time of the most recent evidence to arrive. Meaningless when
    /// [`a_max`](Self::a_max) is `None`.
    pub fn as_of(&self) -> f64 {
        self.as_of
    }

    /// How many observations have been folded in. The SPRT sample count of
    /// §6.5 — detection is a hypothesis test, and this is `n`.
    pub fn samples(&self) -> u32 {
        self.samples
    }

    /// Resolve to a number a decision can use, given a policy for the
    /// never-observed case.
    pub fn assume(&self, unobserved: Unobserved) -> f64 {
        match self.a_max() {
            Some(a) => a,
            None => match unobserved {
                Unobserved::Helpless => 0.0,
                Unobserved::PeerOf(a) => a,
                Unobserved::AtLeast(a) => a,
            },
        }
    }
}

/// What to assume about a contact that has never been observed.
///
/// There is no safe default, so there is no `Default` impl — the caller must
/// say. The variants are ordered by how much trouble they invite.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Unobserved {
    /// Assume it cannot move. **Almost always wrong**, and the way a fleet
    /// walks into something. Kept because it is the right assumption for a
    /// derelict or a known-unpowered hulk, and because naming it makes the
    /// choice visible instead of implicit in a zero.
    Helpless,
    /// Assume it matches the observer. The sane default for a first contact:
    /// symmetric ignorance, no free advantage either way.
    PeerOf(f64),
    /// Assume a floor supplied by doctrine — paranoia with a number on it.
    AtLeast(f64),
}

/// The outcome of a kinematic accept/decline (§6.4, sim §4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Engagement {
    /// The fleet can outrun the other side and may break off at will.
    MayDisengage,
    /// The other side is faster; the fight happens whether or not it is wanted.
    Committed,
}

/// **Can this fleet break off?** Only if it out-accelerates the other side.
///
/// The criterion is kinematic and deterministic: in a stern chase the faster
/// ship dictates whether contact is made, so disengagement is available exactly
/// when `own > other`. Ties are `Committed` — a pursuer that merely matches you
/// never closes but never lets go either, and the conservative reading is the
/// one that does not promise an escape the physics does not.
///
/// This is deliberately a free function over two scalars. The whole content of
/// R-O41 is *which two scalars get passed*, and that is
/// [`decide_engagement`]'s job — keeping the rule itself unaware of where its
/// inputs came from means the belief discipline cannot be smuggled around by
/// reimplementing the comparison.
pub fn can_disengage(own_a_max: f64, other_a_max: f64) -> bool {
    own_a_max > other_a_max
}

/// Accept/decline **on believed kinematics** — the R-O41 entry point.
///
/// Takes the observer's own true `a_max` (you know your own ships) against a
/// [`BeliefAMax`] for the other side (you do not know theirs). A decision made
/// through here can be *wrong*, and wrong in one specific direction: belief is a
/// lower bound, so this can return [`Engagement::MayDisengage`] for a fleet that
/// is in fact committed. It can never make the opposite mistake.
///
/// That asymmetry is the mechanic, not a defect. See
/// `belief_lets_a_masking_target_spring_a_trap`.
pub fn decide_engagement(own_a_max: f64, belief: &BeliefAMax, unobserved: Unobserved) -> Engagement {
    if can_disengage(own_a_max, belief.assume(unobserved)) {
        Engagement::MayDisengage
    } else {
        Engagement::Committed
    }
}

/// What actually happens, resolved on true kinematics.
///
/// Only the resolver may call this. It exists so the gap between
/// [`decide_engagement`] and reality is a thing the engine can *measure* — a
/// surprise is a decision of `MayDisengage` that resolves `Committed`, and that
/// is exactly the event worth surfacing to a player as "they were holding back."
pub fn resolve_engagement_choice(own_a_max: f64, true_other_a_max: f64) -> Engagement {
    if can_disengage(own_a_max, true_other_a_max) {
        Engagement::MayDisengage
    } else {
        Engagement::Committed
    }
}

/// Did the other side's concealment pay off in this decision?
///
/// `true` exactly when the observer believed it could break off and could not.
/// The converse case — believing you are committed when you could have run — is
/// unreachable while belief is a lower bound, which is worth asserting rather
/// than assuming; `belief_can_only_err_in_one_direction` does.
pub fn was_surprised(decided: Engagement, actual: Engagement) -> bool {
    decided == Engagement::MayDisengage && actual == Engagement::Committed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_unobserved_target_is_none_not_zero() {
        let b = BeliefAMax::default();
        assert_eq!(b.a_max(), None);
        assert_eq!(b.samples(), 0);
        // The policy is forced to be explicit, and the three answers differ.
        assert_eq!(b.assume(Unobserved::Helpless), 0.0);
        assert_eq!(b.assume(Unobserved::PeerOf(0.7)), 0.7);
        assert_eq!(b.assume(Unobserved::AtLeast(1.5)), 1.5);
    }

    #[test]
    fn light_lag_gates_admission() {
        let mut b = BeliefAMax::default();
        let obs = Observation { a_observed: 0.9, made_at: 100.0, range_ly: 12.0 };
        assert_eq!(obs.arrives_at(), 112.0);

        // Before the light arrives, the observation does not exist.
        assert!(!b.observe(obs, 111.999));
        assert_eq!(b.a_max(), None);

        assert!(b.observe(obs, 112.0));
        assert_eq!(b.a_max(), Some(0.9));
        assert_eq!(b.as_of(), 112.0);
    }

    #[test]
    fn belief_is_monotone_and_keeps_the_maximum() {
        let mut b = BeliefAMax::default();
        let at = |a: f64, t: f64| Observation { a_observed: a, made_at: t, range_ly: 0.0 };

        assert!(b.observe(at(0.4, 0.0), 0.0));
        assert_eq!(b.a_max(), Some(0.4));

        // A faster burn raises the bound, and reports that it did — this is the
        // moment a concealment stopped working.
        assert!(b.observe(at(0.8, 1.0), 1.0));
        assert_eq!(b.a_max(), Some(0.8));

        // A subsequent slower burn cannot lower it. A capability once shown
        // cannot be un-shown, so masking is spend-once.
        assert!(!b.observe(at(0.1, 2.0), 2.0));
        assert_eq!(b.a_max(), Some(0.8));
        assert_eq!(b.samples(), 3);
    }

    #[test]
    fn nan_and_infinity_are_refused_at_the_door() {
        // Design law #16 — never let one into replicated state.
        let mut b = BeliefAMax::default();
        assert!(!b.observe(Observation { a_observed: f64::NAN, made_at: 0.0, range_ly: 0.0 }, 0.0));
        assert!(!b.observe(Observation { a_observed: f64::INFINITY, made_at: 0.0, range_ly: 0.0 }, 0.0));
        assert!(!b.observe(Observation { a_observed: -1.0, made_at: 0.0, range_ly: 0.0 }, 0.0));
        assert_eq!(b.a_max(), None);
    }

    #[test]
    fn belief_lets_a_masking_target_spring_a_trap() {
        // The marquee case (§6.4). A raider can pull 1.0 g but has only ever
        // been *seen* cruising at 0.4 g. A 0.6 g freighter reads the record,
        // concludes it can outrun the raider, and accepts the risk.
        let true_raider_a_max = 1.0;
        let mut belief = BeliefAMax::default();
        belief.observe(Observation { a_observed: 0.4, made_at: 0.0, range_ly: 5.0 }, 5.0);

        let freighter = 0.6;
        let decided = decide_engagement(freighter, &belief, Unobserved::PeerOf(freighter));
        assert_eq!(decided, Engagement::MayDisengage, "0.6 > believed 0.4, so it thinks it can run");

        let actual = resolve_engagement_choice(freighter, true_raider_a_max);
        assert_eq!(actual, Engagement::Committed, "0.6 < true 1.0, so it cannot");

        assert!(was_surprised(decided, actual));

        // And the trap is spend-once: the raider has now been seen at 1.0, so
        // the same decision next time is correct.
        belief.observe(Observation { a_observed: 1.0, made_at: 10.0, range_ly: 5.0 }, 15.0);
        let decided_after = decide_engagement(freighter, &belief, Unobserved::PeerOf(freighter));
        assert_eq!(decided_after, Engagement::Committed);
        assert!(!was_surprised(decided_after, actual));
    }

    #[test]
    fn belief_can_only_err_in_one_direction() {
        // Because belief ≤ truth, the decision can be optimistic but never
        // pessimistic: you may think you can run when you cannot, never the
        // reverse. Swept over a grid rather than argued.
        let mut cases = 0;
        for own_i in 0..=20 {
            let own = own_i as f64 * 0.1;
            for truth_i in 0..=20 {
                let truth = truth_i as f64 * 0.1;
                for masked_i in 0..=truth_i {
                    let observed = masked_i as f64 * 0.1; // observed ≤ truth, always
                    let mut b = BeliefAMax::default();
                    b.observe(Observation { a_observed: observed, made_at: 0.0, range_ly: 0.0 }, 0.0);
                    assert!(b.a_max().unwrap() <= truth + 1e-12, "belief must be a lower bound");

                    let decided = decide_engagement(own, &b, Unobserved::Helpless);
                    let actual = resolve_engagement_choice(own, truth);
                    if decided != actual {
                        assert!(was_surprised(decided, actual), "the only legal error is optimism");
                    }
                    cases += 1;
                }
            }
        }
        assert!(cases > 4000, "sweep should be dense ({cases})");
    }

    #[test]
    fn an_intercept_solved_on_belief_misses_a_masking_target() {
        // §6.4's other half: the *intercept* solved against a believed a_max is
        // wrong in the same direction. Here belief governs how long the chaser
        // thinks the target needs to escape; under-burning buys the target real
        // separation the chaser did not plan for.
        use crate::combat::{solve_intercept, InterceptCriterion};
        use crate::math::Vec3;

        let rel_pos = Vec3::new(10.0, 0.0, 0.0);
        let rel_vel = Vec3::new(-1.0, 0.0, 0.0);

        let believed = 0.4;
        let truth = 1.0;
        let on_belief = solve_intercept(rel_pos, rel_vel, believed, InterceptCriterion::PositionZero).unwrap();
        let on_truth = solve_intercept(rel_pos, rel_vel, truth, InterceptCriterion::PositionZero).unwrap();

        // The chaser plans for the slower closure it can see justified, so its
        // solution takes strictly longer than the one it would compute if it
        // knew what the target could really do.
        assert!(on_belief.time > on_truth.time, "believed {} vs true {}", on_belief.time, on_truth.time);
    }
}
