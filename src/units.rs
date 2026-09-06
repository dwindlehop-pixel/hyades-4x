//! **Band and kiloton, as types rather than as conventions.**
//!
//! The engine had a unit error that typechecked. `K = min(hab, bio, infra)`
//! took a minimum across `hab` (a suitability score on the Band ladder),
//! `infra` (a development level on the Band ladder) and `bio` — which design
//! law #11 defines as *"a mass in kilotons"* that population consumes 1:1.
//! Two of those three are dimensionless levels and the third is a mass, so the
//! `min` was comparing a mass against a level and the whole
//! `biosphere_regen_rate` result rested on it.
//!
//! Nothing about `f64` could have caught that, which is the argument for these
//! types: **a quantity carries its unit, and the conversion between units is
//! written once, here, where it can be argued with.**
//!
//! ## The two units
//!
//! - [`Band`] — position on the Band ladder (`Hyades_mineral_cost_curve.md`
//!   §2.6). Habitability, infrastructure and population *level* live here.
//!   Roughly `0..=4`; it is an ordinal magnitude tier, not an amount of stuff.
//! - [`Kilotons`] — an amount of stuff. Biomass, minerals, hull dry mass and
//!   cargo live here, and it is the unit conservation is stated in (L6).
//!
//! ## The bridge is multiplicative, because the ladder is
//!
//! §2.6 defines a Band step as a *multiplicative* jump — "crossing from one
//! Band to the next is not 'one more unit'; it is a jump of several times the
//! previous Band's magnitude" — with the `I→II` and `II→III` factors each a
//! rational in `[4, 8]`. So the conversion is exponential:
//!
//! ```text
//! kilotons(b) = KILOTONS_AT_BAND_I · BAND_STEP^(b − 1)
//! band(m)     = 1 + log(m / KILOTONS_AT_BAND_I) / log(BAND_STEP)
//! ```
//!
//! The map is exact and unclamped in the Band→mass direction, because the
//! growth step *conserves mass across it*: the biomass a population step draws
//! is `KT(pop_after) − KT(pop_before)`, and any clamp in that map is mass
//! created or destroyed at the clamp. The reverse direction has no lower bound
//! mathematically (`band(0) = −∞`), and an infinity in replicated state is a
//! fatal error rather than a value (design law #16), so it floors at
//! [`BAND_FLOOR`].
//!
//! ## Why the two constraints on population coincide
//!
//! A world's pristine biosphere `bio_max` is generated as a Band value and
//! stored as its mass `KT(bio_max)`. A population at Band `p` masses `KT(p)`.
//! So "there is enough biomass to make these people" and "`p ≤ bio_max`" are
//! the *same* inequality, for any [`BAND_STEP`] — which is why the old
//! `min(hab, bio, infra)` produced sensible play despite comparing a mass
//! against two levels. The fix keeps that agreement and makes it derivable
//! rather than coincidental: the ceiling is a Band minimum, the draw is a
//! mass, and they bind at the same place.
//!
//! A *linear* bridge would have been the easy choice and would have quietly
//! contradicted the ladder spec — Band II would have been twice Band I instead
//! of four to eight times it. Getting this wrong is the same class of error as
//! the one the types exist to prevent, one level up.
//!
//! **[`BAND_STEP`] is not ratified.** §2.6's R-MC15 is open precisely because
//! no cost ladder this project has shipped satisfies the `[4, 8]` constraint on
//! both steps. `4.0` is the floor of the permitted range, chosen so the value
//! is inside the spec rather than outside it, and flagged rather than
//! presented as settled.

use core::fmt;
use core::ops::{Add, AddAssign, Mul, Sub, SubAssign};

/// Mass of population at **Band I**, in kilotons — the anchor the whole ladder
/// hangs from. One unit, in the §2.6 sense: an abstract quantity standing for
/// many real kilotons.
pub const KILOTONS_AT_BAND_I: f64 = 1.0;

