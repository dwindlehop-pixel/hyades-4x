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
//! previous Band's magnitude". R-MC15 withdrew the idea of one shared step and
//! ratified a **ladder** of them ([`MASS_LADDER`]), so the conversion is
//! exponential *piecewise*, one segment per rung:
//!
//! ```text
//! kilotons(b) = rung_mass(n) · MASS_LADDER[n]^(b − n)      n = segment of b
//! band(m)     = n + log(m / rung_mass(n)) / log(MASS_LADDER[n])
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
//! the *same* inequality, for any ladder at all — which is why the old
//! `min(hab, bio, infra)` produced sensible play despite comparing a mass
//! against two levels. The fix keeps that agreement and makes it derivable
//! rather than coincidental: the ceiling is a Band minimum, the draw is a
//! mass, and they bind at the same place.
//!
//! A *linear* bridge would have been the easy choice and would have quietly
//! contradicted the ladder spec — Band II would have been twice Band I instead
//! of thirty-odd times it. Getting this wrong is the same class of error as the
//! one the types exist to prevent, one level up.
//!
//! **The ladder is ratified; the ladder's *floor* is set separately.** R-MC15
//! fixes the step factors across `I → II → III → IV` and ties them to
//! [`COST_LADDER`] by the shell model's `3/2`. It says nothing about how far
//! below `Band I` the ladder keeps naming magnitudes, which is a per-quantity
//! anchor under §2.6 — see [`KILOTONS_AT_BAND_EMPTY`].

use core::fmt;
use core::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};

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
/// **The ratified cost ladder** (`Hyades_mineral_cost_curve.md` §2.6, R-MC15) —
/// the step factors between adjacent Bands of *mineral spend*.
///
/// This is the primitive; [`MASS_LADDER`] is these raised to `3/2`, which is
/// the ratified tie between the two ladders and the shell model's own exponent
/// (cost tracks `r²`, the hold tracks `r³`).
pub const COST_LADDER: [f64; 4] = [5.0, 10.0, 20.0, 40.0];

pub const MASS_LADDER: [f64; 4] = [
    KILOTONS_AT_BAND_I / KILOTONS_AT_BAND_EMPTY, // 1000  Empty → I — the *floor*, set below
    31.622_776_601_683_793,                      // 10^1.5  I → II
    89.442_719_099_991_59,                       // 20^1.5  II → III
    252.982_212_813_470_36,                      // 40^1.5  III → IV
];

/// **Where the mass ladder bottoms out — one metric tonne.** Directed: *"the
/// setting of `Band Empty` is way too high. Let's set `Band Empty` to 1 metric
/// ton… I want to change the *width* of `Band Empty`, not reset the ladder from
/// there."*
///
/// `Band Empty` is not a rung of the ratified ladder; it is the ladder's
/// **floor**, and its width is the one degree of freedom R-MC15 does not fix.
/// R-MC15 ratified the step factors and the `F_mass = F_cost^(3/2)` tie between
/// the two ladders; both statements are about how the ladder *grows*, and they
/// hold across `I → II → III → IV`, whose factors grow at a uniform 2.83. What
/// sits *below* `Band I` is a different question — "how small a positive
/// magnitude will the ladder still name?" — and §2.6 already answers it
/// per-quantity: every quantity anchors its own scale and only the ratios are
/// shared. So the two ladders' Empty widths are **not** tied to each other, and
/// [`COST_LADDER`]`[0] = 5` is untouched by this: it is the Limited hull's
/// price, which is a real rung on a real ladder.
///
/// **What it buys.** Everything sub-`Band I` is read on this segment, and at
/// the old `1/11.18 ≈ 0.089 kt` floor the segment was far too narrow to
/// resolve anything: a Medium hull's ~0.1 kt hold read as **`Band 0.046`**, a
/// rounding error away from founding nothing at all. At one tonne the same
/// hold reads **`Band 0.67`**. The colony a Medium colonizer founds is the
/// thing this number sets.
///
/// **What it costs, stated rather than hidden:** the two invariants above are
/// now claims about the playable ladder, not about `MASS_LADDER` as an array.
/// `1000 / 31.6` is not in `(1, 10)` and `1000` is not `5^1.5`, and the tests
/// say so in those words.
pub const KILOTONS_AT_BAND_EMPTY: f64 = 0.001;

/// The mass at rung `n` — [`Qty::rung`] on the mass ladder, kept as a free
/// function because the ladder's own tests read more clearly with it.
#[inline]
pub fn rung_mass(n: usize) -> f64 {
    Kilotons::rung(n)
}

