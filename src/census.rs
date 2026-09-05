//! **Binding-site census — where the simulation's tradeoffs actually bind.**
//!
//! Every tradeoff in this engine is written as one of two code shapes: a clamp
//! (`a.min(b)`, `a.max(b)`) or a threshold (`x >= k`). Both are *supposed* to
//! trade one quantity against another. Both stop doing so the moment one side
//! wins every time — and when that happens the losing side's knob has **exactly
//! zero derivative** on the objective, because the objective never sees it.
//!
//! That is the mechanism behind this project's recurring artifact (CLAUDE.md
//! §2, "the artifact pattern"): a plausible number produced by a quantity the
//! shipped configuration never exercises. Three of the recorded cases are
//! saturated sites, and each was found by hand, months apart:
//!
//! | recorded finding | the site that was saturated |
//! |---|---|
//! | `cargo_unit_size` "flat — inert here" | `load = cap.min(avail)`; `avail` always wins |
//! | freighter flew empty round trips forever | its stand-down predicate was never true |
//! | coverage bound by `k_high`, not the economy | the class gate admits only ~51% |
//!
//! A saturated site is cheap to *detect*: count which side bound. This module
//! is those counters. It records nothing the simulation ever reads back, so it
//! cannot affect results — [`Simulation`](crate::sim::Simulation) exposes it
//! behind an explicit enable, and it is off by default like the event log.
//!
//! **What a census cannot tell you.** That a site is saturated is a fact; that
//! it is a *bug* is a judgement. `k_liebig` binding on `infra` early is the
//! design working — infrastructure is meant to be the early constraint. The
//! census narrows a large search to a handful of candidates; it does not
//! adjudicate them.

/// One instrumented tradeoff. Each site records which of its sides bound.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Site {
    /// `K = min(hab, bio, infra)` — the Liebig capacity. Sides: hab / bio / infra.
    KLiebig,
    /// `k_potential = min(hab, bio)`, the ceiling infra may be built to.
    KPotential,
    /// `load = cap.min(avail)` — a freighter's hold against what the rock has.
    FreighterLoad,
    /// Population growth against standing biomass: `delta.min(bio)`.
    GrowthBiomass,
    /// `mineral_pressure` — does the clamp sit at an endpoint or in the interior?
    MineralPressure,
    /// What `rank` called this world: Barren / Mining / Colony / ProductionCenter.
    RankClass,
    /// The production tier gate: is this center below the medium tier?
    ProductionTier,
    /// `infra < k_potential` — is there headroom left to deepen into?
    DeepenHeadroom,
    /// `candidate_count < survey_reserve` — is the empire short of frontier?
    SurveyReserve,
}

impl Site {
    pub const ALL: [Site; 9] = [
        Site::KLiebig,
        Site::KPotential,
        Site::FreighterLoad,
        Site::GrowthBiomass,
        Site::MineralPressure,
        Site::RankClass,
        Site::ProductionTier,
        Site::DeepenHeadroom,
        Site::SurveyReserve,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Site::KLiebig => "K = min(hab, bio, infra)",
            Site::KPotential => "k_potential = min(hab, bio)",
            Site::FreighterLoad => "freighter load = cap.min(avail)",
            Site::GrowthBiomass => "growth capped by biomass",
            Site::MineralPressure => "mineral_pressure clamp",
            Site::RankClass => "rank classification",
            Site::ProductionTier => "production tier gate",
            Site::DeepenHeadroom => "deepen headroom (infra < k_pot)",
            Site::SurveyReserve => "survey reserve gate",
        }
    }

    /// Names of the sides, in slot order. A site is *saturated* when one of
    /// these takes essentially all the traffic.
    pub fn sides(self) -> &'static [&'static str] {
        match self {
            Site::KLiebig => &["hab", "bio", "infra"],
            Site::KPotential => &["hab", "bio"],
            Site::FreighterLoad => &["cap (the hold binds)", "avail (the rock binds)"],
            Site::GrowthBiomass => &["demand (K binds)", "biomass binds"],
            Site::MineralPressure => &["at 0 (rich)", "interior", "at 1 (broke)"],
            Site::RankClass => &["Barren", "MiningOutpost", "Colony", "ProductionCenter"],
            Site::ProductionTier => &["below medium tier", "at/above"],
            Site::DeepenHeadroom => &["headroom left", "capped"],
            Site::SurveyReserve => &["wants survey", "frontier sufficient"],
        }
    }

    fn index(self) -> usize {
        Site::ALL.iter().position(|s| *s == self).unwrap()
    }
}

/// Width of the widest site. Sites with fewer sides leave the tail at zero.
const MAX_SIDES: usize = 4;

/// Write-only counters. Nothing in the simulation reads these back, so enabling
/// the census cannot change a run — `census_does_not_perturb_the_simulation`
/// pins that.
#[derive(Clone, Debug)]
pub struct BindingCensus {
    enabled: bool,
    counts: [[u64; MAX_SIDES]; Site::ALL.len()],
}

impl Default for BindingCensus {
    fn default() -> Self {
        BindingCensus { enabled: false, counts: [[0; MAX_SIDES]; Site::ALL.len()] }
    }
}

impl BindingCensus {
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    #[inline]
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Record that `side` bound at `site`. Inlined and branch-predictable; when
    /// the census is off this is one test against a `bool` that is false for the
    /// whole run.
    #[inline]
    pub fn record(&mut self, site: Site, side: usize) {
        if !self.enabled {
            return;
        }
        if side < MAX_SIDES {
            self.counts[site.index()][side] += 1;
        }
    }

    /// Convenience for a 3-way `min`: records which argument was smallest.
    /// Ties go to the earlier slot, which matches how `f64::min` chains resolve.
    #[inline]
    pub fn record_min3(&mut self, site: Site, a: f64, b: f64, c: f64) {
        if !self.enabled {
            return;
        }
        let side = if a <= b && a <= c {
            0
        } else if b <= c {
            1
        } else {
            2
        };
        self.record(site, side);
    }

    #[inline]
    pub fn record_min2(&mut self, site: Site, a: f64, b: f64) {
        if !self.enabled {
            return;
        }
        self.record(site, if a <= b { 0 } else { 1 });
    }

    pub fn total(&self, site: Site) -> u64 {
        self.counts[site.index()].iter().sum()
    }

    pub fn count(&self, site: Site, side: usize) -> u64 {
        self.counts[site.index()][side]
    }

    /// Fraction of traffic taken by `side`, or `None` if the site never fired.
    pub fn share(&self, site: Site, side: usize) -> Option<f64> {
        let total = self.total(site);
        if total == 0 {
            return None;
        }
        Some(self.count(site, side) as f64 / total as f64)
    }

    /// **The verdict.** A site is saturated when one side takes at least
    /// `threshold` of the traffic — meaning every other side is, at this
    /// operating point, a quantity the objective cannot see.
    ///
    /// Returns the dominant side's index and share.
    pub fn saturated(&self, site: Site, threshold: f64) -> Option<(usize, f64)> {
        let total = self.total(site);
        if total == 0 {
            return None;
        }
        let n = site.sides().len();
        (0..n)
            .map(|i| (i, self.count(site, i) as f64 / total as f64))
            .filter(|(_, share)| *share >= threshold)
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
    }

    /// A site that never fired at all — the strongest possible finding, because
    /// it means the branch is dead rather than merely one-sided.
    pub fn never_fired(&self, site: Site) -> bool {
        self.total(site) == 0
    }
}