/// Multiplicative factor between adjacent Bands.
///
/// **Placeholder pinned to the floor of the ratified range, not a ratified
/// value** (`Hyades_mineral_cost_curve.md` §2.6, R-MC15). The spec requires
/// each of the `I→II` and `II→III` steps to be a rational in `[4, 8]`; `4.0`
/// is the smallest legal choice, so adopting it changes the model as little as
/// the constraint permits while still being *inside* it.
pub const BAND_STEP: f64 = 4.0;

/// The bottom rung the ladder is willing to name.
///
/// `band(m)` diverges as `m → 0`, and design law #16 makes a non-finite value
/// in replicated state a fatal error rather than a number. Masses at or below
/// `Band(BAND_FLOOR).in_kilotons()` therefore read as `BAND_FLOOR`. This is a
/// statement about the *ladder* — it does not describe quantities beneath its
/// first rung — not a claim that such a mass is zero.
pub const BAND_FLOOR: f64 = 0.0;

/// **A named rung on the Band ladder** — the discrete tier, as distinct from
/// [`Band`], which is a *position* and can sit anywhere between rungs.
///
/// This type exists so that a Band-valued constant cannot be written as a bare
/// number. `colony_seed_pop = 1.0` compiled for the entire life of the project
/// and meant "Band I" only by convention; the same `1.0` could as easily have
/// been a mass, a mineral count or a multiplier, and R-O66 is what happens when
/// that convention slips. A rung is now a *name*, and naming it is the only way
/// to write it.
///
/// ```compile_fail
/// # use hyades_engine::sim::SimConfig;
/// let mut cfg = SimConfig::new(1);
/// cfg.colony_seed_pop = 1.0; // a Band level is not a float
/// ```
///
/// ## The rungs
///
/// - [`Empty`](Self::Empty) — below the first threshold: no colony, negligible
///   population, an uncolonizable world, a hull with no hold.
///   `Hyades_mineral_cost_curve.md` §2.6 calls this `Band 0`; it is named
///   rather than numbered here because zero is the one rung that is a
///   *condition* rather than a magnitude.
/// - [`I`](Self::I) — the first crossed threshold, and **each quantity's own
///   reference scale**: a small town, the cost of one General-class hull, a
///   Medium hull's reference hold. §2.6 is explicit that these are anchored
///   *independently* and only the *ratios* are shared.
/// - [`II`](Self::II), [`III`](Self::III) — separated by `F₁`, `F₂`, each a
///   rational in `[4, 8]`.
/// - [`IV`](Self::IV) — the top of the playable ladder. `F₃` (III→IV) is
///   deliberately **unconstrained** (§2.6): it is the last step before a
///   quantity's ceiling and is reserved as a tuning knob.
/// - [`V`](Self::V) — **the maximum, for comparison and clamping only. It is
///   not reachable in play**, and `no_quantity_reaches_band_v` pins that. Its
///   job is to give bounds checks a top end that is a rung rather than a
///   magic number, the way a half-open range wants one past the end.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BandTier {
    Empty,
    I,
    II,
    III,
    IV,
    V,
}

impl BandTier {
    /// Every rung, in order — including [`V`](Self::V), which is why callers
    /// that mean "every rung a world can actually be at" should use
    /// [`PLAYABLE`](Self::PLAYABLE).
    pub const ALL: [BandTier; 6] =
        [BandTier::Empty, BandTier::I, BandTier::II, BandTier::III, BandTier::IV, BandTier::V];
    /// The rungs a quantity can actually occupy in a game. `V` is excluded by
    /// construction.
    pub const PLAYABLE: [BandTier; 5] = [BandTier::Empty, BandTier::I, BandTier::II, BandTier::III, BandTier::IV];
    /// The highest rung anything in a game may reach.
    pub const MAX_PLAYABLE: BandTier = BandTier::IV;

    /// This rung's position on the continuous ladder.
    #[inline]
    pub const fn band(self) -> Band {
        Band(self.index() as f64)
    }

    /// How many rungs above [`Empty`](Self::Empty) — the integer the engine
    /// used before these were named.
    #[inline]
    pub const fn index(self) -> u8 {
        match self {
            BandTier::Empty => 0,
            BandTier::I => 1,
            BandTier::II => 2,
            BandTier::III => 3,
            BandTier::IV => 4,
            BandTier::V => 5,
        }
    }