/// **A Band ladder with its own anchor** — the general form of the bridge.
///
/// §2.6 is explicit that *every quantity anchors its own `Band I`
/// independently; what has to be shared is the ratio, not the absolute value*.
/// [`Measure`] hard-wires the **mass** ladder (`Band I` = one kiloton, a small
/// town), which is right for population, biosphere and cargo and wrong for
/// anything priced in minerals. This is how a second quantity gets a ladder
/// without a second copy of the arithmetic.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ladder {
    rungs: [f64; 5],
    steps: [f64; 4],
}

impl Ladder {
    /// Build a ladder from one known rung and the step factors between them.
    ///
    /// `anchor_rung` is indexed like [`BandTier::index`]: `0` is `Empty`, `1`
    /// is `Band I`, and so on.
    pub fn anchored_at(anchor_rung: usize, value: f64, steps: [f64; 4]) -> Ladder {
        let mut rungs = [0.0f64; 5];
        rungs[anchor_rung] = value;
        let mut i = anchor_rung;
        while i > 0 {
            rungs[i - 1] = rungs[i] / steps[i - 1];
            i -= 1;
        }
        let mut i = anchor_rung;
        while i < 4 {
            rungs[i + 1] = rungs[i] * steps[i];
            i += 1;
        }
        Ladder { rungs, steps }
    }

    /// The magnitude at whole rung `n`.
    #[inline]
    pub fn rung(&self, n: usize) -> f64 {
        self.rungs[n.min(4)]
    }

    /// Where a magnitude sits on this ladder, interpolated log-linearly inside
    /// its segment and extrapolated with the edge factor outside — the same
    /// shape as [`Measure::in_bands`], and floored at [`BAND_FLOOR`] for the
    /// same reason (design law #16: `band(0)` is `−∞`).
    pub fn band_of(&self, v: f64) -> Band {
        if v <= 0.0 {
            return Band(BAND_FLOOR);
        }
        let mut n = 0usize;
        while n < 3 && v >= self.rungs[n + 1] {
            n += 1;
        }
        Band((n as f64 + (v / self.rungs[n]).ln() / self.steps[n].ln()).max(BAND_FLOOR))
    }

    /// The magnitude at a continuous position — the inverse of
    /// [`band_of`](Self::band_of).
    pub fn value_of(&self, b: Band) -> f64 {
        let n = if b.0 < 0.0 {
            0.0
        } else if b.0 >= 3.0 {
            3.0
        } else {
            b.0.floor()
        };
        self.rungs[n as usize] * self.steps[n as usize].powf(b.0 - n)
    }
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

/// **Which ladder a quantity's Band reading is taken on.**
///
/// There are two, they are both ratified (R-MC15), and their step factors
/// genuinely differ: `F_mass = F_cost^(3/2)` is the shell model's own exponent,
/// because cost tracks surface area and the hold tracks volume. A General hull
/// costs 10x a Medium and holds 31.6x, so its price and its hold cannot both
/// land on the same rung of one ladder. That is geometry, not an accident of
/// units, so the ladder has to travel with the value.
///
/// It travels as a **type parameter**, which is what keeps the arithmetic free:
/// [`Qty`] is `#[repr(transparent)]` over one `f64` and the marker is
/// zero-sized, so `Qty<Mass>` and `Qty<Cost>` have the codegen and the
/// vectorisation of a bare `f64`. Only the Band *conversion* costs anything,
/// and it is not on any hot path.
pub trait Scale: Copy + 'static {
    /// Step factors between adjacent rungs, indexed the way [`BandTier::index`]
    /// indexes: `0 = Empty→I`, `1 = I→II`, `2 = II→III`, `3 = III→IV`.
    const STEPS: [f64; 4];
    /// Kilotons at this scale's `Band I` — its anchor. §2.6: every quantity
    /// anchors its own `Band I`; only the ratios are shared.
    const BAND_I: f64;
    /// For diagnostics.
    const NAME: &'static str;
}

/// The **mass** ladder: population, biosphere, cargo hold, ore in the ground.
/// `Band I` is one kiloton.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Mass;

