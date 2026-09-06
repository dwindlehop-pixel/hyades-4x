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
use core::ops::{Add, AddAssign, Div, Mul, Sub, SubAssign};

/// Mass of population at **Band I**, in kilotons — the anchor the whole ladder
/// hangs from, and now a *ratified* number rather than a convenient one
/// (`Hyades_mineral_cost_curve.md` §2.6).
///
/// Every ratio on both ladders is fixed by the step factors, so the only
/// freedom left is where `Band I` sits in real kilotons. It is one kiloton: a
/// small town of ~3,333 people at ~300 kg of person, possessions and
/// pressurised living volume each, and — the same number seen from the other
/// side — a Medium hull's reference hold. That coincidence is the anchor's
/// whole job, and it is why `SimConfig::cargo_unit_size` lands on 1.0 too.
pub const KILOTONS_AT_BAND_I: f64 = 1.0;

/// **The ratified mass ladder** (`Hyades_mineral_cost_curve.md` §2.6, R-MC15).
///
/// `MASS_LADDER[n]` is the factor from rung `n` to rung `n+1`, indexed the way
/// [`BandTier::index`] indexes: `0 = Empty→I`, `1 = I→II`, `2 = II→III`,
/// `3 = III→IV`. There is no single "band step" any more, which is the whole
/// point — the ratified constraint is on how the factors *grow*
/// (`1 < F₍ₙ₊₁₎/Fₙ < 10`), not on their absolute size, and this ladder grows
/// at a uniform ratio of `2^1.5 ≈ 2.83`.
///
/// Each entry is `F_cost^(3/2)` for the cost ladder's `5, 10, 20, 40` — the
/// ratified tie between the two ladders, and the shell model's own exponent:
/// cost tracks `r²` and the hold tracks `r³`.
pub const MASS_LADDER: [f64; 4] = [
    11.180_339_887_498_949, // 5^1.5   Empty → I
    31.622_776_601_683_793, // 10^1.5  I → II
    89.442_719_099_991_59,  // 20^1.5  II → III
    252.982_212_813_470_36, // 40^1.5  III → IV
];

/// The mass at rung `n`, hung off [`KILOTONS_AT_BAND_I`] — `Band I` is the
/// anchor, so `Empty` is *below* it by the first ladder step and every rung
/// above is a running product of the rest.
#[inline]
pub const fn rung_mass(n: usize) -> f64 {
    match n {
        0 => KILOTONS_AT_BAND_I / MASS_LADDER[0],
        1 => KILOTONS_AT_BAND_I,
        2 => KILOTONS_AT_BAND_I * MASS_LADDER[1],
        3 => KILOTONS_AT_BAND_I * MASS_LADDER[1] * MASS_LADDER[2],
        _ => KILOTONS_AT_BAND_I * MASS_LADDER[1] * MASS_LADDER[2] * MASS_LADDER[3],
    }
}

/// Which ladder segment a continuous position falls in, as `(lower rung, its
/// step factor)`. Positions outside the playable ladder extrapolate with the
/// nearest segment's factor rather than clamping, because the bridge must stay
/// continuous *and* invertible: a clamp here is mass created or destroyed at
/// the clamp, which is the one thing L6 does not permit.
#[inline]
fn segment(b: f64) -> (f64, f64) {
    let n = if b < 0.0 {
        0.0
    } else if b >= 3.0 {
        3.0
    } else {
        b.floor()
    };
    (n, MASS_LADDER[n as usize])
}

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
/// - [`Zero`](Self::Zero) — **the bottom sentinel, one past the start of the
///   ladder, and the mirror of [`V`](Self::V) at the other end.** It is not a
///   small quantity; it is the *absence* of one, and no live quantity is ever
///   at it — `BAND_FLOOR` puts the smallest representable magnitude at
///   `Empty`. Its job is to give a `> Zero` guard a rung to compare against
///   instead of a magic number, now that `Empty > 0` and `Empty` can no longer
///   do that job. Its ladder position is `Band(-1.0)`, deliberately outside
///   `[BAND_FLOOR, …]`, so adding it shifts nothing above it.
/// - [`Empty`](Self::Empty) — **a positive magnitude beneath `Band I`'s
///   threshold**, not zero and not an absence: a hamlet rather than a town, a
///   Limited hull's token hold. Ratified this way explicitly — there is no
///   `Band 0`, and `Band Empty > 0`. It is named rather than numbered so it
///   cannot be read as the integer zero, which is the one value that is *off*
///   the ladder rather than on its bottom rung.
/// - [`I`](Self::I) — the first crossed threshold, and **each quantity's own
///   reference scale**: a small town, the cost of one General-class hull, a
///   Medium hull's reference hold. §2.6 is explicit that these are anchored
///   *independently* and only the *ratios* are shared.
/// - [`II`](Self::II), [`III`](Self::III), [`IV`](Self::IV) — separated by
///   step factors `F₁`, `F₂`, `F₃`. The old `[4, 8]` window on those factors
///   is **superseded and removed** (R-MC15, ratified). The constraint is now
///   on how the factors *grow*: `1 < F₍ₙ₊₁₎/Fₙ < 10`, so each step is strictly
///   larger than the last and by less than a decade. There are two ladders —
///   **mass** (population, biosphere, cargo) and **mineral cost** — tied by
///   `F_mass = F_cost^(3/2)`, which is the shell model's own exponent.
/// - [`IV`](Self::IV) is the top of the playable ladder.
/// - [`V`](Self::V) — **the maximum, for comparison and clamping only. It is
///   not reachable in play**, and `no_quantity_reaches_band_v` pins that. Its
///   job is to give bounds checks a top end that is a rung rather than a
///   magic number, the way a half-open range wants one past the end.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BandTier {
    Zero,
    Empty,
    I,
    II,
    III,
    IV,
    V,
}