    /// The rung a continuous position has *reached* — the largest rung at or
    /// below it. Saturates at [`V`](Self::V).
    #[inline]
    pub fn containing(b: Band) -> BandTier {
        match b.bands() {
            x if x < 1.0 => BandTier::Empty,
            x if x < 2.0 => BandTier::I,
            x if x < 3.0 => BandTier::II,
            x if x < 4.0 => BandTier::III,
            x if x < 5.0 => BandTier::IV,
            _ => BandTier::V,
        }
    }
}

impl From<BandTier> for Band {
    #[inline]
    fn from(t: BandTier) -> Band {
        t.band()
    }
}

impl fmt::Display for BandTier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            BandTier::Empty => "Empty",
            BandTier::I => "Band I",
            BandTier::II => "Band II",
            BandTier::III => "Band III",
            BandTier::IV => "Band IV",
            BandTier::V => "Band V",
        };
        f.write_str(s)
    }
}

/// A position on the Band ladder — a magnitude *tier*, not an amount.
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Band(f64);

/// An amount of stuff, in kilotons. The unit conservation is stated in (L6).
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Kilotons(f64);

/// A quantity that can be read in **either** unit.
///
/// This is the "interpreted either way as required" seam: call sites ask for
/// the reading they need by name, so a Band never silently stands in for a
/// mass. Both readings of the same value describe the same thing — the
/// conversion is the ladder, not a reinterpretation of the bits.
pub trait Measure: Copy {
    /// This quantity's position on the Band ladder.
    fn in_bands(self) -> Band;
    /// This quantity as an amount of matter.
    fn in_kilotons(self) -> Kilotons;
}

impl Band {
    /// The bottom of the ladder — a wild world's infrastructure, an
    /// uninhabited world's population.
    pub const ZERO: Band = Band(0.0);
    #[inline]
    pub const fn new(v: f64) -> Self {
        Band(v)
    }
    /// The scalar, **in Bands**. Named so a reader can see the unit at the call
    /// site; there is deliberately no bare `value()`.
    #[inline]
    pub const fn bands(self) -> f64 {
        self.0
    }
    #[inline]
    pub fn min(self, o: Band) -> Band {
        Band(self.0.min(o.0))
    }
    #[inline]
    pub fn max(self, o: Band) -> Band {
        Band(self.0.max(o.0))
    }
    #[inline]
    pub fn clamp(self, lo: Band, hi: Band) -> Band {
        Band(self.0.clamp(lo.0, hi.0))
    }
    /// Nearest whole rung. Infrastructure is built a level at a time, so its
    /// "next level" is a rounding on the ladder, not on a mass.
    #[inline]
    pub fn round(self) -> Band {
        Band(self.0.round())
    }
    #[inline]
    pub fn is_finite(self) -> bool {
        self.0.is_finite()
    }
}

impl Kilotons {
    pub const ZERO: Kilotons = Kilotons(0.0);
    #[inline]
    pub const fn new(v: f64) -> Self {
        Kilotons(v)
    }
    /// The scalar, **in kilotons**.
    #[inline]
    pub const fn kilotons(self) -> f64 {
        self.0
    }
    #[inline]
    pub fn min(self, o: Kilotons) -> Kilotons {
        Kilotons(self.0.min(o.0))
    }
    #[inline]
    pub fn max(self, o: Kilotons) -> Kilotons {
        Kilotons(self.0.max(o.0))
    }
    #[inline]
    pub fn clamp(self, lo: Kilotons, hi: Kilotons) -> Kilotons {
        Kilotons(self.0.clamp(lo.0, hi.0))
    }
    #[inline]
    pub fn is_finite(self) -> bool {
        self.0.is_finite()
    }
}

impl Measure for Band {
    #[inline]
    fn in_bands(self) -> Band {
        self
    }
    /// Exponential, per §2.6, and deliberately **unclamped**: the growth step
    /// conserves mass across this map, so a clamp here is mass appearing or
    /// vanishing at the clamp.
    #[inline]
    fn in_kilotons(self) -> Kilotons {
        Kilotons(KILOTONS_AT_BAND_I * BAND_STEP.powf(self.0 - 1.0))
    }
}