/// The **mineral-cost** ladder: hull price, infrastructure steps, card cost.
/// Its `Band I` is a Medium hull, and its `Band II` — one General hull — is
/// what `SimConfig::general_vehicle_cost` names.
///
/// Costs are masses too (R-O57: a hull's price and its empty mass are one
/// number in one unit), so a `Qty<Cost>` and a `Qty<Mass>` hold the same kind
/// of scalar. What differs is only the ladder their Band readings are taken
/// on, which is why crossing between them is [`Qty::on_scale`] — explicit,
/// free, and visible at the call site.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Cost;

impl Scale for Mass {
    const STEPS: [f64; 4] = MASS_LADDER;
    const BAND_I: f64 = KILOTONS_AT_BAND_I;
    const NAME: &'static str = "mass";
}

impl Scale for Cost {
    const STEPS: [f64; 4] = COST_LADDER;
    const BAND_I: f64 = MINERALS_AT_BAND_I;
    const NAME: &'static str = "cost";
}

/// Minerals at **cost `Band I`** — one Medium Systems hull.
///
/// The cost ladder's anchor, and the same statement as
/// `SimConfig::general_vehicle_cost = 1.0`: a General hull is cost `Band II`,
/// one step of `COST_LADDER[1] = 10` above this.
pub const MINERALS_AT_BAND_I: f64 = KILOTONS_AT_BAND_I / COST_LADDER[1];

/// **An amount of stuff, in kilotons, that knows which ladder it reads on.**
///
/// One number. Kilotons is the storage and the only thing arithmetic touches;
/// a Band is a *reading* of that number — the shorthand the game design is
/// written in, for print and for thresholds, never a second counting system
/// the simulation steps in. Every logistic, price and conservation check runs
/// on the mass.
///
/// It can be written either way and read either way, and the two commute: a
/// value written as `Band II` and a value written as its kiloton count are the
/// same bits, so which end a term came from never shows up in a sum.
///
/// ```
/// # use hyades_engine::units::{Qty, Mass, BandTier};
/// let a = Qty::<Mass>::at(BandTier::I, 0.0);
/// let b = Qty::<Mass>::new(1.0);
/// assert_eq!(a, b);
/// assert_eq!((a + b).kilotons(), 2.0);
/// ```
#[repr(transparent)]
pub struct Qty<S>(f64, core::marker::PhantomData<S>);

/// An amount of stuff on the **mass** ladder — the unit conservation is stated
/// in (L6). The engine's default quantity.
pub type Kilotons = Qty<Mass>;

// Hand-written so the marker never imposes a bound on `S`: a `Qty` is one
// `f64` and copies like one, whatever it is tagged with.
impl<S> Clone for Qty<S> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}
impl<S> Copy for Qty<S> {}
impl<S> Default for Qty<S> {
    #[inline]
    fn default() -> Self {
        Qty(0.0, core::marker::PhantomData)
    }
}
impl<S> PartialEq for Qty<S> {
    #[inline]
    fn eq(&self, o: &Self) -> bool {
        self.0 == o.0
    }
}
impl<S> PartialOrd for Qty<S> {
    #[inline]
    fn partial_cmp(&self, o: &Self) -> Option<core::cmp::Ordering> {
        self.0.partial_cmp(&o.0)
    }
}

impl<S> Qty<S> {
    pub const ZERO: Qty<S> = Qty(0.0, core::marker::PhantomData);

    /// Write it as a plain amount.
    #[inline]
    pub const fn kilotons(self) -> f64 {
        self.0
    }

    /// **Reinterpret the same amount on another ladder.**
    ///
    /// A no-op on the bits, because a cost *is* a mass (R-O57) — what changes
    /// is only which rungs it will be read against. Explicit so that the one
    /// place the two ladders meet is visible rather than inferred.
    #[inline]
    pub const fn on_scale<T>(self) -> Qty<T> {
        Qty(self.0, core::marker::PhantomData)
    }

    #[inline]
    pub fn min(self, o: Self) -> Self {
        Qty(self.0.min(o.0), core::marker::PhantomData)
    }
    #[inline]
    pub fn max(self, o: Self) -> Self {
        Qty(self.0.max(o.0), core::marker::PhantomData)
    }
    #[inline]
    pub fn clamp(self, lo: Self, hi: Self) -> Self {
        Qty(self.0.clamp(lo.0, hi.0), core::marker::PhantomData)
    }
    #[inline]
    pub fn is_finite(self) -> bool {
        self.0.is_finite()
    }
    #[inline]
    pub fn abs(self) -> Self {
        Qty(self.0.abs(), core::marker::PhantomData)
    }
}