impl BandTier {
    /// Every rung, in order — including both sentinels, [`Zero`](Self::Zero)
    /// and [`V`](Self::V), which is why callers that mean "every rung a world
    /// can actually be at" must use [`PLAYABLE`](Self::PLAYABLE).
    ///
    /// **Do not index this by a crossing count.** `PLAYABLE` is the array whose
    /// positions are the ladder's; this one is offset by the bottom sentinel.
    pub const ALL: [BandTier; 7] =
        [BandTier::Zero, BandTier::Empty, BandTier::I, BandTier::II, BandTier::III, BandTier::IV, BandTier::V];
    /// The rungs a quantity can actually occupy in a game. Both sentinels are
    /// excluded by construction, and position `i` in this array is the rung a
    /// quantity has reached after crossing `i` thresholds.
    pub const PLAYABLE: [BandTier; 5] = [BandTier::Empty, BandTier::I, BandTier::II, BandTier::III, BandTier::IV];
    /// The highest rung anything in a game may reach.
    pub const MAX_PLAYABLE: BandTier = BandTier::IV;
    /// The lowest rung anything in a game may reach. `Zero` is beneath it and
    /// is not a magnitude.
    pub const MIN_PLAYABLE: BandTier = BandTier::Empty;

    /// This rung's position on the continuous ladder.
    #[inline]
    pub const fn band(self) -> Band {
        Band(self.index() as f64)
    }