impl Measure for Kilotons {
    #[inline]
    fn in_bands(self) -> Band {
        if self.0 <= 0.0 {
            return Band(BAND_FLOOR);
        }
        Band((1.0 + (self.0 / KILOTONS_AT_BAND_I).ln() / BAND_STEP.ln()).max(BAND_FLOOR))
    }
    #[inline]
    fn in_kilotons(self) -> Kilotons {
        self
    }
}

/// The mass of a population standing at Band `pop`.
///
/// This is [`Measure::in_kilotons`] with one discontinuity, at exactly zero:
/// **no people is no mass.** The ladder is logarithmic and so has no bottom —
/// `KT(0)` is a small positive number, not nothing — but an uninhabited world
/// is not carrying a fifth of a kiloton of latent citizenry, and charging the
/// biosphere for it at founding would be mass conjured from the ladder's
/// shape. Every positive population sits somewhere on the ladder; zero is off
/// it.
#[inline]
pub fn population_mass(pop: Band) -> Kilotons {
    if pop.0 <= 0.0 {
        Kilotons::ZERO
    } else {
        pop.in_kilotons()
    }
}

/// The population a world can stand at, given `mass` of biomass committed to
/// people — the inverse of [`population_mass`].
#[inline]
pub fn population_at_mass(mass: Kilotons) -> Band {
    if mass.0 <= 0.0 {
        Band::ZERO
    } else {
        mass.in_bands()
    }
}

macro_rules! arith {
    ($t:ty) => {
        impl Add for $t {
            type Output = $t;
            #[inline]
            fn add(self, o: $t) -> $t {
                Self(self.0 + o.0)
            }
        }
        impl Sub for $t {
            type Output = $t;
            #[inline]
            fn sub(self, o: $t) -> $t {
                Self(self.0 - o.0)
            }
        }
        impl AddAssign for $t {
            #[inline]
            fn add_assign(&mut self, o: $t) {
                self.0 += o.0;
            }
        }
        impl SubAssign for $t {
            #[inline]
            fn sub_assign(&mut self, o: $t) {
                self.0 -= o.0;
            }
        }
        impl Mul<f64> for $t {
            type Output = $t;
            #[inline]
            fn mul(self, k: f64) -> $t {
                Self(self.0 * k)
            }
        }
    };
}
arith!(Band);
arith!(Kilotons);