impl<S: Scale> Qty<S> {
    /// Write it as a plain amount. `const` so a config constant can be one.
    #[inline]
    pub const fn new(v: f64) -> Self {
        Qty(v, core::marker::PhantomData)
    }

    /// The magnitude at whole rung `n`, `n` indexed like [`BandTier::index`]
    /// with `Empty = 0`. Saturates at `IV`.
    #[inline]
    pub fn rung(n: usize) -> f64 {
        match n {
            0 => S::BAND_I / S::STEPS[0],
            1 => S::BAND_I,
            2 => S::BAND_I * S::STEPS[1],
            3 => S::BAND_I * S::STEPS[1] * S::STEPS[2],
            _ => S::BAND_I * S::STEPS[1] * S::STEPS[2] * S::STEPS[3],
        }
    }

    /// **Write it as a rung plus a fraction of the way to the next** — the
    /// other half of the "either representation" contract.
    ///
    /// `fraction` is the position *within* the rung, in `[0, 1)`; it is a
    /// position on a log scale, so `0.5` is the geometric midpoint of the
    /// segment, not its arithmetic one.
    #[inline]
    pub fn at(tier: BandTier, fraction: f64) -> Self {
        Self::at_band(Band(tier.index() as f64 + fraction))
    }

    /// Write it as a whole rung.
    #[inline]
    pub fn at_tier(tier: BandTier) -> Self {
        Self::at_band(tier.band())
    }

    /// Write it as a continuous ladder position.
    ///
    /// Exact and **unclamped**: the growth step conserves mass across this map,
    /// so a clamp here is mass appearing or vanishing at the clamp. Positions
    /// off either end extrapolate with the nearest segment's factor.
    #[inline]
    pub fn at_band(b: Band) -> Self {
        let n = if b.0 < 0.0 {
            0.0
        } else if b.0 >= 3.0 {
            3.0
        } else {
            b.0.floor()
        };
        Qty(Self::rung(n as usize) * S::STEPS[n as usize].powf(b.0 - n), core::marker::PhantomData)
    }

    /// **Read it as a ladder position.**
    ///
    /// Floors at [`BAND_FLOOR`]: `band(m)` diverges as `m → 0`, and a
    /// non-finite value in replicated state is a fatal error rather than a
    /// number (design law #16). That is a statement about the *ladder*, not a
    /// claim that such an amount is zero — the amount is still there in the
    /// kilotons, which is why storage is never the reading.
    #[inline]
    pub fn band(self) -> Band {
        if self.0 <= 0.0 {
            return Band(BAND_FLOOR);
        }
        let mut n = 0usize;
        while n < 3 && self.0 >= Self::rung(n + 1) {
            n += 1;
        }
        Band((n as f64 + (self.0 / Self::rung(n)).ln() / S::STEPS[n].ln()).max(BAND_FLOOR))
    }

    /// Read it as the rung it has reached.
    #[inline]
    pub fn tier(self) -> BandTier {
        BandTier::containing(self.band())
    }

    /// Read how far it stands into its rung, in `[0, 1)`.
    #[inline]
    pub fn fraction(self) -> f64 {
        let b = self.band().0;
        b - b.floor()
    }
}

impl<S: Scale> fmt::Display for Qty<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.4} kt ({} +{:.3})", self.0, self.tier(), self.fraction())
    }
}

impl<S> fmt::Debug for Qty<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Qty({})", self.0)
    }
}

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

    /// **How much more stuff `other` is than `self`, as a mass.**
    ///
    /// The only sanctioned way to difference two magnitudes, and it returns
    /// [`Kilotons`] because that is the only unit the answer can be in. A Band
    /// difference is a ratio of exponents and means nothing on its own:
    /// stepping from `Band I` to `Band II` is not "one" of anything, it is
    /// `KT(II) − KT(I)` of stuff.
    ///
    /// Negative when `other` is the smaller — a shrinking population returns
    /// its mass, which is what makes growth conserve (L6).
    #[inline]
    fn gap_to(self, other: impl Measure) -> Kilotons {
        Kilotons::new(other.in_kilotons().kilotons() - self.in_kilotons().kilotons())
    }
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
    /// **Move `rungs` along the ladder** — the one sanctioned way to change a
    /// Band, and deliberately not spelled `+`.
    ///
    /// A Band is a *position* on a logarithmic scale, so moving one rung up
    /// does not add anything: it **multiplies the mass** by that rung's ladder
    /// factor. `Band::new(1.0).up(1.0)` is `Band II`, which is 31.6× the stuff
    /// of `Band I`, not one more of it.
    #[inline]
    pub fn up(self, rungs: f64) -> Band {
        Band(self.0 + rungs)
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
        Kilotons::at_band(self)
    }
}