    /// How many rungs above [`Empty`](Self::Empty) — the integer the engine
    /// used before these were named. Signed, because [`Zero`](Self::Zero) sits
    /// one *below* the origin; every other rung keeps the index it had, so
    /// adding the bottom sentinel moved no ladder position.
    #[inline]
    pub const fn index(self) -> i8 {
        match self {
            BandTier::Zero => -1,
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
            x if x < 0.0 => BandTier::Zero,
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
            BandTier::Zero => "Band Zero",
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

/// A length, in **hull units** — the shell model's linear measure
/// (`Hyades_mineral_cost_curve.md` §2.3).
///
/// The convention is normalized radius, `V = r³`: hull units are chosen so that
/// a hull's displaced volume is the cube of its radius, which is what the
/// engine's `hull_radius`/`cargo_capacity` pair has always assumed without
/// saying so. Shell thickness `τ` is a [`Length`] in the same unit, which is
/// the point of the type — the model's whole content is that `r` and `τ` are
/// commensurable and `r − τ` is the hold radius.
///
/// Typed for the same reason [`Band`] and [`Kilotons`] are: R-O66 shipped a
/// `min` across incompatible units that typechecked and read as plausible
/// ecology. Hull geometry has the same shape of hazard — a radius, a thickness,
/// an area and a volume are all `f64` and only one of the twelve ways to
/// combine them is right. None of the wrong ones compile now:
///
/// ```compile_fail
/// # use hyades_engine::units::{Length, Volume};
/// // A length is not a volume, however plausible the arithmetic looks.
/// let r = Length::new(3.0);
/// let v: Volume = r - Length::new(1.0);
/// ```
///
/// ```compile_fail
/// # use hyades_engine::units::{Length, Kilotons};
/// // A hold is a volume; a cargo is a mass. The conversion is a density, and
/// // it is not an implicit one.
/// let hold = Length::new(2.0).cubed();
/// let m: Kilotons = hold;
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Length(f64);

/// An area, in hull units squared — the shell model's **cost basis**
/// (design law #3: surface area is what is paid for).
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Area(f64);

/// A volume, in hull units cubed — the shell model's **value basis**, and the
/// quantity that sits on a Band rung (§2.3: the rung is the *hold*, and
/// `V_reserved` is deducted after it).
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Volume(f64);

impl Length {
    pub const ZERO: Length = Length(0.0);
    #[inline]
    pub const fn new(v: f64) -> Self {
        Length(v)
    }
    /// The scalar, **in hull units**.
    #[inline]
    pub const fn hull_units(self) -> f64 {
        self.0
    }
    #[inline]
    pub const fn squared(self) -> Area {
        Area(self.0 * self.0)
    }
    #[inline]
    pub const fn cubed(self) -> Volume {
        Volume(self.0 * self.0 * self.0)
    }
    #[inline]
    pub fn min(self, o: Length) -> Length {
        Length(self.0.min(o.0))
    }
    #[inline]
    pub fn max(self, o: Length) -> Length {
        Length(self.0.max(o.0))
    }
    #[inline]
    pub fn is_finite(self) -> bool {
        self.0.is_finite()
    }
}

impl Area {
    pub const ZERO: Area = Area(0.0);
    #[inline]
    pub const fn new(v: f64) -> Self {
        Area(v)
    }
    /// The scalar, **in hull units squared**.
    #[inline]
    pub const fn hull_units_squared(self) -> f64 {
        self.0
    }
    #[inline]
    pub fn sqrt(self) -> Length {
        Length(self.0.sqrt())
    }
    #[inline]
    pub fn is_finite(self) -> bool {
        self.0.is_finite()
    }
}

impl Volume {
    pub const ZERO: Volume = Volume(0.0);
    #[inline]
    pub const fn new(v: f64) -> Self {
        Volume(v)
    }
    /// The scalar, **in hull units cubed**.
    #[inline]
    pub const fn hull_units_cubed(self) -> f64 {
        self.0
    }
    #[inline]
    pub fn cbrt(self) -> Length {
        Length(self.0.cbrt())
    }
    #[inline]
    pub fn max(self, o: Volume) -> Volume {
        Volume(self.0.max(o.0))
    }
    #[inline]
    pub fn is_finite(self) -> bool {
        self.0.is_finite()
    }
}

impl Mul<Length> for Area {
    type Output = Volume;
    #[inline]
    fn mul(self, l: Length) -> Volume {
        Volume(self.0 * l.0)
    }
}
impl Div<Area> for Volume {
    type Output = Length;
    #[inline]
    fn div(self, a: Area) -> Length {
        Length(self.0 / a.0)
    }
}
impl Div<Length> for Volume {
    type Output = Area;
    #[inline]
    fn div(self, l: Length) -> Area {
        Area(self.0 / l.0)
    }
}
/// A ratio of two volumes is a pure number — this is the only sanctioned way
/// out of the geometric types, and it is how a hold becomes a load: the ratio
/// against a reference hold, times the mass that reference hold carries.
impl Div<Volume> for Volume {
    type Output = f64;
    #[inline]
    fn div(self, o: Volume) -> f64 {
        self.0 / o.0
    }
}
impl Div<Area> for Area {
    type Output = f64;
    #[inline]
    fn div(self, o: Area) -> f64 {
        self.0 / o.0
    }
}
impl Div<Length> for Length {
    type Output = f64;
    #[inline]
    fn div(self, o: Length) -> f64 {
        self.0 / o.0
    }
}

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
        let (n, step) = segment(self.0);
        Kilotons(rung_mass(n as usize) * step.powf(self.0 - n))
    }
}

impl Measure for Kilotons {
    #[inline]
    fn in_bands(self) -> Band {
        if self.0 <= 0.0 {
            return Band(BAND_FLOOR);
        }
        let mut n = 0usize;
        while n < 3 && self.0 >= rung_mass(n + 1) {
            n += 1;
        }
        Band((n as f64 + (self.0 / rung_mass(n)).ln() / MASS_LADDER[n].ln()).max(BAND_FLOOR))
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
arith!(Length);
arith!(Area);
arith!(Volume);

/// Printed with its unit, for the same reason it is typed: a bare number in a
/// log line is the thing this module exists to stop.
impl fmt::Display for Band {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Band {:.3}", self.0)
    }
}
impl fmt::Display for Length {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.4} hu", self.0)
    }
}
impl fmt::Display for Area {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.4} hu²", self.0)
    }
}
impl fmt::Display for Volume {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.4} hu³", self.0)
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
        // Each rung is its ladder factor times the last, and the factors are
        // the ratified ones — not one shared step, which is what R-MC15
        // withdrew.
        for (n, &step) in MASS_LADDER.iter().enumerate() {
            let lo = Band::new(n as f64).in_kilotons().kilotons();
            let hi = Band::new(n as f64 + 1.0).in_kilotons().kilotons();
            let got = hi / lo;
            assert!((got - step).abs() < 1e-9, "rung {n}→{} must step by {step}, got {got}", n + 1);
        }

        // The ratified constraint is on how the factors *grow*: strictly
        // increasing, and by less than a decade each time.
        for pair in MASS_LADDER.windows(2) {
            let ratio = pair[1] / pair[0];
            assert!(ratio > 1.0 && ratio < 10.0, "{} / {} = {ratio}, outside (1, 10)", pair[1], pair[0]);
        }

        // And the tie to the cost ladder is the shell model's exponent: cost
        // tracks r², the hold tracks r³. A cost ladder of 5, 10, 20, 40.
        for (n, cost_step) in [5.0_f64, 10.0, 20.0, 40.0].into_iter().enumerate() {
            assert!((MASS_LADDER[n] - cost_step.powf(1.5)).abs() < 1e-9, "F_mass must be F_cost^(3/2) at rung {n}");
        }
    }

    /// The bridge is piecewise now, so it has three interior joins where it
    /// could silently develop a step. It must not: the growth draw is
    /// `KT(after) − KT(before)`, and a discontinuity there is mass created or
    /// destroyed at a rung boundary (L6).
    #[test]
    fn the_piecewise_bridge_is_continuous_and_monotone_across_every_join() {
        for rung in 0..=4 {
            let b = rung as f64;
            let below = Band::new(b - 1e-9).in_kilotons().kilotons();
            let at = Band::new(b).in_kilotons().kilotons();
            let above = Band::new(b + 1e-9).in_kilotons().kilotons();
            assert!((below / at - 1.0).abs() < 1e-6, "join at rung {rung} steps from below: {below} vs {at}");
            assert!((above / at - 1.0).abs() < 1e-6, "join at rung {rung} steps from above: {above} vs {at}");
            assert!(below <= at && at <= above, "not monotone at rung {rung}");
        }

        // Monotone everywhere, including the extrapolated ends.
        let mut prev = f64::NEG_INFINITY;
        let mut x = -0.5;
        while x <= 5.0 {
            let m = Band::new(x).in_kilotons().kilotons();
            assert!(m > prev, "mass must rise with Band; fell at {x}");
            assert!(m.is_finite(), "non-finite mass at Band {x}");
            prev = m;
            x += 0.01;
        }
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
        // Two sentinels now bracket the playable set, one at each end.
        assert_eq!(BandTier::PLAYABLE.len() + 2, BandTier::ALL.len());

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
            let rung = i as i8 - 1; // ALL leads with the bottom sentinel
            assert_eq!(t.index(), rung);
            assert_eq!(t.band(), Band::new(rung as f64));
        }
        // PLAYABLE, by contrast, is indexed by crossing count — which is what
        // `PopBands::level` relies on.
        for (i, t) in BandTier::PLAYABLE.iter().enumerate() {
            assert_eq!(t.index() as usize, i);
        }
        assert!(BandTier::Zero < BandTier::Empty);
        assert!(BandTier::Empty < BandTier::I && BandTier::I < BandTier::IV);
    }

    /// **`Zero` is the bottom sentinel and nothing in a game reaches it** — the
    /// mirror of `band_v_is_one_past_the_playable_end`.
    ///
    /// It exists because ratifying `Empty > 0` took away the rung that used to
    /// mean "none of this quantity". A `> Zero` comparison now has a name to
    /// make; without one the check goes back to being a bare `0.0`, which is
    /// the whole failure mode this type exists to close.
    #[test]
    fn band_zero_is_one_past_the_bottom_and_unreachable() {
        assert!(BandTier::Zero < BandTier::MIN_PLAYABLE);
        assert_eq!(BandTier::Zero.band(), Band::new(-1.0));
        assert!(!BandTier::PLAYABLE.contains(&BandTier::Zero));

        // BAND_FLOOR is the smallest position the ladder will name, and it is
        // at `Empty` — so no mass, however small, classifies as `Zero`.
        assert_eq!(BandTier::containing(Band::new(BAND_FLOOR)), BandTier::Empty);
        assert_eq!(Kilotons::ZERO.in_bands(), Band::new(BAND_FLOOR));
        for m in [0.0, 1e-300, 1e-9, 1e-3] {
            assert!(
                BandTier::containing(Kilotons::new(m).in_bands()) >= BandTier::MIN_PLAYABLE,
                "a mass of {m} kt classified beneath the playable floor"
            );
        }
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