/// Printed with its unit, for the same reason it is typed: a bare number in a
/// log line is the thing this module exists to stop.
impl fmt::Display for Band {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Band {:.3}", self.0)
    }
}
impl fmt::Display for Kilotons {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.3} kt", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The bridge must be the *ladder*, not a linear rescale — a linear bridge
    /// would silently contradict `Hyades_mineral_cost_curve.md` §2.6, which is
    /// the same class of error the types exist to prevent.
    #[test]
    fn a_band_step_is_multiplicative_not_additive() {
        let one = Band::new(1.0).in_kilotons().kilotons();
        let two = Band::new(2.0).in_kilotons().kilotons();
        let three = Band::new(3.0).in_kilotons().kilotons();
        assert!((two / one - BAND_STEP).abs() < 1e-9, "I→II must be a factor of {BAND_STEP}, got {}", two / one);
        assert!((three / two - BAND_STEP).abs() < 1e-9, "II→III must be a factor of {BAND_STEP}, got {}", three / two);
        assert!((4.0..=8.0).contains(&BAND_STEP), "§2.6 requires the step in [4, 8], got {BAND_STEP}");
    }

    /// **Band V is a comparison ceiling, not a destination.**
    ///
    /// It exists so a bounds check has a rung one past the end instead of a
    /// magic number, and the design is that nothing in a game ever reaches it.
    /// Pinned here because an unreachable value that quietly becomes reachable
    /// is the worst kind of sentinel — every `< V` guard would keep compiling
    /// and stop meaning anything.
    #[test]
    fn band_v_is_one_past_the_playable_end() {
        assert_eq!(BandTier::MAX_PLAYABLE, BandTier::IV);
        assert!(BandTier::V > BandTier::MAX_PLAYABLE);
        assert!(!BandTier::PLAYABLE.contains(&BandTier::V), "V must not be in the playable set");
        assert_eq!(BandTier::PLAYABLE.len() + 1, BandTier::ALL.len());

        // The generator's ceiling is Band IV, so no world can be seeded past it
        // and `containing` only reports V for a position off the ladder.
        assert_eq!(BandTier::containing(Band::new(4.0)), BandTier::IV);
        assert_eq!(BandTier::containing(Band::new(4.999)), BandTier::IV);
        assert_eq!(BandTier::containing(Band::new(5.0)), BandTier::V);
    }

    /// The rungs are ordered, and the order is the ladder's.
    #[test]
    fn the_rungs_are_ordered_and_indexed_consistently() {
        for (i, t) in BandTier::ALL.iter().enumerate() {
            assert_eq!(t.index() as usize, i);
            assert_eq!(t.band(), Band::new(i as f64));
        }
        assert!(BandTier::Empty < BandTier::I && BandTier::I < BandTier::IV);
    }

    /// `Empty` is the rung that is a *condition*, not a magnitude — no colony,
    /// no hold, an uncolonizable world — which is why it is named rather than
    /// numbered. It sits at the ladder's origin.
    #[test]
    fn empty_is_the_origin_of_the_ladder() {
        assert_eq!(BandTier::Empty.band(), Band::ZERO);
        assert_eq!(BandTier::containing(Band::ZERO), BandTier::Empty);
        assert_eq!(BandTier::containing(Band::new(0.999)), BandTier::Empty);
    }

    #[test]
    fn the_two_readings_round_trip() {
        for b in [0.25, 1.0, 1.5, 2.0, 3.25, 4.0] {
            let there_and_back = Band::new(b).in_kilotons().in_bands().bands();
            assert!((there_and_back - b).abs() < 1e-9, "{b} round-tripped to {there_and_back}");
        }
    }

    /// Zero biomass is the ladder's floor — "below the first rung" — and must
    /// not be a negative infinity that poisons replicated state (design law
    /// #16). Neither may any mass *near* zero, which is the case the naive
    /// `if m <= 0` guard misses: `ln` of a denormal is about −745, finite but
    /// meaningless, and it is the value a nearly-extinct biosphere produces.
    #[test]
    fn a_vanishing_mass_floors_at_the_bottom_rung_and_never_diverges() {
        for m in [0.0, f64::MIN_POSITIVE, 1e-300, 1e-12] {
            let b = Kilotons::new(m).in_bands();
            assert!(b.is_finite(), "{m} kt read as a non-finite Band");
            assert_eq!(b, Band::new(BAND_FLOOR), "{m} kt should floor at the bottom rung");
        }
    }

    /// The whole point of the exponential bridge: a population's mass and the
    /// biosphere's Band ceiling bind at the *same* place, so "enough biomass to
    /// make these people" and "population <= bio_max" are one inequality. If
    /// this fails the two constraints have drifted apart and `k_potential`
    /// stops predicting what growth can actually afford.
    #[test]
    fn the_mass_budget_and_the_band_ceiling_bind_together() {
        for band in [1.0, 2.0, 3.0, 4.0] {
            let ceiling = Band::new(band);
            let bio_max = ceiling.in_kilotons();
            assert!((population_mass(ceiling).kilotons() - bio_max.kilotons()).abs() < 1e-9);
            // A hair above the ceiling costs strictly more than the world holds.
            assert!(population_mass(Band::new(band + 1e-6)) > bio_max);
        }
    }

    /// No people is no mass — a world nobody lives on must not owe the
    /// biosphere `KT(0)` kilotons just because the ladder is logarithmic.
    #[test]
    fn an_empty_world_carries_no_population_mass() {
        assert_eq!(population_mass(Band::ZERO), Kilotons::ZERO);
        assert!(population_mass(Band::new(1e-9)) > Kilotons::ZERO);
    }
}