impl Measure for Kilotons {
    #[inline]
    fn in_bands(self) -> Band {
        self.band()
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

/// **The floor a founding population grows off.**
///
/// The logistic has a fixed point at zero, so a colony seeded at nothing stays
/// at nothing however habitable its world. One tonne is the mass ladder's own
/// bottom rung (`Band Empty`), which makes this the smallest population the
/// design names rather than an arbitrary epsilon — and it is the amount the
/// biosphere is actually charged for, so the bump is paid for like any other
/// growth rather than conjured.
pub const POPULATION_SEED_FLOOR: Kilotons = Kilotons::new(KILOTONS_AT_BAND_EMPTY);

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

/// **Arithmetic is on the kilotons, always.**
///
/// Every logistic, price and conservation check in the engine runs here, on
/// one `f64`, in one instruction — which is the whole reason storage is the
/// mass. Nothing in these operators touches a ladder, so nothing here can cost
/// a `ln` or a `powf`, and two terms written at opposite ends of the contract
/// (one as kilotons, one as a rung) add exactly as if both had been written
/// the same way.
///
/// The scale rides along on the type, so a mass and a price cannot be summed
/// by accident. When they genuinely should be — a recycled hull's minerals
/// becoming a colony's infrastructure — [`Qty::on_scale`] says so out loud.
impl<S> Add for Qty<S> {
    type Output = Qty<S>;
    #[inline]
    fn add(self, o: Self) -> Self {
        Qty(self.0 + o.0, core::marker::PhantomData)
    }
}
impl<S> Sub for Qty<S> {
    type Output = Qty<S>;
    #[inline]
    fn sub(self, o: Self) -> Self {
        Qty(self.0 - o.0, core::marker::PhantomData)
    }
}
impl<S> Neg for Qty<S> {
    type Output = Qty<S>;
    #[inline]
    fn neg(self) -> Self {
        Qty(-self.0, core::marker::PhantomData)
    }
}
impl<S> AddAssign for Qty<S> {
    #[inline]
    fn add_assign(&mut self, o: Self) {
        self.0 += o.0;
    }
}
impl<S> SubAssign for Qty<S> {
    #[inline]
    fn sub_assign(&mut self, o: Self) {
        self.0 -= o.0;
    }
}
impl<S> Mul<f64> for Qty<S> {
    type Output = Qty<S>;
    #[inline]
    fn mul(self, k: f64) -> Self {
        Qty(self.0 * k, core::marker::PhantomData)
    }
}
impl<S> Mul<Qty<S>> for f64 {
    type Output = Qty<S>;
    #[inline]
    fn mul(self, q: Qty<S>) -> Qty<S> {
        Qty(self * q.0, core::marker::PhantomData)
    }
}
impl<S> Div<f64> for Qty<S> {
    type Output = Qty<S>;
    #[inline]
    fn div(self, k: f64) -> Self {
        Qty(self.0 / k, core::marker::PhantomData)
    }
}
/// A ratio of two amounts on the same ladder is a pure number — how many of
/// one the other is, which is the only sanctioned way out of the type.
impl<S> Div<Qty<S>> for Qty<S> {
    type Output = f64;
    #[inline]
    fn div(self, o: Self) -> f64 {
        self.0 / o.0
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
arith!(Length);
arith!(Area);
arith!(Volume);

/// **`Band` has no `Add`, `Sub`, or `Mul` — deliberately, and this is the point
/// of the type.**
///
/// `I + II + III + IV` is not `X`. Band numerals are positions on a
/// multiplicative ladder, so summing or differencing them produces a number
/// with no meaning: the "gap" between two Bands is a *mass*, and masses are
/// what [`Kilotons`] is for. R-O66 was one instance of this (a `min` across a
/// mass and two levels); an infrastructure ladder priced at `b + 1` minerals
/// per rung was another, and a colony-ship budget summed as `b(b+1)/2` was a
/// third, made *after* R-O66 landed and by someone who had read it.
///
/// So the arithmetic is gone from the type. To change a Band, move along the
/// ladder with [`Band::up`]. To ask how much more *stuff* one Band is than
/// another, take the difference in kilotons — [`Measure::gap_to`] does it in
/// one call and names the unit in its return type.
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

#[cfg(test)]
mod tests {
    use super::*;

    /// **One metric tonne is the contract.** Every assertion in this module
    /// that compares two amounts uses it, and it is an *absolute* tolerance on
    /// the kilotons rather than a relative one — a relative bound would be
    /// vacuously easy at `Band IV`, where one tonne is 1.4e-9 of the value,
    /// and vacuously hard at the floor.
    const ONE_TONNE: f64 = 0.001;

    /// A spread of positions covering every segment, both sentinel ends, and
    /// the joins — the places a piecewise ladder can develop a step.
    const POSITIONS: [f64; 17] =
        [-0.5, 0.0, 0.25, 0.5, 0.75, 0.999, 1.0, 1.5, 1.999, 2.0, 2.5, 3.0, 3.5, 3.999, 4.0, 4.5, 5.0];

    fn close(a: f64, b: f64, what: &str) {
        assert!((a - b).abs() <= ONE_TONNE, "{what}: {a} vs {b} differs by more than a tonne");
    }

    /// **The two ways of writing an amount are the same amount** — the whole
    /// contract of the type, and the reason arithmetic never has to ask which
    /// end a term came from.
    ///
    /// Checked on both ladders, because the ladder rides on the type and a
    /// scale that got its rungs wrong would still round-trip against itself.
    #[test]
    fn either_representation_writes_the_same_amount() {
        fn check<S: Scale>() {
            for &b in &POSITIONS {
                let by_band = Qty::<S>::at_band(Band::new(b));
                let by_kt = Qty::<S>::new(by_band.kilotons());
                assert_eq!(by_band, by_kt, "{}: writing the kilotons back must be the same bits", S::NAME);

                // ...and the rung-plus-fraction form is the same again, for
                // every position the ladder actually names.
                if (0.0..5.0).contains(&b) {
                    let split = Qty::<S>::at(by_band.tier(), by_band.fraction());
                    close(split.kilotons(), by_band.kilotons(), S::NAME);
                }
            }
        }
        check::<Mass>();
        check::<Cost>();
    }

    /// The ladder is invertible to within a tonne **at or above its floor**,
    /// and deliberately not below it — which is the sharper half of the claim.
    ///
    /// `band(m)` diverges as `m → 0`, so the reading clamps at [`BAND_FLOOR`]
    /// (design law #16). An amount beneath the floor therefore reads the floor
    /// and would *write back larger than it is*. That is exactly why storage is
    /// the mass and never the reading: the kilotons of such an amount are
    /// untouched and exact, and only the shorthand runs out of rungs.
    ///
    /// This is checked on both ladders because the floors sit at different
    /// masses — 0.001 kt and 0.02 minerals — and the mass ladder happens to
    /// squeak inside a tonne at its floor while the cost ladder does not. A
    /// test that only ran on `Mass` would have called the clamp a round trip.
    #[test]
    fn the_ladder_round_trips_above_its_floor_and_clamps_below_it() {
        fn check<S: Scale>() {
            for &b in &POSITIONS {
                let q = Qty::<S>::at_band(Band::new(b));
                if b >= BAND_FLOOR {
                    close(Qty::<S>::at_band(q.band()).kilotons(), q.kilotons(), S::NAME);
                } else {
                    assert_eq!(q.band(), Band::new(BAND_FLOOR), "{}: below the floor must read the floor", S::NAME);
                    assert!(q.kilotons() < Qty::<S>::rung(0), "{}: and still hold its true amount", S::NAME);
                }
            }
            // And from the other end: an amount written as kilotons reads a
            // position that writes the same amount back, once it is on the
            // ladder at all.
            let floor = Qty::<S>::rung(0);
            for k in [1.0, 2.7, 50.0, 316.0, 2_828.0, 100_000.0, 715_541.0] {
                let m = floor * k;
                close(Qty::<S>::at_band(Qty::<S>::new(m).band()).kilotons(), m, S::NAME);
            }
        }
        check::<Mass>();
        check::<Cost>();
    }

    /// **Every operator agrees with the bare `f64` it is standing in for**, and
    /// agrees whichever way each operand was written. This is the "commutes
    /// regardless of representation" requirement made checkable: the left
    /// operand is written as a rung, the right as kilotons, and the answer must
    /// match doing it in plain `f64` throughout.
    #[test]
    fn operators_match_bare_f64_across_both_representations() {
        for &ba in &POSITIONS {
            for &bb in &POSITIONS {
                let a = Kilotons::at_band(Band::new(ba));
                let b_kt = Kilotons::at_band(Band::new(bb)).kilotons();
                let b = Kilotons::new(b_kt);
                let (x, y) = (a.kilotons(), b_kt);

                close((a + b).kilotons(), x + y, "add");
                close((b + a).kilotons(), x + y, "add commutes");
                close((a - b).kilotons(), x - y, "sub");
                close((-a).kilotons(), -x, "neg");
                close((a * 3.5).kilotons(), x * 3.5, "scale");
                close((3.5 * a).kilotons(), x * 3.5, "scale commutes");
                close((a / 3.5).kilotons(), x / 3.5, "divide by a scalar");
                close(a.min(b).kilotons(), x.min(y), "min");
                close(a.max(b).kilotons(), x.max(y), "max");
                assert_eq!(a > b, x > y, "ordering must be the ordering of the amounts");

                if y != 0.0 {
                    let ratio = a / b;
                    assert!((ratio - x / y).abs() <= 1e-9 * (x / y).abs().max(1.0), "ratio: {ratio} vs {}", x / y);
                }

                let mut acc = a;
                acc += b;
                close(acc.kilotons(), x + y, "add-assign");
                acc -= b;
                close(acc.kilotons(), x, "sub-assign round trips");
            }
        }
    }

    /// A long accumulation must not drift. Two hundred thousand alternating
    /// deposits and withdrawals, half written as rungs and half as kilotons,
    /// must land back within a tonne of where they started — which is the
    /// conservation claim (L6) stated as an arithmetic one.
    #[test]
    fn a_long_mixed_accumulation_does_not_drift() {
        let mut acc = Kilotons::at_tier(BandTier::II);
        let start = acc;
        for i in 0..100_000 {
            let step = if i % 2 == 0 {
                Kilotons::at(BandTier::Empty, (i % 997) as f64 / 997.0)
            } else {
                Kilotons::new(0.0173 * ((i % 31) as f64))
            };
            acc += step;
            acc -= step;
        }
        close(acc.kilotons(), start.kilotons(), "a round trip per iteration must not drift");
    }

    /// **The wrapper is the `f64`.** Layout parity is the exact form of the
    /// O(1) claim, and it is worth more than a timing run: `#[repr(transparent)]`
    /// over one `f64` with a zero-sized marker means a `Qty` is passed in a
    /// register, stored in a slice with the same stride, and auto-vectorised by
    /// the same codegen. A timing test can be noisy; this cannot.
    #[test]
    fn a_quantity_is_laid_out_exactly_as_the_f64_it_wraps() {
        use core::mem::{align_of, size_of};
        assert_eq!(size_of::<Kilotons>(), size_of::<f64>());
        assert_eq!(align_of::<Kilotons>(), align_of::<f64>());
        assert_eq!(size_of::<Qty<Cost>>(), size_of::<f64>());
        // The ladder tag is carried by the type, so it costs no space at all —
        // which is what makes "the type carries which ladder" affordable.
        assert_eq!(size_of::<Mass>(), 0);
        assert_eq!(size_of::<Cost>(), 0);
        // And a slice of them has the stride of a slice of `f64`, which is what
        // vectorisation actually needs.
        assert_eq!(size_of::<[Kilotons; 8]>(), size_of::<[f64; 8]>());
    }

    /// **Arithmetic runs at bare-`f64` speed**, measured rather than asserted.
    ///
    /// Sized to a fraction of a second so it fits the 60-second rule; the
    /// five-second version is `examples/qty_bench`. The bound is deliberately
    /// loose — this is a regression guard against something turning an operator
    /// into a `ln`, not a microbenchmark — and it only *reports* on a run that
    /// looks like it was descheduled rather than failing the suite for it.
    #[test]
    fn arithmetic_costs_what_an_f64_costs() {
        const N: usize = 4_000_000;
        let (raw, wrapped) = bench_pair(N);
        let ratio = wrapped / raw;
        println!("qty/f64 = {ratio:.3}  (raw {raw:.4}s, wrapped {wrapped:.4}s)");
        assert!(
            ratio < 4.0,
            "wrapping an f64 must not change the cost of arithmetic: {ratio:.2}x \
             ({raw:.4}s raw vs {wrapped:.4}s wrapped). A ratio this large means an \
             operator started converting — a `ln` or a `powf` on a path that should \
             only touch the stored kilotons."
        );
    }

    /// The two loops the benchmark times, kept identical line for line so the
    /// only difference is the type. Returns `(bare f64 seconds, Qty seconds)`.
    pub(crate) fn bench_pair(n: usize) -> (f64, f64) {
        use std::time::Instant;

        let t = Instant::now();
        let mut a = 1.0f64;
        for i in 0..n {
            let step = 0.5 + (i % 17) as f64;
            a = ((a + step) * 0.999 - step * 0.5).clamp(-1e12, 1e12);
        }
        let raw = t.elapsed().as_secs_f64();

        let t = Instant::now();
        let mut b = Kilotons::new(1.0);
        for i in 0..n {
            let step = Kilotons::new(0.5 + (i % 17) as f64);
            b = ((b + step) * 0.999 - step * 0.5).clamp(Kilotons::new(-1e12), Kilotons::new(1e12));
        }
        let wrapped = t.elapsed().as_secs_f64();

        // Consume both so neither loop can be optimised away entirely.
        assert!((a - b.kilotons()).abs() < 1e-6, "the two loops must compute the same thing");
        (raw, wrapped)
    }

    /// **The two ladders are genuinely different, and the type is what keeps
    /// them apart.** This is the incompatibility the scan turned up, pinned so
    /// it cannot be quietly "simplified" back into one ladder: the same number
    /// of kilotons reads a different rung depending on which ladder it is on,
    /// because cost tracks area and the hold tracks volume.
    #[test]
    fn the_same_amount_reads_a_different_rung_on_each_ladder() {
        // One General hull: 1.0 kt of minerals, which is its dry mass too
        // (R-O57). Cost `Band II` by ratification; mass `Band I` by anchor.
        let general = 1.0;
        assert_eq!(Qty::<Cost>::new(general).tier(), BandTier::II);
        assert_eq!(Kilotons::new(general).tier(), BandTier::I);
        // A Medium hull, the cost ladder's own anchor.
        assert_eq!(Qty::<Cost>::new(0.1).tier(), BandTier::I);
        assert!(Kilotons::new(0.1).band() < BandTier::I.band());
        // Crossing is explicit and costs nothing — same bits, new rungs.
        let price = Qty::<Cost>::new(general);
        assert_eq!(price.on_scale::<Mass>().kilotons(), price.kilotons());
    }

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
        //
        // **It is a claim about the playable ladder, `I → II → III → IV`.**
        // `MASS_LADDER[0]` is not a rung factor at all — it is the width of
        // `Band Empty`, the ladder's floor, which R-MC15 does not fix and which
        // is set independently per quantity (§2.6). See
        // `KILOTONS_AT_BAND_EMPTY`.
        for pair in MASS_LADDER[1..].windows(2) {
            let ratio = pair[1] / pair[0];
            assert!(ratio > 1.0 && ratio < 10.0, "{} / {} = {ratio}, outside (1, 10)", pair[1], pair[0]);
        }

        // And the tie to the cost ladder is the shell model's exponent: cost
        // tracks r², the hold tracks r³. A cost ladder of 10, 20, 40 across the
        // playable rungs — the floor is excluded for the same reason.
        for (n, cost_step) in [10.0_f64, 20.0, 40.0].into_iter().enumerate() {
            assert!(
                (MASS_LADDER[n + 1] - cost_step.powf(1.5)).abs() < 1e-9,
                "F_mass must be F_cost^(3/2) at rung {}",
                n + 1
            );
        }
    }

    /// `Band Empty` is the ladder's floor and its width is set on its own
    /// (`KILOTONS_AT_BAND_EMPTY`), so pin both halves of that: where the floor
    /// is, and that widening it left `Band I` exactly where it was. The failure
    /// this guards against is re-anchoring — moving the bottom of the ladder and
    /// dragging every rung above it along, which would silently rescale every
    /// mass in the engine.
    #[test]
    fn widening_band_empty_does_not_move_band_i() {
        assert_eq!(rung_mass(0), 0.001, "Band Empty is one metric tonne");
        assert_eq!(rung_mass(1), KILOTONS_AT_BAND_I, "Band I must not move when the floor widens");
        for n in 1..=4 {
            let expect = match n {
                1 => 1.0,
                2 => 31.622_776_601_683_793,
                3 => 31.622_776_601_683_793 * 89.442_719_099_991_59,
                _ => 31.622_776_601_683_793 * 89.442_719_099_991_59 * 252.982_212_813_470_36,
            };
            assert!((rung_mass(n) - expect).abs() < 1e-9 * expect.max(1.0), "rung {n} moved: {}", rung_mass(n));
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
